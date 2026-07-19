//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of runtime GC regressions, including regression assoc value in function, regression iterate assoc in function, and regression arr equals func arr.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use crate::support::*;

/// Regression test: associative array values accessed inside a function after the
/// array is passed as an argument. Verifies that reading multiple keys (`done`,
/// `title`, `priority`) from a passed assoc array produces correct output.
#[test]
fn test_regression_assoc_value_in_function() {
    let out = compile_and_run(
        r#"<?php
function show($todo) {
    $status = $todo["done"] === "1" ? "[x]" : "[ ]";
    $pri = $todo["priority"];
    echo $status . " " . $todo["title"] . " " . $pri;
}
$t = ["title" => "Buy milk", "done" => "0", "priority" => "high", "created" => "now"];
show($t);
"#,
    );
    assert_eq!(out, "[ ] Buy milk high");
}

/// Regression test: iterating a numerically-indexed array of assoc arrays inside a
/// function. Verifies that indexed access into a passed array of assoc arrays
/// (`$items[$i]["name"]`, `$items[$i]["value"]`) works correctly.
#[test]
fn test_regression_iterate_assoc_in_function() {
    let out = compile_and_run(
        r#"<?php
function format($items) {
    $result = "";
    for ($i = 0; $i < count($items); $i++) {
        $item = $items[$i];
        $result .= $item["name"] . ":" . $item["value"] . "\n";
    }
    return $result;
}
$data = [["name" => "a", "value" => "1"], ["name" => "b", "value" => "2"]];
echo format($data);
"#,
    );
    assert_eq!(out, "a:1\nb:2\n");
}

/// Regression test: appending to a function parameter array and returning it.
/// Verifies that `$arr[] = $val` and returning the modified array works across
/// multiple calls and that the result is correctly assigned back to the caller's
/// variable.
#[test]
fn test_regression_arr_equals_func_arr() {
    let out = compile_and_run(
        r#"<?php
function add($arr, $val) {
    $arr[] = $val;
    return $arr;
}
$nums = [1];
$nums = add($nums, 2);
$nums = add($nums, 3);
echo count($nums) . "|" . $nums[0] . "|" . $nums[2];
"#,
    );
    assert_eq!(out, "3|1|3");
}

/// Verifies that nested integerish arithmetic inside a function releases Mixed
/// temporaries cleanly. Runs 1000 iterations and asserts the heap is clean
/// (allocations == deallocations) with no leaks.
#[test]
fn test_nested_integerish_arithmetic_releases_mixed_temporaries() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
function ready(int $slot): bool {
    $offset = ($slot + 1) * 8 + 6;
    return $offset != 0;
}

for ($i = 0; $i < 1000; $i++) {
    if (ready(0)) {
        $seen = 1;
    }
}
echo "done";
"#,
    );
    assert_eq!(out.stdout, "done");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Verifies PHP bytewise string operator results are owned/released cleanly over many
/// iterations. Each `&`/`|`/`^` result feeds `bin2hex` (which allocates an owned heap
/// string) and is reassigned to `$out` every loop, so the previous owned value must be
/// released without a double-free. Exercises the `$a & $a` self-alias release path and
/// all three operators; 300 iterations must leave the heap clean (allocations ==
/// deallocations, no corruption). `bin2hex("ABCD" ^ "\xF0\x0F\xF0\x0F")` is `b14db34b`.
#[test]
fn test_string_bitwise_result_released_cleanly() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
$a = "AB";
$x = "ABCD";
$y = "\xF0\x0F\xF0\x0F";
$out = "";
for ($i = 0; $i < 300; $i++) {
    $out = bin2hex($a & $a);
    $out = bin2hex($x & $y);
    $out = bin2hex($x | $y);
    $out = bin2hex($x ^ $y);
}
echo $out;
"#,
    );
    assert_eq!(out.stdout, "b14db34b");
    assert!(
        !out.stderr.contains("double free") && !out.stderr.contains("bad refcount"),
        "heap corruption detected: {}",
        out.stderr
    );
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Regression: a temporary object implicitly stringified via `__toString` in `echo` must
/// be released, not leaked. 100 iterations would accumulate 100 leaked objects otherwise.
#[test]
fn test_tostring_temp_object_released_on_echo() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class Greeter { public function __toString(): string { return "hi"; } }
for ($i = 0; $i < 100; $i++) { echo new Greeter(); }
echo "\n";
"#,
    );
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Regression: a temporary object stringified via `__toString` in a concatenation must be
/// released.
#[test]
fn test_tostring_temp_object_released_on_concat() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class Greeter { public function __toString(): string { return "hi"; } }
for ($i = 0; $i < 100; $i++) { $s = "x" . new Greeter(); }
echo "done";
"#,
    );
    assert_eq!(out.stdout, "done");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Regression: a temporary object cast to string via `(string)` must be released.
#[test]
fn test_tostring_temp_object_released_on_string_cast() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class Greeter { public function __toString(): string { return "hi"; } }
for ($i = 0; $i < 100; $i++) { $s = (string) new Greeter(); }
echo "done";
"#,
    );
    assert_eq!(out.stdout, "done");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Guard: a variable-held object stringified via `__toString` is owned by the variable
/// slot and must NOT be released by the coercion (otherwise it would be double-freed).
#[test]
fn test_tostring_variable_object_not_double_freed() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class Greeter { public function __toString(): string { return "hi"; } }
$g = new Greeter();
for ($i = 0; $i < 100; $i++) { echo $g; }
echo "\n";
"#,
    );
    assert!(out.success, "program crashed (double free?): {}", out.stderr);
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Regression test: creating assoc arrays via a factory function, pushing them
/// into a numerically-indexed array, then iterating and accessing keys. Verifies
/// that `make()` return values survive being stored and retrieved from a outer
/// array.
#[test]
fn test_regression_make_assoc_then_iterate() {
    let out = compile_and_run(
        r#"<?php
function make($name, $val) { return ["name" => $name, "val" => $val]; }
$items = [];
$items[] = make("x", "1");
$items[] = make("y", "2");
$items[] = make("z", "3");
for ($i = 0; $i < count($items); $i++) {
    $it = $items[$i];
    echo $it["name"] . "=" . $it["val"] . " ";
}
"#,
    );
    assert_eq!(out, "x=1 y=2 z=3 ");
}

/// Regression test: concatenating multiple string accesses from assoc array
/// elements inside a loop and returning the result. Verifies that repeated
/// `$content .= $items[$i]["a"] . "|" . ...` chains are handled correctly.
#[test]
fn test_regression_save_concat_chain() {
    let out = compile_and_run(
        r#"<?php
function save($items) {
    $content = "";
    for ($i = 0; $i < count($items); $i++) {
        $c = $items[$i];
        $content .= $c["a"] . "|" . $c["b"] . "|" . $c["c"] . "\n";
    }
    return $content;
}
$data = [["a" => "x", "b" => "y", "c" => "z"]];
echo save($data);
"#,
    );
    assert_eq!(out, "x|y|z\n");
}

/// Regression test: passing an object to a function and accessing its properties
/// (`$dog->name`, `$dog->breed`). Verifies that object properties are correctly
/// accessible inside function scope.
#[test]
fn test_regression_object_string_property_in_function() {
    let out = compile_and_run(
        r#"<?php
class Dog {
    public $name;
    public $breed;
    public function __construct($n, $b) { $this->name = $n; $this->breed = $b; }
}
function describe($dog) {
    return $dog->name . " (" . $dog->breed . ")";
}
$d = new Dog("Rex", "Labrador");
echo describe($d);
"#,
    );
    assert_eq!(out, "Rex (Labrador)");
}

/// Regression test: objects stored in an array are retrieved and their methods
/// called. Verifies that `$items[$i]->format()` works correctly after objects are
/// stored and fetched from a numerically-indexed array.
#[test]
fn test_regression_objects_in_array_with_methods() {
    let out = compile_and_run(
        r#"<?php
class Item {
    public $name;
    public $price;
    public function __construct($n, $p) { $this->name = $n; $this->price = $p; }
    public function format() { return $this->name . ": $" . $this->price; }
}
$items = [new Item("Apple", 1), new Item("Banana", 2)];
for ($i = 0; $i < count($items); $i++) {
    echo $items[$i]->format() . "\n";
}
"#,
    );
    assert_eq!(out, "Apple: $1\nBanana: $2\n");
}

/// Regression test: early `return` inside a `switch` within a loop. Verifies that
/// `label()` returning from within `switch` cases in a loop does not corrupt
/// control flow or stack state.
#[test]
fn test_regression_switch_return_in_loop() {
    let out = compile_and_run(
        r#"<?php
function label($n) {
    switch ($n % 3) {
        case 0: return "A";
        case 1: return "B";
        default: return "C";
    }
}
$r = "";
for ($i = 0; $i < 6; $i++) {
    $r .= label($i);
}
echo $r;
"#,
    );
    assert_eq!(out, "ABCABC");
}

/// Regression test: chained string operations (`strtolower`, `str_replace`) on a
/// function parameter. Verifies that successive string builtins that modify a
/// local variable work correctly and the result is returned.
#[test]
fn test_regression_string_ops_in_function() {
    let out = compile_and_run(
        r#"<?php
function clean($s) {
    $s = strtolower($s);
    $s = str_replace(" ", "_", $s);
    return $s;
}
echo clean("Hello World");
"#,
    );
    assert_eq!(out, "hello_world");
}

/// Regression test: `explode` result used as an array inside a function, then
/// indexed. Verifies that `$parts[0]` and `$parts[1]` access the correct exploded
/// segments after `explode` is called on a comma-separated string.
#[test]
fn test_regression_explode_in_function_use_parts() {
    let out = compile_and_run(
        r#"<?php
function parse($csv) {
    $parts = explode(",", $csv);
    return $parts[0] . "+" . $parts[1];
}
echo parse("foo,bar");
"#,
    );
    assert_eq!(out, "foo+bar");
}

/// Regression test: function returns an assoc array and the caller reads multiple
/// keys from it. Verifies that `config()["host"]`, `config()["port"]`, etc.
/// access the correct returned values after a single call.
#[test]
fn test_regression_return_assoc_read_keys() {
    let out = compile_and_run(
        r#"<?php
function config() {
    return ["host" => "localhost", "port" => "3306", "db" => "myapp"];
}
$c = config();
echo $c["host"] . ":" . $c["port"] . "/" . $c["db"];
"#,
    );
    assert_eq!(out, "localhost:3306/myapp");
}

/// Regression test: reading multiple distinct keys (`first`, `second`, `third`)
/// from a single assoc array parameter. Verifies correct access to each key
/// without interference between the reads.
#[test]
fn test_regression_multiple_hash_get_locals() {
    let out = compile_and_run(
        r#"<?php
function show($row) {
    $a = $row["first"];
    $b = $row["second"];
    $c = $row["third"];
    echo $a . "|" . $b . "|" . $c;
}
show(["first" => "x", "second" => "y", "third" => "z"]);
"#,
    );
    assert_eq!(out, "x|y|z");
}

/// Regression test: method receives a string parameter and also accesses an
/// object property (`$this->prefix`). Verifies that the property is correctly
/// available inside the method and the result is returned correctly.
#[test]
fn test_regression_method_string_param_and_prop() {
    let out = compile_and_run(
        r#"<?php
class Greeter {
    public $prefix;
    public function __construct($p) { $this->prefix = $p; }
    public function greet($name) { return $this->prefix . " " . $name . "!"; }
}
$g = new Greeter("Hello");
echo $g->greet("World");
"#,
    );
    assert_eq!(out, "Hello World!");
}

/// Regression test: object property stores a string that was derived from a
/// concatenated literal (`"AB" . "CD"`). Verifies that property initialization
/// and subsequent method access (`$this->bytes`) survives constructor parameter
/// cleanup without corrupting the stored value.
#[test]
fn test_regression_string_property_survives_constructor_param_cleanup() {
    let out = compile_and_run(
        r#"<?php
class Reader {
    public $bytes;
    public function __construct(string $bytes) { $this->bytes = $bytes; }
    public function head(): string { return substr($this->bytes, 0, 4); }
}
$bytes = "AB" . "CD";
$reader = new Reader($bytes);
echo $reader->head();
"#,
    );
    assert_eq!(out, "ABCD");
}

/// Regression test: a string variable passed to a constructor is still usable
/// after the object is created. Verifies that the callee (constructor) does not
/// prematurely free the caller's string argument, leaving the original variable
/// with a valid value.
#[test]
fn test_regression_callee_does_not_free_caller_string_argument() {
    let out = compile_and_run(
        r#"<?php
class Greeter {
    public $prefix;
    public function __construct($prefix) {
        $this->prefix = $prefix;
    }
}
$prefix = "IWAD";
$greeter = new Greeter($prefix);
echo $prefix;
echo "|";
echo $greeter->prefix;
"#,
    );
    assert_eq!(out, "IWAD|IWAD");
}

/// Regression test: a large heap-backed string (1 MB file) is read, sliced via
/// `substr`, stored in an object property, and the object is returned from a
/// function. Verifies that heap-allocated string slices survive across object
/// return and are not prematurely collected or corrupted.
#[test]
fn test_regression_string_property_persists_heap_slice_across_object_return() {
    let id = TEST_ID.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("elephc_str_persist_{}.bin", id));
    let mut bytes = vec![b'X'; 1024 * 1024];
    bytes[..8].copy_from_slice(b"PLAYPAL\0");
    fs::write(&path, &bytes).unwrap();

    let source = format!(
        r#"<?php
class WadLike {{
    public $name;
    public function __construct() {{
        $this->name = "";
    }}
}}

class Maker {{
    public function make(): WadLike {{
        $bytes = file_get_contents("{path}");
        $name = substr($bytes, 0, 7);
        $wad = new WadLike();
        $wad->name = $name;
        return $wad;
    }}
}}

$maker = new Maker();
$wad = $maker->make();
echo $wad->name;
"#,
        path = path.display()
    );

    let out = compile_and_run_with_heap_size(&source, 67_108_864);
    let _ = fs::remove_file(&path);
    assert_eq!(out, "PLAYPAL");
}

/// Regression test: an object returned from a function carries a property built
/// via a loop that accumulates string characters. Verifies that the property built
/// inside the loop (`$name .= $ch`) is correctly preserved on the returned object.
#[test]
fn test_regression_returned_object_preserves_loop_built_string_property() {
    let out = compile_and_run(
        r#"<?php
class WadLike {
    public $kind;
    public $firstEntryName;
    public function __construct($kind) {
        $this->kind = $kind;
        $this->firstEntryName = "";
    }
}

class Maker {
    public function make(): WadLike {
        $bytes = "IWADxxxxPLAYPAL\0tail";
        $kind = substr($bytes, 0, 4);
        $raw = substr($bytes, 8, 8);
        $name = "";
        $i = 0;
        while ($i < strlen($raw)) {
            $ch = substr($raw, $i, 1);
            if (ord($ch) == 0) {
                break;
            }
            $name .= $ch;
            $i += 1;
        }
        $wad = new WadLike($kind);
        $wad->firstEntryName = $name;
        return $wad;
    }
}

$maker = new Maker();
$wad = $maker->make();
echo $wad->kind;
echo "|";
echo $wad->firstEntryName;
"#,
    );
    assert_eq!(out, "IWAD|PLAYPAL");
}

/// Regression test: calling a static method with parameters and string
/// concatenation. Verifies that `Fmt::wrap("hello", "b")` correctly concatenates
/// the tag and string and returns the wrapped result.
#[test]
fn test_regression_static_method_string() {
    let out = compile_and_run(
        r#"<?php
class Fmt {
    public static function wrap($s, $tag) { return "<" . $tag . ">" . $s . "</" . $tag . ">"; }
}
echo Fmt::wrap("hello", "b");
"#,
    );
    assert_eq!(out, "<b>hello</b>");
}

/// Regression test: chained property access (`$o->inner->val`) where an inner
/// object is stored in an outer object property and returned. Verifies that
/// accessing a property of a nested object works correctly.
#[test]
fn test_regression_chained_property_access() {
    let out = compile_and_run(
        r#"<?php
class Inner { public $val;
    public function __construct($v) { $this->val = $v; }
}
class Outer { public $inner;
    public function __construct($i) { $this->inner = $i; }
}
$o = new Outer(new Inner(42));
echo $o->inner->val;
"#,
    );
    assert_eq!(out, "42");
}

/// Regression test: object with a float property, used in a method that
/// performs arithmetic. Verifies that float properties are correctly stored and
/// used in method computations.
#[test]
fn test_regression_float_property() {
    let out = compile_and_run(
        r#"<?php
class Circle {
    public $radius;
    public function __construct($r) { $this->radius = $r; }
    public function area() { return 3.14 * $this->radius * $this->radius; }
}
$c = new Circle(10.0);
echo $c->area();
"#,
    );
    assert_eq!(out, "314");
}

/// Regression test: `$obj->prop[] = <scalar>` boxes the scalar into a Mixed cell
/// and `__rt_array_push_refcounted` retains a reference. The codegen was keeping
/// the cell's original (heap_alloc) reference and never releasing it, leaking one
/// reference past the array's deep-free. Asserts heap is clean at exit.
#[test]
fn test_regression_property_array_push_scalar_does_not_leak() {
    // `$obj->prop[] = <scalar>` boxes the scalar into a Mixed cell, then
    // `__rt_array_push_refcounted` retains its own reference to that cell.
    // The codegen kept the cell's original (heap_alloc) reference and never
    // released it, so the boxed Mixed cell leaked one reference and survived
    // past the array's deep-free. Regression: heap must be clean at exit.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class C { public array $a; }
$x = new C();
$x->a = [];
$x->a[] = 4;
echo count($x->a);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Verifies that repeatedly pushing onto an associative-array property across
/// several method calls appends with the next integer key and leaves the heap
/// clean. The property is declared `array` with an associative default (codegen
/// type `AssocArray`), so `$this->rows[] = $v` takes the EIR
/// `lower_property_array_push` assoc branch: it acquires a distinct owned handle
/// to the loaded hash, hash-appends (which COW-splits and may relocate the
/// table), writes the possibly-relocated pointer back via `PropSet`, then
/// releases the acquired copy. This exercises the acquire/release/`PropSet`
/// ownership balance under repeated growth; a mismatched acquire or missing
/// release would leak the hash or its inserted cells. Pushes `0, 10, 20` onto an
/// initial `['a' => 1]`, so the final count is `4` with `$rows[0] == 0` and
/// `$rows[2] == 20`; the heap must be clean at exit.
#[test]
fn test_regression_property_assoc_array_push_loop_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Rows {
    private array $rows = ['a' => 1];
    public function push($v): void { $this->rows[] = $v; }
    public function total(): int { return count($this->rows); }
    public function at(int $i): int { return $this->rows[$i]; }
}
$r = new Rows();
for ($i = 0; $i < 3; $i++) {
    $r->push($i * 10);
}
echo $r->total(), ":", $r->at(0), ",", $r->at(2);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "4:0,20");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Verifies that assigning a Mixed indexed-array cell to a local retains an
/// independent owner and does not leave the array with a dangling cell.
#[test]
fn test_mixed_indexed_array_read_survives_local_unset() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$values = [5, "x"];
$first = $values[0];
unset($first);
echo $values[0];
unset($values);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "5");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Verifies that assigning a Mixed associative-array cell to a local retains an
/// independent owner and does not leave the hash with a dangling cell.
#[test]
fn test_mixed_assoc_array_read_survives_local_unset() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$values = ["a" => 5, "b" => "x"];
$first = $values["a"];
unset($first);
echo $values["a"];
unset($values);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "5");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: pushing an owned array literal into a Mixed-element property
/// array adds a second ownership layer. The inner array is retained by the Mixed
/// box and the Mixed box is retained by `__rt_array_push_refcounted`. The property
/// push path must use the container-aware boxer and release the boxed cell after
/// the append. Asserts heap is clean at exit.
#[test]
fn test_regression_property_array_push_array_value_does_not_leak() {
    // Pushing an owned array literal into a Mixed-element property array adds
    // a second ownership layer: the inner array is retained by the Mixed box,
    // and the Mixed box is retained by `__rt_array_push_refcounted`. The
    // property push path must use the container-aware boxer (so the inner
    // array's original reference is released) and release the boxed cell
    // after the append. Regression: heap must be clean at exit.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class C { public array $a; }
$x = new C();
$x->a = [];
$x->a[] = "hello";
$x->a[] = [1, 2, 3];
echo count($x->a);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: passing an owned array literal through a `mixed` parameter
/// before storing it in a property array must release the argument expression's
/// original owner after the Mixed cell retains the payload.
#[test]
fn test_regression_mixed_arg_array_payload_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class C {
    public array $a;

    public function __construct() {
        $this->a = [];
    }

    public function add(mixed $value): void {
        $this->a[] = $value;
    }
}

$x = new C();
$x->add([1, 2, 3]);
unset($x);
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: a loop pushing scalars into a property array repeatedly
/// exercises the boxing + push path; each iteration must balance its refcount or
/// the leak compounds. Asserts heap is clean after 20 iterations.
#[test]
fn test_regression_property_array_push_in_loop_does_not_leak() {
    // A loop pushing scalars into a property array repeatedly exercises the
    // boxing + push path; each iteration must balance its refcount or the
    // leak compounds.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class C { public array $a; }
$x = new C();
$x->a = [];
for ($i = 0; $i < 20; $i++) {
    $x->a[] = $i;
}
echo count($x->a);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "20");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: overwriting a static array must release the payloads appended
/// to the old array. The scalar is boxed into Mixed and retained by
/// `__rt_array_push_refcounted`; only the replacement static array should remain
/// live at exit. Asserts exactly one live block (the current static array).
#[test]
fn test_regression_static_property_array_push_scalar_releases_old_payload() {
    // Static storage itself is process-lifetime state, but an overwritten
    // static array must release the payloads appended to the old array. The
    // scalar is boxed into Mixed and then retained by `__rt_array_push_refcounted`;
    // only the replacement static array should remain live at exit.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class C { public static array $a; }
C::$a = [];
C::$a[] = 4;
C::$a = [];
echo count(C::$a);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "0");
    assert!(
        out.stderr
            .contains("HEAP DEBUG: leak summary: live_blocks=1"),
        "expected only the current static array to remain live, got: {}",
        out.stderr
    );
}

/// Regression test: pushing an owned array literal into a Mixed-element static
/// property array needs both the container-aware boxer and the post-push release.
/// After the static property is overwritten the old array and appended literal
/// should be gone; only the replacement static array remains live. Asserts exactly
/// one live block.
#[test]
fn test_regression_static_property_array_push_array_value_releases_old_payload() {
    // Pushing an owned array literal into a Mixed-element static property array
    // needs both the container-aware boxer and the post-push release. After the
    // static property is overwritten, the old array and appended literal should
    // be gone; only the replacement static array remains live by design.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class C { public static array $a; }
C::$a = [];
C::$a[] = [1, 2, 3];
C::$a = [];
echo count(C::$a);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "0");
    assert!(
        out.stderr
            .contains("HEAP DEBUG: leak summary: live_blocks=1"),
        "expected only the current static array to remain live, got: {}",
        out.stderr
    );
}

/// Regression test: array literals with spread build their result through
/// `__rt_array_push_refcounted` for refcounted elements. The non-spread element
/// path retained the element via `retain_borrowed_heap_arg` and again inside the
/// push helper without releasing the codegen's owning reference, leaking the
/// appended element. Asserts heap is clean at exit.
#[test]
fn test_regression_spread_array_literal_does_not_leak() {
    // Array literals with spread build their result through
    // `__rt_array_push_refcounted` for refcounted elements. The non-spread
    // element path retained the element via `retain_borrowed_heap_arg` and
    // again inside the push helper without releasing the codegen's owning
    // reference, leaking the appended element. Regression: heap clean at exit.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1], [2]];
$b = [...$a, [3]];
echo count($b);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "3");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: `foreach ($items as &$value)` on a non-empty array where the
/// by-ref loop variable is reassigned after the loop. Verifies that the by-ref
/// loop does not leak the local ref cell, and that reassigning `$value = 99`
/// after the loop correctly mutates the array element.
#[test]
fn test_regression_foreach_by_ref_non_empty_does_not_leak_local_ref_cell() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$items = [1, 2, 3];
foreach ($items as &$value) {
    $value = $value + 10;
}
$value = 99;
echo $items[2];
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "99");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: `foreach ($items as &$value)` on an empty array (after
/// `array_pop` empties it) should not leak the local ref cell. The loop body is
/// never entered but the by-ref binding must still be cleaned up at function exit.
#[test]
fn test_regression_foreach_by_ref_empty_releases_local_ref_cell_at_exit() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$value = 7;
$items = [1];
array_pop($items);
foreach ($items as &$value) {
    $value = 1;
}
echo $value;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "7");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: a by-ref foreach reusing the same variable name (`$value`)
/// across two separate loops where the first array becomes empty mid-way.
/// Verifies that the prior fallback type (`string`) is released when `$value` is
/// rebound to a new by-ref cell in the second loop.
#[test]
fn test_regression_foreach_by_ref_reused_name_releases_prior_fallback_type() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function run() {
    $value = "held";
    $first = [1];
    array_pop($first);
    foreach ($first as &$value) {
        $value = 1;
    }

    $second = [10, 20];
    foreach ($second as &$value) {
        $value += 100;
    }

    $value = 999;
    echo $second[0];
    echo "|";
    echo $second[1];
    echo "|";
    echo $value;
}

run();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "110|999|999");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test for the x86_64 Mixed-property string-read aliasing bug.
///
/// Reading a string-typed property off an object retrieved from a Mixed-valued
/// hash (`$arr['a']->v`) goes through `emit_property_load`'s two-word string
/// path. On x86_64 the string-pointer result register is `rax`, which also
/// serves as the object base register in the Mixed-property dispatch, so loading
/// the pointer word first clobbered the base; the length word was then read from
/// the string payload instead of the object. That garbage length drove
/// `__rt_str_persist` to copy an enormous span and exhaust the heap. The fix
/// reads the length word first when the pointer register aliases the base. ARM64
/// was always correct (its result registers never alias the base). A plain
/// object (not an enum) reproduces it, so this guards the general lowering.
#[test]
fn test_regression_mixed_hash_object_string_property() {
    let out = compile_and_run(
        r#"<?php
class Box { public string $v = 'hi'; }
$arr = ['a' => new Box(), 'b' => 1];
echo $arr['a']->v;
"#,
    );
    assert_eq!(out, "hi");
}

/// Regression test: assigning a by-value foreach element into another local should
/// release the target slot using its widened Mixed storage representation.
#[test]
fn test_regression_foreach_mixed_value_assignment_releases_old_slot_storage() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
for ($n = 0; $n < 50; $n++) {
    foreach (["x", "y", "z"] as $p) {
        $k = $p;
    }
}
echo "x";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "x");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: refcounted hidden ternary merge temps must be released when
/// reassigned across loop iterations and during function/main epilogue cleanup.
#[test]
fn test_regression_refcounted_hidden_ternary_temp_released() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
for ($i = 0; $i < 50; $i++) {
    $a = ["a" => "b"];
    $x = isset($a["missing"]) ? $a["missing"] : "";
}
echo "x";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "x");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test: `isset($hash[$missing])` on a Mixed-valued hash should probe
/// presence/null without allocating a throwaway Mixed miss value every iteration.
#[test]
fn test_regression_mixed_hash_isset_miss_does_not_materialize_leaking_value() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
for ($i = 0; $i < 50; $i++) {
    $a = ["a" => 1, "b" => "s"];
    $x = isset($a["missing"]) ? $a["missing"] : "";
}
echo "x";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "x");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test for the array-to-string echo fix: echoing an owned temporary array
/// stringifies to "Array" and releases the temporary, keeping GC allocs and frees balanced
/// (no leak from the discarded array, no premature/double free).
#[test]
fn test_echo_owned_temp_array_balances_gc_stats() {
    let baseline = compile_and_run_with_gc_stats("<?php");
    let out = compile_and_run_with_gc_stats("<?php echo [1, 2, 3];");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "Array");
    let (baseline_allocs, baseline_frees) = parse_gc_stats(&baseline.stderr);
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert!(
        allocs - baseline_allocs >= 1,
        "expected the temporary array to allocate at least once"
    );
    assert_eq!(allocs - baseline_allocs, frees - baseline_frees);
}

/// Regression test for issue #408: releasing a string-keyed associative array
/// must free everything it owns. Reassigning a hash-typed local each iteration
/// promotes a fresh indexed array literal to hash storage via `array_to_hash`,
/// which builds the result hash from a copy of the source array; the source
/// array was leaked once per conversion. With many iterations the leak would
/// exhaust the fixed heap, so a balanced alloc/free count proves it is freed.
#[test]
fn test_regression_408_reassigned_string_keyed_array_does_not_leak() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
$g = [];
for ($n = 0; $n < 500; $n++) {
    $g = [];
    $g["a"] = "x";
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert!(allocs >= 500, "expected per-iteration allocations: {allocs}");
    assert_eq!(
        allocs, frees,
        "string-keyed array release must free its source array (issue #408)"
    );
}

/// Regression test for issue #408 (heap-debug): the same reassignment loop must
/// report a clean heap with no live blocks at exit and must not trip the
/// double-free detector, confirming the conversion releases exactly one
/// reference of the source array (correct COW ownership).
#[test]
fn test_regression_408_reassigned_string_keyed_array_heap_debug_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$g = [];
for ($n = 0; $n < 500; $n++) {
    $g = [];
    $g["a"] = "x";
    $g["bb"] = "yy";
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test for issue #408 (in-place promotion): a string-key write that
/// promotes a freshly built indexed array literal to hash storage must also free
/// the source array. Repeated promotion in a loop keeps GC allocs and frees
/// balanced with no leaked source arrays.
#[test]
fn test_regression_408_string_key_promotion_does_not_leak() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
$g = [];
for ($n = 0; $n < 500; $n++) {
    $g = [1, 2];
    $g["a"] = "x";
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(
        allocs, frees,
        "promoting an indexed array literal to hash storage must free the source array (issue #408)"
    );
}

/// Regression for the trim self-reassign fix: `$s = trim($s)` now persists an owned copy of the
/// trimmed slice instead of returning a slice into the source buffer, so reassigning a heap string
/// to a trimmed slice of itself under loop churn neither corrupts the string nor leaks/double-frees.
/// Each iteration allocates the persisted copy and frees the previous value, so allocs and frees
/// stay balanced. Mirrors symfony/yaml `Inline::parse`'s `$value = trim($value)` scalar path.
#[test]
fn test_trim_self_reassign_loop_balances_gc_stats() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
$parts = ["  ele", "phc  "];
$total = 0;
for ($k = 0; $k < 6; $k++) {
    $s = $parts[0] . $parts[1];
    $s = trim($s);
    $total = $total + strlen($s);
}
echo $total;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "36");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "trim self-reassign loop leaked or double-freed");
}

/// Plain-concrete (no interface, no narrowing) heap regression for by-ref bug #5: a class that owns
/// a reference property (one it returns by reference, `&ref()`) allocates a 16-byte ref-cell per
/// instance at construction, but `__rt_object_free_deep` never releases that cell or the array it
/// holds — the per-class GC descriptor tags owned reference properties `0` (no cleanup). The leak is
/// purely instance-proportional: this loop only constructs objects (no method call, no `=&` bind)
/// yet leaks ~4 blocks/iteration.
///
/// FIXED (by-ref bug #5): `__rt_object_free_deep` now emits descriptor tag 8 for owned
/// reference-property cells with a refcounted payload (deref cell → `__rt_decref_any` payload →
/// free cell) on both targets, so this construct-only loop reports `allocs == frees`.
#[test]
fn test_owned_reference_property_object_freed_cleanly() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class Box { public array $items = ['a']; public function &ref(): array { return $this->items; } }
for ($i = 0; $i < 5; $i++) {
    $b = new Box();
}
echo "ok";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ok");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "owned reference-property object leaked: {}", out.stderr);
}

/// By-ref bug #5, tag 8 with a STRING payload: a class exposing a `string` property by reference
/// (`public function &ref(): string`) owns a 16-byte ref-cell per instance holding a persisted
/// string pointer. The destructor must decref the string then free the cell, so a construct-only
/// loop stays heap-clean (`allocs == frees`).
#[test]
fn test_owned_string_reference_property_object_freed_cleanly() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class S { public string $name = 'hello'; public function &ref(): string { return $this->name; } }
for ($i = 0; $i < 4; $i++) {
    $s = new S();
}
echo "ok";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ok");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "owned string reference-property object leaked: {}", out.stderr);
}

/// By-ref bug #5, tag 10 with a SCALAR payload: a class exposing an `int` property by reference
/// (`public function &ref(): int`) owns a 16-byte ref-cell holding a non-refcounted scalar. The
/// destructor must free the cell only (no payload decref), so a construct-only loop stays
/// heap-clean (`allocs == frees`) with no spurious decref of an integer payload.
#[test]
fn test_owned_scalar_reference_property_object_freed_cleanly() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class N { public int $n = 7; public function &ref(): int { return $this->n; } }
for ($i = 0; $i < 4; $i++) {
    $x = new N();
}
echo "ok";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ok");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "owned scalar reference-property object leaked: {}", out.stderr);
}

/// By-ref bug #5 double-free safety: `$h->data = &$b->items` overwrites Holder::data's slot with a
/// pointer to Box::items's cell (`BindPropRefCell` shares the cell). Holder::data is a whole-program
/// `=&` rebind target, so `rebound_reference_properties` demotes its owned cell back to descriptor
/// tag 0 — only Box::items (tag 8) frees the shared cell, so the program must run cleanly and print
/// the shared count (2) with no double-free/abort.
#[test]
fn test_cross_object_reference_bind_no_double_free() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class Box { public array $items = ['a']; }
class Holder { public array $data = []; }
$b = new Box();
$h = new Holder();
$h->data = &$b->items;
$h->data[] = 'y';
echo count($b->items);
"#,
    );
    assert!(out.success, "cross-object reference bind aborted (double-free?): {}", out.stderr);
    assert_eq!(out.stdout, "2");
}

/// By-ref bug #5 double-free safety, the narrow case: `$b1->items = &$b2->items` makes Box::items
/// itself both a `=&` source and target. Because Box::items is a rebind target anywhere in the
/// program, `rebound_reference_properties` demotes ALL Box::items cells to tag 0 (leak-as-before),
/// guaranteeing the shared cell is never freed twice. The program must run cleanly and print 2.
#[test]
fn test_same_class_reference_bind_no_double_free() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class Box { public array $items = ['a']; }
$b1 = new Box();
$b2 = new Box();
$b1->items = &$b2->items;
$b1->items[] = 'y';
echo count($b2->items);
"#,
    );
    assert!(out.success, "same-class reference bind aborted (double-free?): {}", out.stderr);
    assert_eq!(out.stdout, "2");
}

/// Heap behavior for an unspecialized `Mixed`-receiver `__call` that reads a forwarded string
/// argument in a loop: the receiver is `mixed` (no singular class), so the checker finalization
/// widens the `__call` `params[1]` from `Array(Never)` to `Array(Mixed)`, and the body reads `$a[0]`
/// with an 8-byte Mixed stride matching the hand-built args array. Functional correctness (the
/// forwarded string prints) is asserted here and fully covered by the codegen tests.
///
/// The `allocs == frees` assertion is left IGNORED because Mixed-receiver dispatch is inherently
/// leaky today, independent of this checker fix: a plain (non-`__call`) Mixed-receiver method call in
/// the same loop shape leaks ~1 block/iteration (the pre-existing union-receiver method-dispatch
/// leak). The Mixed-args build/read path adds a further ~2 blocks/iteration. Neither leak is caused
/// by widening `Array(Never)` to `Array(Mixed)` — the fix only makes the read return the correct
/// value. Un-ignore once the union-receiver dispatch leak and the Mixed-args build/read leak are
/// fixed separately.
#[test]
#[ignore = "pre-existing Mixed-receiver dispatch leak (reproduced without __call) plus Mixed-args build/read leak block heap-clean; checker fix only corrects the read value"]
fn test_mixed_receiver_magic_call_string_arg_loop_heap_clean() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
class P { public function __call($n, $a): string { return "h:" . $a[0]; } }
function mk(int $i): mixed { return new P(); }
for ($i = 0; $i < 5; $i++) {
    $o = mk($i);
    $s = $o->whatever("z");
    echo $s;
}
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "h:zh:zh:zh:zh:z");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Regression: a refcounted method-call result consumed through a ternary must
/// not leak. The ternary merge temp is typed `Mixed` (a method call's syntactic
/// type is `Mixed` before lowering), so each branch value is `MixedBox`-ed. The
/// box persists strings / increfs heap children into its own reference, so the
/// original owned method-call result must be released after boxing; previously it
/// leaked ~2-3 blocks per call. A runtime-unknown `$argc` keeps the ternary from
/// being folded away. Heap must be clean at exit.
#[test]
fn test_regression_ternary_then_method_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function g(): string { return "v" . strlen("ab"); } }
function f(A $a, int $c): string { return $c ? $a->g() : "z"; }
$a = new A();
$last = "";
for ($i = 0; $i < 50; $i++) { $last = f($a, $argc); }
echo $last;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "v2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: the direct-return baseline (no ternary) of a method-call result
/// must stay clean. Kept alongside the ternary shapes as a control so a future
/// change that breaks the plain return path is caught here too.
#[test]
fn test_regression_direct_return_method_result_baseline_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function g(): string { return "v" . strlen("ab"); } }
function f(A $a): string { return $a->g(); }
$a = new A();
$last = "";
for ($i = 0; $i < 50; $i++) { $last = f($a); }
echo $last;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "v2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: a ternary method-call result stored to a local and then returned
/// must not leak. This isolates the merge-temp store path (the box's original
/// source release) from the return-coercion path.
#[test]
fn test_regression_ternary_result_stored_then_returned_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function g(): string { return "v" . strlen("ab"); } }
function f(A $a, int $c): string { $x = $c ? $a->g() : "z"; return $x; }
$a = new A();
$last = "";
for ($i = 0; $i < 50; $i++) { $last = f($a, $argc); }
echo $last;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "v2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: the owning branch of a ternary in the else position must not leak.
/// The condition is false at runtime (`$argc > 100`) so the method-call branch
/// actually executes; the boxed merge temp and its return coercion must both
/// release their owned sources.
#[test]
fn test_regression_ternary_else_method_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function g(): string { return "v" . strlen("ab"); } }
function f(A $a, int $c): string { return $c > 100 ? "z" : $a->g(); }
$a = new A();
$last = "";
for ($i = 0; $i < 50; $i++) { $last = f($a, $argc); }
echo $last;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "v2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: a discarded ternary expression-statement whose taken branch owns a
/// method-call result must not leak. With the result discarded, the merge temp is
/// released at the merge block; the leak here came purely from the boxed branch
/// value's original source never being released.
#[test]
fn test_regression_discarded_ternary_method_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function g(): string { return "v" . strlen("ab"); } }
function f(A $a, int $c): void { $c ? $a->g() : "z"; }
$a = new A();
for ($i = 0; $i < 50; $i++) { f($a, $argc); }
echo "ok";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ok");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: a short-ternary (`?:`) over a method-call result must not leak. It
/// shares the merge-temp machinery with the full ternary, so the boxed value's
/// source release and the return coercion release must both fire.
#[test]
fn test_regression_short_ternary_method_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function g(): string { return "v" . strlen("ab"); } }
function f(A $a): string { return $a->g() ?: "z"; }
$a = new A();
$last = "";
for ($i = 0; $i < 50; $i++) { $last = f($a); }
echo $last;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "v2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: a nested ternary whose innermost taken branch owns a method-call
/// result must not leak. Each ternary level allocates its own boxed merge temp;
/// all owned sources must be released.
#[test]
fn test_regression_nested_ternary_method_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function g(): string { return "v" . strlen("ab"); } }
function f(A $a, int $c): string { return $c ? ($c > 1 ? $a->g() : "y") : "z"; }
$a = new A();
$last = "";
for ($i = 0; $i < 50; $i++) { $last = f($a, $argc); }
echo $last;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "y");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: an array-returning method consumed through a ternary must not leak.
/// The array return path unboxes the Mixed merge temp back to a concrete array via
/// a runtime clone; the source box must be released after the unbox, otherwise the
/// box and its retained array child leak on every call.
#[test]
fn test_regression_ternary_array_method_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function arr(): array { return [1, 2, 3, 4]; } }
function f(A $a, int $c): array { return $c ? $a->arr() : []; }
$a = new A();
$n = 0;
for ($i = 0; $i < 50; $i++) { $r = f($a, $argc); $n = count($r); }
echo $n;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "4");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: a method returning an unannotated (`Mixed`) value consumed through
/// a ternary with a `Mixed` function return must not leak. This exercises the
/// concrete-to-Mixed return coercion (`MixedBox`) release path.
#[test]
fn test_regression_ternary_mixed_return_method_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public function g(): string { return "v" . strlen("ab"); } }
function f(A $a, int $c) { return $c ? $a->g() : "z"; }
$a = new A();
$last = "";
for ($i = 0; $i < 50; $i++) { $last = f($a, $argc); }
echo $last;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "v2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Verifies (Bug A) that an inline `new C()` passed directly as a user-call
/// argument is released after the call. The callee `grab(Plain $p): int` borrows
/// the object and returns an `int`, so the caller owns the temporary; before the
/// fix the owning `object_new` argument was never released and leaked 1 block per
/// loop iteration. The heap must be clean after 5 iterations.
#[test]
fn test_inline_new_call_arg_released_after_user_call() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Plain { public int $v = 1; public function get(): int { return $this->v; } }
function grab(Plain $p): int { return $p->get(); }
$t = 0;
for ($i = 0; $i < 5; $i++) { $t += grab(new Plain()); }
echo $t;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "5");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Verifies (Bug A control) that assigning `new C()` to a local first and passing
/// the local stays heap-clean — the argument is a `load_local`, not an owning
/// temporary, so it must not be released by the caller (no regression, no
/// double-free). The local is released by ordinary scope cleanup.
#[test]
fn test_local_object_call_arg_stays_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Plain { public int $v = 1; public function get(): int { return $this->v; } }
function grab(Plain $p): int { return $p->get(); }
$t = 0;
for ($i = 0; $i < 5; $i++) { $o = new Plain(); $t += grab($o); }
echo $t;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "5");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Verifies (Bug A no-double-free) that when a call returns its own inline-`new`
/// argument (identity passthrough `f(Plain $p): Plain { return $p; }`), the
/// returned object is NOT released early by the caller-side arg cleanup. The
/// result aliases the argument, so `call_result_may_alias_arg` must skip it; the
/// value survives to be read, and the heap stays clean over the loop.
#[test]
fn test_identity_passthrough_inline_new_not_double_freed() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Plain { public int $v = 7; }
function f(Plain $p): Plain { return $p; }
$s = 0;
for ($i = 0; $i < 5; $i++) { $x = f(new Plain()); $s += $x->v; }
echo $s;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "35");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: a discarded `realpath()` result must not leak. `realpath` boxes its
/// result through `box_owned_string_or_false_result`, which allocates a fresh owned
/// Mixed cell (refcount 1) holding an owned persisted string. As a discarded
/// expression statement its temporary must be released like `end`/`array_pop`;
/// before adding `realpath` to `builtin_call_result_owns_storage_as_temporary` it
/// leaked two blocks per call (the Mixed cell + its inner string). Loops 100 times
/// so any per-call leak is unmistakable; the heap must be clean at exit.
#[test]
fn test_regression_discarded_realpath_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$i = 0;
while ($i < 100) { realpath("/tmp"); $i++; }
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression: a discarded `file_get_contents()` result must not leak. Like
/// `realpath`, its EIR lowering boxes the read bytes through
/// `box_owned_string_or_false_result` into a fresh owned Mixed cell, so a discarded
/// statement result must be released as an owning temporary. Reads a portable file
/// (`/etc/hosts` exists on macOS and Linux) 100 times and discards each result; the
/// heap must be clean at exit (was two blocks per call before the fix).
#[test]
fn test_regression_discarded_file_get_contents_result_does_not_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$i = 0;
while ($i < 100) { file_get_contents("/etc/hosts"); $i++; }
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Regression test for the once-guard fix
/// (`crate::ir_lower::stmt::lower_static_var`/`Op::StaticLocalInitialized`): a `static $f;
/// $f ??= function() use (...) {...};` closure default must allocate its closure descriptor
/// exactly once across calls, not once per call. Before the fix, `Op::InitStaticLocal`'s codegen
/// only guarded the final store — the closure-creating instructions ran unconditionally on every
/// call, leaking a fresh (unstored, unreleased) closure descriptor on calls 2..N. Calling the
/// closure-returning function 3 times must leave exactly the one persistent closure live.
#[test]
fn test_regression_static_closure_default_once_guard_no_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function make() {
    $x = 10;
    static $f;
    $f ??= function () use ($x) {
        return $x;
    };
    return $f();
}
echo make();
echo make();
echo make();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "101010");
    assert!(
        out.stderr
            .contains("HEAP DEBUG: leak summary: live_blocks=1"),
        "expected only the one persistent closure descriptor to remain live, got: {}",
        out.stderr
    );
}

/// Regression test for the once-guard fix: a `static $obj = new Sentinel();` direct initializer
/// must construct the object exactly once across calls, not once per call. Before the fix, the
/// `new Sentinel()` value-producing instructions ran unconditionally on every call (only the
/// final store into the persistent slot was once-guarded), leaking a fresh (unstored, unreleased)
/// object on calls 2..N. Calling the function 3 times must leave exactly the one persistent
/// object live.
#[test]
fn test_regression_static_direct_new_object_initializer_once_guard_no_leak() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Sentinel {
    public int $hits = 0;
}
function f() {
    static $s = new Sentinel();
    $s->hits++;
    return $s->hits;
}
echo f();
echo f();
echo f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "123");
    assert!(
        out.stderr
            .contains("HEAP DEBUG: leak summary: live_blocks=1"),
        "expected only the one persistent Sentinel instance to remain live, got: {}",
        out.stderr
    );
}
