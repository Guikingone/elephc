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

/// Verifies a runtime class-name reflection query returns filtered class attributes as an array.
#[test]
fn test_reflection_runtime_class_name_filtered_attributes_are_iterable() {
    let out = compile_and_run(
        r#"<?php
#[Attribute]
class RequiredDependency {
    public function __construct(public string $className) {}
}

#[RequiredDependency('base')]
class DependentClass {}

$class = DependentClass::class;
$attributes = (new ReflectionClass($class))->getAttributes(RequiredDependency::class);
foreach ($attributes as $attribute) {
    echo $attribute->newInstance()->className;
}
"#,
    );
    assert_eq!(out, "base");
}

/// Verifies filtered class attributes remain iterable for runtime-declared classes.
#[test]
fn test_reflection_runtime_declared_class_filtered_attributes_are_iterable() {
    let out = compile_and_run(
        r#"<?php
#[Attribute]
class RequiredDependency {
    public function __construct(public string $className) {}
}

$definition = '#[RequiredDependency(\'base\')] class DependentClass {}';
eval($definition);
$class = 'DependentClass';
$filter = 'RequiredDependency';
$attributes = (new ReflectionClass($class))->getAttributes($filter);
foreach ($attributes as $attribute) {
    echo $attribute->newInstance()->className;
}
"#,
    );
    assert_eq!(out, "base");
}

/// Verifies runtime ReflectionClass construction gives attribute-free classes an empty array.
#[test]
fn test_runtime_reflection_attribute_free_class_returns_empty_array() {
    let out = compile_and_run(
        r#"<?php
class PlainReflectionTarget {}

$source = 'return ["PlainReflectionTarget"];';
$classes = eval($source);
$class = $classes[0];
$attributes = (new ReflectionClass($class))->getAttributes();
echo is_array($attributes) ? count($attributes) : 'not-array';
"#,
    );
    assert_eq!(out, "0");
}

/// Verifies a runtime class name preserves AOT attribute metadata and filtering.
#[test]
fn test_runtime_reflection_aot_attribute_filter_returns_iterable_array() {
    let out = compile_and_run(
        r#"<?php
#[Attribute]
class RequiredDependency {
    public function __construct(public string $className) {}
}

#[RequiredDependency('base')]
class DecoratedReflectionTarget {}

$source = 'return ["DecoratedReflectionTarget"];';
$classes = eval($source);
$attributes = (new ReflectionClass($classes[0]))->getAttributes(RequiredDependency::class);
foreach ($attributes as $attribute) {
    echo $attribute->newInstance()->className;
}
"#,
    );
    assert_eq!(out, "base");
}

/// Verifies runtime reflection materializes an AOT attribute `::class` argument as a string.
#[test]
fn test_runtime_reflection_aot_attribute_class_constant_argument_is_iterable() {
    let out = compile_and_run(
        r#"<?php
#[Attribute]
class RequiredDependency {
    public function __construct(public string $className) {}
}

class DependencyTarget {}

#[RequiredDependency(DependencyTarget::class)]
class DecoratedReflectionTarget {}

$source = 'return ["DecoratedReflectionTarget"];';
$classes = eval($source);
$attributes = (new ReflectionClass($classes[0]))->getAttributes(RequiredDependency::class);
foreach ($attributes as $attribute) {
    echo $attribute->newInstance()->className;
}
"#,
    );
    assert_eq!(out, "DependencyTarget");
}

/// Verifies an AOT method preserves runtime reflection attributes from an eval-produced class name.
#[test]
fn test_aot_method_runtime_reflection_attribute_array_is_iterable() {
    let out = compile_and_run(
        r#"<?php
#[Attribute]
class RequiredDependency {
    public function __construct(public string $className) {}
}

#[RequiredDependency('base')]
class DecoratedReflectionTarget {}

class AttributeResolver {
    public function resolve(string $class): string {
        $result = '';
        foreach ((new ReflectionClass($class))->getAttributes(RequiredDependency::class) as $attribute) {
            $result .= $attribute->newInstance()->className;
        }

        return $result;
    }
}

$source = 'return ["DecoratedReflectionTarget"];';
$classes = eval($source);
echo (new AttributeResolver())->resolve($classes[0]);
"#,
    );
    assert_eq!(out, "base");
}

/// Verifies eval-to-AOT by-reference calls preserve runtime reflection attribute arrays.
#[test]
fn test_eval_aot_by_ref_method_runtime_reflection_attributes_are_iterable() {
    let out = compile_and_run(
        r#"<?php
#[Attribute]
class RequiredDependency {
    public function __construct(public string $className) {}
}

#[RequiredDependency('base')]
class DecoratedReflectionTarget {}

class AttributeResolver {
    public function resolve(string $class, array &$resolved, array &$visiting = []): string {
        foreach ((new ReflectionClass($class))->getAttributes(RequiredDependency::class) as $attribute) {
            $resolved[$class] = true;
            $visiting[$class] = true;
            return $attribute->newInstance()->className;
        }

        return 'empty';
    }
}

$source = '$resolved = []; $visiting = []; $resolver = new AttributeResolver(); echo $resolver->resolve("DecoratedReflectionTarget", $resolved, $visiting);';
eval($source);
"#,
    );
    assert_eq!(out, "base");
}

/// Verifies autoloaded attribute metadata remains iterable through an eval-produced class name.
#[test]
fn test_autoloaded_runtime_reflection_attribute_array_is_iterable() {
    let out = compile_and_run_files(
        &[
            (
                "decorated.php",
                r#"<?php
namespace Fixture;

#[\RequiredDependency('base')]
class DecoratedTarget {}
"#,
            ),
            (
                "entry.php",
                r#"<?php
#[Attribute]
class RequiredDependency {
    public function __construct(public string $className) {}
}

class AttributeResolver {
    public function resolve(string $class): string {
        $result = '';
        foreach ((new ReflectionClass($class))->getAttributes(RequiredDependency::class) as $attribute) {
            $result .= $attribute->newInstance()->className;
        }

        return $result;
    }
}

spl_autoload_register(function (string $class): void {
    if ($class === 'Fixture\\DecoratedTarget') {
        require __DIR__ . '/decorated.php';
    }
});

$source = 'return ["Fixture\\DecoratedTarget"];';
$classes = eval($source);
if (class_exists($classes[0])) {
    echo (new AttributeResolver())->resolve($classes[0]);
}
"#,
            ),
        ],
        "entry.php",
    );
    assert_eq!(out, "base");
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
echo $reflection->name, ':', $reflection->getName();
"#,
    );
    assert_eq!(out, "ReflectedRuntimeClassTarget:ReflectedRuntimeClassTarget");
}

/// Verifies an eval-materialized ReflectionClass exposes the native public name property.
#[test]
fn test_eval_materialized_reflection_class_exposes_public_name() {
    let out = compile_and_run(
        r#"<?php
class EvalReflectedRuntimeClassTarget {}
$inside = eval('$reflection = new ReflectionClass("EvalReflectedRuntimeClassTarget"); return $reflection->name;');
$reflection = eval('return new ReflectionClass("EvalReflectedRuntimeClassTarget");');
echo $inside, ':', $reflection->name, ':', $reflection->getName();
"#,
    );
    assert_eq!(
        out,
        "EvalReflectedRuntimeClassTarget:EvalReflectedRuntimeClassTarget:EvalReflectedRuntimeClassTarget"
    );
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
$objectFile = $object->getFileName();
echo is_string($objectFile) && '' !== $objectFile ? 'f' : 'x';
echo $method->getEndLine() >= $method->getStartLine() ? 'e' : 'x';
echo $function->getFileName() !== false ? 'f' : 'x';
"#,
    );
    assert_eq!(out, "ffef");
}

/// Verifies ReflectionObject on `$this` uses the lexical hierarchy to select runtime metadata.
#[test]
fn test_reflection_object_this_source_file_is_non_empty() {
    let out = compile_and_run(
        r#"<?php
class ReflectionThisSourceBase {
    public function sourceFile(): string|bool {
        return (new ReflectionObject($this))->getFileName();
    }
}
final class ReflectionThisSourceChild extends ReflectionThisSourceBase {}
$file = (new ReflectionThisSourceChild())->sourceFile();
echo is_string($file) && '' !== $file ? 'file' : 'missing';
"#,
    );
    assert_eq!(out, "file");
}

/// Verifies a direct runtime ReflectionObject source-file query avoids eagerly
/// materializing unrelated Reflection metadata while preserving the file result.
#[test]
fn test_direct_reflection_object_this_source_file_is_non_empty() {
    let out = compile_and_run(
        r#"<?php
class DirectReflectionSourceBase {
    public function sourceFile(): string|bool {
        return (new ReflectionObject($this))->getFileName();
    }
}
final class DirectReflectionSourceChild extends DirectReflectionSourceBase {}
$file = (new DirectReflectionSourceChild())->sourceFile();
echo is_string($file) && '' !== $file ? 'file' : 'missing';
"#,
    );
    assert_eq!(out, "file");
}

/// Verifies a source-only ReflectionObject query through an interface-typed property.
#[test]
fn test_reflection_object_interface_property_source_file_is_non_empty() {
    let out = compile_and_run(
        r#"<?php
interface ReflectionSourceProvider { public function configure(): void; }
final class ReflectionSourceProviderImpl implements ReflectionSourceProvider {
    public function configure(): void {}
}
final class ReflectionSourceConsumer {
    public function __construct(private ReflectionSourceProvider $subject) {}
    public function sourceFile(): string|bool {
        return (new ReflectionObject($this->subject))->getFileName();
    }
}
$file = (new ReflectionSourceConsumer(new ReflectionSourceProviderImpl()))->sourceFile();
echo is_string($file) && '' !== $file ? 'file' : 'missing';
"#,
    );
    assert_eq!(out, "file");
}

/// Verifies a local ReflectionObject may read both source metadata fields without full members.
#[test]
fn test_reflection_object_local_source_file_and_name_are_available() {
    let out = compile_and_run(
        r#"<?php
interface ReflectionSourceMetadataProvider { public function configure(): void; }
final class ReflectionSourceMetadataProviderImpl implements ReflectionSourceMetadataProvider {
    public function configure(): void {}
}
final class ReflectionSourceMetadataConsumer {
    public function __construct(private ReflectionSourceMetadataProvider $subject) {}
    public function sourceMetadata(): string {
        $reflection = new ReflectionObject($this->subject);
        $file = $reflection->getFileName();
        return (is_string($file) && '' !== $file ? 'file:' : 'missing:').$reflection->name;
    }
}
echo (new ReflectionSourceMetadataConsumer(new ReflectionSourceMetadataProviderImpl()))->sourceMetadata();
"#,
    );
    assert_eq!(out, "file:ReflectionSourceMetadataProviderImpl");
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

/// Pins that reflecting an object whose method parameter names another class stays bounded.
///
/// ⚠️ This does NOT reproduce the open `--web` defect, and must not be mistaken for it: reflecting
/// an object whose parameter is typed `Symfony\Component\DependencyInjection\ContainerBuilder`
/// exhausts the heap identically at `--heap-size` 8 MiB and 1 GiB, where php charges 96 bytes for
/// the same handle. Every attempt to reproduce that locally has passed — including a
/// self-referential class with 40 methods, 120 classes x 12 methods, traits, attributes and
/// associative class constants. The trigger is not class size, cycles, or any feature reproduced
/// here. See `.plans/reflection-slot-reachability.md`; this case only guards the shape from
/// regressing further.
#[test]
fn test_reflection_object_with_class_typed_parameter_stays_bounded() {
    let out = compile_and_run(
        r#"<?php
class Wide {
    public function a(Wide $x, ?Wide $y = null): ?Wide { return null; }
    public function b(Wide $x, ?Wide $y = null): ?Wide { return null; }
    public function c(Wide $x, ?Wide $y = null): ?Wide { return null; }
    public function d(Wide $x, ?Wide $y = null): ?Wide { return null; }
}
class Narrow {
    public function m(Wide $w): void {}
}
$r = new ReflectionObject(new Narrow());
echo $r->getName(), "|", count($r->getMethods());
"#,
    );
    assert_eq!(out, "Narrow|1");
}

/// Verifies `getFileName()` reports the file a class was DECLARED in, not the entry file.
///
/// Declaration-to-file attribution ran after `require` targets were spliced into their parent
/// program, so a class written in another file was either attributed to whichever file included it
/// or — for the entry program's own includes — recorded nowhere at all, at which point
/// `reflection_source_file` fell back to the module source path. The fallback always yields a real
/// existing path, which is what kept this silent: `is_file()` stayed true and Symfony's
/// `getProjectDir()` even recovered, because its walk-up to `composer.json` compensated.
#[test]
fn test_reflection_get_file_name_reports_the_declaring_file() {
    // Driven through the real CLI: the in-process multi-file helper is a hand-rolled pipeline
    // that never populates declaration-to-file attribution at all, so it would observe the
    // fallback no matter what the compiler does.
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                "<?php\nrequire __DIR__ . '/loader.php';\n$o = new \\Inc\\Widget();\n$r = new \\ReflectionObject($o);\necho basename((string) $r->getFileName()), '|', $o->file() === $r->getFileName() ? 'same' : 'different';\n",
            ),
            ("loader.php", "<?php\nrequire __DIR__ . '/Widget.php';\n"),
            (
                "Widget.php",
                "<?php\nnamespace Inc;\nclass Widget { public function file(): string { return __FILE__; } }\nfunction widget_file(): string { return __FILE__; }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "Widget.php|same");
}

/// Verifies the deprecated `ReflectionParameter::getClass()` still reports the parameter's class
/// when the program actually calls it.
///
/// This is the over-pruning guard for the slot-reachability gate: it is the test that fails if the
/// gate ever decides a slot is dead while a reachable accessor can still read it.
#[test]
fn test_reflection_parameter_get_class_still_reports_the_parameter_class() {
    let out = compile_and_run(
        r#"<?php
class Dep {}
class Uses {
    public function m(Dep $d): void {}
}
$r = new ReflectionObject(new Uses());
$p = $r->getMethod('m')->getParameters()[0];
$c = $p->getClass();
echo $c === null ? "null" : $c->getName();
"#,
    );
    assert_eq!(out, "Dep");
}
