//! Purpose:
//! Injects the PHP `mb_convert_encoding()` standard-library function (written in elephc-PHP),
//! converting a string — or every element of an array — between the byte encodings elephc supports.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, at the same
//!   pipeline stage as `crate::parse_ini_prelude` (after `autoload::run` and the conditional-
//!   function hoist, before the type checker collects functions), so PSR-4 autoloaded usage is
//!   detected and the declaration is present before checking.
//!
//! Key details:
//! - Implemented as a prelude rather than hand-written runtime assembly so the byte loop reuses
//!   ordinary PHP string builtins and works on every supported target with no per-target code. The
//!   `mb_*` family had NO EIR lowering at all before this — the checker recognized the names and
//!   codegen answered `unsupported EIR backend feature: builtin call mb_convert_encoding`, so
//!   `Console\Application::splitStringByWidth` could not compile.
//! - `catalog::is_prelude_overridable_builtin` keeps the NAME in the builtin catalog (so
//!   `function_exists('mb_convert_encoding')` and the mbstring polyfill guards still see a real PHP
//!   function) while allowing this declaration to supply the body.
//! - The RESULT SHAPE follows the subject, as PHP does: an array subject converts element-wise and
//!   returns an array with the same keys; anything else converts as a string.
//! - Encodings handled: `UTF-8` and the single-byte Latin family (`ISO-8859-1` / `Windows-1252` /
//!   `ASCII`, and their common spellings). Latin-1 → UTF-8 widens every byte >= 0x80 into the
//!   two-byte sequence PHP produces; UTF-8 → Latin-1 narrows those pairs back and substitutes `?`
//!   for anything outside U+00FF, which is what PHP's `mbstring` does without an explicit
//!   substitute character. Identical from/to is the identity, byte for byte.
//! - An encoding name outside that set returns the subject unchanged rather than inventing a
//!   conversion: elephc has no ICU-class table, and silently producing wrong bytes would be worse
//!   than passing them through. This is the one modelled gap, and it is stated here rather than
//!   hidden behind a "supported" claim.

use crate::parser::ast::{Program, Stmt, StmtKind};

/// The elephc-PHP `mb_convert_encoding` prelude: the public entry point plus an encoding-name
/// normalizer and the two single-byte transcoders, all prefixed so they cannot collide with user
/// code.
pub const MB_CONVERT_ENCODING_PRELUDE_SRC: &str = r#"<?php
function __elephc_mb_enc_kind(string $enc): int {
    $e = strtoupper($enc);
    if ($e === 'UTF-8' || $e === 'UTF8') {
        return 1;
    }
    if ($e === 'ISO-8859-1' || $e === 'ISO8859-1' || $e === 'LATIN1'
        || $e === 'WINDOWS-1252' || $e === 'CP1252'
        || $e === 'ASCII' || $e === 'US-ASCII') {
        return 2;
    }
    return 0;
}
function __elephc_mb_latin1_to_utf8(string $s): string {
    $out = '';
    $n = strlen($s);
    $i = 0;
    while ($i < $n) {
        $c = ord($s[$i]);
        if ($c < 128) {
            $out .= $s[$i];
        } else {
            $out .= chr(192 + intdiv($c, 64));
            $out .= chr(128 + ($c % 64));
        }
        $i++;
    }
    return $out;
}
function __elephc_mb_utf8_to_latin1(string $s): string {
    $out = '';
    $n = strlen($s);
    $i = 0;
    while ($i < $n) {
        $c = ord($s[$i]);
        if ($c < 128) {
            $out .= $s[$i];
            $i++;
            continue;
        }
        if (($c === 194 || $c === 195) && $i + 1 < $n) {
            $c2 = ord($s[$i + 1]);
            $out .= chr((($c - 192) * 64) + ($c2 - 128));
            $i += 2;
            continue;
        }
        $out .= '?';
        $i++;
        while ($i < $n && ord($s[$i]) >= 128 && ord($s[$i]) < 192) {
            $i++;
        }
    }
    return $out;
}
function __elephc_mb_convert_one(string $s, int $to, int $from): string {
    if ($to === $from || $to === 0 || $from === 0) {
        return $s;
    }
    if ($from === 2 && $to === 1) {
        return __elephc_mb_latin1_to_utf8($s);
    }
    if ($from === 1 && $to === 2) {
        return __elephc_mb_utf8_to_latin1($s);
    }
    return $s;
}
function mb_convert_encoding(mixed $string, string $to_encoding, mixed $from_encoding = null): mixed {
    $to = __elephc_mb_enc_kind($to_encoding);
    $fromName = is_string($from_encoding) ? $from_encoding : 'UTF-8';
    $from = __elephc_mb_enc_kind($fromName);
    if (is_array($string)) {
        $out = [];
        foreach ($string as $k => $v) {
            $out[$k] = __elephc_mb_convert_one((string) $v, $to, $from);
        }
        return $out;
    }
    return __elephc_mb_convert_one((string) $string, $to, $from);
}
"#;

/// Prepends the `mb_convert_encoding` prelude when the program references it and does not declare
/// its own.
///
/// The prelude is hoisted function declarations only, so prepending does not change top-level
/// execution order. The source is static and tested, so a tokenize/parse failure is a compiler bug
/// and panics rather than degrading silently.
pub fn inject_if_used(program: Program) -> Program {
    if !crate::ast_usage::collect(&program).references("mb_convert_encoding")
        || program_declares_mb_convert_encoding(&program)
    {
        return program;
    }
    let tokens = crate::lexer::tokenize(MB_CONVERT_ENCODING_PRELUDE_SRC)
        .expect("mb_convert_encoding prelude must tokenize");
    let mut combined =
        crate::parser::parse(&tokens).expect("mb_convert_encoding prelude must parse");
    combined.extend(program);
    combined
}

/// Returns whether the program already declares its own global `mb_convert_encoding` (at top level
/// or inside a namespace/guard/synthetic block the hoist stage leaves in place), in which case the
/// prelude must not be injected so the user definition wins.
fn program_declares_mb_convert_encoding(program: &[Stmt]) -> bool {
    program.iter().any(stmt_declares_mb_convert_encoding)
}

/// Returns whether one statement declares a `mb_convert_encoding` function, recursing only into the
/// block forms that can host a hoisted function declaration.
fn stmt_declares_mb_convert_encoding(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { name, .. } => name.eq_ignore_ascii_case("mb_convert_encoding"),
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body.iter().any(stmt_declares_mb_convert_encoding),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Function-level tests for the `inject_if_used` pay-for-use guard, mirroring
    //! `parse_ini_prelude`'s shape at the same pipeline stage.
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

    /// A program with no `mb_convert_encoding` usage is returned unchanged.
    #[test]
    fn no_injection_when_unused() {
        let program = parse(r#"<?php $a = [1, 2]; echo count($a);"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }

    /// A program that calls `mb_convert_encoding` gets the prelude prepended.
    #[test]
    fn injection_when_used() {
        let program = parse(r#"<?php $x = mb_convert_encoding("a", "UTF-8");"#);
        let injected = inject_if_used(program.clone());
        assert!(injected.len() > program.len());
    }

    /// A program that declares its own `mb_convert_encoding` is returned unchanged.
    #[test]
    fn no_injection_when_user_declares() {
        let program = parse(
            r#"<?php function mb_convert_encoding($s, $t, $f = null) { return $s; } mb_convert_encoding("a", "UTF-8");"#,
        );
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }
}
