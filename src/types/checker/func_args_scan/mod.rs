//! Purpose:
//! Detects user functions/methods whose body calls `func_num_args()`, `func_get_args()`,
//! or `func_get_arg()`, and validates that no such call appears at the top-level (global)
//! scope, mirroring PHP's own "cannot be called from the global scope" fatal.
//!
//! Called from:
//! - `crate::types::checker::driver::mod` — runs `mark_func_args_functions` after all
//!   user function/method signatures are resolved, and `validate_func_args_global_scope`
//!   alongside the other program-wide validation passes.
//! - `crate::ir_lower` — consumes `CheckResult::func_args_functions` to thread the hidden
//!   arity-count/variadic-tail ABI extension through call sites and callee prologues.
//!
//! Key details:
//! - "Closed-world": only DIRECT free-function and method calls get the hidden ABI
//!   extension. Closures are intentionally out of scope for this pass (see
//!   `detect::body_calls_func_args_intrinsic`, which stops at `Closure` boundaries) and
//!   are rejected defensively in `crate::ir_lower` if reached.
//! - `mark_func_args_functions` mutates each matched signature's `variadic` slot in place
//!   (synthesizing one when the function was not already variadic) so 100% of the existing
//!   variadic call-argument-count relaxation, named-argument exclusion, and EIR local/ABI
//!   slot materialization machinery is reused instead of re-implemented.

mod detect;

pub(crate) use detect::{
    body_calls_func_args_intrinsic, expr_calls_func_args_intrinsic, is_func_args_intrinsic_name,
};

use std::collections::HashSet;

use crate::errors::CompileError;
use crate::parser::ast::{Program, StmtKind};
use crate::types::FunctionSig;

/// Synthetic trailing variadic parameter name appended to a function/method signature that
/// uses `func_num_args()`/`func_get_args()`/`func_get_arg()` but was not already declared
/// variadic. Collects positional arguments beyond the declared parameters, exactly like a
/// real `...$rest` parameter, so `func_get_args()` can read them back.
pub(crate) const SYNTHETIC_VARIADIC_NAME: &str = "__fga_variadic";

/// Hidden trailing arity-count parameter name. Carries the number of PHP-visible
/// positional/named-resolved arguments the caller actually passed (PHP's
/// `func_num_args()` semantics: highest-filled parameter index + 1, plus any variadic
/// tail count). Threaded as a plain EIR ABI parameter OUTSIDE the checker-visible
/// `FunctionSig::params` list (see `crate::ir_lower::function`), so caller-visible
/// arg-count/named-argument matching never sees it.
pub(crate) const HIDDEN_ARGC_PARAM_NAME: &str = "__fga_argc";

/// Ensures `sig` accepts unlimited trailing positional arguments by synthesizing a
/// `SYNTHETIC_VARIADIC_NAME` variadic slot when `sig` is not already variadic. Reuses the
/// existing variadic parameter untouched when one was already declared (e.g.
/// `function f($a, ...$rest)` that also calls `func_get_args()`).
pub(crate) fn ensure_variadic_for_func_args(sig: &mut FunctionSig) {
    if sig.variadic.is_some() {
        return;
    }
    sig.params.push((
        SYNTHETIC_VARIADIC_NAME.to_string(),
        crate::types::PhpType::Array(Box::new(crate::types::PhpType::Mixed)),
    ));
    // Matches `crate::types::signatures::variadic()`'s convention: the variadic slot's
    // own `defaults` entry is `Some(<empty array literal>)`, not `None` — arg-count
    // validation (`call_validation::required`) counts `None` defaults as "required
    // parameters", so a bare `None` here would wrongly demand a value for the variadic
    // tail itself.
    sig.defaults.push(Some(crate::parser::ast::Expr::new(
        crate::parser::ast::ExprKind::ArrayLiteral(Vec::new()),
        crate::span::Span::dummy(),
    )));
    sig.ref_params.push(false);
    sig.declared_params.push(false);
    sig.variadic = Some(SYNTHETIC_VARIADIC_NAME.to_string());
}

/// Scans every top-level (non-declaration) statement of the program for a call to
/// `func_num_args()`, `func_get_args()`, or `func_get_arg()` at the global scope and
/// returns one `CompileError` per illegal call site, mirroring PHP's own
/// `"func_get_args() cannot be called from the global scope"` runtime fatal as a
/// compile-time diagnostic instead.
pub(crate) fn validate_func_args_global_scope(program: &Program) -> Vec<CompileError> {
    let mut errors = Vec::new();
    let top_level: Vec<_> = program
        .iter()
        .filter(|stmt| {
            !matches!(
                stmt.kind,
                StmtKind::FunctionDecl { .. }
                    | StmtKind::ClassDecl { .. }
                    | StmtKind::TraitDecl { .. }
                    | StmtKind::InterfaceDecl { .. }
                    | StmtKind::EnumDecl { .. }
                    | StmtKind::FunctionVariantGroup { .. }
            )
        })
        .cloned()
        .collect();
    if body_calls_func_args_intrinsic(&top_level) {
        for stmt in &top_level {
            collect_global_scope_error(stmt, &mut errors);
        }
    }
    errors
}

/// Recurses into `stmt` reusing the same scope-stopping rules as
/// `detect::body_calls_func_args_intrinsic`, emitting one `CompileError` per direct
/// `func_num_args`/`func_get_args`/`func_get_arg` call found at global scope.
fn collect_global_scope_error(stmt: &crate::parser::ast::Stmt, errors: &mut Vec<CompileError>) {
    use crate::parser::ast::ExprKind;
    match &stmt.kind {
        StmtKind::ExprStmt(expr) | StmtKind::Echo(expr) | StmtKind::Throw(expr) => {
            collect_global_scope_expr_error(expr, errors)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_global_scope_expr_error(condition, errors);
            for s in then_body {
                collect_global_scope_error(s, errors);
            }
            for (c, body) in elseif_clauses {
                collect_global_scope_expr_error(c, errors);
                for s in body {
                    collect_global_scope_error(s, errors);
                }
            }
            if let Some(body) = else_body {
                for s in body {
                    collect_global_scope_error(s, errors);
                }
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            collect_global_scope_expr_error(condition, errors);
            for s in body {
                collect_global_scope_error(s, errors);
            }
        }
        StmtKind::For {
            condition, body, ..
        } => {
            if let Some(condition) = condition {
                collect_global_scope_expr_error(condition, errors);
            }
            for s in body {
                collect_global_scope_error(s, errors);
            }
        }
        StmtKind::Foreach { array, body, .. } => {
            collect_global_scope_expr_error(array, errors);
            for s in body {
                collect_global_scope_error(s, errors);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            for s in try_body {
                collect_global_scope_error(s, errors);
            }
            for c in catches {
                for s in &c.body {
                    collect_global_scope_error(s, errors);
                }
            }
            if let Some(finally_body) = finally_body {
                for s in finally_body {
                    collect_global_scope_error(s, errors);
                }
            }
        }
        StmtKind::Synthetic(stmts) | StmtKind::NamespaceBlock { body: stmts, .. } => {
            for s in stmts {
                collect_global_scope_error(s, errors);
            }
        }
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ConstDecl { value, .. } => collect_global_scope_expr_error(value, errors),
        StmtKind::Return(Some(expr)) => collect_global_scope_expr_error(expr, errors),
        _ => {
            let _ = ExprKind::Null; // keep the `use` above meaningful without a catch-all match arm.
        }
    }
}

/// Emits a `CompileError` at `expr`'s span if it directly calls a
/// `func_num_args`/`func_get_args`/`func_get_arg` intrinsic (not skipping into closures),
/// then recurses to catch further illegal calls in sub-expressions.
fn collect_global_scope_expr_error(expr: &crate::parser::ast::Expr, errors: &mut Vec<CompileError>) {
    use crate::parser::ast::ExprKind;
    if let ExprKind::FunctionCall { name, args } = &expr.kind {
        if detect::is_func_args_intrinsic_name(name.as_str()) {
            errors.push(CompileError::new(
                expr.span,
                &format!(
                    "{}(): Cannot call {}() from the global scope",
                    name.as_str(),
                    name.as_str()
                ),
            ));
        }
        for arg in args {
            collect_global_scope_expr_error(arg, errors);
        }
        return;
    }
    if expr_calls_func_args_intrinsic(expr) {
        // A nested (non-FunctionCall) expression still routes through the shared
        // detector for coverage; report at the outer expression's span since the
        // exact inner call site does not need per-arg granularity here.
        errors.push(CompileError::new(
            expr.span,
            "func_num_args()/func_get_args()/func_get_arg() cannot be called from the global scope",
        ));
    }
}

/// Returns `true` when `args` contains a spread (`...`) argument whose length cannot be
/// determined at compile time. Mirrors `crate::ir_lower::expr`'s
/// `has_static_call_spread_args`/`expand_static_call_spread_args`: a spread of an array OR
/// associative-array LITERAL is statically flattenable (its element count is just the
/// literal's length), so only a spread of anything else (a variable, a function call
/// result, ...) counts as "dynamic" here.
pub(crate) fn call_has_dynamic_spread(args: &[crate::parser::ast::Expr]) -> bool {
    use crate::parser::ast::ExprKind;
    args.iter().any(|arg| match &arg.kind {
        ExprKind::Spread(inner) => !matches!(
            inner.kind,
            ExprKind::ArrayLiteral(_) | ExprKind::ArrayLiteralAssoc(_)
        ),
        _ => false,
    })
}

/// Builds the `CompileError` for a call into an arity-hungry function/method with a
/// dynamic-length spread argument (see `call_has_dynamic_spread`) — the caller-visible
/// counterpart of `crate::ir_lower::expr::func_args_intrinsics`'s defense-in-depth panic.
pub(crate) fn dynamic_spread_call_error(
    callee_desc: &str,
    span: crate::span::Span,
) -> CompileError {
    CompileError::new(
        span,
        &format!(
            "{} calls func_num_args()/func_get_args()/func_get_arg() and cannot be called \
             with a dynamic-length spread (`...`) argument — its element count is not known \
             at compile time; call it with explicit positional/named arguments, or spread a \
             literal array (e.g. `f(...[1, 2, 3])`) instead",
            callee_desc
        ),
    )
}

/// Scans every class method body for a call to `func_num_args`/`func_get_args`/
/// `func_get_arg` and returns one `CompileError` per method that has one — methods are
/// intentionally never marked arity-hungry (see `mark_func_args_functions`'s doc comment:
/// virtual dispatch means a call site cannot always know which concrete override runs), so
/// a method body using one of these builtins is rejected at compile time here instead of
/// risking a call-site ABI mismatch.
pub(crate) fn validate_func_args_method_bodies(
    class_method_bodies: &std::collections::HashMap<String, Vec<(String, bool, Vec<crate::parser::ast::Stmt>)>>,
) -> Vec<CompileError> {
    let mut errors = Vec::new();
    for (class_name, methods) in class_method_bodies {
        for (method_name, _is_static, body) in methods {
            if !body_calls_func_args_intrinsic(body) {
                continue;
            }
            let span = first_func_args_intrinsic_span(body).unwrap_or(crate::span::Span::dummy());
            errors.push(CompileError::new(
                span,
                &format!(
                    "Method {}::{}() calls func_num_args()/func_get_args()/func_get_arg(), \
                     which this compiler does not support in methods — a method call site's \
                     receiver type may not determine which concrete (possibly overriding) \
                     implementation runs, so the caller cannot always supply the required \
                     hidden argument-count information. Extract the logic into a free \
                     function instead.",
                    class_name, method_name
                ),
            ));
        }
    }
    errors
}

/// Finds the span of the first `func_num_args`/`func_get_args`/`func_get_arg` call in
/// `body`, for error reporting. Reuses the same scope-stopping traversal as
/// `detect::body_calls_func_args_intrinsic` by scanning statement-by-statement until one
/// reports a hit.
fn first_func_args_intrinsic_span(body: &[crate::parser::ast::Stmt]) -> Option<crate::span::Span> {
    body.iter()
        .find(|stmt| body_calls_func_args_intrinsic(std::slice::from_ref(stmt)))
        .map(|stmt| stmt.span)
}

/// Computes the set of user function/method canonical keys whose body calls
/// `func_num_args`/`func_get_args`/`func_get_arg` at its own scope, mutating each such
/// signature via `ensure_variadic_for_func_args`. Free functions are keyed by their
/// canonical name (matching `CheckResult::functions`).
///
/// METHODS ARE INTENTIONALLY NOT MARKED (scope cut): a method call site's receiver has a
/// STATIC type that may be a base class/interface, while the concrete implementation that
/// actually runs is chosen by virtual dispatch at runtime — a subclass could override the
/// method WITHOUT calling `func_get_args()` while the base implementation does (or vice
/// versa), so a single call site cannot always know at compile time whether the hidden ABI
/// extension is needed. `class_method_bodies`/`classes` are accepted (and still scanned) so
/// method bodies that DO call these intrinsics fail LOUD — `crate::ir_lower::expr::func_args_intrinsics::lower_func_args_intrinsic`
/// panics with a precise message instead of a call-site ABI mismatch reading garbage — rather
/// than silently miscompiling. See `crate::ir_lower::function::lower_class_method`, which
/// still carries the (currently dead, for future extension) call-site plumbing.
pub(crate) fn mark_func_args_functions(
    functions: &mut std::collections::HashMap<String, FunctionSig>,
    fn_decl_bodies: &std::collections::HashMap<String, Vec<crate::parser::ast::Stmt>>,
    _classes: &mut std::collections::HashMap<String, crate::types::ClassInfo>,
    _class_method_bodies: &std::collections::HashMap<String, Vec<(String, bool, Vec<crate::parser::ast::Stmt>)>>,
) -> HashSet<String> {
    let mut marked = HashSet::new();
    for (name, body) in fn_decl_bodies {
        if !body_calls_func_args_intrinsic(body) {
            continue;
        }
        let Some(sig) = functions.get_mut(name) else {
            continue;
        };
        ensure_variadic_for_func_args(sig);
        marked.insert(name.clone());
    }
    marked
}
