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

/// Reserved helper used when `array_splice()` receives a gradual replacement value.
pub(crate) const ARRAY_SPLICE_MIXED_REPLACEMENT_NAME: &str =
    "__elephc_array_splice_mixed_replacement";

/// Reserved helper used for `array_unshift()` with one trailing positional spread.
pub(crate) const ARRAY_UNSHIFT_TRAILING_SPREAD_NAME: &str =
    "__elephc_array_unshift_trailing_spread";

/// Reserved helper used for three-argument `preg_replace()` calls with an array of patterns.
pub(crate) const PREG_REPLACE_ARRAY_NAME: &str = "__elephc_preg_replace_array";

/// Reserved helper used for supported integer-only `pack()` formats.
pub(crate) const PACK_INTEGER_NAME: &str = "__elephc_pack_integer";

/// Reserved helper used for `random_bytes()`.
pub(crate) const RANDOM_BYTES_NAME: &str = "__elephc_random_bytes";

/// Reserved helper used for one-argument `sort()` calls on gradual indexed arrays.
pub(crate) const SORT_MIXED_NAME: &str = "__elephc_sort_mixed";

/// Reserved helper used for one-argument `rsort()` calls on gradual indexed arrays.
pub(crate) const RSORT_MIXED_NAME: &str = "__elephc_rsort_mixed";

/// Reserved helper used for one-argument `array_unique()` calls on gradual associative arrays.
pub(crate) const ARRAY_UNIQUE_ASSOC_MIXED_NAME: &str = "__elephc_array_unique_assoc_mixed";

/// Reserved helper used for two-argument `array_diff()` calls.
pub(crate) const ARRAY_DIFF_STRING_NAME: &str = "__elephc_array_diff_string";

/// Reserved helper used for two-argument `array_intersect()` calls.
pub(crate) const ARRAY_INTERSECT_NAME: &str = "__elephc_array_intersect";

/// Reserved helper used for one-argument `array_reverse()` calls on gradual arrays.
pub(crate) const ARRAY_REVERSE_GRADUAL_NAME: &str = "__elephc_array_reverse_gradual";

/// Reserved helper used for `array_combine()` across generic array layouts.
pub(crate) const ARRAY_COMBINE_GRADUAL_NAME: &str = "__elephc_array_combine_gradual";

/// Reserved helper used for `array_search()` calls on gradual indexed arrays.
pub(crate) const ARRAY_SEARCH_MIXED_NAME: &str = "__elephc_array_search_mixed";

/// Reserved helper used for `http_build_query()` calls on arrays.
pub(crate) const HTTP_BUILD_QUERY_NAME: &str = "__elephc_http_build_query";

/// Reserved helper used for `escapeshellarg()` calls.
pub(crate) const ESCAPESHELLARG_NAME: &str = "__elephc_escapeshellarg";

/// Reserved helper used for `cli_set_process_title()` calls outside a native CLI bridge.
pub(crate) const CLI_SET_PROCESS_TITLE_NAME: &str = "__elephc_cli_set_process_title";

/// Reserved helper used when `file_put_contents()` receives array payload pieces.
pub(crate) const FILE_PUT_CONTENTS_ARRAY_NAME: &str = "__elephc_file_put_contents_array";

/// Reserved helper used for value-callback sorting while preserving array keys.
pub(crate) const UASORT_MIXED_NAME: &str = "__elephc_uasort_mixed";

/// Reserved helper used for value-callback sorting with numeric reindexing.
pub(crate) const USORT_MIXED_NAME: &str = "__elephc_usort_mixed";

/// Reserved helper used for key-callback sorting while preserving associations.
pub(crate) const UKSORT_MIXED_NAME: &str = "__elephc_uksort_mixed";

/// Reserved helper used for ordinary ascending sorting while preserving keys.
pub(crate) const ASORT_MIXED_NAME: &str = "__elephc_asort_mixed";

/// Reserved helper used when `array_fill_keys()` receives a gradual array operand.
pub(crate) const ARRAY_FILL_KEYS_MIXED_NAME: &str = "__elephc_array_fill_keys_mixed";

/// Reserved helper used for `str_getcsv()` calls.
pub(crate) const STR_GETCSV_NAME: &str = "__elephc_str_getcsv";

/// Canonical name of PHP's randomizer class supplied by the compatibility prelude.
pub(crate) const RANDOMIZER_CLASS_NAME: &str = "Random\\Randomizer";

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

/// Quote-aware tag scanner for the no-allowlist `strip_tags()` form.
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
///
/// The source stays `mixed` so a boxed associative array reaches `count()` and `foreach` through
/// their runtime-tag-aware paths instead of being reinterpreted as an unboxed array pointer at the
/// internal helper boundary.
const ARRAY_SLICE_SRC: &str = r#"<?php
function __elephc_array_slice(mixed $input, int $offset, ?int $length = null, bool $preserveKeys = false): array {
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

/// Sequential regex replacement for the three-argument array-pattern form.
const PREG_REPLACE_ARRAY_SRC: &str = r#"<?php
function __elephc_preg_replace_array(array $patterns, array|string $replacement, string $subject): string {
    $result = $subject;
    $index = 0;
    foreach ($patterns as $pattern) {
        if (is_array($replacement)) {
            $current = $replacement[$index] ?? '';
        } else {
            $current = $replacement;
        }
        $result = preg_replace((string) $pattern, (string) $current, $result);
        $index++;
    }
    return $result;
}
"#;

/// Normalizes a gradual `array_splice()` replacement through PHP's ordinary array cast.
const ARRAY_SPLICE_MIXED_REPLACEMENT_SRC: &str = r#"<?php
function __elephc_array_splice_mixed_replacement(array &$array, int $offset, ?int $length = null, mixed $replacement = []): array {
    $normalized = (array) $replacement;
    return array_splice($array, $offset, $length, $normalized);
}
"#;

/// Prepends fixed values followed by one trailing spread while retaining source order.
const ARRAY_UNSHIFT_TRAILING_SPREAD_SRC: &str = r#"<?php
function __elephc_array_unshift_trailing_spread(mixed &$array, array $leading, array $spread): int {
    if (!is_array($array)) {
        throw new TypeError('array_unshift(): Argument #1 ($array) must be of type array');
    }
    $normalized = (array) $array;
    $index = count($spread);
    while ($index > 0) {
        $index--;
        array_unshift($normalized, $spread[$index]);
    }
    $index = count($leading);
    while ($index > 0) {
        $index--;
        array_unshift($normalized, $leading[$index]);
    }
    $array = $normalized;
    return count($normalized);
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

/// PHP-level implementation of the currently supported Randomizer surface.
const RANDOMIZER_SRC: &str = r#"<?php
namespace Random;
final class Randomizer {
    public function __construct(mixed $engine = null) {}

    public function getBytesFromString(string $string, int $length): string {
        if ($string === '') {
            throw new \ValueError('Random\\Randomizer::getBytesFromString(): Argument #1 ($string) must not be empty');
        }
        if ($length < 1) {
            throw new \ValueError('Random\\Randomizer::getBytesFromString(): Argument #2 ($length) must be greater than 0');
        }
        $result = '';
        $maximum = strlen($string) - 1;
        $position = 0;
        while ($position < $length) {
            $result .= $string[random_int(0, (int) $maximum)];
            $position++;
        }
        return $result;
    }
}
"#;

/// Stable in-place value sort using PHP's ordinary comparison semantics.
const SORT_MIXED_SRC: &str = r#"<?php
function __elephc_sort_mixed_direction(mixed &$array, bool $descending): bool {
    $array = array_values($array);
    $count = count($array);
    $outer = $count;
    while ($outer > 1) {
        $inner = 0;
        while ($inner + 1 < $outer) {
            $next = $inner + 1;
            $current = $array[$inner];
            $candidate = $array[$next];
            $comparison = is_string($current) && is_string($candidate)
                ? strcmp($current, $candidate)
                : ($current == $candidate ? 0 : ($current > $candidate ? 1 : -1));
            $swap = $descending ? $comparison < 0 : $comparison > 0;
            if ($swap) {
                $temporary = $array[$inner];
                $array[$inner] = $array[$next];
                $array[$next] = $temporary;
            }
            $inner++;
        }
        $outer--;
    }
    return true;
}
function __elephc_sort_mixed(mixed &$array): bool {
    return __elephc_sort_mixed_direction($array, false);
}
function __elephc_rsort_mixed(mixed &$array): bool {
    return __elephc_sort_mixed_direction($array, true);
}
"#;

/// Default `SORT_STRING` de-duplication for gradual associative values, preserving first keys.
const ARRAY_UNIQUE_ASSOC_MIXED_SRC: &str = r#"<?php
function __elephc_array_unique_assoc_mixed(array $values): array {
    $seen = ["__elephc_seed" => true];
    unset($seen["__elephc_seed"]);
    $result = ["__elephc_seed" => null];
    unset($result["__elephc_seed"]);
    foreach ($values as $key => $value) {
        $comparison = (string) $value;
        if (!isset($seen[$comparison])) {
            $seen[$comparison] = true;
            $result[$key] = $value;
        }
    }
    return $result;
}
"#;

/// Two-array difference that compares values by PHP string cast and preserves left keys.
const ARRAY_DIFF_STRING_SRC: &str = r#"<?php
function __elephc_array_diff_string(array $left, array $right): array {
    $result = ["__elephc_seed" => null];
    unset($result["__elephc_seed"]);
    foreach ($left as $key => $value) {
        $found = false;
        foreach ($right as $candidate) {
            if ((string) $value === (string) $candidate) {
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

/// Two-array intersection that compares values by PHP string cast and preserves left keys.
const ARRAY_INTERSECT_SRC: &str = r#"<?php
function __elephc_array_intersect(array $left, array $right): mixed {
    $result = [];
    foreach ($left as $key => $value) {
        foreach ($right as $candidate) {
            if ((string) $value === (string) $candidate) {
                if (is_string($key)) {
                    $result[(string) $key] = $value;
                } else {
                    $result[(int) $key] = $value;
                }
                break;
            }
        }
    }
    return $result;
}
"#;

/// Reverses a gradual array, renumbering integer keys while preserving string keys.
const ARRAY_REVERSE_GRADUAL_SRC: &str = r#"<?php
function __elephc_array_reverse_gradual(array $input): array {
    $keys = [];
    $values = [];
    foreach ($input as $key => $value) {
        $keys[] = $key;
        $values[] = $value;
    }
    $result = [];
    $position = count($values);
    while ($position > 0) {
        $position--;
        $key = $keys[$position];
        if (is_string($key)) {
            $result[$key] = $values[$position];
        } else {
            $result[] = $values[$position];
        }
    }
    return $result;
}
"#;

/// Combines generic array layouts in iteration order after validating equal cardinality.
const ARRAY_COMBINE_GRADUAL_SRC: &str = r#"<?php
function __elephc_array_combine_gradual(array $keys, array $values): array {
    if (count($keys) !== count($values)) {
        throw new ValueError('array_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements');
    }
    $orderedValues = [];
    foreach ($values as $value) {
        $orderedValues[] = $value;
    }
    $result = [];
    $position = 0;
    foreach ($keys as $key) {
        if (!is_int($key) && !is_string($key)) {
            $key = (string) $key;
        }
        $result[$key] = $orderedValues[$position];
        $position++;
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

/// Recursive query-string builder covering PHP's RFC 1738 and RFC 3986 array encodings.
const HTTP_BUILD_QUERY_SRC: &str = r#"<?php
function __elephc_http_build_query_pairs(array $data, string $prefix, string $numericPrefix, int $encodingType): array {
    $pairs = [];
    foreach ($data as $key => $value) {
        $keyText = is_int($key) ? $numericPrefix.(string) $key : (string) $key;
        $fullKey = $prefix === '' ? $keyText : $prefix.'['.$keyText.']';
        if (is_array($value)) {
            $nested = __elephc_http_build_query_pairs($value, $fullKey, '', $encodingType);
            foreach ($nested as $pair) {
                $pairs[] = $pair;
            }
        } elseif ($value !== null) {
            $text = is_bool($value) ? ($value ? '1' : '0') : (string) $value;
            $encodedKey = $encodingType === 2 ? rawurlencode($fullKey) : urlencode($fullKey);
            $encodedValue = $encodingType === 2 ? rawurlencode($text) : urlencode($text);
            $pairs[] = $encodedKey.'='.$encodedValue;
        }
    }
    return $pairs;
}

function __elephc_http_build_query(array $data, string $numericPrefix = '', mixed $argSeparator = null, int $encodingType = 1): string {
    $separator = $argSeparator === null ? '&' : (string) $argSeparator;
    return implode($separator, __elephc_http_build_query_pairs($data, '', $numericPrefix, $encodingType));
}
"#;

/// POSIX shell argument quoting compatible with PHP on the supported Unix targets.
const ESCAPESHELLARG_SRC: &str = r#"<?php
function __elephc_escapeshellarg(string $argument): string {
    $result = "'";
    $length = strlen($argument);
    $position = 0;
    while ($position < $length) {
        $character = $argument[$position];
        $result .= $character === "'" ? "'\\''" : $character;
        $position++;
    }
    return $result."'";
}
"#;

/// Reports that process-title mutation is unavailable without changing observable process state.
const CLI_SET_PROCESS_TITLE_SRC: &str = r#"<?php
function __elephc_cli_set_process_title(string $title): bool {
    return false;
}
"#;

/// Concatenates array pieces before delegating to the native file writer, as PHP does.
const FILE_PUT_CONTENTS_ARRAY_SRC: &str = r#"<?php
function __elephc_file_put_contents_array(string $filename, array $data, int $flags = 0, mixed $context = null): int|false {
    $contents = '';
    foreach ($data as $piece) {
        $contents .= (string) $piece;
    }
    return file_put_contents($filename, $contents, $flags, $context);
}
"#;

/// Stable callback sorts for arrays with gradual element layouts, preserving PHP's key rules.
const CALLBACK_SORT_MIXED_SRC: &str = r#"<?php
function __elephc_uasort_mixed(array &$values, callable $callback): bool {
    $keys = [];
    foreach ($values as $key => $value) {
        $keys[] = $key;
    }
    $count = count($keys);
    $outer = $count;
    while ($outer > 1) {
        $inner = 0;
        while ($inner + 1 < $outer) {
            $next = $inner + 1;
            if ($callback($values[$keys[$inner]], $values[$keys[$next]]) > 0) {
                $temporary = $keys[$inner];
                $keys[$inner] = $keys[$next];
                $keys[$next] = $temporary;
            }
            $inner++;
        }
        $outer--;
    }
    $sorted = ["__elephc_seed" => null];
    unset($sorted["__elephc_seed"]);
    foreach ($keys as $key) {
        $sorted[$key] = $values[$key];
    }
    $values = $sorted;
    return true;
}

function __elephc_usort_mixed(array &$values, callable $callback): bool {
    $values = array_values($values);
    $count = count($values);
    $outer = $count;
    while ($outer > 1) {
        $inner = 0;
        while ($inner + 1 < $outer) {
            $next = $inner + 1;
            if ($callback($values[$inner], $values[$next]) > 0) {
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

function __elephc_uksort_mixed(array &$values, callable $callback): bool {
    $keys = [];
    foreach ($values as $key => $value) {
        $keys[] = $key;
    }
    $count = count($keys);
    $outer = $count;
    while ($outer > 1) {
        $inner = 0;
        while ($inner + 1 < $outer) {
            $next = $inner + 1;
            if ($callback($keys[$inner], $keys[$next]) > 0) {
                $temporary = $keys[$inner];
                $keys[$inner] = $keys[$next];
                $keys[$next] = $temporary;
            }
            $inner++;
        }
        $outer--;
    }
    $sorted = ["__elephc_seed" => null];
    unset($sorted["__elephc_seed"]);
    foreach ($keys as $key) {
        $sorted[$key] = $values[$key];
    }
    $values = $sorted;
    return true;
}

function __elephc_asort_mixed(array &$values): bool {
    return __elephc_uasort_mixed($values, static fn(mixed $left, mixed $right): int => $left <=> $right);
}
"#;

/// Builds an associative result from string-or-integer keys held behind a gradual array value.
const ARRAY_FILL_KEYS_MIXED_SRC: &str = r#"<?php
function __elephc_array_fill_keys_mixed(mixed $keys, mixed $value) {
    $result = ["__elephc_seed" => null];
    unset($result["__elephc_seed"]);
    foreach ($keys as $key) {
        $result[$key] = $value;
    }
    return $result;
}
"#;

/// Parses one CSV record while preserving separators and doubled enclosures inside quoted fields.
const STR_GETCSV_SRC: &str = r#"<?php
function __elephc_str_getcsv(string $string, string $separator = ',', string $enclosure = '"', string $escape = "\\"): array {
    if ($string === '') {
        return [null];
    }
    $result = [];
    $field = '';
    $length = strlen($string);
    $index = 0;
    $quoted = false;
    while ($index < $length) {
        $character = $string[$index];
        if ($quoted) {
            if ($escape !== '' && $character === $escape && $index + 1 < $length) {
                $field .= $character;
                $index++;
                $field .= $string[$index];
            } elseif ($character === $enclosure) {
                if ($index + 1 < $length && $string[$index + 1] === $enclosure) {
                    $field .= $enclosure;
                    $index++;
                } else {
                    $quoted = false;
                }
            } else {
                $field .= $character;
            }
        } elseif ($character === $separator) {
            $result[] = $field;
            $field = '';
        } elseif ($character === $enclosure && $field === '') {
            $quoted = true;
        } else {
            $field .= $character;
        }
        $index++;
    }
    $result[] = $field;
    return $result;
}
"#;

/// PHP-visible wrapper for the supported `levenshtein()` call shape.
const LEVENSHTEIN_WRAPPER_SRC: &str = r#"<?php
function levenshtein(string $first, string $second): int { return __elephc_levenshtein_two_arg($first, $second); }
"#;

/// PHP-visible wrapper for the supported `strip_tags()` call shape.
const STRIP_TAGS_WRAPPER_SRC: &str = r#"<?php
function strip_tags(string $string): string { return __elephc_strip_tags_one_arg($string); }
"#;

/// PHP-visible wrapper for `is_countable()`.
const IS_COUNTABLE_WRAPPER_SRC: &str = r#"<?php
function is_countable(mixed $value): bool { return __elephc_is_countable($value); }
"#;

/// PHP-visible wrapper for `random_bytes()`.
const RANDOM_BYTES_WRAPPER_SRC: &str = r#"<?php
function random_bytes(int $length): string { return __elephc_random_bytes($length); }
"#;

/// PHP-visible wrapper for `http_build_query()`.
const HTTP_BUILD_QUERY_WRAPPER_SRC: &str = r#"<?php
function http_build_query(array $data, string $numericPrefix = '', mixed $argSeparator = null, int $encodingType = 1): string {
    return __elephc_http_build_query($data, $numericPrefix, $argSeparator, $encodingType);
}
"#;

/// PHP-visible wrapper for `escapeshellarg()`.
const ESCAPESHELLARG_WRAPPER_SRC: &str = r#"<?php
function escapeshellarg(string $argument): string { return __elephc_escapeshellarg($argument); }
"#;

/// PHP-visible wrappers for the process-title aliases.
const CLI_SET_PROCESS_TITLE_WRAPPER_SRC: &str = r#"<?php
function cli_set_process_title(string $title): bool { return __elephc_cli_set_process_title($title); }
function setproctitle(string $title): bool { return __elephc_cli_set_process_title($title); }
"#;

/// PHP-visible wrapper for `str_getcsv()`.
const STR_GETCSV_WRAPPER_SRC: &str = r#"<?php
function str_getcsv(string $string, string $separator = ',', string $enclosure = '"', string $escape = "\\"): array {
    return __elephc_str_getcsv($string, $separator, $enclosure, $escape);
}
"#;

/// Prepends each narrow helper whose corresponding PHP builtin is referenced.
/// Reachability group every backend-gap helper is recorded under.
pub const BACKEND_GAP_GROUP: &str = "backend_gap";

pub fn inject_if_used(
    program: Program,
    inventory: &mut crate::optimize::reachability::PreludeInventory,
) -> Program {
    let usage = crate::ast_usage::collect(&program);
    let mut sources = Vec::new();
    let inject_randomizer = usage.constructs(RANDOMIZER_CLASS_NAME)
        && !program.iter().any(|stmt| {
            matches!(
                &stmt.kind,
                crate::parser::ast::StmtKind::ClassDecl { name, .. }
                    if php_class_name_is(name, RANDOMIZER_CLASS_NAME)
            )
        });
    if usage.references("levenshtein") {
        sources.push(LEVENSHTEIN_TWO_ARG_SRC);
        sources.push(LEVENSHTEIN_WRAPPER_SRC);
    }
    if usage.references("strip_tags") {
        sources.push(STRIP_TAGS_ONE_ARG_SRC);
        sources.push(STRIP_TAGS_WRAPPER_SRC);
    }
    if usage.references("is_countable") {
        sources.push(IS_COUNTABLE_SRC);
        sources.push(IS_COUNTABLE_WRAPPER_SRC);
    }
    if usage.references("array_slice") {
        sources.push(ARRAY_SLICE_SRC);
    }
    if usage.references("array_splice") {
        sources.push(ARRAY_SPLICE_MIXED_REPLACEMENT_SRC);
    }
    if usage.references("array_unshift") {
        sources.push(ARRAY_UNSHIFT_TRAILING_SPREAD_SRC);
    }
    if usage.references("preg_replace") {
        sources.push(PREG_REPLACE_ARRAY_SRC);
    }
    if usage.references("pack") {
        sources.push(PACK_INTEGER_SRC);
    }
    if usage.references("random_bytes") {
        sources.push(RANDOM_BYTES_SRC);
        sources.push(RANDOM_BYTES_WRAPPER_SRC);
    }
    if usage.references("sort") || usage.references("rsort") {
        sources.push(SORT_MIXED_SRC);
    }
    if usage.references("array_unique") {
        sources.push(ARRAY_UNIQUE_ASSOC_MIXED_SRC);
    }
    if usage.references("array_diff") {
        sources.push(ARRAY_DIFF_STRING_SRC);
    }
    if usage.references("array_intersect") {
        sources.push(ARRAY_INTERSECT_SRC);
    }
    if usage.references("array_reverse") {
        sources.push(ARRAY_REVERSE_GRADUAL_SRC);
    }
    if usage.references("array_combine") {
        sources.push(ARRAY_COMBINE_GRADUAL_SRC);
    }
    if usage.references("array_search") {
        sources.push(ARRAY_SEARCH_MIXED_SRC);
    }
    if usage.references("http_build_query") {
        sources.push(HTTP_BUILD_QUERY_SRC);
        sources.push(HTTP_BUILD_QUERY_WRAPPER_SRC);
    }
    if usage.references("escapeshellarg") {
        sources.push(ESCAPESHELLARG_SRC);
        sources.push(ESCAPESHELLARG_WRAPPER_SRC);
    }
    if usage.references("cli_set_process_title") || usage.references("setproctitle") {
        sources.push(CLI_SET_PROCESS_TITLE_SRC);
        sources.push(CLI_SET_PROCESS_TITLE_WRAPPER_SRC);
    }
    if usage.references("file_put_contents") {
        sources.push(FILE_PUT_CONTENTS_ARRAY_SRC);
    }
    if usage.references("uasort") || usage.references("usort") || usage.references("uksort") || usage.references("asort") {
        sources.push(CALLBACK_SORT_MIXED_SRC);
    }
    if usage.references("array_fill_keys") {
        sources.push(ARRAY_FILL_KEYS_MIXED_SRC);
    }
    if usage.references("str_getcsv") {
        sources.push(STR_GETCSV_SRC);
        sources.push(STR_GETCSV_WRAPPER_SRC);
    }
    if sources.is_empty() && !inject_randomizer {
        return program;
    }
    let mut combined = Vec::new();
    if inject_randomizer {
        let tokens = crate::lexer::tokenize(RANDOMIZER_SRC)
            .expect("Randomizer compatibility prelude must tokenize");
        let parsed = crate::parser::parse(&tokens)
            .expect("Randomizer compatibility prelude must parse");
        combined.extend(
            crate::name_resolver::resolve(parsed)
                .expect("Randomizer compatibility prelude must name-resolve"),
        );
    }
    // Every helper here is named ONLY by the EIR lowering (a Rust constant), never by any PHP
    // source. `prune_unreachable_declarations` walks the PHP, finds no reference, and erases the
    // BODY while the call survives — the failure then surfaces a pass later as
    // `Call to undefined function __elephc_asort_mixed()`. Recording the group is the proof
    // that the injection already happened, which is what keeps the declarations alive.
    for source in sources {
        let tokens = crate::lexer::tokenize(source).expect("backend gap prelude must tokenize");
        let parsed = crate::parser::parse(&tokens).expect("backend gap prelude must parse");
        inventory.record_program(BACKEND_GAP_GROUP, &parsed);
        combined.extend(parsed);
    }
    combined.extend(program);
    combined
}

/// Compares canonical PHP class names case-insensitively and without a leading separator.
fn php_class_name_is(candidate: &str, expected: &str) -> bool {
    crate::names::php_symbol_key(candidate.trim_start_matches('\\'))
        == crate::names::php_symbol_key(expected.trim_start_matches('\\'))
}
