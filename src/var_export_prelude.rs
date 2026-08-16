//! Purpose:
//! Injects the PHP `var_export()` standard-library function (written in elephc-PHP)
//! that renders a parsable representation of a scalar or array value, matching the
//! interpreter's layout: `'…'`-quoted strings, `true`/`false`/`NULL` keywords, and
//! the indented `array ( … )` form with `key => value,` entries.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`,
//!   before name resolution, so a user `var_export(...)` call resolves to the
//!   injected function through the normal pipeline (functions, recursion, arrays,
//!   string builtins) with no dedicated codegen or runtime helper.
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
//! - Objects render exactly as PHP does: `stdClass` as `(object) array( … )`, any
//!   other class as `\Class::__set_state(array( … ))`, and an enum case as
//!   `\Enum::Case`. PHP's object layout is NOT the array layout — an entry key sits
//!   at `indent + 3` (arrays use `indent + 2`) while the value and the closing line
//!   keep the array indents — which is why the object branch does not reuse the
//!   array branch's padding. Property visibility is deliberately absent: unlike
//!   `print_r`, PHP's `var_export` prints the bare property name.
//! - Object properties are reached through four `internal: true` helpers
//!   (`__elephc_object_is_enum`, `__elephc_object_prop_count`,
//!   `__elephc_object_prop_name`, `__elephc_object_prop_value`) because elephc has
//!   no `get_object_vars()`, no object-to-array cast, and no `foreach` over a plain
//!   object — and because `enum_exists()` needs a string literal in AOT mode, so a
//!   prelude holding a runtime `mixed` cannot ask whether it is an enum any other
//!   way. They read the same per-class descriptor `print_r` and `var_dump` walk.
//! - A value that is neither scalar, array nor object renders as the empty string.
//! - KNOWN DIVERGENCE: dynamic (undeclared) properties are not exported, matching
//!   what elephc's `var_dump`/`print_r` already do for the same objects.
//! - `__elephc_var_export_escape` takes `string`, NOT `mixed`, and every caller
//!   casts into a `string` local first. Passing a `string` value to a `mixed`
//!   parameter boxes it into a fresh Mixed cell that nothing releases, so the
//!   `mixed` spelling leaked one heap block per escaped string — one per exported
//!   string VALUE and one per exported string KEY, in every program, long before
//!   objects were in scope. `var_export_and_strstr_result_tests` pins the loop.
//! - The `$return` flag is FLAG-AWARE at the call site, mirroring `print_r`: `name_resolver`
//!   retargets a literal-flag call at [`RENDER_HELPER`] (`: string`) or [`ECHO_HELPER`]
//!   (prints, returns `null`), and only a runtime flag keeps the two-mode `var_export` body
//!   whose `string|null` return type then genuinely describes both outcomes.

use crate::parser::ast::Program;

mod detect;

/// The elephc-PHP `var_export` prelude: the public `var_export($value, $return)`
/// entry point plus the internal helpers — `__elephc_var_export_str` renders a value
/// to its parsable text, `__elephc_var_export_escape` single-quote-escapes a string,
/// `__elephc_var_export_float` reproduces `serialize_precision = -1`, and
/// `__elephc_var_export_prop` renders one object property (its own function so the
/// boxed property value is a short-lived local rather than a loop-carried one).
/// The helpers are prefixed so they cannot collide with user code, and `var_export`
/// itself is injected only when the user does not define their own.
pub const VAR_EXPORT_PRELUDE_SRC: &str = r#"<?php
function __elephc_var_export_escape(string $s): string {
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
function __elephc_var_export_prop(mixed $owner, int $index, int $indent, string $pad): string {
    $pv = __elephc_object_prop_value($owner, $index);
    if (is_array($pv) || is_object($pv)) {
        return "\n" . $pad . '  ' . __elephc_var_export_str($pv, $indent + 2);
    }
    return __elephc_var_export_str($pv, $indent + 2);
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
        $text = (string) $value;
        return "'" . __elephc_var_export_escape($text) . "'";
    }
    if (is_array($value)) {
        $pad = str_repeat(' ', $indent);
        $out = "array (\n";
        foreach ($value as $k => $v) {
            if (is_int($k)) {
                $key = (string) $k;
            } else {
                $keytext = (string) $k;
                $key = "'" . __elephc_var_export_escape($keytext) . "'";
            }
            $out = $out . $pad . '  ' . $key . ' => ';
            if (is_array($v) || is_object($v)) {
                $out = $out . "\n" . $pad . '  ' . __elephc_var_export_str($v, $indent + 2);
            } else {
                $out = $out . __elephc_var_export_str($v, $indent + 2);
            }
            $out = $out . ",\n";
        }
        $out = $out . $pad . ')';
        return $out;
    }
    if (is_object($value)) {
        $class = get_class($value);
        $pad = str_repeat(' ', $indent);
        if (__elephc_object_is_enum($value)) {
            $cases = __elephc_object_prop_count($value);
            for ($c = 0; $c < $cases; $c++) {
                if (__elephc_object_prop_name($value, $c) === 'name') {
                    return '\\' . $class . '::' . __elephc_object_prop_value($value, $c);
                }
            }
            return '\\' . $class;
        }
        if ($class === 'stdClass') {
            $out = "(object) array(\n";
            $close = ')';
        } else {
            $out = '\\' . $class . "::__set_state(array(\n";
            $close = '))';
        }
        $count = __elephc_object_prop_count($value);
        for ($i = 0; $i < $count; $i++) {
            $name = __elephc_object_prop_name($value, $i);
            if ($name === '') {
                continue;
            }
            $out = $out . $pad . '   ' . "'" . __elephc_var_export_escape($name) . "' => ";
            $out = $out . __elephc_var_export_prop($value, $i, $indent, $pad) . ",\n";
        }
        return $out . $pad . $close;
    }
    return '';
}
function __elephc_var_export_echo(mixed $value) {
    echo __elephc_var_export_str($value, 0);
    return null;
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

/// Name of the prelude helper that RENDERS a value to its parsable text and returns it.
///
/// Declared `: string`, so `crate::name_resolver` can retarget `var_export($v, true)` at it and
/// get PHP's `string` return type without contradicting what the callee actually returns. Its
/// presence in the resolved symbol table also doubles as the "the elephc prelude owns
/// `var_export`" marker — `inject_if_used` declares it only when it injects.
pub const RENDER_HELPER: &str = "__elephc_var_export_str";

/// Name of the prelude helper that PRINTS a value and returns `null`, the echo-mode contract of
/// `var_export($v)` / `var_export($v, false)` on reference PHP 8.5.6.
///
/// Left unhinted deliberately: elephc spells PHP `null` as `PhpType::Void`, which a lone
/// `return null;` infers exactly, while a `: void` hint would reject the assignment
/// `$r = var_export($v);` that PHP allows.
pub const ECHO_HELPER: &str = "__elephc_var_export_echo";

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
    let mut combined = crate::parser::parse_internal(&tokens).expect("var_export prelude must parse");
    combined.extend(program);
    combined
}
