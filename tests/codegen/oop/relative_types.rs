//! Purpose:
//! End-to-end codegen tests for the relative class types `self`, `static`, and `parent` used
//! in method parameter, method return, and property type positions.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `self` resolves lexically while method return `static` binds to the call-site receiver.
//! - Trait methods resolve `self`/`static` to the using class, exercised by `test_static_in_trait`.
//! - A `: static` return is late-bound to the receiver class (PHP late static binding): an inherited
//!   or `parent::`-forwarded `: static` method call yields the calling subclass, not the declaring
//!   class. Genuine `: DeclaringClass` / `: self` returns stay early-bound (guarded in error tests).

use super::*;

/// Verifies that a `self` return type lets a method return `$this` and be chained.
#[test]
fn test_self_return_type_chains() {
    let out = compile_and_run(
        "<?php
        class C {
            public function me(): self { return $this; }
            public function v(): string { return \"ok\"; }
        }
        echo (new C())->me()->v();
        ",
    );
    assert_eq!(out, "ok");
}

/// Regression: a `self`-typed VARIADIC parameter (`self ...$items`) must have its `self`
/// rewritten to the enclosing class like every other member type annotation. Previously the
/// variadic-param type was skipped, so `self` survived and was rejected with
/// "Cannot use 'self' as a type outside of a class".
#[test]
fn test_self_typed_variadic_param() {
    let out = compile_and_run(
        "<?php
        final class Bag {
            public function __construct(public string $x) {}
            public static function concat(self ...$items): self {
                $buf = '';
                foreach ($items as $i) { $buf .= $i->x; }
                return new self($buf);
            }
        }
        echo Bag::concat(new Bag('a'), new Bag('b'), new Bag('c'))->x;
        ",
    );
    assert_eq!(out, "abc");
}

/// Regression: a `self`-typed VARIADIC parameter on an ENUM method must be rewritten to the
/// enum name like regular parameters and return types. The enum schema path uses its own
/// relative-type substitution, which previously skipped the variadic-param type.
#[test]
fn test_enum_self_typed_variadic_param() {
    let out = compile_and_run(
        "<?php
        enum Suit: string {
            case Hearts = 'H';
            case Spades = 'S';
            case Clubs = 'C';
            public static function join(self ...$suits): string {
                $buf = '';
                foreach ($suits as $s) { $buf .= $s->value; }
                return $buf;
            }
        }
        echo Suit::join(Suit::Hearts, Suit::Spades, Suit::Clubs);
        ",
    );
    assert_eq!(out, "HSC");
}

/// Verifies that a `static` return type returns a late-bound instance via `new static()`.
#[test]
fn test_static_return_type() {
    let out = compile_and_run(
        "<?php
        class C {
            public static function make(): static { return new static(); }
            public function v(): string { return \"made\"; }
        }
        echo C::make()->v();
        ",
    );
    assert_eq!(out, "made");
}

/// Verifies an inherited static factory returning `static` exposes subclass-only methods.
#[test]
fn test_inherited_static_factory_return_binds_to_called_class() {
    let out = compile_and_run(
        r#"<?php
class Factory {
    public static function make(): static { return new static(); }
}
final class ProductFactory extends Factory {
    public function label(): string { return "product"; }
}
echo ProductFactory::make()->label();
echo ":";
echo (new ReflectionMethod(Factory::class, "make"))->getReturnType()->getName();
"#,
    );
    assert_eq!(out, "product:static");
}

/// Verifies an inherited non-`with*` method returning `static` exposes subclass-only methods.
#[test]
fn test_inherited_static_return_type_binds_to_subclass_receiver() {
    let out = compile_and_run(
        r#"<?php
class Builder {
    public function andWhere(string $condition): static { return $this; }
}
final class QueryBuilder extends Builder {
    public function getSQL(): string { return "SELECT"; }
}
echo (new QueryBuilder())->andWhere('active = 1')->getSQL();
"#,
    );
    assert_eq!(out, "SELECT");
}

/// Verifies nullable late-static returns retain null while binding the object branch.
#[test]
fn test_nullable_static_return_binds_object_branch_to_receiver() {
    let out = compile_and_run(
        r#"<?php
class MaybeBuilder {
    public function maybe(bool $present): ?static {
        return $present ? $this : null;
    }
}
final class ConcreteBuilder extends MaybeBuilder {
    public function build(): string { return "built"; }
}
$builder = new ConcreteBuilder();
echo $builder->maybe(true)?->build();
echo $builder->maybe(false)?->build() ?? "none";
"#,
    );
    assert_eq!(out, "builtnone");
}

/// Verifies a compound late-static return keeps its explicit member in typing, ABI boxing,
/// and Reflection metadata.
#[test]
fn test_late_static_union_preserves_explicit_member() {
    let out = compile_and_run(
        r#"<?php
class Choice {
    public function choose(bool $same): static|Choice {
        return $same ? $this : new Choice();
    }
    public function label(): string { return "choice"; }
}
final class SpecialChoice extends Choice {}
$value = (new SpecialChoice())->choose(false);
echo $value->label() . ":";
$type = (new ReflectionMethod(Choice::class, "choose"))->getReturnType();
if ($type instanceof ReflectionUnionType) {
    echo count($type->getTypes());
    foreach ($type->getTypes() as $member) {
        echo ":" . $member->getName();
    }
}
"#,
    );
    assert_eq!(out, "choice:2:Choice:static");
}

/// Verifies a child override may covariantly narrow `static|false` to `static`.
#[test]
fn test_late_static_union_override_can_narrow_to_static() {
    let out = compile_and_run(
        r#"<?php
class MaybeCloneable {
    public function duplicate(): static|false { return false; }
}
final class AlwaysCloneable extends MaybeCloneable {
    public function duplicate(): static { return $this; }
    public function label(): string { return "clone"; }
}
echo (new AlwaysCloneable())->duplicate()->label();
"#,
    );
    assert_eq!(out, "clone");
}

/// Verifies that a `parent` return type resolves to the parent class and exposes its methods.
#[test]
fn test_parent_return_type() {
    let out = compile_and_run(
        "<?php
        class P { public function who(): string { return \"P\"; } }
        class C extends P {
            public function up(): parent { return $this; }
        }
        echo (new C())->up()->who();
        ",
    );
    assert_eq!(out, "P");
}

/// Verifies that a `self` parameter type accepts another instance of the same class.
#[test]
fn test_self_parameter_type() {
    let out = compile_and_run(
        "<?php
        class C {
            public int $n = 0;
            public function plus(self $other): int { return $this->n + $other->n; }
        }
        $a = new C(); $a->n = 2;
        $b = new C(); $b->n = 3;
        echo $a->plus($b);
        ",
    );
    assert_eq!(out, "5");
}

/// Verifies that a nullable `?self` property stores a same-class instance and null.
#[test]
fn test_self_nullable_property() {
    let out = compile_and_run(
        "<?php
        class Node {
            public ?self $next = null;
            public int $v = 0;
        }
        $a = new Node(); $a->v = 1;
        $b = new Node(); $b->v = 2;
        $a->next = $b;
        echo $a->next->v;
        echo $a->next->next === null ? \"end\" : \"?\";
        ",
    );
    assert_eq!(out, "2end");
}

/// Verifies that a `?self` return type returns either a same-class instance or null.
#[test]
fn test_self_nullable_return() {
    let out = compile_and_run(
        "<?php
        class C {
            public function maybe(bool $b): ?self { return $b ? $this : null; }
            public function v(): string { return \"M\"; }
        }
        $c = new C();
        echo $c->maybe(true)->v();
        echo $c->maybe(false) === null ? \"N\" : \"?\";
        ",
    );
    assert_eq!(out, "MN");
}

/// Verifies that `static` inside a trait method resolves to the using class, not the trait,
/// so the returned instance exposes the using class's own methods.
#[test]
fn test_static_in_trait() {
    let out = compile_and_run(
        "<?php
        trait Fluent {
            public function chain(): static { return $this; }
        }
        class Builder {
            use Fluent;
            public function build(): string { return \"built\"; }
        }
        echo (new Builder())->chain()->build();
        ",
    );
    assert_eq!(out, "built");
}

/// Compiles and runs the checked-in `examples/relative-class-types/main.php` fixture, which
/// exercises `self`, a late-bound inherited `static` return, and a nullable `?self` property.
#[test]
fn test_example_relative_class_types_compiles_and_runs() {
    let out = compile_and_run(include_str!("../../../examples/relative-class-types/main.php"));
    assert_eq!(out, "599\n3\n6\ntail\nSELECT * WHERE active = 1\n");
}

/// Verifies PHP late static binding for a `: static` return declared in a base class. `Mid::pad`
/// (contract `: static` = `Mid`) returns `$this->append(...)`, where `append(): static` is declared
/// in the abstract base; because `$this` is a `Mid`, the inherited `: static` return late-binds to
/// `Mid` and satisfies `pad`'s contract instead of yielding the declaring base class. Regression for
/// the Symfony String `pad`/`trim*` covariance errors.
#[test]
fn test_static_return_inherited_method_late_binds_to_receiver() {
    let out = compile_and_run(
        "<?php
        abstract class Base {
            public string $s = \"\";
            public function append(string $x): static { $c = clone $this; $c->s = $this->s . $x; return $c; }
        }
        class Mid extends Base {
            public function pad(string $y): static { return $this->append($y); }
        }
        $m = new Mid();
        echo $m->pad(\"!\")->s;
        ",
    );
    assert_eq!(out, "!");
}

/// Verifies that an inherited `: static` method declared in an abstract base late-binds to the
/// calling subclass. `Square::make` (contract `Square`) returns `$this->withSides(4)`, and
/// `withSides` (inherited from `Shape`, `: static`) is treated as returning `Square` because
/// `$this` is a `Square` — otherwise `make`'s `: static` contract would reject the base type.
#[test]
fn test_static_return_inherited_from_abstract_base() {
    let out = compile_and_run(
        "<?php
        abstract class Shape {
            public int $sides = 0;
            public function withSides(int $n): static { $c = clone $this; $c->sides = $n; return $c; }
        }
        class Square extends Shape {
            public function make(): static { return $this->withSides(4); }
        }
        echo (new Square())->make()->sides;
        ",
    );
    assert_eq!(out, "4");
}

/// Verifies that `parent::method()` — a forwarding call — late-binds a `: static` return to the
/// current class, not the resolved parent class. `MidS::tail` overrides `AbstractS::tail` and
/// returns `parent::tail(...)`; the parent's `: static` return (collapsed to `AbstractS`) must
/// late-bind to `MidS` to satisfy `MidS::tail`'s `: static` contract. This is the exact Symfony
/// String `join`/`trimPrefix`/`trimSuffix` shape.
#[test]
fn test_static_return_parent_forwarding_binds_current_class() {
    let out = compile_and_run(
        "<?php
        abstract class AbstractS {
            public string $s = \"\";
            public function tail(string $x): static { $c = clone $this; $c->s = $this->s . $x; return $c; }
        }
        class MidS extends AbstractS {
            public function tail(string $x): static { return parent::tail($x . $x); }
        }
        echo (new MidS())->tail(\"a\")->s;
        ",
    );
    assert_eq!(out, "aa");
}

/// Verifies a same-class `$this->staticMethod()` chain: `b()` returns `$this->a()` and both `a` and
/// `b` are declared `: static` in the same class, resolving to that class. Guards that the
/// late-binding refinement does not disturb the already-correct same-class case.
#[test]
fn test_static_return_same_class_chain() {
    let out = compile_and_run(
        "<?php
        class Cc {
            public string $s = \"ok\";
            public function a(): static { return $this; }
            public function b(): static { return $this->a(); }
        }
        echo (new Cc())->b()->s;
        ",
    );
    assert_eq!(out, "ok");
}

/// Verifies a `?static` return: the non-null branch late-binds to the receiver class (so its
/// methods are callable) and the null branch returns null. `maybe(true)` yields a usable `C`;
/// `maybe(false)` yields null. The nullable shape is preserved by the late-binding substitution.
#[test]
fn test_static_return_nullable() {
    let out = compile_and_run(
        "<?php
        class C {
            public function v(): string { return \"C\"; }
            public function maybe(bool $b): ?static { return $b ? $this : null; }
        }
        $c = new C();
        echo $c->maybe(true)->v();
        echo $c->maybe(false) === null ? \"N\" : \"?\";
        ",
    );
    assert_eq!(out, "CN");
}

/// Deferred follow-up: a `static` factory (`public static function create(): static { return new
/// static(); }`) called on a SUBCLASS should return the subclass, but this cluster's late binding is
/// scoped to instance and `parent::`/`self::` dispatch; the static-method-call path and `new static`
/// remain early-bound to the declaring class. Codegen also reads the collapsed return type, so a
/// subclass-only method on the result cannot be lowered even with a checker-only refinement. Ignored
/// until static-method / `new static()` late binding is implemented.
#[test]
#[ignore]
fn test_static_return_static_factory_subclass_deferred() {
    let out = compile_and_run(
        "<?php
        class Base2 { public static function create(): static { return new static(); } }
        class Sub2 extends Base2 { public function subonly(): string { return \"S\"; } }
        echo Sub2::create()->subonly();
        ",
    );
    assert_eq!(out, "S");
}

/// Regression: `new static(...)` written inside a method of an ABSTRACT class must not be rejected
/// with "Cannot instantiate abstract class". `static` is late static binding, so it resolves at
/// runtime to the concrete called class (never abstract). The checker's late-bound constructor
/// validator must skip abstract classes in the hierarchy. Here `AbstractString::make` returns
/// `new static($v)`; calling it on a concrete `ByteString` receiver late-binds `static` to
/// `ByteString`, so the resulting instance's `s` property is set and printed.
#[test]
fn test_new_static_in_abstract_class_binds_to_concrete_subclass() {
    let out = compile_and_run(
        "<?php
        abstract class AbstractString {
            public string $s = \"\";
            public function make(string $v): static { return new static($v); }
            public function __construct(string $v) { $this->s = $v; }
        }
        class ByteString extends AbstractString {}
        class UnicodeString extends AbstractString {}
        $b = new ByteString(\"hi\");
        echo $b->make(\"x\")->s;
        ",
    );
    assert_eq!(out, "x");
}
