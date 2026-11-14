//! Purpose:
//! Injects the elephc-PHP implementation of PHP's formatted-output family (`sprintf`,
//! `printf`, `vsprintf`, `vprintf`, `fprintf`, `vfprintf`) used for every call the native
//! fast path cannot handle exactly: dynamic format strings, positional (`%1$s`) and `*`
//! arguments, custom padding (`%'*8s`), the `b`/`u`/`e`/`E`/`g`/`G`/`F`/`h`/`H`
//! conversions, the `l` modifier, and calls with spread or too few arguments.
//!
//! Called from:
//! - `crate::name_resolver` rewrites such calls to the `__elephc_*printf` entry points
//!   (see `name_resolver::printf_calls`).
//! - `crate::pipeline::compile()` and the codegen test harness call [`inject_if_used`] at the
//!   same stage as the other preludes (after name resolution, before the checker).
//!
//! Key details:
//! - Mirrors `ext/standard/formatted_print.c`: modifiers (`'c`, `0`, ` `, `-`, `+`), width and
//!   precision (`*` included), sign-aware zero padding, "can't right-pad 0's on integers",
//!   `%.Nx` printing nothing, 53-digit float precision cap (without PHP's notice), `NaN`/`INF` spellings (fixed 3-column width), PHP's
//!   exponent layout (`1.5e+3`), `%g` through `zend_gcvt` rules, and the `ValueError` /
//!   `ArgumentCountError` messages.
//! - Arguments are converted with PHP casts (`(int)`, `(float)`, `(string)`), i.e.
//!   `zval_get_long` / `zval_get_double` / `zval_get_string`.
//! - Correctly rounded float digits come from the native `sprintf` with a C-compatible
//!   `%.Nf` / `%.Ne` format and a `float` argument; prelude bodies are not name-resolved, so
//!   those inner calls stay on the builtin and never recurse into this prelude.

use crate::parser::ast::{ExprKind, Program};

/// Names of the prelude entry points that rewritten calls target.
pub const PRINTF_PRELUDE_FUNCTIONS: &[&str] = &[
    "__elephc_sprintf",
    "__elephc_printf",
    "__elephc_vsprintf",
    "__elephc_vprintf",
    "__elephc_fprintf",
    "__elephc_vfprintf",
    "__elephc_sprintf_all",
    "__elephc_printf_all",
    "__elephc_fprintf_all",
    "__elephc_explode_limit",
    "__elephc_array_max",
    "__elephc_array_min",
    "__elephc_array_max_empty",
    "__elephc_array_min_empty",
    "__elephc_var_dump",
];

/// Prelude entry points that do not need the shared formatter helpers.
const STANDALONE_ENTRY_POINTS: &[&str] = &[
    "__elephc_var_dump",
    "__elephc_explode_limit",
    "__elephc_array_max",
    "__elephc_array_min",
    "__elephc_array_max_empty",
    "__elephc_array_min_empty",
];

/// The shared helpers of the elephc-PHP formatted-output prelude.
pub const PRINTF_PRELUDE_SRC: &str = r#"<?php
function __elephc_printf_pad(string $s, int $width, string $pad, bool $left, bool $neg, bool $plus): string {
    $len = strlen($s);
    $fill = $width > $len ? str_repeat($pad, $width - $len) : '';
    if ($left) {
        return $s . $fill;
    }
    if (($neg || $plus) && $pad === '0') {
        return ($neg ? '-' : '+') . $fill . substr($s, 1);
    }
    return $fill . $s;
}
function __elephc_printf_unsigned(int $n): string {
    if ($n >= 0) {
        return (string) $n;
    }
    $half = ($n >> 1) & PHP_INT_MAX;
    $q = intdiv($half, 5);
    $r = 2 * ($half - 5 * $q) + ($n & 1);
    return (string) $q . (string) $r;
}
function __elephc_printf_base(int $n, int $bits, bool $upper): string {
    $digits = $upper ? '0123456789ABCDEF' : '0123456789abcdef';
    $mask = (1 << $bits) - 1;
    $keep = PHP_INT_MAX >> ($bits - 1);
    $out = '';
    do {
        $out = $digits[$n & $mask] . $out;
        $n = ($n >> $bits) & $keep;
    } while ($n !== 0);
    return $out;
}
function __elephc_printf_c(string $format, float $value): string {
    return sprintf($format, $value);
}
function __elephc_printf_float(float $v, int $prec, string $conv, bool $plus): string {
    if (is_nan($v)) {
        return 'NaN';
    }
    if (is_infinite($v)) {
        return 'INF';
    }
    if ($conv === 'g' || $conv === 'G' || $conv === 'h' || $conv === 'H') {
        $shortest = false;
        if ($prec === 0) {
            $prec = 1;
        }
        $neg = $v < 0 || ($v == 0.0 && (string) $v === '-0');
        $a = abs($v);
        if ($a == 0.0) {
            $digits = '0';
            $decpt = 1;
        } else {
            if ($prec === -1) {
                // Shortest representation that round-trips (zend_gcvt mode 0).
                $prec = 1;
                while ($prec < 17 && (float) __elephc_printf_c('%.' . ($prec - 1) . 'e', $a) !== $a) {
                    $prec++;
                }
                $shortest = true;
            }
            $c = __elephc_printf_c('%.' . ($prec - 1) . 'e', $a);
            $e = strrpos($c, 'e');
            $digits = rtrim(str_replace('.', '', substr($c, 0, $e)), '0');
            $decpt = (int) substr($c, $e + 1) + 1;
        }
        $echar = ($conv === 'G' || $conv === 'H') ? 'E' : 'e';
        // Mode 0 (precision -1) uses ndigit 17 for the exponential-form threshold.
        if ($decpt < 0 ? $decpt < -3 : $decpt > ($shortest ? 17 : $prec)) {
            $e = $decpt - 1;
            $out = $digits[0] . '.' . (strlen($digits) > 1 ? substr($digits, 1) : '0') . $echar . ($e < 0 ? '-' : '+') . (string) abs($e);
        } elseif ($decpt <= 0) {
            $out = '0.' . str_repeat('0', -$decpt) . $digits;
        } else {
            $n = strlen($digits);
            if ($n <= $decpt) {
                $out = $digits . str_repeat('0', $decpt - $n);
            } else {
                $out = substr($digits, 0, $decpt) . '.' . substr($digits, $decpt);
            }
        }
        if ($neg) {
            return '-' . $out;
        }
        return $plus ? '+' . $out : $out;
    }
    $neg = $v < 0;
    $a = abs($v);
    if ($conv === 'f' || $conv === 'F') {
        $out = __elephc_printf_c('%.' . $prec . 'f', $a);
    } else {
        $c = __elephc_printf_c('%.' . $prec . 'e', $a);
        $e = strrpos($c, 'e');
        $exp = (int) substr($c, $e + 1);
        $out = substr($c, 0, $e) . ($conv === 'E' ? 'E' : 'e') . ($exp < 0 ? '-' : '+') . (string) abs($exp);
    }
    if ($neg) {
        return '-' . $out;
    }
    return $plus ? '+' . $out : $out;
}
function __elephc_printf_format(string $format, array $args, bool $array_form): string {
    $out = '';
    $len = strlen($format);
    $argc = count($args);
    $cur = 0;
    $missing = -1;
    $i = 0;
    while ($i < $len) {
        $p = strpos($format, '%', $i);
        if ($p === false) {
            $out .= substr($format, $i);
            break;
        }
        $out .= substr($format, $i, $p - $i);
        $i = $p + 1;
        if ($i >= $len) {
            if ($cur >= $argc) {
                $missing = max($missing, $cur);
                break;
            }
            throw new ValueError('Missing format specifier at end of string');
        }
        if ($format[$i] === '%') {
            $out .= '%';
            $i++;
            continue;
        }
        $argnum = -1;
        $pad = ' ';
        $left = false;
        $plus = false;
        $width = 0;
        $prec = 0;
        $has_prec = false;
        $expprec = false;
        if (!ctype_alpha($format[$i])) {
            $j = $i;
            while ($j < $len && ctype_digit($format[$j])) {
                $j++;
            }
            if ($j < $len && $format[$j] === '$') {
                $argnum = (int) substr($format, $i, $j - $i);
                if ($argnum <= 0) {
                    throw new ValueError('Argument number specifier must be greater than zero and less than 2147483647');
                }
                $argnum--;
                $i = $j + 1;
            }
            while ($i < $len) {
                $c = $format[$i];
                if ($c === ' ' || $c === '0') {
                    $pad = $c;
                } elseif ($c === '-') {
                    $left = true;
                } elseif ($c === '+') {
                    $plus = true;
                } elseif ($c === "'") {
                    if ($i + 1 < $len) {
                        $i++;
                        $pad = $format[$i];
                    } else {
                        throw new ValueError('Missing padding character');
                    }
                } else {
                    break;
                }
                $i++;
            }
            if ($i < $len && $format[$i] === '*') {
                $i++;
                $j = $i;
                while ($j < $len && ctype_digit($format[$j])) {
                    $j++;
                }
                if ($j > $i && $j < $len && $format[$j] === '$') {
                    $warg = (int) substr($format, $i, $j - $i) - 1;
                    $i = $j + 1;
                } else {
                    $warg = $cur;
                    $cur++;
                }
                if ($warg >= $argc) {
                    $missing = max($missing, $warg);
                    $width = 0;
                } else {
                    if (!is_int($args[$warg])) {
                        throw new ValueError('Width must be an integer');
                    }
                    $width = (int) $args[$warg];
                    if ($width < 0 || $width > 2147483647) {
                        throw new ValueError('Width must be greater than zero and less than 2147483647');
                    }
                }
            } else {
                $j = $i;
                while ($j < $len && ctype_digit($format[$j])) {
                    $j++;
                }
                if ($j > $i) {
                    $width = (int) substr($format, $i, $j - $i);
                    $i = $j;
                }
            }
            if ($i < $len && $format[$i] === '.') {
                $i++;
                if ($i < $len && $format[$i] === '*') {
                    $has_prec = true;
                    $i++;
                    $j = $i;
                    while ($j < $len && ctype_digit($format[$j])) {
                        $j++;
                    }
                    if ($j > $i && $j < $len && $format[$j] === '$') {
                        $parg = (int) substr($format, $i, $j - $i) - 1;
                        $i = $j + 1;
                    } else {
                        $parg = $cur;
                        $cur++;
                    }
                    if ($parg >= $argc) {
                        $missing = max($missing, $parg);
                    } else {
                        if (!is_int($args[$parg])) {
                            throw new ValueError('Precision must be an integer');
                        }
                        $prec = (int) $args[$parg];
                        if ($prec < -1 || $prec > 2147483647) {
                            throw new ValueError('Precision must be between -1 and 2147483647');
                        }
                    }
                    $expprec = true;
                } else {
                    $j = $i;
                    while ($j < $len && ctype_digit($format[$j])) {
                        $j++;
                    }
                    if ($j > $i) {
                        $prec = (int) substr($format, $i, $j - $i);
                        $i = $j;
                        $expprec = true;
                        $has_prec = true;
                    }
                }
            }
        }
        if ($i < $len && $format[$i] === 'l') {
            $i++;
        }
        if ($argnum < 0) {
            $argnum = $cur;
            $cur++;
        }
        if ($i >= $len) {
            if ($argnum >= $argc) {
                $missing = max($missing, $argnum);
                break;
            }
            throw new ValueError('Missing format specifier at end of string');
        }
        $conv = $format[$i];
        $i++;
        if ($argnum >= $argc) {
            if (strpos('sduceEfFgGhHoxXb%', $conv) === false) {
                throw new ValueError('Unknown format specifier "' . $conv . '"');
            }
            $missing = max($missing, $argnum);
            continue;
        }
        $arg = $args[$argnum];
        if ($conv === 's') {
            $s = (string) $arg;
            if ($expprec && $prec >= 0 && $prec < strlen($s)) {
                $s = substr($s, 0, $prec);
            }
            $out .= __elephc_printf_pad($s, $width, $pad, $left, false, false);
        } elseif ($conv === 'd') {
            $n = (int) $arg;
            $s = (string) $n;
            if ($n >= 0 && $plus) {
                $s = '+' . $s;
            }
            if ($left && $pad === '0') {
                $pad = ' ';
            }
            $out .= __elephc_printf_pad($s, $width, $pad, $left, $n < 0, $plus);
        } elseif ($conv === 'u') {
            if ($left && $pad === '0') {
                $pad = ' ';
            }
            $out .= __elephc_printf_pad(__elephc_printf_unsigned((int) $arg), $width, $pad, $left, false, false);
        } elseif ($conv === 'e' || $conv === 'E' || $conv === 'f' || $conv === 'F' || $conv === 'g' || $conv === 'G' || $conv === 'h' || $conv === 'H') {
            $is_g = $conv === 'g' || $conv === 'G' || $conv === 'h' || $conv === 'H';
            if (!$has_prec) {
                $prec = 6;
            } elseif ($prec > 53) {
                $prec = 53;
            } elseif ($prec === -1 && !$is_g) {
                throw new ValueError('Precision -1 is only supported for %g, %G, %h and %H');
            }
            $fv = (float) $arg;
            $s = __elephc_printf_float($fv, $prec, $conv, $plus);
            if (is_nan($fv) || is_infinite($fv)) {
                $out .= __elephc_printf_pad($s, 3, $pad, $left, $fv < 0, $plus);
            } else {
                $out .= __elephc_printf_pad($s, $width, $pad, $left, $s[0] === '-', $plus);
            }
        } elseif ($conv === 'c') {
            $out .= chr((int) $arg);
        } elseif ($conv === 'o' || $conv === 'x' || $conv === 'X' || $conv === 'b') {
            $bits = $conv === 'o' ? 3 : ($conv === 'b' ? 1 : 4);
            $s = $expprec ? '' : __elephc_printf_base((int) $arg, $bits, $conv === 'X');
            $out .= __elephc_printf_pad($s, $width, $pad, $left, false, false);
        } elseif ($conv === '%') {
            $out .= '%';
        } else {
            throw new ValueError('Unknown format specifier "' . $conv . '"');
        }
    }
    if ($missing >= 0) {
        if ($array_form) {
            throw new ValueError('The arguments array must contain ' . ($missing + 1) . ' items, ' . $argc . ' given');
        }
        throw new ArgumentCountError(($missing + 2) . ' arguments are required, ' . ($argc + 1) . ' given');
    }
    return $out;
}
"#;

/// Entry points, each injected only when a rewritten call targets it (an unused
/// `fprintf` body would otherwise be checked without any call site to type its stream).
const PRINTF_ENTRY_POINTS: &[(&str, &str)] = &[
    (
        // `var_dump()`: arrays are walked here (any nesting, PHP's two-space indentation and
        // `[int]=>` / `["string"]=>` keys); every other value, nested or not, is printed by the
        // native `var_dump` after the indentation (prelude bodies are not name-resolved, so
        // that inner call stays on the builtin).
        "__elephc_var_dump",
        r#"function __elephc_var_dump_array(array $v, string $ind): void {
    echo $ind, 'array(', count($v), ") {\n";
    foreach ($v as $k => $item) {
        if (is_int($k)) {
            echo $ind, '  [', $k, "]=>\n";
        } else {
            echo $ind, '  ["', $k, "\"]=>\n";
        }
        __elephc_var_dump_value($item, $ind . '  ');
    }
    echo $ind, "}\n";
}
function __elephc_var_dump_value(mixed $v, string $ind): void {
    if (is_array($v)) {
        __elephc_var_dump_array($v, $ind);
        return;
    }
    echo $ind;
    var_dump($v);
}
function __elephc_var_dump(mixed ...$values): void {
    foreach ($values as $v) {
        __elephc_var_dump_value($v, '');
    }
}"#,
    ),
    (
        "__elephc_sprintf_all",
        r#"function __elephc_sprintf_all(array $all): string {
    $all = array_values($all);
    $format = (string) array_shift($all);
    return __elephc_printf_format($format, $all, false);
}"#,
    ),
    (
        "__elephc_printf_all",
        r#"function __elephc_printf_all(array $all): int {
    $all = array_values($all);
    $format = (string) array_shift($all);
    $s = __elephc_printf_format($format, $all, false);
    echo $s;
    return strlen($s);
}"#,
    ),
    (
        "__elephc_fprintf_all",
        r#"function __elephc_fprintf_all(array $all): int {
    $all = array_values($all);
    $stream = array_shift($all);
    $format = (string) array_shift($all);
    $s = __elephc_printf_format($format, $all, false);
    fwrite($stream, $s);
    return strlen($s);
}"#,
    ),
    (
        "__elephc_sprintf",
        r#"function __elephc_sprintf(string $format, mixed ...$values): string {
    return __elephc_printf_format($format, $values, false);
}"#,
    ),
    (
        "__elephc_vsprintf",
        r#"function __elephc_vsprintf(string $format, array $values): string {
    return __elephc_printf_format($format, array_values($values), true);
}"#,
    ),
    (
        "__elephc_printf",
        r#"function __elephc_printf(string $format, mixed ...$values): int {
    $s = __elephc_printf_format($format, $values, false);
    echo $s;
    return strlen($s);
}"#,
    ),
    (
        "__elephc_vprintf",
        r#"function __elephc_vprintf(string $format, array $values): int {
    $s = __elephc_printf_format($format, array_values($values), true);
    echo $s;
    return strlen($s);
}"#,
    ),
    (
        "__elephc_fprintf",
        r#"function __elephc_fprintf(mixed $stream, string $format, mixed ...$values): int {
    $s = __elephc_printf_format($format, $values, false);
    fwrite($stream, $s);
    return strlen($s);
}"#,
    ),
    (
        "__elephc_vfprintf",
        r#"function __elephc_vfprintf(mixed $stream, string $format, array $values): int {
    $s = __elephc_printf_format($format, array_values($values), true);
    fwrite($stream, $s);
    return strlen($s);
}"#,
    ),
    (
        "__elephc_explode_limit",
        r#"function __elephc_explode_limit(string $separator, string $string, int $limit): array {
    $parts = explode($separator, $string);
    if ($limit === 0) {
        $limit = 1;
    }
    $count = count($parts);
    $out = [];
    if ($limit > 0) {
        if ($count <= $limit) {
            return $parts;
        }
        for ($i = 0; $i < $limit - 1; $i++) {
            $out[] = $parts[$i];
        }
        $rest = $parts[$limit - 1];
        for ($i = $limit; $i < $count; $i++) {
            $rest .= $separator . $parts[$i];
        }
        $out[] = $rest;
        return $out;
    }
    for ($i = 0; $i < $count + $limit; $i++) {
        $out[] = $parts[$i];
    }
    return $out;
}"#,
    ),
    (
        "__elephc_array_max",
        r#"function __elephc_array_max(array $value): mixed {
    if (count($value) === 0) {
        throw new ValueError("max(): Argument #1 (\$value) must contain at least one element");
    }
    $first = true;
    $best = null;
    foreach ($value as $item) {
        if ($first || $item > $best) {
            $best = $item;
            $first = false;
        }
    }
    return $best;
}"#,
    ),
    (
        "__elephc_array_min",
        r#"function __elephc_array_min(array $value): mixed {
    if (count($value) === 0) {
        throw new ValueError("min(): Argument #1 (\$value) must contain at least one element");
    }
    $first = true;
    $best = null;
    foreach ($value as $item) {
        if ($first || $item < $best) {
            $best = $item;
            $first = false;
        }
    }
    return $best;
}"#,
    ),
    (
        "__elephc_array_max_empty",
        r#"function __elephc_array_max_empty(array $value): mixed {
    throw new ValueError("max(): Argument #1 (\$value) must contain at least one element");
}"#,
    ),
    (
        "__elephc_array_min_empty",
        r#"function __elephc_array_min_empty(array $value): mixed {
    throw new ValueError("min(): Argument #1 (\$value) must contain at least one element");
}"#,
    ),
];

/// Prepends the prelude when the program calls one of its entry points (the name resolver
/// rewrites eligible `*printf` calls to them), so other binaries carry nothing.
pub fn inject_if_used(program: Program) -> Program {
    let mut used: Vec<&'static str> = Vec::new();
    crate::int_range::walk_program(&program, &mut |node| {
        if let crate::int_range::Node::Expr(expr) = node {
            if let ExprKind::FunctionCall { name, .. } = &expr.kind {
                if let Some(entry) = name.last_segment().and_then(|segment| {
                    PRINTF_PRELUDE_FUNCTIONS.iter().find(|entry| **entry == segment)
                }) {
                    if !used.contains(entry) {
                        used.push(entry);
                    }
                }
            }
        }
    });
    if used.is_empty() {
        return program;
    }
    let mut source = if used.iter().all(|name| STANDALONE_ENTRY_POINTS.contains(name)) {
        String::from("<?php\n")
    } else {
        PRINTF_PRELUDE_SRC.to_string()
    };
    for (name, body) in PRINTF_ENTRY_POINTS {
        if used.contains(name) {
            source.push_str(body);
            source.push('\n');
        }
    }
    let tokens = crate::lexer::tokenize(&source).expect("printf prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("printf prelude must parse");
    combined.extend(program);
    combined
}
