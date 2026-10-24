//! Purpose:
//! Reads runtime-built fileinfo data URLs without relying on literal-only stream lowering.
//!
//! Called from:
//! - The synthetic finfo file query builder, after filename and context validation.
//!
//! Key details:
//! - Data URLs preserve binary payloads, strict base64 errors and literal plus signs.
//! - Local file URLs use the same descriptor path and native flags as local filenames.

use crate::parser::ast::{BinOp, Stmt};
use crate::synthetic_class::*;

/// Emits the recoverable warning used for malformed or non-local wrapper inputs.
fn invalid(message: &str) -> Vec<Stmt> {
    vec![s_expr(e_call("__elephc_builtin_warning", vec![e_str(message)])), s_return(e_bool(false))]
}

/// Decodes a data URL and delegates its bytes to the object's final buffer implementation.
pub(super) fn data_uri() -> Stmt {
    s_if(e_call("str_starts_with", vec![e_call("strtolower", vec![e_var("filename")]), e_str("data://")]), vec![
        s_assign("_uri_comma", e_call("strpos", vec![e_var("filename"), e_str(",")])),
        s_if(e_binop(e_var("_uri_comma"), BinOp::StrictEq, e_bool(false)), invalid("Invalid data URL: missing comma"), vec![], None),
        s_assign("_uri_meta", e_call("substr", vec![e_var("filename"), e_int(7), e_binop(e_var("_uri_comma"), BinOp::Sub, e_int(7))])),
        s_assign("_uri_payload", e_call("substr", vec![e_var("filename"), e_binop(e_var("_uri_comma"), BinOp::Add, e_int(1))])),
        s_if(e_call("str_ends_with", vec![e_call("strtolower", vec![e_var("_uri_meta")]), e_str(";base64")]), vec![
            s_assign("_uri_data", e_call("base64_decode", vec![e_var("_uri_payload"), e_bool(true)])),
        ], vec![], Some(vec![s_assign("_uri_data", e_call("rawurldecode", vec![e_var("_uri_payload")]))])),
        s_if(e_binop(e_var("_uri_data"), BinOp::StrictEq, e_bool(false)), invalid("Invalid data URL: invalid base64 payload"), vec![], None),
        s_return(e_method_call(e_this(), "__elephc_buffer", vec![e_var("_function"), e_call("substr", vec![e_var("_uri_data"), e_int(0), e_int(7340032)]), e_var("flags")])),
    ], vec![], None)
}

/// Normalizes local file URLs before native descriptor and directory handling.
pub(super) fn file_uri() -> Stmt {
    s_if(e_call("str_starts_with", vec![e_call("strtolower", vec![e_var("filename")]), e_str("file://")]), vec![
        s_assign("filename", e_call("substr", vec![e_var("filename"), e_int(7)])),
        s_if(e_call("str_starts_with", vec![e_call("strtolower", vec![e_var("filename")]), e_str("localhost/")]), vec![
            s_assign("filename", e_call("substr", vec![e_var("filename"), e_int(9)])),
        ], vec![], None),
        s_if(e_not(e_call("str_starts_with", vec![e_var("filename"), e_str("/")])), invalid("Remote host file access is not supported"), vec![], None),
    ], vec![], None)
}
