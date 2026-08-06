//! Purpose:
//! Resolves individual include and require statements during resolver traversal.
//! Parses target files, handles include_once state, and merges resolved included statements.
//!
//! Called from:
//! - `crate::resolver::engine::resolve_stmts()`.
//!
//! Key details:
//! - Include paths are folded in the caller's constant state and file base directory.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::errors::CompileError;
use crate::names::Name;
use crate::parser::ast::{BinOp, Expr, ExprKind, Stmt, StmtKind, TypeExpr};
use crate::span::Span;

use super::declarations::strip_discoverable_declarations;
use super::discovery::FunctionVariantRegistry;
use super::engine::resolve_stmts;
use super::files::{parse_file, resolve_path};
use super::include_once::include_once_label;
use super::include_path::{fold_include_path, runtime_dynamic_include_path_detail};
use super::include_returns::{assign_flag, rewrite_scope_returns, IncludeReturnRewrite};
use super::state::ResolveState;

/// Process-global counter producing unique hidden temporary names for value-position includes.
static VALUE_INCLUDE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Where the value produced by an expression-position include must be delivered.
pub(super) enum IncludeValueCapture {
    /// `$name = require X;` — assign the include's value to the named caller variable.
    Assign(String),
    /// `return require X;` — return the include's value from the enclosing function.
    Return,
}

/// Lowers a `$obj->prop = require $dynamic;` / `Class::$prop = require $dynamic;` whose include
/// path is an unresolvable runtime-dynamic string under lenient lowering directly to the diverging
/// runtime-fatal stub, dropping the property store entirely.
///
/// Such a store is dead: the include is evaluated (and fatals) before the assignment completes, so
/// the value never reaches the property. Routing it through the normal value-include hoist instead
/// seeds a hidden `mixed` temporary and captures it into the typed property — which is
/// representation-safe for object/union properties but has no EIR lowering for an *array*-typed
/// property (`prop_set mixed -> Array` is unsupported), turning the checker-level fix into a
/// codegen false-green (`$this->bundles = require $cachePath;` in Symfony's Kernel). Emitting only
/// the stub keeps that store out of both the checker and the backend. Nested/conditional include
/// stores (`... && is_object($this->container = include $p)`, `$x = $c ? require $p : []`) are not
/// direct property statements and stay on the hoist path, where the `mixed` seed is EIR-safe
/// (object property / untyped local).
///
/// Returns `None` (fall through to normal resolution) unless the statement is exactly a
/// property/static-property assignment whose direct right-hand side is a degraded runtime-dynamic
/// include; a statically-invalid path is left to raise its usual hard error.
pub(super) fn try_expand_degraded_property_include(
    stmt: &Stmt,
    state: &ResolveState,
) -> Option<Vec<Stmt>> {
    let value = match &stmt.kind {
        StmtKind::PropertyAssign { value, .. }
        | StmtKind::StaticPropertyAssign { value, .. } => value,
        _ => return None,
    };
    let ExprKind::IncludeValue { path, .. } = &value.kind else {
        return None;
    };
    if !(state.lenient_dynamic_includes && fold_include_path(path, state).is_err()) {
        return None;
    }
    dynamic_include_fatal_stub(path, stmt.span)
}

/// Resolves a single include/require statement by parsing the target file,
/// recursively resolving its statements, and returning them wrapped in
/// appropriate include_once guards.
///
/// - `once`: when true, skips already-included files and wraps output in `IncludeOnceGuard`
/// - `required`: when true, returns an error if the target file does not exist
/// - `declared_once`: tracks files already processed; updated on return
/// - `include_chain`: current include path for cycle detection; must not contain `canonical`
/// - State (`namespace`, `const_imports`) is saved before recursion and restored after
/// - Returns `None` if the file does not exist and `required` is false, or if a once file was already included
/// - For `once`: wraps body in `IncludeOnceGuard` with the file's label
/// - For non-once: emits `IncludeOnceMark` before the body for later once/require_once checks
pub(super) fn resolve_include_stmt(
    stmt: &Stmt,
    path: &Expr,
    once: bool,
    required: bool,
    base_dir: &Path,
    declared_once: &mut HashSet<PathBuf>,
    include_chain: &mut Vec<PathBuf>,
    state: &mut ResolveState,
    function_variants: &FunctionVariantRegistry,
) -> Result<Option<Vec<Stmt>>, CompileError> {
    let path_str = match fold_include_path(path, state) {
        Ok(s) => s,
        Err(msg) => {
            // Under lenient include lowering (autoloader-spliced library code), an
            // unresolvable *runtime-dynamic* path becomes a diverging runtime-fatal stub so
            // the closed-world compile is not blocked by a lazy include that may never run.
            // Statically-invalid shapes (e.g. an integer path) still hard-error.
            if state.lenient_dynamic_includes {
                if let Some(stub) = dynamic_include_fatal_stub(path, stmt.span) {
                    return Ok(Some(stub));
                }
            }
            return Err(CompileError::new(stmt.span, &msg));
        }
    };
    let resolved = resolve_path(&path_str, base_dir);
    let canonical = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());

    // elephc's `crate::autoload` reads composer.json (PSR-4/PSR-0/classmap/files) directly and
    // splices the referenced classes and `autoload.files` helpers itself, fully replacing
    // composer's runtime autoloader. Following composer's autoloader entry / machinery here is
    // therefore redundant, and only exposes composer's own dynamic `include $file;` machinery to
    // the strict include resolver. Treat the entry / machinery as a resolver no-op so class
    // discovery is owned exclusively by `crate::autoload`.
    if is_composer_autoloader_entry(&resolved) {
        return Ok(Some(Vec::new()));
    }

    if !resolved.exists() {
        if required {
            return Err(CompileError::new(
                stmt.span,
                &format!("Required file not found: '{}'", path_str),
            ));
        }
        return Ok(None);
    }

    if include_chain.contains(&canonical) {
        if once {
            return Ok(None);
        }
        return Err(CompileError::new(
            stmt.span,
            &format!("Circular include detected: '{}'", path_str),
        ));
    }

    let included_stmts = parse_file(&resolved, stmt.span)?;
    let included_stmts =
        crate::magic_constants::substitute_file_and_scope_constants(included_stmts, &resolved);
    // Strict-PHP audit of the included user file on its freshly parsed AST,
    // before include-variant synthesis introduces compiler-internal names.
    crate::strict_php::check_file(&included_stmts, &resolved.display().to_string())?;

    let included_dir = resolved.parent().unwrap_or(base_dir);
    include_chain.push(canonical.clone());

    let saved_namespace = state.namespace.clone();
    let saved_imports = state.const_imports.clone();
    state.namespace = None;
    state.const_imports = HashMap::new();
    let resolved_stmts = resolve_stmts(
        included_stmts,
        included_dir,
        declared_once,
        include_chain,
        state,
        function_variants,
    )?;
    state.namespace = saved_namespace;
    state.const_imports = saved_imports;

    include_chain.pop();

    let include_label = include_once_label(&canonical);
    let executable =
        strip_discoverable_declarations(resolved_stmts, Some(&canonical), function_variants);
    if once {
        // Declaration discovery already hoisted compile-time declarations;
        // executable include body statements are guarded so runtime order matches PHP.
        declared_once.insert(canonical);
        return Ok(Some(vec![Stmt::new(
            StmtKind::IncludeOnceGuard {
                label: include_label,
                body: vec![Stmt::new(
                    StmtKind::NamespaceBlock {
                        name: None,
                        body: executable,
                    },
                    stmt.span,
                )],
            },
            stmt.span,
        )]));
    }

    // Regular includes still mark the file as loaded for a later
    // include_once/require_once, while executable statements stay at
    // the include point.
    declared_once.insert(canonical);
    Ok(Some(vec![
        Stmt::new(
            StmtKind::IncludeOnceMark {
                label: include_label,
            },
            stmt.span,
        ),
        Stmt::new(
            StmtKind::NamespaceBlock {
                name: None,
                body: executable,
            },
            stmt.span,
        ),
    ]))
}

/// Returns `true` when `path` refers to composer's autoloader entry or its internal machinery,
/// which elephc's `crate::autoload` replaces wholesale and must therefore not splice into the
/// program via the include resolver.
///
/// The detector is intentionally narrow so ordinary vendor class files
/// (`vendor/<pkg>/src/*.php`) and `autoload.files` helpers are never skipped. It matches only:
/// - `autoload.php` or `autoload_runtime.php` whose immediate parent directory is named `vendor`
///   (the composer autoloader entry points), or
/// - any path under a `vendor/composer/` directory (the generated autoloader machinery:
///   `ClassLoader.php`, `autoload_real.php`, `autoload_static.php`, `installed.php`, …).
pub(super) fn is_composer_autoloader_entry(path: &Path) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str());
    let parent_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str());

    // `vendor/autoload.php` or `vendor/autoload_runtime.php` entry points.
    if matches!(file_name, Some("autoload.php") | Some("autoload_runtime.php"))
        && parent_name == Some("vendor")
    {
        return true;
    }

    // Anything under a `vendor/composer/` directory is generated autoloader machinery.
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    components
        .windows(2)
        .any(|w| w == ["vendor", "composer"])
}

/// Expands an expression-position `include`/`require` (`$x = require X;` or `return require X;`)
/// into a sequence of statements that run the included file *in the caller's scope* and deliver
/// its value to `capture`.
///
/// The included file's statements are inlined directly (sharing the caller's variables), and its
/// first top-level `return E` is rewritten to assign a hidden temporary. A successful include with
/// no top-level `return` yields `1`; a missing non-required include yields `false`, matching PHP.
///
/// Nested top-level returns inside control flow within the included file are not rewritten and keep
/// the same semantics as a statement-position include (they return from the enclosing function).
pub(super) fn expand_value_include(
    span: Span,
    path: &Expr,
    once: bool,
    required: bool,
    capture: IncludeValueCapture,
    base_dir: &Path,
    declared_once: &mut HashSet<PathBuf>,
    include_chain: &mut Vec<PathBuf>,
    state: &mut ResolveState,
    function_variants: &FunctionVariantRegistry,
) -> Result<Vec<Stmt>, CompileError> {
    // Under lenient include lowering, a value-position `return require $dynamic;` whose path
    // cannot be resolved becomes a diverging runtime-fatal stub. Returning it directly (rather
    // than the usual `<tmp> = ...; return <tmp>;` scaffolding) keeps the enclosing function's
    // declared return type satisfied: the stub's `exit` diverges, so no value is returned and
    // the unreachable `return <tmp>` that would otherwise mismatch the return type is omitted.
    if state.lenient_dynamic_includes
        && matches!(capture, IncludeValueCapture::Return)
        && fold_include_path(path, state).is_err()
    {
        if let Some(stub) = dynamic_include_fatal_stub(path, span) {
            return Ok(stub);
        }
    }

    // An unresolvable runtime-dynamic path under lenient lowering degrades to a diverging
    // runtime-fatal stub (`resolve_include_stmt` returns the `dynamic_include_fatal_stub` body).
    // For a non-`Return` capture (`$x = require $dynamic;`, or the hoisted temp for a nested
    // `... = include $dynamic`), the stub cannot early-return as the `Return` case above does, so
    // the temp is still seeded and captured into the host statement. Because the file is
    // unresolvable, the value it would "return" is genuinely unknown — typing the seed `mixed`
    // (rather than the `int(1)` no-return default) keeps that capture representation-safe to store
    // into any typed target the host assigns it to. The stub diverges before the store, so the
    // value is never observed at runtime; this only prevents an impossible `got Int` type error
    // (e.g. `$this->container = require $cachePath;` where `$container` is object-typed).
    let degraded_dynamic = state.lenient_dynamic_includes && fold_include_path(path, state).is_err();

    let tmp = format!(
        "__elephc_inc_{}",
        VALUE_INCLUDE_COUNTER.fetch_add(1, Ordering::Relaxed)
    );

    let include_stmt = Stmt::new(
        StmtKind::Include {
            path: path.clone(),
            once,
            required,
        },
        span,
    );
    let resolved = resolve_include_stmt(
        &include_stmt,
        path,
        once,
        required,
        base_dir,
        declared_once,
        include_chain,
        state,
        function_variants,
    )?;

    let mut out = Vec::new();
    match resolved {
        // Missing, non-required include: PHP evaluates the expression to `false`.
        None => {
            out.push(assign_temp(
                &tmp,
                Expr::new(ExprKind::BoolLiteral(false), span),
                span,
            ));
        }
        Some(mut wrapped) => {
            let flag = format!("{tmp}_returned");
            let rewrite = rewrite_first_include_return(&mut wrapped, &tmp, &flag);
            // Pre-seed the default include value of `1` when the included body cannot set the
            // temporary itself: it has no top-level `return`, its `return` is conditional, or it is
            // an `_once` include whose guarded body may be skipped on a repeat include. For a
            // degraded runtime-fatal dynamic include the seed is typed `mixed` instead (see
            // `degraded_dynamic` above): the unresolvable file's value is unknown, and the diverging
            // stub means it is never read.
            if rewrite != IncludeReturnRewrite::Unconditional || once {
                out.push(seed_include_temp(&tmp, degraded_dynamic, span));
            }
            // A conditionally-returning body branches on the flag, so it must be defined before the
            // body runs — and OUTSIDE any include-once guard, whose body a repeat include skips
            // entirely while the guarded statements' flag reads remain.
            if rewrite == IncludeReturnRewrite::Conditional {
                out.push(assign_flag(&flag, false, span));
            }
            out.extend(wrapped);
        }
    }

    let value = Expr::new(ExprKind::Variable(tmp), span);
    match capture {
        IncludeValueCapture::Assign(name) => {
            out.push(Stmt::new(StmtKind::Assign { name, value }, span));
        }
        IncludeValueCapture::Return => {
            out.push(Stmt::new(StmtKind::Return(Some(value)), span));
        }
    }
    Ok(out)
}

/// Builds the IN-PLACE diverging expression that replaces an unresolvable runtime-dynamic
/// value-position `include`/`require` under lenient include lowering, for the nested positions
/// `hoist_includes` would otherwise lift out of their owning statement.
///
/// Returns `None` — meaning "not a degraded dynamic include, take the normal hoist path" — unless
/// lenient lowering is active, the path does not fold to a compile-time constant, and the path is
/// a runtime-dynamic shape (a statically-invalid path keeps its hard compile error).
///
/// # Why in place rather than hoisted
///
/// `hoist_includes` evaluates a nested value-include EAGERLY as a statement emitted BEFORE the
/// owning statement. For a nested position inside a CONDITION that also defines the path variable
/// — Symfony's `if (!$file = …) { … } elseif (false === include $file) { … }` — the hoisted
/// statements are placed above the whole `if`, i.e. before `$file` is assigned, and the stub's
/// message concatenation then reads an undefined `$file` ("Undefined variable: $file"). Rewriting
/// the include in place keeps it at its real program point, where the path variable is defined.
///
/// # Shape
///
/// `fwrite(STDERR, <msg>) ? exit(255) : exit(255)` — the same stderr message and PHP fatal exit
/// code 255 as the statement-position `dynamic_include_fatal_stub`, expressed as one expression.
/// The `fwrite` is the ternary CONDITION so it is always evaluated (a short-circuit `&&`/`||`/`?:`
/// would skip the exit on one of the two truth values), and BOTH branches diverge, so the whole
/// expression diverges regardless of what `fwrite` returns and can therefore stand in any operand
/// position without ever yielding a value.
pub(super) fn degraded_dynamic_include_expr(
    path: &Expr,
    span: Span,
    state: &ResolveState,
) -> Option<Expr> {
    if !(state.lenient_dynamic_includes && fold_include_path(path, state).is_err()) {
        return None;
    }
    runtime_dynamic_include_path_detail(path)?;
    Some(Expr::new(
        ExprKind::Ternary {
            condition: Box::new(dynamic_include_fatal_write_expr(path, span)),
            then_expr: Box::new(dynamic_include_exit_expr(span)),
            else_expr: Box::new(dynamic_include_exit_expr(span)),
        },
        span,
    ))
}

/// Builds the `fwrite(STDERR, "<prefix>" . <path> . "<suffix>")` call shared by the
/// statement-position stub and the in-place diverging expression.
///
/// The message concatenates the original `path` expression so the runtime diagnostic names the
/// actual (computed) path that could not be resolved. Re-evaluating `path` here also keeps any
/// variable it reads marked as used, so degrading `$p = ...; require $p;` does not turn the `$p`
/// assignment into a spurious "unused variable" warning.
fn dynamic_include_fatal_write_expr(path: &Expr, span: Span) -> Expr {
    let prefix = Expr::new(
        ExprKind::StringLiteral(
            "Fatal error: could not resolve dynamic include/require path at compile time: "
                .to_string(),
        ),
        span,
    );
    let suffix = Expr::new(
        ExprKind::StringLiteral(" (elephc compiled it as a runtime fatal)\n".to_string()),
        span,
    );
    // `prefix . <path> . suffix`
    let message = concat(concat(prefix, path.clone(), span), suffix, span);
    Expr::new(
        ExprKind::FunctionCall {
            name: Name::unqualified("fwrite"),
            args: vec![
                Expr::new(ExprKind::ConstRef(Name::unqualified("STDERR")), span),
                message,
            ],
        },
        span,
    )
}

/// Builds the `exit(255)` call (PHP's fatal-error exit code) shared by the statement-position stub
/// and the in-place diverging expression.
fn dynamic_include_exit_expr(span: Span) -> Expr {
    Expr::new(
        ExprKind::FunctionCall {
            name: Name::unqualified("exit"),
            args: vec![Expr::new(ExprKind::IntLiteral(255), span)],
        },
        span,
    )
}

/// Builds the diverging runtime-fatal stub that replaces an unresolvable runtime-dynamic
/// include/require under lenient include lowering. The stub writes a descriptive message to
/// stderr (`fwrite(STDERR, ...)`) and then calls `exit(255)` — PHP's fatal-error exit code.
///
/// `exit` is recognized as a function-exit guarantee by termination analysis, so a function
/// body whose only remaining path runs this stub satisfies any declared return type without an
/// explicit `return` (the value-position `return require $dynamic;` case). The synthetic nodes
/// mirror exactly what the parser produces for `fwrite(STDERR, ...)` and `exit(255)`, so they
/// flow unchanged through name resolution, type checking, and EIR lowering.
///
/// The path is only evaluated on the fatal path, which is reached exactly when the original
/// include would have run; see `dynamic_include_fatal_write_expr` for why the message re-evaluates
/// it at all.
///
/// Returns `None` when `path` is not a runtime-dynamic expression: statically-invalid include
/// shapes (e.g. an integer or boolean literal path) keep their hard compile error.
fn dynamic_include_fatal_stub(path: &Expr, span: Span) -> Option<Vec<Stmt>> {
    // Gate: only runtime-dynamic shapes degrade; statically-invalid paths keep their hard error.
    runtime_dynamic_include_path_detail(path)?;

    Some(vec![
        Stmt::new(
            StmtKind::ExprStmt(dynamic_include_fatal_write_expr(path, span)),
            span,
        ),
        Stmt::new(StmtKind::ExprStmt(dynamic_include_exit_expr(span)), span),
    ])
}

/// Builds a `left . right` string-concatenation expression at `span`, used to assemble the
/// runtime-fatal stub message from a static prefix/suffix and the original include path.
fn concat(left: Expr, right: Expr, span: Span) -> Expr {
    Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(left),
            op: BinOp::Concat,
            right: Box::new(right),
        },
        span,
    )
}

/// Builds the include temporary's default-value seed. The seed value is always the `int(1)`
/// no-return default; when `mixed_typed` is set (a degraded runtime-fatal dynamic include), the
/// seed is emitted as a `mixed`-typed local declaration so the temp — captured into a typed host
/// target the diverging stub never actually reaches — types as `mixed` rather than `int`, keeping
/// the store representation-safe instead of raising an impossible `got Int` mismatch.
fn seed_include_temp(temp: &str, mixed_typed: bool, span: Span) -> Stmt {
    let value = Expr::new(ExprKind::IntLiteral(1), span);
    if mixed_typed {
        Stmt::new(
            StmtKind::TypedAssign {
                type_expr: TypeExpr::Named(Name::unqualified("mixed")),
                name: temp.to_string(),
                value,
            },
            span,
        )
    } else {
        assign_temp(temp, value, span)
    }
}

/// Builds a `<temp> = <value>;` assignment statement for the hidden include temporary.
fn assign_temp(temp: &str, value: Expr, span: Span) -> Stmt {
    Stmt::new(
        StmtKind::Assign {
            name: temp.to_string(),
            value,
        },
        span,
    )
}

/// Rewrites the include body's top-level `return`s to assign the include temporary.
///
/// Recurses through the `IncludeOnceGuard`/`NamespaceBlock` wrappers produced by
/// `resolve_include_stmt` and delegates the actual rewrite to
/// `include_returns::rewrite_scope_returns`, whose outcome says whether the temporary is assigned
/// on every path (no seed needed) and whether the body reads the `flag` variable.
fn rewrite_first_include_return(
    wrapped: &mut [Stmt],
    temp: &str,
    flag: &str,
) -> IncludeReturnRewrite {
    for stmt in wrapped.iter_mut() {
        let rewrite = match &mut stmt.kind {
            StmtKind::NamespaceBlock { body, .. } => rewrite_scope_returns(body, temp, flag),
            StmtKind::IncludeOnceGuard { body, .. } => {
                rewrite_first_include_return(body, temp, flag)
            }
            _ => continue,
        };
        if rewrite != IncludeReturnRewrite::Absent {
            return rewrite;
        }
    }
    IncludeReturnRewrite::Absent
}
