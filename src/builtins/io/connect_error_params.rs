//! Purpose:
//! Shared helper for the socket-connect builtin homes in the io area. Provides
//! `check_connect_error_params`, which validates the by-reference `$error_code` /
//! `$error_message` out-params and returns the common `Union(stream_resource, Bool)`
//! result type.
//!
//! Called from:
//! - `crate::builtins::io::fsockopen` (check hook)
//! - `crate::builtins::io::pfsockopen` (check hook)
//! - `crate::builtins::io::stream_socket_client` (check hook)
//!
//! Key details:
//! - The out-params sit at different argument positions per builtin — `fsockopen` and
//!   `pfsockopen` put them at args 2 and 3, `stream_socket_client` at args 1 and 2 — so the
//!   caller passes the indices rather than this helper hardcoding one convention. This mirrors
//!   `store_socket_error_outputs` in the codegen layer, which is parameterized the same way so
//!   the checker and the emitter cannot drift apart about where the out-params live.
//! - PHP requires an lvalue for a by-reference parameter, so a non-variable argument is a
//!   compile error rather than a silently discarded write.
//! - The return type is `Union(stream_resource, Bool)`, reflecting PHP's false-on-failure return.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

/// Validates the by-reference error out-params, then returns `Union(stream_resource, Bool)`.
///
/// `errno_index` and `errstr_index` locate `$error_code` and `$error_message` in the argument
/// list. Each is optional: an absent argument is fine, but a present one must be a plain
/// variable so the compiled code has a slot to write back into.
pub(crate) fn check_connect_error_params(
    cx: &mut BuiltinCheckCtx,
    errno_index: usize,
    errstr_index: usize,
) -> Result<PhpType, CompileError> {
    for (index, param) in [(errno_index, "error_code"), (errstr_index, "error_message")] {
        if let Some(arg) = cx.args.get(index) {
            if !matches!(arg.kind, ExprKind::Variable(_)) {
                return Err(CompileError::new(
                    arg.span,
                    &format!("{}() parameter ${} must be passed a variable", cx.name, param),
                ));
            }
        }
    }
    Ok(cx.checker.normalize_union_type(vec![PhpType::stream_resource(), PhpType::False]))
}
