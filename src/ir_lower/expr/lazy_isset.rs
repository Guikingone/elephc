//! Purpose:
//! Lazy isset and empty lowering for magic-property semantics.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers `isset()` as a lazy language construct instead of an eager builtin call.
/// Reports whether an operand is a pure property/dim WALK, with no call anywhere inside it.
///
/// PHP fixes the quiet fetch at COMPILE time: the mode propagates down property and dim fetch
/// nodes and is never applied to a call, so `f($o) ?? 'D'` and `$o->getT() ?? 'D'` raise where
/// `$o->t ?? 'D'` answers. Deciding it on the shape of the operand is therefore not an
/// approximation of PHP's rule, it IS the rule -- and it means the compiled side needs no
/// barrier at call sites, because an operand containing a call is simply never wrapped.
///
/// Deliberately conservative: an index expression is walked too, so `isset($o->a[f()])` is left
/// alone rather than run with the mode held across `f()`. Refusing to go quiet is always safe;
/// going quiet where PHP would not is what silently swallows a real error.
pub(super) fn expr_is_quiet_fetch_chain(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Variable(_) | ExprKind::This => true,
        ExprKind::PropertyAccess { object, .. } => expr_is_quiet_fetch_chain(object),
        ExprKind::StaticPropertyAccess { .. } => true,
        ExprKind::ArrayAccess { array, index } => {
            expr_is_quiet_fetch_chain(array) && expr_is_quiet_fetch_chain(index)
        }
        ExprKind::IntLiteral(_) | ExprKind::StringLiteral(_) | ExprKind::BoolLiteral(_) => true,
        _ => false,
    }
}

/// Emits one side of the compiled quiet-fetch scope.
pub(in crate::ir_lower) fn emit_quiet_property_fetch(ctx: &mut LoweringContext<'_, '_>, enter: bool, span: Span) {
    ctx.emit_void(
        Op::RuntimeCall,
        Vec::new(),
        Some(Immediate::RuntimeCall(
            crate::ir::RuntimeCallTarget::EvalQuietPropertyFetch { enter },
        )),
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers one `isset`/`empty`/`??` operand inside PHP's quiet property fetch.
///
/// Compiled code reaching an EVAL-OWNED object's property goes through the bridge, which raises
/// for an uninitialized typed property unless the mode is on; the interpreter sets the mode
/// around its own operands and this is the same door for the compiled side, sharing one
/// thread-local depth so a chain that crosses between them sees one mode. Without it,
/// `empty($this->p[$k])` compiled over an eval-declared class raised where the identical source
/// interpreted answered -- the Symfony stop, `CheckCircularReferencesPass::$checkedLazyNodes`.
///
/// Only a pure property/dim walk is wrapped, which is what keeps "the mode does not cross a
/// call" true without a barrier at every compiled call site.
pub(super) fn lower_operand_in_quiet_property_fetch<F>(
    ctx: &mut LoweringContext<'_, '_>,
    operand: &Expr,
    lower_operand: F,
) -> LoweredValue
where
    F: FnOnce(&mut LoweringContext<'_, '_>) -> LoweredValue,
{
    if !expr_is_quiet_fetch_chain(operand) {
        return lower_operand(ctx);
    }
    emit_quiet_property_fetch(ctx, true, operand.span);
    let value = lower_operand(ctx);
    emit_quiet_property_fetch(ctx, false, operand.span);
    value
}

pub(super) fn lower_lazy_isset(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "isset" {
        return None;
    }
    if crate::types::call_args::has_named_args(args) || args.iter().any(is_spread_arg) {
        return None;
    }
    if args.is_empty() {
        return Some(lower_bool_literal(ctx, false, expr));
    }

    let temp_name = ctx.declare_hidden_temp(PhpType::Bool);
    let false_block = ctx.builder.create_named_block("isset.lazy_false", Vec::new());
    let merge = ctx.builder.create_named_block("isset.lazy_merge", Vec::new());
    for (idx, arg) in args.iter().enumerate() {
        let checked = lower_operand_in_quiet_property_fetch(ctx, arg, |ctx| {
        lower_lazy_isset_operand(ctx, arg).unwrap_or_else(|| {
            // `isset()` never emits undefined-offset warnings, so eager array
            // operands must be lowered with the silent read variants.
            let value = if let ExprKind::ArrayAccess { array, index } = &arg.kind {
                lower_array_access_with_missing_warning(ctx, array, index, arg, false)
            } else {
                lower_expr(ctx, arg)
            };
            emit_builtin_call_value(ctx, name, vec![value.value], PhpType::Int, arg.span, None)
        })
        });
        let then_target = if idx + 1 == args.len() {
            ctx.builder.create_named_block("isset.lazy_true", Vec::new())
        } else {
            ctx.builder.create_named_block("isset.lazy_next", Vec::new())
        };
        ctx.builder.terminate(Terminator::CondBr {
            cond: checked.value,
            then_target,
            then_args: Vec::new(),
            else_target: false_block,
            else_args: Vec::new(),
        });
        ctx.builder.position_at_end(then_target);
    }

    let true_value = lower_bool_literal(ctx, true, expr);
    store_value_into_temp(ctx, &temp_name, PhpType::Bool, true_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(false_block);
    let false_value = lower_bool_literal(ctx, false, expr);
    store_value_into_temp(ctx, &temp_name, PhpType::Bool, false_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    Some(take_owned_temp(ctx, &temp_name, expr.span))
}

/// Lowers a single `isset()` operand that has special lazy PHP semantics.
pub(super) fn lower_lazy_isset_operand(
    ctx: &mut LoweringContext<'_, '_>,
    arg: &Expr,
) -> Option<LoweredValue> {
    match &arg.kind {
        // No syntactic gate on the receiver. A pre-lowering guess types every user call `int`
        // (`infer_expr_type_syntactic`), so `isset(f()['k'])` used to be refused here and fall
        // through to the read-then-test path in `lower_lazy_isset`. That path RELEASES the
        // call's temporary array as soon as the element is read, and the backend's `isset`
        // lowering then re-probes the read's producer (`source_instruction`), calling
        // `__rt_hash_get` on storage already freed — a present key answered `false`. Symfony's
        // `isset(static::getProvidedTypes()[$prefix])` is exactly that shape, and it took down
        // every env-var placeholder in the prod container.
        //
        // `lower_native_isset_offset_probe` dispatches on the receiver's REAL lowered IR type
        // instead of on the shape of its expression, and its final arm is the same
        // read-then-test for receivers no native probe fits — with the receiver evaluated once
        // and kept, so nothing re-reads it after the release.
        ExprKind::ArrayAccess { array, index } => {
            if array_access_expr_satisfies_array_access(ctx, array) {
                return Some(lower_array_access_offset_exists(ctx, array, index, arg));
            }
            Some(lower_native_isset_offset_probe(ctx, array, index, arg))
        }
        ExprKind::PropertyAccess { object, property }
        | ExprKind::NullsafePropertyAccess { object, property } => {
            lower_lazy_property_isset_operand(ctx, object, property, arg)
        }
        // A typed static property starts uninitialized and `isset()` must answer false there
        // rather than take the ordinary read, whose backend guard is fatal.
        ExprKind::StaticPropertyAccess { receiver, property }
            if static_property_can_be_uninitialized(ctx, receiver, property) =>
        {
            Some(lower_initialized_static_property_isset(
                ctx, receiver, property, arg,
            ))
        }
        // `isset($this)` inside a static closure always evaluates to `false`
        // because static closures have no `$this` binding. PHP allows this
        // probe and returns false; elephc must not try to load a missing slot.
        ExprKind::This if !ctx.local_slots.contains_key("this") => {
            Some(lower_bool_literal(ctx, false, arg))
        }
        _ => None,
    }
}

/// Lowers `isset($arrayAccess[$key])`, returning false without a call for a null receiver.
fn lower_array_access_offset_exists(
    ctx: &mut LoweringContext<'_, '_>,
    array: &Expr,
    index: &Expr,
    arg: &Expr,
) -> LoweredValue {
    let receiver = lower_expr(ctx, array);
    let args = vec![index.clone()];
    if !value_is_nullable(ctx, receiver.value) {
        return lower_method_call_with_receiver(
            ctx,
            receiver,
            "offsetExists",
            &args,
            Op::MethodCall,
            arg,
        );
    }

    let is_null = ctx.emit_value(
        Op::IsNull,
        vec![receiver.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        Some(arg.span),
    );
    let temp_name = ctx.declare_hidden_temp(PhpType::Bool);
    let null_block = ctx
        .builder
        .create_named_block("isset.array_access.null", Vec::new());
    let call_block = ctx
        .builder
        .create_named_block("isset.array_access.call", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("isset.array_access.merge", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: null_block,
        then_args: Vec::new(),
        else_target: call_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(null_block);
    let false_value = emit_bool_literal(ctx, false, Some(arg.span));
    store_value_into_temp(ctx, &temp_name, PhpType::Bool, false_value, arg.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(call_block);
    let exists = lower_method_call_with_receiver(
        ctx,
        receiver,
        "offsetExists",
        &args,
        Op::MethodCall,
        arg,
    );
    store_value_into_temp(ctx, &temp_name, PhpType::Bool, exists, arg.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    take_owned_temp(ctx, &temp_name, arg.span)
}

/// Lowers `empty($obj->magicProp)` with PHP's overloaded-property semantics:
/// `empty` consults `__isset` first and only evaluates `__get` when `__isset`
/// is truthy, so an unset virtual property is empty without ever reading it.
/// Returns `None` for operands the eager `empty` builtin already handles (plain
/// variables, declared properties, array elements), letting that path run.
pub(super) fn lower_lazy_empty(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "empty" {
        return None;
    }
    if args.len() != 1
        || crate::types::call_args::has_named_args(args)
        || args.iter().any(is_spread_arg)
    {
        return None;
    }
    if let ExprKind::ArrayAccess { array, index } = &args[0].kind {
        // `empty($this->p[$k])` is the Symfony stop's exact shape, and its property read is one
        // level BELOW the operand root, so the mode has to wrap the whole walk rather than the
        // outermost node.
        let value = lower_operand_in_quiet_property_fetch(ctx, &args[0], |ctx| {
            lower_array_access_with_missing_warning(ctx, array, index, &args[0], false)
        });
        return Some(emit_builtin_call_value(
            ctx,
            name,
            vec![value.value],
            PhpType::Bool,
            expr.span,
            None,
        ));
    }
    // A typed static property that is still uninitialized is EMPTY in PHP, and reading it the
    // ordinary way to find that out is fatal — the same reason `isset()` and `??` need the
    // slot probe. Uninitialized answers true without the read.
    if let ExprKind::StaticPropertyAccess { receiver, property } = &args[0].kind {
        if static_property_can_be_uninitialized(ctx, receiver, property) {
            return Some(lower_initialized_static_property_empty(
                ctx, receiver, property, name, &args[0],
            ));
        }
    }
    // The instance twin of the static arm above: an uninitialized typed slot is EMPTY, and the
    // ordinary read that would say so raises. `property_isset_action` already decides whether the
    // slot is a declared one worth probing — the same decision `isset()` makes, reused rather than
    // re-derived so the two constructs cannot drift on which properties they consider declared.
    if let ExprKind::PropertyAccess { object, property } = &args[0].kind {
        if matches!(
            property_isset_action(ctx, object, property),
            Some(IssetPropertyAction::Initialized)
        ) {
            let object = lower_expr(ctx, object);
            return Some(lower_initialized_property_empty(
                ctx, object, property, name, &args[0],
            ));
        }
    }
    let (exists_call, get_call) = lazy_empty_magic_property_calls(ctx, &args[0])?;

    let temp_name = ctx.declare_hidden_temp(PhpType::Bool);
    let present_block = ctx.builder.create_named_block("empty.present", Vec::new());
    let absent_block = ctx.builder.create_named_block("empty.absent", Vec::new());
    let merge = ctx.builder.create_named_block("empty.merge", Vec::new());

    // `__isset(prop)` decides whether the property is considered set at all.
    let exists = lower_expr(ctx, &exists_call);
    ctx.builder.terminate(Terminator::CondBr {
        cond: exists.value,
        then_target: present_block,
        then_args: Vec::new(),
        else_target: absent_block,
        else_args: Vec::new(),
    });

    // Set: empty is the emptiness of the `__get` value (reuses the eager builtin).
    ctx.builder.position_at_end(present_block);
    let get_value = lower_expr(ctx, &get_call);
    let empty_name = ctx.intern_function_name(name);
    let empty_value = ctx.emit_value(
        Op::LanguageConstructCall,
        vec![get_value.value],
        Some(Immediate::Data(empty_name)),
        PhpType::Bool,
        effects_lookup::language_construct_effects(name),
        Some(expr.span),
    );
    store_value_into_temp(ctx, &temp_name, PhpType::Bool, empty_value, expr.span);
    branch_to(ctx, merge);

    // Not set: empty is true and `__get` is never called.
    ctx.builder.position_at_end(absent_block);
    let true_value = lower_bool_literal(ctx, true, expr);
    store_value_into_temp(ctx, &temp_name, PhpType::Bool, true_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    Some(ctx.load_local(&temp_name, Some(expr.span)))
}

/// For an `empty()` operand that is an overloaded (magic) property access,
/// returns the `(__isset, __get)` synthetic call expressions PHP would evaluate.
/// The property name is a string literal, so reusing it for both calls is
/// side-effect free. Returns `None` for any other operand shape.
pub(super) fn lazy_empty_magic_property_calls(
    ctx: &LoweringContext<'_, '_>,
    arg: &Expr,
) -> Option<(Expr, Expr)> {
    match &arg.kind {
        ExprKind::PropertyAccess { object, property } => {
            property_existence_magic_class(ctx, object, property, "__isset")?;
            let key = Expr::new(ExprKind::StringLiteral(property.clone()), arg.span);
            let exists = Expr::new(
                ExprKind::MethodCall {
                    object: object.clone(),
                    method: "__isset".to_string(),
                    args: vec![key.clone()],
                },
                arg.span,
            );
            let get = Expr::new(
                ExprKind::MethodCall {
                    object: object.clone(),
                    method: "__get".to_string(),
                    args: vec![key],
                },
                arg.span,
            );
            Some((exists, get))
        }
        ExprKind::NullsafePropertyAccess { object, property } => {
            property_existence_magic_class(ctx, object, property, "__isset")?;
            let key = Expr::new(ExprKind::StringLiteral(property.clone()), arg.span);
            let exists = Expr::new(
                ExprKind::NullsafeMethodCall {
                    object: object.clone(),
                    method: "__isset".to_string(),
                    args: vec![key.clone()],
                },
                arg.span,
            );
            let get = Expr::new(
                ExprKind::NullsafeMethodCall {
                    object: object.clone(),
                    method: "__get".to_string(),
                    args: vec![key],
                },
                arg.span,
            );
            Some((exists, get))
        }
        _ => None,
    }
}

/// Returns the class whose `magic` method (`__isset`/`__unset`) should handle
/// property existence/removal: a property that cannot be accessed normally on an
/// object whose class declares the magic method.
pub(super) fn property_existence_magic_class(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    magic: &str,
) -> Option<String> {
    let class_name = instance_callable_object_class(ctx, object)?;
    let class_info = ctx.classes.get(&class_name)?;
    if property_is_accessible_for_ir(ctx, &class_name, class_info, property) {
        return None;
    }
    class_method_signature(ctx, &class_name, &php_symbol_key(magic)).map(|_| class_name)
}
