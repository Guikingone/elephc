//! Purpose:
//! Builds the fileinfo PHP surface as synthetic AST declarations.
//!
//! Called from:
//! - fileinfo_declarations before name resolution.
//!
//! Key details:
//! - Objects exclusively own native handles; finfo_close is a PHP 8 no-op.
//! - Query overrides are temporary. Streams use existing PHP stream wrappers.

use crate::parser::ast::{BinOp, CType, Expr, Program, Stmt, TypeExpr};
use crate::synthetic_class::*;

/// Returns a comparison with strict identity semantics.
fn eq(left: Expr, right: Expr) -> Expr { e_binop(left, BinOp::StrictEq, right) }
/// Returns a concatenation of two PHP strings.
fn concat(left: Expr, right: Expr) -> Expr { e_binop(left, BinOp::Concat, right) }
/// Returns the exact string-or-false PHP result declaration.
fn result_type() -> TypeExpr { t_union(vec![TypeExpr::Str, TypeExpr::False]) }
/// Throws the indicated PHP exception when a condition holds.
fn reject(condition: Expr, class: &str, message: &str) -> Stmt {
    s_if(condition, vec![s_throw(e_new_fq(class, vec![e_str(message)]))], vec![], None)
}
/// Emits a PHP warning and returns false for a recoverable native failure.
fn failure() -> Vec<Stmt> {
    vec![s_expr(e_call("__elephc_builtin_warning", vec![concat(concat(e_var("_function"), e_str("(): ")), e_call("elephc_fileinfo_error", vec![]))])), s_return(e_bool(false))]
}
/// Requires a valid object handle before each operation.
fn require_handle() -> Vec<Stmt> {
    vec![s_assign("_raw", e_this_prop("__elephc_handle")), reject(eq(e_var("_raw"), e_int(0)), "Error", "Invalid finfo object")]
}
/// Validates the nullable resource accepted by fileinfo query methods.
fn context_check() -> Stmt {
    reject(e_binop(e_binop(e_var("context"), BinOp::StrictNotEq, e_null()), BinOp::And,
        e_not(e_call("is_resource", vec![e_var("context")]))), "TypeError", "Fileinfo context must be of type resource or null")
}

/// Emits PHP 8.5 deprecations while leaving older selected profiles unchanged.
fn deprecated(message: &str, supplied_context: Option<i64>) -> Stmt {
    let mut condition = e_binop(e_const("PHP_VERSION_ID"), BinOp::GtEq, e_int(80500));
    if let Some(count) = supplied_context {
        condition = e_binop(condition, BinOp::And, e_binop(e_call("func_num_args", vec![]), BinOp::Gt, e_int(count)));
    }
    s_if(condition, vec![s_expr(e_call("__elephc_builtin_deprecated", vec![e_str(message)]))], vec![], None)
}

/// Builds one query method, preserving bytes and PHP stream-context behavior.
fn query_engine(file: bool) -> crate::synthetic_class::MethodBuilder {
    let (name, parameter) = if file { ("file", "filename") } else { ("buffer", "string") };
    let mut body = require_handle();
    body.push(context_check());
    if file {
        body.extend([
            reject(e_binop(e_binop(e_var("context"), BinOp::StrictNotEq, e_null()), BinOp::And,
                e_binop(e_call("get_resource_type", vec![e_var("context")]), BinOp::StrictNotEq, e_str("stream-context"))), "TypeError", "finfo::file(): context must be a stream context resource"),
            reject(eq(e_var(parameter), e_str("")), "ValueError", "finfo::file(): Argument #1 ($filename) must not be empty"),
            reject(e_binop(e_call("strpos", vec![e_var(parameter), e_str("\0")]), BinOp::StrictNotEq, e_bool(false)), "ValueError", "finfo::file(): Argument #1 ($filename) must not contain any null bytes"),
            super::streams::data_uri(),
            super::streams::file_uri(),
            s_if(e_call("is_dir", vec![e_var(parameter)]), vec![s_return(e_str("directory"))], vec![], None),
            s_if(e_binop(e_call("strpos", vec![e_var(parameter), e_str("://")]), BinOp::StrictNotEq, e_bool(false)), vec![
                s_assign("_stream", e_call("fopen", vec![e_var(parameter), e_str("rb"), e_bool(false), e_var("context")])),
                s_if(eq(e_var("_stream"), e_bool(false)), vec![s_return(e_bool(false))], vec![], None),
                s_assign("_data", e_call("stream_get_contents", vec![e_var("_stream"), e_int(7340032)])),
                s_expr(e_call("fclose", vec![e_var("_stream")])),
                s_if(eq(e_var("_data"), e_bool(false)), vec![s_return(e_bool(false))], vec![], None),
                s_return(e_method_call(e_this(), "__elephc_buffer", vec![e_var("_function"), e_var("_data"), e_var("flags")])),
            ], vec![], None),
            s_assign("_result", e_call("elephc_fileinfo_file", vec![e_var("_raw"), e_var(parameter), e_var("flags")])),
        ]);
    } else {
        body.push(s_assign("_result", e_call("elephc_fileinfo_buffer", vec![e_var("_raw"), e_var(parameter), e_call("strlen", vec![e_var(parameter)]), e_var("flags")])));
    }
    body.push(s_if(eq(e_call("elephc_fileinfo_ok", vec![]), e_int(0)), failure(), vec![], None));
    body.push(s_return(e_var("_result")));
    method(&format!("__elephc_{name}")).final_().param("_function", TypeExpr::Str).param(parameter, TypeExpr::Str)
        .param_default("flags", TypeExpr::Int, e_int(0))
        .param_default("context", t_mixed(), e_null()).returns(result_type()).body(body)
}

/// Public method dispatch remains overridable; procedural calls use the final engine helper.
fn query_method(file: bool) -> crate::synthetic_class::MethodBuilder {
    let (name, parameter) = if file { ("file", "filename") } else { ("buffer", "string") };
    let mut body = Vec::new();
    if !file {
        body.push(deprecated("finfo::buffer(): The $context parameter has no effect for finfo_buffer()", Some(2)));
    }
    body.push(s_return(e_method_call(e_this(), &format!("__elephc_{name}"), vec![
        e_str(&format!("finfo::{name}")), e_var(parameter), e_var("flags"), e_var("context"),
    ])));
    method(name).param(parameter, TypeExpr::Str)
        .param_default("flags", TypeExpr::Int, e_int(0))
        .param_default("context", t_mixed(), e_null()).returns(result_type())
.body(body)
}

/// Builds the owning class, including PHP's clone and serialization restrictions.
fn finfo_class() -> Stmt {
    let mut constructor = vec![
        s_assign("_database", e_null_coalesce(e_var("magic_database"), e_str(""))),
        reject(e_binop(e_call("strpos", vec![e_var("_database"), e_str("\0")]), BinOp::StrictNotEq, e_bool(false)), "ValueError", "finfo::__construct(): Argument #2 ($magic_database) must not contain any null bytes"),
        s_prop_assign(e_this(), "__elephc_handle", e_int(0)),
        s_prop_assign(e_this(), "__elephc_owner", e_null()),
        s_assign("_raw", e_call("elephc_fileinfo_open", vec![e_var("flags"), e_var("_database")])),
    ];
    constructor.push(s_if(eq(e_var("_raw"), e_int(0)), vec![s_throw(e_new_fq("Exception", vec![e_call("elephc_fileinfo_error", vec![])]))], vec![], None));
    constructor.push(s_prop_assign(e_this(), "__elephc_owner", e_new_fq("__ElephcFinfoHandle", vec![e_var("_raw")])));
    constructor.push(s_prop_assign(e_this(), "__elephc_handle", e_var("_raw")));
    let mut set_flags = require_handle();
    set_flags.extend([s_expr(e_call("elephc_fileinfo_set_flags", vec![e_var("_raw"), e_var("flags")])), s_return(e_bool(true))]);
    class("finfo")
        .private_prop("__elephc_handle", TypeExpr::Int, Some(e_int(0)))
        .private_prop("__elephc_owner", t_nullable(t_class("__ElephcFinfoHandle")), Some(e_null()))
        .method(method("__construct").param_default("flags", TypeExpr::Int, e_int(0))
            .param_default("magic_database", t_nullable(TypeExpr::Str), e_null()).body(constructor))
        .method(query_method(false)).method(query_method(true))
        .method(query_engine(false)).method(query_engine(true))
        .method(method("set_flags").param("flags", TypeExpr::Int).returns(TypeExpr::Bool)
            .returning(e_method_call(e_this(), "__elephc_set_flags", vec![e_var("flags")])))
        .method(method("__elephc_set_flags").final_().param("flags", TypeExpr::Int).returns(TypeExpr::Bool).body(set_flags))
        .method(method("__debugInfo").returns(t_array()).returning(e_array(vec![])))
        .method(method("__clone").returns(TypeExpr::Void).body(vec![
            s_prop_assign(e_this(), "__elephc_handle", e_int(0)),
            s_prop_assign(e_this(), "__elephc_owner", e_null()),
            s_throw(e_new_fq("Error", vec![concat(e_str("Trying to clone an uncloneable object of class "), e_call("get_class", vec![e_this()]))])),
        ]))
        .method(method("__serialize").returns(t_array()).body(vec![s_throw(e_new_fq("Exception", vec![e_str("Serialization of 'finfo' is not allowed")]))]))
        .method(method("__unserialize").param("data", t_array()).returns(TypeExpr::Void).body(vec![s_throw(e_new_fq("Exception", vec![e_str("Unserialization of 'finfo' is not allowed")]))]))
        .build()
}

/// Keeps native cleanup independent of a finfo subclass's user-defined destructor.
fn handle_owner() -> Stmt {
    class("__ElephcFinfoHandle").final_()
        .private_prop("handle", TypeExpr::Int, Some(e_int(0)))
        .method(method("__construct").param("handle", TypeExpr::Int).body(vec![s_prop_assign(e_this(), "handle", e_var("handle"))]))
        .method(method("__destruct").body(vec![
            s_assign("_handle", e_this_prop("handle")),
            s_prop_assign(e_this(), "handle", e_int(0)),
            s_expr(e_call("elephc_fileinfo_free", vec![e_var("_handle")])),
        ])).build()
}

/// Builds procedural wrappers with the shared contract's PHP parameter names.
fn functions() -> Program {
    let mut out = vec![
        function("finfo_open").param_default("flags", TypeExpr::Int, e_int(0))
            .param_default("magic_database", t_nullable(TypeExpr::Str), e_null())
            .returns(t_union(vec![t_class("finfo"), TypeExpr::False]))
            .body(vec![s_try(vec![s_return(e_new_fq("finfo", vec![e_var("flags"), e_var("magic_database")]))],
                vec![(vec!["Exception"], Some("_error"), vec![
                    s_expr(e_call("__elephc_builtin_warning", vec![e_method_call(e_var("_error"), "getMessage", vec![])])),
                    s_return(e_bool(false)),
                ])], None)]).build(),
        function("finfo_close").param("finfo", t_class("finfo")).returns(TypeExpr::Bool).body(vec![
            deprecated("Function finfo_close() is deprecated since 8.5, as finfo objects are freed automatically", None),
            s_return(e_bool(true)),
        ]).build(),
        function("finfo_set_flags").param("finfo", t_class("finfo")).param("flags", TypeExpr::Int)
            .returns(TypeExpr::Bool).returning(e_method_call(e_var("finfo"), "__elephc_set_flags", vec![e_var("flags")])).build(),
    ];
    out.push(function("finfo_buffer").param("finfo", t_class("finfo")).param("string", TypeExpr::Str)
        .param_default("flags", TypeExpr::Int, e_int(0)).param_default("context", t_mixed(), e_null())
        .returns(result_type()).body(vec![
            deprecated("finfo_buffer(): The $context parameter has no effect for finfo_buffer()", Some(3)),
            s_return(e_method_call(e_var("finfo"), "__elephc_buffer", vec![e_str("finfo_buffer"), e_var("string"), e_var("flags"), e_var("context")])),
        ]).build());
    out.push(function("finfo_file").param("finfo", t_class("finfo")).param("filename", TypeExpr::Str)
        .param_default("flags", TypeExpr::Int, e_int(0)).param_default("context", t_mixed(), e_null())
        .returns(result_type()).returning(e_method_call(e_var("finfo"), "__elephc_file", vec![e_str("finfo_file"), e_var("filename"), e_var("flags"), e_var("context")])).build());
    out.push(function("mime_content_type").param("filename", t_mixed()).returns(result_type()).body(vec![
        s_assign("_finfo", e_new_fq("finfo", vec![e_const("FILEINFO_MIME_TYPE")])),
        s_if(e_call("is_resource", vec![e_var("filename")]), vec![
            reject(e_binop(e_call("get_resource_type", vec![e_var("filename")]), BinOp::StrictNotEq, e_str("stream")), "TypeError", "mime_content_type(): supplied resource is not a valid stream resource"),
            s_assign("_position", e_call("ftell", vec![e_var("filename")])),
            s_expr(e_call("fseek", vec![e_var("filename"), e_int(0)])),
            s_assign("_data", e_call("stream_get_contents", vec![e_var("filename"), e_int(7340032)])),
            s_expr(e_call("fseek", vec![e_var("filename"), e_var("_position")])),
            s_if(eq(e_var("_data"), e_bool(false)), vec![s_return(e_bool(false))], vec![], None),
            s_return(e_method_call(e_var("_finfo"), "buffer", vec![e_var("_data")])),
        ], vec![], None),
        reject(e_not(e_call("is_string", vec![e_var("filename")])), "TypeError", "mime_content_type(): Argument #1 ($filename) must be of type resource|string"),
        s_return(e_method_call(e_var("_finfo"), "file", vec![e_var("filename")])),
    ]).build());
    out
}

/// Builds externs first, followed by the class and six PHP functions.
pub(super) fn declarations() -> Program {
    internal_declarations(|| {
        let mut out = vec![
            extern_fn("elephc_fileinfo_open", "elephc_fileinfo").param("flags", CType::Int).param("path", CType::Str).returns(CType::Int).build(),
            extern_fn("elephc_fileinfo_free", "elephc_fileinfo").param("handle", CType::Int).returns(CType::Int).build(),
            extern_fn("elephc_fileinfo_set_flags", "elephc_fileinfo").param("handle", CType::Int).param("flags", CType::Int).returns(CType::Int).build(),
            extern_fn("elephc_fileinfo_file", "elephc_fileinfo").param("handle", CType::Int).param("path", CType::Str).param("flags", CType::Int).returns(CType::Str).build(),
            extern_fn("elephc_fileinfo_buffer", "elephc_fileinfo").param("handle", CType::Int).param("data", CType::Str).param("length", CType::Int).param("flags", CType::Int).returns(CType::Str).build(),
            extern_fn("elephc_fileinfo_ok", "elephc_fileinfo").returns(CType::Int).build(),
            extern_fn("elephc_fileinfo_error", "elephc_fileinfo").returns(CType::Str).build(),
            handle_owner(),
            finfo_class(),
        ];
        out.extend(functions());
        out
    })
}
