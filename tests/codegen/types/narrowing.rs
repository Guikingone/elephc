//! Purpose:
//! Integration tests for flow-sensitive type narrowing on `is_*` / `instanceof` guards.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Fixtures exercise scalar narrowing (functions and methods), negated guards, the early-return
//!   idiom, `instanceof` narrowing with method dispatch on a runtime-Mixed receiver, `if`/`elseif`
//!   chains, and `: never`-function divergence that keeps the complement after an exhaustive chain.
//!   The guarded variables are untyped parameters that are unions at runtime (heterogeneous calls),
//!   so these tests depend on both the union parameter inference and the narrowing. Outputs match PHP.
//! - Ternary-branch narrowing (`guard ? then : else`) mirrors the `if`/`else` narrowing: the guarded
//!   variable is narrowed to its then-type in the then-branch and its else-type in the else-branch,
//!   scoped to each branch so it never leaks past the ternary.
//! - Pure `&&` conditions narrow every proven local in `if` and `while` bodies, including the
//!   assignment-plus-`instanceof` loop shape used by Symfony's child-definition traversal.

use super::*;

/// Verifies the literal `false` subtype remains callable and uses the normal boolean runtime
/// representation while `int|false` narrows to int after a divergent strict-false guard.
#[test]
fn test_literal_false_type_and_strict_guard_runtime() {
    let out = compile_and_run(
        r#"<?php
        function onlyFalse(false $value): string { return $value === false ? "false" : "bad"; }
        function returnFalse(): false { return false; }
        function requireInt(int|false $value): int {
            if ($value === false) { throw new Exception("false"); }
            return $value;
        }
        echo onlyFalse(returnFalse()), ":", requireInt(9);
        "#,
    );
    assert_eq!(out, "false:9");
}

/// Verifies a truthy local guard removes the nullable arm before numeric use without claiming
/// that the false branch excludes the still-representable integer zero.
#[test]
fn test_truthy_guard_narrows_nullable_int_for_numeric_use() {
    let out = compile_and_run_capture(
        r#"<?php
        function negateTruthy(?int $lines): int {
            if ($lines) {
                return -$lines;
            }
            return 0;
        }
        echo negateTruthy(3), ":", negateTruthy(null), ":", negateTruthy(0);
        "#,
    );
    assert!(
        out.success,
        "program failed: stdout={} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(out.stdout, "-3:0:0");
}

/// Verifies a divergent negated `is_array()` guard narrows the target of an inline `??=`
/// assignment, leaving a concrete array for a following by-reference mutation.
#[test]
fn test_negated_is_array_guard_narrows_null_coalesce_assignment_target() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
        function consume(array|string|null $service): int {
            if (is_string($service)) {
                return -1;
            }
            if (!is_array($service ??= [])) {
                throw new InvalidArgumentException("array required");
            }
            $before = count($service);
            array_shift($service);
            return 10 * $before + count($service);
        }
        echo consume([1, 2]), ":", consume(null), ":", consume("alias");
        "#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "21:0:-1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected the narrowed by-ref array store-back to stay balanced, got: {}",
        out.stderr
    );
}

/// Verifies a truthiness-tested assignment replaces a nullable parameter's flow type on both
/// branches, so a divergent falsey branch leaves the assigned array available to mutation.
#[test]
fn test_negated_truthy_assignment_replaces_nullable_array_after_return() {
    let out = compile_and_run(
        r#"<?php
        function formatAlternatives(?array $alternatives = null): string {
            if (null === $alternatives) {
                if (!$alternatives = array_keys(["one" => 1, "two" => 2])) {
                    return "empty";
                }
            }
            $last = array_pop($alternatives);
            return implode(",", $alternatives).":".$last;
        }
        echo formatAlternatives(), "|", formatAlternatives(["x", "y"]);
        "#,
    );
    assert_eq!(out, "one:two|x:y");
}

/// Verifies ternary arms with different indexed element types retain their shared array container.
#[test]
fn test_conditional_array_reassignment_keeps_array_container_type() {
    let out = compile_and_run(
        r#"<?php
function itemCount(string $value, bool $numeric): int {
    if ('' === $value) {
        return 0;
    } else {
        $value = $numeric ? [1] : explode(",", $value);
        return count($value);
    }
}

echo itemCount("a,b", false), ":", itemCount("x", true);
"#,
    );
    assert_eq!(out, "2:1");
}

/// Verifies `??=` rebinds inferred parameter/local flow types without enforcing entry hints.
#[test]
fn test_null_coalesce_assignment_rebinds_nullable_inferred_locals() {
    let out = compile_and_run(
        r#"<?php
function takeFirst(?array $values, mixed $fallback): string {
    $values ??= $fallback;
    if (!is_array($values)) {
        return "invalid";
    }
    return array_shift($values);
}

function runDefault(?object $application): int {
    $application ??= static fn () => 7;
    return $application();
}

function canonical(?string $input): ?string {
    $value = null;
    if ($input && false !== $pos = strpos($input, ";")) {
        $value = trim(substr($input, 0, $pos));
    }
    if (!$value ??= $input) {
        return null;
    }
    return $value;
}

echo takeFirst(null, ["first"]), ":", runDefault(null), ":", canonical("text/plain");
"#,
    );
    assert_eq!(out, "first:7:text/plain");
}

/// Verifies `!== false` narrows an `array_search()` result to its integer success arm
/// inside the guarded branch before the value is reused as an indexed-array key.
#[test]
fn test_strict_not_false_guard_narrows_array_search_result() {
    let out = compile_and_run(
        r#"<?php
        $values = ["first", "REMOTE_ADDR", "last"];
        $index = array_search("REMOTE_ADDR", $values, true);
        if ($index !== false) {
            unset($values[$index]);
        }
        $values = array_values($values);
        echo count($values), ":", $values[0], ":", $values[1];
        "#,
    );
    assert_eq!(out, "2:first:last");
}

/// Verifies a negated `is_array()` guard around an assignment leaves the successful array arm
/// precise after a divergent body, including a following by-reference array mutation.
#[test]
fn test_negated_is_array_assignment_guard_narrows_fallthrough() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
        function prependCandidate(mixed $candidate): string {
            $values = [];
            if (!is_array($values = $candidate)) {
                throw new InvalidArgumentException("array required");
            }
            array_unshift($values, 9);
            return count($values).":".$values[0].":".$values[1];
        }
        echo prependCandidate([2]);
        "#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "2:9:2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected the narrowed Mixed-array unshift path to stay balanced, got: {}",
        out.stderr
    );
}

/// Verifies a `match (true)` arm result sees every narrowing proven by its `&&` condition.
/// The first guard makes the mixed value indexable both in the second guard and in the result.
#[test]
fn test_match_true_and_chain_narrows_arm_result() {
    let out = compile_and_run(
        r#"<?php
function firstInt(mixed $value): int {
    return match (true) {
        is_array($value) && is_int($value[0]) => $value[0],
        default => 0,
    };
}

echo firstInt([7]), ':', firstInt('no');
"#,
    );
    assert_eq!(out, "7:0");
}

/// Verifies a local overwritten with the same object type on both `if` branches
/// keeps that converged object type for a following nested property write.
#[test]
fn test_if_both_branches_overwrite_local_with_same_object_type() {
    let out = compile_and_run(
        r#"<?php
        final class BranchBox {
            public array $attributes = [];
        }
        function render(bool $alternate): string {
            $value = "source";
            if ($alternate) {
                $value = new BranchBox();
            } else {
                $value = new BranchBox();
            }
            $value->attributes["language"] = "php";
            return $value->attributes["language"];
        }
        echo render(false), ":", render(true);
        "#,
    );
    assert_eq!(out, "php:php");
}

/// Verifies a builtin `int|false` result keeps the literal-false member precise, allowing the
/// standard divergent guard to leave a plain integer on the fallthrough path.
#[test]
fn test_builtin_false_sentinel_narrows_to_success_type() {
    let out = compile_and_run(
        r#"<?php
        function requirePosition(string $haystack, string $needle): int {
            $position = strpos($haystack, $needle);
            if ($position === false) { throw new Exception("not found"); }
            return $position;
        }
        echo requirePosition("abcdef", "cd");
        "#,
    );
    assert_eq!(out, "2");
}

/// Verifies an unrelated local assignment preserves a stable property narrowing in the same
/// branch; invalidation is scoped to the receiver that was actually rebound.
#[test]
fn test_property_narrowing_survives_unrelated_local_assignment() {
    let out = compile_and_run(
        r#"<?php
        final class NarrowedValue { public function __construct(public string $text) {} }
        final class NarrowedBox {
            public function __construct(public ?NarrowedValue $value) {}
        }
        function readBox(NarrowedBox $box): NarrowedValue {
            if (!$box->value instanceof NarrowedValue) { throw new Exception("missing"); }
            $unrelated = 1;
            return $box->value;
        }
        echo readBox(new NarrowedBox(new NarrowedValue("ok")))->text;
        "#,
    );
    assert_eq!(out, "ok");
}

/// Verifies `is_int` narrowing in a function: the then-branch uses the value as an int and the
/// else-branch as a string, with the parameter being `int|string` across the two call sites.
#[test]
fn test_is_int_narrowing_function_then_else() {
    let out = compile_and_run(
        r#"<?php
        function f($x): void {
            if (is_int($x)) { echo "int:", $x, "\n"; } else { echo "str:", $x, "\n"; }
        }
        f(5);
        f("hi");
        "#,
    );
    assert_eq!(out, "int:5\nstr:hi\n");
}

/// Verifies `is_int` narrowing on an instance-method parameter feeding a typed `int` property:
/// the narrowed value is stored into `int $a`, while non-int calls are ignored.
#[test]
fn test_is_int_narrowing_method_into_typed_property() {
    let out = compile_and_run(
        r#"<?php
        class Bar {
            public int $a = 0;
            public function set($x): void { if (is_int($x)) { $this->a = $x; } }
        }
        $o = new Bar();
        $o->set(7);
        $o->set("ignored");
        echo $o->a;
        "#,
    );
    assert_eq!(out, "7");
}

/// Verifies a negated guard (`!is_int`) narrows the else-path (fallthrough) to int.
#[test]
fn test_negated_is_int_guard_narrows_fallthrough() {
    let out = compile_and_run(
        r#"<?php
        function f($x): string { if (!is_int($x)) { return "notint"; } return "int:" . $x; }
        echo f(5), "|", f("hi");
        "#,
    );
    assert_eq!(out, "int:5|notint");
}

/// Verifies `is_string` narrowing lets the guarded value be used by a string builtin.
#[test]
fn test_is_string_narrowing_allows_strlen() {
    let out = compile_and_run(
        r#"<?php
        function f($x): int { if (is_string($x)) { return strlen($x); } return -1; }
        echo f("abc"), "|", f(5);
        "#,
    );
    assert_eq!(out, "3|-1");
}

/// Verifies the early-return idiom: a guard with no `else` whose body always returns narrows the
/// statements after the `if` to the complement type.
#[test]
fn test_early_return_guard_narrows_remainder() {
    let out = compile_and_run(
        r#"<?php
        function f($x): string { if (!is_string($x)) { return "no"; } return "len" . strlen($x); }
        echo f("abc"), "|", f(5);
        "#,
    );
    assert_eq!(out, "len3|no");
}

/// Verifies `instanceof` narrowing lets a method be called on the narrowed object, dispatched on
/// the runtime class of a value that is `Foo|int` at runtime.
#[test]
fn test_instanceof_narrowing_allows_method_call() {
    let out = compile_and_run(
        r#"<?php
        class Foo { public function ts(): int { return 42; } }
        function g($x): int { if ($x instanceof Foo) { return $x->ts(); } return -1; }
        echo g(new Foo()), "|", g(5);
        "#,
    );
    assert_eq!(out, "42|-1");
}

/// Verifies `instanceof` narrowing picks one class out of a union of two object classes and
/// dispatches its method correctly, with the non-matching class taking the else-path.
#[test]
fn test_instanceof_narrowing_two_object_union() {
    let out = compile_and_run(
        r#"<?php
        class A { public function name(): string { return "A"; } }
        class B { public function name(): string { return "B"; } }
        function pick($x): string { if ($x instanceof A) { return $x->name(); } return "notA"; }
        echo pick(new A()), "|", pick(new B());
        "#,
    );
    assert_eq!(out, "A|notA");
}

/// Verifies a subtype-only method remains available in true bodies guarded by a pure `&&`,
/// including a `while` condition whose first operand assigns a nullable value. The narrowing must
/// affect both checking and EIR method lowering without escaping to unguarded code.
#[test]
fn test_instanceof_and_chain_narrows_if_and_while_bodies() {
    let out = compile_and_run(
        r#"<?php
        class Definition {
            public function getClass(): ?string { return null; }
        }
        class ChildDefinition extends Definition {
            public function getParent(): string { return "parent"; }
        }
        function fromIf(Definition $definition): string {
            if ($definition instanceof ChildDefinition && strlen($definition->getParent()) > 0) {
                return $definition->getParent();
            }
            return "none";
        }
        function fromTernary(Definition $definition): string {
            return $definition instanceof ChildDefinition ? $definition->getParent() : "none";
        }
        function fromWhile(Definition $definition): string {
            while ((null === $class = $definition->getClass()) && $definition instanceof ChildDefinition) {
                return $definition->getParent();
            }
            return $class ?? "none";
        }
        echo fromIf(new ChildDefinition()), "|", fromTernary(new ChildDefinition()), "|",
            fromWhile(new ChildDefinition());
        "#,
    );
    assert_eq!(out, "parent|parent|parent");
}

/// Verifies the full overload pattern: an `is_int` guard stores the int into a typed property,
/// while the else-branch calls a method on the narrowed object (dispatched on its runtime class).
#[test]
fn test_overload_pattern_int_or_object() {
    let out = compile_and_run(
        r#"<?php
        class Foo { public function ts(): int { return 42; } }
        class Bar {
            public int $a = 0;
            public int $b = 0;
            public function set($x): void {
                if (is_int($x)) { $this->a = $x; } else { $this->b = $x->ts(); }
            }
        }
        $o = new Bar();
        $o->set(5);
        $o->set(new Foo());
        echo $o->a, "|", $o->b;
        "#,
    );
    assert_eq!(out, "5|42");
}

/// Tests flow-sensitive narrowing across an `if` / `elseif` / `else` chain.
/// Each clause should see the appropriate narrowed (or complement) type.
#[test]
fn test_elseif_narrowing_chain() {
    let out = compile_and_run(
        r#"<?php
        function describe($x): string {
            if (is_int($x)) {
                return "int:" . ($x + 1);
            } elseif (is_string($x)) {
                return "str:" . $x;
            }
            return "other";
        }
        echo describe(41), "|", describe("hi"), "|", describe(3.14);
        "#,
    );
    assert_eq!(out, "int:42|str:hi|other");
}

/// Tests that a branch ending in a `: never` function call lets the code after an
/// if/elseif chain (with no final else) keep the complement type. After the chain `$x`
/// must be narrowed to `int`, which the `: int` return type requires; without `never`
/// detection `$x` would still be `mixed` and `return $x` would be rejected (a `mixed`
/// value does not satisfy an `int` return). The `mixed` parameter avoids the method-call
/// route, which would not distinguish the cases since `mixed` receivers dispatch
/// dynamically.
#[test]
fn test_elseif_chain_with_never_divergence() {
    let out = compile_and_run(
        r#"<?php
        function fail(): never {
            throw new \Exception("boom");
        }

        function classify(mixed $x): int {
            if (is_string($x)) {
                return 0;
            } elseif (!is_int($x)) {
                fail();
            }
            // All clauses diverge and there is no else, so reaching here means every guard
            // was false => $x must be an int.
            return $x;
        }

        echo classify("hi"), "|", classify(41);
        "#,
    );
    assert_eq!(out, "0|41");
}

/// Regression: a non-diverging clause *before* a diverging type guard must not leave the
/// variable narrowed after the `if`. The `$flag` branch reaches the trailing statement
/// without ever evaluating the `instanceof` guard, so `$x` must stay `mixed` (where `+` is
/// allowed) rather than being narrowed to `Box` (where arithmetic is rejected). A rule that
/// kept the complement whenever only the last clause diverged would fail to compile here.
#[test]
fn test_narrowing_not_kept_when_earlier_clause_falls_through() {
    let out = compile_and_run(
        r#"<?php
        class Box {}
        function f(mixed $x, bool $flag): void {
            if ($flag) {
                echo "";
            } elseif (!($x instanceof Box)) {
                return;
            }
            echo $x + 1;
        }
        f(41, true);
        "#,
    );
    assert_eq!(out, "42");
}

/// Regression: when different clauses narrow different variables, every narrowed variable
/// must be restored after the `if`, not only the first. The non-diverging `$x` clause means
/// the trailing statement can run with `$y` unconstrained, so `$y` must stay `mixed` rather
/// than leaking the `Box` narrowing from its diverging clause. A single-slot restore would
/// leave `$y` as `Box` and reject the arithmetic.
#[test]
fn test_narrowing_restores_all_narrowed_variables() {
    let out = compile_and_run(
        r#"<?php
        class Box {}
        function f(mixed $x, mixed $y): void {
            if ($x instanceof Box) {
                echo "";
            } elseif (!($y instanceof Box)) {
                return;
            }
            echo $y + 1;
        }
        f(new Box(), 7);
        "#,
    );
    assert_eq!(out, "8");
}

/// Verifies `instanceof self` narrows the guarded variable to the enclosing class so that a
/// typed property can be read off it. Before the fix the target became `Object("self")` (a
/// non-existent class) and the property access failed to resolve. Matches PHP (`5`).
#[test]
fn test_instanceof_self_narrows_property_access() {
    let out = compile_and_run(
        r#"<?php
        class Node {
            public int $val = 5;
            public function check(mixed $x): int {
                if ($x instanceof self) { return $x->val; }
                return 0;
            }
        }
        $n = new Node();
        echo $n->check($n);
        "#,
    );
    assert_eq!(out, "5");
}

/// Verifies `instanceof self` narrowing resolves a method on the enclosing class, including a
/// generator method whose real `iterable`/`Generator` return type must be recognized so
/// `yield from` accepts it. Before the fix the narrowed receiver was `Object("self")`, the
/// method returned an `Int` fallback, and `yield from` rejected it. Matches PHP (`12`).
#[test]
fn test_instanceof_self_narrows_method_call_yield_from() {
    let out = compile_and_run(
        r#"<?php
        class Spec {
            public array $def = [];
            public function g(): iterable { yield 1; yield 2; }
            public function combined(): iterable {
                foreach ($this->def as $item) {
                    if ($item instanceof self) { yield from $item->g(); }
                }
            }
        }
        $s = new Spec();
        $c = new Spec();
        $s->def = [$c];
        foreach ($s->combined() as $v) { echo $v; }
        "#,
    );
    assert_eq!(out, "12");
}

/// Verifies `instanceof static` narrows the guarded variable to the enclosing class (a sound,
/// conservative narrowing for the closed-world checker), letting a typed property be read.
/// Matches PHP (`7`).
#[test]
fn test_instanceof_static_narrows_property_access() {
    let out = compile_and_run(
        r#"<?php
        class Node {
            public int $v = 7;
            public function check(mixed $x): int {
                if ($x instanceof static) { return $x->v; }
                return 0;
            }
        }
        $n = new Node();
        echo $n->check($n);
        "#,
    );
    assert_eq!(out, "7");
}

/// Verifies `instanceof parent` narrows the guarded variable to the current class's parent, so
/// a parent-declared property resolves. Before the fix the target became `Object("parent")`.
/// Matches PHP (`3`).
#[test]
fn test_instanceof_parent_narrows_property_access() {
    let out = compile_and_run(
        r#"<?php
        class Base { public int $b = 3; }
        class Sub extends Base {
            public function check(mixed $x): int {
                if ($x instanceof parent) { return $x->b; }
                return 0;
            }
        }
        $s = new Sub();
        echo $s->check($s);
        "#,
    );
    assert_eq!(out, "3");
}

/// Regression: an explicit class name in an `instanceof` guard still narrows (property access
/// path), guarding the passthrough arm of the relative-name resolver. Matches PHP (`5`).
#[test]
fn test_instanceof_explicit_class_still_narrows_property() {
    let out = compile_and_run(
        r#"<?php
        class Node {
            public int $val = 5;
            public function check(mixed $x): int {
                if ($x instanceof Node) { return $x->val; }
                return 0;
            }
        }
        $n = new Node();
        echo $n->check($n);
        "#,
    );
    assert_eq!(out, "5");
}

/// Verifies a negated `instanceof self` guard narrows the fallthrough (post-early-return) path
/// to the enclosing class, so the property access after the guarded early return resolves. The
/// resolved target must flow through the complement/else swap. Matches PHP (`5`).
#[test]
fn test_negated_instanceof_self_narrows_fallthrough() {
    let out = compile_and_run(
        r#"<?php
        class Node {
            public int $val = 5;
            public function pick(mixed $x): int {
                if (!($x instanceof self)) { return 0; }
                return $x->val;
            }
        }
        $n = new Node();
        echo $n->pick($n);
        "#,
    );
    assert_eq!(out, "5");
}

/// Verifies `instanceof` narrowing applies inside a ternary's two branches. Without ternary
/// narrowing, `$a->speak()` / `$a->bark()` on the un-narrowed `Cat|Dog` union would each be an
/// "Undefined method" error (Dog has no `speak`, Cat has no `bark`); the guard narrows `$a` to the
/// concrete class per branch so both dispatch. Runs the compiled binary; matches PHP (`meow\nwoof\n`).
#[test]
fn test_ternary_instanceof_narrowing_method_dispatch() {
    let out = compile_and_run(
        r#"<?php
        class Cat { public function speak(): string { return "meow"; } }
        class Dog { public function bark(): string { return "woof"; } }
        function talk(Cat|Dog $a): string {
            return $a instanceof Cat ? $a->speak() : $a->bark();
        }
        echo talk(new Cat()), "\n";
        echo talk(new Dog()), "\n";
        "#,
    );
    assert_eq!(out, "meow\nwoof\n");
}

/// Verifies a scalar `is_int` guard narrows both ternary branches: the then-branch uses `$x` as an
/// int (`$x + 1`) and the else-branch as a string (`strlen($x)`), with `$x` being `int|string`.
/// Matches PHP: `g(5)` is `6`, `g("hello")` is `5` (`6\n5\n`).
#[test]
fn test_ternary_is_int_narrowing() {
    let out = compile_and_run(
        r#"<?php
        function g(int|string $x): int {
            return is_int($x) ? $x + 1 : strlen($x);
        }
        echo g(5), "\n";
        echo g("hello"), "\n";
        "#,
    );
    assert_eq!(out, "6\n5\n");
}

/// Regression: ternary-branch narrowing must not leak into the outer scope. `is_string($x)` narrows
/// `$x` to `string` in the then-branch, but after the ternary `$x` must still be `array|string`, so
/// `count($x)` (which requires an array-containing type) type-checks. If the then-branch narrowing
/// leaked, `count($x)` would become a static error. Matches PHP (`count([10,20,30]) + 2 == 5`).
#[test]
fn test_ternary_narrowing_does_not_leak() {
    let out = compile_and_run(
        r#"<?php
        function leak(array|string $x): int {
            $marker = is_string($x) ? 1 : 2;
            return count($x) + $marker;
        }
        echo leak([10, 20, 30]), "\n";
        "#,
    );
    assert_eq!(out, "5\n");
}

/// Verifies both edges of an `instanceof` guard converge after the guarded object branch
/// overwrites the union local with the false-edge scalar type.
#[test]
fn test_instanceof_branch_reassignment_converges_local_type() {
    let out = compile_and_run(
        r#"<?php
class TaggedName {
    public function getTag(): string {
        return "from-object";
    }
}

function printTag(string $tag): void {
    echo $tag, "\n";
}

function normalizeTag(string|TaggedName $tag): void {
    if ($tag instanceof TaggedName) {
        $tag = $tag->getTag();
    }
    printTag($tag);
}

normalizeTag(new TaggedName());
normalizeTag("already-string");
"#,
    );
    assert_eq!(out, "from-object\nalready-string\n");
}

/// Verifies an always-true null guard keeps the type assigned on its only reachable exit.
#[test]
fn test_null_guard_reassignment_keeps_only_reachable_type() {
    let out = compile_and_run(
        r#"<?php
function printInitializedItems(): void {
    $items = null;
    if (null === $items) {
        $items = ["A", "B"];
    }
    foreach ($items as $item) {
        echo $item;
    }
}

printInitializedItems();
"#,
    );
    assert_eq!(out, "AB");
}

/// Verifies an unconditional sequential assignment replaces the local's current flow type.
#[test]
fn test_sequential_reassignment_replaces_current_flow_type() {
    let out = compile_and_run(
        r#"<?php
class SequentialAssignmentValue {
    public function label(): string {
        return "object";
    }
}

function printSequentialAssignment(SequentialAssignmentValue $value): void {
    echo $value->label();
}

function replaceSequentialAssignment(): void {
    $value = "stale";
    $value = new SequentialAssignmentValue();
    printSequentialAssignment($value);
}

replaceSequentialAssignment();
"#,
    );
    assert_eq!(out, "object");
}

/// Verifies a diverging negative `is_numeric` guard preserves numeric strings for arithmetic
/// without treating an unguarded string as numeric.
#[test]
fn test_is_numeric_guard_narrows_string_for_arithmetic() {
    let out = compile_and_run(
        r#"<?php
function scaleNumericString(string $value) {
    if (!is_numeric($value)) {
        throw new InvalidArgumentException("not numeric");
    }

    return $value * 2;
}

echo scaleNumericString("12");
"#,
    );
    assert_eq!(out, "24");
}

/// Verifies the De Morgan complement of a diverging `if ($cond || !$x instanceof I) { return; }`
/// early-exit: reaching the code after the `if` proves `$x instanceof I`, so a method that lives
/// only on the narrower interface `I` (not its `Wide` base) type-checks and dispatches. Without the
/// `||` fall-through narrowing the checker keeps `$bag` at its wide `Wide` type and rejects `ph()`.
#[test]
fn test_or_early_exit_demorgan_narrows_interface_receiver() {
    let out = compile_and_run(
        r#"<?php
interface Wide { public function w(): string; }
interface I extends Wide { public function ph(): string; }
class Bag implements I {
    public function w(): string { return "w"; }
    public function ph(): string { return "p"; }
}
function run(bool $cond, Wide $bag): string {
    if ($cond || !$bag instanceof I) { return "x"; }
    return $bag->ph();
}
echo run(false, new Bag());
"#,
    );
    assert_eq!(out, "p");
}

/// Verifies a member-path `instanceof` carried through a `&&` chain narrows the property receiver:
/// `if ($this->container instanceof C && true)` proves `$this->container` is the sub-interface `C`,
/// so `$this->container->param()` (a method on `C`, not its `Wide` base) type-checks and dispatches.
#[test]
fn test_member_path_and_chain_narrows_interface_property() {
    let out = compile_and_run(
        r#"<?php
interface Wide { public function w(): int; }
interface C extends Wide { public function param(): int; }
class Container implements C {
    public function w(): int { return 1; }
    public function param(): int { return 7; }
}
class App {
    public ?Wide $container = null;
    public function __construct() { $this->container = new Container(); }
    public function go(): int {
        if ($this->container instanceof C && true) { return $this->container->param(); }
        return 0;
    }
}
echo (new App())->go();
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies a direct property write to an *unrelated* object's slot (`$clone->pool = $repl`) keeps
/// a prior `$this->pool` narrowing: invalidation is scoped to the exact `<root>->prop` rebound, so
/// `$this->pool` stays proven as the sub-interface `Ns` and `$this->pool->sub()` still dispatches.
#[test]
fn test_direct_property_write_to_other_receiver_preserves_this_narrowing() {
    let out = compile_and_run(
        r#"<?php
interface Wide { public function w(): string; }
interface Ns extends Wide { public function sub(): string; }
class Pool implements Ns {
    public function w(): string { return "w"; }
    public function sub(): string { return "s"; }
}
class Ad {
    public ?Wide $pool;
    public function __construct(Wide $pool) { $this->pool = $pool; }
    public function go(Wide $repl): string {
        if (!$this->pool instanceof Ns) { throw new \Exception(); }
        $clone = clone $this;
        $clone->pool = $repl;
        return $this->pool->sub();
    }
}
echo (new Ad(new Pool()))->go(new Pool());
"#,
    );
    assert_eq!(out, "s");
}

/// Verifies the guarded `string|false` narrowing forms Symfony relies on. A value from a
/// `string|false` source reaches a `string` parameter after each dominating guard: a
/// `false === $x` early return, a `false !== $x` guard, a truthy `if ($x)`, and a `!$x` early
/// return. Every form strips the `False` arm so the value is a plain `string` on the used path,
/// and the compiled output matches PHP. An *unguarded* `string|false` still errors (covered by the
/// checker's false-sentinel policy), so this only locks the narrowing, not over-acceptance.
#[test]
fn test_guarded_string_or_false_narrows_to_string() {
    let out = compile_and_run(
        r#"<?php
        function needStr(string $s): string { return "[$s]"; }
        function src(bool $b): string|false { return $b ? "hi" : false; }
        function eqEarly(bool $b): string { $x = src($b); if (false === $x) { return "none"; } return needStr($x); }
        function neq(bool $b): string { $x = src($b); if (false !== $x) { return needStr($x); } return "none"; }
        function truthy(bool $b): string { $x = src($b); if ($x) { return needStr($x); } return "none"; }
        function notEarly(bool $b): string { $x = src($b); if (!$x) { return "none"; } return needStr($x); }
        echo eqEarly(true), eqEarly(false), "|", neq(true), neq(false), "|",
             truthy(true), truthy(false), "|", notEarly(true), notEarly(false);
        "#,
    );
    assert_eq!(out, "[hi]none|[hi]none|[hi]none|[hi]none");
}

/// Verifies a null guard whose body returns before unreachable trailing code still narrows the
/// `?array` value to `Array` for the list unpack after the `if` (issue #590 shape 1). The former
/// last-statement-only model saw the dead `echo` instead of the terminal `return` and dropped the
/// narrowing.
#[test]
fn test_narrow_after_terminal_return_before_unreachable_code() {
    let out = compile_and_run(
        r#"<?php
        function consume(?array $entry): string {
            if ($entry === null) {
                return "empty";
                echo "unreachable";
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"]), "|", consume(null);
        "#,
    );
    assert_eq!(out, "a=b|empty");
}

/// Verifies a null guard whose nested `if` terminates on every branch (return / throw) narrows the
/// `?array` value for the following list unpack (issue #590 shape 2).
#[test]
fn test_narrow_after_nested_if_all_branches_terminate() {
    let out = compile_and_run(
        r#"<?php
        function consume(?array $entry, bool $flag): string {
            if ($entry === null) {
                if ($flag) {
                    return "flag";
                } else {
                    throw new Exception("missing");
                }
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"], true);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies `exit()` and `die()` participate in the shared structural traversal when both paths of
/// a nested `if` diverge. The old shallow checker overlay did not inspect the inner branches.
#[test]
fn test_narrow_after_nested_if_exit_and_die_paths() {
    let out = compile_and_run(
        r#"<?php
        function consume(?array $entry, bool $flag): string {
            if ($entry === null) {
                if ($flag) {
                    exit(1);
                } else {
                    die(2);
                }
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"], true);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies checker-known `never` calls are threaded through nested structural analysis and can
/// combine with an ordinary terminal branch.
#[test]
fn test_narrow_after_nested_if_never_and_return_paths() {
    let out = compile_and_run(
        r#"<?php
        function fail(string $message): never { throw new Exception($message); }
        function consume(?array $entry, bool $flag): string {
            if ($entry === null) {
                if ($flag) {
                    fail("missing");
                } else {
                    return "empty";
                }
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"], true);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies a statically infinite loop without a reachable `break` is non-fallthrough for
/// post-guard narrowing, matching the shared function-exit analysis.
#[test]
fn test_narrow_after_statically_infinite_loop() {
    let out = compile_and_run(
        r#"<?php
        function consume(?array $entry): string {
            if ($entry === null) {
                while (true) {
                    continue;
                }
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"]);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies a `do`/`while` whose mandatory first iteration exits through a checker-known `never`
/// call is non-fallthrough even when its condition is false.
#[test]
fn test_narrow_after_do_while_body_never_call() {
    let out = compile_and_run(
        r#"<?php
        function fail(string $message): never { throw new Exception($message); }
        function consume(?array $entry): string {
            if ($entry === null) {
                do {
                    fail("missing");
                } while (false);
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"]);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies a null guard whose nested `switch` exits on its single case and its `default` narrows
/// the `?array` value for the following list unpack.
#[test]
fn test_narrow_after_nested_switch_all_paths_terminate() {
    let out = compile_and_run(
        r#"<?php
        function consume(?array $entry, int $mode): string {
            if ($entry === null) {
                switch ($mode) {
                    case 1:
                        return "one";
                    default:
                        throw new Exception("missing");
                }
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"], 1);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies a nested exhaustive `switch` combines a checker-known `never` call with shared
/// `exit()` termination and still preserves the post-guard complement.
#[test]
fn test_narrow_after_nested_switch_never_and_exit_paths() {
    let out = compile_and_run(
        r#"<?php
        function fail(string $message): never { throw new Exception($message); }
        function consume(?array $entry, int $mode): string {
            if ($entry === null) {
                switch ($mode) {
                    case 1:
                        fail("missing");
                    default:
                        exit(2);
                }
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"], 1);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies a null guard whose nested `try`/`catch` exits on both the try and the catch bodies
/// narrows the `?array` value for the following list unpack.
#[test]
fn test_narrow_after_nested_try_all_paths_terminate() {
    let out = compile_and_run(
        r#"<?php
        function consume(?array $entry): string {
            if ($entry === null) {
                try {
                    throw new Exception("inner");
                } catch (Throwable $e) {
                    return "caught";
                }
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"]), "|", consume(null);
        "#,
    );
    assert_eq!(out, "a=b|caught");
}

/// Verifies checker-known `never` divergence is propagated through `try` analysis while the catch
/// path uses shared `die()` termination.
#[test]
fn test_narrow_after_nested_try_never_and_die_paths() {
    let out = compile_and_run(
        r#"<?php
        function fail(string $message): never { throw new Exception($message); }
        function consume(?array $entry): string {
            if ($entry === null) {
                try {
                    fail("missing");
                } catch (Throwable $e) {
                    die(2);
                }
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"]);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies a null guard ending in `continue` before unreachable trailing code keeps the `?array`
/// narrowing for the list unpack later in the same loop body. `continue` cannot reach the following
/// statement even though it does not exit the function, so the complement is still sound.
#[test]
fn test_narrow_after_continue_before_unreachable_code() {
    let out = compile_and_run(
        r#"<?php
        function row(int $n): ?array {
            if ($n < 0) { return null; }
            return ["k" . $n, "v" . $n];
        }
        $out = "";
        foreach ([1, -1, 2] as $n) {
            $entry = row($n);
            if ($entry === null) {
                continue;
                echo "unreachable";
            }
            [$key, $value] = $entry;
            $out .= $key . "=" . $value . ";";
        }
        echo $out;
        "#,
    );
    assert_eq!(out, "k1=v1;k2=v2;");
}

/// Verifies shared `exit()` termination still fires when the call is not the block's last
/// statement, allowing the structural block scan to ignore unreachable trailing code.
#[test]
fn test_narrow_after_exit_before_unreachable_code() {
    let out = compile_and_run(
        r#"<?php
        function consume(?array $entry): string {
            if ($entry === null) {
                exit(1);
                echo "unreachable";
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"]);
        "#,
    );
    assert_eq!(out, "a=b");
}

/// Verifies checker-known `never` divergence still fires when the call precedes unreachable
/// trailing code. The checker supplies the function-table lookup to the shared structural model.
#[test]
fn test_narrow_after_never_call_before_unreachable_code() {
    let out = compile_and_run(
        r#"<?php
        function fail(string $m): never { throw new Exception($m); }
        function consume(?array $entry): string {
            if ($entry === null) {
                fail("empty");
                echo "unreachable";
            }
            [$key, $value] = $entry;
            return $key . "=" . $value;
        }
        echo consume(["a", "b"]);
        "#,
    );
    assert_eq!(out, "a=b");
}
