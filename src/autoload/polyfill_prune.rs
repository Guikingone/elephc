//! Purpose:
//! Compile-time pruning of PHP polyfill redefinition guards for functions elephc
//! declares to provide. Removes `if (!function_exists('X')) { function X() {...} }`
//! (and the inverse `if (function_exists('X'))`) blocks for an allowlisted set of
//! provided functions so their bodies — and any class references those bodies
//! carry — never enter the autoload reference graph.
//!
//! Called from:
//! - `crate::autoload::run()`, after the always-included prefix is assembled and
//!   before class-reference collection.
//!
//! Key details:
//! - Only an explicit allowlist (`PROVIDED_POLYFILL_FUNCTIONS`) is pruned, keeping
//!   the blast radius small: unrelated `function_exists` guards are left untouched.
//! - The rewrite is purely structural over `if` / `!` / `function_exists('literal')`.
//!   It never reasons about control flow, returns, or runtime values, so it cannot
//!   change the behavior of any code outside a matched guard.
//! - Guards with `elseif` clauses are left alone; the polyfill idiom never uses them
//!   and skipping them avoids having to model PHP's elseif-chain evaluation.

use std::collections::HashSet;

// `collect_called_function_names` is only referenced by the `#[cfg(test)]` convenience wrapper
// `prune_unused_optional_helpers`; gate the import so non-test builds stay warning-free.
#[cfg(test)]
use super::walk::collect_called_function_names;
use crate::parser::ast::{Expr, ExprKind, Program, Stmt, StmtKind};

/// Functions whose PHP polyfill redefinition guards are pruned at compile time, either because
/// elephc genuinely provides them, OR (see the `deepclone` case below) because the vendor
/// polyfill body they guard cannot be compiled at all and letting it through is verified to be
/// strictly worse than a clean "Undefined function" diagnostic.
///
/// `deepclone_to_array`/`deepclone_from_array`/`deepclone_hydrate` (the PHP 8.5 `deepclone`
/// surface `symfony/polyfill-deepclone` guards) were briefly REMOVED from this list during the M1
/// easy sweep on the theory that elephc doesn't implement them (true) so the guard should stay
/// intact and let the real vendor body compile instead (WRONG — reverted after `--web` sentinel
/// verification below). Restored here with the corrected root cause:
/// - The vendor guard's then-branch declares `deepclone_to_array()` etc. delegating to
///   `Symfony\Polyfill\DeepClone\DeepClone::toArray()`/`fromArray()`/`hydrate()`, which PSR-4
///   autoload then has to resolve and PARSE (`vendor/symfony/polyfill-deepclone/DeepClone.php`,
///   ~97 KB). That file uses DYNAMIC first-class-callable syntax elephc's parser does not support
///   at all — `return $name(...);`, `$obj->$name(...)`, `$obj::$name(...)` (callee is a runtime
///   variable, not a literal name) — and fails with a hard parse error ("Unexpected token: RParen"
///   at `DeepClone.php:1268`), which `crate::pipeline::compile()` treats as fatal
///   (`process::exit(1)` right after reporting it).
/// - Verified with the `--web` sentinel on `examples/symfony-app`: leaving the guard un-pruned
///   collapses the diagnostic corpus from 864 real errors (spanning the whole app) down to 83
///   parse-cascade errors from ONLY `DeepClone.php` — the entire rest of the program is never even
///   type-checked, since the pipeline aborts at the parse stage. That is an anomalous drop per this
///   campaign's sentinel discipline, not a fix: it trades 4 clean, informative "Undefined function"
///   diagnostics for hiding ~99% of the corpus's signal.
/// - So these three stay pruned — NOT because elephc provides a working reimplementation (it does
///   not; no catalog entry, no prelude, no EIR lowering exists for any of them), but because
///   pruning the guard is the only way to keep the unparseable vendor body out of the reference
///   graph. A bare/namespaced call to any of the three still fails loudly as "Undefined function"
///   (via `crate::autoload::composer_global_functions`'s scan feeding the namespace fallback with
///   the correct unqualified name, now that the scan follows the guard's `require
///   __DIR__.'/bootstrapNN.php'` — see that module's doc — even though the declaration itself is
///   pruned away here) — this is the honestly-reported "keep loud" verdict for the M1 spec's
///   deepclone item, not a silent accept-and-ignore. Extend this list only with names that are
///   either genuinely implemented elsewhere (a real catalog builtin or prelude) or, like this case,
///   provably unparseable/unreachable without the prune — never as a placeholder for a
///   still-missing feature that WOULD otherwise compile. Comparison is case-insensitive to match
///   PHP function-name semantics.
const PROVIDED_POLYFILL_FUNCTIONS: &[&str] = &[
    "deepclone_to_array",
    "deepclone_from_array",
    "deepclone_hydrate",
];

/// Optional `autoload.files` helper functions whose definition guards are pruned when the
/// program never calls them, keeping the heavy classes their bodies reference out of the
/// closed-world closure. Names are canonical fully-qualified, lowercased, to match PHP's
/// case-insensitive, namespace-aware function resolution.
///
/// - `Symfony\Component\String\{u,b,s}` (`symfony/string` `Resources/functions.php`) construct
///   `UnicodeString` / `ByteString`.
/// - `dump` / `dd` (`symfony/var-dumper` `Resources/functions/dump.php`) construct `VarDumper`.
const OPTIONAL_HELPER_FUNCTIONS: &[&str] = &[
    "symfony\\component\\string\\u",
    "symfony\\component\\string\\b",
    "symfony\\component\\string\\s",
    "dump",
    "dd",
];

/// Removes provided-function polyfill guards from the program, replacing each matched
/// `if`/`else` with its statically live branch and recursing into nested bodies.
pub fn prune_provided_function_polyfills(program: Program) -> Program {
    prune_provided_function_polyfills_with(program, PROVIDED_POLYFILL_FUNCTIONS)
}

/// Same as `prune_provided_function_polyfills` but accepts an explicit `provided` allowlist
/// instead of the module's `PROVIDED_POLYFILL_FUNCTIONS` constant, so the pruning MECHANISM can
/// be exercised by tests independently of which (if any) names are currently listed as provided
/// in production.
fn prune_provided_function_polyfills_with(program: Program, provided: &[&str]) -> Program {
    prune_stmt_list(program, &move |stmt| provided_guard_live_branch(stmt, provided))
}

/// Removes definition guards (`if (!function_exists('X')) { function X(...) { ... } }`) for the
/// optional `autoload.files` helpers in `OPTIONAL_HELPER_FUNCTIONS` that the program never calls,
/// so the heavy classes those helper bodies reference (Symfony's `UnicodeString`/`ByteString`
/// behind `u()`/`b()`, `VarDumper` behind `dump()`/`dd()`) never enter the closed-world closure.
///
/// Conservative: a helper is dropped only when no `FunctionCall` or first-class callable names it
/// anywhere in the program assembled so far (the main file plus eagerly-spliced helpers). A later
/// direct call from a lazily-loaded class would surface a clean "undefined function" compile error
/// rather than a miscompile.
///
/// Test-only convenience: the autoload pass now gathers the call set from a survey iteration and
/// calls `prune_unused_optional_helpers_with` directly, so this self-computing wrapper is kept only
/// for unit tests that exercise the prune without a survey.
#[cfg(test)]
pub fn prune_unused_optional_helpers(program: Program) -> Program {
    let called = collect_called_function_names(&program);
    prune_unused_optional_helpers_with(program, &called)
}

/// Same as `prune_unused_optional_helpers` but accepts a precomputed call set instead of recomputing
/// one from `program`. The autoload pass uses this to prune with a call set gathered from a prior
/// survey iteration (which has loaded all PSR-4-referenced caller classes) so a helper called only
/// from a lazily-loaded class is correctly retained.
pub fn prune_unused_optional_helpers_with(program: Program, called: &HashSet<String>) -> Program {
    let called = called.clone();
    prune_stmt_list(program, &move |stmt| {
        unused_optional_guard_live_branch(stmt, &called)
    })
}

/// Removes the definition guards of EVERY optional `autoload.files` helper in
/// `OPTIONAL_HELPER_FUNCTIONS`, regardless of whether the program calls it. Used by the autoload
/// pass for the survey iteration: stripping all optional helper bodies from the survey program
/// keeps the heavy classes those bodies reference (e.g. `UnicodeString`, `VarDumper`) out of the
/// survey's reference graph, so the survey only loads the caller classes (e.g. `OutputFormatter`
/// calling `b()`/`s()` via `use function`) and exposes their calls before the real prune decides
/// which helpers to keep.
pub fn strip_all_optional_helper_guards(program: Program) -> Program {
    prune_stmt_list(program, &any_optional_guard_live_branch)
}

/// Rewrites a statement list, splicing in the live branch of every guard `classify` matches and
/// recursing into the child statement lists of all other statements.
fn prune_stmt_list(
    stmts: Vec<Stmt>,
    classify: &dyn Fn(Stmt) -> Result<Vec<Stmt>, Stmt>,
) -> Vec<Stmt> {
    let mut out = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        match classify(stmt) {
            Ok(live) => out.extend(prune_stmt_list(live, classify)),
            Err(stmt) => out.push(recurse_stmt(stmt, classify)),
        }
    }
    out
}

/// If `stmt` is a provided-function guard (`if (function_exists('X'))` or
/// `if (!function_exists('X'))` with no `elseif` clauses, `X` in `provided`), returns `Ok` with
/// the statically live branch's statements. Otherwise returns `Err(stmt)` unchanged.
fn provided_guard_live_branch(stmt: Stmt, provided: &[&str]) -> Result<Vec<Stmt>, Stmt> {
    let Stmt { kind, span, attributes } = stmt;
    match kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } if elseif_clauses.is_empty() => match provided_function_exists_condition(&condition, provided) {
            // `function_exists('X')` is true for a provided function: the then-branch lives.
            Some(true) => Ok(then_body),
            // `!function_exists('X')` is false for a provided function: the else-branch lives.
            Some(false) => Ok(else_body.unwrap_or_default()),
            None => Err(Stmt {
                kind: StmtKind::If {
                    condition,
                    then_body,
                    elseif_clauses,
                    else_body,
                },
                span,
                attributes,
            }),
        },
        kind => Err(Stmt {
            kind,
            span,
            attributes,
        }),
    }
}

/// Recurses into the child statement lists of control-flow and grouping statements,
/// leaving leaf statements untouched. Declaration bodies (functions, classes) are not
/// descended into: the polyfill idiom never nests its guards there.
fn recurse_stmt(stmt: Stmt, classify: &dyn Fn(Stmt) -> Result<Vec<Stmt>, Stmt>) -> Stmt {
    let Stmt { kind, span, attributes } = stmt;
    let kind = match kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => StmtKind::If {
            condition,
            then_body: prune_stmt_list(then_body, classify),
            elseif_clauses: elseif_clauses
                .into_iter()
                .map(|(cond, body)| (cond, prune_stmt_list(body, classify)))
                .collect(),
            else_body: else_body.map(|body| prune_stmt_list(body, classify)),
        },
        StmtKind::While { condition, body } => StmtKind::While {
            condition,
            body: prune_stmt_list(body, classify),
        },
        StmtKind::DoWhile { body, condition } => StmtKind::DoWhile {
            body: prune_stmt_list(body, classify),
            condition,
        },
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => StmtKind::For {
            init,
            condition,
            update,
            body: prune_stmt_list(body, classify),
        },
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body,
        } => StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body: prune_stmt_list(body, classify),
        },
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => StmtKind::Switch {
            subject,
            cases: cases
                .into_iter()
                .map(|(exprs, body)| (exprs, prune_stmt_list(body, classify)))
                .collect(),
            default: default.map(|body| prune_stmt_list(body, classify)),
        },
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => StmtKind::Try {
            try_body: prune_stmt_list(try_body, classify),
            catches: catches
                .into_iter()
                .map(|mut catch| {
                    catch.body = prune_stmt_list(catch.body, classify);
                    catch
                })
                .collect(),
            finally_body: finally_body.map(|body| prune_stmt_list(body, classify)),
        },
        StmtKind::Synthetic(body) => StmtKind::Synthetic(prune_stmt_list(body, classify)),
        StmtKind::IncludeOnceGuard { label, body } => StmtKind::IncludeOnceGuard {
            label,
            body: prune_stmt_list(body, classify),
        },
        StmtKind::NamespaceBlock { name, body } => StmtKind::NamespaceBlock {
            name,
            body: prune_stmt_list(body, classify),
        },
        other => other,
    };
    Stmt {
        kind,
        span,
        attributes,
    }
}

/// Classifies a statement for unused-optional-helper pruning. Returns `Ok(live_branch)` to
/// replace a definition guard `if (!function_exists('X')) { function X(...) { ... } }` with its
/// (typically empty) else branch when `X` is an optional helper the program never calls; returns
/// `Err(stmt)` to leave the statement in place (and let `recurse_stmt` descend into its bodies).
fn unused_optional_guard_live_branch(
    stmt: Stmt,
    called: &HashSet<String>,
) -> Result<Vec<Stmt>, Stmt> {
    let Stmt { kind, span, attributes } = stmt;
    match kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } if elseif_clauses.is_empty()
            && is_unused_optional_def_guard(&condition, &then_body, called) =>
        {
            Ok(else_body.unwrap_or_default())
        }
        kind => Err(Stmt {
            kind,
            span,
            attributes,
        }),
    }
}

/// Classifies a statement for the survey-time strip of ALL optional helper guards. Returns
/// `Ok(else_branch)` for any `if (!function_exists('X')) { function X(...) { ... } }` whose
/// declared function is in `OPTIONAL_HELPER_FUNCTIONS`, regardless of whether it is called, so
/// the survey program carries no optional helper bodies (and thus none of their class
/// references) into the class-load iteration.
fn any_optional_guard_live_branch(stmt: Stmt) -> Result<Vec<Stmt>, Stmt> {
    let Stmt { kind, span, attributes } = stmt;
    match kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } if elseif_clauses.is_empty() && is_optional_def_guard(&condition, &then_body) =>
        {
            Ok(else_body.unwrap_or_default())
        }
        kind => Err(Stmt {
            kind,
            span,
            attributes,
        }),
    }
}

/// Returns whether an `if` is the definition guard of an optional helper the program never calls:
/// the condition is `!function_exists(...)` and the then-body declares a function whose canonical
/// name is in `OPTIONAL_HELPER_FUNCTIONS` and absent from `called`. Matching the declared function
/// keeps this robust to the guard argument form (`'Name'` vs `Name::class`, which is not yet
/// constant-folded at autoload time).
fn is_unused_optional_def_guard(
    condition: &Expr,
    then_body: &[Stmt],
    called: &HashSet<String>,
) -> bool {
    match optional_helper_name_in_guard(condition, then_body) {
        Some(key) => !called.contains(&key),
        None => false,
    }
}

/// Returns whether an `if` is the definition guard of ANY optional helper: the condition is
/// `!function_exists(...)` and the then-body declares a function whose canonical name is in
/// `OPTIONAL_HELPER_FUNCTIONS`. Unlike `is_unused_optional_def_guard` this ignores the call set,
/// so the survey strip removes every optional helper guard regardless of whether it is called.
fn is_optional_def_guard(condition: &Expr, then_body: &[Stmt]) -> bool {
    optional_helper_name_in_guard(condition, then_body).is_some()
}

/// Returns the canonical lowercased name of the optional helper declared inside a
/// `!function_exists(...)` definition guard, or `None` when the statement is not such a guard.
/// The condition must be a negated `function_exists` call and the then-body must declare a
/// function whose canonical name is in `OPTIONAL_HELPER_FUNCTIONS`. The guard argument form
/// (`'Name'` vs `Name::class`) is not yet constant-folded at autoload time, so the match is made
/// on the declared function name, not the guard argument.
fn optional_helper_name_in_guard(condition: &Expr, then_body: &[Stmt]) -> Option<String> {
    let ExprKind::Not(inner) = &condition.kind else {
        return None;
    };
    let ExprKind::FunctionCall { name, .. } = &inner.kind else {
        return None;
    };
    if !name
        .as_str()
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("function_exists")
    {
        return None;
    }
    then_body.iter().find_map(|stmt| match &stmt.kind {
        StmtKind::FunctionDecl { name, .. } => {
            let key = name.trim_start_matches('\\').to_ascii_lowercase();
            OPTIONAL_HELPER_FUNCTIONS
                .contains(&key.as_str())
                .then_some(key)
        }
        _ => None,
    })
}

/// Classifies an `if` condition as a provided-function existence guard.
///
/// Returns `Some(true)` for `function_exists('X')` and `Some(false)` for
/// `!function_exists('X')` when `X` is in `provided`; `None` otherwise.
fn provided_function_exists_condition(condition: &Expr, provided: &[&str]) -> Option<bool> {
    match &condition.kind {
        ExprKind::Not(inner) => is_provided_function_exists_call(inner, provided).then_some(false),
        _ => is_provided_function_exists_call(condition, provided).then_some(true),
    }
}

/// Returns whether `expr` is a `function_exists('X')` call naming a provided function.
///
/// Matches `function_exists` case-insensitively after trimming a leading namespace
/// separator (PHP resolves the call to the global builtin), and requires a single
/// string-literal argument naming one of `provided`.
fn is_provided_function_exists_call(expr: &Expr, provided: &[&str]) -> bool {
    let ExprKind::FunctionCall { name, args } = &expr.kind else {
        return false;
    };
    if !name
        .as_str()
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("function_exists")
    {
        return false;
    }
    let [arg] = args.as_slice() else {
        return false;
    };
    let ExprKind::StringLiteral(fn_name) = &arg.kind else {
        return false;
    };
    let key = fn_name.trim_start_matches('\\');
    provided
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::Name;
    use crate::span::Span;

    /// Fixture allowlist for exercising the `provided`-function-guard pruning MECHANISM in
    /// isolation from production reality: `PROVIDED_POLYFILL_FUNCTIONS` is currently empty (see
    /// its doc comment — elephc provides none of the deepclone functions it once incorrectly
    /// listed there), so these tests inject their own names via
    /// `prune_provided_function_polyfills_with` instead of asserting against the real constant.
    const TEST_PROVIDED: &[&str] = &["provided_fn_a", "provided_fn_b", "provided_fn_c"];

    /// Builds a `function_exists("name")` call expression.
    fn function_exists_call(name: &str) -> Expr {
        Expr::new(
            ExprKind::FunctionCall {
                name: Name::unqualified("function_exists"),
                args: vec![Expr::new(ExprKind::StringLiteral(name.to_string()), Span::dummy())],
            },
            Span::dummy(),
        )
    }

    /// Builds a placeholder statement standing in for a guarded function definition.
    fn marker_stmt() -> Stmt {
        Stmt::new(
            StmtKind::ExprStmt(Expr::new(ExprKind::IntLiteral(1), Span::dummy())),
            Span::dummy(),
        )
    }

    /// Builds `if (condition) { marker_stmt() }` with no elseif/else clauses.
    fn guard_if(condition: Expr) -> Stmt {
        Stmt::new(
            StmtKind::If {
                condition,
                then_body: vec![marker_stmt()],
                elseif_clauses: Vec::new(),
                else_body: None,
            },
            Span::dummy(),
        )
    }

    /// `if (!function_exists('provided_fn_a')) { def }` is removed entirely:
    /// the function is provided, so the redefinition then-branch is dead.
    #[test]
    fn prunes_negated_guard_for_provided_function() {
        let program = vec![guard_if(Expr::new(
            ExprKind::Not(Box::new(function_exists_call("provided_fn_a"))),
            Span::dummy(),
        ))];
        let pruned = prune_provided_function_polyfills_with(program, TEST_PROVIDED);
        assert!(pruned.is_empty(), "provided-function guard should be removed");
    }

    /// `if (function_exists('provided_fn_b')) { body }` keeps its then-branch:
    /// the function is provided, so the positive condition is statically true.
    #[test]
    fn keeps_then_branch_for_positive_provided_guard() {
        let program = vec![guard_if(function_exists_call("provided_fn_b"))];
        let pruned = prune_provided_function_polyfills_with(program, TEST_PROVIDED);
        assert_eq!(pruned.len(), 1, "live then-branch statement should remain");
        assert!(matches!(pruned[0].kind, StmtKind::ExprStmt(_)));
    }

    /// A guard for a function elephc does not provide is left untouched.
    #[test]
    fn leaves_unrelated_function_exists_guard_intact() {
        let program = vec![guard_if(Expr::new(
            ExprKind::Not(Box::new(function_exists_call("some_other_helper"))),
            Span::dummy(),
        ))];
        let pruned = prune_provided_function_polyfills_with(program, TEST_PROVIDED);
        assert_eq!(pruned.len(), 1, "unrelated guard should be preserved");
        assert!(matches!(pruned[0].kind, StmtKind::If { .. }));
    }

    /// Matching is case-insensitive and tolerant of a leading namespace separator,
    /// since PHP resolves the call to the global builtin regardless of namespace.
    #[test]
    fn matches_case_insensitively_and_through_namespace_separator() {
        let call = Expr::new(
            ExprKind::FunctionCall {
                name: Name::unqualified("\\Function_Exists"),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("Provided_Fn_A".to_string()),
                    Span::dummy(),
                )],
            },
            Span::dummy(),
        );
        let program = vec![guard_if(Expr::new(ExprKind::Not(Box::new(call)), Span::dummy()))];
        let pruned = prune_provided_function_polyfills_with(program, TEST_PROVIDED);
        assert!(pruned.is_empty(), "case/namespace variations should still prune");
    }

    /// Guards nested inside another statement's body are pruned by recursion.
    #[test]
    fn prunes_guard_nested_in_outer_block() {
        let inner = guard_if(Expr::new(
            ExprKind::Not(Box::new(function_exists_call("provided_fn_c"))),
            Span::dummy(),
        ));
        let program = vec![Stmt::new(StmtKind::Synthetic(vec![inner]), Span::dummy())];
        let pruned = prune_provided_function_polyfills_with(program, TEST_PROVIDED);
        assert_eq!(pruned.len(), 1, "outer wrapper should remain");
        let StmtKind::Synthetic(body) = &pruned[0].kind else {
            panic!("expected synthetic wrapper");
        };
        assert!(body.is_empty(), "nested provided-function guard should be removed");
    }

    /// The production `prune_provided_function_polyfills` entry point (using the real
    /// `PROVIDED_POLYFILL_FUNCTIONS` allowlist, not `TEST_PROVIDED`) prunes the `deepclone_*`
    /// guards — see the constant's doc comment for the `--web`-sentinel-verified root cause: this
    /// is deliberate even though elephc has no real implementation of them, because leaving the
    /// guard intact pulls in an unparseable vendor class body and aborts the whole compile.
    #[test]
    fn production_entry_point_prunes_deepclone_guards() {
        assert_eq!(
            PROVIDED_POLYFILL_FUNCTIONS,
            &["deepclone_to_array", "deepclone_from_array", "deepclone_hydrate"]
        );
        let program = vec![guard_if(Expr::new(
            ExprKind::Not(Box::new(function_exists_call("deepclone_to_array"))),
            Span::dummy(),
        ))];
        let pruned = prune_provided_function_polyfills(program);
        assert!(pruned.is_empty(), "deepclone_to_array guard should be pruned in production");
    }

    /// A guard for an UNRELATED function name is left untouched by the production entry point —
    /// the allowlist's blast radius stays scoped to the three `deepclone_*` names.
    #[test]
    fn production_entry_point_leaves_unrelated_guards_alone() {
        let program = vec![guard_if(Expr::new(
            ExprKind::Not(Box::new(function_exists_call("some_other_helper"))),
            Span::dummy(),
        ))];
        let pruned = prune_provided_function_polyfills(program);
        assert_eq!(pruned.len(), 1, "unrelated guard should be preserved");
        assert!(matches!(pruned[0].kind, StmtKind::If { .. }));
    }

    /// Builds `if (!function_exists('name')) { function name() {} }`, the definition-guard shape
    /// the unused-optional-helper prune matches (by the declared function, not the guard argument).
    fn guard_def(name: &str) -> Stmt {
        let def = Stmt::new(
            StmtKind::FunctionDecl {
                name: name.to_string(),
                params: Vec::new(),
                variadic: None,
                variadic_type: None,
                return_type: None,
                by_ref_return: false,
                body: Vec::new(),
                param_attributes: Vec::new(),
                variadic_by_ref: false,
            },
            Span::dummy(),
        );
        Stmt::new(
            StmtKind::If {
                condition: Expr::new(
                    ExprKind::Not(Box::new(function_exists_call(name))),
                    Span::dummy(),
                ),
                then_body: vec![def],
                elseif_clauses: Vec::new(),
                else_body: None,
            },
            Span::dummy(),
        )
    }

    /// Builds a `name();` call statement.
    fn call_stmt(name: &str) -> Stmt {
        Stmt::new(
            StmtKind::ExprStmt(Expr::new(
                ExprKind::FunctionCall {
                    name: Name::unqualified(name),
                    args: Vec::new(),
                },
                Span::dummy(),
            )),
            Span::dummy(),
        )
    }

    /// An optional helper (`dump`) defined but never called is pruned, dropping its definition.
    #[test]
    fn prunes_unused_optional_helper_definition() {
        let program = vec![guard_def("dump")];
        let pruned = prune_unused_optional_helpers(program);
        assert!(pruned.is_empty(), "unused optional helper should be removed");
    }

    /// An optional helper that the program calls is kept, so its definition (and any class its
    /// body would reference) survives.
    #[test]
    fn keeps_called_optional_helper_definition() {
        let program = vec![guard_def("dump"), call_stmt("dump")];
        let pruned = prune_unused_optional_helpers(program);
        assert_eq!(pruned.len(), 2, "called optional helper guard must be preserved");
        assert!(matches!(pruned[0].kind, StmtKind::If { .. }));
    }

    /// A `function_exists` definition guard for a function not in the optional allowlist is left
    /// intact even when uncalled, confirming the prune is allowlist-scoped.
    #[test]
    fn leaves_non_optional_unused_helper_intact() {
        let program = vec![guard_def("my_app_helper")];
        let pruned = prune_unused_optional_helpers(program);
        assert_eq!(pruned.len(), 1, "non-allowlisted guard should be preserved");
        assert!(matches!(pruned[0].kind, StmtKind::If { .. }));
    }

    /// `prune_unused_optional_helpers_with` retains a helper that the supplied call set names,
    /// so the autoload pass can keep a helper called only from a lazily-loaded class once that
    /// caller has been observed by the survey iteration.
    #[test]
    fn prune_with_retains_helper_named_in_supplied_call_set() {
        let program = vec![guard_def("dump")];
        let mut called = HashSet::new();
        called.insert("dump".to_string());
        let pruned = prune_unused_optional_helpers_with(program, &called);
        assert_eq!(
            pruned.len(),
            1,
            "helper named in the supplied call set must be retained"
        );
        assert!(matches!(pruned[0].kind, StmtKind::If { .. }));
    }

    /// `prune_unused_optional_helpers_with` drops a helper absent from the supplied call set,
    /// preserving the `u`-case regression guard: a genuinely-uncalled helper is still pruned even
    /// when the call set came from outside rather than recomputed internally.
    #[test]
    fn prune_with_drops_helper_absent_from_supplied_call_set() {
        let program = vec![guard_def("dump")];
        let called = HashSet::new();
        let pruned = prune_unused_optional_helpers_with(program, &called);
        assert!(
            pruned.is_empty(),
            "helper absent from the supplied call set must be pruned"
        );
    }

    /// `strip_all_optional_helper_guards` removes every optional helper guard regardless of
    /// whether it is called, so the survey program carries no optional helper bodies (and none of
    /// their class references) into the class-load iteration.
    #[test]
    fn strip_all_removes_every_optional_helper_guard() {
        let program = vec![
            guard_def("dump"),
            guard_def("dd"),
            call_stmt("dump"),
        ];
        let stripped = strip_all_optional_helper_guards(program);
        assert_eq!(
            stripped.len(),
            1,
            "all optional helper guards must be stripped, leaving only the non-guard statement"
        );
        assert!(matches!(stripped[0].kind, StmtKind::ExprStmt(_)));
    }

    /// `strip_all_optional_helper_guards` leaves a non-optional helper guard intact, confirming
    /// the survey strip is allowlist-scoped just like the real prune.
    #[test]
    fn strip_all_leaves_non_optional_guard_intact() {
        let program = vec![guard_def("my_app_helper")];
        let stripped = strip_all_optional_helper_guards(program);
        assert_eq!(stripped.len(), 1, "non-allowlisted guard should be preserved");
        assert!(matches!(stripped[0].kind, StmtKind::If { .. }));
    }
}
