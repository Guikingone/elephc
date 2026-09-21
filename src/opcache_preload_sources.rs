//! Purpose:
//! Turns the file `opcache.preload` names into compile-time source, so its declarations join the
//! closed world without its startup statements running once per request.
//!
//! Called from:
//! - `crate::pipeline::compile()`, immediately before the autoload pass.
//!
//! Key details:
//! - Only the `require`/`include` operations the file performs are kept; everything else is dropped.
//! - The resolver follows each one, so the whole preloaded world arrives as ordinary declarations.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind, Program, Stmt, StmtKind};
use crate::span::Span;

/// Loads the program `opcache.preload` describes, as declarations for the closed world.
///
/// php preloads at STARTUP: it runs the file once, before any request, and what survives is the
/// symbol table it built. A compiled binary has no startup — its `--web` handler re-runs the
/// top-level body for every request — so splicing the file in whole would re-run its startup work
/// on each one. Symfony's generated preload file makes that concrete and expensive: it ends with
/// `Preloader::preload($classes)`, which reflects over every class it loaded, and a request that
/// did that did not finish.
///
/// What a preload file means to a compiler is therefore its REQUIRES: the set of files whose
/// declarations become permanently available. Those are kept, in order, and everything else in the
/// file is dropped — the SAPI guard, the container's `->set()`, the `Preloader` call. The resolver
/// then follows each require exactly as it follows one written in the entry file, so the whole
/// preloaded world arrives as ordinary declarations with no per-request cost at all.
///
/// A require whose path is not statically resolvable contributes nothing rather than being guessed
/// at, which is the same position every other dynamic include is already in.
///
/// The result is DECLARATIONS ONLY. That is what survives a php preload: the symbol table it built,
/// not the work it did building it. Keeping the executable statements too was measured, and it is
/// the whole reason this filter exists -- Symfony's generated preload file ends with
/// `Preloader::preload($classes)`, which reflects over every class it loaded, and a request that
/// re-ran that did not finish (9,474 interpreter frames inside `Preloader` before the timeout).
///
/// The preload graph is resolved on its OWN pass, deliberately: sharing one pass with the entry
/// would let `require_once` mark `vendor/autoload.php` as already included, and the entry's own
/// `require_once` of it would then be skipped -- taking the composer autoloader REGISTRATION with
/// it, which is request work, not preload work. Resolving separately keeps both, and the caller
/// drops any declaration the entry already makes so nothing is declared twice.
pub fn preload_declarations(
    preload_path: &Path,
    base_dir: &Path,
    defines: &HashSet<String>,
) -> Result<Program, CompileError> {
    let content = crate::source::read_physical_source(preload_path).map_err(|error| {
        CompileError::new(
            Span::dummy(),
            &format!(
                "opcache.preload: cannot read '{}': {}",
                preload_path.display(),
                error
            ),
        )
    })?;
    let file_label = preload_path.display().to_string();
    let source_mode = crate::source::SourceMode::from_path(preload_path);
    let tokens = crate::lexer::tokenize_with_mode(&content, source_mode)
        .map_err(|error| error.with_file(file_label.clone()))?;
    let parsed = crate::parser::parse_with_mode(&tokens, source_mode)
        .map_err(|error| error.with_file(file_label.clone()))?;
    // `__DIR__` and `__FILE__` are substituted here, which is what makes the paths foldable:
    // Symfony writes every require as `__DIR__.'/…'` or `dirname(__DIR__, 3).'/…'`.
    let parsed =
        crate::source::finalize_physical_program(parsed, preload_path, source_mode, defines)?;
    let requires = include_statements(&parsed);
    if requires.is_empty() {
        return Ok(Vec::new());
    }
    let (resolved, _, _) = crate::resolver::resolve_collecting_includes_with_defines_and_sources(
        requires,
        preload_path.parent().unwrap_or(base_dir),
        defines,
    )?;
    let resolved = crate::name_resolver::resolve(resolved)?;
    Ok(retain_declarations(resolved))
}

/// Keeps the declarations of one resolved program and drops everything else.
///
/// A namespace block is flattened rather than kept: its declarations have already been
/// canonicalized by the name resolver, so the block is only a scope marker by this point and
/// keeping it would re-scope the statements that follow it in the caller's program.
fn retain_declarations(program: Program) -> Program {
    let mut kept = Vec::new();
    retain_declarations_in(program, &mut kept);
    kept
}

/// Appends every declaration in one statement list to `out`, recursing into grouping statements.
fn retain_declarations_in(body: Program, out: &mut Program) {
    for stmt in body {
        match stmt.kind {
            StmtKind::ClassDecl { .. }
            | StmtKind::InterfaceDecl { .. }
            | StmtKind::TraitDecl { .. }
            | StmtKind::EnumDecl { .. }
            | StmtKind::FunctionDecl { .. }
            | StmtKind::ConstDecl { .. } => out.push(stmt),
            StmtKind::NamespaceBlock { body, .. }
            | StmtKind::Synthetic(body)
            | StmtKind::IncludeOnceGuard { body, .. } => retain_declarations_in(body, out),
            _ => {}
        }
    }
}

/// Returns the names every declaration in one program introduces.
///
/// Used by the caller to drop a preloaded declaration the entry program makes for itself: both
/// halves reach `vendor/autoload.php`, and php declares each class once.
pub fn declared_names(program: &[Stmt]) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_declared_names(program, &mut names);
    names
}

/// Appends the declaration names of one statement list, recursing into grouping statements.
fn collect_declared_names(body: &[Stmt], out: &mut HashSet<String>) {
    for stmt in body {
        match &stmt.kind {
            StmtKind::ClassDecl { name, .. }
            | StmtKind::InterfaceDecl { name, .. }
            | StmtKind::TraitDecl { name, .. }
            | StmtKind::EnumDecl { name, .. }
            | StmtKind::FunctionDecl { name, .. }
            | StmtKind::ConstDecl { name, .. } => {
                out.insert(declaration_key(name));
            }
            StmtKind::NamespaceBlock { body, .. }
            | StmtKind::Synthetic(body)
            | StmtKind::IncludeOnceGuard { body, .. } => collect_declared_names(body, out),
            _ => {}
        }
    }
}

/// Drops every declaration in `preloaded` whose name `declared` already contains.
pub fn without_redeclarations(preloaded: Program, declared: &HashSet<String>) -> Program {
    preloaded
        .into_iter()
        .filter(|stmt| match &stmt.kind {
            StmtKind::ClassDecl { name, .. }
            | StmtKind::InterfaceDecl { name, .. }
            | StmtKind::TraitDecl { name, .. }
            | StmtKind::EnumDecl { name, .. }
            | StmtKind::FunctionDecl { name, .. }
            | StmtKind::ConstDecl { name, .. } => !declared.contains(&declaration_key(name)),
            _ => true,
        })
        .collect()
}

/// Canonicalizes a declaration name the way every other symbol table in the compiler keys one.
fn declaration_key(name: &str) -> String {
    crate::names::php_symbol_key(name.trim_start_matches('\\'))
}

/// Keeps one bare include statement per `require`/`include` the program performs, in source order.
fn include_statements(program: &[Stmt]) -> Program {
    let mut kept = Vec::new();
    collect_include_statements(program, &mut kept);
    kept
}

/// Walks one statement list, appending a bare include statement for every include it performs.
fn collect_include_statements(body: &[Stmt], out: &mut Program) {
    for stmt in body {
        match &stmt.kind {
            StmtKind::Include { .. } => out.push(stmt.clone()),
            // A preload file's body is a flat list in practice, but a `namespace {}` block or an
            // `if` guard around the requires costs nothing to look inside.
            StmtKind::NamespaceBlock { body, .. } | StmtKind::Synthetic(body) => {
                collect_include_statements(body, out);
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_include_statements(then_body, out);
                for (_, body) in elseif_clauses {
                    collect_include_statements(body, out);
                }
                if let Some(body) = else_body {
                    collect_include_statements(body, out);
                }
            }
            StmtKind::ExprStmt(expr) => collect_include_exprs(expr, stmt.span, out),
            _ => {}
        }
    }
}

/// Appends a bare include statement for the include operation inside one expression, if any.
///
/// The operation is kept even when it sits inside a larger expression, because that is how a
/// generated container writes it: `(require __DIR__.'/Container.php')->set(…)` requires the file
/// and then calls a method on what it returned. The require is the part that declares something;
/// the method call is startup work, and dropping it is the point.
///
/// The shapes looked into are the ones a require is ever written in — a method call's receiver, an
/// assignment's value, an operand, a call argument. An include buried anywhere else contributes
/// nothing, which is the same answer a dynamic include already gets.
fn collect_include_exprs(expr: &Expr, span: Span, out: &mut Program) {
    match &expr.kind {
        ExprKind::IncludeValue {
            path,
            once,
            required,
        } => out.push(Stmt::new(
            StmtKind::Include {
                path: (**path).clone(),
                once: *once,
                required: *required,
            },
            span,
        )),
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            collect_include_exprs(object, span, out);
            for arg in args {
                collect_include_exprs(arg, span, out);
            }
        }
        ExprKind::Assignment { target, value, .. } => {
            collect_include_exprs(target, span, out);
            collect_include_exprs(value, span, out);
        }
        ExprKind::BinaryOp { left, right, .. } => {
            collect_include_exprs(left, span, out);
            collect_include_exprs(right, span, out);
        }
        ExprKind::FunctionCall { args, .. } | ExprKind::StaticMethodCall { args, .. } => {
            for arg in args {
                collect_include_exprs(arg, span, out);
            }
        }
        _ => {}
    }
}

/// Resolves the `opcache.preload` directive to a path.
///
/// A relative directive is resolved against the entry file's directory: php resolves a relative
/// `opcache.preload` against the process's working directory at startup, which is the deployment's
/// rather than the compiler's, and the entry's directory is the closest thing a compile has to it.
pub fn resolve_preload_path(directive: &str, base_dir: &Path) -> Option<PathBuf> {
    let trimmed = directive.trim();
    if trimmed.is_empty() {
        return None;
    }
    let candidate = PathBuf::from(trimmed);
    let candidate = if candidate.is_absolute() {
        candidate
    } else {
        base_dir.join(candidate)
    };
    Some(candidate.canonicalize().unwrap_or(candidate))
}
