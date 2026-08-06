//! Purpose:
//! Injects the PHP `strncmp()` standard-library function, written in elephc-PHP.
//! It compares at most `$length` leading bytes of two strings and returns their byte difference.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, at the same
//!   pipeline stage as `crate::mb_convert_encoding_prelude` (after `autoload::run` and the
//!   conditional-function hoist, before the type checker collects functions), so PSR-4 autoloaded
//!   usage is detected and the declaration is present before checking.
//!
//! Key details:
//! - `strncmp` was one of the catalog builtins with NO EIR lowering at all: the checker recognized
//!   the name and codegen then answered `unsupported EIR backend feature: builtin call strncmp`.
//! - Implemented on top of `substr` + `strcmp` rather than as per-target runtime assembly.
//!   Comparing at most `$length` leading bytes is exactly comparing the two strings truncated to
//!   that length, and elephc's `strcmp`/`substr` were verified byte-identical to PHP 8.5 on the
//!   raw-byte-difference convention (`strncmp("hello", "help", 4)` is `-4`, not `-1`).
//! - `catalog::is_prelude_overridable_builtin` keeps the NAME in the builtin catalog (so
//!   `function_exists('strncmp')` still reports a real PHP function) while allowing this
//!   declaration to supply the body.

use crate::parser::ast::{Program, Stmt, StmtKind};

/// The elephc-PHP `strncmp` prelude.
pub const STRNCMP_PRELUDE_SRC: &str = r#"<?php
function strncmp(string $string1, string $string2, int $length): int {
    if ($length < 0) {
        throw new \ValueError('strncmp(): Argument #3 ($length) must be greater than or equal to 0');
    }
    if ($length === 0) {
        return 0;
    }
    return strcmp(substr($string1, 0, $length), substr($string2, 0, $length));
}
"#;

/// Prepends the `strncmp` prelude when the program references it and does not declare its own.
pub fn inject_if_used(program: Program) -> Program {
    if !crate::ast_usage::collect(&program).references("strncmp")
        || program_declares_strncmp(&program)
    {
        return program;
    }
    let tokens =
        crate::lexer::tokenize(STRNCMP_PRELUDE_SRC).expect("strncmp prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("strncmp prelude must parse");
    combined.extend(program);
    combined
}

/// Returns whether the program already declares its own global `strncmp` (at top level or inside a
/// namespace/guard/synthetic block the hoist stage leaves in place), in which case the prelude must
/// not be injected so the user definition wins.
fn program_declares_strncmp(program: &[Stmt]) -> bool {
    program.iter().any(stmt_declares_strncmp)
}

/// Returns whether one statement declares a `strncmp` function, recursing only into the block forms
/// that can host a hoisted function declaration.
fn stmt_declares_strncmp(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { name, .. } => name.eq_ignore_ascii_case("strncmp"),
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body.iter().any(stmt_declares_strncmp),
        _ => false,
    }
}
