//! Purpose:
//! Integration or regression tests for diagnostic coverage of type system, including null coalesce assignment missing rhs, null coalesce assignment type change, and string index requires integer.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Invalid PHP snippets are checked through shared diagnostic helpers for messages, spans, and recovery behavior.

use super::*;

/// Verifies that `??=` with no right-hand side expression produces an "Unexpected token" error.
/// Input: `$x ??=;` — the semicolon terminates the expression with no RHS.
#[test]
fn test_error_null_coalesce_assignment_missing_rhs() {
    expect_error("<?php $x ??=;", "Unexpected token");
}

/// Verifies that `??=` rejects a type-changing initializer on an explicitly typed local.
/// Input: `int $x = 5; $x ??= 2.5;` — the declared int contract rejects the float RHS.
#[test]
fn test_error_null_coalesce_assignment_type_change() {
    expect_error(
        "<?php int $x = 5; $x ??= 2.5;",
        "null coalescing assignment for $x must keep int, got float",
    );
}

/// Verifies that a non-integer string subscript is rejected on a string value.
/// Input: `$s = "hello"; echo $s["x"];` — string key "x" is not integer.
#[test]
fn test_error_string_index_requires_integer() {
    expect_error(
        "<?php $s = \"hello\"; echo $s[\"x\"];",
        "String index must be integer",
    );
}

/// Verifies that an object index into a string offset stays rejected under gradual typing.
/// Input: `$s = "hi"; $o = new stdClass(); echo $s[$o];` — an object is not a coercible offset,
/// so the safety boundary still reports a type error rather than accepting/miscompiling it.
#[test]
fn test_error_object_string_offset_is_rejected() {
    expect_error(
        "<?php $s = \"hi\"; $o = new stdClass(); echo $s[$o];",
        "String index must be integer",
    );
}

/// Verifies that an object key into an array inferred packed/int-keyed stays rejected under
/// gradual typing. Input: `$a = [1, 2, 3]; $o = new stdClass(); echo $a[$o];` — an object is not
/// a coercible array key, so the safety boundary still reports a type error.
#[test]
fn test_error_object_array_key_is_rejected() {
    expect_error(
        "<?php $a = [1, 2, 3]; $o = new stdClass(); echo $a[$o];",
        "Array index must be integer",
    );
}

/// Verifies that a NON-coercible key (an array-typed expression) used to index a
/// property-array WRITE still fires "Array index must be integer" after the
/// write-path key check was widened to mirror the read path. Guards the genuine-error
/// boundary: only PHP-coercible key types (int/string/mixed/bool/float/null-like and
/// unions thereof) are accepted; heap container types remain rejected.
#[test]
fn test_error_property_array_write_array_key_is_rejected() {
    expect_error(
        "<?php class C { public array $p = []; } $c = new C(); $k = [1, 2]; $c->p[$k] = 1;",
        "Array index must be integer",
    );
}

/// Verifies that assigning to a string offset (character replacement) is rejected.
/// Input: `$s = "hello"; $s[0] = "H";` — offset assignment on a string is unsupported.
#[test]
fn test_error_string_offset_assignment_is_not_supported() {
    expect_error(
        "<?php $s = \"hello\"; $s[0] = \"H\";",
        "String offset assignment is not supported",
    );
}

/// Verifies gradual `mixed` support for `yield from` does not admit a statically concrete scalar
/// operand, which remains a compile-time type error instead of reaching iterator lowering.
#[test]
fn test_error_yield_from_rejects_concrete_integer() {
    expect_error(
        "<?php function invalidDelegate(): iterable { yield from 42; }",
        "yield from expects an array or Generator",
    );
}

/// Verifies that by-reference foreach over a parameter typed `iterable` is rejected.
/// Input: `function f(iterable $items) { foreach ($items as &$value) {} }`
#[test]
fn test_error_by_reference_foreach_rejects_iterable_type() {
    expect_error(
        "<?php function f(iterable $items) { foreach ($items as &$value) {} }",
        "by-reference foreach over Iterator/IteratorAggregate objects",
    );
}

/// Verifies that by-reference foreach over a parameter typed `Iterator` is rejected.
/// Input: `function f(Iterator $items) { foreach ($items as &$value) {} }`
#[test]
fn test_error_by_reference_foreach_rejects_iterator_object_type() {
    expect_error(
        "<?php function f(Iterator $items) { foreach ($items as &$value) {} }",
        "by-reference foreach over Iterator/IteratorAggregate objects",
    );
}

/// Verifies that by-reference foreach over a concrete class implementing `Iterator` is rejected.
/// Uses a `Counter` class that implements Iterator with an int counter field.
#[test]
fn test_error_by_reference_foreach_rejects_concrete_iterator_object() {
    expect_error(
        r#"<?php
class Counter implements Iterator {
    private int $i = 0;
    public function rewind(): void { $this->i = 0; }
    public function valid(): bool { return $this->i < 3; }
    public function current(): mixed { return $this->i; }
    public function key(): mixed { return $this->i; }
    public function next(): void { $this->i = $this->i + 1; }
}
foreach (new Counter() as &$value) {}
"#,
        "by-reference foreach over Iterator/IteratorAggregate objects",
    );
}

/// Verifies that by-reference foreach over a concrete class implementing `IteratorAggregate` is rejected.
/// Uses a `Counters` class that returns a `Counter` iterator via `getIterator()`.
#[test]
fn test_error_by_reference_foreach_rejects_iterator_aggregate_object() {
    expect_error(
        r#"<?php
class Counter implements Iterator {
    private int $i = 0;
    public function rewind(): void { $this->i = 0; }
    public function valid(): bool { return $this->i < 3; }
    public function current(): mixed { return $this->i; }
    public function key(): mixed { return $this->i; }
    public function next(): void { $this->i = $this->i + 1; }
}
class Counters implements IteratorAggregate {
    public function getIterator(): Traversable { return new Counter(); }
}
foreach (new Counters() as &$value) {}
"#,
        "by-reference foreach over Iterator/IteratorAggregate objects",
    );
}

/// Verifies that a union-typed local variable rejects an initializer of an incompatible type.
/// Input: `int|string $value = 1.5;` — float is not int or string.
#[test]
fn test_error_union_typed_local_rejects_invalid_initializer() {
    expect_error("<?php int|string $value = 1.5;", "cannot initialize $value");
}

/// Verifies a boxed `mixed` value is gradually accepted at an object parameter boundary.
/// The campaign's gradual-typing rule (`Mixed` is call-compatible with every parameter
/// type) defers the object check to runtime, matching how framework code passes `mixed`
/// containers into typed APIs.
#[test]
fn test_error_mixed_rejected_at_object_parameter_boundary() {
    expect_ok(
        "<?php final class Box {} function take(Box $box): void {} function relay(mixed $value): void { take($value); }",
    );
}

/// Verifies a boxed `mixed` value is gradually accepted through an array return boundary
/// under the same gradual-typing rule as parameter boundaries.
#[test]
fn test_error_mixed_rejected_at_array_return_boundary() {
    expect_ok("<?php function relay(mixed $value): array { return $value; }");
}

/// Verifies that referencing an undefined variable produces an "Undefined variable" error.
#[test]
fn test_error_undefined_variable() {
    expect_error("<?php echo $x;", "Undefined variable: $x");
}

/// Verifies that walking an `isset()` operand's always-evaluated index expression for
/// assignment effects (so `isset($a[$h = f()])` defines `$h`) does not broaden definite
/// assignment beyond that index: a genuinely undefined, unrelated variable read after the
/// `isset()` call must still error loudly.
#[test]
fn test_error_isset_index_assignment_does_not_define_unrelated_variable() {
    expect_error(
        r#"<?php
$a = [];
if (isset($a[$k = 1])) {}
echo $other;
"#,
        "Undefined variable: $other",
    );
}

/// Verifies the same scoping restriction for `unset()`: an assignment inside the index defines
/// only that variable, not an unrelated genuinely-undefined one read afterward.
#[test]
fn test_error_unset_index_assignment_does_not_define_unrelated_variable() {
    expect_error(
        r#"<?php
$a = [1, 2, 3];
unset($a[$k = 1]);
echo $other;
"#,
        "Undefined variable: $other",
    );
}

/// Regression for the JURY ADDENDUM #2 finding: `PDOStatement::bindParam()`'s second parameter
/// is intentionally NOT declared by-reference (see `pdo_prelude.rs`), because `PDO::prepare()`
/// returns `PDOStatement|bool` and the codegen by-reference argument materializer for a
/// union/register-held receiver either loudly rejects the scalar-to-Mixed promotion or silently
/// miscompiles an already-auto-vivified variable into a runtime crash. A previously-undefined
/// variable passed to `bindParam()` must therefore still be reported loudly at compile time
/// instead of auto-vivifying into an unsound codegen path.
#[test]
fn test_error_pdo_statement_bind_param_does_not_autovivify_undefined_variable() {
    expect_error(
        r#"<?php
$pdo = new PDO('sqlite::memory:');
$stmt = $pdo->prepare('SELECT :id');
$stmt->bindParam(':id', $id);
"#,
        "Undefined variable: $id",
    );
}

/// Verifies that a variable assigned in one `match` arm's body is not visible in a sibling
/// arm's body: only one arm body ever runs, so the assignment is not definitely-assigned in
/// the other arms. Reading it in a sibling arm must still error.
#[test]
fn test_error_match_arm_body_assignment_not_visible_in_sibling_arm() {
    expect_error(
        r#"<?php
function g(int $n): string {
    return match ($n) {
        1 => ($x = 'a'),
        2 => $x,
        default => 'd',
    };
}"#,
        "Undefined variable: $x",
    );
}

/// Verifies that reassigning an *inferred* local to an incompatible type is accepted under the
/// gradual-typing model: the local widens to a boxed union instead of erroring.
/// Input: `$x = 42; $x = "hello";` — `$x` widens from `Int` to `Union([Int, Str])`.
/// Verifies that a plain self-referential assignment is not mistaken for `+=`.
#[test]
fn test_error_plain_self_read_assignment_remains_undefined() {
    expect_error("<?php $x = $x + 1;", "Undefined variable: $x");
}

/// Verifies that reassigning a typed variable to a different type is rejected.
/// Input: `$x = 42; $x = "hello";` — `$x` is int, reassignment to string fails.
#[test]
fn test_inferred_local_reassign_widens_instead_of_error() {
    assert!(
        check_source("<?php $x = 42; $x = \"hello\"; echo $x;").is_ok(),
        "reassigning an inferred local to an incompatible type should widen, not reject",
    );
}

/// Verifies that arithmetic on a string operand produces an error.
/// Input: `$x = "hi"; echo $x + 1;` — string is not numeric.
#[test]
fn test_error_arithmetic_on_string() {
    expect_error(
        "<?php $x = \"hi\"; echo $x + 1;",
        "Arithmetic operators require numeric operands",
    );
}

/// Verifies a name beginning with `with` does not imply a late-static fluent return.
#[test]
fn test_error_with_prefix_does_not_refine_declared_ancestor_return() {
    expect_error(
        r#"<?php
interface Account {
    public function withdraw(int $amount): Account;
}
interface Savings extends Account {
    public function interestRate(): int;
}
final class SavingsAccount implements Savings {
    public function withdraw(int $amount): Account { return $this; }
    public function interestRate(): int { return 4; }
}
function rate(Savings $account): int {
    return $account->withdraw(10)->interestRate();
}
echo rate(new SavingsAccount());
"#,
        "Undefined method: Account::interestRate",
    );
}

/// Verifies that binding `static` preserves distinct explicit union members.
///
/// `static|Choice` called on `SpecialChoice` becomes `SpecialChoice|Choice`. Union-receiver
/// method dispatch is PHP-faithfully lenient (a call resolves as long as at least one member
/// declares the method), so a subclass-only method is accepted; the union's preservation is
/// instead observable in the no-member diagnostic, which must name BOTH members. A collapse
/// to a single member would report `SpecialChoice::missing` instead.
#[test]
fn test_error_late_static_union_keeps_explicit_ancestor_member() {
    expect_error(
        r#"<?php
class Choice {
    public function choose(bool $same): static|Choice {
        return $same ? $this : new Choice();
    }
}
class SpecialChoice extends Choice {
    public function special(): string { return "special"; }
}
function render(SpecialChoice $choice): string {
    return $choice->choose(false)->missing();
}
"#,
        "Undefined method: SpecialChoice|Choice::missing",
    );
}

/// Verifies an interface `static` contract cannot be implemented as the concrete class name.
#[test]
fn test_error_interface_static_return_requires_late_static_implementation() {
    expect_error(
        r#"<?php
interface CreatesLateBound {
    public function create(): static;
}
class ConcreteCreator implements CreatesLateBound {
    public function create(): ConcreteCreator { return $this; }
}
"#,
        "incompatible return type",
    );
}

/// Verifies overriding `static` with the immediate child name is rejected for future subclasses.
#[test]
fn test_error_static_return_override_cannot_become_concrete_child() {
    expect_error(
        r#"<?php
class LateBoundBase {
    public function copy(): static { return $this; }
}
class ConcreteCopy extends LateBoundBase {
    public function copy(): ConcreteCopy { return $this; }
}
"#,
        "incompatible return type",
    );
}

/// Verifies a child interface must preserve its parent's late-static return contract.
#[test]
fn test_error_interface_redeclaration_cannot_replace_static_with_child_name() {
    expect_error(
        r#"<?php
interface LateBoundContract {
    public function copy(): static;
}
interface ConcreteContract extends LateBoundContract {
    public function copy(): ConcreteContract;
}
"#,
        "compatible late-static return type",
    );
}

/// Verifies that negating a non-numeric string produces an error.
/// Input: `$x = "hi"; echo -$x;`
#[test]
fn test_error_negate_string() {
    expect_error(
        "<?php $x = \"hi\"; echo -$x;",
        "Cannot negate a non-numeric value",
    );
}

/// Verifies that ordered comparison operators on an array operand produce an error.
/// PHP 8 allows string/numeric ordered comparison (lowered through `__rt_php_compare`),
/// but array/object ordering stays rejected. Input: `$x = [1, 2]; echo $x < 1;`.
#[test]
fn test_error_comparison_on_array() {
    expect_error(
        "<?php $x = [1, 2]; echo $x < 1;",
        "Comparison operators require numeric or string operands",
    );
}

/// Verifies that `xor` with no right-hand side produces an "Unexpected token" error.
#[test]
fn test_error_word_logical_missing_rhs() {
    expect_error("<?php echo true xor;", "Unexpected token: Semicolon");
}

/// Verifies that an assignment expression with a non-lvalue target is rejected.
/// Input: `echo 1 = 2;` — 1 is not a valid assignment target.
#[test]
fn test_error_assignment_expression_rejects_non_lvalue() {
    expect_error("<?php echo 1 = 2;", "Invalid assignment target");
}

/// Verifies that the short ternary (`?:`) with no default expression produces an error.
#[test]
fn test_error_short_ternary_missing_default() {
    expect_error("<?php echo $x ?:;", "Unexpected token: Semicolon");
}

/// Verifies that `break` outside any loop or switch produces an error.
#[test]
fn test_error_break_outside_loop_or_switch() {
    expect_error("<?php break;", "Cannot 'break' 1 levels");
}

/// Verifies that `break N` with N exceeding the available nesting levels produces an error.
#[test]
fn test_error_break_too_many_levels() {
    expect_error("<?php while (1) { break 2; }", "Cannot 'break' 2 levels");
}

/// Verifies that `continue N` with N exceeding available loop nesting produces an error.
#[test]
fn test_error_continue_too_many_levels() {
    expect_error(
        "<?php while (1) { continue 2; }",
        "Cannot 'continue' 2 levels",
    );
}

/// Verifies that `break` inside a `finally` block cannot jump out of the finally.
#[test]
fn test_error_break_cannot_jump_out_of_finally() {
    expect_error(
        "<?php while (1) { try { echo 1; } finally { break; } }",
        "Cannot jump out of a finally block",
    );
}

/// Verifies that `continue` inside a `finally` block cannot jump out of the finally.
#[test]
fn test_error_continue_cannot_jump_out_of_finally() {
    expect_error(
        "<?php while (1) { try { echo 1; } finally { continue; } }",
        "Cannot jump out of a finally block",
    );
}

/// Verifies that a multi-level `break N` inside a `finally` block cannot jump out of the finally.
#[test]
fn test_error_multilevel_break_cannot_jump_out_of_finally() {
    expect_error(
        "<?php while (1) { try { echo 1; } finally { while (1) { break 2; } } }",
        "Cannot jump out of a finally block",
    );
}

/// Verifies that calling an undefined function produces an error.
#[test]
fn test_error_undefined_function() {
    expect_error("<?php nope();", "Undefined function: nope");
}

/// Verifies that passing too many arguments to a user-defined function is rejected.
#[test]
fn test_error_wrong_arg_count() {
    expect_error(
        "<?php function f($a) { return $a; } f(1, 2);",
        "expects 1 arguments, got 2",
    );
}

/// Verifies that increment/decrement on a string is rejected.
#[test]
fn test_error_increment_string() {
    expect_error("<?php $x = \"hi\"; $x++;", "Cannot increment/decrement");
}

/// Verifies the kind predicates `is_array`/`is_object`/`is_scalar` reject a wrong argument
/// count, matching the other single-argument type predicates.
#[test]
fn test_error_is_kind_predicates_arity() {
    expect_error(
        "<?php is_array([1], [2]);",
        "is_array() takes exactly 1 argument",
    );
    expect_error("<?php is_object();", "is_object() takes exactly 1 argument");
    expect_error(
        "<?php is_scalar(1, 2, 3);",
        "is_scalar() takes exactly 1 argument",
    );
}

// --- Error positions ---

/// Verifies that the null coalesce operator widens the inferred return type to float
/// when one branch is int and the other is a float literal.
/// Input: `function fallback_pi($x) { return $x ?? 3.14159; }`
#[test]
fn test_null_coalesce_widens_function_return_type_in_checker() {
    let tokens = tokenize("<?php function fallback_pi($x) { return $x ?? 3.14159; }")
        .expect("tokenize failed");
    let ast = parse(&tokens).expect("parse failed");
    let ast = elephc::optimize::fold_constants(ast);
    let check_result = types::check(&ast).expect("type check failed");

    let sig = check_result
        .functions
        .get("fallback_pi")
        .expect("missing function signature for fallback_pi");
    assert_eq!(sig.return_type, PhpType::Float);

    // Verifies that `array` return hints preserve the element type through property storage
    // and method return inference, using a `Wad` class with `Entry` objects.








    // Verifies that `array` parameter and return hints preserve string element types
    // through a chain of `paint`, `pickSecond`, and `loadNames`.





}

/// Verifies generic array return hint keeps specific method and property types.
#[test]
fn test_generic_array_return_hint_keeps_specific_method_and_property_types() {
    let result = check_source_full(
        r#"<?php
class Entry {
    public $name;

    public function __construct($name) {
        $this->name = $name;
    }
}

class Wad {
    public $entries;

    public function __construct() {
        $this->entries = $this->loadEntries();
    }

    public function loadEntries(): array {
        return [new Entry("PLAYPAL"), new Entry("COLORMAP")];
    }

    public function secondName(): string {
        $i = 1;
        return $this->entries[$i]->name;
    }
}
"#,
    )
    .expect("expected source to type-check");

    let wad = result.classes.get("Wad").expect("missing Wad class");
    let entries_ty = wad
        .properties
        .iter()
        .find(|(name, _)| name == "entries")
        .map(|(_, ty)| ty.clone())
        .expect("missing entries property");
    assert_eq!(
        entries_ty,
        PhpType::Array(Box::new(PhpType::Object("Entry".to_string())))
    );

    let load_entries = wad
        .methods
        .get(&elephc::names::php_symbol_key("loadEntries"))
        .expect("missing loadEntries");
    assert_eq!(
        load_entries.return_type,
        PhpType::Array(Box::new(PhpType::Object("Entry".to_string())))
    );
}

/// Verifies generic array param and return hints keep specific string array types.
#[test]
fn test_generic_array_param_and_return_hints_keep_specific_string_array_types() {
    let result = check_source_full(
        r#"<?php
function paint(string $name): string {
    return $name;
}

function pickSecond(array $names): string {
    return paint($names[1]);
}

function loadNames(): array {
    return ["foo", "bar"];
}

echo pickSecond(loadNames());
"#,
    )
    .expect("expected source to type-check");

    let pick_second = result
        .functions
        .get("pickSecond")
        .expect("missing pickSecond signature");
    assert_eq!(
        pick_second.params[0].1,
        PhpType::Array(Box::new(PhpType::Str))
    );

    let load_names = result
        .functions
        .get("loadNames")
        .expect("missing loadNames signature");
    assert_eq!(load_names.return_type, PhpType::Array(Box::new(PhpType::Str)));
}

// --- Include/Require errors ---

/// Verifies that passing more arguments than a function with optional parameters accepts is rejected.
/// Input: `function f($a, $b = 1) { return $a + $b; } f(1, 2, 3);`
#[test]
fn test_error_too_many_args_with_defaults() {
    expect_error(
        "<?php function f($a, $b = 1) { return $a + $b; } f(1, 2, 3);",
        "expects 1 to 2 arguments, got 3",
    );
}

/// Verifies that passing fewer arguments than a function with optional parameters requires is rejected.
/// Input: `function f($a, $b = 1) { return $a + $b; } f();`
#[test]
fn test_error_too_few_args_with_defaults() {
    expect_error(
        "<?php function f($a, $b = 1) { return $a + $b; } f();",
        "expects 1 to 2 arguments, got 0",
    );
}

/// Verifies that a promoted constructor parameter with a type mismatch is rejected.
/// Input: `class Box { public function __construct(public int $value) {} } new Box("bad");`
#[test]
fn test_error_promoted_property_type_mismatch() {
    expect_error(
        r#"<?php
class Box {
    public function __construct(public int $value) {}
}
$box = new Box("bad");
"#,
        "Constructor 'Box::__construct' parameter $value expects Int, got Str",
    );
}

/// Verifies that an unrelated object default is rejected after class relationships are known.
#[test]
fn test_error_promoted_property_rejects_incompatible_object_default() {
    expect_error(
        r#"<?php
class Expected {}
class Unrelated {}
class Box {
    public function __construct(public Expected $value = new Unrelated()) {}
}
"#,
        "Method parameter $value expects Object(\"Expected\"), got Object(\"Unrelated\")",
    );
}

/// Verifies an enum-typed parameter default rejects a missing enum case semantically.
#[test]
fn test_error_enum_case_parameter_default_rejects_missing_case() {
    expect_error(
        r#"<?php
enum A {
    case One;
}
function unused_enum_default(A $a = A::Nope): void {}
"#,
        "Undefined enum case: A::Nope",
    );
}

/// Verifies a scalar class constant cannot default an object-typed parameter.
#[test]
fn test_error_object_parameter_default_rejects_scalar_class_constant() {
    expect_error(
        r#"<?php
class Foo {
    public const BAR = 1;
}
function unused_class_constant_default(Foo $value = Foo::BAR): void {}
"#,
        "Function 'unused_class_constant_default' parameter $value expects Object(\"Foo\"), got Int",
    );
}

/// Verifies plain property enum case defaults remain outside the supported EIR surface.
#[test]
fn test_error_plain_property_enum_case_default_remains_unsupported() {
    expect_error(
        r#"<?php
enum Level {
    case Low;
}
class Config {
    public Level $level = Level::Low;
}
"#,
        "Property Config::$level default expects Object(\"Level\"), got Str",
    );
}

/// Verifies that assigning an incompatible value to a static property is rejected.
/// Input: `class Box { public static int $count = 1; } Box::$count = "x";`
#[test]
fn test_error_static_property_type_mismatch() {
    expect_error(
        "<?php class Box { public static int $count = 1; } Box::$count = \"x\";",
        "Static property Box::$count expects",
    );
}

/// Verifies that a child class static property redeclared with an incompatible type is rejected.
/// Input: `class Base { public static int $count = 1; } class Child extends Base { public static string $count = "x"; }`
#[test]
fn test_error_static_property_redeclaration_type_mismatch() {
    expect_error(
        "<?php class Base { public static int $count = 1; } class Child extends Base { public static string $count = \"x\"; }",
        "Type of Child::$count must be int, not string (as in class Base)",
    );
}

/// Verifies that `date()` with too many arguments is rejected.
#[test]
fn test_error_date_too_many_args() {
    expect_error(r#"<?php date("Y", 0, 0);"#, "date() takes 1 or 2 arguments");
}

/// Verifies that `json_encode()` flags argument must be int (not string).
#[test]
fn test_error_json_encode_flag_must_be_int() {
    expect_error(
        r#"<?php json_encode("a", "b");"#,
        "json_encode() flags and depth must be integers",
    );
}

/// Verifies that `json_encode()` depth argument must be int (not string).
#[test]
fn test_error_json_encode_depth_must_be_int() {
    expect_error(
        r#"<?php json_encode("a", 0, "deep");"#,
        "json_encode() flags and depth must be integers",
    );
}

/// Verifies that `json_encode()` with too many arguments is rejected.
#[test]
fn test_error_json_encode_too_many_args() {
    expect_error(
        "<?php json_encode(1, 2, 3, 4);",
        "json_encode() takes 1 to 3 arguments",
    );
}

/// Verifies that `json_decode()` with too many arguments is rejected.
#[test]
fn test_error_json_decode_too_many_args() {
    expect_error(
        r#"<?php json_decode("1", true, 1, 0, 99);"#,
        "json_decode() takes 1 to 4 arguments",
    );
}

/// Verifies that `json_decode()` requires a string-compatible first argument (array is rejected).
#[test]
fn test_error_json_decode_json_arg_must_be_string_compatible() {
    expect_error(
        r#"<?php json_decode([]);"#,
        "json_decode() json argument must be string-compatible",
    );
}

/// Verifies that `json_decode()` associative argument must be bool-compatible or null (array is rejected).
#[test]
fn test_error_json_decode_associative_must_be_bool_compatible() {
    expect_error(
        r#"<?php json_decode("{}", []);"#,
        "json_decode() associative argument must be bool-compatible or null",
    );
}

/// Verifies that `json_decode()` depth argument must be int (not string).
#[test]
fn test_error_json_decode_depth_must_be_int() {
    expect_error(
        r#"<?php json_decode("{}", false, "deep");"#,
        "json_decode() depth and flags must be integers",
    );
}

/// Verifies that `json_decode()` flags argument must be int (not string).
#[test]
fn test_error_json_decode_flags_must_be_int() {
    expect_error(
        r#"<?php json_decode("{}", false, 512, "flags");"#,
        "json_decode() depth and flags must be integers",
    );
}

/// Verifies that `json_validate()` with too many arguments is rejected.
#[test]
fn test_error_json_validate_too_many_args() {
    expect_error(
        r#"<?php json_validate("1", 1, 0, 99);"#,
        "json_validate() takes 1 to 3 arguments",
    );
}

/// Verifies that `json_validate()` requires a string-compatible first argument (array is rejected).
#[test]
fn test_error_json_validate_json_arg_must_be_string_compatible() {
    expect_error(
        r#"<?php json_validate([]);"#,
        "json_validate() json argument must be string-compatible",
    );
}

/// Verifies that `json_validate()` depth argument must be int (not string).
#[test]
fn test_error_json_validate_flag_must_be_int() {
    expect_error(
        r#"<?php json_validate("1", "deep");"#,
        "json_validate() depth and flags must be integers",
    );
}

/// Verifies that `json_validate()` rejects `JSON_THROW_ON_ERROR` in flags.
#[test]
fn test_error_json_validate_rejects_throw_on_error_flag() {
    expect_error(
        r#"<?php json_validate("1", 512, JSON_THROW_ON_ERROR);"#,
        "json_validate() flags must be 0 or JSON_INVALID_UTF8_IGNORE",
    );
}

/// Verifies that `json_validate()` rejects combined flags mixing invalid values.
#[test]
fn test_error_json_validate_rejects_combined_invalid_flags() {
    expect_error(
        r#"<?php json_validate("1", 512, JSON_INVALID_UTF8_IGNORE | JSON_THROW_ON_ERROR);"#,
        "json_validate() flags must be 0 or JSON_INVALID_UTF8_IGNORE",
    );
}

/// Verifies that `sin()` with more than 1 argument is rejected.
#[test]
fn test_error_sin_too_many_args() {
    expect_error("<?php sin(1, 2);", "sin() takes exactly 1 argument");
}

/// Verifies that `log()` with more than 2 arguments is rejected.
#[test]
fn test_error_log_too_many_args() {
    expect_error("<?php log(1, 2, 3);", "log() takes 1 or 2 arguments");
}

/// Verifies that a closure `use()` clause referencing an undefined variable is rejected.
#[test]
fn test_error_closure_use_undefined_variable() {
    expect_error(
        r#"<?php
$fn = function() use ($undefined) { echo $undefined; };
"#,
        "Undefined variable in use(): $undefined",
    );
}

// --- Pointer error tests ---

/// Verifies that loose pointer comparison (`==` or `!=`) is rejected; only `===`/`!==` are allowed.
/// Input: `$p = ptr($x); $q = ptr($x); echo $p == $q;`
#[test]
fn test_error_pointer_loose_comparison_is_rejected() {
    expect_error(
        "<?php $x = 1; $p = ptr($x); $q = ptr($x); echo $p == $q;",
        "Loose pointer comparison is not supported; use === or !==",
    );
}

// --- FFI error tests ---

/// Verifies that using `$this` inside a static closure via a short ternary expression is rejected.
/// Input: `class C { public int $count = 5; public function bad() { $f = static fn($x) => $x ?: $this->count; } }`
#[test]
fn test_error_static_closure_uses_this_through_short_ternary() {
    expect_error(
        "<?php class C { public int $count = 5; public function bad() { $f = static fn($x) => $x ?: $this->count; return $f; } }",
        "Cannot use $this inside a static closure",
    );
}

/// Verifies that combining the nullable shorthand `?T` with a pipe union is rejected, and
/// that the diagnostic points the user at the now-supported `T|null` spelling.
#[test]
fn test_error_nullable_shorthand_with_union() {
    expect_error(
        "<?php function f(): ?int|string { return 1; }",
        "Nullable shorthand cannot be combined directly with union types; write T|null",
    );
}

/// Verifies that a union type with a trailing pipe and no following member is rejected with
/// the type-expression diagnostic, confirming `null`/`false`/`true` did not loosen the
/// requirement that every pipe be followed by a real type.
#[test]
fn test_error_union_trailing_pipe() {
    expect_error(
        "<?php function f(): int| { return 1; }",
        "Expected type expression",
    );
}

/// Verifies that the relative class type `self` is rejected when used as a type outside of any
/// class body (a free function), where it has no enclosing class to resolve to.
#[test]
fn test_error_self_type_outside_class() {
    expect_error(
        "<?php function f(): self { return 1; }",
        "Cannot use 'self' as a type outside of a class",
    );
}

/// Verifies that `static` is likewise rejected as a free-function parameter type.
#[test]
fn test_error_static_type_outside_class() {
    expect_error(
        "<?php function f(static $x): int { return 1; }",
        "Cannot use 'static' as a type outside of a class",
    );
}

/// Verifies that variable variables (`$$name`) are rejected with an explanatory message, since
/// elephc allocates locals to fixed compile-time slots with no runtime variable-name table.
#[test]
fn test_error_variable_variables_unsupported() {
    expect_error(
        "<?php $x = \"y\"; $$x = 1;",
        "Variable variables (`$$name`) are not supported",
    );
}

/// Verifies the local variable-variable expression form `${$name}` still reports the preserved
/// unsupported-variable-variable diagnostic, now that the lexer emits a bare `$` token and the
/// parser (not the lexer) owns the rejection outside a static-receiver `::` context.
#[test]
fn test_error_variable_variable_brace_form_unsupported() {
    expect_error(
        "<?php $x = \"y\"; echo ${$x};",
        "Variable variables (`$$name`) are not supported",
    );
}

/// Verifies a dynamic static property read on a class that is not statically known
/// (`Nonexistent::${$n}`) is a loud error, since codegen cannot enumerate candidate static
/// properties without a resolvable class.
#[test]
fn test_error_dynamic_static_property_unknown_class() {
    expect_error(
        "<?php $n = 'x'; echo Nonexistent::${$n};",
        "Dynamic static property access requires a statically-known class",
    );
}

/// Verifies a WRITE through a dynamic-named static property on a class that is not statically
/// known is a loud deferred error (the receiver class must be resolvable so codegen can enumerate
/// candidate static properties), mirroring the read-side rejection.
#[test]
fn test_error_dynamic_static_property_write_unknown_class() {
    expect_error(
        "<?php $n = 'x'; Nonexistent::${$n} = 5;",
        "Dynamic static property access requires a statically-known class",
    );
}

/// Verifies that the nullable shorthand cannot be combined with an intersection type (`?A&B`),
/// which is a syntax error in PHP. Previously this silently parsed and dropped a member.
#[test]
fn test_error_nullable_intersection_type_rejected() {
    assert!(
        check_source("<?php interface A {} interface B {} function f(?A&B $x): int { return 1; }")
            .is_err(),
        "?A&B should be rejected, not silently accepted",
    );
}

/// Verifies the gradual-typing boundary model still rejects a statically-`int` argument flowing
/// into a `string` parameter: a concrete source type that is disjoint from the target is a real
/// type error and must NOT be loosened (only Mixed/union sources are accepted gradually).
#[test]
fn test_error_concrete_int_into_string_param_still_rejected() {
    expect_error(
        "<?php function f(string $s) {} f(5);",
        "parameter $s expects Str, got Int",
    );
}

/// Verifies a concrete `array` argument flowing into an unrelated class parameter is still a real
/// type error under the gradual model (array is not Mixed and the target is not in any union).
#[test]
fn test_error_concrete_array_into_class_param_still_rejected() {
    expect_error(
        "<?php class C {} function f(C $c) {} f([1, 2]);",
        "parameter $c expects",
    );
}

/// Verifies a concrete `bool` argument flowing into a class parameter is still a real type error;
/// the gradual loosening only applies to Mixed sources and union-containing-target shapes.
#[test]
fn test_error_concrete_bool_into_class_param_still_rejected() {
    expect_error(
        "<?php class C {} function f(C $c) {} f(true);",
        "parameter $c expects",
    );
}

/// Verifies the gradual model accepts a `Mixed` source (associative-array read) flowing into a
/// `string` parameter — the boundary the type checker previously rejected. This is the positive
/// counterpart to the concrete-disjoint rejections above.
#[test]
fn test_gradual_mixed_into_string_param_accepted() {
    assert!(
        check_source(
            "<?php function f(string $s): string { return $s; } \
             $m = []; $m[\"k\"] = \"hi\"; $v = $m[\"k\"]; echo f($v);"
        )
        .is_ok(),
        "Mixed value should be accepted into a string parameter under gradual typing",
    );
}

/// Verifies the gradual model accepts a local reassigned to an incompatible type (`int` then
/// `string`) instead of reporting "cannot reassign"; the local widens to a boxed union.
#[test]
fn test_gradual_reassign_widening_accepted() {
    assert!(
        check_source("<?php $x = 1; $x = \"a\"; echo $x;").is_ok(),
        "reassigning a local to an incompatible type should widen, not reject",
    );
}

/// Verifies the gradual-typing safety property for `end()`: a concretely non-array argument
/// (`int`) is still rejected rather than accepted, so genuine type errors keep being reported.
#[test]
fn test_error_end_on_non_array_is_rejected() {
    expect_error("<?php end(5);", "end() argument must be array");
}

/// Verifies the gradual-typing safety property for `in_array()`: a concretely non-array haystack
/// (`int`) is still rejected. Only `Mixed`/union-containing-array haystacks are accepted.
#[test]
fn test_error_in_array_non_array_haystack_is_rejected() {
    expect_error("<?php in_array(1, 5);", "in_array() second argument must be array");
}

/// Verifies the gradual-typing safety property for `array_key_exists()`: a concretely non-array
/// second argument (`int`) is still rejected.
#[test]
fn test_error_array_key_exists_non_array_is_rejected() {
    expect_error(
        "<?php array_key_exists(\"k\", 5);",
        "array_key_exists() second argument must be array",
    );
}

/// Verifies the gradual-typing safety property for increment/decrement: a concrete object operand
/// stays a compile error ("Cannot increment/decrement"), so `$obj++` is not silently accepted.
#[test]
fn test_error_increment_on_object_is_rejected() {
    expect_error(
        "<?php $o = new stdClass(); $o++;",
        "Cannot increment/decrement",
    );
}

/// Verifies that a reference INTO an INSTANCE-property array element (`$o->arr[$k] = &$src`) still
/// hard-errors: the local-array-element form (`$a[$k] = &$src`) is implemented, but a
/// property-base element target is a follow-up slice and must loud-error rather than miscompile.
#[test]
fn test_error_ref_assign_into_property_array_element_unsupported() {
    expect_error(
        "<?php class C { public array $arr = []; } \
         $o = new C(); $s = 5; $o->arr[\"k\"] = &$s;",
        "Reference assignment into an array element is not supported",
    );
}

/// A reference to a static-property array element (`$x = &self::$arr[$k]`, the former
/// SLICE 3 deferral) is now supported end-to-end: the alias write-through is covered
/// behaviorally by the references codegen tests; here the shape must simply type-check.
#[test]
fn test_ref_assign_static_property_array_element_supported() {
    expect_ok(
        "<?php class C { public static $a = [1, 2]; \
         static function t() { $x = &self::$a[0]; return $x; } }",
    );
}

/// SLICE 2/3: binding a static-property array element by reference (`self::$a[$d] = &self::$a[$k]`)
/// against an UNKNOWN class must loud-error at the checker (via
/// `resolve_static_property_assignment_target`) rather than miscompile the aliasing.
#[test]
fn test_error_ref_assign_static_prop_element_unknown_class() {
    expect_error(
        "<?php U::$a[$d] = &U::$a[$k];",
        "Undefined class: U",
    );
}

/// SLICE 2/3: the reference SOURCE for a static-property array element must itself be a
/// static-property array element. A plain-variable source (`self::$a[$d] = &$local`) is a follow-up
/// slice and must loud-error, not silently become a value copy.
#[test]
fn test_error_ref_assign_static_prop_element_non_static_source() {
    expect_error(
        "<?php class C { public static array $a = []; \
         static function t() { $x = 1; self::$a[\"d\"] = &$x; } }",
        "Reference source for a static-property array element must be another static-property array element",
    );
}

/// SLICE 2/3: aliasing across TWO DIFFERENT static-property arrays (`self::$a[$d] = &self::$b[$k]`)
/// is a follow-up slice; only the same-array GATE is supported, so the cross-array form loud-errors.
#[test]
fn test_error_ref_assign_static_prop_element_cross_array_source() {
    expect_error(
        "<?php class C { public static array $a = []; public static array $b = []; \
         static function t() { $k = \"k\"; $d = \"d\"; self::$a[$d] = &self::$b[$k]; } }",
        "Reference between two different static-property arrays is not yet supported",
    );
}

/// SLICE 2/3: a NON-array static property cannot back a reference-into-element alias; the checker
/// rejects `self::$n[0] = &self::$n[1]` on a scalar static property loudly.
#[test]
fn test_error_ref_assign_static_prop_element_non_array_property() {
    expect_error(
        "<?php class C { public static int $n = 0; \
         static function t() { self::$n[0] = &self::$n[1]; } }",
        "Reference assignment into a static-property array element requires an array static property",
    );
}

/// Verifies that a by-reference assignment in expression position (`if (null !== $x = &$a[$k])`,
/// the SLICE 4 form) is a parse error: `=&` is statement-only, so the `&` is rejected rather than
/// silently accepted as a bitwise operator.
#[test]
fn test_error_ref_assign_in_expression_position_unsupported() {
    expect_error(
        "<?php $a = [\"k\" => 1]; if (null !== $x = &$a[\"k\"]) { echo \"y\"; }",
        "Unexpected token",
    );
}

/// Verifies that a reference to a DYNAMIC-named property on an untyped/`Mixed` receiver (`$x =
/// &$obj->$p` where `$obj` has no static class) loud-errors instead of silently miscompiling: the
/// SLICE 5 runtime-name dispatch needs a concrete class to enumerate promotable slots.
#[test]
fn test_error_ref_dynamic_property_on_mixed_receiver_unsupported() {
    expect_error(
        "<?php function f($obj, $p) { $x = &$obj->$p; return $x; }",
        "Reference to a dynamic property is only supported on a statically-typed object receiver",
    );
}

/// SLICE 2 (local): the reference SOURCE for a local array-element reference must be a plain
/// variable (`&$x`). Aliasing an array element (`$a[] = &$b[$j]`) is a follow-up slice and is
/// rejected loudly rather than silently value-copied.
#[test]
fn test_error_ref_local_array_append_non_variable_source() {
    expect_error(
        "<?php $a = []; $b = [1, 2]; $a[] = &$b[0];",
        "Reference source for a local array-element reference must be a plain variable",
    );
}

/// SLICE 2 (local): a string-VALUED source cannot back a reference-into-element alias — a kind-6
/// reference cell holds a single machine word, so a two-word `{ptr,len}` string would drop its
/// length. `$a[] = &$s` for a string `$s` is rejected loudly (mirrors the SLICE-1 string guard).
#[test]
fn test_error_ref_local_array_append_string_source() {
    expect_error(
        "<?php $a = []; $s = \"x\"; $a[] = &$s;",
        "Reference to a string-valued source in a local array element is not yet supported",
    );
}

/// SLICE 2 (local): appending a reference into a STATIC-property array (`self::$a[] = &$x`) is a
/// follow-up slice, not the local-array-element form; the checker rejects it loudly.
#[test]
fn test_error_ref_local_array_append_into_static_property() {
    expect_error(
        "<?php class C { public static array $a = []; \
         static function t() { $x = 1; self::$a[] = &$x; } }",
        "Appending a reference into a static or instance property array is not supported",
    );
}

/// SLICE 2 (local): appending a reference into an INSTANCE-property array (`$o->p[] = &$x`) is a
/// follow-up slice, not the local-array-element form; the checker rejects it loudly.
#[test]
fn test_error_ref_local_array_append_into_instance_property() {
    expect_error(
        "<?php class C { public array $p = []; } \
         $o = new C(); $x = 1; $o->p[] = &$x;",
        "Appending a reference into a static or instance property array is not supported",
    );
}

/// Verifies a PHP 8.2 DNF group with a single type and no intersection (`(A)|B`) is rejected, as
/// PHP requires at least one `&` inside the parentheses.
#[test]
fn test_error_dnf_single_type_group_rejected() {
    expect_error(
        "<?php interface A {} interface B {} function s((A)|B $x): void {}",
        "A parenthesized DNF type group must contain an intersection",
    );
}

/// Verifies an unclosed DNF group (`(A&B` with no `)`) is rejected loudly at the missing paren.
#[test]
fn test_error_dnf_unclosed_group_rejected() {
    expect_error(
        "<?php interface A {} interface B {} function u((A&B $x): void {}",
        "Expected ')' to close DNF intersection type group",
    );
}

/// Verifies the `?` nullable shorthand cannot be combined with a DNF group (`?(A&B)`), matching
/// PHP where that is a parse error; nullability must be spelled as the `(A&B)|null` union arm.
#[test]
fn test_error_dnf_nullable_shorthand_rejected() {
    expect_error(
        "<?php interface A {} interface B {} class Z { public ?(A&B) $p = null; }",
        "Nullable shorthand cannot be combined with a DNF type group",
    );
}

/// Verifies a value implementing only one arm of a DNF intersection (`(A&B)|null`, value implements
/// only `B`) is rejected: the intersection is typed as its first member `A`, which the argument
/// does not satisfy.
#[test]
fn test_error_dnf_value_missing_intersection_member_rejected() {
    expect_error(
        "<?php interface A {} interface B {} class OnlyB implements B {} \
         function g((A&B)|null $x): string { return $x === null ? \"null\" : \"obj\"; } \
         echo g(new OnlyB());",
        "parameter $x expects",
    );
}

/// Verifies a nested DNF group with the inner parentheses on the left (`((A&B)&C)`) is rejected
/// in parameter position, matching PHP's hard `syntax error, unexpected token "("` — DNF groups
/// forbid nesting, only a flat `(A&B&C)` intersection is valid.
#[test]
fn test_error_dnf_nested_group_left_param_rejected() {
    expect_error(
        "<?php interface A {} interface B {} interface C {} \
         function f(((A&B)&C) $x): void {}",
        "Nested parentheses are not allowed in a DNF type group",
    );
}

/// Verifies a nested DNF group with the inner parentheses on the right (`(A&(B&C))`) is rejected
/// in parameter position, matching PHP's hard parse error for nested DNF parentheses.
#[test]
fn test_error_dnf_nested_group_right_param_rejected() {
    expect_error(
        "<?php interface A {} interface B {} interface C {} \
         function g((A&(B&C)) $x): void {}",
        "Nested parentheses are not allowed in a DNF type group",
    );
}

/// Verifies a nested DNF group in property position (`((A&B)&C)|null`) is rejected the same way
/// as in parameter position — both type positions share the same DNF-group parser.
#[test]
fn test_error_dnf_nested_group_property_rejected() {
    expect_error(
        "<?php interface A {} interface B {} interface C {} \
         class Z { protected ((A&B)&C)|null $p = null; }",
        "Nested parentheses are not allowed in a DNF type group",
    );
}

// --- Gradual-typing quick-wins batch (throw / arith+comparison / nullsafe+spread / list-unpack) ---

/// R1 boundary: `throw 5;` on a proven non-object stays loud. PHP rejects throwing a
/// scalar with a `TypeError` ("Can only throw objects"), so the checker keeps it loud.
#[test]
fn test_error_throw_scalar_stays_loud() {
    expect_error("<?php throw 5;", "throw requires an object value");
}

/// R1 boundary: throwing a concrete non-Throwable object still reports the specific
/// "implementing Throwable" diagnostic (the gradual relaxation only affects Mixed/union
/// operands, never a proven non-Throwable class).
#[test]
fn test_error_throw_non_throwable_object_stays_loud() {
    expect_error(
        "<?php class Foo {} throw new Foo();",
        "throw requires an object implementing Throwable",
    );
}

/// R1 accept: `throw $mixed` type-checks (a `mixed` value may hold a Throwable at
/// runtime; PHP defers the check to runtime).
#[test]
fn test_gradual_throw_mixed_operand_accepted() {
    expect_ok("<?php function f(mixed $e): void { throw $e; }");
}

/// R2 accept: arithmetic on two `mixed` operands type-checks (gradual numeric dispatch).
#[test]
fn test_gradual_mixed_arithmetic_accepted() {
    expect_ok("<?php function f(mixed $a, mixed $b) { return $a + $b; }");
}

/// R2 accept: arithmetic on a nullable-float union (`?float`) type-checks — every union
/// member is a numeric operand, which the recursive operand check now accepts.
#[test]
fn test_gradual_nullable_float_arithmetic_accepted() {
    expect_ok("<?php function f(?float $a): float { return $a - 1.0; }");
}

/// R2 accept: ordered comparison on two `mixed` operands type-checks.
#[test]
fn test_gradual_mixed_comparison_accepted() {
    expect_ok("<?php function f(mixed $a, mixed $b): bool { return $a < $b; }");
}

/// R2 boundary: arithmetic on a proven array stays loud. PHP fatals on `array - int`
/// ("Unsupported operand types"), so the checker keeps it loud.
#[test]
fn test_error_arithmetic_on_proven_array_stays_loud() {
    expect_error(
        "<?php function f(array $a) { return $a - 1; }",
        "Arithmetic operators require numeric operands",
    );
}

/// R3 accept: a nullsafe method call on a `mixed` receiver type-checks (unknown runtime
/// class → gradual `Mixed` result, mirroring the plain `->` path).
#[test]
fn test_gradual_nullsafe_method_on_mixed_accepted() {
    expect_ok("<?php function f(mixed $x): mixed { return $x?->doThing(); }");
}

/// R3 boundary (correction round 1): spreading a `mixed` value stays loud. The EIR spread
/// lowering unpacks its operand as an array with no runtime array/Traversable guard, so
/// accepting a `mixed` operand here would trade this loud checker error for a runtime
/// SIGSEGV / silent garbage read (`function go(mixed $a){return total(...$a);} go(5)` and a
/// `mixed`-holding-string both corrupted at runtime before this revert). php-check: PHP
/// raises a catchable `TypeError: Only arrays and Traversables can be unpacked` at runtime,
/// so elephc keeping this a compile-time error is conservative, not a regression.
#[test]
fn test_error_spread_mixed_stays_loud() {
    expect_error(
        "<?php function g(int ...$xs): int { return 0; } \
         function f(mixed $args): int { return g(...$args); }",
        "Spread operator requires an array",
    );
}

/// R3 boundary: spreading a proven non-iterable (a bare callable) stays loud. A closure
/// is not Traversable, so PHP fatals ("Only arrays and Traversables can be unpacked").
#[test]
fn test_error_spread_non_iterable_stays_loud() {
    expect_error(
        "<?php $c = fn() => 1; $a = [...$c];",
        "Spread operator requires an array",
    );
}

/// R3 boundary (correction round 1): spreading a union that CAN hold a non-array member
/// (`array|false`) stays loud even though one member is an array — the operand is not
/// PROVABLY an array at runtime, and the EIR spread lowering has no runtime guard to catch
/// the `false` case. php-check: PHP raises `TypeError` unpacking a non-iterable.
#[test]
fn test_error_spread_array_or_false_union_stays_loud() {
    expect_error(
        "<?php function g(int ...$xs): int { return 0; } \
         function f(array|false $x): int { return g(...$x); }",
        "Spread operator requires an array",
    );
}

/// R4 accept: list-unpacking a `mixed` right-hand side type-checks (each positional
/// target binds as `Mixed`; PHP reads offsets at runtime).
#[test]
fn test_gradual_list_unpack_mixed_accepted() {
    expect_ok("<?php function f(mixed $arr) { [$a, $b] = $arr; return $a; }");
}

/// R4 boundary: list-unpacking a proven bare scalar stays loud. PHP assigns nulls with a
/// warning rather than fatalling; elephc keeps this conservative loud error for now.
#[test]
fn test_error_list_unpack_scalar_stays_loud() {
    expect_error(
        "<?php function f(int $n) { [$a, $b] = $n; return $a; }",
        "List unpacking requires an array on the right-hand side",
    );
}

// --- Family A boundary: PHP's `array` hint rejects every NON-array value ---

/// A `string` value into an `array` parameter stays loud: PHP's monolithic `array` hint accepts
/// any array shape but TypeErrors on a non-array in both strict and coercive mode. Only the
/// element type is unenforced; a scalar actual is a genuine error.
#[test]
fn test_error_string_into_array_param_stays_loud() {
    expect_error(
        "<?php function f(array $x): int { return count($x); } echo f(\"s\");",
        "expects Array(Mixed), got Str",
    );
}

/// An object value into an `array` parameter stays loud. Unlike `iterable`, PHP's `array` hint
/// rejects even a `Traversable` object; a plain object is always a TypeError in both modes.
#[test]
fn test_error_object_into_array_param_stays_loud() {
    expect_error(
        "<?php class C {} function f(array $x): int { return count($x); } echo f(new C());",
        "expects Array(Mixed), got Object",
    );
}

// --- Family F boundary: two provably-disjoint concrete classes stay loud ---

/// A value of an unrelated concrete class into a concrete-class parameter stays loud: PHP single
/// inheritance makes the two classes disjoint, so PHP ALWAYS raises a TypeError. Only subtype
/// relations (either direction) or an interface on either side are deferred to runtime.
#[test]
fn test_error_disjoint_concrete_object_param_stays_loud() {
    expect_error(
        "<?php class A { function a(): int { return 1; } } class B {} \
         function needA(A $x): int { return $x->a(); } \
         $b = new B(); echo needA($b);",
        "expects Object(\"A\"), got Object(\"B\")",
    );
}

/// A union with NO member assignable to the concrete object target stays loud: every possible
/// runtime value would TypeError, so it is a guaranteed error, not a runtime-deferred one.
#[test]
fn test_error_union_no_assignable_object_member_stays_loud() {
    expect_error(
        "<?php class A { function a(): int { return 1; } } class B {} class C {} \
         function needA(A $x): int { return $x->a(); } \
         function pick(int $n): B|C { return new B(); } \
         $x = pick($argc); echo needA($x);",
        "expects Object(\"A\")",
    );
}

/// A union whose extra member is a scalar (`Q|string`) into an object parameter stays loud even
/// though one member matches: bit-casting a string payload to an object pointer at runtime is
/// unsound, so this shape is kept loud rather than deferred (PHP would TypeError on the string
/// value anyway).
#[test]
fn test_error_union_scalar_member_into_object_param_stays_loud() {
    expect_error(
        "<?php class Q { function q(): int { return 1; } } \
         function need(Q $x): int { return $x->q(); } \
         function pick(int $n): Q|string { return new Q(); } \
         $x = pick($argc); echo need($x);",
        "expects Object(\"Q\")",
    );
}

/// A scalar assignment inside a conditional branch remains part of the post-branch flow type.
#[test]
fn test_error_conditional_scalar_assignment_into_object_param_stays_loud() {
    expect_error(
        "<?php class ConditionalObject {} \
         function needConditionalObject(ConditionalObject $value): void {} \
         $value = new ConditionalObject(); \
         if ($argc > 1) { $value = 'bad'; } \
         needConditionalObject($value);",
        "expects Object(\"ConditionalObject\")",
    );
}

// --- Family F boundary: object flows with no proven-subtype edge stay loud (R1-R4 revert).
//     The return-position member of this family (R1/R2 on the return boundary) was later
//     SUPERSEDED by the checked-downcast-on-return runtime guard (SPEC I2) and moved below;
//     parameter positions are unaffected and still stay loud, guarded here. ---

/// A base-typed value flowing into a derived-class parameter stays loud (`Base`-typed value into
/// `Sub $x`). elephc emits no runtime instanceof guard at an object boundary, so accepting a
/// non-proven-subtype flow would be a silent miscompile; PHP raises a TypeError here, matching
/// this loud rejection. Guards the R1/R2 revert of the base→derived gradual-object acceptance.
#[test]
fn test_error_object_base_into_derived_param_stays_loud() {
    expect_error(
        "<?php class B{} class S extends B{} function need(S $x){} \
         function mk(int $n):B{return new S();} echo need(mk(1));",
        "expects Object(\"S\"), got Object(\"B\")",
    );
}

/// A base/interface-typed value flowing into a derived-class typed PROPERTY stays loud (`I`-typed
/// value into `Impl $p`). A property write, like a parameter boundary, emits NO runtime instanceof
/// guard (unlike the RETURN boundary, which does — see `object_return_downcast_guardable`), so the
/// static property slot offset would be used to read fields that a non-`Impl` runtime value does not
/// have, bit-reading off-layout memory (empirically a SIGSEGV / garbage read — verified with a
/// `mixed`-boxed sibling object reaching the slot). PHP raises a catchable TypeError here instead of
/// crashing, so elephc must stay loud rather than accept the unguarded base->derived downcast at a
/// property write. Locks the property-position half of the object-downcast memory-safety boundary.
#[test]
fn test_error_base_into_derived_typed_property_stays_loud() {
    expect_error(
        "<?php interface I {} class Impl implements I { public int $sx = 5; } \
         class Holder { public Impl $p; } \
         function mk(int $n): I { return new Impl(); } \
         $h = new Holder(); $h->p = mk(1);",
        "Property Holder::$p expects Object(\"Impl\"), got Object(\"I\")",
    );
}

/// An all-object union whose extra member is UNRELATED to the concrete object target stays loud
/// (`RC|Route` into `RC $x`, where `Route` is not assignable to `RC`). The unrelated member could
/// be the runtime value and would bit-read as the wrong object; PHP raises a TypeError, so it must
/// stay loud. Guards the R3/R4 revert of the union-object gradual acceptance.
#[test]
fn test_error_object_union_unrelated_member_into_param_stays_loud() {
    expect_error(
        "<?php class Route{} class RC{} function add(RC $x){} \
         function pick(int $n):RC|Route{return new RC();} echo add(pick(1));",
        "expects Object(\"RC\"), got Union([Object(\"RC\"), Object(\"Route\")])",
    );
}

/// SUPERSEDED by the checked-downcast-on-return feature (SPEC I2): a base-typed return
/// expression flowing into a derived declared RETURN type (unlike the sibling PARAMETER-
/// position tests above, which are unaffected and still stay loud) is now accepted, but ONLY
/// because `crate::ir_lower::stmt::return_type_guard` always emits a runtime `instanceof`
/// guard at the return boundary that throws a catchable `TypeError` on an actual mismatch —
/// this is no longer a silent miscompile. See `Checker::object_return_downcast_guardable`
/// (`src/types/checker/type_compat/object_types.rs`) for the checker-side relaxation and
/// `tests/codegen/oop/checked_downcast_return.rs` for full end-to-end/guard coverage
/// (including the negative/mismatch case, which DOES still throw at runtime). This exact
/// shape — `scalarNode(): ScalarNodeDef { return $this->node(); }` where `node(): NodeDef` —
/// is the canonical Symfony `NodeBuilder::scalarNode()`/`node()` pattern the feature targets.
#[test]
fn test_object_base_return_into_derived_return_accepted_with_runtime_guard() {
    expect_ok(
        "<?php \
         class NodeDef { public function label(): string { return \"node\"; } } \
         class ScalarNodeDef extends NodeDef { public function label(): string { return \"scalar\"; } } \
         class Builder { \
             public function scalarNode(): ScalarNodeDef { return $this->node(); } \
             public function node(): NodeDef { return new ScalarNodeDef(); } \
         } \
         $b = new Builder(); echo $b->scalarNode()->label();",
    );
}

// --- Family F boundary: union supertype-object direction into a derived target stays loud ---

/// A nullable base type (`?Base`) flowing into a derived-class parameter stays loud (`?B` into
/// `S $x`). The `B` member of the union is a supertype of the concrete `S` target, which is the
/// unprovable base→derived direction; elephc emits no runtime instanceof guard, so accepting it
/// would SIGSEGV when the runtime value is a bare `B`. PHP raises a TypeError, matching this loud
/// rejection. Guards the round-3 union supertype-object exclusion in `gradual_union_flows_into`.
#[test]
fn test_error_nullable_base_into_derived_param_stays_loud() {
    expect_error(
        "<?php class B{} class S extends B{} function need(S $x){} \
         function mk(int $n): ?B { return new S(); } echo need(mk(1));",
        "expects Object(\"S\"), got Union([Object(\"B\"), Void])",
    );
}

/// A non-nullable base|sub union (`Base|Sub`) flowing into the derived-class parameter stays loud
/// (`B|S` into `S $x`). The `B` member is a supertype of the concrete `S` target — the unprovable
/// base→derived direction with no runtime guard — so PHP raises a TypeError and elephc must too.
/// Guards the round-3 union supertype-object exclusion in `gradual_union_flows_into`.
#[test]
fn test_error_base_or_sub_union_into_derived_param_stays_loud() {
    expect_error(
        "<?php class B{} class S extends B{} function need(S $x){} \
         function mk(int $n): B|S { return new S(); } echo need(mk(1));",
        "expects Object(\"S\"), got Union([Object(\"B\"), Object(\"S\")])",
    );
}

/// `Exception::__construct` third parameter must be `?Throwable`, matching PHP.
#[test]
fn test_error_exception_previous_rejects_non_throwable() {
    expect_error(
        "<?php throw new Exception('x', 0, previous: 123);",
        "previous",
    );
}
