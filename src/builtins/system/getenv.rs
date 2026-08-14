//! Purpose:
//! Home of the PHP `getenv` builtin: its registry declaration and arity-sensitive result contract.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Calls without a name return the process environment as an associative string map.
//! - Named calls return `string|false`; dynamically nullable names therefore use `Mixed` storage.

use crate::builtins::semantics::{
    runtime_fn_semantics, BuiltinResultType, BuiltinSemanticInput, BuiltinSemantics,
};
use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "getenv",
    area: System,
    params: [
        name: Mixed = DefaultSpec::Null,
        local_only: Bool = DefaultSpec::Bool(false)
    ],
    arity_error: "getenv() takes 0 to 2 arguments",
    returns: Mixed,
    check: check,
    semantics: getenv_semantics(),
    summary: "Gets the value of an environment variable.",
}

/// Builds semantics whose EIR result matches the selected array or boxed-union representation.
const fn getenv_semantics() -> BuiltinSemantics {
    let mut semantics = runtime_fn_semantics(crate::ir::RuntimeFnId::Getenv);
    semantics.result_type = BuiltinResultType::Shared(eir_result_type);
    semantics
}

/// Returns the concrete backend layout selected by the statically known name type.
fn eir_result_type(input: &BuiltinSemanticInput<'_>) -> PhpType {
    match input.arg_types.first() {
        None | Some(PhpType::Void) => environment_array_type(),
        Some(PhpType::Mixed | PhpType::Union(_)) => PhpType::Mixed,
        Some(_) => PhpType::Mixed,
    }
}

/// Returns the environment map, named lookup union, or dynamic union representation.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let Some(name) = cx.args.first() else {
        return Ok(environment_array_type());
    };
    let name_ty = cx.checker.infer_type(name, cx.env)?;
    if let Some(local_only) = cx.args.get(1) {
        cx.checker.infer_type(local_only, cx.env)?;
    }
    match name_ty {
        PhpType::Void => Ok(environment_array_type()),
        PhpType::Mixed | PhpType::Union(_) => Ok(PhpType::Mixed),
        _ => Ok(cx.checker.normalize_union_type(vec![PhpType::Str, PhpType::False])),
    }
}

/// Returns the associative string-to-string type produced by an unnamed lookup.
fn environment_array_type() -> PhpType {
    PhpType::AssocArray {
        key: Box::new(PhpType::Str),
        value: Box::new(PhpType::Str),
    }
}
