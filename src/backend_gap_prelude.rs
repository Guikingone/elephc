//! Purpose:
//! Supplies narrow elephc-PHP implementations for builtin call shapes whose native EIR backend
//! is not available yet.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness before type checking.
//!
//! Key details:
//! - Only the exact supported arities are rewritten, leaving richer builtin forms on their native
//!   diagnostic path instead of silently weakening their PHP semantics.

use crate::parser::ast::Program;

/// Reserved helper used for two-argument `levenshtein()` calls.
pub(crate) const LEVENSHTEIN_TWO_ARG_NAME: &str = "__elephc_levenshtein_two_arg";

/// Reserved helper used for one-argument `strip_tags()` calls.
pub(crate) const STRIP_TAGS_ONE_ARG_NAME: &str = "__elephc_strip_tags_one_arg";

/// Reserved helper used for `is_countable()` calls.
pub(crate) const IS_COUNTABLE_NAME: &str = "__elephc_is_countable";

/// Reserved helper used for ordinary positional `array_slice()` calls.
pub(crate) const ARRAY_SLICE_NAME: &str = "__elephc_array_slice";

/// Reserved helper used for the integer-only `pack()` formats required by Symfony.
pub(crate) const PACK_INTEGER_NAME: &str = "__elephc_pack_integer";

/// Reserved helper used for `random_bytes()`.
pub(crate) const RANDOM_BYTES_NAME: &str = "__elephc_random_bytes";

/// Reserved helper used for one-argument `sort()` calls on gradual indexed arrays.
pub(crate) const SORT_MIXED_NAME: &str = "__elephc_sort_mixed";

/// Reserved helper used for two-string-list `array_diff()` calls.
pub(crate) const ARRAY_DIFF_STRING_NAME: &str = "__elephc_array_diff_string";

/// Reserved helper used for one-argument `array_reverse()` calls on string lists.
pub(crate) const ARRAY_REVERSE_STRING_NAME: &str = "__elephc_array_reverse_string";

/// Reserved helper used for `array_search()` calls on gradual indexed arrays.
pub(crate) const ARRAY_SEARCH_MIXED_NAME: &str = "__elephc_array_search_mixed";

/// Byte-oriented dynamic-programming implementation matching PHP's default edit costs.
const LEVENSHTEIN_TWO_ARG_SRC: &str = r#"<?php
function __elephc_levenshtein_two_arg(string $first, string $second): int {
    $secondLength = strlen($second);
    $previous = [];
    $j = 0;
    while ($j <= $secondLength) {
        $previous[] = $j;
        $j++;
    }
    $firstLength = strlen($first);
    $i = 1;
    while ($i <= $firstLength) {
        $current = [$i];
        $j = 1;
        while ($j <= $secondLength) {
            $insert = $current[$j - 1] + 1;
            $delete = $previous[$j] + 1;
            $replace = $previous[$j - 1] + ($first[$i - 1] === $second[$j - 1] ? 0 : 1);
            $best = $insert;
            if ($delete < $best) {
                $best = $delete;
            }
            if ($replace < $best) {
                $best = $replace;
            }
            $current[] = $best;
            $j++;
        }
        $previous = $current;
        $i++;
    }
    return $previous[$secondLength];
}
"#;

/// Quote-aware tag scanner for the no-allowlist `strip_tags()` form used by Symfony renderers.
const STRIP_TAGS_ONE_ARG_SRC: &str = r#"<?php
function __elephc_strip_tags_one_arg(string $input): string {
    $output = '';
    $length = strlen($input);
    $inside = false;
    $quote = '';
    $i = 0;
    while ($i < $length) {
        $char = $input[$i];
        if (!$inside) {
            if ($char === '<') {
                $inside = true;
                $quote = '';
            } else {
                $output .= $char;
            }
        } elseif ($quote !== '') {
            if ($char === $quote) {
                $quote = '';
            }
        } elseif ($char === '"' || $char === "'") {
            $quote = $char;
        } elseif ($char === '>') {
            $inside = false;
        }
        $i++;
    }
    return $output;
}
"#;

/// PHP's countability predicate for arrays and objects implementing `Countable`.
const IS_COUNTABLE_SRC: &str = r#"<?php
function __elephc_is_countable(mixed $value): bool {
    return is_array($value) || $value instanceof \Countable;
}
"#;

/// Representation-neutral array slicing with PHP key-preservation rules.
const ARRAY_SLICE_SRC: &str = r#"<?php
function __elephc_array_slice(array $input, int $offset, mixed $length = null, bool $preserveKeys = false): array {
    $count = count($input);
    if ($offset < 0) {
        $start = $count + $offset;
        if ($start < 0) {
            $start = 0;
        }
    } else {
        $start = $offset;
        if ($start > $count) {
            $start = $count;
        }
    }
    $end = $count;
    if ($length !== null) {
        $lengthValue = (int) $length;
        if ($lengthValue >= 0) {
            $end = $start + $lengthValue;
            if ($end > $count) {
                $end = $count;
            }
        } else {
            $end = $count + $lengthValue;
            if ($end < $start) {
                $end = $start;
            }
        }
    }
    $result = [];
    $position = 0;
    foreach ($input as $key => $value) {
        if ($position >= $start && $position < $end) {
            if ($preserveKeys || is_string($key)) {
                $result[$key] = $value;
            } else {
                $result[] = $value;
            }
        }
        $position++;
    }
    return $result;
}
"#;

/// Packs little- and big-endian unsigned integer fields for `V`, `N`, and `n` formats.
const PACK_INTEGER_SRC: &str = r#"<?php
function __elephc_pack_integer(string $format, int ...$values): string {
    $result = '';
    $length = strlen($format);
    $i = 0;
    while ($i < $length) {
        $value = $values[$i];
        $code = $format[$i];
        if ($code === 'V') {
            $result .= chr($value & 255);
            $result .= chr(($value >> 8) & 255);
            $result .= chr(($value >> 16) & 255);
            $result .= chr(($value >> 24) & 255);
        } elseif ($code === 'N') {
            $result .= chr(($value >> 24) & 255);
            $result .= chr(($value >> 16) & 255);
            $result .= chr(($value >> 8) & 255);
            $result .= chr($value & 255);
        } else {
            $result .= chr(($value >> 8) & 255);
            $result .= chr($value & 255);
        }
        $i++;
    }
    return $result;
}
"#;

/// Builds a binary string from Elephc's cryptographic `random_int()` primitive.
const RANDOM_BYTES_SRC: &str = r#"<?php
function __elephc_random_bytes(int $length): string {
    if ($length < 1) {
        throw new \ValueError('random_bytes(): Argument #1 ($length) must be greater than 0');
    }
    $result = '';
    $i = 0;
    while ($i < $length) {
        $result .= chr(random_int(0, 255));
        $i++;
    }
    return $result;
}
"#;

/// Stable in-place ascending sort using PHP's ordinary comparison semantics.
const SORT_MIXED_SRC: &str = r#"<?php
function __elephc_sort_mixed(array &$values): bool {
    $count = count($values);
    $outer = $count;
    while ($outer > 1) {
        $inner = 0;
        while ($inner + 1 < $outer) {
            $next = $inner + 1;
            if ($values[$inner] > $values[$next]) {
                $temporary = $values[$inner];
                $values[$inner] = $values[$next];
                $values[$next] = $temporary;
            }
            $inner++;
        }
        $outer--;
    }
    return true;
}
"#;

/// Two-list string difference that preserves the first list's keys and order.
const ARRAY_DIFF_STRING_SRC: &str = r#"<?php
function __elephc_array_diff_string(array $left, array $right): array {
    $result = [];
    foreach ($left as $key => $value) {
        $found = false;
        foreach ($right as $candidate) {
            if ($value === $candidate) {
                $found = true;
                break;
            }
        }
        if (!$found) {
            $result[$key] = $value;
        }
    }
    return $result;
}
"#;

/// Reverses a packed string list while renumbering its integer keys from zero.
const ARRAY_REVERSE_STRING_SRC: &str = r#"<?php
function __elephc_array_reverse_string(array $input): array {
    $result = [];
    $position = count($input);
    while ($position > 0) {
        $position--;
        $result[] = $input[$position];
    }
    return $result;
}
"#;

/// Searches a gradual packed list using PHP's loose or strict equality semantics.
const ARRAY_SEARCH_MIXED_SRC: &str = r#"<?php
function __elephc_array_search_mixed(mixed $needle, array $haystack, bool $strict = false): int|false {
    foreach ($haystack as $key => $value) {
        if ($strict ? $needle === $value : $needle == $value) {
            return $key;
        }
    }
    return false;
}
"#;

/// Prepends each narrow helper whose corresponding PHP builtin is referenced.
pub fn inject_if_used(program: Program) -> Program {
    let usage = crate::ast_usage::collect(&program);
    let mut sources = Vec::new();
    if usage.references("levenshtein") {
        sources.push(LEVENSHTEIN_TWO_ARG_SRC);
    }
    if usage.references("strip_tags") {
        sources.push(STRIP_TAGS_ONE_ARG_SRC);
    }
    if usage.references("is_countable") {
        sources.push(IS_COUNTABLE_SRC);
    }
    if usage.references("array_slice") {
        sources.push(ARRAY_SLICE_SRC);
    }
    if usage.references("pack") {
        sources.push(PACK_INTEGER_SRC);
    }
    if usage.references("random_bytes") {
        sources.push(RANDOM_BYTES_SRC);
    }
    if usage.references("sort") {
        sources.push(SORT_MIXED_SRC);
    }
    if usage.references("array_diff") {
        sources.push(ARRAY_DIFF_STRING_SRC);
    }
    if usage.references("array_reverse") {
        sources.push(ARRAY_REVERSE_STRING_SRC);
    }
    if usage.references("array_search") {
        sources.push(ARRAY_SEARCH_MIXED_SRC);
    }
    if sources.is_empty() {
        return program;
    }
    let mut combined = Vec::new();
    for source in sources {
        let tokens = crate::lexer::tokenize(source).expect("backend gap prelude must tokenize");
        combined.extend(crate::parser::parse(&tokens).expect("backend gap prelude must parse"));
    }
    combined.extend(program);
    combined
}
