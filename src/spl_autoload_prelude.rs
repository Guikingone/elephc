//! Purpose:
//! Gives standalone programs php's run-time SPL autoload stack: `spl_autoload_register`,
//! `spl_autoload_unregister`, `spl_autoload_functions`, `spl_autoload_call`, and the autoload
//! step of `class_exists`/`interface_exists`/`trait_exists`/`enum_exists`.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, after
//!   `autoload::run`, next to the other late preludes.
//!
//! Key details:
//! - One elephc-PHP dispatcher, `__elephc_spl_autoload($op, ...)`, owns the loader list in a
//!   static property of a hidden `__elephc_spl_autoload_state` class, so every registration
//!   site shares one table and one routine.
//! - The program's calls are rewritten to the helpers; the builtins themselves stay the
//!   compile-time stubs a closed-world build relies on.
//! - Pay-for-use and scoped: injected only when the program names one of the four
//!   `spl_autoload_*` functions, and never for a Composer project, whose classes the compiler
//!   resolves from the PSR-4 index at compile time (those builds keep today's behaviour).
//! - Registration validates the callback (php's `TypeError` wording for an unknown function),
//!   ignores a second registration of the same callback (function names compare
//!   case-insensitively, everything else with `===`), and honours `$prepend`.
//! - `spl_autoload_call` runs the loaders in order and stops at the first one after which
//!   the name exists as a class, interface, trait or enum.

use crate::errors::CompileError;
use crate::names::{Name, NameKind};
use crate::parser::ast::{BinOp, CastType, Expr, ExprKind, Program, Stmt, StmtKind, TypeExpr};
use crate::synthetic_class::{
    e_array, e_binop, e_bool, e_call, e_cast, e_concat_all, e_index, e_new, e_not, e_null, e_static_prop,
    e_str, e_var, function, internal_declarations, s_assign, s_expr, s_foreach, s_if, s_return,
    s_return_void, s_static_prop_array_push, s_static_prop_assign, s_throw, t_mixed,
};

/// The dispatcher every rewritten `spl_autoload_*` call goes through.
pub(crate) const DISPATCHER: &str = "__elephc_spl_autoload";

/// The autoloading `*_exists` helper.
pub(crate) const EXISTS_HELPER: &str = "__elephc_class_like_exists";

/// The helper `spl_autoload_call` and an autoloading `*_exists` run the loaders through.
pub(crate) const CALL_HELPER: &str = "__elephc_spl_autoload_call";

/// The helper an instantiation of a class declared nowhere throws through: it runs the loaders
/// and returns php's "not found" message.
pub(crate) const MISSING_HELPER: &str = "__elephc_spl_autoload_missing";

/// The class-like kinds whose `*_exists` builtin autoloads.
const KINDS: [&str; 4] = ["class", "interface", "trait", "enum"];

/// Builds `<kind>_exists($name, false)` for each kind, or-ed together.
fn defined_anywhere(name: Expr) -> Expr {
    KINDS
        .iter()
        .map(|kind| e_call(&format!("{kind}_exists"), vec![name.clone(), e_bool(false)]))
        .reduce(|left, right| e_binop(left, BinOp::Or, right))
        .expect("KINDS is not empty")
}

/// Builds `__elephc_spl_autoload_same($a, $b)`: whether two callbacks name the same loader.
fn same_decl() -> Stmt {
    let lowered = |var: &str| e_call("strtolower", vec![e_call("ltrim", vec![e_var(var), e_str("\\")])]);
    let lowered_index = |var: &str, index: i64| e_call(
        "strtolower",
        vec![e_cast(CastType::String, e_index(e_var(var), crate::synthetic_class::e_int(index)))],
    );
    // The class a `[class or object, method]` pair names.
    let class_of = |var: &str| {
        let first = e_index(e_var(var), crate::synthetic_class::e_int(0));
        crate::synthetic_class::e_ternary(
            e_call("is_object", vec![first.clone()]),
            e_call("get_class", vec![first.clone()]),
            e_cast(CastType::String, first),
        )
    };
    function("__elephc_spl_autoload_same")
        .param("a", t_mixed())
        .param("b", t_mixed())
        .returns(TypeExpr::Bool)
        .body(vec![
            s_if(
                e_binop(
                    e_call("is_string", vec![e_var("a")]),
                    BinOp::And,
                    e_call("is_string", vec![e_var("b")]),
                ),
                vec![s_return(e_binop(lowered("a"), BinOp::StrictEq, lowered("b")))],
                vec![],
                None,
            ),
            // Two `[class or object, method]` pairs name one loader when the method and the class
            // agree and at least one side names the class (a static method) or both hold the
            // same object.
            s_if(
                e_binop(
                    e_call("is_array", vec![e_var("a")]),
                    BinOp::And,
                    e_call("is_array", vec![e_var("b")]),
                ),
                vec![
                    s_assign("class_a", class_of("a")),
                    s_assign("class_b", class_of("b")),
                    s_return(e_binop(
                        e_binop(
                            e_binop(lowered_index("a", 1), BinOp::StrictEq, lowered_index("b", 1)),
                            BinOp::And,
                            e_binop(lowered("class_a"), BinOp::StrictEq, lowered("class_b")),
                        ),
                        BinOp::And,
                        e_binop(
                            e_binop(
                                e_call("is_string", vec![e_index(e_var("a"), crate::synthetic_class::e_int(0))]),
                                BinOp::Or,
                                e_call("is_string", vec![e_index(e_var("b"), crate::synthetic_class::e_int(0))]),
                            ),
                            BinOp::Or,
                            e_binop(
                                e_index(e_var("a"), crate::synthetic_class::e_int(0)),
                                BinOp::StrictEq,
                                e_index(e_var("b"), crate::synthetic_class::e_int(0)),
                            ),
                        ),
                    )),
                ],
                vec![],
                None,
            ),
            s_return(e_binop(e_var("a"), BinOp::StrictEq, e_var("b"))),
        ])
        .build()
}

/// The hidden class whose static property holds the loader list (`__elephc*` classes are not
/// reported by `class_exists` or `get_declared_classes`).
const STATE_CLASS: &str = "__elephc_spl_autoload_state";

/// The loader list property.
const LOADERS: &str = "loaders";

/// Reads the loader list.
fn loaders() -> Expr {
    e_static_prop(STATE_CLASS, LOADERS)
}

/// Builds the state class.
fn state_decl() -> Stmt {
    crate::synthetic_class::class(STATE_CLASS)
        .static_untyped_prop(LOADERS, Some(e_array(vec![])))
        .build()
}

/// Builds the dispatcher `__elephc_spl_autoload($op, $callback, $throw, $prepend)`.
fn dispatcher_decl() -> Stmt {
    let op_is = |op: &str| e_binop(e_var("op"), BinOp::StrictEq, e_str(op));
    let same = |other: &str| e_call("__elephc_spl_autoload_same", vec![e_var("entry"), e_var(other)]);
    // php's refusal of a callback that is not callable, for `register` (which also takes null)
    // and `unregister`.
    let invalid = |prefix: &str| e_new("TypeError", vec![e_concat_all(vec![
        e_str(prefix),
        crate::synthetic_class::e_ternary(
            e_call("is_string", vec![e_var("callback")]),
            e_concat_all(vec![
                e_str("function \""),
                e_var("callback"),
                e_str("\" not found or invalid function name"),
            ]),
            e_str("no array or string given"),
        ),
    ])]);
    let refuse_uncallable = |prefix: &str| s_if(
        e_not(e_call("is_callable", vec![e_var("callback")])),
        vec![s_throw(invalid(prefix))],
        vec![],
        None,
    );
    // The entry a callback is stored and compared as: copied before the `is_callable` guard
    // narrows `$callback` (so it keeps the list's boxed element type), and a `Class::method`
    // string kept as `[Class, method]`, as php does.
    let normalize_entry = || vec![
        s_assign("entry", e_var("callback")),
        s_if(
            e_binop(
                e_call("is_string", vec![e_var("entry")]),
                BinOp::And,
                e_call("str_contains", vec![e_var("entry"), e_str("::")]),
            ),
            vec![s_assign(
                "entry",
                e_call("explode", vec![e_str("::"), e_var("entry"), crate::synthetic_class::e_int(2)]),
            )],
            vec![],
            None,
        ),
    ];
    function(DISPATCHER)
        .param("op", TypeExpr::Str)
        // Declared `mixed`, so every stored loader is a boxed value of the one list.
        .param_default("callback", t_mixed(), e_null())
        .param_untyped_default("throw", e_bool(true))
        .param_untyped_default("prepend", e_bool(false))
        .returns(t_mixed())
        .body(vec![
            s_if(
                op_is("functions"),
                vec![s_return(loaders())],
                vec![],
                None,
            ),
            s_if(
                op_is("register"),
                vec![
                    s_if(
                        e_not(e_var("throw")),
                        vec![s_expr(e_call("trigger_error", vec![
                            e_str("spl_autoload_register(): Argument #2 ($do_throw) has been ignored, \
                                   spl_autoload_register() will always throw"),
                            crate::synthetic_class::e_const("E_USER_NOTICE"),
                        ]))],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_var("callback"), BinOp::StrictEq, e_null()),
                        vec![s_assign("callback", e_str("spl_autoload"))],
                        vec![],
                        None,
                    ),
                ]
                .into_iter()
                .chain(normalize_entry())
                .chain([
                    refuse_uncallable(
                        "spl_autoload_register(): Argument #1 ($callback) must be a valid callback or null, ",
                    ),
                    s_foreach(
                        loaders(),
                        None,
                        "loader",
                        vec![s_if(same("loader"), vec![s_return(e_bool(true))], vec![], None)],
                    ),
                    s_if(
                        e_var("prepend"),
                        vec![s_static_prop_assign(
                            STATE_CLASS,
                            LOADERS,
                            e_call("array_merge", vec![e_array(vec![e_var("entry")]), loaders()]),
                        )],
                        vec![],
                        Some(vec![s_static_prop_array_push(STATE_CLASS, LOADERS, e_var("entry"))]),
                    ),
                    s_return(e_bool(true)),
                ])
                .collect(),
                vec![],
                None,
            ),
        ]
        .into_iter()
        // unregister: a static-property element cannot be unset in place, so the list is
        // copied, the entry removed, and the re-packed list stored back.
        .chain(normalize_entry())
        .chain([
            // `spl_autoload_call` as the callback clears every loader, with php's deprecation.
            s_if(
                e_binop(
                    e_call("is_string", vec![e_var("callback")]),
                    BinOp::And,
                    e_binop(
                        e_call("strtolower", vec![e_call("ltrim", vec![
                            e_cast(CastType::String, e_var("callback")),
                            e_str("\\"),
                        ])]),
                        BinOp::StrictEq,
                        e_str("spl_autoload_call"),
                    ),
                ),
                vec![
                    // php's engine deprecation (`E_DEPRECATED`), at the caller's line.
                    s_expr(e_call("__elephc_deprecated", vec![
                        e_str("spl_autoload_unregister(): Using spl_autoload_call() as a callback for \
                               spl_autoload_unregister() is deprecated, to remove all registered autoloaders, \
                               call spl_autoload_unregister() for all values returned from spl_autoload_functions()"),
                    ])),
                    s_static_prop_assign(STATE_CLASS, LOADERS, e_array(vec![])),
                    s_return(e_bool(true)),
                ],
                vec![],
                None,
            ),
            refuse_uncallable("spl_autoload_unregister(): Argument #1 ($callback) must be a valid callback, "),
            s_foreach(
                loaders(),
                Some("index"),
                "loader",
                vec![s_if(
                    same("loader"),
                    vec![
                        s_assign("list", loaders()),
                        s_expr(e_call("unset", vec![e_index(e_var("list"), e_var("index"))])),
                        s_static_prop_assign(
                            STATE_CLASS,
                            LOADERS,
                            e_call("array_values", vec![e_var("list")]),
                        ),
                        s_return(e_bool(true)),
                    ],
                    vec![],
                    None,
                )],
            ),
            s_return(e_bool(false)),
        ])
        .collect())
        .build()
}

/// Builds `__elephc_spl_autoload_call($name)`: runs the loaders until one defines `$name`.
fn call_decl() -> Stmt {
    function(CALL_HELPER)
        .param("name", TypeExpr::Str)
        .returns(TypeExpr::Void)
        .body(vec![
            s_assign("name", e_call("ltrim", vec![e_var("name"), e_str("\\")])),
            s_assign("loaders", e_call(DISPATCHER, vec![e_str("functions")])),
            s_foreach(
                e_var("loaders"),
                None,
                "loader",
                vec![
                    // The default loader is a builtin a dynamic call cannot reach.
                    s_if(
                        e_binop(
                            e_call("is_string", vec![e_var("loader")]),
                            BinOp::And,
                            e_binop(
                                e_call("strtolower", vec![e_cast(CastType::String, e_var("loader"))]),
                                BinOp::StrictEq,
                                e_str("spl_autoload"),
                            ),
                        ),
                        vec![s_expr(e_call("spl_autoload", vec![e_var("name")]))],
                        vec![],
                        Some(vec![s_expr(e_call("call_user_func", vec![e_var("loader"), e_var("name")]))]),
                    ),
                    s_if(
                        e_call(EXISTS_HELPER, vec![e_str("any"), e_var("name"), e_bool(false)]),
                        vec![s_return_void()],
                        vec![],
                        None,
                    ),
                ],
            ),
        ])
        .build()
}

/// Builds `__elephc_spl_autoload_missing($name)`: runs the loaders for a class no declaration
/// names, then returns the message of php's `Error`.
fn missing_decl() -> Stmt {
    function(MISSING_HELPER)
        .param("name", TypeExpr::Str)
        .returns(TypeExpr::Str)
        .body(vec![
            s_expr(e_call(CALL_HELPER, vec![e_var("name")])),
            s_return(e_concat_all(vec![e_str("Class \""), e_var("name"), e_str("\" not found")])),
        ])
        .build()
}

/// Builds `__elephc_class_like_exists($kind, $name, $autoload)`; kind `any` asks whether the
/// name exists as any class-like, without autoloading.
fn exists_decl() -> Stmt {
    let probe = |kind: &str| e_call(&format!("{kind}_exists"), vec![e_var("name"), e_bool(false)]);
    let mut body = vec![s_if(
        e_binop(e_var("kind"), BinOp::StrictEq, e_str("any")),
        vec![s_return(defined_anywhere(e_var("name")))],
        vec![],
        None,
    )];
    for kind in KINDS {
        body.push(s_if(
            e_binop(e_var("kind"), BinOp::StrictEq, e_str(kind)),
            vec![
                s_if(probe(kind), vec![s_return(e_bool(true))], vec![], None),
                // php autoloads only a name no class, interface, trait or enum uses yet.
                s_if(
                    e_binop(e_not(e_var("autoload")), BinOp::Or, defined_anywhere(e_var("name"))),
                    vec![s_return(e_bool(false))],
                    vec![],
                    None,
                ),
                s_expr(e_call(CALL_HELPER, vec![e_var("name")])),
                s_return(probe(kind)),
            ],
            vec![],
            None,
        ));
    }
    body.push(s_return(e_bool(false)));
    function(EXISTS_HELPER)
        .param("kind", TypeExpr::Str)
        .param("name", TypeExpr::Str)
        .param_untyped_default("autoload", e_bool(true))
        .returns(TypeExpr::Bool)
        .body(body)
        .build()
}

/// Builds the prelude declarations.
pub(crate) fn declarations() -> Program {
    internal_declarations(|| vec![state_decl(), same_decl(), dispatcher_decl(), call_decl(), exists_decl(), missing_decl()])
}

/// Returns the bare lowercase name of a call to a global function, if it is one.
fn global_call_name(expr: &Expr) -> Option<String> {
    let ExprKind::FunctionCall { name, args } = &expr.kind else { return None; };
    if args.iter().any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. } | ExprKind::Spread(_))) {
        return None;
    }
    let bare = name.as_str().trim_start_matches('\\');
    (!bare.contains('\\')).then(|| bare.to_ascii_lowercase())
}

/// Returns whether the program names one of the stack's functions.
fn program_uses_stack(program: &mut Program) -> bool {
    let mut used = false;
    crate::undefined_constants::walk_program_exprs_mut(program, &mut |expr| {
        if let Some(name) = global_call_name(expr) {
            used |= matches!(
                name.as_str(),
                "spl_autoload_register" | "spl_autoload_unregister" | "spl_autoload_functions" | "spl_autoload_call"
            );
        }
    });
    used
}

/// Rewrites the program's stack calls and autoloading `*_exists` calls to the helpers.
fn rewrite_calls(program: &mut Program) {
    crate::undefined_constants::walk_program_exprs_mut(program, &mut |expr| {
        let Some(name) = global_call_name(expr) else { return; };
        let ExprKind::FunctionCall { name: callee, args } = &mut expr.kind else { return; };
        let span = expr.span;
        if name == "spl_autoload_call" {
            *callee = Name::from_parts(NameKind::FullyQualified, vec![CALL_HELPER.to_string()]);
            return;
        }
        let op = match name.as_str() {
            "spl_autoload_register" => Some("register"),
            "spl_autoload_unregister" => Some("unregister"),
            "spl_autoload_functions" => Some("functions"),
            _ => None,
        };
        let (helper, lead) = if let Some(op) = op {
            (DISPATCHER, op)
        } else if let Some(kind) = name.strip_suffix("_exists").filter(|kind| KINDS.contains(kind)) {
            (EXISTS_HELPER, kind)
        } else {
            return;
        };
        let lead = lead.to_string();
        *callee = Name::from_parts(NameKind::FullyQualified, vec![helper.to_string()]);
        args.insert(0, Expr::new(ExprKind::StringLiteral(lead), span));
    });
}

/// Collects the lowercase bare names of every class-like the program declares, at any depth.
fn declared_class_likes(stmts: &mut Vec<Stmt>, names: &mut std::collections::HashSet<String>) {
    for stmt in stmts.iter_mut() {
        match &stmt.kind {
            StmtKind::ClassDecl { name, .. }
            | StmtKind::EnumDecl { name, .. }
            | StmtKind::InterfaceDecl { name, .. }
            | StmtKind::TraitDecl { name, .. } => {
                let bare = name.rsplit('\\').next().unwrap_or(name);
                names.insert(bare.to_ascii_lowercase());
            }
            _ => {}
        }
        crate::undefined_reads::for_each_child_body_mut_pub(stmt, &mut |body| {
            declared_class_likes(body, names)
        });
    }
}

/// Rewrites `new X(...)` of a class no declaration and no builtin names into php's behaviour:
/// the loaders run, then an `Error` is thrown. php looks the class up before it evaluates the
/// constructor arguments, so they are dropped.
fn rewrite_missing_instantiations(program: &mut Program) {
    let mut declared = std::collections::HashSet::new();
    declared_class_likes(program, &mut declared);
    crate::undefined_constants::walk_program_exprs_mut(program, &mut |expr| {
        let ExprKind::NewObject { class_name, .. } = &expr.kind else { return; };
        let name = class_name.as_str().trim_start_matches('\\').to_string();
        let bare = name.rsplit('\\').next().unwrap_or(&name).to_ascii_lowercase();
        if matches!(bare.as_str(), "self" | "static" | "parent")
            || declared.contains(&bare)
            || elephc_builtin_contract::lookup_class(&name).is_some()
        {
            return;
        }
        // The program is already name-resolved: both names must be fully qualified.
        let mut message = e_call(MISSING_HELPER, vec![e_str(&name)]);
        if let ExprKind::FunctionCall { name: callee, .. } = &mut message.kind {
            *callee = crate::synthetic_class::fq_name(MISSING_HELPER);
        }
        let mut error = crate::synthetic_class::e_new_fq("Error", vec![message]);
        // The `Error` reports the line of the `new` it replaces.
        error.span = expr.span;
        *expr = Expr::new(ExprKind::Throw(Box::new(error)), expr.span);
    });
}

/// Prepends the autoload stack and routes the program's calls to it when the program uses the
/// stack and resolves nothing at compile time (no PSR-4 index, no consumed closure rule);
/// otherwise returns the program unchanged.
pub fn inject_if_used(
    mut program: Program,
    compile_time_autoload: bool,
    inventory: &mut crate::optimize::reachability::PreludeInventory,
) -> Result<Program, CompileError> {
    if compile_time_autoload || !program_uses_stack(&mut program) {
        return Ok(program);
    }
    rewrite_calls(&mut program);
    rewrite_missing_instantiations(&mut program);
    let mut combined = crate::name_resolver::resolve(declarations())?;
    inventory.record_program("spl_autoload", &combined);
    combined.extend(program);
    Ok(combined)
}
