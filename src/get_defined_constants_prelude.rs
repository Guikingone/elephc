//! Purpose:
//! Injects the PHP `get_defined_constants()` standard-library function, written in elephc-PHP.
//! Its body is GENERATED from the compiler's own constant tables, so the two cannot drift.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, at the same
//!   pipeline stage as `crate::mb_convert_encoding_prelude`.
//!
//! Key details:
//! - `get_defined_constants` was catalog-visible with no EIR lowering: the checker accepted the
//!   call and codegen answered `unsupported EIR backend feature: builtin call
//!   get_defined_constants`.
//! - Each entry is emitted as `'NAME' => NAME`, i.e. it REFERENCES the constant rather than
//!   restating its value. Values therefore always match what the program actually sees, including
//!   the pcntl signals whose numbers differ between macOS and Linux and are chosen per target.
//! - Only the extension categories elephc has a constant table for are reported (`pcre`, `pcntl`).
//!   That mirrors PHP, where a category exists exactly when its extension is loaded — but it does
//!   mean the UNCATEGORIZED form under-reports relative to a stock PHP build, which also lists the
//!   Core/standard constants elephc registers elsewhere. That gap is stated here rather than
//!   hidden: the categorized form (the one Symfony uses) is complete for the categories present.

use crate::parser::ast::{Program, Stmt, StmtKind};
use crate::types::pcntl_constants::{PCNTL_INT_CONSTANTS, PCNTL_PLATFORM_SIGNALS};
use crate::types::preg_constants::PREG_INT_CONSTANTS;

/// Builds the elephc-PHP `get_defined_constants()` source from the compiler's constant tables.
fn prelude_source() -> String {
    let pcre = entries(PREG_INT_CONSTANTS.iter().map(|(name, _)| *name));
    let pcntl = entries(
        PCNTL_INT_CONSTANTS
            .iter()
            .map(|(name, _)| *name)
            .chain(PCNTL_PLATFORM_SIGNALS.iter().map(|(name, _, _)| *name)),
    );
    format!(
        r#"<?php
function get_defined_constants(bool $categorize = false): array {{
    $categories = [
        'pcre' => [
{pcre}        ],
        'pcntl' => [
{pcntl}        ],
    ];
    if ($categorize) {{
        return $categories;
    }}
    $flat = [];
    foreach ($categories as $constants) {{
        foreach ($constants as $name => $value) {{
            $flat[$name] = $value;
        }}
    }}
    return $flat;
}}
"#
    )
}

/// Renders one `'NAME' => NAME,` line per constant, indented to sit inside the category literal.
fn entries<'a>(names: impl Iterator<Item = &'a str>) -> String {
    names
        .map(|name| format!("            '{name}' => {name},\n"))
        .collect()
}

/// Prepends the `get_defined_constants` prelude when the program references it and does not
/// declare its own.
pub fn inject_if_used(program: Program) -> Program {
    if !crate::ast_usage::collect(&program).references("get_defined_constants")
        || program_declares(&program)
    {
        return program;
    }
    let source = prelude_source();
    let tokens = crate::lexer::tokenize(&source)
        .expect("get_defined_constants prelude must tokenize");
    let mut combined =
        crate::parser::parse(&tokens).expect("get_defined_constants prelude must parse");
    combined.extend(program);
    combined
}

/// Returns whether the program already declares its own global `get_defined_constants`.
fn program_declares(program: &[Stmt]) -> bool {
    program.iter().any(stmt_declares)
}

/// Returns whether one statement declares a `get_defined_constants` function, recursing only into
/// the block forms that can host a hoisted function declaration.
fn stmt_declares(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { name, .. } => name.eq_ignore_ascii_case("get_defined_constants"),
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body.iter().any(stmt_declares),
        _ => false,
    }
}
