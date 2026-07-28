//! Purpose:
//! Injects the elephc-PHP implementations of PHP's rfc1867 file-upload predicates,
//! `is_uploaded_file()` and `move_uploaded_file()`, together with the single upload registry
//! they both read.
//!
//! Called from:
//! - `crate::pipeline::compile()` via `inject_if_used`, at the same pipeline stage as
//!   `crate::var_export_prelude` (after `autoload::run` and the conditional-function hoist, so
//!   PSR-4 autoloaded usage is detected, and before the type checker collects functions).
//! - `crate::web_prelude`'s multipart parser, which calls `__elephc_register_uploaded_file()`
//!   for every temp file it materializes. That reference is what triggers injection under
//!   `--web`, because the web prelude is prepended before this injection point.
//!
//! Key details:
//! - This is NOT a stub. PHP decides "was this path uploaded?" by consulting the set of temp
//!   files its own rfc1867 parser created for the current request, and nothing else. elephc's
//!   `--web` multipart parser (`crate::web_prelude`) is the ONLY producer of upload temp files
//!   in a compiled program, so registering there makes the registry complete by construction.
//!   Outside `--web` the registry is empty and both functions return `false` for every path —
//!   which is exactly what PHP CLI does, where no request upload ever exists.
//! - One source of truth: both predicates read `__elephc_upload_registry()`. There is no second
//!   place that decides upload-ness.
//! - The registry is a function-`static` array seeded with `['' => false]` rather than `[]`.
//!   An empty `static $u = []` currently trips a backend gap (`init_static_local assigning PHP
//!   type Array(Never) to static local $u with PHP type Mixed`); the seed gives the slot a
//!   concrete `array<string, bool>` type. The seed key is the empty string, which can never be
//!   a temp-file path, and its value is `false`, so `is_uploaded_file('')` still answers `false`.
//! - Deregistration writes `false` instead of `unset()`ing the entry, so a moved file stops
//!   being an upload without depending on `unset()` of a static-local array element.
//! - The registry is queried with `array_key_exists()`, never `isset()`: `isset($static[$k])` on
//!   a `static` array emits a spurious `Warning: Undefined array key` (a separate pre-existing
//!   bug), which would corrupt the output of any program calling these predicates.

use crate::parser::ast::{Program, Stmt, StmtKind};

/// The bare symbol name of the upload registrar the `--web` multipart parser calls. Kept as a
/// constant so `crate::web_prelude`'s call site and this prelude's declaration cannot drift
/// apart, and so `inject_if_used` can key injection on the same name. The `__elephc_` prefix is
/// reserved, so this never collides with user code.
pub(crate) const REGISTER_UPLOADED_FILE_NAME: &str = "__elephc_register_uploaded_file";

/// The bare symbol name of the per-request registry reset the `--web` prelude calls before each
/// request's multipart parse. Kept beside `REGISTER_UPLOADED_FILE_NAME` for the same reason.
pub(crate) const RESET_UPLOADED_FILES_NAME: &str = "__elephc_reset_uploaded_files";

/// The elephc-PHP upload prelude: the shared registry plus PHP's two upload predicates.
///
/// `move_uploaded_file()` mirrors PHP's semantics exactly: it refuses (returns `false`) any path
/// that is not a registered upload, moves the file otherwise, and drops the path from the
/// registry so a second move fails. `rename()` is attempted first and falls back to
/// copy + unlink, because `rename()` cannot cross filesystems and the upload temp directory is
/// frequently on a different one from the destination.
pub const UPLOAD_PRELUDE_SRC: &str = r#"<?php
function __elephc_upload_registry(int $op, string $path): bool {
    static $uploads = ['' => false];
    if ($op === 1) { $uploads[$path] = true; return true; }
    if ($op === 2) { $uploads[$path] = false; return true; }
    if ($op === 3) { $uploads = ['' => false]; return true; }
    return array_key_exists($path, $uploads) && $uploads[$path];
}
function __elephc_register_uploaded_file(string $path): void {
    __elephc_upload_registry(1, $path);
}
function __elephc_reset_uploaded_files(): void {
    __elephc_upload_registry(3, '');
}
function is_uploaded_file(string $filename): bool {
    return __elephc_upload_registry(0, $filename);
}
function move_uploaded_file(string $from, string $to): bool {
    if (!__elephc_upload_registry(0, $from)) { return false; }
    if (!rename($from, $to)) {
        if (!copy($from, $to)) { return false; }
        unlink($from);
    }
    __elephc_upload_registry(2, $from);
    return true;
}
"#;

/// Prepends the upload prelude when the program references either upload predicate or the
/// registrar, and does not declare its own `is_uploaded_file`/`move_uploaded_file`.
///
/// Under `--web` the registrar reference comes from `crate::web_prelude`, which is prepended
/// earlier in `crate::pipeline::compile()`, so a program that merely receives uploads still
/// gets a populated registry.
pub fn inject_if_used(program: Program) -> Program {
    let usage = crate::ast_usage::collect(&program);
    if !usage.references("is_uploaded_file")
        && !usage.references("move_uploaded_file")
        && !usage.references(REGISTER_UPLOADED_FILE_NAME)
        && !usage.references(RESET_UPLOADED_FILES_NAME)
    {
        return program;
    }
    if program_declares_upload_predicate(&program) {
        return program;
    }
    let tokens =
        crate::lexer::tokenize(UPLOAD_PRELUDE_SRC).expect("upload prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("upload prelude must parse");
    combined.extend(program);
    combined
}

/// Returns whether the program already declares its own global `is_uploaded_file` or
/// `move_uploaded_file` (at top level or inside a namespace/guard/synthetic block that the
/// hoist stage leaves in place), in which case the prelude must not be injected so the user
/// definition wins.
fn program_declares_upload_predicate(program: &[Stmt]) -> bool {
    program.iter().any(stmt_declares_upload_predicate)
}

/// Returns whether one statement declares an upload predicate, recursing only into the block
/// forms that can host a hoisted function declaration.
fn stmt_declares_upload_predicate(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { name, .. } => {
            name.eq_ignore_ascii_case("is_uploaded_file")
                || name.eq_ignore_ascii_case("move_uploaded_file")
        }
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body.iter().any(stmt_declares_upload_predicate),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Function-level tests for the `inject_if_used` pay-for-use guard.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Source is parsed the way `inject_if_used` sees it: tokenize then parse.

    use super::*;

    /// Parses source the way `inject_if_used` sees it: tokenize then parse.
    fn parse(source: &str) -> Program {
        let tokens = crate::lexer::tokenize(source).expect("test source must tokenize");
        crate::parser::parse(&tokens).expect("test source must parse")
    }

    /// A program with no upload usage is returned unchanged.
    #[test]
    fn no_injection_when_unused() {
        let program = parse(r#"<?php $a = [1, 2]; echo count($a);"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }

    /// A program that calls `is_uploaded_file` gets the prelude prepended.
    #[test]
    fn injection_when_is_uploaded_file_used() {
        let program = parse(r#"<?php var_dump(is_uploaded_file("/tmp/x"));"#);
        let injected = inject_if_used(program.clone());
        assert!(injected.len() > program.len());
    }

    /// A program that calls `move_uploaded_file` gets the prelude prepended.
    #[test]
    fn injection_when_move_uploaded_file_used() {
        let program = parse(r#"<?php move_uploaded_file("/tmp/a", "/tmp/b");"#);
        let injected = inject_if_used(program.clone());
        assert!(injected.len() > program.len());
    }

    /// The `--web` multipart parser's registrar call alone triggers injection.
    #[test]
    fn injection_when_registrar_referenced() {
        let program = parse(r#"<?php __elephc_register_uploaded_file("/tmp/x");"#);
        let injected = inject_if_used(program.clone());
        assert!(injected.len() > program.len());
    }

    /// A program declaring its own `is_uploaded_file` must not be given the prelude copy.
    #[test]
    fn no_injection_when_user_declares_predicate() {
        let program = parse(
            r#"<?php function is_uploaded_file(string $f): bool { return false; } is_uploaded_file("x");"#,
        );
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }
}
