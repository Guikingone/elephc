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
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::span::Span;

use super::declarations::strip_discoverable_declarations;
use super::discovery::FunctionVariantRegistry;
use super::engine::resolve_stmts;
use super::files::{parse_file_with_source, resolve_path};
use super::include_path::fold_include_path;
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

/// Resolves a single include/require statement by parsing the target file,
/// recursively resolving its statements, and returning them wrapped in
/// appropriate include_once guards.
///
/// - `once`: when true, skips already-included files and wraps output in `IncludeOnceGuard`
/// - `required`: when true, returns an error if the target file does not exist
/// - `declared_once`: tracks files already processed; updated on return
/// - `include_chain`: current include path for cycle detection; must not contain `canonical`
/// - State (`namespace`, `const_imports`) is saved before recursion and restored after
/// - `preserve_return`: retains a top-level included-file return only when its value is consumed
/// - Returns `None` if the file does not exist and `required` is false, or if a once file was already included
/// - For `once`: wraps body in `IncludeOnceGuard` with the canonical source path
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
    preserve_return: bool,
) -> Result<Option<Vec<Stmt>>, CompileError> {
    if include_chain.len() >= super::MAX_INCLUDE_DEPTH {
        return Err(CompileError::new(
            stmt.span,
            "maximum include depth exceeded",
        ));
    }
    let path_str =
        fold_include_path(path, state).map_err(|msg| CompileError::new(stmt.span, &msg))?;
    let resolved = resolve_path(&path_str, base_dir);
    let canonical = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());

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

    let (included_stmts, source_unit) =
        parse_file_with_source(&resolved, stmt.span, &state.conditional_defines)?;
    state.source_units.record(source_unit, stmt.span)?;
    record_declaration_sources(&included_stmts, &canonical, state)?;

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

    let mut executable =
        strip_discoverable_declarations(resolved_stmts, Some(&canonical), function_variants);
    if !preserve_return {
        discard_statement_include_return(&mut executable);
    }
    if once {
        // Declaration discovery already hoisted compile-time declarations;
        // executable include body statements are guarded so runtime order matches PHP.
        declared_once.insert(canonical.clone());
        return Ok(Some(vec![Stmt::new(
            StmtKind::IncludeOnceGuard {
                source_path: canonical,
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
    declared_once.insert(canonical.clone());
    Ok(Some(vec![
        Stmt::new(
            StmtKind::IncludeOnceMark {
                source_path: canonical,
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
        true,
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
            let captured_return = rewrite_first_include_return(&mut wrapped, &tmp);
            // Pre-seed the default include value of `1` when the included body cannot set the
            // temporary itself: either it has no top-level `return`, or it is an `_once` include
            // whose guarded body may be skipped on a repeat include.
            if !captured_return || once {
                out.push(assign_temp(
                    &tmp,
                    Expr::new(ExprKind::IntLiteral(1), span),
                    span,
                ));
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

/// Converts a top-level included-file return into its side-effecting expression.
///
/// A statement-position include discards its value: `return E` stops the included
/// file after evaluating `E`, but must not return from the caller that contains the
/// include. The resolver inlines static includes, so it must restore that boundary
/// before lowerings see the caller's statements.
fn discard_statement_include_return(body: &mut Vec<Stmt>) {
    for index in 0..body.len() {
        if !matches!(body[index].kind, StmtKind::Return(_)) {
            continue;
        }
        let span = body[index].span;
        let placeholder = Stmt::new(StmtKind::Synthetic(Vec::new()), span);
        let original = std::mem::replace(&mut body[index], placeholder);
        if let StmtKind::Return(Some(value)) = original.kind {
            body[index] = Stmt::new(StmtKind::ExprStmt(value), span);
        }
        body.truncate(index + 1);
        return;
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

/// Rewrites the first top-level `return` inside the wrapped include body to assign the include
/// temporary, dropping any statements after it (they are unreachable once the include returns).
///
/// Recurses through the `IncludeOnceGuard`/`NamespaceBlock` wrappers produced by
/// `resolve_include_stmt`. Returns `true` if a top-level `return` was found and rewritten.
fn rewrite_first_include_return(wrapped: &mut [Stmt], temp: &str) -> bool {
    for stmt in wrapped.iter_mut() {
        match &mut stmt.kind {
            StmtKind::NamespaceBlock { body, .. } => {
                if rewrite_top_level_return(body, temp) {
                    return true;
                }
            }
            StmtKind::IncludeOnceGuard { body, .. } => {
                if rewrite_first_include_return(body, temp) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Replaces the first top-level `return E;` in `body` with `<temp> = E;` (or drops a bare
/// `return;`, leaving the temporary at its default) and truncates the now-unreachable tail.
/// Returns `true` if a top-level `return` was rewritten.
fn rewrite_top_level_return(body: &mut Vec<Stmt>, temp: &str) -> bool {
    for i in 0..body.len() {
        if matches!(body[i].kind, StmtKind::Return(_)) {
            let span = body[i].span;
            let placeholder = Stmt::new(StmtKind::Return(None), span);
            let original = std::mem::replace(&mut body[i], placeholder);
            if let StmtKind::Return(Some(value)) = original.kind {
                body[i] = assign_temp(temp, value, span);
            } else {
                // Bare `return;` carries no value; leave the temporary at its default and drop the
                // statement by replacing it with an empty sequence.
                body[i] = Stmt::new(StmtKind::Synthetic(Vec::new()), span);
            }
            body.truncate(i + 1);
            return true;
        }
    }
    false
}

/// Records which class-likes and functions one PHYSICAL file declares, against that file's path.
///
/// `Reflection*::getFileName()` must report the file a declaration was written in. The only place
/// that knows this is here, while the file's own statements are still separate: once its includes
/// are spliced in, every declaration looks like it came from the same program.
///
/// The statements are not name-resolved yet, so `class Widget` inside `namespace Inc;` is still
/// spelled `Widget`. Canonicalizing by hand would duplicate the name resolver's rules and drift
/// from them, so this reuses `name_resolver::resolve` on a copy and reads the canonical names back.
/// Name resolution errors are propagated with this physical file's path: the same statements
/// would fail again after splicing, while swallowing the error here would leave a silently wrong
/// Reflection source-file fallback.
fn record_declaration_sources(
    stmts: &[Stmt],
    path: &Path,
    state: &mut ResolveState,
) -> Result<(), CompileError> {
    let canonical = crate::name_resolver::resolve(stmts.to_vec())
        .map_err(|error| error.with_file(path.display().to_string()))?;
    let file = path.display().to_string();
    collect_declared_names(&canonical, &file, state);
    Ok(())
}

/// Walks canonicalized statements, recording each declaration against `file`.
fn collect_declared_names(stmts: &[Stmt], file: &str, state: &mut ResolveState) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::ClassDecl { name, .. }
            | StmtKind::InterfaceDecl { name, .. }
            | StmtKind::TraitDecl { name, .. }
            | StmtKind::EnumDecl { name, .. }
            | StmtKind::PackedClassDecl { name, .. } => {
                state.declared_class_files.insert(
                    crate::names::php_symbol_key(name.trim_start_matches('\\')),
                    file.to_string(),
                );
            }
            StmtKind::FunctionDecl { name, .. } => {
                state.declared_function_files.insert(
                    crate::names::php_symbol_key(name.trim_start_matches('\\')),
                    file.to_string(),
                );
            }
            StmtKind::NamespaceBlock { body, .. } => collect_declared_names(body, file, state),
            _ => {}
        }
    }
}
