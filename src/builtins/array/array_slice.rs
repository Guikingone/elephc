//! Purpose:
//! Home of the PHP `array_slice` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` reproduces the legacy rule: a slice preserves the array shape, so the
//!   return type is the (array-or-assoc) input type unchanged; a boxed `Mixed`/`Union`
//!   input yields `Mixed`. A check hook is required because the return type depends on
//!   the inferred first-argument type.
//! - The declared signature carries the golden param list (`array`, `offset`,
//!   `length`), with `length` optional (default `null`), so the registry's
//!   `check_arity` accepts 2 or 3 arguments — matching the legacy CHECK arm.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::builtins::semantics::{
    runtime_fn_semantics, BuiltinResultType, BuiltinSemanticInput, BuiltinSemantics,
};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_slice",
    area: Array,
    params: [array: Mixed, offset: Mixed, length: Mixed = DefaultSpec::Null, preserve_keys: Bool = crate::builtins::spec::DefaultSpec::Bool(false)],
    returns: Mixed,
    check: check,
    semantics: array_slice_semantics(),
    summary: "Extracts a slice of an array.",
    php_manual: "https://www.php.net/manual/en/function.array-slice.php",
}

/// Builds semantics with the boxed-Mixed indexed result layout used by the slice runtime.
const fn array_slice_semantics() -> BuiltinSemantics {
    let mut semantics = runtime_fn_semantics(crate::ir::RuntimeFnId::ArraySlice);
    semantics.result_type = BuiltinResultType::Shared(eir_result_type);
    semantics
}

/// Returns the indexed array type the slice runtime actually produces.
///
/// This MUST agree with `check` below on the element type. It used to hard-code
/// `array<mixed>` while `check` preserved the source element type, and the two are read by
/// different consumers: codegen widened the result's slots to boxed cells on the EIR type, while
/// a caller read elements back using the CHECKER's type. Reading a raw `Str`/`Int` slot out of an
/// array that really held boxed cells loaded the box POINTER as the payload, so an
/// `array_slice()` result crossing a function return read back empty (strings) or as a heap
/// address (ints) — silently, with `count()` still correct.
///
/// A slice re-emits the very elements it was handed, so the element type is the source's; only a
/// boxed `Mixed`/`Union` source (whose runtime layout is genuinely boxed) yields `array<mixed>`.
fn eir_result_type(input: &BuiltinSemanticInput<'_>) -> PhpType {
    let mixed_elements = PhpType::Array(Box::new(PhpType::Mixed));
    let Some(source) = input.arg_types.first() else {
        return mixed_elements;
    };
    match source.codegen_repr() {
        PhpType::Array(elem) => PhpType::Array(elem),
        PhpType::AssocArray { value, .. } => PhpType::Array(value),
        _ => mixed_elements,
    }
}

/// Returns the slice's array type for an `array_slice` call.
///
/// A slice preserves the input array shape, so the (array-or-assoc) first-argument
/// type is returned unchanged; a boxed `Mixed`/`Union` first argument yields `Mixed`.
/// Non-array first arguments are rejected. The first argument is re-inferred here;
/// the registry already inferred every argument once for side effects, and arity
/// (2 or 3) is pre-validated by the registry.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
        return Ok(PhpType::Mixed);
    }
    if !matches!(ty, PhpType::Array(_) | PhpType::AssocArray { .. }) {
        return Err(CompileError::new(
            cx.span,
            "array_slice() first argument must be array",
        ));
    }
    Ok(ty)
}
