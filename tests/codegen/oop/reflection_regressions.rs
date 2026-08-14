//! Purpose:
//! End-to-end regressions for Reflection metadata materialization and typed dispatch.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Fixtures exercise closed-world reflection metadata using real user declarations.

use super::*;

/// Verifies Reflection materializes indexed and associative literal class constants as arrays.
#[test]
fn test_reflection_class_materializes_literal_array_constants() {
    let out = compile_and_run(
        r#"<?php
class ArrayConstantTarget {
    private const SECOND = 'second';
    public const PAD = STR_PAD_RIGHT;
    public const TAGS = ['first', self::SECOND];
    public const OPTIONS = ['mode' => 'strict', 7 => 40 + 2, 'pad' => STR_PAD_RIGHT];
}

$reflection = new ReflectionClass(ArrayConstantTarget::class);
$tags = $reflection->getConstant('TAGS');
$options = $reflection->getConstants()['OPTIONS'];
$memberTags = $reflection->getReflectionConstant('TAGS')->getValue();
echo $tags[0] . ':' . $tags[1] . ':';
echo $options['mode'] . ':' . $options[7] . ':' . $options['pad'] . ':';
echo $memberTags[0] . ':' . $memberTags[1];
echo ':' . $reflection->getConstant('PAD');
"#,
    );
    assert_eq!(out, "first:second:strict:42:1:first:second:1");
}

/// Verifies a `Reflector`-typed parameter dispatches the common concrete
/// `getAttributes()` method through the internal interface table.
#[test]
fn test_reflector_typed_get_attributes_dispatch() {
    let out = compile_and_run(
        r#"<?php
class PlainReflectionTarget {}
function attribute_count(Reflector $reflector): int {
    return count($reflector->getAttributes());
}
echo attribute_count(new ReflectionClass(PlainReflectionTarget::class));
"#,
    );
    assert_eq!(out, "0");
}

/// Verifies public Reflection name properties share the same populated metadata as `getName()`.
#[test]
fn test_reflection_public_name_properties_use_populated_metadata() {
    let out = compile_and_run(
        r#"<?php
class NamedReflectionTarget {
    public function execute(string $value): void {}
}
$class = new ReflectionClass(NamedReflectionTarget::class);
$method = new ReflectionMethod(NamedReflectionTarget::class, 'execute');
$parameter = $method->getParameters()[0];
echo $class->name . '|' . $method->name . '|' . $method->class . '|' . $parameter->name;
"#,
    );
    assert_eq!(
        out,
        "NamedReflectionTarget|execute|NamedReflectionTarget|value"
    );
}

/// Verifies callable reflectors are assignable to their shared abstract parent and dispatch the
/// inherited contracts to the concrete method reflector implementation.
#[test]
fn test_reflection_method_through_function_abstract_contract() {
    let out = compile_and_run(
        r#"<?php
class AbstractReflectionTarget {
    public function execute(string $value): int { return 1; }
}
function describe(ReflectionFunctionAbstract $reflector): string {
    return $reflector->name . ':' . $reflector->class . ':' . $reflector->getName() . ':' .
        count($reflector->getParameters()) . ':' .
        ($reflector->hasReturnType() ? 'typed' : 'untyped');
}
$method = new ReflectionMethod(AbstractReflectionTarget::class, 'execute');
echo $method instanceof ReflectionFunctionAbstract ? 'base|' : 'bad|';
echo describe($method);
"#,
    );
    assert_eq!(
        out,
        "base|execute:AbstractReflectionTarget:execute:1:typed"
    );
}

/// Verifies reflected instance, static, and function targets materialize ordinary callables that
/// preserve their receiver and declared argument behavior.
#[test]
fn test_reflection_get_closure_materializes_tracked_callable_target() {
    let out = compile_and_run(
        r#"<?php
class ReflectedClosureTarget {
    public function __construct(private int $base) {}
    public function add(int $value): int { return $this->base + $value; }
    public static function twice(int $value): int { return $value * 2; }
}
function reflected_increment(int $value): int { return $value + 1; }
$target = new ReflectedClosureTarget(4);
$instance = (new ReflectionMethod(ReflectedClosureTarget::class, 'add'))->getClosure($target);
$static = (new ReflectionMethod(ReflectedClosureTarget::class, 'twice'))->getClosure();
$function = (new ReflectionFunction('reflected_increment'))->getClosure();
echo $instance(3), ':', $static(5), ':', $function(8);
"#,
    );
    assert_eq!(out, "7:10:9");
}

/// Verifies a callable descriptor supplies ReflectionFunction signature metadata and remains
/// invokable through both the reflector and the Closure returned by `getClosure()`.
#[test]
fn test_reflection_function_accepts_callable_descriptor() {
    let out = compile_and_run(
        r#"<?php
$callable = function (string $name, int $count = 3): string {
    return $name . ':' . $count;
};
$reflection = new ReflectionFunction($callable);
$parameters = $reflection->getParameters();
echo count($parameters), ':', $reflection->getNumberOfRequiredParameters(), ':';
echo $parameters[0]->getName(), ':', $parameters[0]->getType()->getName(), ':';
echo $parameters[1]->getDefaultValue(), ':', $reflection->getReturnType()->getName(), ':';
echo $reflection->invoke('Ada', 4), ':';
$copy = $reflection->getClosure();
echo $copy('Lin');
"#,
    );
    assert_eq!(out, "2:1:name:string:3:string:Ada:4:Lin:3");
}

/// Verifies callable-backed ReflectionFunction metadata preserves binding, scope, and staticity.
#[test]
fn test_reflection_function_reports_bound_closure_metadata() {
    let out = compile_and_run(
        r#"<?php
class ReflectedBoundClosureTarget {
    public function make(): Closure {
        return function (): object { return $this; };
    }
}
$target = new ReflectedBoundClosureTarget();
$reflection = new ReflectionFunction($target->make());
echo $reflection->isAnonymous() ? 'anonymous:' : 'named:';
echo $reflection->isStatic() ? 'static:' : 'bound:';
echo $reflection->isClosure() ? 'closure:' : 'function:';
echo $reflection->getClosureThis() === $target ? 'this:' : 'missing:';
echo $reflection->getClosureScopeClass()->getName(), ':';
echo $reflection->getClosureCalledClass()->getName();
"#,
    );
    assert_eq!(
        out,
        "anonymous:bound:closure:this:ReflectedBoundClosureTarget:ReflectedBoundClosureTarget"
    );
}

/// Verifies runtime string targets use registered function metadata instead of requiring a
/// compile-time literal.
#[test]
fn test_reflection_function_accepts_runtime_string_target() {
    let out = compile_and_run(
        r#"<?php
function reflected_runtime_target(string $value = 'ok'): string {
    return strtoupper($value);
}
$name = 'reflected_runtime_target';
$reflection = new ReflectionFunction($name);
echo $reflection->getName(), ':', $reflection->getNumberOfParameters(), ':';
echo $reflection->getNumberOfRequiredParameters(), ':', $reflection->invoke('ready');
"#,
    );
    assert_eq!(out, "reflected_runtime_target:1:0:READY");
}

/// Verifies a gradual class-name operand is resolved and validated by runtime metadata.
#[test]
fn test_reflection_class_accepts_gradual_runtime_target() {
    let out = compile_and_run(
        r#"<?php
class ReflectedRuntimeClassTarget {}
$name = $argc > 0 ? ReflectedRuntimeClassTarget::class : null;
$reflection = new ReflectionClass($name);
echo $reflection->getName();
"#,
    );
    assert_eq!(out, "ReflectedRuntimeClassTarget");
}

/// Verifies source-backed reflection owners expose file and declaration-line metadata.
#[test]
fn test_reflection_source_location_metadata() {
    let out = compile_and_run(
        r#"<?php
class ReflectionSourceTarget {
    public function execute(): void {}
}
function reflection_source_function(): void {}

$class = new ReflectionClass(ReflectionSourceTarget::class);
$object = new ReflectionObject(new ReflectionSourceTarget());
$method = new ReflectionMethod(ReflectionSourceTarget::class, 'execute');
$function = new ReflectionFunction('reflection_source_function');
echo $class->getFileName() !== false ? 'f' : 'x';
echo $object->getFileName() !== false ? 'f' : 'x';
echo $method->getEndLine() >= $method->getStartLine() ? 'e' : 'x';
echo $function->getFileName() !== false ? 'f' : 'x';
"#,
    );
    assert_eq!(out, "ffef");
}

/// Verifies the built-in Reflection attribute-filter flag is available as a class constant.
#[test]
fn test_reflection_attribute_instanceof_filter_constant() {
    let out = compile_and_run("<?php echo ReflectionAttribute::IS_INSTANCEOF;");
    assert_eq!(out, "2");
}

/// Verifies writes in the sole fallthrough `else` branch survive guard restoration so a
/// reflected attribute instance keeps its concrete runtime value after `newInstance()`.
#[test]
fn test_reflection_attribute_instance_survives_terminal_guard_chain() {
    let out = compile_and_run(
        r#"<?php
#[Attribute(Attribute::TARGET_PARAMETER)]
class GuardedAttributePayload {
    public function __construct(public string $value) {}
}
class GuardedAttributeOwner {
    public function execute(#[GuardedAttributePayload('ready')] string $argument): void {}
}
$parameter = (new ReflectionMethod(GuardedAttributeOwner::class, 'execute'))->getParameters()[0];
foreach ([1] as $index) {
    if (false) {
        continue;
    } elseif (!$attribute = $parameter->getAttributes(GuardedAttributePayload::class)[0] ?? null) {
        continue;
    } else {
        $attribute = $attribute->newInstance();
    }
    echo $attribute->value;
}
$arguments = ['$argument' => new GuardedAttributePayload('direct')];
foreach ([1] as $index) {
    if (array_key_exists('$'.$parameter->name, $arguments)) {
        $attribute = $arguments['$'.$parameter->name];
        if (!$attribute instanceof GuardedAttributePayload) {
            continue;
        }
    } elseif (!$attribute = $parameter->getAttributes(GuardedAttributePayload::class)[0] ?? null) {
        continue;
    } else {
        $attribute = $attribute->newInstance();
    }
    echo '|'.$attribute->value;
}
"#,
    );
    assert_eq!(out, "ready|direct");
}
