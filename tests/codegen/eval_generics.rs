//! Purpose:
//! Exercises generic specializations through the real Magician bridge and runtime cells.
//!
//! Called from:
//! - `cargo test --test codegen_tests eval_generics`.
//!
//! Key details:
//! - Runtime-selected source prevents literal eval from taking AOT's shortcut.
//! - Fixtures cover persistence, class storage, type failures, and portable strict syntax.

use crate::support::*;

/// Builds dynamic eval source without allowing compile-time evaluation of the string.
fn dynamic_eval(body: &str) -> String {
    format!("<?php\n$code = $argc > 1 ? $argv[1] : <<<'CODE'\n{body}\nCODE;\neval($code);\n")
}

/// Native templates persist across eval barriers and specialize scalar and nested array calls.
#[test]
fn test_eval_generics_functions_persist() {
    let source = r#"<?php
$declarations = $argc > 1 ? $argv[1] : 'function identity<T>(T $v): T { return $v; } function first<T>(array<T> $v): T { return $v[0]; }';
eval($declarations);
$use = $argc > 1 ? $argv[1] : 'echo identity(7), "|", identity<string>("ok"), "|", first([[9]])[0], "|", identity(...["v" => "spread"]); $f = identity<int>(...); echo "|", $f(8), "|", call_user_func_array("identity", ["v" => "callback"]);';
eval($use);
"#;
    assert_eq!(compile_and_run(source), "7|ok|9|spread|8|callback");
}

/// Native code can invoke generic templates declared by Magician before their first specialization.
#[test]
fn test_eval_generics_calls_from_native_code() {
    let source = r#"<?php
$code = $argc > 1 ? $argv[1] : 'function id<T>(T $v): T { return $v; } class Box<T> { public function __construct(public T $v) {} public function get(): T { return $this->v; } public static function make(T $v): Box<T> { return new Box<T>($v); } }';
eval($code);
echo id(7), '|', (new Box('native'))->get(), '|', Box::make(8)->get();
"#;
    assert_eq!(compile_and_run(source), "7|native|8");
}

/// Class and constructor-local slots, generic methods, and closures use real boxed storage.
#[test]
fn test_eval_generics_classes_methods_and_constructors() {
    let source = dynamic_eval(r#"
class Box<T> {
    public function __construct(private T $value) {}
    public function get(): T { return $this->value; }
    public function map<U>(callable(T): U $f): Box<U> { return new Box<U>($f($this->value)); }
}
class Pair<T> { public function __construct<U>(public T $left, public U $right) {} }
function Box<T>(T $v): T { return $v; }
$b = new Box('hello');
$p = new Pair<int>(1, 'two');
echo Box(9), '|', $b->map(function(string $s): int { return strlen($s); })->get(), '|', $p->right, '|', get_class($p);
"#);
    assert_eq!(compile_and_run(&source), "9|5|two|Pair<int, string>");
}

/// Runtime binding failures and structural contracts use the normal catchable TypeError path.
#[test]
fn test_eval_generics_bounds_arrays_and_errors() {
    let source = dynamic_eval(r#"
class Base {} class Child extends Base {}
function bounded<T: Base>(T $v): T { return $v; }
function same<T>(T $a, T $b): T { return $a; }
function countInts(array<int> $a): int { return count($a); }
function bad(): array<int> { return ['bad']; }
echo get_class(bounded(new Child())), '|', countInts([1, 2]);
try { bounded(1); } catch (TypeError $e) { echo '|bound'; }
try { same(1, 'a'); } catch (TypeError $e) { echo '|conflict'; }
try { countInts(['a']); } catch (TypeError $e) { echo '|array'; }
try { bad(); } catch (TypeError $e) { echo '|return'; }
"#);
    assert_eq!(compile_and_run(&source), "Child|2|bound|conflict|array|return");
}

/// PHPDoc templates stay portable in strict PHP mode, including promoted constructor slots.
#[test]
fn test_eval_generics_docblocks_in_strict_php() {
    let source = dynamic_eval(r#"
/** @template T */
class Box {
    /** @param T $value */
    public function __construct(private $value) {}
    /** @return T */
    public function get() { return $this->value; }
}
class Item {
    /**
     * @template U
     * @param U $value
     */
    public function __construct(public $value) {}
}
/**
 * @template V
 * @param V $value
 * @return V
 */
function identity($value) { return $value; }
echo (new Box(7))->get(), '|', identity('ok'), '|', (new Item('item'))->value;
"#);
    for flags in [&[][..], &["--strict-php"][..]] {
        assert_eq!(compile_cli_file_and_run_with_flags(&source, flags), "7|ok|item");
    }
}

/// Retained generic templates from an included file execute correctly on repeated calls.
#[test]
fn test_eval_generics_runtime_include() {
    let files = [
        ("main.php", "<?php $code = $argc > 1 ? $argv[1] : 'require_once \"templates.php\"; echo identity(7), \"|\", identity(\"ok\");'; eval($code);"),
        ("templates.php", "<?php /**\n * @template T\n * @param T $value\n * @return T\n */ function identity($value) { return $value; }"),
    ];
    assert_eq!(compile_and_run_files(&files, "main.php"), "7|ok");
}

/// Keeps the documented Magician example executable through the full compiler pipeline.
#[test]
fn test_eval_generics_example() {
    assert_eq!(compile_and_run(include_str!("../../examples/eval-generics/main.php")), "42|Magician|8\ntype checked\n");
}

/// Evaluated argument owners protect by-reference calls, COW arrays, and rejected calls.
#[test]
fn test_eval_generics_argument_ownership_and_cow() {
    let source = dynamic_eval(r#"
function swap<T>(T &$left, T &$right): void { T $held = $left; $left = $right; $right = $held; }
function change<T>(array<T> $values, T $value): array<T> { $values[0] = $value; return $values; }
function id<T>(T $v): T { return $v; }
$a = 'left'; $b = 'right'; swap($a, $b);
$original = ['old']; $changed = change($original, 'new');
echo $a, '|', $b, '|', $original[0], '|', $changed[0];
for ($i = 0; $i < 100; $i++) {
    $value = id(['payload']);
    try { id<int>(['bad']); } catch (TypeError $e) {}
    unset($value);
}
"#);
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, "right|left|old|new");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Ordinary eval callables share the corrected reference-copy and array parameter boundaries.
#[test]
fn test_eval_generics_ordinary_call_ownership_regression() {
    let source = dynamic_eval(r#"
function swap(&$left, &$right): void { $held = $left; $left = $right; $right = $held; }
function change(array $values): array { $values[0] = 'new'; $values[] = 'tail'; return $values; }
$a = 'left'; $b = 'right'; swap($a, $b);
$original = ['old']; $changed = change($original);
echo $a, '|', $b, '|', $original[0], '|', $changed[0], '|', count($original), '|', count($changed);
"#);
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, "right|left|old|new|1|2");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Array contracts preserve indexed storage and nullable/union writes with balanced cleanup.
#[test]
fn test_eval_generics_array_local_contract_regressions() {
    let source = dynamic_eval(r#"
?array<int> $nullable = [1]; $nullable[] = 2; $nullable[0] = 3;
array<int>|null $union = [4]; $union[] = 5; $union[0] = 6;
echo $nullable[0], '|', $nullable[1], '|', $union[0], '|', $union[1];
try { $nullable[] = 'bad'; } catch (TypeError $e) { echo '|value'; }
try { $union['key'] = 7; } catch (TypeError $e) { echo '|key'; }
$union['0'] = 8; echo '|', $union[0], '|', count($union);
function length(array<int> $items): int { return count($items); }
function bad(): array<int> { return ['key' => 1]; }
try { length(['key' => 1]); } catch (TypeError $e) { echo '|argument'; }
try { bad(); } catch (TypeError $e) { echo '|return'; }
try { array<int> $items = ['key' => 1]; } catch (TypeError $e) { echo '|local'; }
array<int>|array<string> $items = [1];
try { $items[] = 'bad'; } catch (TypeError $e) { echo '|arm'; }
echo '|', count($items);
"#);
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, "3|2|6|5|value|key|8|2|argument|return|local|arm|1");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Constructor slots retain identities while class variance and inference see native hierarchy.
#[test]
fn test_eval_generics_constructor_class_variance_regression() {
    let body = dynamic_eval(r#"
class Box<out T: Animal> {
    public function __construct<U>(private T $value, private U $label) {}
    public function get(): T { return $this->value; }
}
function read(Box<Animal> $box): string { return get_class($box->get()); }
function relay<V>(Box<V> $box): V { return $box->get(); }
function bounded<V: Box<Animal>>(V $box): string { return read($box); }
$i = new Box(new Dog(), 1); $s = new Box<Dog>(new Dog(), 'label');
echo read($i), '|', read($s), '|', get_class(relay($i)), '|', bounded($s), '|', get_class($i), '|', get_class($s);
function packed(array<int> $items): void {}
function associative(array<string, int> $items): void {}
try { packed(new Dog()); } catch (TypeError $e) { echo '|packed'; }
try { associative(new Dog()); } catch (TypeError $e) { echo '|assoc'; }
"#);
    let source = format!("<?php\nclass Animal {{}} class Dog extends Animal {{}}\necho get_class(new Dog()), '|';\n{}", body.trim_start_matches("<?php"));
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "stdout: {}\nstderr: {}", output.stdout, output.stderr);
    assert_eq!(output.stdout, "Dog|Dog|Dog|Dog|Dog|Box<Dog, int>|Box<Dog, string>|packed|assoc");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Same short parent names receive distinct qualified PHPDoc arguments in both PHP modes.
#[test]
fn test_eval_generics_docblock_qualified_parent_regression() {
    let source = dynamic_eval(r#"
namespace Ns;
/** @template T */
class Foo {
    /** @param T $value */
    public function __construct(private $value) {}
    /** @return T */
    public function get() { return $this->value; }
}
namespace Other;
/** @template T */
interface Foo { /** @return T */ public function label(); }
namespace App;
/**
 * @extends \Ns\Foo<int>
 * @implements \Other\Foo<string>
 */
class Child extends \Ns\Foo implements \Other\Foo {
    public function label(): string { return 'label'; }
}
$child = new Child(7); echo $child->get(), '|', $child->label();
"#);
    for flags in [&[][..], &["--strict-php"][..]] {
        assert_eq!(compile_cli_file_and_run_with_flags(&source, flags), "7|label");
    }
}

/// Fully written class and constructor arguments work on a cold cache and through inheritance.
#[test]
fn test_eval_generics_written_constructor_arguments_regression() {
    let source = dynamic_eval(r#"
class Pair<T> { public function __construct<U>(public T $left, public U $right) {} }
$cold = new Pair<int, string>(1, 'cold');
$inferred = new Pair<int>(2, 'inferred');
$warm = new Pair<int, string>(3, 'warm');
echo get_class($cold), '|', get_class($inferred), '|', $warm->right;
try { new Pair<int, string>(4, []); } catch (TypeError $e) { echo '|value'; }
class ParentPair<T> { public function __construct<U>(public T $left, public U $right) {} }
class Child extends ParentPair<int, string> {}
$child = new Child(6, 'child'); echo '|', $child->left, '|', $child->right, '|', get_parent_class($child);
class Base {} class Derived extends Base {}
class DefaultPair<T> { public function __construct<U: Base, V = U>(public T $left, public U $right, public V $other) {} }
$default = new DefaultPair<int, Derived>(7, new Derived(), new Derived());
echo '|', get_class($default);
try { new DefaultPair<int, int>(7, 8, 9); } catch (TypeError $e) { echo '|bound'; }
try { new Pair<int, string, bool>(1, 'two'); } catch (TypeError $e) { echo '|arity'; }
class Shadow<T> { public function __construct<T>(public T $value) {} }
echo '|', get_class(new Shadow<int, string>('ok'));
"#);
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "stdout: {}\nstderr: {}", output.stdout, output.stderr);
    assert_eq!(output.stdout, "Pair<int, string>|Pair<int, string>|warm|value|6|child|ParentPair<int, string>|DefaultPair<int, Derived, Derived>|bound|arity|Shadow<int, string>");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Float and bool writes validate their stored key type without changing caller keys or COW.
#[test]
fn test_eval_generics_normalized_scalar_array_keys_regression() {
    let source = dynamic_eval(r#"
array<int> $items = [1, 2]; $original = $items;
$float = 1.0; $bool = false;
$items[$float] = 3; $items[true] = 4; $items[$bool] = 5; $items[-0.0] = 6; $items[2.0] = 7;
try { $items[false] = 'bad'; } catch (TypeError $e) { echo 'value|'; }
echo $items[0], '|', $items[1], '|', $items[2], '|', count($items), '|', $original[0], '|', $original[1], '|', is_float($float), '|', is_bool($bool);
array<int, int> $map = [1, 2]; $map[true] = 8; $map[0.0] = 9;
?array<int> $nullable = [1]; $nullable[false] = 10;
array<int>|null $union = [1]; $union[0.0] = 11;
echo '|', $map[0], '|', $map[1], '|', $nullable[0], '|', $union[0];
array<string, int> $strings = ['k' => 1];
try { $strings[1.0] = 2; } catch (TypeError $e) { echo '|float'; }
try { $strings[false] = 3; } catch (TypeError $e) { echo '|bool'; }
echo '|', count($strings);
"#);
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "stdout: {}\nstderr: {}", output.stdout, output.stderr);
    assert_eq!(output.stdout, "value|6|4|7|3|1|2|1|1|9|8|10|11|float|bool|1");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Null keys keep PHP's empty-string identity across contracts, COW, and autovivification.
#[test]
fn test_eval_generics_null_array_keys_regression() {
    let source = dynamic_eval(r#"
array<string, int> $map = ['k' => 1]; $original = $map; $key = null;
$map[$key] = 2; $map[null] = 3;
echo $map[''], '|', count($map), '|', count($original), '|', is_null($key);
?array<string, int> $nullable = null; $nullable[null] = 4;
array<string, int>|null $union = null; $union[null] = 5;
echo '|', $nullable[''], '|', $union[''];
array<int> $indexed = [1];
try { $indexed[null] = 2; } catch (TypeError $e) { echo '|indexed'; }
try { $map[null] = 'bad'; } catch (TypeError $e) { echo '|value'; }
echo '|', $map[''];
for ($i = 0; $i < 20; $i++) {
    ?array<int> $failed = null;
    try { $failed['k'] = 2; } catch (TypeError $e) {}
    try { $failed[] = 'bad'; } catch (TypeError $e) {}
}
"#);
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "stdout: {}\nstderr: {}", output.stdout, output.stderr);
    assert_eq!(output.stdout, "3|2|1|1|4|5|indexed|value|3");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Union writes preserve scalars, update string bytes, and only autovivify null or false.
#[test]
fn test_eval_generics_non_array_union_writes_regression() {
    let source = dynamic_eval(r#"
function rhs(&$calls): int { $calls++; return 9; }
$calls = 0; array<int>|string $items = 'keep'; $original = $items; $alias =& $items;
try { $items[] = rhs($calls); } catch (Error $e) { echo $e->getMessage(), '|'; }
echo $items, '|', $calls;
$items[0] = rhs($calls); $alias[-1] = 'X'; $items['01'] = 'Y';
echo '|', $items, '|', $original, '|', $alias;
try { $items['bad'] = 1; } catch (TypeError $e) { echo '|key'; }
try { $items[0] = ''; } catch (Error $e) { echo '|empty'; }
$items[6] = 'Z'; echo '|', $items;
$cases = [true, 7, 1.5];
foreach ($cases as $value) {
    array<int>|bool|int|float $scalar = $value;
    try { $scalar[] = rhs($calls); } catch (Error $e) { echo '|append'; }
    try { $scalar[0] = rhs($calls); } catch (Error $e) { echo '|index'; }
    echo '|', $scalar;
}
echo '|', $calls;
array<int>|bool|null $empty = false; $empty[] = 1; $empty = null; $empty[0] = 2;
echo '|', $empty[0];
"#);
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "stdout: {}\nstderr: {}", output.stdout, output.stderr);
    assert_eq!(output.stdout, "[] operator not supported for strings|keep|1|9YeX|keep|9YeX|key|empty|9YeX  Z|append|index|1|append|index|7|append|index|1.5|8|2");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Ordinary scalar offset writes share PHP's conversion, diagnostics, and reference boundaries.
#[test]
fn test_eval_generics_ordinary_scalar_offsets_regression() {
    let source = dynamic_eval(r#"
class Text { public function __toString(): string { echo 'convert|'; return 'XY'; } }
$items = 'keep'; $original = $items;
$items[null] = 'N'; $items[false] = 'F'; $items[1.0] = 'G'; $items['1x'] = 'P';
$items[2] = new Text();
echo $items, '|', $original;
$keys = ['1.9', '1.9x', '1e2', '1e2x', [], new stdClass()];
foreach ($keys as $key) {
    try { $items[$key] = 'bad'; } catch (TypeError $e) { echo '|key'; }
}
$items[-99] = new Text();
echo '|', $items;
$items[' 1 '] = 'W'; $items['1efoo'] = 'V';
echo '|', $items;
$number = 7;
try { $number[] = 'x'; } catch (Error $e) { echo '|scalar'; }
echo '|', $number;
"#);
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "stdout: {}\nstderr: {}", output.stdout, output.stderr);
    assert_eq!(output.stdout, "convert|FPXp|keep|key|key|key|key|key|key|FPXp|FVXp|scalar|7");
    assert_eq!(output.stderr.matches("String offset cast occurred").count(), 3, "{}", output.stderr);
    assert!(output.stderr.contains("Only the first byte will be assigned"), "{}", output.stderr);
    assert!(output.stderr.contains("Illegal string offset -99"), "{}", output.stderr);
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}

/// Failed construction frees receivers and preserves escaped metadata without running destructors.
#[test]
fn test_eval_generics_failed_construction_cleanup_regression() {
    let body = dynamic_eval(r#"
class Failed<T> {
    public function __construct(public T $value) { throw new Exception('body'); }
    public function __destruct() { echo 'bad-destructor|'; }
}
try { new Failed<int>(1); } catch (Exception $e) { echo $e->getMessage(), '|'; }
class Escaped<T> {
    public function __construct(public T $value, &$saved) { $saved = $this; throw new Exception('escaped'); }
    public function __destruct() { echo 'bad-escaped|'; }
}
$saved = null;
try { new Escaped<int>(2, $saved); } catch (Exception $e) { echo $e->getMessage(), '|'; }
echo get_class($saved), '|', $saved->value, '|';
unset($saved);
class Ordinary {
    public function __construct(int $value) { throw new Exception('ordinary'); }
    public function __destruct() { echo 'bad-argument|'; }
}
try { new Ordinary(3); } catch (Exception $e) { echo $e->getMessage(), '|'; }
class NativeChild<T> extends NativeParent {}
try { new NativeChild<int>(4); } catch (Exception $e) { echo $e->getMessage(); }
"#);
    let source = format!("<?php\nclass NativeParent {{ public function __construct(int $value) {{ throw new Exception('native'); }} public function __destruct() {{ echo 'bad-native|'; }} }}\n{}", body.trim_start_matches("<?php"));
    let output = compile_and_run_with_heap_debug(&source);
    assert!(output.success, "stdout: {}\nstderr: {}", output.stdout, output.stderr);
    assert_eq!(output.stdout, "body|escaped|Escaped<int>|2|ordinary|native");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}
