//! Purpose:
//! Home of the PHP `stream_socket_accept` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates arg[0] is a stream resource and that `peer_name` (arg[2]), if provided,
//!   is a plain variable (it is written by reference).
//! - Arguments are pre-inferred by the registry before the hook runs.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    contract: "stream_socket_accept",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSocketAccept,
    ),
}

/// Validates arg[0] is a stream resource and that `peer_name` (arg[2]) is a plain variable.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    crate::types::checker::builtins::io::common::ensure_stream_resource(cx.checker, cx.name, &cx.args[0], cx.env)?;
    if let Some(peer) = cx.args.get(2) {
        if !matches!(peer.kind, ExprKind::Variable(_)) {
            return Err(CompileError::new(
                peer.span,
                "stream_socket_accept() parameter $peer_name must be passed a variable",
            ));
        }
    }
    Ok(cx.checker.normalize_union_type(vec![PhpType::stream_resource(), PhpType::False]))
}
