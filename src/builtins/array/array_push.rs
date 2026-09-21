//! Purpose:
//! Home of the PHP `array_push` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The golden signature is `first_param_ref(variadic(["array"], "values"))`: `array`
//!   by-ref plus a variadic `values` param, and `min_args: 1, max_args: None` is that shape.
//!   A legacy CHECK arm used to cap it at exactly 2 arguments, which refused PHP's own
//!   `array_push($a, $x, $y)`; Twig's `ArrayExpression::addElement` writes exactly that. The
//!   minimum is ONE: `array_push($a)` appends nothing and answers `count($a)`, and PHP's own
//!   diagnostic is `expects at least 1 argument`.
//! - The `ref` marker on `array` is mandatory — it is what makes by-reference mutation
//!   lower correctly (ir_lower reads `ref_params` from the registry sig).
//! - Returns the new element count, like PHP and like the eval interpreter
//!   (`eval_array_push_unshift_count_result`). It used to return `Void`, so `$n = array_push(...)`
//!   was empty when compiled and correct when interpreted.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "array_push",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayPush,
    ),
}

/// Validates the receiver of an `array_push` call and infers every pushed value.
///
/// Arity (at least 1 arg) is pre-validated by `check_arity`. Every argument is inferred so its
/// side effects happen, and the receiver must be a shape `lower_array_push` dispatches on: an
/// indexed `Array`, a `Mixed`-valued `AssocArray`, or the boxed `Mixed`/`Union` cell it hands to
/// `lower_mixed_array_append`.
///
/// The hash receiver is php's own behaviour -- `array_push()` appends under the next free integer
/// key whatever storage the array has -- and the backend reaches it through the same runtime
/// heap-kind dispatch a packed `array<mixed>` already used when a by-reference write had promoted
/// it. `twig/twig`'s `ArrayExpression::addElement` writes `array_push($this->nodes, $key, $value)`
/// against a string-keyed child map, and refusing it stopped the build. A hash with a NARROWER
/// value payload stays refused: the append would store a boxed cell into raw-pointer slots.
/// Returns the array's new element count, as PHP does.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let arr_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    for index in 1..cx.args.len() {
        cx.checker.infer_type(&cx.args[index], cx.env)?;
    }
    if matches!(
        arr_ty,
        PhpType::Array(_) | PhpType::Mixed | PhpType::Union(_)
    ) || matches!(
        arr_ty,
        PhpType::AssocArray { ref value, .. } if value.codegen_repr() == PhpType::Mixed
    ) {
        Ok(PhpType::Int)
    } else {
        Err(CompileError::new(cx.span, "array_push() first argument must be array"))
    }
}
