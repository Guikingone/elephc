//! Purpose:
//! Home of the PHP `stream_socket_server` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Union(stream_resource, Bool)` reflecting PHP's false-on-failure return.
//! - `returns: Mixed` is used because the union cannot be expressed through the scalar field.
//! - The `error_code` and `error_message` parameters are by-reference: the caller passes
//!   plain variables that the runtime writes on failure.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    name: "stream_socket_server",
    area: Io,
    params: [
        address: Str,
        ref error_code: Mixed = DefaultSpec::Null,
        ref error_message: Mixed = DefaultSpec::Null,
        flags: Int = DefaultSpec::Int(12),
        context: Mixed = DefaultSpec::Null,
        peername: Mixed = DefaultSpec::Null
    ],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSocketServer,
    ),
    summary: "Create an Internet or Unix domain server socket.",
    php_manual: "function.stream-socket-server",
}

/// Validates by-ref output params are plain variables, then returns the union return type.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if let Some(ec) = cx.args.get(1) {
        if !matches!(ec.kind, ExprKind::Variable(_)) {
            return Err(CompileError::new(
                ec.span,
                &format!("{}() parameter $error_code must be passed a variable", cx.name),
            ));
        }
    }
    if let Some(em) = cx.args.get(2) {
        if !matches!(em.kind, ExprKind::Variable(_)) {
            return Err(CompileError::new(
                em.span,
                &format!("{}() parameter $error_message must be passed a variable", cx.name),
            ));
        }
    }
    Ok(cx.checker.normalize_union_type(vec![PhpType::stream_resource(), PhpType::False]))
}