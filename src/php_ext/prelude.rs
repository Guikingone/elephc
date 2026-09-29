//! Purpose:
//! Declares hosted extensions to the compiler: every function an extension
//! registers becomes an ordinary PHP function whose body calls it through the
//! engine, and its exception classes and constants are declared alongside.
//!
//! Called from:
//! - `crate::pipeline`, before name resolution, when the program's project
//!   declares `[php-ext]` extensions.
//!
//! Key details:
//! - Built as AST, never parsed from PHP source.
//! - The engine is reached only through `extern` declarations with no library
//!   (the symbols come from archives the link plan carries) and the zval
//!   bridge builtins, so this adds no assembly and runs on every target the FFI
//!   already supports.
//! - Signatures come from the extension's own arginfo. An optional parameter
//!   whose default is a literal (or a constant) is declared with that default
//!   and always passed, which an internal function cannot tell apart from an
//!   omitted argument. One whose default only the C code knows (`UNKNOWN`, or
//!   arginfo that states none, as simdjson's does) cannot be declared that way:
//!   from there on the remaining parameters become a variadic tail, so exactly
//!   the arguments the caller wrote are passed.
//! - Results are rebuilt by walking the engine's value (`__elephc_php_ext_value`)
//!   with indexed accessors, never with `foreach ($array as $k => $v)`: a
//!   string-keyed rebuild through a fresh `[]` is miscompiled on the current
//!   backend (it keeps the list layout statically while the runtime promotes it
//!   to a hash), so the walk writes string keys only, through `$map[$key]`.
//! - Object-free subtrees go through `zval_unpack` in one call; only stdClass
//!   objects (and the arrays holding them) are rebuilt piece by piece.

use std::collections::BTreeSet;

use crate::parser::ast::{BinOp, CType, CastType, Expr, Program, Stmt, TypeExpr};
use crate::synthetic_class::{
    e_array, e_assign, e_binop, e_bool, e_call, e_cast, e_const, e_dyn_prop, e_float, e_int, e_new,
    e_null, e_str, e_var, extern_fn_unbound, function, internal_declarations, s_array_assign,
    s_array_push, s_assign, s_const, s_expr, s_for, s_foreach, s_if, s_namespace, s_return,
    s_return_void, s_static, s_throw, t_array, t_mixed, t_ptr,
};

use super::install::{HostedExtensions, InstalledExtension};
use super::surface::{ConstantValue, SurfaceFunction, SurfaceParam};

/// The inventory group the declarations are recorded under, for pruning.
pub const PRELUDE_GROUP: &str = "php_ext";

/// Exception classes the engine registers and Elephc also declares, so an
/// extension class may extend them and a thrown one can be rebuilt. The
/// engine's `ErrorException`, `CompileError` and `ParseError` are absent: Elephc
/// declares none of them, so an exception of one of those classes is rebuilt as
/// its nearest declared ancestor.
const ENGINE_THROWABLES: &[&str] = &[
    "Exception", "Error", "TypeError",
    "ArgumentCountError", "ValueError", "ArithmeticError", "DivisionByZeroError",
    "UnhandledMatchError", "LogicException", "BadFunctionCallException", "BadMethodCallException",
    "DomainException", "InvalidArgumentException", "LengthException", "OutOfRangeException",
    "RuntimeException", "OutOfBoundsException", "OverflowException", "RangeException",
    "UnderflowException", "UnexpectedValueException",
];

/// Constants a default may name that Elephc itself defines.
const KNOWN_CONSTANTS: &[&str] = &[
    "PHP_INT_MAX", "PHP_INT_MIN", "PHP_INT_SIZE", "PHP_FLOAT_EPSILON", "PHP_FLOAT_MAX",
    "PHP_FLOAT_MIN", "PHP_EOL", "E_ALL", "E_ERROR", "E_WARNING", "E_NOTICE", "E_DEPRECATED",
    "M_PI", "INF", "NAN",
];

/// Builds every declaration for the hosted extensions.
pub fn declarations(hosted: &HostedExtensions) -> Program {
    internal_declarations(|| {
        let mut program = abi_externs();
        for extension in &hosted.extensions {
            program.push(
                extern_fn_unbound(&extension.record.module_accessor())
                    .returns(CType::Ptr)
                    .build(),
            );
        }
        let throwables = declared_throwables(hosted);
        program.push(setup_helper(&hosted.ini));
        program.extend(transfer_helpers(&throwables));
        for extension in &hosted.extensions {
            program.extend(class_declarations(extension));
            program.extend(constant_declarations(extension));
            for function in &extension.surface.functions {
                program.push(wrap_in_namespace(&function.name, |local| wrapper(extension, function, local, !hosted.ini.is_empty())));
            }
        }
        program
    })
}

/// The engine's C ABI (`recipes/php_src/host.c`).
fn abi_externs() -> Vec<Stmt> {
    let ptr = || CType::Ptr;
    vec![
        extern_fn_unbound("elephc_php_ext_ini").param("name", CType::Str).param("value", CType::Str).build(),
        extern_fn_unbound("elephc_php_ext_call_new")
            .param("module", ptr())
            .param("name", CType::Str)
            .param("argc", CType::Int)
            .returns(ptr())
            .build(),
        extern_fn_unbound("elephc_php_ext_call_arg")
            .param("call", ptr())
            .param("index", CType::Int)
            .param("value", ptr())
            .param("by_ref", CType::Int)
            .returns(CType::Int)
            .build(),
        extern_fn_unbound("elephc_php_ext_call_invoke").param("call", ptr()).returns(CType::Int).build(),
        extern_fn_unbound("elephc_php_ext_call_result").param("call", ptr()).returns(ptr()).build(),
        extern_fn_unbound("elephc_php_ext_call_ref").param("call", ptr()).param("index", CType::Int).returns(ptr()).build(),
        extern_fn_unbound("elephc_php_ext_call_error_class").param("call", ptr()).returns(CType::Str).build(),
        extern_fn_unbound("elephc_php_ext_call_error_message").param("call", ptr()).returns(CType::Str).build(),
        extern_fn_unbound("elephc_php_ext_call_error_code").param("call", ptr()).returns(CType::Int).build(),
        extern_fn_unbound("elephc_php_ext_call_free").param("call", ptr()).build(),
        extern_fn_unbound("elephc_php_ext_value_kind").param("value", ptr()).returns(CType::Int).build(),
        extern_fn_unbound("elephc_php_ext_value_type_name").param("value", ptr()).returns(CType::Str).build(),
        extern_fn_unbound("elephc_php_ext_value_export").param("call", ptr()).param("value", ptr()).returns(ptr()).build(),
        extern_fn_unbound("elephc_php_ext_value_count").param("value", ptr()).returns(CType::Int).build(),
        extern_fn_unbound("elephc_php_ext_value_is_list").param("value", ptr()).returns(CType::Int).build(),
        extern_fn_unbound("elephc_php_ext_value_key").param("value", ptr()).param("index", CType::Int).returns(CType::Str).build(),
        extern_fn_unbound("elephc_php_ext_value_at").param("value", ptr()).param("index", CType::Int).returns(ptr()).build(),
    ]
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    e_call(name, args)
}

fn strict_eq(left: Expr, right: Expr) -> Expr {
    e_binop(left, BinOp::StrictEq, right)
}

/// `__elephc_php_ext_setup()`: applies `[php-ext.ini]` once, before any
/// extension starts.
fn setup_helper(ini: &[(String, String)]) -> Stmt {
    let mut body = vec![
        s_static("__elephc_done", e_bool(false)),
        s_if(e_var("__elephc_done"), vec![s_return_void()], vec![], None),
        s_assign("__elephc_done", e_bool(true)),
    ];
    for (directive, value) in ini {
        body.push(s_expr(call("elephc_php_ext_ini", vec![e_str(directive), e_str(value)])));
    }
    function("__elephc_php_ext_setup").returns(TypeExpr::Void).body(body).build()
}

/// The helpers every wrapper shares: argument transfer, invocation with
/// exception rebuilding, and result walking.
fn transfer_helpers(throwables: &[String]) -> Vec<Stmt> {
    let call_var = || e_var("__elephc_call");
    let value_var = || e_var("__elephc_value");

    // function __elephc_php_ext_arg(ptr $call, int $index, mixed $value, int $byRef): void
    let arg = function("__elephc_php_ext_arg")
        .param("__elephc_call", t_ptr())
        .param("__elephc_index", TypeExpr::Int)
        .param("__elephc_value", t_mixed())
        .param("__elephc_by_ref", TypeExpr::Int)
        .returns(TypeExpr::Void)
        .body(vec![
            s_assign("__elephc_zval", call("zval_pack", vec![value_var()])),
            s_assign(
                "__elephc_status",
                call(
                    "elephc_php_ext_call_arg",
                    vec![call_var(), e_var("__elephc_index"), e_var("__elephc_zval"), e_var("__elephc_by_ref")],
                ),
            ),
            s_expr(call("zval_free", vec![e_var("__elephc_zval")])),
            s_if(
                e_binop(e_var("__elephc_status"), BinOp::StrictNotEq, e_int(0)),
                vec![
                    s_assign("__elephc_message", call("elephc_php_ext_call_error_message", vec![call_var()])),
                    s_expr(call("elephc_php_ext_call_free", vec![call_var()])),
                    s_throw(e_new("\\TypeError", vec![e_var("__elephc_message")])),
                ],
                vec![],
                None,
            ),
        ])
        .build();

    // function __elephc_php_ext_throw(string $chain, string $message, int $code): void
    // `$chain` is the thrown class and its ancestors, most derived first: the
    // first one declared here is rethrown.
    let mut arms: Vec<(Expr, Vec<Stmt>)> = throwables
        .iter()
        .map(|class_name| {
            (
                strict_eq(e_var("__elephc_class"), e_str(class_name)),
                vec![s_throw(e_new(
                    &format!("\\{class_name}"),
                    vec![e_var("__elephc_message"), e_var("__elephc_code")],
                ))],
            )
        })
        .collect();
    let mut throw_body = Vec::new();
    if !arms.is_empty() {
        let (first_condition, first_body) = arms.remove(0);
        throw_body.push(s_foreach(
            call("explode", vec![e_str(","), e_var("__elephc_chain")]),
            None,
            "__elephc_class",
            vec![s_if(first_condition, first_body, arms, None)],
        ));
    }
    throw_body.push(s_throw(e_new("\\Error", vec![e_var("__elephc_message"), e_var("__elephc_code")])));
    let throw = function("__elephc_php_ext_throw")
        .param("__elephc_chain", TypeExpr::Str)
        .param("__elephc_message", TypeExpr::Str)
        .param("__elephc_code", TypeExpr::Int)
        .returns(TypeExpr::Void)
        .body(throw_body)
        .build();

    // function __elephc_php_ext_invoke(ptr $call): void
    let invoke = function("__elephc_php_ext_invoke")
        .param("__elephc_call", t_ptr())
        .returns(TypeExpr::Void)
        .body(vec![
            s_assign("__elephc_status", call("elephc_php_ext_call_invoke", vec![call_var()])),
            s_if(strict_eq(e_var("__elephc_status"), e_int(0)), vec![s_return_void()], vec![], None),
            // A fatal error was already reported by the engine, as PHP reports it.
            s_if(
                strict_eq(e_var("__elephc_status"), e_int(2)),
                vec![
                    s_expr(call("elephc_php_ext_call_free", vec![call_var()])),
                    s_expr(call("exit", vec![e_int(255)])),
                ],
                vec![],
                None,
            ),
            s_assign("__elephc_class", call("elephc_php_ext_call_error_class", vec![call_var()])),
            s_assign("__elephc_message", call("elephc_php_ext_call_error_message", vec![call_var()])),
            s_assign("__elephc_code", call("elephc_php_ext_call_error_code", vec![call_var()])),
            s_expr(call("elephc_php_ext_call_free", vec![call_var()])),
            s_expr(call(
                "__elephc_php_ext_throw",
                vec![e_var("__elephc_class"), e_var("__elephc_message"), e_var("__elephc_code")],
            )),
        ])
        .build();

    // function __elephc_php_ext_value(ptr $call, ptr $value): mixed
    let at = || call("elephc_php_ext_value_at", vec![value_var(), e_var("__elephc_i")]);
    let recurse = || call("__elephc_php_ext_value", vec![call_var(), at()]);
    let counted_loop = |body: Vec<Stmt>| {
        s_for(
            Some(s_assign("__elephc_i", e_int(0))),
            Some(e_binop(e_var("__elephc_i"), BinOp::Lt, e_var("__elephc_count"))),
            Some(s_assign("__elephc_i", e_binop(e_var("__elephc_i"), BinOp::Add, e_int(1)))),
            body,
        )
    };
    let value = function("__elephc_php_ext_value")
        .param("__elephc_call", t_ptr())
        .param("__elephc_value", t_ptr())
        .returns(t_mixed())
        .body(vec![
            s_assign("__elephc_kind", call("elephc_php_ext_value_kind", vec![value_var()])),
            s_if(
                strict_eq(e_var("__elephc_kind"), e_int(0)),
                vec![s_return(call(
                    "zval_unpack",
                    vec![call("elephc_php_ext_value_export", vec![call_var(), value_var()])],
                ))],
                vec![],
                None,
            ),
            s_if(
                strict_eq(e_var("__elephc_kind"), e_int(3)),
                vec![
                    s_assign("__elephc_type", call("elephc_php_ext_value_type_name", vec![value_var()])),
                    s_throw(e_new(
                        "\\Error",
                        vec![e_binop(
                            e_binop(
                                e_str("a hosted PHP extension returned a value of type "),
                                BinOp::Concat,
                                e_var("__elephc_type"),
                            ),
                            BinOp::Concat,
                            e_str(", which Elephc cannot represent yet"),
                        )],
                    )),
                ],
                vec![],
                None,
            ),
            s_assign("__elephc_count", call("elephc_php_ext_value_count", vec![value_var()])),
            s_if(
                strict_eq(e_var("__elephc_kind"), e_int(1)),
                vec![
                    s_assign("__elephc_object", e_new("\\stdClass", vec![])),
                    counted_loop(vec![
                        s_assign("__elephc_name", call("elephc_php_ext_value_key", vec![value_var(), e_var("__elephc_i")])),
                        s_expr(e_assign(
                            e_dyn_prop(e_var("__elephc_object"), e_var("__elephc_name")),
                            recurse(),
                        )),
                    ]),
                    s_return(e_var("__elephc_object")),
                ],
                vec![],
                None,
            ),
            s_if(
                strict_eq(call("elephc_php_ext_value_is_list", vec![value_var()]), e_int(1)),
                vec![
                    s_assign("__elephc_list", e_array(vec![])),
                    counted_loop(vec![s_array_push("__elephc_list", recurse())]),
                    s_return(e_var("__elephc_list")),
                ],
                vec![],
                None,
            ),
            s_assign("__elephc_map", e_array(vec![])),
            counted_loop(vec![
                s_assign("__elephc_key", call("elephc_php_ext_value_key", vec![value_var(), e_var("__elephc_i")])),
                s_array_assign("__elephc_map", e_var("__elephc_key"), recurse()),
            ]),
            s_return(e_var("__elephc_map")),
        ])
        .build();

    // function __elephc_php_ext_result(ptr $call): mixed
    let result = function("__elephc_php_ext_result")
        .param("__elephc_call", t_ptr())
        .returns(t_mixed())
        .body(vec![
            s_assign(
                "__elephc_result",
                call("__elephc_php_ext_value", vec![call_var(), call("elephc_php_ext_call_result", vec![call_var()])]),
            ),
            s_expr(call("elephc_php_ext_call_free", vec![call_var()])),
            s_return(e_var("__elephc_result")),
        ])
        .build();

    vec![arg, throw, invoke, value, result]
}

/// Every throwable a wrapper may have to rebuild: the engine's own and each
/// hosted extension's exception classes, subclasses before their parents so
/// the first matching arm is the exact class.
fn declared_throwables(hosted: &HostedExtensions) -> Vec<String> {
    let mut classes: Vec<String> = Vec::new();
    for extension in &hosted.extensions {
        for class_name in exception_classes(extension) {
            classes.push(class_name);
        }
    }
    classes.extend(ENGINE_THROWABLES.iter().map(|name| name.to_string()));
    classes
}

/// The extension classes Elephc can declare: exception classes whose parent
/// is a known throwable or another such class, with no methods of their own.
fn exception_classes(extension: &InstalledExtension) -> Vec<String> {
    let mut known: BTreeSet<String> = ENGINE_THROWABLES.iter().map(|name| name.to_string()).collect();
    let mut ordered = Vec::new();
    loop {
        let before = ordered.len();
        for class in &extension.surface.classes {
            let Some(parent) = &class.parent else { continue };
            if class.interface || !class.methods.is_empty() || known.contains(&class.name) {
                continue;
            }
            if known.contains(parent) {
                known.insert(class.name.clone());
                ordered.push(class.name.clone());
            }
        }
        if ordered.len() == before {
            break;
        }
    }
    // Deepest first, so a catch-all parent arm never shadows its subclass.
    ordered.reverse();
    ordered
}

fn class_declarations(extension: &InstalledExtension) -> Vec<Stmt> {
    let mut declared = exception_classes(extension);
    declared.reverse(); // parents before children
    declared
        .iter()
        .filter_map(|name| extension.surface.classes.iter().find(|class| &class.name == name))
        .map(|class| {
            let parent = format!("\\{}", class.parent.as_deref().unwrap_or("Exception"));
            wrap_in_namespace(&class.name, |local| {
                let builder = crate::synthetic_class::class(local).extends(&parent);
                if class.is_final { builder.final_().build() } else { builder.build() }
            })
        })
        .collect()
}

fn constant_declarations(extension: &InstalledExtension) -> Vec<Stmt> {
    extension
        .surface
        .constants
        .iter()
        .filter_map(|constant| {
            let value = constant_expr(&constant.value)?;
            Some(wrap_in_namespace(&constant.name, |local| s_const(local, value)))
        })
        .collect()
}

fn constant_expr(value: &ConstantValue) -> Option<Expr> {
    match value {
        ConstantValue::Null => Some(e_null()),
        ConstantValue::Bool { value } => Some(e_bool(*value)),
        ConstantValue::Int { value } => Some(e_int(*value)),
        ConstantValue::Float { value } => value.parse::<f64>().ok().map(e_float),
        ConstantValue::String { value } => Some(e_str(value)),
        ConstantValue::Unsupported => None,
    }
}

/// Emits `build(local_name)` inside `namespace Ns { … }` when `qualified` is
/// namespaced, so a hosted `Ns\f` is declared where PHP code looks for it.
fn wrap_in_namespace(qualified: &str, build: impl FnOnce(&str) -> Stmt) -> Stmt {
    match qualified.rsplit_once('\\') {
        Some((namespace, local)) => s_namespace(namespace, vec![build(local)]),
        None => build(qualified),
    }
}

/// Maps an arginfo type to a declarable Elephc type. `None` means the type
/// needs a value Elephc cannot pass to an extension yet (an object, a callable).
fn param_type(ty: Option<&str>) -> Option<Option<TypeExpr>> {
    let Some(ty) = ty else { return Some(None) };
    let simple = |name: &str| -> Option<TypeExpr> {
        match name {
            "int" => Some(TypeExpr::Int),
            "float" => Some(TypeExpr::Float),
            "string" => Some(TypeExpr::Str),
            "bool" => Some(TypeExpr::Bool),
            "array" => Some(t_array()),
            "mixed" => Some(t_mixed()),
            _ => None,
        }
    };
    if let Some(declared) = simple(ty) {
        return Some(Some(declared));
    }
    if let Some(inner) = ty.strip_prefix('?') {
        return simple(inner).map(|declared| Some(TypeExpr::Nullable(Box::new(declared))));
    }
    let scalar_members = ["int", "float", "string", "bool", "array", "null", "false", "true", "mixed"];
    if ty.split('|').all(|member| scalar_members.contains(&member)) {
        // A scalar union: the extension's own argument parsing checks it.
        return Some(Some(t_mixed()));
    }
    None
}

/// The Elephc return type and the cast applied to the walked result.
fn return_type(ty: Option<&str>) -> (TypeExpr, Option<CastType>) {
    match ty {
        Some("int") => (TypeExpr::Int, Some(CastType::Int)),
        Some("float") => (TypeExpr::Float, Some(CastType::Float)),
        Some("string") => (TypeExpr::Str, Some(CastType::String)),
        Some("bool") => (TypeExpr::Bool, Some(CastType::Bool)),
        Some("array") => (t_array(), Some(CastType::Array)),
        Some("void") => (TypeExpr::Void, None),
        _ => (t_mixed(), None),
    }
}

/// A default Elephc can declare, from arginfo's source text.
fn default_expr(text: &str, extension: &InstalledExtension) -> Option<Expr> {
    let text = text.trim();
    match text {
        "null" | "NULL" => return Some(e_null()),
        "true" => return Some(e_bool(true)),
        "false" => return Some(e_bool(false)),
        "[]" => return Some(e_array(vec![])),
        _ => {}
    }
    if let Ok(value) = text.parse::<i64>() {
        return Some(e_int(value));
    }
    if text.contains(['.', 'e', 'E']) && !text.starts_with(['"', '\'']) {
        if let Ok(value) = text.parse::<f64>() {
            return Some(e_float(value));
        }
    }
    for quote in ['"', '\''] {
        if let Some(inner) = text.strip_prefix(quote).and_then(|rest| rest.strip_suffix(quote)) {
            if !inner.contains('\\') && !inner.contains('$') {
                return Some(e_str(inner));
            }
            return None;
        }
    }
    if let Some(constant) = extension.surface.constants.iter().find(|constant| constant.name == text) {
        return constant_expr(&constant.value);
    }
    if KNOWN_CONSTANTS.contains(&text) {
        return Some(e_const(text));
    }
    None
}

/// How one wrapper passes its parameters.
struct Plan {
    /// Declared parameters that are always passed, in order.
    fixed: Vec<PlannedParam>,
    /// Name of the variadic tail, when optional arguments (or the extension's
    /// own variadic parameter) are passed through it.
    tail: Option<String>,
}

struct PlannedParam {
    name: String,
    ty: Option<TypeExpr>,
    default: Option<Expr>,
    by_ref: bool,
}

fn local_name(param: &SurfaceParam) -> String {
    // Arginfo names are PHP identifiers already; prefixing would change named
    // arguments, so they are kept, and the wrapper's own locals are prefixed.
    param.name.clone()
}

/// Plans a wrapper, or explains why the function cannot be called yet.
fn plan(extension: &InstalledExtension, function: &SurfaceFunction) -> Result<Plan, String> {
    let mut fixed = Vec::new();
    let mut tail = None;
    for (index, param) in function.params.iter().enumerate() {
        let ty = param_type(param.ty.as_deref())
            .ok_or_else(|| format!("parameter ${} takes {}, which Elephc cannot pass to an extension yet", param.name, param.ty.as_deref().unwrap_or("?")))?;
        if param.variadic {
            if param.by_ref {
                return Err(format!("parameter ${} is a by-reference variadic", param.name));
            }
            tail = Some(local_name(param));
            break;
        }
        let required = (index as u32) < function.required;
        if required {
            fixed.push(PlannedParam { name: local_name(param), ty, default: None, by_ref: param.by_ref });
            continue;
        }
        match param.default.as_deref().and_then(|text| default_expr(text, extension)) {
            Some(default) => fixed.push(PlannedParam { name: local_name(param), ty, default: Some(default), by_ref: param.by_ref }),
            None => {
                if function.params[index..].iter().any(|rest| rest.by_ref) {
                    return Err(format!(
                        "parameter ${} is by-reference with a default only the C code knows",
                        param.name
                    ));
                }
                tail = Some("__elephc_optional".to_string());
                break;
            }
        }
    }
    Ok(Plan { fixed, tail })
}

/// The wrapper for one hosted function.
fn wrapper(extension: &InstalledExtension, function: &SurfaceFunction, local: &str, has_ini: bool) -> Stmt {
    let (declared_return, cast) = return_type(function.returns.as_deref());
    let plan = match plan(extension, function) {
        Ok(plan) => plan,
        Err(reason) => return unsupported_wrapper(function, local, &reason),
    };
    let mut builder = crate::synthetic_class::function(local);
    for param in &plan.fixed {
        builder = match (param.by_ref, &param.default) {
            (true, Some(default)) => builder.param_by_ref_default(&param.name, param.ty.clone(), default.clone()),
            (true, None) => builder.param_by_ref(&param.name, param.ty.clone()),
            (false, Some(default)) => match &param.ty {
                Some(ty) => builder.param_default(&param.name, ty.clone(), default.clone()),
                None => builder.param_untyped_default(&param.name, default.clone()),
            },
            (false, None) => match &param.ty {
                Some(ty) => builder.param(&param.name, ty.clone()),
                None => builder.param_untyped(&param.name),
            },
        };
    }
    if let Some(tail) = &plan.tail {
        builder = builder.variadic(tail, Some(t_mixed()));
    }
    builder = builder.returns(declared_return.clone());

    let call_var = || e_var("__elephc_call");
    let fixed_count = plan.fixed.len() as i64;
    let argc = match &plan.tail {
        Some(tail) => e_binop(e_int(fixed_count), BinOp::Add, call("count", vec![e_var(tail)])),
        None => e_int(fixed_count),
    };
    let mut body = Vec::new();
    if has_ini {
        body.push(s_expr(call("__elephc_php_ext_setup", vec![])));
    }
    body.push(s_assign(
        "__elephc_call",
        call(
            "elephc_php_ext_call_new",
            vec![call(&extension.record.module_accessor(), vec![]), e_str(&function.name), argc],
        ),
    ));
    // The engine already reported why (a module that failed to start, a
    // function it does not export), as PHP would for a fatal.
    body.push(s_if(call("ptr_is_null", vec![call_var()]), vec![s_expr(call("exit", vec![e_int(255)]))], vec![], None));
    for (index, param) in plan.fixed.iter().enumerate() {
        body.push(s_expr(call(
            "__elephc_php_ext_arg",
            vec![call_var(), e_int(index as i64), e_var(&param.name), e_int(i64::from(param.by_ref))],
        )));
    }
    if let Some(tail) = &plan.tail {
        body.push(s_assign("__elephc_index", e_int(fixed_count)));
        body.push(s_foreach(
            e_var(tail),
            None,
            "__elephc_extra",
            vec![
                s_expr(call(
                    "__elephc_php_ext_arg",
                    vec![call_var(), e_var("__elephc_index"), e_var("__elephc_extra"), e_int(0)],
                )),
                s_assign("__elephc_index", e_binop(e_var("__elephc_index"), BinOp::Add, e_int(1))),
            ],
        ));
    }
    body.push(s_expr(call("__elephc_php_ext_invoke", vec![call_var()])));
    for (index, param) in plan.fixed.iter().enumerate() {
        if param.by_ref {
            body.push(s_assign(
                &param.name,
                call(
                    "__elephc_php_ext_value",
                    vec![call_var(), call("elephc_php_ext_call_ref", vec![call_var(), e_int(index as i64)])],
                ),
            ));
        }
    }
    let result = call("__elephc_php_ext_result", vec![call_var()]);
    if declared_return == TypeExpr::Void {
        body.push(s_expr(result));
    } else {
        body.push(s_return(match cast {
            Some(cast) => e_cast(cast, result),
            None => result,
        }));
    }
    builder.body(body).build()
}

/// A declaration for a function whose parameters cannot be passed yet. It is
/// still declared, so calling it fails with the reason instead of an
/// "undefined function".
fn unsupported_wrapper(function: &SurfaceFunction, local: &str, reason: &str) -> Stmt {
    crate::synthetic_class::function(local)
        .variadic("__elephc_arguments", Some(t_mixed()))
        .returns(t_mixed())
        .body(vec![s_throw(e_new(
            "\\Error",
            vec![e_str(&format!("{}() cannot be called from Elephc yet: {reason}", function.name))],
        ))])
        .build()
}

/// Names the functions whose wrappers only raise, for `php-ext add` to report.
pub fn unsupported_functions(extension: &InstalledExtension) -> Vec<(String, String)> {
    extension
        .surface
        .functions
        .iter()
        .filter_map(|function| plan(extension, function).err().map(|reason| (function.name.clone(), reason)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::StmtKind;
    use crate::php_ext::build::BuildRecord;
    use crate::php_ext::surface::ExtensionSurface;
    use std::path::PathBuf;

    fn installed(surface_json: &str) -> InstalledExtension {
        InstalledExtension {
            name: "demo".into(),
            artifact_dir: PathBuf::from("/x"),
            record: BuildRecord { extension: "demo".into(), archive: "lib/libelephc_ext_demo.a".into(), cxx: false, libraries: vec![] },
            surface: ExtensionSurface::from_json(surface_json).expect("valid surface"),
        }
    }

    const SURFACE: &str = r#"{"module":"demo","version":"1.0.0","functions":[
        {"name":"demo_decode","required":1,"returns":null,"params":[
            {"name":"json","type":"string","default":null},
            {"name":"assoc","type":"bool","default":null},
            {"name":"depth","type":"int","default":null}]},
        {"name":"demo_add","required":1,"returns":"int","params":[
            {"name":"a","type":"int","default":null},
            {"name":"b","type":"int","default":"DEMO_STEP"}]},
        {"name":"demo_inc","required":1,"returns":"int|false","params":[
            {"name":"key","type":"string","default":null},
            {"name":"success","type":null,"by_ref":true,"default":"null"}]},
        {"name":"demo_map","required":2,"returns":"array","params":[
            {"name":"callback","type":"callable","default":null},
            {"name":"items","type":"array","default":null}]},
        {"name":"demo_sum","required":0,"returns":"int","params":[
            {"name":"values","type":"int","variadic":true,"default":null}]}],
        "classes":[
            {"name":"DemoException","parent":"RuntimeException","methods":[]},
            {"name":"DemoParseException","parent":"DemoException","methods":[]},
            {"name":"DemoThing","parent":null,"methods":[{"name":"go","required":0,"returns":null,"params":[]}]}],
        "constants":[{"name":"DEMO_STEP","value":{"type":"int","value":3}},
                     {"name":"DEMO_LIST","value":{"type":"array"}}],
        "ini":[]}"#;

    fn function<'a>(program: &'a Program, name: &str) -> &'a StmtKind {
        &program
            .iter()
            .find(|stmt| matches!(&stmt.kind, StmtKind::FunctionDecl { name: n, .. } if n == name))
            .unwrap_or_else(|| panic!("{name} is declared"))
            .kind
    }

    fn hosted() -> HostedExtensions {
        HostedExtensions { extensions: vec![installed(SURFACE)], ini: vec![] }
    }

    /// simdjson's shape: optional parameters with no stated default must be
    /// passed only when the caller passes them, so they become a variadic tail.
    #[test]
    fn unknown_defaults_become_a_variadic_tail() {
        let program = declarations(&hosted());
        let StmtKind::FunctionDecl { params, variadic, return_type, .. } = function(&program, "demo_decode") else { unreachable!() };
        assert_eq!(params.len(), 1, "only the required parameter is declared");
        assert_eq!(params[0].0, "json");
        assert_eq!(variadic.as_deref(), Some("__elephc_optional"));
        assert_eq!(return_type.as_ref(), Some(&t_mixed()), "no declared return type means mixed");
    }

    /// A default naming the extension's own constant is inlined; a declared
    /// scalar return type is declared and cast.
    #[test]
    fn known_defaults_are_declared_and_always_passed() {
        let program = declarations(&hosted());
        let StmtKind::FunctionDecl { params, variadic, return_type, .. } = function(&program, "demo_add") else { unreachable!() };
        assert_eq!(params.len(), 2);
        assert!(params[1].2.is_some(), "DEMO_STEP inlined as the default");
        assert!(variadic.is_none());
        assert_eq!(return_type.as_ref(), Some(&TypeExpr::Int));
    }

    #[test]
    fn by_reference_parameters_are_declared_by_reference() {
        let program = declarations(&hosted());
        let StmtKind::FunctionDecl { params, .. } = function(&program, "demo_inc") else { unreachable!() };
        assert!(params[1].3, "&$success stays by reference");
    }

    /// A callable cannot be handed to an extension yet; the function is still
    /// declared so a call explains itself.
    #[test]
    fn unpassable_parameters_produce_an_explaining_wrapper() {
        let extension = installed(SURFACE);
        let unsupported = unsupported_functions(&extension);
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].0, "demo_map");
        assert!(unsupported[0].1.contains("callable"));
        let program = declarations(&hosted());
        let StmtKind::FunctionDecl { variadic, .. } = function(&program, "demo_map") else { unreachable!() };
        assert_eq!(variadic.as_deref(), Some("__elephc_arguments"));
    }

    #[test]
    fn variadic_parameters_stay_variadic() {
        let program = declarations(&hosted());
        let StmtKind::FunctionDecl { params, variadic, .. } = function(&program, "demo_sum") else { unreachable!() };
        assert!(params.is_empty());
        assert_eq!(variadic.as_deref(), Some("values"));
    }

    /// Exception classes are declared parents-first, and rebuilt
    /// subclass-first so a parent's arm never catches its child.
    #[test]
    fn exception_classes_are_declared_and_rebuilt_in_order() {
        let program = declarations(&hosted());
        let classes: Vec<&str> = program
            .iter()
            .filter_map(|stmt| match &stmt.kind {
                StmtKind::ClassDecl { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(classes, vec!["DemoException", "DemoParseException"], "DemoThing has methods: not declared");
        let throwables = declared_throwables(&hosted());
        let child = throwables.iter().position(|name| name == "DemoParseException").unwrap();
        let parent = throwables.iter().position(|name| name == "DemoException").unwrap();
        let base = throwables.iter().position(|name| name == "RuntimeException").unwrap();
        assert!(child < parent && parent < base);
    }

    #[test]
    fn scalar_constants_are_declared_and_others_skipped() {
        let program = declarations(&hosted());
        let constants: Vec<&str> = program
            .iter()
            .filter_map(|stmt| match &stmt.kind {
                StmtKind::ConstDecl { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(constants, vec!["DEMO_STEP"]);
    }

    #[test]
    fn maps_arginfo_types() {
        assert_eq!(param_type(Some("?string")), Some(Some(TypeExpr::Nullable(Box::new(TypeExpr::Str)))));
        assert_eq!(param_type(Some("array|string")), Some(Some(t_mixed())));
        assert_eq!(param_type(Some("callable")), None);
        assert_eq!(param_type(Some("Foo\\Bar")), None);
        assert_eq!(param_type(None), Some(None));
    }

    #[test]
    fn reads_arginfo_defaults() {
        let extension = installed(SURFACE);
        assert!(default_expr("512", &extension).is_some());
        assert!(default_expr("-1", &extension).is_some());
        assert!(default_expr("1.5", &extension).is_some());
        assert!(default_expr("\"\"", &extension).is_some());
        assert!(default_expr("PHP_INT_MAX", &extension).is_some());
        assert!(default_expr("DEMO_STEP", &extension).is_some());
        assert!(default_expr("Foo::BAR", &extension).is_none(), "class constants are left to C");
        assert!(default_expr("\"a\\n\"", &extension).is_none(), "escapes are not re-encoded");
    }
}
