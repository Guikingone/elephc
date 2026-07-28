//! Purpose:
//! Emits the checked-downcast-on-return runtime guard: when a `return`'s value is statically
//! only known to be an ANCESTOR (base class or implemented interface) of a declared return-type
//! arm — the base→derived relaxation accepted by
//! `crate::types::checker::type_compat::object_types::Checker::object_return_downcast_guardable`
//! — this inserts a chain of `Op::InstanceOf` checks that pass through on a match and throw a
//! catchable `\TypeError` (naming the ACTUAL runtime class) when every arm mismatches.
//!
//! Called from:
//! - `crate::ir_lower::stmt::lower_return`.
//!
//! Key details:
//! - Proven-safe returns (the value's static type already IS, or is a subtype of, a declared
//!   arm) pass through completely untouched — zero new EIR ops, zero runtime cost.
//! - The class-hierarchy walk here (`is_subclass_of`/`object_type_implements_interface`) mirrors
//!   the checker's own predicate over `Checker`-private tables; the checker's acceptance decision
//!   and this guard's emission decision must stay in lock-step (a divergence would either emit a
//!   redundant-but-harmless guard, or under-guard a value the checker had no business accepting
//!   — the "proven safe" fast path is intentionally the exact same walk, so it can only ever add
//!   a safety net, never remove one).
//! - The runtime message is built entirely in `crate::codegen::lower_inst::objects::
//!   return_type_guard`; only the compile-time prefix (function name + declared type, in PHP's
//!   own type-declaration syntax) is precomputed here and interned as `Op::ThrowCheckedReturnTypeError`'s
//!   immediate.

use std::collections::HashSet;

use crate::ir::{Immediate, Op, Terminator};
use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::span::Span;
use crate::types::PhpType;

/// Inserts the checked-downcast-on-return guard around `value` (the already-lowered return
/// expression result) when needed, and returns the value to continue lowering with. A no-op
/// (returns `value` unchanged) whenever the value isn't statically an `Object`, the declared
/// return type has no `Object` arm, or the value is already a proven subtype of a declared arm.
pub(crate) fn emit_checked_downcast_return_guard(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    let actual_ty = ctx.builder.value_php_type(value.value);
    let PhpType::Object(actual_name) = actual_ty else {
        return value;
    };
    if declared_accepts_any_object(&ctx.return_php_type) {
        return value; // bare `object` accepts every object: no guard, zero runtime cost
    }
    let candidates = declared_object_arms(&ctx.return_php_type);
    if candidates.is_empty() {
        return value;
    }
    if candidates
        .iter()
        .any(|declared| class_is_subtype_or_equal(ctx, &actual_name, declared))
    {
        return value; // proven safe: no guard, zero runtime cost
    }
    emit_guard_chain(ctx, value, &candidates, span)
}

/// Returns whether the declared return type has a bare `object` arm — PHP's `object` pseudo-type,
/// which the checker models as `PhpType::Object("")` (an EMPTY class name, not a class called
/// `""`). `object` accepts EVERY object, so such a return needs no runtime downcast check at all:
/// the caller takes the unguarded fast path.
///
/// Without this, `function mk(): object { return new A(); }` fell through to `emit_guard_chain`
/// with the empty name as the sole candidate, emitting `Op::InstanceOf` against class `""` — a
/// check no runtime class can ever satisfy — so every such return died at runtime with
/// `TypeError: mk(): Return value must be of type , A returned`. `declared_object_arms` also
/// filters the empty name out, so no `Object("")` arm can reach the chain from a union either.
fn declared_accepts_any_object(ty: &PhpType) -> bool {
    match ty {
        PhpType::Object(name) => name.is_empty(),
        PhpType::Union(members) => members.iter().any(declared_accepts_any_object),
        _ => false,
    }
}

/// Collects the `Object(name)` arms of a (possibly-union) declared PHP type, in source order,
/// deduped by name. Non-object arms (`Void`, scalars, …) don't participate in this guard, and
/// neither does the bare `object` pseudo-type (`Object("")`): it is not a class name, so it can
/// never be an `Op::InstanceOf` target. A declared type carrying one is handled entirely by
/// `declared_accepts_any_object`'s unguarded fast path before this is consulted.
fn declared_object_arms(ty: &PhpType) -> Vec<String> {
    /// Recursively appends every named `Object(name)` arm of `ty` to `out`, deduped by name.
    fn walk(ty: &PhpType, out: &mut Vec<String>) {
        match ty {
            PhpType::Object(name) => {
                if !name.is_empty() && !out.contains(name) {
                    out.push(name.clone());
                }
            }
            PhpType::Union(members) => {
                for member in members {
                    walk(member, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(ty, &mut out);
    out
}

/// Emits the `Op::InstanceOf` chain (one check per candidate, in declared order) and the
/// throwing fail path. The first matching candidate continues into `ok_block` with `value`
/// untouched; falling through every candidate throws a catchable `\TypeError`.
fn emit_guard_chain(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    candidates: &[String],
    span: Span,
) -> LoweredValue {
    let ok_block = ctx.builder.create_named_block("return_type_guard.ok", Vec::new());
    let fail_block = ctx.builder.create_named_block("return_type_guard.fail", Vec::new());

    for (index, candidate) in candidates.iter().enumerate() {
        let class_data = ctx.intern_class_name(candidate);
        let matched = ctx.emit_value(
            Op::InstanceOf,
            vec![value.value],
            Some(Immediate::Data(class_data)),
            PhpType::Bool,
            Op::InstanceOf.default_effects(),
            Some(span),
        );
        let is_last = index + 1 == candidates.len();
        let else_target = if is_last {
            fail_block
        } else {
            ctx.builder.create_named_block("return_type_guard.check", Vec::new())
        };
        ctx.builder.terminate(Terminator::CondBr {
            cond: matched.value,
            then_target: ok_block,
            then_args: Vec::new(),
            else_target,
            else_args: Vec::new(),
        });
        if !is_last {
            ctx.builder.position_at_end(else_target);
        }
    }

    ctx.builder.position_at_end(fail_block);
    let prefix_text = return_type_error_prefix(ctx);
    let prefix = ctx.intern_string(&prefix_text);
    ctx.emit_void(
        Op::ThrowCheckedReturnTypeError,
        vec![value.value],
        Some(Immediate::Data(prefix)),
        Op::ThrowCheckedReturnTypeError.default_effects(),
        Some(span),
    );
    ctx.builder.terminate(Terminator::Unreachable);

    ctx.builder.position_at_end(ok_block);
    value
}

/// Builds the compile-time message prefix `"F(): Return value must be of type D, "` (PHP's own
/// return-type `TypeError` wording; php-verified: `"f(): Return value must be of type D, B
/// returned"`, nullable single-object arms rendered `?D`). The runtime backend appends the
/// mismatched value's ACTUAL class name and a fixed `" returned"` suffix.
fn return_type_error_prefix(ctx: &LoweringContext<'_, '_>) -> String {
    format!(
        "{}(): Return value must be of type {}, ",
        ctx.owner_name(),
        format_declared_type_for_type_error(&ctx.return_php_type),
    )
}

/// Formats a declared PHP type using PHP's own type-declaration syntax, matching how a
/// `TypeError` return-type-mismatch message renders it: a two-member `T|null` union (the
/// shape nullable declarations normalize to) renders as `?T`; other unions join arms with `|`
/// in source order; everything else renders as a single type name.
fn format_declared_type_for_type_error(ty: &PhpType) -> String {
    if let PhpType::Union(members) = ty {
        if members.len() == 2 && members.iter().any(|member| matches!(member, PhpType::Void)) {
            if let Some(non_void) = members.iter().find(|member| !matches!(member, PhpType::Void)) {
                return format!("?{}", format_type_member(non_void));
            }
        }
        return members
            .iter()
            .map(format_type_member)
            .collect::<Vec<_>>()
            .join("|");
    }
    format_type_member(ty)
}

/// Formats a single (non-union) `PhpType` member using PHP's type-declaration spelling.
fn format_type_member(ty: &PhpType) -> String {
    match ty {
        // `Object("")` is PHP's bare `object` pseudo-type, spelled `object` in a declaration.
        PhpType::Object(name) if name.is_empty() => "object".to_string(),
        PhpType::Object(name) => name.clone(),
        PhpType::Int => "int".to_string(),
        PhpType::Float => "float".to_string(),
        PhpType::Str => "string".to_string(),
        PhpType::Bool => "bool".to_string(),
        PhpType::Void => "null".to_string(),
        PhpType::Mixed => "mixed".to_string(),
        PhpType::Iterable => "iterable".to_string(),
        PhpType::Callable => "callable".to_string(),
        PhpType::Array(_) | PhpType::AssocArray { .. } => "array".to_string(),
        other => format!("{:?}", other),
    }
}

/// True if `class_name` is provably `ancestor_name` or a subtype of it (class inheritance or
/// interface implementation). Mirrors `Checker::is_subclass_of`/`object_type_implements_interface`
/// (`src/types/checker/type_compat/object_types.rs`) over `ir_lower`'s own class/interface tables.
fn class_is_subtype_or_equal(ctx: &LoweringContext<'_, '_>, class_name: &str, ancestor_name: &str) -> bool {
    class_name == ancestor_name
        || is_subclass_of(ctx, class_name, ancestor_name)
        || object_type_implements_interface(ctx, class_name, ancestor_name)
}

/// Returns true if `class_name` is or inherits from `ancestor_name` (excluding self equality).
fn is_subclass_of(ctx: &LoweringContext<'_, '_>, class_name: &str, ancestor_name: &str) -> bool {
    let mut current = ctx.classes.get(class_name).and_then(|class| class.parent.clone());
    while let Some(parent_name) = current {
        if parent_name == ancestor_name {
            return true;
        }
        current = ctx.classes.get(&parent_name).and_then(|class| class.parent.clone());
    }
    false
}

/// Returns true if `class_name` directly implements `interface_name` (not via inheritance).
fn class_implements_interface(ctx: &LoweringContext<'_, '_>, class_name: &str, interface_name: &str) -> bool {
    ctx.classes.get(class_name).is_some_and(|class_info| {
        class_info.interfaces.iter().any(|name| name == interface_name)
    })
}

/// Returns true if `type_name` (a class or interface) implements `interface_name`, checking
/// direct implementation and interface inheritance chains.
fn object_type_implements_interface(ctx: &LoweringContext<'_, '_>, type_name: &str, interface_name: &str) -> bool {
    if ctx.classes.contains_key(type_name) {
        return class_implements_interface(ctx, type_name, interface_name);
    }
    if ctx.interfaces.contains_key(type_name) {
        return type_name == interface_name || interface_extends_interface(ctx, type_name, interface_name);
    }
    false
}

/// Returns true if `interface_name` is or transitively extends `ancestor_name`. DFS with cycle
/// detection over interface parent chains.
fn interface_extends_interface(ctx: &LoweringContext<'_, '_>, interface_name: &str, ancestor_name: &str) -> bool {
    if interface_name == ancestor_name {
        return true;
    }
    let mut stack = vec![interface_name.to_string()];
    let mut seen = HashSet::new();
    while let Some(current_name) = stack.pop() {
        if !seen.insert(current_name.clone()) {
            continue;
        }
        let Some(interface_info) = ctx.interfaces.get(&current_name) else {
            continue;
        };
        for parent_name in &interface_info.parents {
            if parent_name == ancestor_name {
                return true;
            }
            stack.push(parent_name.clone());
        }
    }
    false
}
