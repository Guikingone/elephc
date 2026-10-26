//! Purpose:
//! Home of the PHP `array_push` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The golden signature is `first_param_ref(variadic(["array"], "values"))`: `array`
//!   by-ref plus a variadic `values` param, so `min_args: 1` is the only `check_arity`
//!   override and the maximum is unbounded. It used to be pinned to `2, 2`, which rejected
//!   ordinary PHP (`array_push($a, 3, 4)`) with an arity error (issue #677).
//! - The `ref` marker on `array` is mandatory — it is what makes by-reference mutation
//!   lower correctly (ir_lower reads `ref_params` from the registry sig).
//! - `lazy_check: true` lets this builtin admit generator and `Traversable` spreads through
//!   its runtime iterator coordinator before generic call inference rejects non-array unpacking.
//! - Returns `Int` — the new number of elements, as PHP does and as the `array_unshift`
//!   sibling already did. It used to return `Void`, so `$n = array_push($a, 1)` read `NULL`.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    contract: "array_push",
    check: check,
    lazy_check: true,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayPush,
    ),
}

/// Validates indexed, associative and boxed PHP array receivers for an `array_push` call.
///
/// Arity (at least 1 arg) is pre-validated by `check_arity`. Every argument is inferred so the
/// appended values still produce their side effects; spreads may be arrays or Traversables because
/// the associative coordinator owns their runtime iterator walk. The receiver must be indexed,
/// associative or a declared PHP array with boxed storage. Returns the new element count as `Int`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let arr_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    let aliased_indexed_receiver = cx
        .checker
        .local_has_non_builtin_reference_alias(&cx.args[0]);
    let value_constraint = match arr_ty.codegen_repr() {
        PhpType::AssocArray { value, .. }
            if !matches!(value.codegen_repr(), PhpType::Mixed | PhpType::Iterable) => Some((
                value.as_ref().clone(),
                "array_push() values must match the associative receiver entry type",
            )),
        PhpType::Array(element)
            if aliased_indexed_receiver
                && !matches!(element.codegen_repr(), PhpType::Mixed | PhpType::Iterable) =>
        {
            Some((
                element.as_ref().clone(),
                "array_push() cannot widen a reference-aliased indexed receiver",
            ))
        }
        _ => None,
    };
    for index in 1..cx.args.len() {
        let value_ty = if matches!(cx.args[index].kind, ExprKind::Spread(_)) {
            let (_, element) = cx
                .checker
                .infer_runtime_iterated_call_arg_types(&cx.args[index], cx.env)?;
            element
        } else {
            cx.checker.infer_type(&cx.args[index], cx.env)?
        };
        if let Some((expected, message)) = &value_constraint {
            if value_ty.codegen_repr() != expected.codegen_repr() {
                return Err(CompileError::new(
                    cx.args[index].span,
                    message,
                ));
            }
        }
    }
    let denied_place = cx.checker.record_array_push_reference_place(&cx.args[0], cx.env)?;
    if denied_place || matches!(arr_ty, PhpType::Array(_) | PhpType::AssocArray { .. }) || arr_ty.is_php_array() {
        Ok(PhpType::Int)
    } else {
        Err(CompileError::new(cx.span, "array_push() first argument must be array"))
    }
}
