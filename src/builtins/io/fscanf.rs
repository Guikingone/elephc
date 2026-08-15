//! Purpose:
//! Home of the PHP `fscanf` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` calls `ensure_stream_resource` on the stream argument for validation and
//!   returns `Array<Str>` reflecting the 2-argument form that returns matched fields.
//!   `returns: Mixed` is used because `Array<Str>` cannot be expressed through the
//!   scalar `returns:` field. Arguments are pre-inferred by the registry before the
//!   hook runs.
//! - The by-ref `$vars` output form is REFUSED rather than mis-executed, mirroring `sscanf()`.
//!   php assigns each field through the reference and returns the field COUNT; this backend
//!   cannot express that (`variadic: Some("vars")` is a bare NAME with no by-ref marker), and
//!   left accepted the call silently returned the ARRAY and assigned nothing.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "fscanf",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Fscanf,
    ),
}

/// Validates the stream argument and returns `Array<Str>` for the matched-fields result.
///
/// Also refuses the by-ref `$vars` output form, which this backend cannot express and
/// previously mis-executed in silence.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    crate::types::checker::builtins::io::common::ensure_stream_resource(
        cx.checker,
        cx.name,
        &cx.args[0],
        cx.env,
    )?;
    crate::builtins::string::sscanf::reject_by_ref_vars(cx.name, cx.args.len(), cx.span)?;
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
