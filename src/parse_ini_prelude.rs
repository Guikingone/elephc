//! Purpose:
//! Injects the PHP `parse_ini_file()` standard-library function (written in elephc-PHP) that
//! reads and parses an `.ini` file into an array, matching PHP's `ext/standard` scanner for the
//! common cases: `key = value`, `[section]` headers (when `$process_sections` is set), `;`/`#`
//! comments, quoted values, `key[] = value` array appends, and the NORMAL / RAW / TYPED scanner
//! modes.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, at the same
//!   pipeline stage as `crate::var_export_prelude` (after `autoload::run` and the conditional-
//!   function hoist, before the type checker collects functions), so PSR-4 autoloaded usage is
//!   detected and the declaration is present before checking. Namespaced bare calls resolve to
//!   this global via `name_resolver::PRELUDE_GLOBAL_FUNCTIONS`.
//!
//! Key details:
//! - Implemented as a prelude (not hand-written runtime assembly) so the parser reuses ordinary
//!   PHP string/array control flow and works on every supported target with no per-target code.
//! - Value coercion is php-verified (PHP 8.5.6 local): NORMAL maps `on/true/yes` -> "1" and
//!   `off/false/no/none` -> "" and leaves everything else a string; RAW returns the raw token
//!   verbatim (no boolean/comment coercion beyond quote stripping and inline-`;` trimming); TYPED
//!   returns real `bool`/`int`/`float` for boolean tokens and numeric strings. A double-quoted
//!   value is returned as its literal contents in EVERY mode (no coercion, embedded `;` kept).
//! - Unreadable file: emits a warning via `error_log` (the EIR backend does not lower
//!   `trigger_error`) and returns `false`, matching PHP's failure contract (the exact warning text
//!   is not byte-identical to PHP's stream error, but the `false` return and a stderr warning are).
//! - SHIPPED + verified php-exact: the FLAT scalar forms — `parse_ini_file($f)` and
//!   `parse_ini_file($f, false, INI_SCANNER_RAW|NORMAL|TYPED)` — over `key = value` entries
//!   (booleans, quoted values, numbers, comments), plus the unreadable-file `false` return.
//! - DEFERRED (pre-existing elephc Mixed-array-element-access gap): the SECTIONED
//!   (`$process_sections = true`) and `key[] =` array-append forms build NESTED sub-arrays. The
//!   parser constructs them, but a nested sub-array read back out of the returned array arrives as
//!   a `mixed`-boxed array, and element access / iteration on a `mixed`-boxed array LOUDLY fatals
//!   (`array builtin argument must be of type array`) on THIS branch — the same regression that
//!   breaks `var_export()` of any array here. This is a LOUD failure on consumption, never a
//!   silently-wrong nested result. Symfony's `IniFileLoader` uses `$process_sections = true`, so
//!   its `parse_ini_file` COMPILES (the `--web` "Undefined function" wall is cleared) but its
//!   nested runtime consumption depends on that separate array fix.

use crate::parser::ast::{Program, Stmt, StmtKind};

/// The elephc-PHP `parse_ini_file` prelude: the public entry point plus a value-coercion helper
/// and a pure integer-token predicate, all prefixed so they cannot collide with user code.
pub const PARSE_INI_PRELUDE_SRC: &str = r#"<?php
function __elephc_ini_is_int_token(string $s): bool {
    $n = strlen($s);
    if ($n === 0) {
        return false;
    }
    $i = 0;
    $c0 = ord($s[0]);
    if ($c0 === 43 || $c0 === 45) {
        $i = 1;
    }
    if ($i >= $n) {
        return false;
    }
    while ($i < $n) {
        $c = ord($s[$i]);
        if ($c < 48 || $c > 57) {
            return false;
        }
        $i++;
    }
    return true;
}
function __elephc_ini_value(string $raw, int $mode): mixed {
    $len = strlen($raw);
    if ($len >= 2 && $raw[0] === '"' && $raw[$len - 1] === '"') {
        return substr($raw, 1, $len - 2);
    }
    $semi = strpos($raw, ';');
    if ($semi !== false) {
        $raw = rtrim(substr($raw, 0, $semi));
    }
    if ($mode === 1) {
        return $raw;
    }
    $lower = strtolower($raw);
    if ($mode === 2) {
        if ($lower === 'true' || $lower === 'on' || $lower === 'yes') {
            return true;
        }
        if ($lower === 'false' || $lower === 'off' || $lower === 'no' || $lower === 'none') {
            return false;
        }
        if ($raw === '') {
            return '';
        }
        if (__elephc_ini_is_int_token($raw)) {
            return (int) $raw;
        }
        if (is_numeric($raw)) {
            return (float) $raw;
        }
        return $raw;
    }
    if ($lower === 'true' || $lower === 'on' || $lower === 'yes') {
        return '1';
    }
    if ($lower === 'false' || $lower === 'off' || $lower === 'no' || $lower === 'none') {
        return '';
    }
    return $raw;
}
function parse_ini_file(string $filename, bool $process_sections = false, int $scanner_mode = 0): array|false {
    if (!is_file($filename) || !is_readable($filename)) {
        error_log("parse_ini_file(" . $filename . "): Failed to open stream: No such file or directory");
        return false;
    }
    $content = file_get_contents($filename);
    if ($content === false) {
        return false;
    }
    $content = str_replace("\r\n", "\n", $content);
    $content = str_replace("\r", "\n", $content);
    $lines = explode("\n", $content);
    $result = [];
    $section = '';
    $in_section = false;
    foreach ($lines as $line) {
        $line = trim($line);
        if ($line === '') {
            continue;
        }
        $first = $line[0];
        if ($first === ';' || $first === '#') {
            continue;
        }
        $llen = strlen($line);
        if ($first === '[' && $line[$llen - 1] === ']') {
            if ($process_sections) {
                $section = trim(substr($line, 1, $llen - 2));
                $in_section = true;
                if (!isset($result[$section])) {
                    $result[$section] = [];
                }
            }
            continue;
        }
        $eq = strpos($line, '=');
        if ($eq === false) {
            continue;
        }
        $key = trim(substr($line, 0, $eq));
        if ($key === '') {
            continue;
        }
        $value = __elephc_ini_value(trim(substr($line, $eq + 1)), $scanner_mode);
        $append = false;
        $klen = strlen($key);
        if ($klen >= 2 && substr($key, $klen - 2) === '[]') {
            $append = true;
            $key = substr($key, 0, $klen - 2);
        }
        if ($process_sections && $in_section) {
            if ($append) {
                $result[$section][$key][] = $value;
            } else {
                $result[$section][$key] = $value;
            }
        } else {
            if ($append) {
                $result[$key][] = $value;
            } else {
                $result[$key] = $value;
            }
        }
    }
    return $result;
}
"#;

/// Prepends the `parse_ini_file` prelude when the program references `parse_ini_file` and does not
/// declare its own, so unrelated binaries carry nothing and a user definition is never clobbered.
/// The prelude is hoisted function declarations only, so prepending does not change top-level
/// execution order. The source is static and tested, so a tokenize/parse failure is a compiler bug
/// and panics rather than degrading silently.
pub fn inject_if_used(program: Program) -> Program {
    if !crate::ast_usage::collect(&program).references("parse_ini_file")
        || program_declares_parse_ini_file(&program)
    {
        return program;
    }
    let tokens =
        crate::lexer::tokenize(PARSE_INI_PRELUDE_SRC).expect("parse_ini_file prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("parse_ini_file prelude must parse");
    combined.extend(program);
    combined
}

/// Returns whether the program already declares its own global `parse_ini_file` function (at top
/// level or inside a namespace/guard/synthetic block that the hoist stage leaves in place), in
/// which case the prelude must not be injected so the user definition wins.
fn program_declares_parse_ini_file(program: &[Stmt]) -> bool {
    program.iter().any(stmt_declares_parse_ini_file)
}

/// Returns whether one statement declares a `parse_ini_file` function, recursing only into the
/// block forms that can host a hoisted function declaration.
fn stmt_declares_parse_ini_file(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { name, .. } => name.eq_ignore_ascii_case("parse_ini_file"),
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body.iter().any(stmt_declares_parse_ini_file),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Function-level tests for the `inject_if_used` pay-for-use guard, mirroring
    //! `var_export_prelude`'s shape at the same pipeline stage.
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

    /// A program with no `parse_ini_file` usage is returned unchanged.
    #[test]
    fn no_injection_when_unused() {
        let program = parse(r#"<?php $a = [1, 2]; echo count($a);"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }

    /// A program that calls `parse_ini_file` gets the prelude prepended.
    #[test]
    fn injection_when_used() {
        let program = parse(r#"<?php $x = parse_ini_file("/tmp/x.ini");"#);
        let injected = inject_if_used(program.clone());
        assert!(injected.len() > program.len());
    }

    /// A program that declares its own `parse_ini_file` is returned unchanged.
    #[test]
    fn no_injection_when_user_declares() {
        let program = parse(r#"<?php function parse_ini_file($f, $s = false, $m = 0) { return []; } parse_ini_file("x");"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }
}
