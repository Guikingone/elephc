//! Purpose:
//! Injects the PHP `var_export()` standard-library function (written in elephc-PHP)
//! that renders a parsable representation of a scalar or array value, matching the
//! interpreter's layout: `'…'`-quoted strings, `true`/`false`/`NULL` keywords, and
//! the indented `array ( … )` form with `key => value,` entries.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`,
//!   after `autoload::run` and the conditional-function hoist, so the detection scan
//!   sees `var_export` usage in PSR-4 autoloaded files too. A user `var_export(...)`
//!   call resolves to the injected declaration through the name_resolver prelude-global
//!   fallback (`canonical_prelude_global_function_name`), which canonicalizes bare
//!   namespaced calls to the global `var_export` during the main name-resolution pass
//!   and during each autoloaded file's isolated name-resolution. The prelude's internal
//!   builtins are matched by `check_builtin` on their bare lowercase names, which the
//!   prelude source already uses, so they need no name-resolution pass.
//!
//! Key details:
//! - Implemented as a prelude rather than a runtime walker because the recursive,
//!   string-building format reuses ordinary PHP control flow; this keeps it correct
//!   on every supported target with no per-target assembly.
//! - Pay-for-use: injected only when `detect::program_references_var_export` finds a
//!   call or a `"var_export"` string (covering `function_exists`/callable forms), and
//!   never when the program already declares its own `var_export` (so user
//!   definitions win and there is no redeclaration conflict).
//! - Floats render with the interpreter's `serialize_precision = -1` semantics: the
//!   shortest decimal string that round-trips back to the same `double`, formatted
//!   with PHP's decimal/scientific layout (`1.0`, `0.3333333333333333`, `1.0E+17`,
//!   `1.0E-6`). `__elephc_var_export_float` finds the shortest precision by probing
//!   `sprintf("%.{p}e", ...)` until `(float)` of the result equals the input, then
//!   rebuilds the digit string per PHP's exponent thresholds — independent of the
//!   default `(string)`/`echo` precision used elsewhere.
//! - Objects are out of scope (PHP renders `\Class::__set_state(...)`); a non
//!   scalar/array value renders as the empty string.

use crate::parser::ast::Program;

mod detect;

/// The elephc-PHP `var_export` prelude: the public `var_export($value, $return)`
/// entry point plus two internal helpers (`__elephc_var_export_str` renders a value
/// to its parsable text, `__elephc_var_export_escape` single-quote-escapes a string).
/// The helpers are prefixed so they cannot collide with user code, and `var_export`
/// itself is injected only when the user does not define their own.
pub const VAR_EXPORT_PRELUDE_SRC: &str = r#"<?php
function __elephc_var_export_escape(mixed $s): string {
    $s = (string) $s;
    return str_replace("'", "\\'", str_replace("\\", "\\\\", $s));
}
function __elephc_var_export_float(float $f): string {
    if (is_nan($f)) {
        return 'NAN';
    }
    if (is_infinite($f)) {
        return $f < 0 ? '-INF' : 'INF';
    }
    if ($f === 0.0) {
        return ((string) $f)[0] === '-' ? '-0.0' : '0.0';
    }
    $s = '';
    for ($p = 0; $p <= 16; $p++) {
        $s = sprintf("%.{$p}e", $f);
        if ((float) $s === $f) {
            break;
        }
    }
    $start = ($s[0] === '-') ? 1 : 0;
    $neg = $start === 1;
    $epos = strpos($s, 'e');
    $exp = (int) substr($s, $epos + 1);
    $digits = str_replace('.', '', substr($s, $start, $epos - $start));
    $ndigits = strlen($digits);
    $decpt = $exp + 1;
    if ($decpt < -3 || $decpt > 17) {
        $out = $digits[0];
        $out = $out . (($ndigits > 1) ? '.' . substr($digits, 1) : '.0');
        $e = $decpt - 1;
        $out = $out . 'E' . ($e >= 0 ? '+' : '-') . abs($e);
    } else if ($decpt <= 0) {
        $out = '0.' . str_repeat('0', -$decpt) . $digits;
    } else if ($decpt >= $ndigits) {
        $out = $digits . str_repeat('0', $decpt - $ndigits) . '.0';
    } else {
        $out = substr($digits, 0, $decpt) . '.' . substr($digits, $decpt);
    }
    return ($neg ? '-' : '') . $out;
}
function __elephc_var_export_str(mixed $value, int $indent): string {
    if (is_int($value)) {
        return (string) $value;
    }
    if (is_float($value)) {
        return __elephc_var_export_float((float) $value);
    }
    if (is_bool($value)) {
        return $value ? 'true' : 'false';
    }
    if (is_null($value)) {
        return 'NULL';
    }
    if (is_string($value)) {
        return "'" . __elephc_var_export_escape($value) . "'";
    }
    if (is_array($value)) {
        $pad = str_repeat(' ', $indent);
        $out = "array (\n";
        foreach ($value as $k => $v) {
            if (is_int($k)) {
                $key = (string) $k;
            } else {
                $key = "'" . __elephc_var_export_escape($k) . "'";
            }
            $out = $out . $pad . '  ' . $key . ' => ';
            if (is_array($v)) {
                $out = $out . "\n" . $pad . '  ' . __elephc_var_export_str($v, $indent + 2);
            } else {
                $out = $out . __elephc_var_export_str($v, $indent + 2);
            }
            $out = $out . ",\n";
        }
        $out = $out . $pad . ')';
        return $out;
    }
    return '';
}
function var_export(mixed $value, bool $return = false) {
    $rendered = __elephc_var_export_str($value, 0);
    if ($return) {
        return $rendered;
    }
    echo $rendered;
    return null;
}
"#;

/// Prepends the `var_export` prelude when the program references `var_export` and does
/// not declare its own, so unrelated binaries pay nothing and a user definition is not
/// clobbered. The prelude is hoisted function declarations only, so prepending does not
/// change top-level execution order. The source is static and tested, so a
/// tokenize/parse failure is a compiler bug and panics rather than degrading silently.
pub fn inject_if_used(program: Program) -> Program {
    if !detect::program_references_var_export(&program)
        || detect::program_declares_var_export(&program)
    {
        return program;
    }
    let tokens = crate::lexer::tokenize(VAR_EXPORT_PRELUDE_SRC).expect("var_export prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("var_export prelude must parse");
    combined.extend(program);
    combined
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Function-level tests for the `inject_if_used` pay-for-use guard, covering the
    //! "only when used" contract and the user-declaration skip, mirroring the stage at
    //! which the injection now runs (after autoload and the conditional-function hoist,
    //! on the fully-expanded program).
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

    /// A program with no `var_export` usage is returned unchanged (the prelude is not
    /// injected), guarding the "only when used" contract.
    #[test]
    fn no_injection_when_unused() {
        let program = parse(r#"<?php $a = [1, 2]; echo count($a);"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }

    /// A program that calls `var_export` gets the prelude prepended (the program gains
    /// the three prelude function declarations: `var_export` plus the two helpers).
    #[test]
    fn injection_when_used() {
        let program = parse(r#"<?php var_export(42);"#);
        let injected = inject_if_used(program.clone());
        assert!(injected.len() > program.len());
    }

    /// A program that declares its own `var_export` is returned unchanged, so the user
    /// definition wins and there is no redeclaration conflict.
    #[test]
    fn no_injection_when_user_declares() {
        let program = parse(r#"<?php function var_export($v, $r = false) { return ""; }"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }
}
