//! Purpose:
//! Routes an ancestor-typed method call to overrides that declare MORE parameters than the
//! ancestor, so their defaulted extra parameters are materialized.
//!
//! Called from:
//! - `crate::ir_lower::expr::method_calls::lower_method_call`.
//!
//! Key details:
//! - A virtual call site pads arguments from the signature of the class it is typed against.
//!   `Container::make()` calls `$container->load($file)` against `Container::load(string $file)`,
//!   and the generated container overrides it as `load($file, $lazyLoad = true)`: the call passed
//!   one argument and the override read `$lazyLoad` from whatever the register still held — a
//!   stack address, which crashed Symfony `--web` workers in some memory layouts.
//! - The closed world knows every class. For each concrete descendant whose method takes more
//!   parameters (all of the extra ones defaulted), the site compares the runtime class id and
//!   re-types the receiver to that class (`Op::Move`), so the call is lowered against the
//!   override's own signature and pads its defaults. Every other class keeps the original call.
//! - A shared vtable slot cannot carry this: sites typed against the subclass pass the full
//!   argument list through the same slot, so an ancestor-arity thunk would drop their arguments.

use super::*;

/// Lowers the call through a class-id ladder when some override is wider than the site's
/// signature; `None` leaves the ordinary lowering in charge.
pub(super) fn lower_wider_override_dispatch(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    method: &str,
    args: &[Expr],
    site_sig: Option<&FunctionSig>,
    expr: &Expr,
) -> Option<LoweredValue> {
    let site_sig = site_sig?;
    if site_sig.variadic.is_some() || !plain_positional_call_args(args) {
        return None;
    }
    // The ladder re-uses the receiver in several branches; an owning temporary would need a
    // release decision per branch, and the shape that needs the ladder is a variable receiver.
    if ctx.value_is_owning_temporary(object) {
        return None;
    }
    let base = match ctx.builder.value_php_type(object.value) {
        PhpType::Object(name) => name.trim_start_matches('\\').to_string(),
        _ => return None,
    };
    let site_arity = site_sig.params.len();
    if args.len() > site_arity {
        return None;
    }
    let targets = wider_override_classes(ctx, &base, &php_symbol_key(method), site_arity);
    if targets.is_empty() {
        return None;
    }

    let result_type = method_call_result_type(ctx, object.value, method, Op::MethodCall, expr);
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let merge = ctx
        .builder
        .create_named_block("method.override_arity.merge", Vec::new());
    let class_id = ctx.emit_value(
        Op::ObjectClassId,
        vec![object.value],
        None,
        PhpType::Int,
        Op::ObjectClassId.default_effects(),
        Some(expr.span),
    );
    for (class_name, id) in targets {
        let hit = ctx
            .builder
            .create_named_block("method.override_arity.hit", Vec::new());
        let next = ctx
            .builder
            .create_named_block("method.override_arity.next", Vec::new());
        let expected = ctx.emit_value(
            Op::ConstI64,
            Vec::new(),
            Some(Immediate::I64(id as i64)),
            PhpType::Int,
            Op::ConstI64.default_effects(),
            Some(expr.span),
        );
        let matches = ctx.emit_value(
            Op::ICmp,
            vec![class_id.value, expected.value],
            Some(Immediate::CmpPredicate(crate::ir::CmpPredicate::Eq)),
            PhpType::Bool,
            Op::ICmp.default_effects(),
            Some(expr.span),
        );
        ctx.builder.terminate(Terminator::CondBr {
            cond: matches.value,
            then_target: hit,
            then_args: Vec::new(),
            else_target: next,
            else_args: Vec::new(),
        });

        ctx.builder.position_at_end(hit);
        let narrowed = ctx.emit_value(
            Op::Move,
            vec![object.value],
            None,
            PhpType::Object(class_name),
            Op::Move.default_effects(),
            Some(expr.span),
        );
        let call = lower_method_call_with_receiver(ctx, narrowed, method, args, Op::MethodCall, expr);
        store_value_into_temp(ctx, &temp_name, result_type.clone(), call, expr.span);
        branch_to(ctx, merge);

        ctx.builder.position_at_end(next);
    }
    let call = lower_method_call_with_receiver(ctx, object, method, args, Op::MethodCall, expr);
    store_value_into_temp(ctx, &temp_name, result_type, call, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    Some(take_owned_temp(ctx, &temp_name, expr.span))
}

/// Concrete classes at or below `base` whose `method` declares more than `site_arity` parameters,
/// every extra one defaulted and by value. Each class is matched by its exact id, so the order
/// only needs to be deterministic.
fn wider_override_classes(
    ctx: &LoweringContext<'_, '_>,
    base: &str,
    method_key: &str,
    site_arity: usize,
) -> Vec<(String, u64)> {
    let mut found = Vec::new();
    for (name, info) in ctx.classes.iter() {
        if info.is_abstract || !class_is_or_implements(ctx, name, base) {
            continue;
        }
        let Some(sig) = info.methods.get(method_key) else {
            continue;
        };
        if sig.variadic.is_some() || sig.params.len() <= site_arity {
            continue;
        }
        let extra_defaulted = sig
            .defaults
            .get(site_arity..)
            .is_some_and(|defaults| defaults.iter().all(Option::is_some));
        let extra_by_value = sig.ref_params.iter().skip(site_arity).all(|by_ref| !by_ref);
        if extra_defaulted && extra_by_value {
            found.push((name.to_string(), info.class_id));
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// Whether `class_name` is `target`, extends it, or implements it (directly or through a parent).
fn class_is_or_implements(ctx: &LoweringContext<'_, '_>, class_name: &str, target: &str) -> bool {
    let target_key = php_symbol_key(target);
    let mut current = Some(class_name.trim_start_matches('\\').to_string());
    while let Some(name) = current {
        if php_symbol_key(&name) == target_key {
            return true;
        }
        let Some(info) = ctx.classes.get(name.as_str()) else {
            return false;
        };
        if info
            .interfaces
            .iter()
            .any(|interface| php_symbol_key(interface.trim_start_matches('\\')) == target_key)
        {
            return true;
        }
        current = info.parent.clone();
    }
    false
}
