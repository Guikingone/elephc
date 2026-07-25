//! Purpose:
//! Integration or regression tests for diagnostic coverage of class and trait diagnostics, including instanceof parent requires parent class, trait method conflict requires insteadof, and trait property conflict must be compatible.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Invalid PHP snippets are checked through shared diagnostic helpers for messages, spans, and recovery behavior.

use super::*;

/// Verifies that `instanceof parent` reports "Class has no parent class" when the class
/// has no parent.
#[test]
fn test_error_instanceof_parent_requires_parent_class() {
    expect_error(
        "<?php class A { public function f(A $x) { return $x instanceof parent; } }",
        "Class has no parent class",
    );
}

/// Verifies that a class using two traits with conflicting method names (both `foo`)
/// reports "ambiguous trait method" when no `insteadof` resolution is provided.
#[test]
fn test_error_trait_method_conflict_requires_insteadof() {
    expect_error(
        r#"<?php
trait A { public function foo() { return 1; } }
trait B { public function foo() { return 2; } }
class C { use A, B; }
"#,
        "ambiguous trait method 'foo'",
    );
}

/// Verifies that a class using two traits with a property of the same name but
/// incompatible visibility reports "incompatible duplicate property".
#[test]
fn test_error_trait_property_conflict_must_be_compatible() {
    expect_error(
        r#"<?php
trait A { public $value = 1; }
trait B { private $value = 1; }
class C { use A, B; }
"#,
        "incompatible duplicate property",
    );
}

/// Verifies that incompatible constants imported from two traits abort class composition.
#[test]
fn test_error_trait_constant_conflict_must_be_compatible() {
    expect_error(
        r#"<?php
trait A { public const VALUE = 1; }
trait B { public const VALUE = 2; }
class C { use A, B; }
"#,
        "incompatible duplicate trait constant 'VALUE'",
    );
}

/// Verifies that circular trait composition (trait A uses B, B uses A) is detected
/// and reported as an error.
#[test]
fn test_error_circular_trait_composition() {
    expect_error(
        r#"<?php
trait A { use B; }
trait B { use A; }
class C { use A; }
"#,
        "Circular trait composition detected",
    );
}

/// Verifies that accessing a protected property from outside the class hierarchy
/// reports "Cannot access protected property: Secret::value".
#[test]
fn test_error_cannot_access_protected_property_outside_class() {
    expect_error(
        r#"<?php
class Secret {
    protected $value = 7;
}
$s = new Secret();
echo $s->value;
"#,
        "Cannot access protected property: Secret::value",
    );
}

/// Verifies sibling subclasses cannot access a protected property declared by one another.
#[test]
fn test_error_sibling_cannot_access_child_protected_property() {
    expect_error(
        r#"<?php
class ProtectedRoot {}
class ProtectedLeft extends ProtectedRoot {
    public static function read(ProtectedRight $right): int {
        return $right->value;
    }
}
class ProtectedRight extends ProtectedRoot {
    protected int $value = 7;
}
"#,
        "Cannot access protected property: ProtectedRight::value",
    );
}

/// Verifies that declaring two classes differing only by case (Box vs box) reports
/// "Duplicate class declaration: box".
#[test]
fn test_error_duplicate_classes_differing_only_by_case() {
    expect_error(
        "<?php class Box {} class box {}",
        "Duplicate class declaration: box",
    );
}

/// Verifies that declaring two interfaces differing only by case reports
/// "Duplicate interface declaration: named".
#[test]
fn test_error_duplicate_interfaces_differing_only_by_case() {
    expect_error(
        "<?php interface Named {} interface named {}",
        "Duplicate interface declaration: named",
    );
}

/// Verifies that declaring two traits differing only by case reports
/// "Duplicate trait declaration: reusable".
#[test]
fn test_error_duplicate_traits_differing_only_by_case() {
    expect_error(
        "<?php trait Reusable {} trait reusable {}",
        "Duplicate trait declaration: reusable",
    );
}

/// Verifies that declaring two enums differing only by case reports
/// "Duplicate class or enum declaration: mode".
#[test]
fn test_error_duplicate_enums_differing_only_by_case() {
    expect_error(
        "<?php enum Mode { case A; } enum mode { case B; }",
        "Duplicate class or enum declaration: mode",
    );
}

/// Verifies that a class with two methods differing only by case (Save vs save) reports
/// "Duplicate method declaration in Box: save".
#[test]
fn test_error_duplicate_methods_differing_only_by_case() {
    expect_error(
        "<?php class Box { public function Save() { return 1; } public function save() { return 2; } }",
        "Duplicate method declaration in Box: save",
    );
}

/// Verifies that calling `parent::boot()` inside a class with no parent reports
/// "Class Solo has no parent class".
#[test]
fn test_error_parent_without_parent_class() {
    expect_error(
        "<?php class Solo { public function boot() { return parent::boot(); } } $s = new Solo(); $s->boot();",
        "Class Solo has no parent class",
    );
}

/// Verifies that a subclass cannot override a final trait method and reports
/// "Cannot override final method Base::run".
#[test]
fn test_error_trait_final_method_cannot_be_overridden_by_subclass() {
    expect_error(
        "<?php trait T { final public function run() { return 1; } } class Base { use T; } class Child extends Base { public function run() { return 2; } }",
        "Cannot override final method Base::run",
    );
}

/// Verifies that a subclass cannot override a final trait property and reports
/// "Cannot override final property Base::$value".
#[test]
fn test_error_trait_final_property_cannot_be_overridden_by_subclass() {
    expect_error(
        "<?php trait T { final public $value; } class Base { use T; } class Child extends Base { public $value; }",
        "Cannot override final property Base::$value",
    );
}

/// Verifies that `self::class` outside a class context reports
/// "Cannot use self::class or static::class outside a class context".
#[test]
fn test_error_self_class_outside_class() {
    expect_error(
        "<?php echo self::class;",
        "Cannot use self::class or static::class outside a class context",
    );
}

/// Verifies that `parent::class` inside a class with no parent reports
/// "Class 'C' has no parent class".
#[test]
fn test_error_parent_class_without_parent() {
    expect_error(
        "<?php class C { public static function name() { return parent::class; } }",
        "Class 'C' has no parent class",
    );
}

/// Verifies that using `static::` in a class constant expression reports
/// "Cannot use static:: in class constant expression".
#[test]
fn test_error_static_constant_reference_in_class_constant_expression() {
    expect_error(
        "<?php class C { const A = 1; const B = static::A + 1; } echo C::B;",
        "Cannot use static:: in class constant expression",
    );
}

/// Verifies a typed class constant rejects an initializer outside its declared type.
#[test]
fn test_error_typed_class_constant_initializer_mismatch() {
    expect_error(
        "<?php class C { public const int VALUE = 'wrong'; }",
        "Cannot use string as value for class constant C::VALUE of type int",
    );
}

/// Verifies PHP-forbidden callable class-constant types are rejected.
#[test]
fn test_error_typed_class_constant_forbids_callable() {
    expect_error(
        "<?php class C { public const callable VALUE = null; }",
        "Class constant C::VALUE cannot have type callable",
    );
}

/// Verifies PHP-forbidden `void` and `never` class-constant types are rejected.
#[test]
fn test_error_typed_class_constant_forbids_void_and_never() {
    expect_error(
        "<?php class C { public const void VALUE = null; }",
        "Class constant C::VALUE cannot have type void",
    );
    expect_error(
        "<?php class C { public const never VALUE = null; }",
        "Class constant C::VALUE cannot have type never",
    );
}

/// Verifies a child class cannot widen an inherited typed constant contract.
#[test]
fn test_error_typed_class_constant_override_must_be_covariant() {
    expect_error(
        "<?php class Base { public const int VALUE = 1; } class Child extends Base { public const int|string VALUE = 2; }",
        "Type of Child::VALUE must be compatible with Base::VALUE of type int",
    );
}

/// Verifies a class constant must preserve the type required by an implemented interface.
#[test]
fn test_error_typed_interface_constant_implementation_must_be_compatible() {
    expect_error(
        "<?php interface Contract { public const int VALUE = 1; } class Impl implements Contract { public const string VALUE = 'wrong'; }",
        "Type of Impl::VALUE must be compatible with Contract::VALUE of type int",
    );
}

/// Verifies that `new static()` on a child with a required constructor parameter
/// reports a missing argument error.
#[test]
fn test_error_new_static_validates_child_constructor() {
    expect_error(
        "<?php class Base { public static function make(): Base { return new static(); } } class Child extends Base { public function __construct(string $name) {} } echo Child::make();",
        "Constructor 'Child::__construct' expects 1 arguments, got 0",
    );
}

/// Verifies that `new static(...)` inside a method of an ABSTRACT class type-checks cleanly.
/// `static` is late static binding: it resolves at runtime to the concrete called class, which
/// can never be abstract, so the late-bound constructor validator must skip abstract classes in
/// the hierarchy instead of falsely reporting "Cannot instantiate abstract class". Regression for
/// the Symfony String `AbstractString::wrap` false positive.
#[test]
fn test_new_static_in_abstract_class_accepts() {
    expect_ok(
        "<?php abstract class AbstractString { public string $s = \"\"; public function make(string $v): static { return new static($v); } public function __construct(string $v) { $this->s = $v; } } class ByteString extends AbstractString {} class UnicodeString extends AbstractString {} $b = new ByteString(\"hi\"); echo $b->make(\"x\")->s;",
    );
}

/// Verifies the builtin `DatePeriod` constructor enforces its 3-to-4 argument arity.
#[test]
fn test_error_date_period_too_few_args() {
    expect_error(
        "<?php $p = new DatePeriod(new DateTime(\"2024-01-01\"));",
        "Constructor 'DatePeriod::__construct' expects 3 to 4 arguments, got 1",
    );
}

/// Verifies the builtin `DateTime` constructor rejects more than its 0-to-2 arguments.
#[test]
fn test_error_datetime_too_many_args() {
    expect_error(
        "<?php $d = new DateTime(\"now\", null, 3);",
        "Constructor 'DateTime::__construct' expects 0 to 2 arguments, got 3",
    );
}

/// Verifies the builtin `DateTimeImmutable` constructor rejects more than its 0-to-2 arguments.
#[test]
fn test_error_datetime_immutable_too_many_args() {
    expect_error(
        "<?php $d = new DateTimeImmutable(\"now\", null, 3);",
        "Constructor 'DateTimeImmutable::__construct' expects 0 to 2 arguments, got 3",
    );
}

/// Verifies the builtin `DateInterval` constructor requires its single duration-string argument.
#[test]
fn test_error_date_interval_too_few_args() {
    expect_error(
        "<?php $i = new DateInterval();",
        "Constructor 'DateInterval::__construct' expects 1 arguments, got 0",
    );
}

// --- #[\Override] enforcement (PHP 8.3) ---

/// Verifies that `#[Override]` on a method with no matching parent method reports
/// "no matching parent method".
#[test]
fn test_error_override_attribute_with_no_parent_method() {
    expect_error(
        "<?php class Base {} class Child extends Base { #[\\Override] public function nope(): void {} }",
        "no matching parent method",
    );
}

/// Verifies that `#[Override]` on a root class (with no parent) reports
/// "no matching parent method".
#[test]
fn test_error_override_attribute_on_root_class() {
    expect_error(
        "<?php class Solo { #[\\Override] public function alone(): void {} }",
        "no matching parent method",
    );
}

/// Verifies that `#[Override]` on a misspelled method name reports "no matching parent
/// method" rather than silently allowing the typo.
#[test]
fn test_error_override_attribute_on_misspelled_method() {
    expect_error(
        "<?php class Base { public function fetchAll(): void {} } class Child extends Base { #[\\Override] public function fetchAl(): void {} }",
        "no matching parent method",
    );
}

/// Verifies that the unqualified `#[Override]` form (without a leading backslash) is
/// recognized as the PHP 8.3 built-in and enforces parent-method matching.
#[test]
fn test_error_override_attribute_unqualified_form_is_recognized() {
    expect_error(
        "<?php class Base {} class Child extends Base { #[Override] public function nope(): void {} }",
        "no matching parent method",
    );
}

/// Verifies that `#[Override]` imported under an alias (e.g., `use Override as
/// MustOverride`) is still recognized as the built-in and enforces parent-method matching.
#[test]
fn test_error_override_attribute_import_alias_is_recognized() {
    expect_error(
        "<?php use Override as MustOverride; class Base {} class Child extends Base { #[MustOverride] public function nope(): void {} }",
        "no matching parent method",
    );
}

/// Verifies that a namespaced user attribute that looks like `Foo\Override` is NOT
/// treated as the PHP 8.3 built-in `#[Override]` and therefore does not enforce
/// parent-method matching.
#[test]
fn test_override_attribute_qualified_lookalike_is_not_builtin() {
    check_source("<?php class Solo { #[Foo\\Override] public function alone(): void {} }")
        .expect("qualified user attribute should not enforce #[\\Override]");
}

/// Verifies that a namespaced `#[Override]` attribute is NOT treated as the PHP 8.3
/// built-in when the `Override` class does not resolve to the built-in, so no
/// parent-method enforcement occurs.
#[test]
fn test_override_attribute_namespaced_unqualified_lookalike_is_not_builtin() {
    check_source("<?php namespace N; class Solo { #[Override] public function alone(): void {} }")
        .expect("namespaced user attribute should not enforce #[\\Override]");
}

/// Verifies that `#[Override]` on a static method with no matching parent static method
/// reports "no matching parent method".
#[test]
fn test_error_override_attribute_on_static_with_no_parent() {
    expect_error(
        "<?php class Base {} class Child extends Base { #[\\Override] public static function gone(): void {} }",
        "no matching parent method",
    );
}

/// Verifies that `#[AllowDynamicProperties]` inside a namespace is treated as a
/// user-defined attribute (not the built-in), so dynamic properties are rejected
/// with "Undefined property".
#[test]
fn test_allow_dynamic_properties_namespaced_unqualified_lookalike_is_not_builtin() {
    expect_error(
        "<?php namespace N; #[AllowDynamicProperties] class Bag {} $b = new Bag(); $b->x = 1;",
        "Undefined property: N\\Bag::x",
    );
}

// --- class_attribute_names() argument validation ---

/// Verifies that `class_attribute_names()` with an undefined class reports
/// "undefined class 'DoesNotExist'".
#[test]
fn test_error_class_attribute_names_undefined_class() {
    expect_error(
        "<?php $x = class_attribute_names('DoesNotExist');",
        "undefined class 'DoesNotExist'",
    );
}

/// Verifies that `class_attribute_names()` with a dynamic (variable) argument instead
/// of a string literal reports "requires a string literal class name".
#[test]
fn test_error_class_attribute_names_dynamic_argument() {
    expect_error(
        "<?php $name = 'Foo'; class_attribute_names($name);",
        "requires a string literal class name",
    );
}

/// Verifies that `class_attribute_names()` with no argument reports "exactly 1
/// argument".
#[test]
fn test_error_class_attribute_names_no_argument() {
    expect_error("<?php class_attribute_names();", "exactly 1 argument");
}

/// Verifies that `class_attribute_names()` with a non-string argument (e.g., integer)
/// reports "must be a string class name".
#[test]
fn test_error_class_attribute_names_non_string_argument() {
    expect_error(
        "<?php class_attribute_names(42);",
        "must be a string class name",
    );
}

// --- class_attribute_args() argument validation ---

/// Verifies that `class_attribute_args()` with an undefined class reports
/// "undefined class 'DoesNotExist'".
#[test]
fn test_error_class_attribute_args_undefined_class() {
    expect_error(
        "<?php $x = class_attribute_args('DoesNotExist', 'Foo');",
        "undefined class 'DoesNotExist'",
    );
}

/// Verifies that `class_attribute_args()` with a dynamic class name argument reports
/// "requires a string literal class name".
#[test]
fn test_error_class_attribute_args_dynamic_class_argument() {
    expect_error(
        "<?php $name = 'Foo'; class_attribute_args($name, 'Bar');",
        "requires a string literal class name",
    );
}

/// Verifies that `class_attribute_args()` with a dynamic attribute name argument reports
/// "requires a string literal attribute name".
#[test]
fn test_error_class_attribute_args_dynamic_attr_argument() {
    expect_error(
        "<?php #[Foo] class C {} $name = 'Foo'; class_attribute_args('C', $name);",
        "requires a string literal attribute name",
    );
}

/// Verifies that `class_attribute_args()` called with only one argument (instead of
/// two) reports "exactly 2 arguments".
#[test]
fn test_error_class_attribute_args_wrong_arity() {
    expect_error("<?php class_attribute_args('Foo');", "exactly 2 arguments");
}

/// Verifies that `class_attribute_args()` with a non-string first argument reports
/// "first argument must be a string class name".
#[test]
fn test_error_class_attribute_args_non_string_class() {
    expect_error(
        "<?php class_attribute_args(1, 'Foo');",
        "first argument must be a string class name",
    );
}

/// Verifies that `class_attribute_args()` with a non-string second argument reports
/// "second argument must be a string attribute name".
#[test]
fn test_error_class_attribute_args_non_string_attr() {
    expect_error(
        "<?php #[Foo] class C {} class_attribute_args('C', 1);",
        "second argument must be a string attribute name",
    );
}

/// Verifies that `class_attribute_args()` on an attribute with an unmaterialized
/// symbolic constant argument reports "requested attribute uses argument metadata
/// that is not supported yet".
#[test]
fn test_error_class_attribute_const_args_are_not_silently_dropped() {
    expect_error(
        "<?php #[Attribute(Attribute::TARGET_CLASS)] class MyAttr {} class_attribute_args('MyAttr', 'Attribute');",
        "requested attribute uses argument metadata that is not supported yet",
    );
}

/// Verifies that `class_attribute_args()` on an attribute with expression arguments
/// (e.g., `1 + 2`) reports "requested attribute uses argument metadata that is not
/// supported yet".
#[test]
fn test_error_class_attribute_expression_args_are_not_silently_dropped() {
    expect_error(
        "<?php #[Foo(1 + 2)] class C {} class_attribute_args('C', 'Foo');",
        "requested attribute uses argument metadata that is not supported yet",
    );
}

/// Verifies that `class_get_attributes()` on a class with a still-unsupported
/// (non-foldable arithmetic) attribute argument reports "class has attribute
/// argument metadata that is not supported yet" rather than silently dropping it.
/// Float arguments are now supported, so this guards a genuinely unsupported shape.
#[test]
fn test_error_class_get_attributes_unsupported_arg_not_silently_dropped() {
    expect_error(
        "<?php #[Foo(1 + 2)] class C {} class_get_attributes('C');",
        "class has attribute argument metadata that is not supported yet",
    );
}

// --- class_get_attributes() argument validation ---

/// Verifies that `class_get_attributes()` with an undefined class reports
/// "undefined class 'DoesNotExist'".
#[test]
fn test_error_class_get_attributes_undefined_class() {
    expect_error(
        "<?php $x = class_get_attributes('DoesNotExist');",
        "undefined class 'DoesNotExist'",
    );
}

/// Verifies that `class_get_attributes()` with a dynamic (variable) argument reports
/// "requires a string literal class name".
#[test]
fn test_error_class_get_attributes_dynamic_argument() {
    expect_error(
        "<?php $name = 'Foo'; class_get_attributes($name);",
        "requires a string literal class name",
    );
}

/// Verifies that `class_get_attributes()` with no argument reports "exactly 1
/// argument".
#[test]
fn test_error_class_get_attributes_no_argument() {
    expect_error("<?php class_get_attributes();", "exactly 1 argument");
}

/// Verifies that `class_get_attributes()` with a non-string argument reports
/// "must be a string class name".
#[test]
fn test_error_class_get_attributes_non_string_argument() {
    expect_error(
        "<?php class_get_attributes(42);",
        "must be a string class name",
    );
}

/// Verifies that declaring a class named `ReflectionAttribute` reports
/// "Cannot redeclare built-in reflection type: ReflectionAttribute".
#[test]
fn test_error_reflection_attribute_redeclaration() {
    expect_error(
        "<?php class ReflectionAttribute {}",
        "Cannot redeclare built-in reflection type: ReflectionAttribute",
    );
}

/// Verifies that declaring an interface named `ReflectionAttribute` reports
/// "Cannot redeclare built-in reflection type: ReflectionAttribute".
#[test]
fn test_error_reflection_attribute_interface_redeclaration() {
    expect_error(
        "<?php interface ReflectionAttribute {}",
        "Cannot redeclare built-in reflection type: ReflectionAttribute",
    );
}

/// Verifies that declaring a trait named `ReflectionAttribute` reports
/// "Cannot redeclare built-in reflection type: ReflectionAttribute".
#[test]
fn test_error_reflection_attribute_trait_redeclaration() {
    expect_error(
        "<?php trait ReflectionAttribute {}",
        "Cannot redeclare built-in reflection type: ReflectionAttribute",
    );
}

/// Verifies that `new ReflectionAttribute()` reports "Cannot access private
/// constructor: ReflectionAttribute::__construct".
#[test]
fn test_error_reflection_attribute_constructor_is_private() {
    expect_error(
        "<?php $r = new ReflectionAttribute();",
        "Cannot access private constructor: ReflectionAttribute::__construct",
    );
}

/// Verifies that `new ReflectionParameter()` rejects unknown parameter names.
#[test]
fn test_error_reflection_parameter_constructor_unknown_name() {
    expect_error(
        "<?php class C { public function f($a) {} } $r = new ReflectionParameter([C::class, 'f'], 'b');",
        "parameter specified by name could not be found",
    );
}

/// Verifies that `new ReflectionParameter()` rejects unknown function targets.
#[test]
fn test_error_reflection_parameter_constructor_unknown_function() {
    expect_error(
        "<?php $r = new ReflectionParameter('missing_reflect_function', 'a');",
        "Function missing_reflect_function() does not exist",
    );
}

/// Verifies that `new ReflectionParameter()` rejects dynamic function names
/// because runtime function reflection lookup metadata is not available.
#[test]
fn test_error_reflection_parameter_constructor_dynamic_function_name() {
    expect_error(
        "<?php function reflected_function($a) {} $f = 'reflected_function'; $r = new ReflectionParameter($f, 'a');",
        "requires a string literal function name",
    );
}

/// Verifies that `new ReflectionParameter()` still rejects dynamic method names
/// because runtime reflection lookup metadata is not available.
#[test]
fn test_error_reflection_parameter_constructor_dynamic_method_name() {
    expect_error(
        "<?php class C { public function f($a) {} } $m = 'f'; $r = new ReflectionParameter([C::class, $m], 'a');",
        "requires a string literal method name",
    );
}

/// Verifies that `new ReflectionFunction()` rejects unknown function targets.
#[test]
fn test_error_reflection_function_constructor_unknown_function() {
    expect_error(
        "<?php $r = new ReflectionFunction('missing_reflection_function');",
        "Function missing_reflection_function() does not exist",
    );
}

/// Verifies that `new ReflectionFunction()` accepts a string-typed dynamic function name for
/// resolution through the runtime callable registry.
#[test]
fn test_reflection_function_constructor_dynamic_function_name_compiles() {
    expect_ok(
        "<?php function reflected_function($a) {} $f = 'reflected_function'; $r = new ReflectionFunction($f);",
    );
}

/// Verifies an assignment in a negated condition replaces an earlier reflection subtype that
/// only exists on branches which have already returned.
#[test]
fn test_negated_condition_assignment_replaces_returned_branch_reflection_type() {
    expect_ok(
        r#"<?php
class ReflectionFlowFactory {
    public function getFactory(): mixed {
        return null;
    }

    public function getClassName(): ?string {
        return DateTime::class;
    }

    public function getReflectionClass(?string $class): ?ReflectionClass {
        return null === $class ? null : new ReflectionClass($class);
    }

    public function resolve(): ?ReflectionFunctionAbstract {
        if (is_string($factory = $this->getFactory())) {
            $r = new ReflectionFunction($factory);
            return $r;
        }

        if ($factory) {
            return new ReflectionMethod(DateTime::class, "format");
        }

        $class = $this->getClassName();
        if (!$r = $this->getReflectionClass($class)) {
            return null;
        }
        if (!$r = $r->getConstructor()) {
            return null;
        }
        return $r;
    }
}
"#,
    );
}

/// Verifies PHP's `ReflectionParameter implements Reflector` relationship is available to
/// ordinary function-argument compatibility checks.
#[test]
fn test_reflection_parameter_is_assignable_to_reflector() {
    expect_ok(
        "<?php function accepts_reflector(Reflector $reflector): void {} function pass_parameter(ReflectionParameter $parameter): void { accepts_reflector($parameter); }",
    );
}

/// Verifies `ReflectionFunction` rejects attributes whose arguments cannot yet
/// be materialized into `ReflectionAttribute` metadata.
#[test]
fn test_error_reflection_function_get_attributes_unsupported_arg_metadata() {
    expect_error(
        "<?php $name = 'x'; #[FuncAttr($name)] function reflected_function_attr() {} $r = new ReflectionFunction('reflected_function_attr');",
        "function has attribute argument metadata that is not supported yet",
    );
}

/// Verifies that accessing `ReflectionAttribute::__name` property reports
/// "Cannot access private property: ReflectionAttribute::__name".
#[test]
fn test_error_reflection_attribute_internal_properties_are_private() {
    expect_error(
        "<?php #[A] class C {} $attrs = class_get_attributes('C'); echo $attrs[0]->__name;",
        "Cannot access private property: ReflectionAttribute::__name",
    );
}

/// Verifies that declaring a class named `ReflectionClass` reports
/// "Cannot redeclare built-in reflection type: ReflectionClass".
#[test]
fn test_error_reflection_class_redeclaration() {
    expect_error(
        "<?php class ReflectionClass {}",
        "Cannot redeclare built-in reflection type: ReflectionClass",
    );
}

/// Verifies that `new ReflectionClass('Missing')` reports "undefined class 'Missing'".
#[test]
fn test_error_reflection_class_undefined_class() {
    expect_error(
        "<?php $r = new ReflectionClass('Missing');",
        "ReflectionClass::__construct(): undefined class 'Missing'",
    );
}

/// SUPERSEDES the old `test_error_reflection_class_non_string_argument`: `new
/// ReflectionClass($name)` with a non-`string`-typed dynamic argument is NO LONGER a compile
/// error — PHP's real `__construct(object|string $objectOrClass)` signature accepts ANY runtime
/// value and only rejects the wrong shape at RUNTIME (php -n verified: `new ReflectionClass(42)`
/// compiles and throws a real `\ReflectionException`/`\TypeError` at runtime, never a parse/type
/// error), so elephc now compiles this too (see `crate::types::checker::inference::objects::
/// constructors::reflection_class_literal_arg` and `crate::codegen_ir::lower_inst::objects::
/// reflection::lower_reflection_class_new_dynamic`). Full runtime-behavior coverage (weak
/// scalar coercion, object resolution, `\TypeError` on array/resource) lives in the codegen
/// tests in `tests/codegen/oop/reflection.rs`. K1 (see
/// `test_reflection_method_dynamic_class_argument_compiles`) later extended the SAME
/// class-name-argument relaxation to `ReflectionMethod`/`ReflectionProperty` too — the relaxation
/// is no longer `ReflectionClass`-only.
#[test]
fn test_reflection_class_non_string_argument_compiles() {
    expect_ok("<?php $name = 42; $r = new ReflectionClass($name); echo get_class($r);");
}

/// SUPERSEDES the old `test_error_reflection_method_dynamic_class_argument_stays_loud`: K1 (see
/// `crate::codegen_ir::lower_inst::objects::reflection_members`) extends the SAME
/// `ReflectionClass`-style dynamic-dispatch relaxation to `ReflectionMethod`'s class-name
/// argument too (the earlier "PART C is `ReflectionClass`-only" scoping note this test's name
/// referenced no longer holds) — a non-literal `Str`/`Mixed`/`Union`/`Object` class-name argument
/// compiles and resolves through the J4 flat member-table dispatcher at runtime instead of
/// erroring at compile time. Full runtime-behavior coverage (weak scalar coercion, object
/// resolution, `\TypeError` on array) lives in `tests/codegen/oop/reflection.rs`.
#[test]
fn test_reflection_method_dynamic_class_argument_compiles() {
    expect_ok(
        "<?php $name = 'C'; class C { public function foo(): void {} } $r = new ReflectionMethod($name, 'foo'); echo $r->getName();",
    );
}

/// Property counterpart of `test_reflection_method_dynamic_class_argument_compiles`.
#[test]
fn test_reflection_property_dynamic_class_argument_compiles() {
    expect_ok(
        "<?php $name = 'C'; class C { public int $prop = 1; } $r = new ReflectionProperty($name, 'prop'); echo $r->getName();",
    );
}

/// K1 Part B did NOT widen the SECOND (member-name) argument the same way: the constructor
/// signature still declares `method_name`/`property_name` as `Str` (see
/// `builtin_reflection_owner_class`/`builtin_reflection_property` in
/// `crate::types::checker::builtin_types::reflection`), so a non-`Str` argument there is still
/// rejected by the normal callable-signature type check, before `reflection_member_name_arg`
/// (which ALSO keeps requiring `Str`, no `Mixed`/`Object`/weak-coercion acceptance — php -n
/// verified real PHP does not weak-coerce this argument either) ever runs.
#[test]
fn test_error_reflection_method_non_string_member_name_argument_stays_loud() {
    expect_error(
        "<?php class C { public function foo(): void {} } $r = new ReflectionMethod('C', 42);",
        "parameter $method_name expects Str, got Int",
    );
}

/// Property counterpart of `test_error_reflection_method_non_string_member_name_argument_stays_loud`.
#[test]
fn test_error_reflection_property_non_string_member_name_argument_stays_loud() {
    expect_error(
        "<?php class C { public int $prop = 1; } $r = new ReflectionProperty('C', 42);",
        "parameter $property_name expects Str, got Int",
    );
}

/// Verifies that `new ReflectionMethod('C', 'missing')` on an undefined method reports
/// "undefined method 'C::missing'".
#[test]
fn test_error_reflection_method_undefined_method() {
    expect_error(
        "<?php class C {} $r = new ReflectionMethod('C', 'missing');",
        "undefined method 'C::missing'",
    );
}

/// Verifies that `new ReflectionProperty('C', 'missing')` on an undefined property
/// reports "undefined property 'C::$missing'".
#[test]
fn test_error_reflection_property_undefined_property() {
    expect_error(
        "<?php class C {} $r = new ReflectionProperty('C', 'missing');",
        "undefined property 'C::$missing'",
    );
}

/// Verifies that `new ReflectionMethod` on a method with unsupported attribute
/// argument metadata reports "method has attribute argument metadata that is not
/// supported yet".
#[test]
fn test_error_reflection_method_unsupported_attribute_args() {
    expect_error(
        "<?php class C { #[A(1 + 2)] public function f() {} } $r = new ReflectionMethod('C', 'f');",
        "method has attribute argument metadata that is not supported yet",
    );
}

/// Verifies the Reflection API-completeness fix: `getAttributes(?string, int)`
/// optional params (1- and 2-arg calls), `getClosure(?object)` optional param,
/// the `ReflectionAttribute::IS_INSTANCEOF` class constant, and the public
/// `name`/`class` readonly properties all type-check cleanly. Regression guard
/// for the 8 symfony/console probe errors these pieces previously produced
/// (`expects 0 arguments`, `Undefined class constant`, `Undefined property`).
/// Uses string-literal reflection constructors (the only AOT-supported form) and
/// the type-check-only `expect_ok` helper — reflection is largely unbacked at
/// runtime, so this asserts the type-check phase alone is clean.
#[test]
fn test_reflection_api_completeness_accepts() {
    expect_ok(
        r#"<?php
class Foo { public function bar(): void {} public int $baz = 0; }
$rc = new ReflectionClass('Foo');
$rc->getAttributes('Attr');
$rc->getAttributes('Attr', ReflectionAttribute::IS_INSTANCEOF);
echo $rc->name;
$rm = new ReflectionMethod('Foo', 'bar');
$rm->getClosure($rc);
echo $rm->class, $rm->name;
$rp = new ReflectionProperty('Foo', 'baz');
echo $rp->class, $rp->name;
"#,
    );
}

/// Negative control for the Reflection API-completeness fix: an unknown
/// `ReflectionAttribute` class constant still reports "Undefined class constant".
/// Documents that adding `IS_INSTANCEOF` did not blanket-accept arbitrary
/// constants.
#[test]
fn test_error_reflection_attribute_unknown_constant() {
    expect_error(
        "<?php echo ReflectionAttribute::NOT_A_REAL_CONST;",
        "Undefined class constant",
    );
}

/// Negative control for the Reflection API-completeness fix: an unknown property
/// on a reflection object still reports "Undefined property". Documents that
/// adding the public `name`/`class` props did not blanket-accept arbitrary
/// property reads.
#[test]
fn test_error_reflection_unknown_property() {
    expect_error(
        "<?php class Foo {} $rc = new ReflectionClass('Foo'); echo $rc->notARealProp;",
        "Undefined property",
    );
}

/// Verifies that an anonymous class missing its body is rejected with a clear diagnostic.
#[test]
fn test_error_anonymous_class_missing_body() {
    expect_error(
        "<?php $o = new class;",
        "Expected '{' to open anonymous class body",
    );
}

/// Verifies that nullsafe dynamic method calls still reject named arguments.
#[test]
fn test_error_nullsafe_dynamic_method_call_named_arguments() {
    expect_error(
        "<?php $obj?->$m(value: 1);",
        "Named arguments are not supported in dynamic calls",
    );
}

/// Verifies the covariant-return fix does not over-accept: a child override that
/// *widens* the return type (parent returns `Dog`, child returns the supertype
/// `Animal`) is contravariant, which PHP rejects. This guards that accepting
/// covariant (subtype) returns did not also start accepting contravariant ones.
#[test]
fn test_error_override_contravariant_return_rejected() {
    expect_error(
        r#"<?php
class Animal {}
class Dog extends Animal {}
class Base { public function make(): Dog { return new Dog(); } }
class Sub extends Base { public function make(): Animal { return new Animal(); } }
"#,
        "incompatible return type",
    );
}

/// Verifies the covariant-return fix does not over-accept for interface
/// implementations either: an interface method declared to return `Dog` cannot be
/// implemented with the widened (supertype) return `Animal`. Return types are
/// covariant, not contravariant, so this must still error.
#[test]
fn test_error_interface_contravariant_return_rejected() {
    expect_error(
        r#"<?php
class Animal {}
class Dog extends Animal {}
interface I { public function f(): Dog; }
class C implements I { public function f(): Animal { return new Animal(); } }
"#,
        "incompatible return type",
    );
}

/// Checked-downcast-on-return: unrelated classes (neither a subtype of the other) stay loud
/// at compile time even though a runtime `instanceof` guard mechanism now exists for the
/// legitimate base→derived relaxation — that guard covers `D` a subtype/subinterface of the
/// actual returned type only. A hopeless cast (PHP would always throw `TypeError` at runtime,
/// with no possible matching branch) is rejected up front instead of silently compiling a
/// guard chain that could never pass.
#[test]
fn test_error_checked_downcast_return_unrelated_classes_rejected() {
    expect_error(
        r#"<?php
class B {}
class D extends B {}
class E extends B {}
function makeD(): D {
    return new E();
}
"#,
        "expects Object(\"D\"), got Object(\"E\")",
    );
}

/// Checked-downcast-on-return: a return type completely unrelated to any built-in/user class
/// hierarchy (no shared ancestor at all) still stays loud.
#[test]
fn test_error_checked_downcast_return_wholly_unrelated_classes_rejected() {
    expect_error(
        r#"<?php
class Foo {}
class Bar {}
function makeFoo(): Foo {
    return new Bar();
}
"#,
        "expects Object(\"Foo\"), got Object(\"Bar\")",
    );
}

/// Verifies that an explicit `abstract` modifier on an interface method is rejected. Every
/// interface method is implicitly abstract, and PHP 8 fatals on the redundant keyword
/// ("Interface method I::f() must not be abstract", `php -n` verified). Applies uniformly to
/// static and instance interface methods (see the sibling static-method test below).
#[test]
fn test_error_interface_method_explicit_abstract_rejected() {
    expect_error(
        "<?php interface I { abstract public function f(): int; }",
        "must not be declared abstract",
    );
}

/// Verifies the explicit-`abstract` rejection above also fires for a static interface method
/// declaration, confirming the parser-level check does not special-case static-ness
/// (`php -n` verified: PHP's fatal wording is identical for static and instance methods).
#[test]
fn test_error_interface_static_method_explicit_abstract_rejected() {
    expect_error(
        "<?php interface I { abstract public static function f(): int; }",
        "must not be declared abstract",
    );
}

/// Verifies a non-public static interface method is rejected the same way a non-public
/// instance interface method already is (`php -n` verified: "Access type for interface
/// method I::f() must be public").
#[test]
fn test_error_interface_static_method_must_be_public() {
    expect_error(
        "<?php interface I { protected static function f(): int; }",
        "Interface methods must be public",
    );
}

/// Verifies a class that implements an interface declaring `public static function f(): int;`
/// but never defines `f` at all is rejected (PHP: "Class C contains 1 abstract method...",
/// `php -n` verified; elephc reports the interface-contract-not-satisfied message).
#[test]
fn test_error_implementor_missing_static_interface_method() {
    expect_error(
        r#"<?php
interface I { public static function f(): int; }
class C implements I {}
"#,
        "must implement interface method I::f",
    );
}

/// Verifies that satisfying a *static* interface contract with a non-static (instance) method
/// is rejected loudly, matching PHP 8's exact fatal wording (`php -n` verified): "Cannot make
/// static method I::f() non static in class C".
#[test]
fn test_error_implementor_makes_static_interface_method_non_static() {
    expect_error(
        r#"<?php
interface I { public static function f(): int; }
class C implements I { public function f(): int { return 1; } }
"#,
        "Cannot make static method I::f() non static in class C",
    );
}

/// Verifies the reverse direction: satisfying a *non-static* interface contract with a static
/// method is rejected loudly, matching PHP 8's exact fatal wording (`php -n` verified):
/// "Cannot make non static method I::f() static in class C".
#[test]
fn test_error_implementor_makes_instance_interface_method_static() {
    expect_error(
        r#"<?php
interface I { public function f(): int; }
class C implements I { public static function f(): int { return 1; } }
"#,
        "Cannot make non static method I::f() static in class C",
    );
}

/// Verifies that an interface redeclaring a parent interface's static method as an instance
/// method (or vice versa) is rejected during interface flattening itself — before any
/// implementor even exists — matching PHP 8's fatal wording (`php -n` verified): "Cannot make
/// static method A::f() non static in class B".
#[test]
fn test_error_interface_extends_conflicting_static_kind() {
    expect_error(
        r#"<?php
interface A { public static function f(): int; }
interface B extends A { public function f(): int; }
"#,
        "Cannot make static method A::f() non static in class B",
    );
}

/// Verifies that calling a static interface method directly on the interface (`I::f()`, never
/// on a concrete implementor) is rejected at compile time. PHP defers this to a runtime `Error`
/// ("Cannot call abstract method I::f()", `php -n` verified: interfaces have no runtime object
/// to dispatch a call on), but elephc's closed world can detect the literal `InterfaceName::`
/// receiver statically instead of leaving it to fail at runtime.
#[test]
fn test_error_direct_static_call_on_interface_rejected() {
    expect_error(
        r#"<?php
interface I { public static function f(): int; }
I::f();
"#,
        "Cannot call abstract method I::f()",
    );
}

/// Verifies the direct-interface-call rejection above also fires for a non-static interface
/// method invoked via `I::method()` — PHP's fatal wording does not distinguish between static
/// and instance interface methods here (`php -n` verified).
#[test]
fn test_error_direct_static_call_on_interface_instance_method_rejected() {
    expect_error(
        r#"<?php
interface I { public function f(): int; }
I::f();
"#,
        "Cannot call abstract method I::f()",
    );
}

/// Verifies that a genuine `: DeclaringClass` return (NOT `: static`) is early-bound to the
/// declaring class and is NOT late-bound to the receiver: `Base::make(): Base` called on a `Sub`
/// still yields `Base` (the ancestor-return intent this test guards — PHP late-binds only `: static`).
/// The follow-up `$r->only()` on that `Base`-typed result is then accepted via PHP-faithful lenient
/// dispatch, because a concrete subclass (`Sub`, which IS-A `Base`) declares `only` and PHP
/// dispatches on the runtime class rather than checking method existence at compile time. Here
/// `make()` returns `new Base()`, which lacks `only`, so this faults cleanly at runtime with a
/// PHP-style `Error` (`php` verified: "Call to undefined method Base::only()"). It therefore COMPILES
/// (this test) and faults at runtime, exactly like PHP, instead of the previous compile-time rejection.
#[test]
fn test_declaring_class_return_not_late_bound_dispatches_at_runtime() {
    expect_ok(
        r#"<?php
class Base { public function make(): Base { return new Base(); } }
class Sub extends Base { public function only(): string { return "s"; } }
$s = new Sub();
$r = $s->make();
echo $r->only();
"#,
    );
}

/// Verifies that a `: self` return is early-bound to the declaring class (like an explicit class
/// name) and is NOT late-bound to the receiver, mirroring PHP where only `: static` is late-bound:
/// `Base::make(): self` on a `Sub` yields `Base` (the ancestor-return intent this test guards). The
/// follow-up `$r->only()` on that `Base`-typed result is then accepted via PHP-faithful lenient
/// dispatch (`Sub` IS-A `Base` declares `only`; PHP dispatches on the runtime class). Here `make()`
/// returns `new Base()`, which lacks `only`, so it faults cleanly at runtime with a PHP-style `Error`
/// (`php` verified: "Call to undefined method Base::only()") — COMPILES here and faults at runtime.
#[test]
fn test_self_return_not_late_bound_dispatches_at_runtime() {
    expect_ok(
        r#"<?php
class Base { public function make(): self { return new Base(); } }
class Sub extends Base { public function only(): string { return "s"; } }
$s = new Sub();
$r = $s->make();
echo $r->only();
"#,
    );
}

/// SPEC G1: verifies lenient union-receiver method dispatch still errors loudly when NO union
/// member declares the called method. Reports against the FULL union type (JURY ADDENDUM #5),
/// not a single arbitrarily-picked member.
#[test]
fn test_error_union_method_no_member_resolves() {
    expect_error(
        r#"<?php
class A { function foo(): string { return "A"; } }
class B { function bar(): string { return "B"; } }
function make(bool $b): A|B { return $b ? new A() : new B(); }
$u = make(true);
echo $u->frobnicate();
"#,
        "Undefined method: A|B::frobnicate",
    );
}

/// Verifies real PHP does NOT declare `ReflectionProperty::getFileName()` (php -n verified: a
/// hard "Call to undefined method ReflectionProperty::getFileName()" fatal — only
/// `ReflectionClass` and `ReflectionFunctionAbstract`, i.e. `ReflectionFunction`/
/// `ReflectionMethod`, have it), and elephc matches by rejecting the call at compile time rather
/// than fabricating an answer PHP itself does not provide.
#[test]
fn test_error_reflection_property_has_no_get_file_name_method() {
    expect_error(
        r#"<?php
class ElephcNoFileProp { public string $name = ""; }
$rp = new ReflectionProperty("ElephcNoFileProp", "name");
echo $rp->getFileName();
"#,
        "Undefined method: ReflectionProperty::getFileName",
    );
}

/// SPEC G1 / JURY ADDENDUM #1: when TWO OR MORE union members resolve the called method but
/// their signatures disagree on whether the call's arguments are acceptable (`A::m(int)` vs
/// `B::m(array)`, called with an `int` argument), the call must stay loud. Codegen materializes
/// the call's arguments once for whichever runtime branch executes; a per-branch ABI mismatch
/// would silently pass garbage, so this is a conservative under-accept, not a checker gap.
#[test]
fn test_error_union_method_multi_resolving_disagreeing_signatures() {
    expect_error(
        r#"<?php
class A { function m(int $x): string { return "A"; } }
class B { function m(array $x): string { return "B"; } }
function make(bool $b): A|B { return $b ? new A() : new B(); }
$u = make(true);
echo $u->m(5);
"#,
        "Method B::m parameter $x expects",
    );
}

/// `ReflectionFunction::__construct(Closure|string $function)`: a `Closure`-typed value whose
/// identity is not statically resolvable at the `new ReflectionFunction(...)` call site (here, a
/// declared `Closure $c` parameter) is NOW ACCEPTED (M2 PART A — was rejected at compile time
/// before this feature; see `crate::codegen_ir::lower_inst::objects::reflection_function_dynamic`
/// and `tests/codegen/oop/reflection.rs`'s
/// `test_reflection_function_dynamic_closure_backed_methods_and_guarded_throws` for the full
/// runtime-behavior coverage this now compiles to). This is a type-check-only regression test
/// confirming the call no longer raises the old compile-time rejection; see the codegen test
/// above for the actual runtime output.
#[test]
fn test_reflection_function_dynamic_closure_argument_accepted() {
    expect_ok(
        r#"<?php
function reflect(Closure $c) {
    return new ReflectionFunction($c);
}
reflect(function ($x) { return $x; });
"#,
    );
}

/// `ReflectionFunction::__construct(Closure|string $function)` rejects an argument that is
/// neither `Closure`-shaped nor a `string`, mirroring PHP's real `TypeError` for this argument
/// (php -n verified: `new ReflectionFunction([1, 2])` throws `TypeError:
/// ReflectionFunction::__construct(): Argument #1 ($function) must be of type Closure|string,
/// array given`).
#[test]
fn test_error_reflection_function_wrong_type_argument_rejected() {
    expect_error(
        "<?php new ReflectionFunction([1, 2]);",
        "must be of type Closure|string",
    );
}
