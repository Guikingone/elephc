//! Purpose:
//! Regression tests for defects found when the Laravel `--web` line was merged with this branch:
//! each one only appeared once both sides' changes met.

use super::*;

/// A hash property returned under a bare `array` return type is handed back with its own
/// reference. The return used to fall to the generic runtime conversion, which gave the caller the
/// property's storage without one: the caller's release freed the property, and the next call read
/// freed memory (Symfony's `ResponseHeaderBag::all()` in Laravel's `sendHeaders()`).
#[test]
fn test_hash_property_returned_as_array_keeps_its_own_reference() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Bag {
    protected array $headers = [];
    public function set(string $key, string $value): void {
        $this->headers[strtolower($key)] = [$value];
    }
    public function all(?string $key = null): array {
        if (null !== $key) {
            return $this->headers[$key] ?? [];
        }
        return $this->headers;
    }
}
class RBag extends Bag {
    protected array $cookies = [];
    public function all(?string $key = null): array {
        $headers = parent::all();
        foreach ($this->cookies as $cookie) {
            $headers['set-cookie'][] = (string) $cookie;
        }
        return $headers;
    }
}
function run(): void {
    $h = new RBag();
    $h->set('Content-Type', 'application/json');
    for ($i = 0; $i < 3; $i++) {
        echo json_encode($h->all()), "\n";
    }
}
run();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "{\"content-type\":[\"application\\/json\"]}\n".repeat(3));
}

/// A callback that declares fewer parameters than `array_map()` passes ignores the surplus, as in
/// php. The literal `['K', 's']` became a direct static binding, whose call carried the element as
/// an operand the method has no slot for, and the build failed.
#[test]
fn test_array_map_with_a_static_callback_taking_fewer_parameters() {
    let out = compile_and_run(
        r#"<?php
class K { static function s() { return "x" . mt_rand(1, 1); } }
echo implode(",", array_map(['K', 's'], [1, 2])), "\n";
echo implode(",", array_map('K::s', [3])), "\n";
"#,
    );
    assert_eq!(out, "x1,x1\nx1\n");
}

/// An unqualified constant nothing defines is reported under the name the source wrote: php's
/// `Error` names `App\BARE_UNKNOWN` even though the lookup falls back to the global constant.
#[test]
fn test_undefined_namespaced_constant_error_names_the_written_form() {
    let out = compile_and_run(
        r#"<?php
namespace App;
try { echo BARE_UNKNOWN; } catch (\Error $e) { echo $e->getMessage(), "\n"; }
try { echo \FOURTH_UNKNOWN; } catch (\Error $e) { echo $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(out, "Undefined constant \"App\\BARE_UNKNOWN\"\nUndefined constant \"FOURTH_UNKNOWN\"\n");
}

/// Runtime `constant()` misses caught twice at the top level keep their `Error` intact. With the
/// eval interpreter linked into every program (an `isset($mixed[$k])` in the session prelude
/// asked `offsetExists()` through it), the second catch tripped the heap validator.
#[test]
fn test_runtime_constant_misses_are_caught_twice_at_top_level() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$n = $argc > 5 ? "x" : "Y";
try { echo constant("FOURTH" . $n); } catch (\Error $e) { echo $e->getMessage(), "\n"; }
try { echo constant("FIFTH" . $n); } catch (\Error $e) { echo $e->getMessage(), "\n"; }
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "Undefined constant \"FOURTHY\"\nUndefined constant \"FIFTHY\"\n");
}
