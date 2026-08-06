//! Purpose:
//! Pre-scans Composer `autoload.files` entries for globally declared (non-namespaced) free
//! function names, including ones nested inside `if (!function_exists('X')) { function X() {}
//! }`-style guards, before any name resolution runs.
//!
//! Called from:
//! - `crate::pipeline::compile()`, which installs the result via
//!   `name_resolver::with_known_composer_global_functions` for the span covering the main
//!   name-resolution pass and `autoload::run`.
//!
//! Key details:
//! - Deliberately scoped to `autoload.files` entries and the files they statically `require`/
//!   `include` (see below), not the full PSR-4-referenced class tree. `Registry::
//!   always_included_files()` is known upfront (before any resolution begins), so this scan can
//!   run before it; a PSR-4-referenced class file's own global-function declarations are only
//!   discovered as the reference graph is walked, too late for a pre-pass to see, so calls to
//!   those stay out of scope (the fully general cross-file case is a separate, harder problem).
//! - Uses lex+parse only (no magic-constants/conditional/resolver/name-resolver): only free-function
//!   NAMES are needed, not their bodies. A read or parse failure here is silently skipped — such a
//!   file is either genuinely absent (an edge case in test fixtures) or will surface its own error
//!   later through `autoload::run`'s own (tolerant or strict) loading, so this pre-scan must not be
//!   a second place that can hard-fail compilation.
//! - Recurses into `if`/`while`/`do-while`/`for`/`foreach`/`switch`/`try`/synthetic/include-once-guard
//!   bodies (the same shapes `name_resolver::symbols::collect_symbols` now recurses through) but
//!   never into function/closure/method bodies, matching PHP scoping: a function declared inside
//!   another function is a runtime-local declaration, not a free global one.
//! - Also follows a `require`/`require_once`/`include`/`include_once` statement whose path is
//!   STATICALLY resolvable (a bare string literal, or `__DIR__ . '/literal/suffix.php'`) relative
//!   to the CURRENTLY-scanned file's own directory, and recurses into the target file the same
//!   way. This matters for real Composer polyfills such as
//!   `symfony/polyfill-deepclone/bootstrap.php`, whose `autoload.files` entry only version-dispatches
//!   (`if (\PHP_VERSION_ID >= 80100) { require __DIR__.'/bootstrap81.php'; }`) to a SIBLING file
//!   that holds the actual `if (!function_exists(...)) { function deepclone_to_array() {} }` guard —
//!   the guarding `if` is already recursed into unconditionally (this scanner never evaluates
//!   conditions, it treats every branch as "maybe declares this global" — see the `If` arm below),
//!   so no PHP_VERSION_ID evaluation is needed here, only following the `require` textually. A path
//!   that is not statically resolvable (built from a variable, a function call, etc.) is silently
//!   skipped — same tolerant-miss policy as an unreadable/unparsable file. A `visited` set guards
//!   against `require` cycles between polyfill files.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{BinOp, Expr, ExprKind, MagicConstant, Stmt, StmtKind};

use super::Registry;

/// Scans every `autoload.files` entry in `registry` (plus any file it statically `require`s/
/// `include`s — see the module doc) and returns every globally-declared (non-namespaced)
/// free-function name found, keyed by `php_symbol_key` with the original declared-case name as
/// the value — mirrors `name_resolver::symbols`'s folded-symbol convention so callers can format
/// a resolved name the same way.
pub fn scan_composer_global_functions(registry: &Registry) -> HashMap<String, String> {
    let mut names = HashMap::new();
    let mut visited = HashSet::new();
    for path in registry.always_included_files() {
        scan_file_for_global_function_decls(path, &mut visited, &mut names);
    }
    names
}

/// Reads, lexes, and parses `path` (skipping silently on any failure, matching the pre-scan's
/// tolerant-miss policy), then collects its global function declarations into `out`. Marks `path`
/// visited first so a `require` cycle back to it is a no-op rather than infinite recursion.
fn scan_file_for_global_function_decls(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    out: &mut HashMap<String, String>,
) {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical) {
        return;
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(tokens) = crate::lexer::tokenize(&content) else {
        return;
    };
    let Ok(program) = crate::parser::parse(&tokens) else {
        return;
    };
    let dir = path.parent().unwrap_or_else(|| Path::new(""));
    collect_global_function_decls(&program, None, dir, visited, out);
}

/// Recursively collects top-level (namespace-empty) `FunctionDecl` names from `stmts` into `out`,
/// tracking the innermost enclosing namespace so a declaration inside `namespace X { ... }` is
/// correctly excluded (only the GLOBAL fallback target matters here; a namespaced declaration is
/// already visible to `name_resolver::symbols::collect_symbols`'s own namespace-local lookup).
/// `dir` is the directory of the file currently being scanned, used to resolve statically-known
/// `require`/`include` targets relative to it.
fn collect_global_function_decls(
    stmts: &[Stmt],
    current_namespace: Option<&str>,
    dir: &Path,
    visited: &mut HashSet<PathBuf>,
    out: &mut HashMap<String, String>,
) {
    let mut namespace = current_namespace.map(str::to_string);
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::NamespaceDecl { name } => {
                namespace = Some(namespace_name(name));
            }
            StmtKind::NamespaceBlock { name, body } => {
                collect_global_function_decls(body, Some(&namespace_name(name)), dir, visited, out);
            }
            StmtKind::FunctionDecl { name, .. } => {
                if namespace.as_deref().is_none_or(str::is_empty) {
                    out.entry(php_symbol_key(name)).or_insert_with(|| name.clone());
                }
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_global_function_decls(then_body, namespace.as_deref(), dir, visited, out);
                for (_, body) in elseif_clauses {
                    collect_global_function_decls(body, namespace.as_deref(), dir, visited, out);
                }
                if let Some(body) = else_body {
                    collect_global_function_decls(body, namespace.as_deref(), dir, visited, out);
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. } => {
                collect_global_function_decls(body, namespace.as_deref(), dir, visited, out);
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    collect_global_function_decls(body, namespace.as_deref(), dir, visited, out);
                }
                if let Some(body) = default {
                    collect_global_function_decls(body, namespace.as_deref(), dir, visited, out);
                }
            }
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                collect_global_function_decls(try_body, namespace.as_deref(), dir, visited, out);
                for catch in catches {
                    collect_global_function_decls(&catch.body, namespace.as_deref(), dir, visited, out);
                }
                if let Some(body) = finally_body {
                    collect_global_function_decls(body, namespace.as_deref(), dir, visited, out);
                }
            }
            StmtKind::Synthetic(body) | StmtKind::IncludeOnceGuard { body, .. } => {
                collect_global_function_decls(body, namespace.as_deref(), dir, visited, out);
            }
            StmtKind::Include { path, .. } => {
                if let Some(target) = resolve_static_include_path(path, dir) {
                    scan_file_for_global_function_decls(&target, visited, out);
                }
            }
            // A VALUE-position include (`return require __DIR__.'/bootstrap80.php';`) is a
            // statement carrying an `IncludeValue` expression rather than a `StmtKind::Include`,
            // and it is how `symfony/polyfill-mbstring` (and the intl/php8x polyfills) dispatch.
            // Following only the statement form would leave every `mb_*` without its global
            // fallback in namespaced callers.
            StmtKind::Return(Some(expr))
            | StmtKind::ExprStmt(expr)
            | StmtKind::Echo(expr)
            | StmtKind::Assign { value: expr, .. }
            | StmtKind::TypedAssign { value: expr, .. } => {
                follow_value_position_includes(expr, dir, visited, out);
            }
            _ => {}
        }
    }
}

/// Follows every statically-resolvable value-position `include`/`require` reachable from `expr`.
///
/// Recursion is limited to the operand-carrying expression shapes an include realistically appears
/// under (`$x = A ?: require F`, `$x = $y ?? require F`, a parenthesized/negated form, …). Any
/// other shape is left unfollowed, matching the module's tolerant-miss policy.
fn follow_value_position_includes(
    expr: &Expr,
    dir: &Path,
    visited: &mut HashSet<PathBuf>,
    out: &mut HashMap<String, String>,
) {
    match &expr.kind {
        ExprKind::IncludeValue { path, .. } => {
            if let Some(target) = resolve_static_include_path(path, dir) {
                scan_file_for_global_function_decls(&target, visited, out);
            }
        }
        ExprKind::BinaryOp { left, right, .. } => {
            follow_value_position_includes(left, dir, visited, out);
            follow_value_position_includes(right, dir, visited, out);
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            follow_value_position_includes(condition, dir, visited, out);
            follow_value_position_includes(then_expr, dir, visited, out);
            follow_value_position_includes(else_expr, dir, visited, out);
        }
        ExprKind::ShortTernary { value, default } => {
            follow_value_position_includes(value, dir, visited, out);
            follow_value_position_includes(default, dir, visited, out);
        }
        ExprKind::Not(inner) | ExprKind::BitNot(inner) => {
            follow_value_position_includes(inner, dir, visited, out);
        }
        _ => {}
    }
}

/// Statically resolves an `include`/`require` path expression to an absolute file path when it is
/// shaped as a bare string literal or as `__DIR__ . '/literal/suffix'` (the concat order real
/// Composer polyfills use), relative to `dir`. Returns `None` for any other shape (a variable, a
/// function call, `__FILE__`-based paths, etc.) — those are dynamic from this pre-scan's
/// perspective and are silently left unfollowed, matching the module's tolerant-miss policy.
fn resolve_static_include_path(path: &Expr, dir: &Path) -> Option<PathBuf> {
    match &path.kind {
        ExprKind::StringLiteral(literal) => Some(dir.join(literal.trim_start_matches('/'))),
        ExprKind::BinaryOp { left, op: BinOp::Concat, right } => {
            if !matches!(left.kind, ExprKind::MagicConstant(MagicConstant::Dir)) {
                return None;
            }
            let ExprKind::StringLiteral(suffix) = &right.kind else {
                return None;
            };
            Some(dir.join(suffix.trim_start_matches('/')))
        }
        _ => None,
    }
}

/// Extracts the namespace name as a canonical string from an optional `Name`, matching
/// `name_resolver`'s own `namespace_name` helper (kept as a small local copy since that one is
/// private to `name_resolver`).
fn namespace_name(name: &Option<Name>) -> String {
    name.as_ref().map(Name::as_canonical).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::Expr;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A bare string-literal include path resolves relative to the scanning file's directory.
    #[test]
    fn resolves_bare_string_literal_path() {
        let dir = Path::new("/vendor/pkg");
        let path = Expr::string_lit("helper.php");
        assert_eq!(
            resolve_static_include_path(&path, dir),
            Some(dir.join("helper.php"))
        );
    }

    /// `__DIR__ . '/sibling.php'` — the exact shape real Composer polyfills use (e.g.
    /// `symfony/polyfill-deepclone/bootstrap.php`'s `require __DIR__.'/bootstrap81.php';`) —
    /// resolves relative to the scanning file's directory.
    #[test]
    fn resolves_dir_concat_literal_path() {
        let dir = Path::new("/vendor/symfony/polyfill-deepclone");
        let path = Expr::binop(
            Expr::new(ExprKind::MagicConstant(MagicConstant::Dir), crate::span::Span::dummy()),
            BinOp::Concat,
            Expr::string_lit("/bootstrap81.php"),
        );
        assert_eq!(
            resolve_static_include_path(&path, dir),
            Some(dir.join("bootstrap81.php"))
        );
    }

    /// A dynamically-built path (e.g. a variable) is not statically resolvable and is skipped.
    #[test]
    fn rejects_dynamic_path() {
        let dir = Path::new("/vendor/pkg");
        assert_eq!(resolve_static_include_path(&Expr::var("path"), dir), None);
    }

    /// `__FILE__ . '/x.php'` (as opposed to `__DIR__`) is not a recognized static shape.
    #[test]
    fn rejects_non_dir_magic_constant_concat() {
        let dir = Path::new("/vendor/pkg");
        let path = Expr::binop(
            Expr::new(ExprKind::MagicConstant(MagicConstant::File), crate::span::Span::dummy()),
            BinOp::Concat,
            Expr::string_lit("/x.php"),
        );
        assert_eq!(resolve_static_include_path(&path, dir), None);
    }

    /// End-to-end: a Composer `autoload.files` entry that only version-dispatches
    /// (`if (...) { require __DIR__.'/bootstrap81.php'; }`) to a sibling file holding the real
    /// `if (!function_exists('deepclone_to_array')) { function deepclone_to_array() {} }` guard
    /// is followed, so the sibling's global function is discovered — reproducing the exact shape
    /// of `symfony/polyfill-deepclone` that motivated this fix.
    #[test]
    fn follows_static_require_into_sibling_file_for_guarded_function() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("elephc_composer_global_fns_test_{}_{}", std::process::id(), n));
        let vendor_dir = root.join("vendor").join("pkg");
        std::fs::create_dir_all(&vendor_dir).expect("create temp vendor dir");

        std::fs::write(
            root.join("composer.json"),
            r#"{"autoload": {"files": ["vendor/pkg/bootstrap.php"]}}"#,
        )
        .expect("write composer.json");
        std::fs::write(
            vendor_dir.join("bootstrap.php"),
            "<?php\nif (\\PHP_VERSION_ID >= 80100) {\n    require __DIR__.'/bootstrap81.php';\n}\n",
        )
        .expect("write bootstrap.php");
        std::fs::write(
            vendor_dir.join("bootstrap81.php"),
            "<?php\nif (!function_exists('deepclone_to_array')) {\n    function deepclone_to_array(mixed $value): array { return []; }\n}\n",
        )
        .expect("write bootstrap81.php");

        let (registry, _program) = super::super::Registry::build(&root, Vec::new());
        let names = scan_composer_global_functions(&registry);

        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(
            names.get(&php_symbol_key("deepclone_to_array")),
            Some(&"deepclone_to_array".to_string())
        );
    }

    /// The other dispatch shape real Composer polyfills use: `symfony/polyfill-mbstring`'s
    /// `autoload.files` entry dispatches with `return require __DIR__.'/bootstrap80.php';` — a
    /// value-position include, which is a `Return` statement carrying an `IncludeValue`
    /// EXPRESSION, not a `StmtKind::Include`. It must be followed just like the statement form,
    /// or every `mb_*` call in a namespaced file loses its global fallback.
    #[test]
    fn follows_value_position_require_into_sibling_file_for_guarded_function() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "elephc_composer_global_fns_value_test_{}_{}",
            std::process::id(),
            n
        ));
        let vendor_dir = root.join("vendor").join("pkg");
        std::fs::create_dir_all(&vendor_dir).expect("create temp vendor dir");

        std::fs::write(
            root.join("composer.json"),
            r#"{"autoload": {"files": ["vendor/pkg/bootstrap.php"]}}"#,
        )
        .expect("write composer.json");
        std::fs::write(
            vendor_dir.join("bootstrap.php"),
            "<?php\nif (\\PHP_VERSION_ID >= 80000) {\n    return require __DIR__.'/bootstrap80.php';\n}\n\nreturn require __DIR__.'/bootstrap72.php';\n",
        )
        .expect("write bootstrap.php");
        std::fs::write(
            vendor_dir.join("bootstrap80.php"),
            "<?php\nif (!function_exists('mb_str_split')) {\n    function mb_str_split(?string $string): array { return []; }\n}\n",
        )
        .expect("write bootstrap80.php");
        std::fs::write(
            vendor_dir.join("bootstrap72.php"),
            "<?php\nif (!function_exists('mb_str_split')) {\n    function mb_str_split($string) { return []; }\n}\n",
        )
        .expect("write bootstrap72.php");

        let (registry, _program) = super::super::Registry::build(&root, Vec::new());
        let names = scan_composer_global_functions(&registry);

        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(
            names.get(&php_symbol_key("mb_str_split")),
            Some(&"mb_str_split".to_string())
        );
    }
}
