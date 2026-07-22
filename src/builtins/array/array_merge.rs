//! Purpose:
//! Home of the PHP `array_merge` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The PHP golden signature is `variadic(&[], "arrays")` (min=0). The legacy CHECK
//!   arm requires exactly 2 arguments. `min_args: 2, max_args: 2` reproduce that
//!   enforcement in `check_arity` only; `function_sig` and the parity gate keep the
//!   variadic shape from the golden.
//! - `check` validates that the first argument is an indexed or associative array and
//!   returns the merged result type. The return type logic mirrors the legacy checker:
//!   when the first operand is an empty array (element type `Void`), the result adopts
//!   the second operand's element type if it is a scalar-merge type.
//! - Arity is pre-validated by `check_arity`; the hook can assume exactly 2 args.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_merge",
    area: Array,
    params: [],
    variadic: "arrays",
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayMerge,
    ),
    summary: "Merges the elements of two arrays.",
    php_manual: "https://www.php.net/manual/en/function.array-merge.php",
}

/// Validates the first argument is an array and returns the merged result type.
///
/// Arity (exactly 2 args) is pre-validated by `check_arity`. The hook re-infers both
/// argument types to derive the precise result type: when the left operand is an empty
/// indexed array (element type `Void`), the result adopts the right operand's element
/// type if it is a scalar-merge-compatible type.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    // PHP 8: `array_merge(array ...$arrays): array` — variadic, accepting zero or more
    // array arguments. Each argument is accepted under the gradual boundary (concrete
    // array, `Mixed`, or a union containing an array); a concretely non-array argument
    // is rejected. `array_merge()` with no arguments returns an empty array.
    let mut result: Option<PhpType> = None;
    for (idx, arg) in cx.args.iter().enumerate() {
        let ty = cx.checker.infer_type(arg, cx.env)?;
        if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&ty) {
            return Err(CompileError::new(
                cx.span,
                &format!("array_merge() argument #{} must be array", idx + 1),
            ));
        }
        result = Some(match result {
            None => ty,
            Some(acc) => array_merge_return_type(acc, ty),
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
