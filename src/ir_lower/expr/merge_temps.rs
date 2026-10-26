//! Purpose:
//! Branch merge temporaries, container widening, and fallback typing.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Stores a lowered expression result into a hidden merge temporary.
pub(super) fn store_expr_into_temp(
    ctx: &mut LoweringContext<'_, '_>,
    temp_name: &str,
    temp_type: PhpType,
    expr: &Expr,
    span: crate::span::Span,
) {
    let value = lower_expr(ctx, expr);
    store_value_into_temp(ctx, temp_name, temp_type, value, span);
}

/// Stores an already lowered value into a hidden merge temporary.
pub(in crate::ir_lower) fn store_value_into_temp(
    ctx: &mut LoweringContext<'_, '_>,
    temp_name: &str,
    temp_type: PhpType,
    value: LoweredValue,
    span: crate::span::Span,
) {
    let value = coerce_value_for_temp(ctx, value, &temp_type, span);
    let source = value;
    let stored = crate::ir_lower::ownership::acquire_if_refcounted(ctx, value, Some(span));
    ctx.store_local(temp_name, stored, temp_type, Some(span));
    if stored.value != source.value && ctx.value_needs_release_after_retaining_store(source) {
        crate::ir_lower::ownership::release_if_owned(ctx, source, Some(span));
    }
}

/// Loads an owned hidden temp into SSA and clears the backing slot without releasing it.
pub(super) fn take_owned_temp(
    ctx: &mut LoweringContext<'_, '_>,
    temp_name: &str,
    span: crate::span::Span,
) -> LoweredValue {
    let value = ctx.load_local(temp_name, Some(span));
    ctx.clear_owned_hidden_temp(temp_name, Some(span));
    value
}

/// Chooses a merge temp type from contextual branch materialization and fallback metadata.
pub(super) fn branch_merge_result_type(
    ctx: &LoweringContext<'_, '_>,
    then_expr: &Expr,
    else_expr: &Expr,
    expr: &Expr,
) -> PhpType {
    let then_ty = materialized_expr_type_for_merge(ctx, then_expr);
    let else_ty = materialized_expr_type_for_merge(ctx, else_expr);
    // Two resources merge into a resource, not into the `int` their storage erases to: the stream
    // builtins accept a resource handle only, so `fwrite($type === 'out' ? STDOUT : STDERR, $line)`
    // (illuminate/console `Scheduling\Event::execute()`) was refused "fwrite stream argument PHP
    // type Int".
    if let (PhpType::Resource(left), PhpType::Resource(right)) = (&then_ty, &else_ty) {
        return PhpType::Resource(if left == right { left.clone() } else { None });
    }
    let then_repr = then_ty.codegen_repr();
    let else_repr = else_ty.codegen_repr();
    if then_repr == else_repr {
        // An element read of a container answers `null` when the key is missing. A packed list
        // slot carries that as its canonical-null sentinel, but a HASH slot has no null:
        // `$c ? $rows[5] : ["a" => 1]` merged as a hash and handed the missed read's null
        // pointer to the next hash operation. Box such a merge instead.
        if matches!(then_repr, PhpType::AssocArray { .. })
            && (container_element_read_may_miss(ctx, then_expr)
                || container_element_read_may_miss(ctx, else_expr))
        {
            return PhpType::Mixed;
        }
        return then_repr;
    }
    let branch_ty = nullable_aware_branch_merge_type(&then_ty, &else_ty);
    if php_type_allows_null(&branch_ty) {
        return branch_ty;
    }
    let fallback_ty = fallback_expr_type(expr).codegen_repr();
    wider_type_for_merge(&fallback_ty, &branch_ty.codegen_repr())
}

/// Whether `expr` reads an element of an array container, which yields `null` on a missing key.
fn container_element_read_may_miss(ctx: &LoweringContext<'_, '_>, expr: &Expr) -> bool {
    let ExprKind::ArrayAccess { array, .. } = &expr.kind else {
        return false;
    };
    let container = match &array.kind {
        ExprKind::Variable(name) => ctx.local_types.get(name).cloned(),
        _ => None,
    };
    match container.map(|ty| ty.codegen_repr()) {
        Some(PhpType::Array(_) | PhpType::AssocArray { .. }) => true,
        // An unknown container shape: only an element read of a nested element is known to be
        // an array read here.
        None => matches!(array.kind, ExprKind::ArrayAccess { .. }),
        Some(_) => false,
    }
}

/// Chooses a match hidden-temp type by merging every arm result type, so
/// heterogeneous arms (e.g. object/array/string) materialize a Mixed temp
/// boxed per arm instead of coercing all arms to one unified scalar type.
pub(super) fn match_merge_result_type(
    ctx: &LoweringContext<'_, '_>,
    arms: &[(Vec<Expr>, Expr)],
    default: Option<&Expr>,
    expr: &Expr,
) -> PhpType {
    let mut merged: Option<PhpType> = None;
    for result in arms.iter().map(|(_, result)| result).chain(default) {
        let arm_ty = materialized_expr_type_for_merge(ctx, result);
        merged = Some(match merged {
            Some(acc) => nullable_aware_branch_merge_type(&acc, &arm_ty),
            None => arm_ty,
        });
    }
    let Some(merged) = merged else {
        return fallback_expr_type(expr);
    };
    if php_type_allows_null(&merged) {
        return merged;
    }
    let fallback_ty = fallback_expr_type(expr).codegen_repr();
    wider_type_for_merge(&fallback_ty, &merged.codegen_repr())
}

/// Chooses a short-ternary hidden-temp type without reintroducing the
/// scalar-biased syntactic join used by the parser-only fallback inference.
///
/// A null default keeps the merge nullable, as the full ternary does: `$cond ?: null` merged as
/// `bool`, so the null branch stored the null sentinel into a bool temp and the result read back
/// as `false`-typed garbage -- `sprintf($format ??= ...)` printed `9223372036854775806` for
/// Symfony's `resolveEnvPlaceholders($config, $resolveEnvs ?: null)`.
pub(super) fn short_ternary_merge_result_type(
    ctx: &LoweringContext<'_, '_>,
    value: &Expr,
    default: &Expr,
) -> PhpType {
    let value_ty = materialized_expr_type_for_merge(ctx, value);
    let default_ty = materialized_expr_type_for_merge(ctx, default);
    let value_repr = value_ty.codegen_repr();
    let default_repr = default_ty.codegen_repr();
    if value_repr == default_repr {
        return value_repr;
    }
    let merged = nullable_aware_branch_merge_type(&value_ty, &default_ty);
    if php_type_allows_null(&merged) {
        return merged;
    }
    wider_type_for_merge(&value_repr, &default_repr)
}

/// Chooses a ternary branch merge type without erasing PHP null branches.
pub(super) fn nullable_aware_branch_merge_type(left: &PhpType, right: &PhpType) -> PhpType {
    if php_type_allows_null(left) || php_type_allows_null(right) {
        let left_non_null = strip_void_from_union(left.clone());
        let right_non_null = strip_void_from_union(right.clone());
        return normalize_union_members(vec![PhpType::Void, left_non_null, right_non_null])
            .unwrap_or(PhpType::Void);
    }
    wider_type_for_merge(&left.codegen_repr(), &right.codegen_repr())
}

/// Returns true when a PHP type can materialize PHP null at runtime.
pub(super) fn php_type_allows_null(php_type: &PhpType) -> bool {
    match php_type {
        PhpType::Void | PhpType::Never | PhpType::Mixed => true,
        PhpType::Union(members) => members
            .iter()
            .any(|member| matches!(member, PhpType::Void | PhpType::Never | PhpType::Mixed)),
        _ => false,
    }
}

/// Estimates the value type an expression will materialize during branch lowering.
pub(super) fn materialized_expr_type_for_merge(ctx: &LoweringContext<'_, '_>, expr: &Expr) -> PhpType {
    match &expr.kind {
        ExprKind::Variable(name) => normalize_value_php_type(ctx.local_type(name).codegen_repr()),
        ExprKind::ErrorSuppress(inner) => materialized_expr_type_for_merge(ctx, inner),
        ExprKind::BinaryOp { left, op, right } if mixed_numeric_op(op).is_some() => {
            let left_ty = materialized_expr_type_for_merge(ctx, left).codegen_repr();
            let right_ty = materialized_expr_type_for_merge(ctx, right).codegen_repr();
            if matches!(left_ty, PhpType::Mixed | PhpType::Union(_))
                || matches!(right_ty, PhpType::Mixed | PhpType::Union(_))
            {
                PhpType::Mixed
            } else {
                fallback_expr_type(expr)
            }
        }
        ExprKind::Ternary {
            then_expr,
            else_expr,
            ..
        } => branch_merge_result_type(ctx, then_expr, else_expr, expr),
        ExprKind::Match { arms, default, .. } => {
            match_merge_result_type(ctx, arms, default.as_deref(), expr)
        }
        ExprKind::ShortTernary { value, default } => {
            short_ternary_merge_result_type(ctx, value, default)
        }
        // `??` yields its value or its default, so it merges like a two-arm branch. The syntactic
        // fallback answered `int`: `$c ? $all[$k] ?? null : null` materialized a `?int` temp, and
        // the string the coalesce produced was stored through it as "". Symfony's
        // `ParameterBagUtils::getRequestParameterValue()` reads the login form's CSRF token
        // exactly this way, so every login on the demo failed "Invalid double-submit CSRF token".
        ExprKind::NullCoalesce { value, default } => {
            let value_ty = materialized_expr_type_for_merge(ctx, value);
            let default_ty = materialized_expr_type_for_merge(ctx, default);
            if value_ty.codegen_repr() == default_ty.codegen_repr() {
                value_ty
            } else {
                nullable_aware_branch_merge_type(&value_ty, &default_ty)
            }
        }
        // An element of a container this cannot type -- a call's result, most often -- merges as
        // boxed Mixed, like the call arms below. The syntactic fallback answered `int`, so
        // Laravel's `head($a)` (`empty($a) ? false : array_values($a)[0]`, or `reset()`)
        // returned `int(0)` for a string element, and `--env=staging` detected environment 0.
        ExprKind::ArrayAccess { array, .. } => {
            array_access_expr_value_type_for_ir(ctx, array).unwrap_or(PhpType::Mixed)
        }
        // A property the receiver's static class cannot type -- an untyped receiver, or a name
        // only a subclass, a dynamic property or `__get()` supplies -- is read as a boxed Mixed
        // value. The syntactic fallback answered `int` for it, so `$ok ? $o->items : 0` and
        // `$o->items ?: 0` materialized an int temp and read an array as its pointer's integer
        // cast: `gettype()` said "integer" where php says "array".
        ExprKind::PropertyAccess { object, property } => {
            property_access_expr_type_for_ir(ctx, object, property).unwrap_or(PhpType::Mixed)
        }
        ExprKind::NullsafePropertyAccess { object, property } => {
            super::nullsafe_property_access_expr_type_for_ir(ctx, object, property)
                .unwrap_or(PhpType::Mixed)
        }
        ExprKind::DynamicPropertyAccess { .. } | ExprKind::NullsafeDynamicPropertyAccess { .. } => {
            PhpType::Mixed
        }
        ExprKind::StaticPropertyAccess { receiver, property } => {
            static_property_result_type(ctx, receiver, property, expr)
        }
        // `!` and `instanceof` produce a bool in every PHP program, whatever their operand is.
        // Without these arms they reached the scalar-biased syntactic fallback, which answers
        // `int` for an expression it cannot type — and `wider_type_for_merge` then widened a
        // bool/int pair to `int`, so `$flag ?? !in_array(…)` materialized an INT merge temp and
        // a `bool` property refused the store. Symfony's `DebugHandlersListener::__construct`
        // writes exactly that.
        ExprKind::Not(_) | ExprKind::InstanceOf { .. } => PhpType::Bool,
        // `STDOUT` / `STDERR` are stream resources; the syntactic fallback answers `int`, which the
        // stream builtins then refuse (see `branch_merge_result_type`).
        ExprKind::ConstRef(name)
            if matches!(
                ctx.constant_value(name.as_str()),
                Some((_, PhpType::Resource(_)))
            ) =>
        {
            ctx.constant_value(name.as_str())
                .map(|(_, ty)| ty)
                .unwrap_or(PhpType::Mixed)
        }
        // A literal is typed from its items, the same way its own lowering will type it. The
        // syntactic fallback answered `array<int>` for `[$class, $name]`, and a ternary whose arms
        // were two such literals re-read every string and object as a heap address:
        // `$o === null ? [$c, $n] : [$o, $n]` printed `int(4351153472)` for the class name.
        ExprKind::ArrayLiteral(items) if !items.iter().any(|item| matches!(item.kind, ExprKind::Spread(_))) => {
            super::array_literal_type_for_ir(ctx, items, expr).codegen_repr()
        }
        // A literal with a spread is built with boxed elements whatever it unpacks; the syntactic
        // fallback answered `array<int>`, so `$c ? [] : [$m, ...$rest]` read every boxed element
        // back as its pointer: `json_encode()` printed `[4362301248]` for `["s"]`.
        ExprKind::ArrayLiteral(_) => PhpType::Array(Box::new(PhpType::Mixed)),
        ExprKind::ArrayLiteralAssoc(pairs) => {
            super::assoc_array_literal_type_for_ir(ctx, pairs, expr).codegen_repr()
        }
        // `C::A` is typed from the constant's value, the way `lower_scoped_constant` lowers it.
        // The syntactic fallback answers `string` for every scoped constant, so a ternary over two
        // int constants stored each arm through `i_to_str`: symfony/console's
        // `$decorated ? OutputInterface::OUTPUT_NORMAL : OutputInterface::OUTPUT_RAW` reached an
        // `int $options` parameter as 0, and `list --format=json` ran its JSON through the
        // formatter. An untypeable constant (late-bound `static::`) keeps the boxed Mixed temp.
        ExprKind::ScopedConstantAccess { receiver, name } => {
            super::scoped_constant_type_for_ir(ctx, receiver, name, 0)
                .map(|ty| normalize_value_php_type(ty.codegen_repr()))
                .unwrap_or(PhpType::Mixed)
        }
        // Calls are typed from the return metadata their own lowering uses. The syntactic
        // fallback answers `int` for every call outside its builtin allowlist, and a merge temp
        // typed from that guess coerced a string, array or object branch into an integer slot or,
        // beside a `null` branch, into a tagged `?int` scalar the backend cannot store an array
        // in (issue #1364). A call whose result cannot be named here merges as boxed `mixed`,
        // which holds whatever the call returns.
        ExprKind::FunctionCall { name, .. } => function_call_type_for_merge(ctx, name.as_str(), expr),
        ExprKind::MethodCall { object, method, .. } => {
            method_call_expr_type_for_ir(ctx, object, method).unwrap_or(PhpType::Mixed)
        }
        ExprKind::NullsafeMethodCall { object, method, .. } => {
            nullsafe_method_call_expr_type_for_ir(ctx, object, method).unwrap_or(PhpType::Mixed)
        }
        ExprKind::StaticMethodCall { receiver, method, .. } => {
            static_method_call_expr_type_for_ir(ctx, receiver, method).unwrap_or(PhpType::Mixed)
        }
        ExprKind::ClosureCall { var, .. } => closure_call_type_for_merge(ctx, var),
        ExprKind::ExprCall { .. } => PhpType::Mixed,
        // An assignment yields the value it stored: a typed property's own type (the value is
        // coerced into it), otherwise the assigned value's. The syntactic fallback answered `int`,
        // so a `match` whose arms assign an int property in one arm and a string property in
        // another merged into a `?int` temp the string arm could not be stored in -- Laravel
        // Prompts' `TypedValue::trackTypedValue()` key handler, which left the whole prompt
        // family to the runtime autoloader.
        ExprKind::Assignment { target, value, .. } => match &target.kind {
            ExprKind::PropertyAccess { object, property } => {
                property_access_expr_type_for_ir(ctx, object, property)
                    .map(|ty| normalize_value_php_type(ty.codegen_repr()))
                    .unwrap_or_else(|| materialized_expr_type_for_merge(ctx, value))
            }
            _ => materialized_expr_type_for_merge(ctx, value),
        },
        _ => fallback_expr_type(expr),
    }
}

/// Returns the merge type of a named function call before it is lowered.
///
/// A user or extern function answers the return type `lower_function_call` stamps on the call.
/// A pop or shift uses its shared EIR result contract, which is boxed `Mixed` even when
/// the checker sees a scalar array element: removing from an empty array returns null.
/// Other builtins keep the syntactic answer when that names a type, and otherwise consult
/// the checker's result type for this very call (`int`, `bool`, `float` and `string` stay
/// scalar, anything else is boxed), because syntactic `int` is only a default outside its
/// allowlist.
fn function_call_type_for_merge(ctx: &LoweringContext<'_, '_>, canonical: &str, expr: &Expr) -> PhpType {
    if !source_prefers_extension_builtin(canonical) {
        if let Some(signature) = ctx.functions.get(canonical) {
            return normalize_value_php_type(eir_user_function_return_type(signature));
        }
    }
    if let Some(signature) = ctx.extern_functions.get(canonical) {
        return normalize_value_php_type(signature.return_type.clone());
    }
    if let Some(def) = crate::builtins::registry::lookup(canonical.trim_start_matches('\\')) {
        if matches!(
            def.spec.semantics.lowering,
            crate::builtins::semantics::BuiltinLowering::Runtime(
                crate::ir::RuntimeCallTarget::Function(
                    crate::ir::RuntimeFnId::ArrayPop | crate::ir::RuntimeFnId::ArrayShift
                )
            )
        ) {
            return resolve_registry_builtin_result_type(
                ctx,
                def.name,
                &[],
                &[],
                expr.span,
                None,
            )
            .expect("registered pop/shift builtin must resolve its result type");
        }
    }
    let syntactic = fallback_expr_type(expr);
    if syntactic != PhpType::Int {
        return syntactic;
    }
    builtin_call_result_type_for_ir(ctx, expr.span).unwrap_or(syntactic)
}

/// Returns the merge type of a `$callable(...)` call through a local variable.
///
/// Mirrors `lower_closure_call`: a statically known closure or user function answers its
/// signature's return type and a typed callable parameter its descriptor signature's.
/// Everything else (builtin names, methods, invokable objects, unknown callables) merges as
/// boxed `mixed`.
fn closure_call_type_for_merge(ctx: &LoweringContext<'_, '_>, var: &str) -> PhpType {
    match ctx.static_callable_local(var) {
        Some(target @ StaticCallableBinding::Closure { .. })
        | Some(target @ StaticCallableBinding::UserFunction(_)) => {
            return static_callable_return_type(ctx, &target);
        }
        Some(_) => return PhpType::Mixed,
        None => {}
    }
    ctx.callable_param_signature(var)
        .map(|signature| descriptor_invoker_result_type(Some(signature)))
        .unwrap_or(PhpType::Mixed)
}

/// Coerces branch values to the hidden temp storage type before storing them.
pub(super) fn coerce_value_for_temp(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    temp_type: &PhpType,
    span: crate::span::Span,
) -> LoweredValue {
    let target_ty = temp_type.codegen_repr();
    let source_ty = ctx.builder.value_php_type(value.value).codegen_repr();
    if source_ty == target_ty {
        return value;
    }
    // A nullable named object uses boxed Mixed storage, while a `??` merge strips the null
    // arm and therefore chooses the concrete object representation for its result temporary.
    // Reuse the ownership-aware gradual narrowing path before storing the non-null branch: a
    // raw object slot must never receive the outer Mixed-cell pointer as its payload.
    if matches!(target_ty, PhpType::Object(_)) {
        return crate::ir_lower::gradual_coercions::coerce_gradual_value_to_boundary(
            ctx,
            value,
            &target_ty,
            Some(span),
        );
    }
    match &target_ty {
        PhpType::Mixed => ctx.box_value_as_mixed(value, PhpType::Mixed, Some(span)),
        PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never => {
            coerce_to_int_at_span(ctx, value, Some(span))
        }
        PhpType::Float => coerce_to_float_at_span(ctx, value, Some(span)),
        PhpType::Str => coerce_to_string_at_span(ctx, value, Some(span)),
        _ => coerce_container_to_mixed_payload(ctx, value, &source_ty, &target_ty, Some(span)),
    }
}

/// Widens a typed container value to boxed-Mixed element storage before it is stored.
///
/// Branch merges and stable loop-local contracts can require `Mixed` element storage, so each
/// concrete container must box its slots via `ArrayToMixed` / `HashToMixed`: storing the raw
/// pointer would let Mixed-element reads misinterpret typed slot bytes. Borrowed sources are
/// retained first so the conversion's copy-on-write split rewrites a private copy; owning
/// temporaries transfer their reference into the converted result.
pub(in crate::ir_lower) fn coerce_container_to_mixed_payload(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    source_ty: &PhpType,
    target_ty: &PhpType,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    let target_has_mixed_payload = match target_ty {
        PhpType::Array(elem) => elem.codegen_repr() == PhpType::Mixed,
        PhpType::AssocArray { value, .. } => value.codegen_repr() == PhpType::Mixed,
        _ => false,
    };
    if !target_has_mixed_payload {
        return value;
    }
    let (op, result_ty) = match (source_ty, target_ty) {
        (PhpType::Array(source_elem), PhpType::Array(_))
            if source_elem.codegen_repr() != PhpType::Mixed =>
        {
            (Op::ArrayToMixed, target_ty.clone())
        }
        (PhpType::AssocArray { value: source_value, .. }, PhpType::AssocArray { .. })
            if source_value.codegen_repr() != PhpType::Mixed =>
        {
            (Op::HashToMixed, target_ty.clone())
        }
        (
            PhpType::AssocArray {
                key,
                value: source_value,
            },
            PhpType::Array(target_element),
        ) if source_value.codegen_repr() != PhpType::Mixed
            && target_element.codegen_repr() == PhpType::Mixed =>
        {
            (
                Op::HashToMixed,
                PhpType::AssocArray {
                    key: key.clone(),
                    value: Box::new(PhpType::Mixed),
                },
            )
        }
        (PhpType::Array(source_elem), PhpType::AssocArray { .. }) => {
            // An INDEXED argument bound to a hash contract. A bare `array` hint resolves to the
            // hash whenever some call site passes one (`respecialized_param_types_for_call`), so
            // the packed vector is CONVERTED here -- restamping it would hand the callee a
            // pointer to the wrong layout. `Op::ArrayToHash` carries the element type across
            // unchanged, so raw slots are boxed first to reach the contract's Mixed payload.
            let source = if source_elem.codegen_repr() == PhpType::Mixed {
                value
            } else {
                let borrowed = if ctx.value_is_owning_temporary(value)
                    && !ctx.value_is_owned_unboxed_local_load(value.value)
                {
                    value
                } else {
                    crate::ir_lower::ownership::acquire_if_refcounted(ctx, value, span)
                };
                ctx.emit_value(
                    Op::ArrayToMixed,
                    vec![borrowed.value],
                    None,
                    PhpType::Array(Box::new(PhpType::Mixed)),
                    Op::ArrayToMixed.default_effects(),
                    span,
                )
            };
            return ctx.emit_value(
                Op::ArrayToHash,
                vec![source.value],
                None,
                target_ty.clone(),
                Op::ArrayToHash.default_effects(),
                span,
            );
        }
        (PhpType::Mixed | PhpType::Union(_), _)
            if value.ir_type == IrType::Heap(IrHeapKind::Mixed) =>
        {
            // A whole-boxed gradual array can carry either indexed or associative storage at
            // runtime. Normalize both layouts through the checked sparse-container operation;
            // selecting the indexed-only runtime conversion from the declared `array<mixed>`
            // target would reinterpret an associative payload with the wrong layout. The
            // converted hash owns its payload independently and must keep an associative result
            // type so typed call boundaries can adapt its values to their concrete ABI storage.
            let cell_is_owning = ctx.value_is_owning_temporary(value);
            let result_ty = if matches!(target_ty, PhpType::Array(_)) {
                PhpType::AssocArray {
                    key: Box::new(PhpType::Mixed),
                    value: Box::new(PhpType::Mixed),
                }
            } else {
                target_ty.clone()
            };
            let converted = ctx.emit_value(
                Op::MixedToHash,
                vec![value.value],
                None,
                result_ty,
                Op::MixedToHash.default_effects(),
                span,
            );
            if cell_is_owning {
                crate::ir_lower::ownership::release_if_owned(ctx, value, span);
            }
            return converted;
        }
        _ => return value,
    };
    // Local loads report as *provisional* owners (their compensating releases
    // are pruned at builder finalization when the slot stays concrete), so
    // they must be treated as borrowed here: without a real retain the
    // conversion's copy-on-write split would never trigger and the local's
    // own array would be boxed in place while its slot type stays concrete.
    let source_is_consumable = ctx.value_is_owning_temporary(value)
        && !ctx.value_is_owned_unboxed_local_load(value.value);
    let source = if source_is_consumable {
        value
    } else {
        crate::ir_lower::ownership::acquire_if_refcounted(ctx, value, span)
    };
    ctx.emit_value(
        op,
        vec![source.value],
        None,
        result_ty,
        op.default_effects(),
        span,
    )
}

/// Emits a branch to a target block when the current block can still fall through.
pub(super) fn branch_to(ctx: &mut LoweringContext<'_, '_>, target: BlockId) {
    if !ctx.builder.insertion_block_is_terminated() {
        ctx.builder.terminate(Terminator::Br { target, args: Vec::new() });
    }
}

/// Computes definitely initialized slots after a two-way expression split.
pub(super) fn merge_initialized_slots_for_expr(
    split_initialized: &HashSet<LocalSlotId>,
    then_initialized: HashSet<LocalSlotId>,
    then_reachable: bool,
    else_initialized: HashSet<LocalSlotId>,
    else_reachable: bool,
) -> HashSet<LocalSlotId> {
    match (then_reachable, else_reachable) {
        (true, true) => then_initialized
            .intersection(&else_initialized)
            .copied()
            .collect(),
        (true, false) => then_initialized,
        (false, true) => else_initialized,
        (false, false) => split_initialized.clone(),
    }
}

/// Emits a boolean literal value for control-expression lowering.
///
/// Also emits the trailing warn-on-missing flag that boxed-`Mixed` subscript reads
/// pass to `__rt_mixed_array_get`, so every producer of such a read builds the
/// operand the same way.
pub(crate) fn emit_bool_literal(
    ctx: &mut LoweringContext<'_, '_>,
    value: bool,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    let value = ctx
        .builder
        .emit_with_effects(
            Op::ConstBool,
            Vec::new(),
            Some(Immediate::Bool(value)),
            IrType::I64,
            PhpType::Bool,
            Ownership::NonHeap,
            Op::ConstBool.default_effects(),
            span,
        )
        .expect("const_bool produces a value");
    LoweredValue { value, ir_type: IrType::I64 }
}

/// Returns a printable static receiver name.
pub(super) fn receiver_name(receiver: &StaticReceiver) -> String {
    match receiver {
        StaticReceiver::Named(name) => name.as_str().to_string(),
        StaticReceiver::Self_ => "self".to_string(),
        StaticReceiver::Static => "static".to_string(),
        StaticReceiver::Parent => "parent".to_string(),
    }
}

/// Returns a printable callable target name.
pub(super) fn callable_target_name(target: &CallableTarget) -> String {
    match target {
        CallableTarget::Function(name) => name.as_str().to_string(),
        CallableTarget::StaticMethod { receiver, method } => {
            format!("{}::{}", receiver_name(receiver), method)
        }
        CallableTarget::Method { method, .. } => format!("object::{}", method),
    }
}

/// Returns a syntactic fallback PHP type for an expression.
pub(super) fn fallback_expr_type(expr: &Expr) -> PhpType {
    normalize_value_php_type(infer_expr_type_syntactic(expr))
}

/// Normalizes non-materializable expression types to the EIR null sentinel.
pub(super) fn normalize_value_php_type(php_type: PhpType) -> PhpType {
    if matches!(php_type, PhpType::Never) {
        PhpType::Void
    } else {
        php_type
    }
}
