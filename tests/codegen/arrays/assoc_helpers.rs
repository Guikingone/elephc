//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of array associative-array helper builtins, including array key exists, in array string, and in array integer.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use crate::support::*;

// --- Associative array function tests ---

/// Verifies array_key_exists() returns true for present keys and false for absent ones.
/// Fixture: two-element string-keyed assoc array, two lookups (one present, one absent).
#[test]
fn test_assoc_array_key_exists() {
    let out = compile_and_run(
        r#"<?php
$m = ["name" => "Alice", "age" => "30"];
if (array_key_exists("name", $m)) { echo "yes"; }
if (array_key_exists("missing", $m)) { echo "bad"; } else { echo "no"; }
"#,
    );
    assert_eq!(out, "yesno");
}

/// Verifies in_array() with string needle finds string values in assoc array.
/// Fixture: two-element string-keyed assoc array, string needle "apple" present, "cherry" absent.
#[test]
fn test_assoc_in_array_str() {
    let out = compile_and_run(
        r#"<?php
$m = ["a" => "apple", "b" => "banana"];
if (in_array("apple", $m)) { echo "yes"; }
if (in_array("cherry", $m)) { echo "bad"; } else { echo "no"; }
"#,
    );
    assert_eq!(out, "yesno");
}

/// Verifies in_array() with integer needle finds integer values in assoc array.
/// Fixture: two-element string-keyed assoc array, integer needle 10 present, 99 absent.
#[test]
fn test_assoc_in_array_int() {
    let out = compile_and_run(
        r#"<?php
$m = ["x" => 10, "y" => 20];
if (in_array(10, $m)) { echo "yes"; }
if (in_array(99, $m)) { echo "bad"; } else { echo "no"; }
"#,
    );
    assert_eq!(out, "yesno");
}

/// Verifies array_search() returns the key for a found string value.
/// Fixture: two-element string-keyed assoc array, searches for "Bob".
#[test]
fn test_assoc_array_search_str() {
    let out = compile_and_run(
        r#"<?php
$m = ["first" => "Alice", "second" => "Bob"];
$key = array_search("Bob", $m);
echo $key;
"#,
    );
    assert_eq!(out, "second");
}

/// Verifies array_search() returns integer keys as integers and string keys as strings.
/// Fixture: assoc array with int key `10` and string key `"02"`, each holding a distinct value.
#[test]
fn test_assoc_array_search_returns_integer_and_string_keys() {
    let out = compile_and_run(
        r#"<?php
$m = [10 => "Alice", "02" => "Bob"];
echo array_search("Alice", $m);
echo "|";
echo array_search("Bob", $m);
"#,
    );
    assert_eq!(out, "10|02");
}

/// Verifies array_search() with an integer key fits the int|bool return type annotation.
/// Fixture: int-keyed assoc array with declared return type `int|bool` on the wrapper function.
#[test]
fn test_assoc_array_search_integer_key_matches_declared_union_return() {
    let out = compile_and_run(
        r#"<?php
function find_key(): int|bool {
    $m = [10 => "Alice", 20 => "Bob"];
    return array_search("Alice", $m);
}

echo find_key();
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies array_search() returns strictly false (not 0 or empty string) when value is absent.
/// Fixture: two-element string-keyed assoc array, searches for "Carol" which is not present.
#[test]
fn test_assoc_array_search_not_found_is_strict_false() {
    let out = compile_and_run(
        r#"<?php
$m = ["first" => "Alice", "second" => "Bob"];
echo array_search("Carol", $m) === false ? "miss" : "hit";
"#,
    );
    assert_eq!(out, "miss");
}

/// Verifies array_keys() returns all keys of an assoc array in insertion order.
/// Fixture: two-element string-keyed assoc array, iterates and echoes keys with spaces.
#[test]
fn test_assoc_array_keys() {
    let out = compile_and_run(
        r#"<?php
$m = ["x" => 1, "y" => 2];
$keys = array_keys($m);
$n = count($keys);
for ($i = 0; $i < $n; $i++) {
    echo $keys[$i] . " ";
}
"#,
    );
    assert_eq!(out, "x y ");
}

/// Verifies array_keys() preserves integer key `1` and string key `"02"` as distinct types.
/// Fixture: assoc array with mixed int/string keys; echoes both keys separated by `|`.
#[test]
fn test_assoc_array_keys_preserves_integer_and_string_keys() {
    let out = compile_and_run(
        r#"<?php
$m = [1 => "one", "02" => "two"];
$keys = array_keys($m);
echo $keys[0] . "|" . $keys[1];
"#,
    );
    assert_eq!(out, "1|02");
}

/// Verifies keys accumulated into an initially empty associative array retain their runtime key kind.
#[test]
fn test_assoc_array_keys_after_empty_union_accumulation() {
    let out = compile_and_run(
        r#"<?php
function accumulatedKeys(array $groups): array {
    $known = [];
    foreach ($groups as $group) {
        $known += $group;
    }
    return array_keys($known);
}
echo implode(',', accumulatedKeys([['dev' => true], ['test' => true]]));
"#,
    );
    assert_eq!(out, "dev,test");
}

/// Verifies keys nested directly in a heterogeneous return array use a concrete Mixed layout
/// even when the surrounding aggregate initially infers an uninhabited element type.
#[test]
fn test_assoc_array_keys_nested_result_normalizes_void_element_type() {
    let out = compile_and_run(
        r#"<?php
final class EnvironmentMap {
    private function allowed(): array {
        return [];
    }

    public function parameters(): array {
        if (!$known = array_flip($this->allowed())) {
            $known += ['prod' => true];
        }

        return ['known' => array_keys($known)];
    }
}

echo implode(',', (new EnvironmentMap())->parameters()['known']);
"#,
    );
    assert_eq!(out, "prod");
}

/// Verifies a declared `array` property with associative runtime storage materializes mixed keys
/// without forcing them into the integer-only result layout used by statically indexed arrays.
#[test]
fn test_array_keys_declared_array_property_preserves_runtime_key_types() {
    let out = compile_and_run(
        r#"<?php
class KeyHolder {
    private array $values;

    public function __construct() {
        $this->values = ['name' => 1, 7 => 2];
    }

    public function keys(): array {
        return array_keys($this->values);
    }

    public function describe(): void {
        foreach ($this->values as $key => $value) {
            echo gettype($key), ':', $key, ';';
        }
    }
}
$holder = new KeyHolder();
$holder->describe();
$keys = $holder->keys();
$direct = array_keys(['name' => 1, 7 => 2]);
echo gettype($direct[0]), ':', $direct[0], '|', gettype($direct[1]), ':', $direct[1], ';';
echo gettype($keys[0]), ':', $keys[0], '|', gettype($keys[1]), ':', $keys[1];
"#,
    );
    assert_eq!(
        out,
        "string:name;integer:7;string:name|integer:7;string:name|integer:7"
    );
}

/// Verifies array_search() returns the first-matching key in insertion order, not the last.
/// Fixture: three-element assoc array where "same" maps to two keys; confirms only first is returned and array size is unchanged.
#[test]
fn test_assoc_array_search_returns_first_inserted_matching_key() {
    let out = compile_and_run(
        r#"<?php
$m = ["first" => "same", "second" => "same", "third" => "other"];
$key = array_search("same", $m);
echo $key;
echo "|";
echo count($m);
"#,
    );
    assert_eq!(out, "first|3");
}

/// Verifies array_values() returns all string values of a string-keyed assoc array.
/// Fixture: two-element string-keyed assoc array, iterates and echoes values with spaces.
#[test]
fn test_assoc_array_values_str() {
    let out = compile_and_run(
        r#"<?php
$m = ["a" => "one", "b" => "two"];
$vals = array_values($m);
$n = count($vals);
for ($i = 0; $i < $n; $i++) {
    echo $vals[$i] . " ";
}
"#,
    );
    assert_eq!(out, "one two ");
}

/// Verifies array_values() returns integer values and they can be used in arithmetic.
/// Fixture: three-element string-keyed assoc array with integer values; sums them to confirm int type.
#[test]
fn test_assoc_array_values_int() {
    let out = compile_and_run(
        r#"<?php
$m = ["a" => 10, "b" => 20, "c" => 30];
$vals = array_values($m);
echo $vals[0] + $vals[1] + $vals[2];
"#,
    );
    assert_eq!(out, "60");
}

/// Verifies foreach over a mixed-type assoc array yields correct key/value pairs.
/// Fixture: three-element assoc array with mixed int and string values, echoes key=value; pairs.
#[test]
fn test_assoc_array_mixed_foreach() {
    let out = compile_and_run(
        r#"<?php
$m = ["id" => 7, "name" => "Alice", "score" => 12];
foreach ($m as $key => $value) {
    echo $key;
    echo "=";
    echo $value;
    echo ";";
}
"#,
    );
    assert_eq!(out, "id=7;name=Alice;score=12;");
}

/// Verifies array_values() on a mixed-type assoc array returns values in insertion order.
/// Fixture: three-element assoc array with mixed int/string values, echoes val,val,... format.
#[test]
fn test_assoc_array_values_mixed() {
    let out = compile_and_run(
        r#"<?php
$m = ["id" => 7, "name" => "Alice", "score" => 12];
$vals = array_values($m);
$n = count($vals);
for ($i = 0; $i < $n; $i++) {
    echo $vals[$i];
    echo ",";
}
"#,
    );
    assert_eq!(out, "7,Alice,12,");
}

/// Verifies in_array() finds both string and integer values in a mixed-type assoc array.
/// Fixture: three-element assoc array with mixed types, three lookups: string present, int present, string absent.
#[test]
fn test_assoc_in_array_mixed() {
    let out = compile_and_run(
        r#"<?php
$m = ["id" => 7, "name" => "Alice", "score" => 12];
if (in_array("Alice", $m)) { echo "name"; }
if (in_array(12, $m)) { echo " score"; }
if (!in_array("Bob", $m)) { echo " missing"; }
"#,
    );
    assert_eq!(out, "name score missing");
}

/// Verifies array_search() on a mixed-type assoc array returns correct key for string and int values.
/// Fixture: three-element assoc array with mixed types, searches for "Alice" (string value) and 12 (int value).
#[test]
fn test_assoc_array_search_mixed() {
    let out = compile_and_run(
        r#"<?php
$m = ["id" => 7, "name" => "Alice", "score" => 12];
echo array_search("Alice", $m);
echo ":";
echo array_search(12, $m);
"#,
    );
    assert_eq!(out, "name:score");
}

/// Verifies gradual haystacks return indexed or associative keys, preserve false misses, and
/// honor the runtime strictness flag through the shared dynamic-container scan.
#[test]
fn test_array_search_gradual_container_keys_and_strictness() {
    let out = compile_and_run(
        r#"<?php
function searchGradual(mixed $needle, mixed $haystack, bool $strict = false): mixed {
    return array_search($needle, $haystack, $strict);
}

echo searchGradual('target', ['other', 'target']);
echo ':';
echo searchGradual('target', ['name' => 'target']);
echo ':';
echo gettype(searchGradual('missing', ['target']));
echo ':';
echo searchGradual(1, ['1']);
echo ':';
echo gettype(searchGradual(1, ['1'], true));
"#,
    );
    assert_eq!(out, "1:name:boolean:0:boolean");
}

/// Verifies direct array access via string key on a mixed-type assoc array.
/// Fixture: three-element assoc array with mixed types, accesses $m["name"] and echoes result.
#[test]
fn test_assoc_array_access_mixed_echo() {
    let out = compile_and_run(
        r#"<?php
$m = ["id" => 7, "name" => "Alice", "score" => 12];
echo $m["name"];
"#,
    );
    assert_eq!(out, "Alice");
}

/// Verifies array_values() produces a borrowed reference that survives unset of the source array.
/// Regression: array_values must not copy-or-free source data while borrowed; $vals[0] must remain valid after unset($map).
#[test]
fn test_gc_assoc_array_values_borrowed_array_survives_source_unset() {
    let out = compile_and_run(
        r#"<?php
$map = ["nums" => [7, 8, 9]];
$vals = array_values($map);
unset($map);
$saved = $vals[0];
echo $saved[1];
"#,
    );
    assert_eq!(out, "8");
}

/// Verifies `array_fill_keys()` iterates a gradual array operand.
#[test]
fn test_array_fill_keys_gradual_operand() {
    let output = compile_and_run(
        r#"<?php
function fill(mixed $keys): array { return array_fill_keys($keys, 7); }
$result = fill(["a", "b"]);
echo $result["a"].$result["b"];
"#,
    );
    assert_eq!(output, "77");
}

// --- Associative array function tests ---

/// Verifies conditional literal-string hash keys retain the string result layout consumed by
/// `str_replace()`, even when the source hash key metadata was widened while merging branches.
#[test]
fn test_assoc_array_keys_gradual_keys_into_string_consumer() {
    let out = compile_and_run(
        r#"<?php
function replaceKey(?string $scriptNonce, ?string $styleNonce): string {
    $replacements = [];
    if (null !== $scriptNonce) {
        $replacements["<script>"] = $scriptNonce;
    }
    if (null !== $styleNonce) {
        $replacements["<style>"] = $styleNonce;
    }

    $keys = array_keys($replacements);
    return $keys[0]."|".$keys[1];
}

echo replaceKey("script", "style");
"#,
    );
    assert_eq!(out, "<script>|<style>");
}

/// Verifies a boxed Mixed needle scans Mixed-valued associative storage with PHP loose and strict
/// comparison semantics, including concrete hash entries that require temporary boxing.
#[test]
fn test_assoc_in_array_mixed_needle_loose_and_strict() {
    let out = compile_and_run(
        r#"<?php
function containsMixed(mixed $needle, array $haystack, bool $strict): bool {
    return in_array($needle, $haystack, $strict);
}

$values = ["int" => 12, "string" => "12", "bool" => true];
echo containsMixed(12, $values, true) ? "1" : "0";
echo containsMixed("12", $values, true) ? "1" : "0";
echo containsMixed(false, ["value" => 0], false) ? "1" : "0";
echo containsMixed(false, ["value" => 0], true) ? "1" : "0";
"#,
    );
    assert_eq!(out, "1110");
}
