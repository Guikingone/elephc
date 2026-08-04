//! Purpose:
//! Home of the PHP `json_encode` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The check hook validates that all flag/depth arguments are integers, reporting
//!   each type error at the offending argument's span (not the call span).

use crate::builtins::semantics::{
    runtime_fn_semantics, BuiltinResultType, BuiltinSemanticInput, BuiltinSemantics,
};
use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "json_encode",
    area: System,
    params: [
        value: Mixed,
        flags: Int = DefaultSpec::Int(0),
        depth: Int = DefaultSpec::Int(512),
    ],
    returns: Mixed,
    check: check,
    semantics: json_encode_semantics(),
    summary: "Returns the JSON representation of a value.",
}

/// Builds semantics whose EIR result matches the runtime's boxed string-or-false value.
const fn json_encode_semantics() -> BuiltinSemantics {
    let mut semantics = runtime_fn_semantics(crate::ir::RuntimeFnId::JsonEncode);
    semantics.result_type = BuiltinResultType::Shared(eir_result_type);
    semantics
}

/// Returns the representation-safe EIR type for the runtime-selected string-or-false result.
fn eir_result_type(_input: &BuiltinSemanticInput<'_>) -> PhpType {
    PhpType::Mixed
}

/// Validates that all flag and depth arguments are integers.
///
/// Reports type errors at the span of the offending argument, not the call span.
///
/// The strict `!= Int` test is deliberate and must NOT be widened to the shared
/// `accepts_gradual_int` boundary yet: doing so moves Symfony's `json_encode($data, $flags)`
/// (JsonDescriptor.php:88, where `$flags` is `$opts['x'] ?? 0`) from `error[…]` to an
/// `EIR backend error`, i.e. an invisible relocation the `^error\[` counter cannot see.
///
/// The blocker MOVED, so re-measure before trusting an older note: this used to fail as
/// `runtime_call missing operand 2` (an EIR const-fold operand drop), and that is fixed. Widening
/// the boundary today reaches codegen and stops at
/// `unsupported EIR backend feature: json_encode flags for PHP type Mixed` —
/// `lower_json_encode_flags` in `crate::codegen::lower_inst::builtins::json` runs the operand
/// through `require_integer_like`, which admits only `Int`/`Bool`. Widen this together with an
/// unbox-and-truncate of a boxed flags operand there, not before.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    for extra in &cx.args[1..] {
        let ty = cx.checker.infer_type(extra, cx.env)?;
        if !crate::types::checker::builtins::numeric::accepts_gradual_int(&ty) {
            return Err(CompileError::new(
                extra.span,
                "json_encode() flags and depth must be integers",
            ));
        }
    }
    Ok(PhpType::Str)
}
