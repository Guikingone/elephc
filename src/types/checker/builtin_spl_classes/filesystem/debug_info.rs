//! Purpose:
//! Builds the SplFileInfo debug snapshot from its stored filename.
//!
//! Called from:
//! - `super::spl_file_info_methods()`.
//!
//! Key details:
//! - Native path fields bypass overridable getFilename/getPathname methods.
//! - User members retain PHP's visibility-mangled keys and insertion order.

use crate::parser::ast::Stmt;

use super::super::common::*;

/// Copies user members and appends SplFileInfo's canonical private path fields.
pub(super) fn snapshot_body() -> Vec<Stmt> {
    let mut body = super::super::debug_info::member_snapshot(
        "SplFileInfo", super::spl_file_info_properties(),
    );
    body.extend([
        array_assign_stmt("debug", string_expr("\0SplFileInfo\0pathName"), super::file_path_arg_expr()),
        array_assign_stmt("debug", string_expr("\0SplFileInfo\0fileName"),
            function_call("basename", vec![super::file_path_arg_expr()])),
        return_stmt(var_expr("debug")),
    ]);
    body
}
