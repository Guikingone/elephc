//! Purpose:
//! Home of the PHP `array_merge` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - PHP accepts zero or more arrays; the registry signature and checker preserve that variadic
//!   contract instead of imposing the legacy backend's former two-array restriction.
//! - `check` validates every argument as an indexed, associative, or gradual array and
//!   accumulates the merged result type. The return type logic mirrors the legacy checker:
//!   when the first operand is an empty array (element type `Void`), the result adopts
//!   the second operand's element type if it is a scalar-merge type.
//! - Zero arguments produce an empty `array<void>` result.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "array_merge",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayMerge,
    ),
}

/// Validates every variadic argument as array-compatible and returns the merged result type.
///
/// With no operands the result is an empty indexed array. Otherwise each concrete non-array is
/// rejected while gradual array boundaries remain accepted, and compatible packed element types
/// are accumulated from left to right.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let mut result = None;
    for (index, arg) in cx.args.iter().enumerate() {
        let ty = cx.checker.infer_type(arg, cx.env)?;
        if !crate::types::checker::builtins::array_arg_is_gradually_acceptable(&ty) {
            return Err(CompileError::new(
                cx.span,
                &format!("array_merge() argument #{} must be array", index + 1),
            ));
        }
        result = Some(match result {
            None => ty,
            Some(previous) => array_merge_return_type(previous, ty),
        });
    }
    Ok(result.unwrap_or_else(|| PhpType::Array(Box::new(PhpType::Void))))
}


/// Infers the return type for `array_merge(first, second)`.
///
/// When `first` is an empty indexed array (element type `Void`), the merged result
/// adopts `second`'s element type if it is a scalar-merge-compatible type; otherwise
/// the result keeps `first`'s type. For non-empty indexed arrays, the left operand
/// type is returned unchanged (matching legacy checker behavior).
fn array_merge_return_type(first: PhpType, second: PhpType) -> PhpType {
    if array_merge_uses_gradual_storage(&first) || array_merge_uses_gradual_storage(&second) {
        return PhpType::Array(Box::new(PhpType::Mixed));
    }
    match first {
        PhpType::Array(elem) if is_empty_array_element_type(elem.as_ref()) => match second {
            PhpType::Array(right) if is_scalar_merge_element_type(right.as_ref()) => {
                PhpType::Array(right)
            }
            _ => PhpType::Array(elem),
        },
        other => other,
    }
}

/// Returns whether array merging needs the key-aware, runtime-dispatched array representation.
///
/// Associative, boxed, union, and already-generic operands route through the PHP compatibility
/// prelude in EIR lowering. Its result can be either indexed or hash storage and can contain
/// heterogeneous values, so the checker must expose the same `array<mixed>` contract.
fn array_merge_uses_gradual_storage(ty: &PhpType) -> bool {
    match ty.codegen_repr() {
        PhpType::AssocArray { .. } | PhpType::Mixed | PhpType::Union(_) => true,
        PhpType::Array(element) => element.codegen_repr() == PhpType::Mixed,
        _ => false,
    }
}

/// Returns true for the element sentinel used by statically empty indexed arrays.
fn is_empty_array_element_type(ty: &PhpType) -> bool {
    matches!(ty.codegen_repr(), PhpType::Void)
}

/// Returns true for element types that the scalar merge runtime helper copies safely.
fn is_scalar_merge_element_type(ty: &PhpType) -> bool {
    matches!(
        ty.codegen_repr(),
        PhpType::Int
        | PhpType::Bool
        | PhpType::False
        | PhpType::Float
        | PhpType::Callable
        | PhpType::Void
    )
}
