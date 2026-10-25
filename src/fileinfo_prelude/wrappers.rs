//! Purpose:
//! Reads fileinfo wrapper inputs whose names are known only at runtime.
//!
//! Called from:
//! - The private finfo stream reader, including recursive php://filter resources.
//!
//! Key details:
//! - Known filter names use literal builtin calls, sharing existing EIR filter semantics.
//! - Each filter stage owns a temporary stream and closes it even when a handler throws.
//! - Standard descriptors are borrowed and never closed by a fileinfo query.

use crate::parser::ast::{BinOp, CastType, Expr, Stmt, TypeExpr};
use crate::synthetic_class::*;

/// Compares two PHP values without coercion.
fn eq(left: Expr, right: Expr) -> Expr { e_binop(left, BinOp::StrictEq, right) }

/// Tests a wrapper prefix without changing its filename bytes.
fn prefix(value: &str) -> Expr {
    e_call("str_starts_with", vec![e_call("strtolower", vec![e_var("filename")]), e_str(value)])
}

/// Emits one recoverable stream warning and returns false.
fn invalid(message: &str) -> Vec<Stmt> {
    vec![s_expr(e_call("__elephc_builtin_warning", vec![e_str(message)])), s_return(e_bool(false))]
}

/// Reads and closes an owned stream, including exceptional filter callbacks.
fn read_owned() -> Stmt {
    s_try(vec![s_return(e_call("stream_get_contents", vec![e_var("_stream"), e_int(7340032)]))], vec![],
        Some(vec![s_expr(e_call("fclose", vec![e_var("_stream")]))]))
}

/// Reads a wrapper recursively before applying its read-direction filters.
fn filtered_url() -> Stmt {
    s_if(prefix("php://filter/"), vec![
        s_assign("_split", e_call("strpos", vec![e_var("filename"), e_str("/resource=")])),
        s_if(eq(e_var("_split"), e_bool(false)), invalid("Invalid php://filter URL: missing resource"), vec![], None),
        s_assign("_resource", e_call("substr", vec![e_var("filename"), e_binop(e_var("_split"), BinOp::Add, e_int(10))])),
        s_if(eq(e_var("_resource"), e_str("")), invalid("Invalid php://filter URL: empty resource"), vec![], None),
        s_assign("_data", e_method_call(e_this(), "__elephc_read_stream", vec![e_var("_resource"), e_var("context")])),
        s_if(eq(e_var("_data"), e_bool(false)), vec![s_return(e_bool(false))], vec![], None),
        s_assign("_spec", e_str("")),
        s_if(e_binop(e_var("_split"), BinOp::Gt, e_int(13)), vec![
            s_assign("_spec", e_call("substr", vec![e_var("filename"), e_int(13), e_binop(e_var("_split"), BinOp::Sub, e_int(13))])),
        ], vec![], None),
        s_foreach(e_call("explode", vec![e_str("/"), e_var("_spec")]), None, "_part", vec![
            s_if(e_call("str_starts_with", vec![e_var("_part"), e_str("write=")]), vec![s_continue(1)], vec![], None),
            s_if(e_call("str_starts_with", vec![e_var("_part"), e_str("read=")]), vec![
                s_assign("_part", e_call("substr", vec![e_var("_part"), e_int(5)])),
            ], vec![], None),
            s_foreach(e_call("explode", vec![e_str("|"), e_var("_part")]), None, "_filter", vec![
                s_if(eq(e_var("_filter"), e_str("")), vec![s_continue(1)], vec![], None),
                // Keep the fallible result boxed while the loop's byte accumulator stays string-typed.
                s_assign("_stage", e_array(vec![
                    e_method_call(e_this(), "__elephc_filter_bytes", vec![e_var("_data"), e_var("_filter")]), e_bool(true),
                ])),
                s_if(eq(e_index(e_var("_stage"), e_int(0)), e_bool(false)), vec![s_return(e_bool(false))], vec![], None),
                s_assign("_data", e_cast(CastType::String, e_index(e_var("_stage"), e_int(0)))),
            ]),
        ]),
        s_return(e_var("_data")),
    ], vec![], None)
}

/// Builds the runtime reader used by both object and procedural fileinfo calls.
pub(super) fn read_method() -> MethodBuilder {
    let mut body = vec![super::streams::data_uri(), super::streams::file_uri(),
        super::streams::network_uri(), filtered_url()];
    for scheme in ["compress.zlib://", "compress.bzip2://"] {
        body.push(s_if(prefix(scheme), vec![
            // PHP opens the wrapper but cannot identify it when stream stat is unavailable.
            s_assign("_stream", e_call("fopen", vec![
                e_call("substr", vec![e_var("filename"), e_int(scheme.len() as i64)]), e_str("rb"), e_bool(false), e_var("context")])),
            s_if(eq(e_var("_stream"), e_bool(false)), vec![s_return(e_bool(false))], vec![], None),
            s_expr(e_call("fclose", vec![e_var("_stream")])),
            s_return(e_bool(false)),
        ], vec![], None));
    }
    for (url, resource) in [("php://stdin", "STDIN"), ("php://input", "STDIN"),
        ("php://stdout", "STDOUT"), ("php://output", "STDOUT"), ("php://stderr", "STDERR")] {
        body.push(s_if(eq(e_call("strtolower", vec![e_var("filename")]), e_str(url)), vec![
            s_return(e_call("stream_get_contents", vec![e_const(resource), e_int(7340032)])),
        ], vec![], None));
    }
    body.push(s_if(e_binop(eq(e_call("strtolower", vec![e_var("filename")]), e_str("php://memory")), BinOp::Or,
        e_binop(eq(e_call("strtolower", vec![e_var("filename")]), e_str("php://temp")), BinOp::Or, prefix("php://temp/"))),
        vec![s_return(e_str(""))], vec![], None));
    body.extend([
        s_assign("_stream", e_call("fopen", vec![e_var("filename"), e_str("rb"), e_bool(false), e_var("context")])),
        s_if(eq(e_var("_stream"), e_bool(false)), vec![s_return(e_bool(false))], vec![], None),
        read_owned(),
    ]);
    method("__elephc_read_stream").private().param("filename", TypeExpr::Str)
        .param("context", t_mixed()).returns(t_union(vec![TypeExpr::Str, TypeExpr::False])).body(body)
}

/// Applies one filter through the existing stream implementation and its PHP callbacks.
pub(super) fn filter_method() -> MethodBuilder {
    let mut attach = vec![s_assign("_attached", e_call("stream_filter_append", vec![e_var("_stream"), e_var("name"), e_const("STREAM_FILTER_READ")]))];
    for name in ["string.toupper", "string.tolower", "string.rot13", "string.strip_tags", "dechunk",
        "convert.base64-encode", "convert.base64-decode", "convert.quoted-printable-encode",
        "convert.quoted-printable-decode", "bzip2.decompress"].into_iter().rev() {
        attach = vec![s_if(eq(e_var("name"), e_str(name)), vec![
            s_assign("_attached", e_call("stream_filter_append", vec![e_var("_stream"), e_str(name), e_const("STREAM_FILTER_READ")])),
        ], vec![], Some(attach))];
    }
    let mut work = vec![s_expr(e_call("fwrite", vec![e_var("_stream"), e_var("data")])),
        s_expr(e_call("rewind", vec![e_var("_stream")]))];
    work.extend(attach);
    work.push(s_if(eq(e_var("_attached"), e_bool(false)), vec![
        s_expr(e_call("__elephc_builtin_warning", vec![e_binop(e_str("Unable to create filter: "), BinOp::Concat, e_var("name"))])),
        // PHP warns about an unknown filter and leaves its input bytes unchanged.
        s_return(e_var("data")),
    ], vec![], None));
    work.push(s_return(e_call("stream_get_contents", vec![e_var("_stream"), e_int(7340032)])));
    method("__elephc_filter_bytes").private().param("data", TypeExpr::Str).param("name", TypeExpr::Str)
        .returns(t_union(vec![TypeExpr::Str, TypeExpr::False])).body(vec![
            // PHP's filter default is raw DEFLATE, unlike the auto-detecting fopen helper.
            s_if(eq(e_var("name"), e_str("zlib.inflate")), vec![s_return(e_call("gzinflate", vec![e_var("data")]))], vec![], None),
            s_if(eq(e_var("name"), e_str("zlib.deflate")), vec![s_return(e_call("gzdeflate", vec![e_var("data")]))], vec![], None),
            s_assign("_stream", e_call("fopen", vec![e_str("php://temp"), e_str("w+b")])),
            s_if(eq(e_var("_stream"), e_bool(false)), vec![s_return(e_bool(false))], vec![], None),
            s_try(work, vec![], Some(vec![s_expr(e_call("fclose", vec![e_var("_stream")]))])),
        ])
}
