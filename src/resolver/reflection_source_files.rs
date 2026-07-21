//! Purpose:
//! Snapshots which top-level classes/enums/functions are declared directly in one PHP source
//! file, before `include`/`require` merging or autoloaded-library splicing can add declarations
//! from OTHER files into the same AST.
//!
//! Called from:
//! - `crate::pipeline::compile()`, right after magic-constant substitution and before
//!   `autoload::Registry::build`/`resolver::resolve()`.
//! - `tests/codegen/support/compiler.rs`'s EIR codegen-fixture frontend, mirroring the same
//!   placement for the synthetic per-test main file.
//!
//! Key details:
//! - Backs `ReflectionClass`/`ReflectionFunction`/`ReflectionMethod`/`ReflectionProperty::
//!   getFileName()` (see `crate::codegen::lower_inst::objects::reflection` for the EIR
//!   bakers, and `crate::types::checker::builtin_types::reflection` for the shell bodies).
//! - A name declared more than once while scanning (e.g. two same-named classes under different
//!   `namespace` blocks in one file) is dropped from its map entirely rather than risk baking
//!   the WRONG file for an ambiguous name.
//! - A class/function absent from the returned maps — because it is declared in a different
//!   file (an `include`d/autoloaded one) or is one of the ambiguous duplicates above — simply
//!   gets PHP's `false` from `getFileName()`, never a guess.

use std::collections::HashMap;
use std::path::Path;

use crate::names::php_symbol_key;
use crate::parser::ast::{Stmt, StmtKind};

/// Scans `stmts` for top-level `class`/`enum`/free-function declarations physically written in
/// `file`, recursing into `namespace { }` blocks and `if`/`elseif`/`else` bodies (the common PHP
/// conditional-polyfill shape: `if (!class_exists(...)) { class Foo {} }`), and returns two
/// case-insensitive name -> file-path maps (`classes`, `functions`).
pub fn scan_reflection_source_files(
    stmts: &[Stmt],
    file: &Path,
) -> (HashMap<String, String>, HashMap<String, String>) {
    let file_str = file.display().to_string();
    let mut class_hits: Vec<(String, String)> = Vec::new();
    let mut function_hits: Vec<(String, String)> = Vec::new();
    collect_reflection_source_file_hits(stmts, &file_str, &mut class_hits, &mut function_hits);
    (
        dedup_unique_reflection_source_files(class_hits),
        dedup_unique_reflection_source_files(function_hits),
    )
}

/// Recursive worker for `scan_reflection_source_files`: appends one `(case-folded name, file)`
/// hit per top-level class-like/function declaration found in `stmts`, descending into
/// `namespace { }` blocks and `if`/`elseif`/`else` bodies only (declarations nested inside
/// ordinary function/method bodies are out of scope for this feature and left unrecorded, so
/// they resolve to `getFileName() === false`, same as any other unmapped name).
fn collect_reflection_source_file_hits(
    stmts: &[Stmt],
    file: &str,
    class_hits: &mut Vec<(String, String)>,
    function_hits: &mut Vec<(String, String)>,
) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::ClassDecl { name, .. } | StmtKind::EnumDecl { name, .. } => {
                class_hits.push((php_symbol_key(name), file.to_string()));
            }
            StmtKind::FunctionDecl { name, .. } => {
                function_hits.push((php_symbol_key(name), file.to_string()));
            }
            StmtKind::NamespaceBlock { body, .. } => {
                collect_reflection_source_file_hits(body, file, class_hits, function_hits);
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_reflection_source_file_hits(then_body, file, class_hits, function_hits);
                for (_, body) in elseif_clauses {
                    collect_reflection_source_file_hits(body, file, class_hits, function_hits);
                }
                if let Some(body) = else_body {
                    collect_reflection_source_file_hits(body, file, class_hits, function_hits);
                }
            }
            _ => {}
        }
    }
}

/// Collapses `(case-folded name, file)` hits into a map, dropping any name that occurred more
/// than once (an ambiguous declaration this snapshot cannot safely attribute to one file).
fn dedup_unique_reflection_source_files(hits: Vec<(String, String)>) -> HashMap<String, String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (key, _) in &hits {
        *counts.entry(key.clone()).or_insert(0) += 1;
    }
    hits.into_iter()
        .filter(|(key, _)| counts.get(key) == Some(&1))
        .collect()
}
