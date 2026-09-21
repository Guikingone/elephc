//! Purpose:
//! Collects statically resolvable `class_alias("Original", "Alias")` calls.
//! Synthesizes subclass declarations that approximate alias use in the AOT class table.
//!
//! Called from:
//! - `crate::autoload::registry::Registry::build()`
//! - `crate::autoload::collect_aliases()` after include/autoload expansion
//!
//! Key details:
//! - A post-name-resolution pass also recognizes `Original::class` and `Alias::class`
//!   in nested statement lists while leaving the defensive call in place.
//! - Runtime-dynamic alias calls are left in the program and rejected by the checker.
//! - Resolver-created include wrappers still count as top-level for included-file aliases.
//! - The alias is a subclass, not a true PHP runtime alias, so identity checks differ in documented cases.

use std::collections::HashSet;

use crate::names::{php_symbol_key, Name, NameKind};
use crate::parser::ast::{Expr, ExprKind, Program, Stmt, StmtKind};

/// Walk top-level statements for `class_alias("Orig", "Alias")` calls
/// (with literal arguments). Strip every collected call and append a
/// synthesized `class Alias extends Orig {}` declaration. Calls with
/// non-literal or runtime-dependent arguments stay in the program and are
/// rejected by the checker.
pub fn collect_aliases(program: Program) -> Program {
    let mut alias_decls: Vec<Stmt> = Vec::new();
    let mut cleaned = collect_aliases_in_top_level(program, &mut alias_decls);
    cleaned.extend(alias_decls);
    cleaned
}

/// Collects aliases whose class-string arguments became static only after name resolution.
///
/// Calls remain in their original statement lists so their boolean result and control-flow
/// position stay visible. The synthesized declaration makes the alias available to the closed
/// world class table before type checking; a retained call therefore lowers as the defensive
/// "already defined" result.
pub fn collect_resolved_aliases(mut program: Program) -> Program {
    let mut declared = super::walk::collect_declared_fqns(&program)
        .into_iter()
        .map(|name| php_symbol_key(&name))
        .collect::<HashSet<_>>();
    let mut alias_decls = Vec::new();
    collect_resolved_aliases_in_program(&mut program, &mut alias_decls, &mut declared);
    program.extend(alias_decls);
    program
}

/// Scans one resolved statement list for static alias calls and nested statement lists.
fn collect_resolved_aliases_in_program(
    program: &mut Program,
    alias_decls: &mut Vec<Stmt>,
    declared: &mut HashSet<String>,
) {
    for stmt in program {
        collect_resolved_aliases_in_stmt(stmt, alias_decls, declared);
    }
}

/// Records a resolved alias call and descends through every statement-owned body.
fn collect_resolved_aliases_in_stmt(
    stmt: &mut Stmt,
    alias_decls: &mut Vec<Stmt>,
    declared: &mut HashSet<String>,
) {
    if let Some((original, alias)) = extract_resolved_class_alias(stmt) {
        if declared.insert(php_symbol_key(&alias)) {
            alias_decls.push(synthesise_resolved_alias_decl(
                &original,
                &alias,
                stmt.span,
            ));
        }
    }

    match &mut stmt.kind {
        StmtKind::Synthetic(body)
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::FunctionDecl { body, .. } => {
            collect_resolved_aliases_in_program(body, alias_decls, declared);
        }
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            collect_resolved_aliases_in_program(then_body, alias_decls, declared);
            for (_, body) in elseif_clauses {
                collect_resolved_aliases_in_program(body, alias_decls, declared);
            }
            if let Some(body) = else_body {
                collect_resolved_aliases_in_program(body, alias_decls, declared);
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            collect_resolved_aliases_in_program(then_body, alias_decls, declared);
            if let Some(body) = else_body {
                collect_resolved_aliases_in_program(body, alias_decls, declared);
            }
        }
        StmtKind::For {
            init, update, body, ..
        } => {
            if let Some(init) = init {
                collect_resolved_aliases_in_stmt(init, alias_decls, declared);
            }
            if let Some(update) = update {
                collect_resolved_aliases_in_stmt(update, alias_decls, declared);
            }
            collect_resolved_aliases_in_program(body, alias_decls, declared);
        }
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases {
                collect_resolved_aliases_in_program(body, alias_decls, declared);
            }
            if let Some(body) = default {
                collect_resolved_aliases_in_program(body, alias_decls, declared);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_resolved_aliases_in_program(try_body, alias_decls, declared);
            for catch in catches {
                collect_resolved_aliases_in_program(&mut catch.body, alias_decls, declared);
            }
            if let Some(body) = finally_body {
                collect_resolved_aliases_in_program(body, alias_decls, declared);
            }
        }
        StmtKind::ClassDecl { methods, .. }
        | StmtKind::TraitDecl { methods, .. }
        | StmtKind::InterfaceDecl { methods, .. }
        | StmtKind::EnumDecl { methods, .. } => {
            for method in methods {
                collect_resolved_aliases_in_program(&mut method.body, alias_decls, declared);
            }
        }
        _ => {}
    }
}

/// Iterates over top-level statements, removing each `class_alias("Orig", "Alias")`
/// call with literal arguments and appending the corresponding synthesized
/// `class Alias extends Orig {}` declaration to `alias_decls`. Returns the
/// filtered program with all collected alias declarations appended at the end.
/// Non-literal or runtime-dependent `class_alias` calls remain in the program
/// and are not collected — the caller is responsible for rejecting them.
fn collect_aliases_in_top_level(program: Program, alias_decls: &mut Vec<Stmt>) -> Program {
    program
        .into_iter()
        .filter_map(|stmt| collect_aliases_in_stmt(stmt, alias_decls))
        .collect()
}

/// Inspects a single statement for a `class_alias` call. If found, pushes the
/// synthesized subclass declaration to `alias_decls` and returns `None` to remove
/// the original call from the program. Descends into `NamespaceBlock`,
/// `IncludeOnceGuard`, and `Synthetic` wrappers; all other statement kinds are
/// returned unchanged after the alias check.
fn collect_aliases_in_stmt(stmt: Stmt, alias_decls: &mut Vec<Stmt>) -> Option<Stmt> {
    if let Some((orig, alias)) = extract_class_alias(&stmt) {
        alias_decls.push(synthesise_alias_decl(&orig, &alias, stmt.span));
        return None;
    }

    let span = stmt.span;
    let source_mode = stmt.source_mode;
    let strict_types = stmt.strict_types;
    let attributes = stmt.attributes;
    match stmt.kind {
        StmtKind::NamespaceBlock { name, body } => Some(Stmt {
            kind: StmtKind::NamespaceBlock {
                name,
                body: collect_aliases_in_top_level(body, alias_decls),
            },
            span,
            source_mode,
            strict_types,
            attributes,
        }),
        StmtKind::IncludeOnceGuard { source_path, body } => Some(Stmt {
            kind: StmtKind::IncludeOnceGuard {
                source_path,
                body: collect_aliases_in_top_level(body, alias_decls),
            },
            span,
            source_mode,
            strict_types,
            attributes,
        }),
        StmtKind::Synthetic(body) => Some(Stmt {
            kind: StmtKind::Synthetic(collect_aliases_in_top_level(body, alias_decls)),
            span,
            source_mode,
            strict_types,
            attributes,
        }),
        kind => Some(Stmt {
            kind,
            span,
            source_mode,
            strict_types,
            attributes,
        }),
    }
}

/// Extract class alias pair from a statement if it is a literal `class_alias` call.
fn extract_class_alias(stmt: &Stmt) -> Option<(String, String)> {
    let StmtKind::ExprStmt(expr) = &stmt.kind else {
        return None;
    };
    let ExprKind::FunctionCall { name, args } = &expr.kind else {
        return None;
    };
    let canonical = name.as_canonical();
    if !canonical
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("class_alias")
    {
        return None;
    }
    if !class_alias_arity_is_supported(args) {
        return None;
    }
    let orig = literal_string(args.first()?)?.to_string();
    let alias = literal_string(args.get(1)?)?.to_string();
    Some((orig, alias))
}

/// Checks the argument shape of a `class_alias()` call the compiler can resolve statically.
///
/// The third argument is PHP's `$autoload`: whether to AUTOLOAD the original class before
/// aliasing it. It says nothing about whether the two names are statically known, so a literal
/// `false` is just as resolvable as a literal `true` — in a closed world the original is either
/// compiled in or the alias is rejected anyway, and autoloading never enters into it.
///
/// Refusing `false` is what kept Symfony's generated container out of the compiled world: it
/// emits `\class_alias(\ContainerXXXX\App_KernelProdContainer::class, App_KernelProdContainer::class, false)`
/// — both names `::class` constants — and the whole entry failed with
/// "class_alias() requires statically resolvable class names".
///
/// A NON-literal flag is still refused: it is a value the compiler cannot see, and accepting it
/// would mean guessing at a call it cannot model.
fn class_alias_arity_is_supported(args: &[Expr]) -> bool {
    if args.len() < 2 || args.len() > 3 {
        return false;
    }
    match args.get(2).map(|arg| &arg.kind) {
        None => true,
        Some(ExprKind::BoolLiteral(_)) => true,
        Some(ExprKind::IntLiteral(_)) => true,
        Some(_) => false,
    }
}

/// Extracts a class alias pair from a resolved statement call with static class strings.
fn extract_resolved_class_alias(stmt: &Stmt) -> Option<(String, String)> {
    let StmtKind::ExprStmt(expr) = &stmt.kind else {
        return None;
    };
    let ExprKind::FunctionCall { name, args } = &expr.kind else {
        return None;
    };
    if !name
        .as_canonical()
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("class_alias")
    {
        return None;
    }
    resolved_class_alias_args(args)
}

/// Returns the statically known original and alias class names for supported call arguments.
pub(crate) fn resolved_class_alias_args(args: &[Expr]) -> Option<(String, String)> {
    if !class_alias_arity_is_supported(args) {
        return None;
    }
    Some((
        resolved_class_name(args.first()?)?,
        resolved_class_name(args.get(1)?)?,
    ))
}

/// Resolves a literal string or a canonical named `ClassName::class` expression.
fn resolved_class_name(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::StringLiteral(name) => Some(name.trim_start_matches('\\').to_string()),
        ExprKind::ClassConstant {
            receiver: crate::parser::ast::StaticReceiver::Named(name),
        } => Some(name.as_canonical().trim_start_matches('\\').to_string()),
        _ => None,
    }
}

/// Extract a string value from a literal string expression.
fn literal_string(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::StringLiteral(s) => Some(s.as_str()),
        _ => None,
    }
}

/// Synthesize `class Alias extends Original {}` for the given pair of
/// FQNs. When the alias name itself is namespaced, wrap the declaration
/// in a `NamespaceBlock` so name resolution canonicalises it correctly.
fn synthesise_alias_decl(orig: &str, alias: &str, span: crate::span::Span) -> Stmt {
    let orig_parts: Vec<String> = orig
        .trim_start_matches('\\')
        .split('\\')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let alias_parts: Vec<String> = alias
        .trim_start_matches('\\')
        .split('\\')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let alias_local = alias_parts.last().cloned().unwrap_or_default();
    let alias_namespace_parts = alias_parts
        .iter()
        .take(alias_parts.len().saturating_sub(1))
        .cloned()
        .collect::<Vec<_>>();

    let extends_name = Name::from_parts(NameKind::FullyQualified, orig_parts);

    let class_stmt = Stmt::new(
        StmtKind::ClassDecl {
            name: alias_local,
            doc_comment: None,
            extends: Some(extends_name),
            implements: Vec::new(),
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            trait_uses: Vec::new(),
            properties: Vec::new(),
            methods: Vec::new(),
            constants: Vec::new(),
        },
        span,
    );

    if alias_namespace_parts.is_empty() {
        class_stmt
    } else {
        let ns_name = Name::from_parts(NameKind::Qualified, alias_namespace_parts);
        Stmt::new(
            StmtKind::NamespaceBlock {
                name: Some(ns_name),
                body: vec![class_stmt],
            },
            span,
        )
    }
}

/// Synthesizes an alias declaration from already-canonical class names.
fn synthesise_resolved_alias_decl(
    orig: &str,
    alias: &str,
    span: crate::span::Span,
) -> Stmt {
    let orig_parts = orig
        .trim_start_matches('\\')
        .split('\\')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();
    Stmt::new(
        StmtKind::ClassDecl {
            name: alias.trim_start_matches('\\').to_string(),
            doc_comment: None,
            extends: Some(Name::from_parts(NameKind::FullyQualified, orig_parts)),
            implements: Vec::new(),
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            trait_uses: Vec::new(),
            properties: Vec::new(),
            methods: Vec::new(),
            constants: Vec::new(),
        },
        span,
    )
}
