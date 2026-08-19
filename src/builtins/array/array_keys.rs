//! Purpose:
//! Home of the PHP `array_keys` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Array<Int>` for a concretely indexed array and `Array<key>`
//!   for an associative array. A gradual `Array<Mixed>` keeps `Array<Mixed>`
//!   because its runtime storage may be either representation.
//!   A check hook is required because the return type depends on the inferred
//!   argument type, which the `builtin!` `returns:` field cannot express.
//! - A `Mixed` argument (an array read out of a `mixed`-typed value: a builtin/prelude return,
//!   `json_decode()`, an index read on a `mixed` container) is ACCEPTED and yields
//!   `Array<Mixed>`, because the runtime key kind is only known once the box is opened. This
//!   matches `count()`, which has always accepted `Mixed`. The backend unboxes and dispatches
//!   on the runtime tag, raising PHP's `TypeError` when the box does not hold an array.
//! - Arity (exactly 1 argument) is validated by the registry's `check_arity` before
//!   the hook fires; the inline arity check from the legacy arm is not reproduced here.

use crate::builtins::semantics::{
    BuiltinResultType, BuiltinSemanticInput, BuiltinSemantics,
};
use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "array_keys",
    check: check,
    semantics: array_keys_semantics(),
}

/// Builds semantics whose EIR result follows the concrete lowered container representation.
const fn array_keys_semantics() -> BuiltinSemantics {
    let mut semantics =
        crate::builtins::semantics::runtime_fn_semantics(crate::ir::RuntimeFnId::ArrayKeys);
    semantics.result_type = BuiltinResultType::Shared(eir_result_type);
    semantics
}

/// Returns the key payload layout required by the lowered array or hash operand.
fn eir_result_type(input: &BuiltinSemanticInput<'_>) -> PhpType {
    let element = match input.arg_types.first().map(|ty| ty.codegen_repr()) {
        Some(PhpType::AssocArray { key, .. }) => key.codegen_repr(),
        Some(PhpType::Array(value)) if value.codegen_repr() != PhpType::Mixed => PhpType::Int,
        Some(PhpType::Array(_) | PhpType::Mixed | PhpType::Union(_)) => PhpType::Mixed,
        _ => PhpType::Mixed,
    };
    PhpType::Array(Box::new(element))
}

/// Returns the key-array type for an `array_keys` call.
///
/// A concretely indexed array produces `Array<Int>`; an associative array produces
/// `Array<key>`. Gradual arrays and `Mixed` values produce `Array<Mixed>` because their
/// runtime key kind is only known once the container is opened. Every other argument type
/// is rejected. The argument is re-inferred here to drive the return type; the registry
/// already inferred it once for side effects, and arity is pre-validated by the registry.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    match ty {
        PhpType::Array(element) if element.codegen_repr() == PhpType::Mixed => {
            Ok(PhpType::Array(Box::new(PhpType::Mixed)))
        }
        PhpType::Array(_) => Ok(PhpType::Array(Box::new(PhpType::Int))),
        PhpType::AssocArray { key, .. } => Ok(PhpType::Array(key)),
        PhpType::Mixed | PhpType::Union(_) => Ok(PhpType::Array(Box::new(PhpType::Mixed))),
        _ => Err(CompileError::new(
            cx.span,
            "array_keys() argument must be array",
        )),
    }
}
