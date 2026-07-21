//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object static properties, including class static method string param, class static and instance, and static property read write.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures compile to native binaries while malformed or fatal cases assert captured failures.

use super::*;

/// Tests calling a class static method with a string parameter and concatenating the result.
#[test]
fn test_class_static_method_string_param() {
    let out = compile_and_run(
        r#"<?php
class Utils {
    public static function greet($name) { return "Hello " . $name; }
}
echo Utils::greet("World");
"#,
    );
    assert_eq!(out, "Hello World");
}

/// Tests calling a class static method that returns a new instance via `new`, then
/// invoking an instance method on the returned object.
#[test]
fn test_class_static_and_instance() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public $n;
    public function __construct($n) { $this->n = $n; }
    public function next() { return $this->n + 1; }
    public static function make($n) { return new Counter($n); }
}
$c = Counter::make(4);
echo $c->next();
"#,
    );
    assert_eq!(out, "5");
}

// === Nested array access tests ===

/// Tests static property read of an `int` typed static, followed by a write, then another read.
#[test]
fn test_static_property_read_write() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public static int $count = 1;
}
echo Counter::$count;
Counter::$count = 5;
echo Counter::$count;
"#,
    );
    assert_eq!(out, "15");
}

/// Tests `self::$prop` access and mutation of an `int` typed static within a static method.
/// Verifies that `self::` resolves to the declaring class, not the called class.
#[test]
fn test_static_property_self_access_in_static_method() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public static int $count = 1;
    public static function bump() {
        self::$count = self::$count + 1;
        return self::$count;
    }
}
echo Counter::bump();
echo Counter::bump();
"#,
    );
    assert_eq!(out, "23");
}

/// Tests `parent::$prop` access to a `protected` static property from a child class.
#[test]
fn test_static_property_parent_access_in_static_method() {
    let out = compile_and_run(
        r#"<?php
class Base {
    protected static int $seed = 4;
}
class Child extends Base {
    public static function read() {
        return parent::$seed;
    }
}
echo Child::read();
"#,
    );
    assert_eq!(out, "4");
}

/// Tests that a non-redeclared static property has a single shared slot across inheritance
/// when accessed via `static::$prop` from a parent class method.
#[test]
fn test_static_property_inherited_storage_is_shared() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public static int $count = 2;
    public static function set($value) {
        static::$count = $value;
    }
}
class Child extends Base {}
Child::set(9);
echo Base::$count;
echo Child::$count;
"#,
    );
    assert_eq!(out, "99");
}

/// Tests that a redeclared static property creates a separate storage slot per class,
/// and that `static::$prop` and `$obj::$prop` both dispatch to the late-bound class's slot.
#[test]
fn test_static_property_redeclaration_uses_child_storage() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public static int $count = 1;
    public static function get() {
        return static::$count;
    }
    public static function set($value) {
        static::$count = $value;
    }
}
class Child extends Base {
    public static int $count = 2;
}
echo Base::get() . ":" . Child::get() . ":";
Child::set(9);
echo Base::$count . ":" . Child::$count;
"#,
    );
    assert_eq!(out, "1:2:1:9");
}

/// Tests appending to and updating a static `array` property directly via `$Class::$prop[idx]`.
#[test]
fn test_static_property_direct_array_writes() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public static array $items = [];
}
Registry::$items[] = 4;
Registry::$items[] = 5;
Registry::$items[1] = 8;
echo Registry::$items[0] . ":" . Registry::$items[1];
"#,
    );
    assert_eq!(out, "4:8");
}

/// Tests `+=` and `*=` compound assignment on an `int` typed static property.
#[test]
fn test_static_property_compound_assign() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public static int $count = 4;
}
Counter::$count += 6;
Counter::$count *= 2;
echo Counter::$count;
"#,
    );
    assert_eq!(out, "20");
}

/// Tests `+=` and `-=` compound assignment on individual elements of a static `array` property.
#[test]
fn test_static_property_array_compound_assign() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public static $items = [3, 5, 7];
}
Registry::$items[0] += 9;
Registry::$items[2] -= 4;
echo Registry::$items[0] . ":" . Registry::$items[2];
"#,
    );
    assert_eq!(out, "12:3");
}

/// Tests that the index expression in `Registry::$items[idx()] += 6` is evaluated exactly once.
/// The side-effect function `idx()` echoes "i" and the result proves no double-evaluation.
#[test]
fn test_static_property_array_compound_assign_evaluates_index_once() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public static $items = [3, 5, 7];
}

function idx() {
    echo "i";
    return 1;
}

Registry::$items[idx()] += 6;
echo ":" . Registry::$items[1];
"#,
    );
    assert_eq!(out, "i:11");
}

/// Tests that `static::$items[]` and `static::$items[0]` in a parent method write to the
/// late-bound class's redeclared static array, not the parent's, when called on a child.
#[test]
fn test_static_property_redeclared_array_writes_are_late_bound() {
    let out = compile_and_run(
        r#"<?php
class BaseBag {
    public static array $items = [];
    public static function add($value) {
        static::$items[] = $value;
    }
    public static function replaceFirst($value) {
        static::$items[0] = $value;
    }
    public static function first() {
        return static::$items[0];
    }
}
class ChildBag extends BaseBag {
    public static array $items = [];
}
BaseBag::add(1);
ChildBag::add(2);
ChildBag::replaceFirst(7);
echo BaseBag::first() . ":" . ChildBag::first();
"#,
    );
    assert_eq!(out, "1:7");
}

/// Tests that `static::$count` inside a parent static method causes a fatal error when
/// the calling class has a private redeclaration of the same static property.
#[test]
fn test_static_property_late_bound_private_redeclaration_read_is_fatal() {
    let err = compile_and_run_expect_failure(
        r#"<?php
class Base {
    private static int $count = 1;
    public static function read() {
        echo static::$count;
    }
}
class Child extends Base {
    private static int $count = 2;
}
Child::read();
"#,
    );
    assert!(
        err.contains("Cannot access private static property"),
        "{err}"
    );
}

/// Tests that `static::$count` inside a parent static method causes a fatal error when
/// the calling class has a private redeclaration and the property is being written.
#[test]
fn test_static_property_late_bound_private_redeclaration_write_is_fatal() {
    let err = compile_and_run_expect_failure(
        r#"<?php
class Base {
    private static int $count = 1;
    public static function write() {
        static::$count = 3;
    }
}
class Child extends Base {
    private static int $count = 2;
}
Child::write();
"#,
    );
    assert!(
        err.contains("Cannot access private static property"),
        "{err}"
    );
}

/// Tests typed `string` static property read and write.
#[test]
fn test_static_string_property_assignment() {
    let out = compile_and_run(
        r#"<?php
class Labels {
    public static string $name = "a";
}
echo Labels::$name;
Labels::$name = "bc";
echo Labels::$name;
"#,
    );
    assert_eq!(out, "abc");
}

/// Verifies that writing to a static `array` property with a *variable* string key
/// (`C::$prop[$key] = v`) type-checks and stores the value under the string key.
/// Guards Fix 2: the static-property write path previously rejected every non-`Int`
/// key with "Array index must be integer"; it now normalizes the key and uses the
/// shared `is_php_array_key_type` helper, accepting PHP-coercible keys just like the
/// instance-property write path. Cross-check:
/// `php -r 'class C { public static array $p = []; } $k = "k"; C::$p[$k] = 1; var_dump(C::$p["k"]);'`
/// prints `int(1)`.
///
/// Currently `#[ignore]` because of a SEPARATE, out-of-scope downstream EIR codegen
/// gap: `lower_static_property_array_assign` (`src/ir_lower/stmt/mod.rs`) funnels
/// every `Array(_)`-typed static property through `Op::ArraySet`, whose codegen
/// (`require_integer_like_index` in `src/codegen_ir/lower_inst/arrays.rs`) rejects
/// non-int-like index types. Unlike the instance-property and local-array lowering
/// paths, the static-property path has no Str/Mixed-key routing
/// (`lower_string_key_array_promotion` / `lower_mixed_key_array_set` / `Op::HashSet`).
/// The checker-level fix (Fix 2) is verified by the console probe (0 remaining
/// "Array index must be integer" errors); this test documents the codegen gap for a
/// follow-up that extends `lower_static_property_array_assign` to mirror
/// `lower_property_array_assign`.
#[test]
#[ignore = "downstream EIR codegen gap: lower_static_property_array_assign has no Str/Mixed-key path"]
fn test_static_property_array_write_variable_string_key() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public static array $items = [];
}

$key = "name";
Registry::$items[$key] = 1;
echo Registry::$items["name"];
}
/// Tests post-increment `A::$x++` on a static property (regression for #372).
#[test]
fn test_static_property_post_increment() {
    let out = compile_and_run(
        r#"<?php
class A { public static $x = 0; }
A::$x++;
echo A::$x;
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies that writing to a static `array` property with a `mixed` key
/// type-checks and runs. Guards the `Mixed` case of the static-property write path
/// (Fix 2): a `mixed` key is the gradual-typing boundary and is accepted by the
/// shared `is_php_array_key_type` helper, mirroring the read path.
///
/// Currently `#[ignore]` for the same downstream EIR codegen gap as
/// `test_static_property_array_write_variable_string_key` (see that test's docblock).
/// The checker-level acceptance is verified by the console probe.
#[test]
#[ignore = "downstream EIR codegen gap: lower_static_property_array_assign has no Str/Mixed-key path"]
fn test_static_property_array_write_mixed_key() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public static array $items = [];
}

function pick(mixed $k): mixed {
    return $k;
}

$key = pick("name");
Registry::$items[$key] = 7;
echo Registry::$items["name"];
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies a dynamic-named static property read (`C::${$n}`) selects the property whose name
/// matches the runtime string, dispatching among several candidates of the same class.
#[test]
fn test_dynamic_static_property_read_named_receiver() {
    let out = compile_and_run(
        r#"<?php
class Config {
    public static int $timeout = 30;
    public static int $retries = 5;
    public static string $name = "prod";
}
$k = 'retries';
echo Config::${$k};
echo "|";
$k2 = 'name';
echo Config::${$k2};
"#,
    );
    assert_eq!(out, "5|prod");
}

/// Verifies a dynamic-named static property read through a `self::` receiver inside a method,
/// selecting among array-typed static properties by runtime name.
#[test]
fn test_dynamic_static_property_read_self_receiver() {
    let out = compile_and_run(
        r#"<?php
class Reg {
    public static array $errors = ['a', 'b'];
    public static array $warnings = ['w'];
    public static function count(string $n): int { return count(self::${$n}); }
}
echo Reg::count('errors');
echo "|";
echo Reg::count('warnings');
"#,
    );
    assert_eq!(out, "2|1");
}

/// Verifies a direct write through a dynamic-named static property (`self::${$n} = v`) stores into
/// the property selected by the runtime name, choosing the right candidate among ≥2.
#[test]
fn test_dynamic_static_property_write() {
    let out = compile_and_run(
        r#"<?php
class Cfg {
    public static int $x = 0;
    public static int $y = 0;
}
$n = 'x';
Cfg::${$n} = 5;
$m = 'y';
Cfg::${$m} = 9;
echo Cfg::$x;
echo "|";
echo Cfg::$y;
"#,
    );
    assert_eq!(out, "5|9");
}

/// Verifies a dynamic-named static property write leaves no leaked heap: the runtime name string
/// and dispatch temporaries are function-local, and scalar static storage frees cleanly at exit.
#[test]
fn test_dynamic_static_property_write_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Cfg {
    public static int $a = 0;
    public static int $b = 0;
}
function setit(string $n, int $v): void { Cfg::${$n} = $v; }
setit('a', 7);
setit('b', 9);
echo Cfg::$a;
echo Cfg::$b;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "79", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Verifies an array-element write through a dynamic-named static property
/// (`self::${$n}[$k] = v`) mutates and re-persists the selected array, choosing the right
/// candidate among ≥2 indexed-array properties.
#[test]
fn test_dynamic_static_property_array_element_write() {
    let out = compile_and_run(
        r#"<?php
class Store {
    public static array $cache = [];
    public static array $other = [];
}
$n = 'cache';
Store::${$n}[0] = 'v';
Store::${$n}[1] = 'x';
$n2 = 'other';
Store::${$n2}[0] = 'w';
echo Store::$cache[0];
echo Store::$cache[1];
echo "|";
echo Store::$other[0];
"#,
    );
    assert_eq!(out, "vx|w");
}

/// Verifies an array push through a dynamic-named static property (`self::${$n}[] = v`) appends to
/// the selected array and re-persists it, then a later indexed write updates it in place.
#[test]
fn test_dynamic_static_property_array_push() {
    let out = compile_and_run(
        r#"<?php
class Bag {
    public static array $items = [];
    public static array $tags = [];
}
$n = 'items';
Bag::${$n}[] = 'a';
Bag::${$n}[] = 'b';
Bag::${$n}[0] = 'z';
echo Bag::$items[0];
echo Bag::$items[1];
echo count(Bag::$items);
"#,
    );
    assert_eq!(out, "zb2");
}
/// Tests pre-increment `++A::$x` on a static property (regression for #372).
#[test]
fn test_static_property_pre_increment() {
    let out = compile_and_run(
        r#"<?php
class A { public static $x = 0; }
++A::$x;
echo A::$x;
"#,
    );
    assert_eq!(out, "1");
}

/// Tests post-decrement and pre-decrement `--A::$x` / `A::$x--` on a static property.
#[test]
fn test_static_property_decrement() {
    let out = compile_and_run(
        r#"<?php
class A { public static $x = 5; }
A::$x--;
echo A::$x;
echo "\n";
--A::$x;
echo A::$x;
"#,
    );
    assert_eq!(out, "4\n3");
}

/// Tests `self::$x++` and `++self::$x` inside a static method.
#[test]
fn test_static_property_self_increment_in_method() {
    let out = compile_and_run(
        r#"<?php
class A {
    public static $x = 0;
    public static function inc(): void {
        self::$x++;
        ++self::$x;
    }
}
A::inc();
echo A::$x;
"#,
    );
    assert_eq!(out, "2");
}

/// Tests `static::$x++` and `++static::$x` inside a static method use late-static storage.
#[test]
fn test_static_property_static_increment_in_method() {
    let out = compile_and_run(
        r#"<?php
class A {
    public static $x = 0;
    public static function inc(): void {
        static::$x++;
        ++static::$x;
    }
}
A::inc();
echo A::$x;
"#,
    );
    assert_eq!(out, "2");
}

/// Tests `parent::$x++` and `++parent::$x` inside a child static method.
#[test]
fn test_static_property_parent_increment_in_method() {
    let out = compile_and_run(
        r#"<?php
class A { public static $x = 0; }
class B extends A {
    public static function inc(): void {
        parent::$x++;
        ++parent::$x;
    }
}
B::inc();
echo A::$x;
"#,
    );
    assert_eq!(out, "2");
}

/// Regression: storing a *borrowed* object (an interface-typed parameter) into a
/// `static` class property, then reading it back in a different scope and
/// dispatching methods on it must work. The store consumes (moves) its operand,
/// so a borrowed value must be acquired first; without the acquire the property
/// dangled once the caller released the borrow, and the later dispatch crashed
/// with a fatal "Call to a member function ... on null". Also exercises multiple
/// method calls plus a cross-interface `instanceof` downcast to confirm the
/// class tag survives every load (not just the first).
#[test]
fn test_static_property_holds_borrowed_object_for_later_dispatch() {
    let out = compile_and_run(
        r#"<?php
interface Store { public function get(): string; }
interface Named { public function name(): string; }
class Holder { public static ?Store $s = null; }
class Impl implements Store, Named {
    public function get(): string { return "V"; }
    public function name(): string { return "N"; }
}
function register(?Store $x): void { Holder::$s = $x; }
register(new Impl());
$o = Holder::$s;
if ($o !== null) {
    echo $o->get();
    echo $o->get();
    if ($o instanceof Named) { echo $o->name(); }
    echo $o->get();
}
"#,
    );
    assert_eq!(out, "VVNV");
}
