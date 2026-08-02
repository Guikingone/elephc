//! Purpose:
//! Appends PHP's implicit "fell off the end of a value-returning function" `TypeError` to any
//! function or method body that can reach its closing brace with a non-`void` declared return
//! type.
//!
//! Called from:
//! - `crate::pipeline::compile()`, after name resolution (so declaration names are already
//!   canonical) and immediately before type checking.
//!
//! Key details:
//! - PHP performs NO static return-coverage analysis. `function f(): int { if (false) return 1; }`
//!   compiles, and only the call that actually falls off the end raises a CATCHABLE `TypeError`
//!   (php -n 8.5 verified). elephc used to reject such a body outright
//!   (`Method 'C::m' must return a value on every path`), which refuses programs PHP runs — a
//!   `foreach ($items as $i) { return $i; }` body and any path ending in `goto` are the shapes
//!   that hit it in `symfony/cache` and `symfony/dependency-injection`.
//! - Materializing the failure as a real `throw new TypeError(...)` in the AST, rather than
//!   relaxing the checker alone, is what keeps the semantics honest end to end: the checker sees
//!   a body that always exits (so its coverage rule passes on its own terms), the backend lowers
//!   an ordinary throw, and the error is catchable exactly where PHP makes it catchable. It also
//!   replaces `ir_lower::function::terminate_open_block`'s placeholder return value on this path,
//!   which would otherwise hand the caller a silently fabricated value.
//! - CLOSURES ARE DELIBERATELY EXCLUDED. PHP names a closure in this message by source position
//!   (`{closure:/abs/path.php:29}(): Return value must be of type int, none returned`), and
//!   elephc has no per-declaration file identity (the resolver inlines every include into one AST
//!   and `scan_reflection_source_files` stamps the entry file on everything). Emitting the throw
//!   with a wrong name would trade a loud compile error for a quietly wrong runtime message, so
//!   closures keep the existing diagnostic until file identity exists.
//! - GENERATORS ARE EXCLUDED: a `yield`-bodied function legitimately runs off the end, and PHP
//!   raises nothing (php -n verified).
//! - The guard is appended only when the body cannot be PROVEN to exit. A body that always
//!   returns is left untouched, so the common case emits no extra code at all.

use crate::names::Name;
use crate::parser::ast::{
    ClassMethod, Expr, ExprKind, Program, Stmt, StmtKind, TypeExpr,
};
use crate::span::Span;

/// Appends the implicit return-type `TypeError` wherever a value-returning body can fall through.
pub fn inject(program: Program) -> Program {
    program.into_iter().map(rewrite_stmt).collect()
}

/// Rewrites one top-level statement, recursing into class-like declarations.
fn rewrite_stmt(mut stmt: Stmt) -> Stmt {
    match &mut stmt.kind {
        StmtKind::FunctionDecl {
            name,
            return_type,
            body,
            ..
        } => {
            guard_body(body, return_type.as_ref(), name, stmt.span);
        }
        StmtKind::ClassDecl { name, methods, .. }
        | StmtKind::TraitDecl { name, methods, .. }
        | StmtKind::EnumDecl { name, methods, .. } => {
            let class_name = name.clone();
            for method in methods.iter_mut() {
                guard_method(method, &class_name);
            }
        }
        _ => {}
    }
    stmt
}

/// Applies the guard to one class-like method, skipping bodiless declarations.
fn guard_method(method: &mut ClassMethod, class_name: &str) {
    if !method.has_body || method.is_abstract {
        return;
    }
    let qualified = format!("{}::{}", class_name, method.name);
    let span = method.span;
    guard_body(&mut method.body, method.return_type.as_ref(), &qualified, span);
}

/// Appends `throw new TypeError("<name>(): Return value must be of type <T>, none returned");`
/// when `body` has a value-returning declared type it can reach the end of.
fn guard_body(
    body: &mut Vec<Stmt>,
    return_type: Option<&TypeExpr>,
    qualified_name: &str,
    span: Span,
) {
    let Some(declared) = return_type else {
        return; // No declared type: PHP checks nothing, and neither does elephc.
    };
    if matches!(declared, TypeExpr::Void | TypeExpr::Never) {
        return;
    }
    // A generator's body legitimately runs off its end; PHP raises nothing there.
    if crate::types::checker::yield_validation::body_contains_yield(body) {
        return;
    }
    // "Can control reach the closing brace" — PHP's own question, and exactly the predicate
    // `types::warnings::unreachable` uses. NOT `block_guarantees_function_exit`: that asks
    // whether the body leaves the FUNCTION, which a `goto` does not, so a body ending in
    // `goto` would get a guard appended AND then be reported as unreachable code by the
    // warning pass (measured on `Cache\PhpArrayAdapter::get`'s `catch { … goto … }`).
    if body.iter().any(crate::termination::stmt_guarantees_termination) {
        return;
    }
    let message = format!(
        "{}(): Return value must be of type {}, none returned",
        qualified_name,
        php_type_spelling(declared)
    );
    body.push(Stmt {
        kind: StmtKind::Throw(Expr {
            kind: ExprKind::NewObject {
                class_name: Name::unqualified("TypeError"),
                args: vec![Expr {
                    kind: ExprKind::StringLiteral(message),
                    span,
                }],
            },
            span,
        }),
        span,
        attributes: Vec::new(),
    });
}

/// Spells a declared type the way PHP spells it back in a `TypeError`.
///
/// PHP echoes the DECLARED type, not an inferred one: `?string` stays `?string`, a class name
/// keeps its fully-qualified form, and `mixed` is reported as `mixed` even though it admits null
/// (php -n verified: `m(): Return value must be of type mixed, none returned`).
///
/// Union members are reordered by PHP's own canonical ranking rather than source order —
/// `string|array` is reported as `array|string`, `int|string|null` as `string|int|null`. That
/// ranking already exists for the checked-downcast diagnostics, so this reuses the same order
/// (class arms first in source order, then array, string, int, float, bool, false, and null last).
fn php_type_spelling(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Int => "int".to_string(),
        TypeExpr::Float => "float".to_string(),
        TypeExpr::Bool => "bool".to_string(),
        TypeExpr::False => "false".to_string(),
        TypeExpr::Str => "string".to_string(),
        TypeExpr::Void => "void".to_string(),
        TypeExpr::Never => "never".to_string(),
        // php -n: a `: iterable` function reports the EXPANDED `Traversable|array`.
        TypeExpr::Iterable => "Traversable|array".to_string(),
        TypeExpr::Array(_) => "array".to_string(),
        TypeExpr::Buffer(_) => "array".to_string(),
        TypeExpr::Ptr(_) => "mixed".to_string(),
        TypeExpr::Named(name) => name.as_str().trim_start_matches('\\').to_string(),
        TypeExpr::Nullable(inner) => format!("?{}", php_type_spelling(inner)),
        TypeExpr::Intersection(members) => members
            .iter()
            .map(php_type_spelling)
            .collect::<Vec<_>>()
            .join("&"),
        TypeExpr::Union(members) => {
            let mut ordered: Vec<&TypeExpr> = members.iter().collect();
            ordered.sort_by_key(|member| union_arm_rank(member));
            ordered
                .into_iter()
                .map(php_type_spelling)
                .collect::<Vec<_>>()
                .join("|")
        }
    }
}

/// A union member's rank in PHP's canonical rendering. A STABLE sort on it keeps source order
/// within each rank, which is what makes several class arms come out in declaration order
/// (php -n: `Zed|Alpha|null` stays `Zed|Alpha|null`, while `int|string|null` becomes
/// `string|int|null`). Mirrors `ir_lower::checked_downcast::union_arm_rank`.
fn union_arm_rank(member: &TypeExpr) -> u8 {
    match member {
        TypeExpr::Named(_) | TypeExpr::Intersection(_) => 0,
        TypeExpr::Iterable => 2,
        TypeExpr::Array(_) | TypeExpr::Buffer(_) => 3,
        TypeExpr::Str => 4,
        TypeExpr::Int => 5,
        TypeExpr::Float => 6,
        TypeExpr::Bool => 7,
        TypeExpr::False => 8,
        // PHP always renders the null arm last.
        TypeExpr::Void | TypeExpr::Nullable(_) => 10,
        _ => 9,
    }
}
