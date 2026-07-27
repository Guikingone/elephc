//! Purpose:
//! Regression tests for sound flow-sensitive narrowing diagnostics.
//!
//! Called from:
//! - `cargo test --test error_tests` through Rust's test harness.
//!
//! Key details:
//! - Negative fixtures ensure literal-false and property facts are not retained beyond mutations,
//!   receiver rebindings, or user-code property getters.

use super::*;

/// Verifies the literal `false` parameter type rejects `true` rather than widening to bool.
#[test]
fn test_literal_false_parameter_rejects_true() {
    expect_error(
        "<?php function onlyFalse(false $value): void {} onlyFalse(true);",
        "expects False, got Bool",
    );
}

/// Verifies the fallthrough after `$value === false` keeps a full bool member (`true` remains
/// possible), and that the remaining `int|bool` value is gradually accepted through the `int`
/// return boundary (PHP coercive-mode bool→int return coercion).
#[test]
fn test_strict_false_guard_keeps_full_bool_member() {
    expect_ok(
        "<?php function requireInt(int|bool $value): int { if ($value === false) { throw new Exception('false'); } return $value; }",
    );
}

/// Verifies a direct property write clears a prior property narrowing before a later return.
/// The widened `?W` value is then gradually accepted through the `W` return boundary (the
/// campaign's gradual union-into-object rule defers the null case to runtime), so the program
/// compiles instead of erroring.
#[test]
fn test_property_write_invalidates_narrowing() {
    expect_ok(
        "<?php class W {} class Box { public function __construct(public ?W $value) {} } function read(Box $box): W { if (!$box->value instanceof W) { throw new Exception('missing'); } $box->value = null; return $box->value; }",
    );
}

/// Verifies rebinding the local receiver clears property facts tied to the old object.
/// The widened `?W` read is then gradually accepted through the `W` return boundary
/// under the gradual union-into-object rule.
#[test]
fn test_property_receiver_rebinding_invalidates_narrowing() {
    expect_ok(
        "<?php class W {} class Box { public function __construct(public ?W $value) {} } function read(Box $box, Box $replacement): W { if (!$box->value instanceof W) { throw new Exception('missing'); } $box = $replacement; return $box->value; }",
    );
}

/// Verifies a hooked property is never treated as a stable flow binding across two reads.
/// The unnarrowed `?W` second read is gradually accepted through the `W` return boundary
/// under the gradual union-into-object rule.
#[test]
fn test_property_get_hook_is_not_persistently_narrowed() {
    expect_ok(
        "<?php class W {} class Box { private ?W $stored; public function __construct(?W $stored) { $this->stored = $stored; } public ?W $value { get { $result = $this->stored; $this->stored = null; return $result; } } } function read(Box $box): W { if (!$box->value instanceof W) { throw new Exception('missing'); } return $box->value; }",
    );
}

/// Verifies an undeclared property served by `__get` is not treated as a stable flow binding.
/// The unnarrowed `?W` second read is gradually accepted through the `W` return boundary
/// under the gradual union-into-object rule.
#[test]
fn test_magic_get_property_is_not_persistently_narrowed() {
    expect_ok(
        "<?php class W {} class Box { private ?W $stored; public function __construct(?W $stored) { $this->stored = $stored; } public function __get(string $name): ?W { $result = $this->stored; $this->stored = null; return $result; } } function read(Box $box): W { if (!$box->value instanceof W) { throw new Exception('missing'); } return $box->value; }",
    );
}

/// Verifies `$this instanceof I` narrows the bare `$this` receiver so a method that lives only on
/// the proven sub-interface `I` (absent from the enclosing abstract base) type-checks. This is
/// asserted at the checker layer: the EIR backend still dispatches a bare-`$this` call through the
/// enclosing class's method table, so an abstract base cannot yet lower this end to end.
#[test]
fn test_this_instanceof_narrows_to_subinterface_method() {
    expect_ok(
        "<?php interface Base { public function base(): string; } interface Kid extends Base { public function kids(): int; } abstract class NodeBase implements Base { public function go(): int { if ($this instanceof Kid) { return $this->kids(); } return 0; } }",
    );
}

/// Verifies the `$this instanceof I` narrowing stays sound: a method that is not declared on the
/// proven interface is still rejected inside the guarded branch (no blanket over-acceptance).
#[test]
fn test_this_instanceof_rejects_method_absent_from_interface() {
    expect_error(
        "<?php interface Base { public function base(): string; } interface Kid extends Base { public function kids(): int; } abstract class NodeBase implements Base { public function go(): int { if ($this instanceof Kid) { return $this->notOnKid(); } return 0; } }",
        "Undefined method: Kid::notOnKid",
    );
}

/// Verifies the De Morgan complement of a diverging `if ($cond || !$x instanceof I) { return; }`
/// narrows the fall-through path to `I`, so a method living only on `I` type-checks afterward.
#[test]
fn test_or_early_exit_demorgan_narrows_fallthrough() {
    expect_ok(
        "<?php interface Wide { public function w(): string; } interface I extends Wide { public function ph(): string; } function run(bool $cond, Wide $bag): string { if ($cond || !$bag instanceof I) { return \"x\"; } return $bag->ph(); }",
    );
}

/// Verifies the `||` complement narrowing does not fire when the guarded body can fall through:
/// reaching the following code no longer proves the condition was false, so `$bag` keeps its wide
/// `Wide` type and the method that lives only on `I` is still rejected.
#[test]
fn test_or_early_exit_does_not_narrow_when_body_falls_through() {
    expect_error(
        "<?php interface Wide { public function w(): string; } interface I extends Wide { public function ph(): string; } function run(bool $cond, Wide $bag): string { if ($cond || !$bag instanceof I) { $bag->w(); } return $bag->ph(); }",
        "Undefined method: Wide::ph",
    );
}

/// Verifies the true branch of an `if ($cond || !$x instanceof I)` guard does not see `$x` narrowed
/// to `I` — that branch is reached when `$x` is *not* an `I`, so the method-on-`I` call must error.
#[test]
fn test_or_early_exit_true_branch_is_not_narrowed() {
    expect_error(
        "<?php interface Wide { public function w(): string; } interface I extends Wide { public function ph(): string; } function run(bool $cond, Wide $bag): string { if ($cond || !$bag instanceof I) { return $bag->ph(); } return \"y\"; }",
        "Undefined method: Wide::ph",
    );
}

/// Verifies a direct property write to an unrelated receiver keeps a prior `$this->pool` narrowing:
/// `$clone->pool = $repl` invalidates only the `$clone->pool` fact, so `$this->pool` stays `Ns`.
#[test]
fn test_unrelated_property_write_preserves_this_narrowing() {
    expect_ok(
        "<?php interface Wide { public function w(): string; } interface Ns extends Wide { public function sub(): string; } class Ad { public ?Wide $pool; public function __construct(Wide $pool) { $this->pool = $pool; } public function go(Wide $repl): string { if (!$this->pool instanceof Ns) { throw new \\Exception(); } $clone = clone $this; $clone->pool = $repl; return $this->pool->sub(); } }",
    );
}

/// Verifies flow-sensitive branch-assignment typing does not leak a derived class past the merge:
/// one branch assigns `Base`, another assigns `Derived`, so after the `if`/`else` the local joins to
/// the base LUB `Base`, NOT `Derived` (the non-leakage this test guards — the precise derived type is
/// scoped to the assigning branch, never the post-merge read). The follow-up `$n->d()` on that
/// `Base`-typed local is then accepted via PHP-faithful lenient dispatch, because a concrete subclass
/// (`Derived`, IS-A `Base`) declares `d` and PHP dispatches on the runtime class rather than checking
/// method existence at compile time. It faults cleanly with a PHP-style `Error` only for the `Base`
/// branch (`php` verified: "Call to undefined method Base::d()"). It therefore COMPILES (the merged
/// type stays `Base`) and faults at runtime for the `Base` branch, superseding the compile-time
/// rejection.
#[test]
fn test_branch_derived_assignment_does_not_leak_past_merge() {
    expect_ok(
        "<?php class Base { public function b(): string { return \"b\"; } } class Derived extends Base { public function d(): int { return 7; } } function run(bool $f): int { if ($f) { $n = new Base(); } else { $n = new Derived(); } return $n->d(); }",
    );
}

/// Verifies an `else`-branch reassignment reaches the post-`if` join: `$x` narrowed to `Autowire`
/// in the `if`, then reassigned to `Producer::newInstance()`'s `Made` result in the `else`, joins to
/// `Autowire|Made` after the construct — a member declared on both arms type-checks. The pre-`if`
/// `Producer` member (absent from the reachable join) must no longer leak in and reject the access.
#[test]
fn test_else_branch_reassignment_reaches_post_if_join() {
    expect_ok(
        "<?php class Producer { public function newInstance(): Made { return new Made(); } } class Made { public int $value = 42; public function tag(): string { return \"made\"; } } class Autowire { public int $value = 7; public function tag(): string { return \"auto\"; } } function resolve(Autowire|Producer $attribute): string { if ($attribute instanceof Autowire) { $seen = $attribute->value; } else { $attribute = $attribute->newInstance(); } return $attribute->tag() . \":\" . $attribute->value; }",
    );
}

/// Verifies the 3-way `if`/`elseif`/`else` variant of the join: `$attribute` is assigned in the
/// `elseif` condition, that clause diverges (`return`), and the `else` reassigns it via a method
/// call. Only the `if` body (`Attr`) and the `else` body (`Attr` from `newInstance()`) reach the
/// following code, so the join is `Attr` and a member on it type-checks — the `elseif`-condition
/// `?Wrapper` type must not survive to the join. Mirrors Symfony's `ResolveAutowireInlineAttributesPass`.
#[test]
fn test_elseif_condition_assignment_reassigned_in_else_joins() {
    expect_ok(
        "<?php class Attr { public int $value = 5; public function build(): string { return \"attr\"; } } class Wrapper { public function newInstance(): Attr { return new Attr(); } } function pick(int $mode, Attr $direct, ?Wrapper $wrapped): string { if ($mode === 1) { $attribute = $direct; } elseif (!$attribute = $wrapped) { return \"none\"; } else { $attribute = $attribute->newInstance(); } return $attribute->build() . \":\" . $attribute->value; }",
    );
}

/// Verifies the branch join stays sound: the post-`if` type is the *union* of the reaching arms, so
/// a member present on only one arm (`A::$a`, absent from the `else`-branch `B`) is still rejected.
/// Guards against the union fix over-accepting by collapsing to a single arm.
#[test]
fn test_branch_join_union_rejects_member_absent_from_one_arm() {
    expect_error(
        "<?php class A { public int $a = 1; public function m(): int { return $this->a; } } class B { public int $b = 2; public function m(): int { return $this->b; } } class Maker { public function make(): B { return new B(); } } function f(A|Maker $x): int { if ($x instanceof A) { $seen = 1; } else { $x = $x->make(); } return $x->a; }",
        "Undefined property: B::a",
    );
}
