//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object-oriented PHP class modifiers and properties, including readonly class constructor initialization, final class instantiates and dispatches methods, and final method dispatches normally without override.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Uses checked-in example PHP fixtures through include_str! in addition to inline native-output assertions.

use super::*;

/// Verifies that a `readonly` class permits property initialization inside its constructor.
/// The property is assigned in `__construct` and read back via `$user->id`.
#[test]
fn test_readonly_class_constructor_initialization() {
    let out = compile_and_run(
        r#"<?php
readonly class User {
    public $id;

    public function __construct($id) {
        $this->id = $id;
    }
}

$user = new User(42);
echo $user->id;
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies that a `final` class can be instantiated and that method calls on the
/// resulting object dispatch correctly (no vtable override possible).
#[test]
fn test_final_class_instantiates_and_dispatches_methods() {
    let out = compile_and_run(
        r#"<?php
final class Receipt {
    public $code = 41;

    public function next() {
        return $this->code + 1;
    }
}

$receipt = new Receipt();
echo $receipt->next();
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies that a `final` method on a base class is callable on a child instance
/// and does not permit overriding. The child defines a separate method to confirm
/// the child object is fully functional.
#[test]
fn test_final_method_dispatches_normally_without_override() {
    let out = compile_and_run(
        r#"<?php
class Base {
    final public function label() {
        return "base";
    }
}

class Child extends Base {
    public function suffix() {
        return "child";
    }
}

$child = new Child();
echo $child->label();
echo ":";
echo $child->suffix();
"#,
    );
    assert_eq!(out, "base:child");
}

/// Verifies that a `final` property on a base class is readable by a child instance
/// and that a method on the child can read and augment the property value.
#[test]
fn test_final_property_reads_normally_without_override() {
    let out = compile_and_run(
        r#"<?php
class Base {
    final public $value = 40;

    public function value() {
        return $this->value + 2;
    }
}

class Child extends Base {
    public function label() {
        return "answer:";
    }
}

$child = new Child();
echo $child->label();
echo $child->value();
"#,
    );
    assert_eq!(out, "answer:42");
}

/// Verifies typed instance properties with mixed initialization strategies:
/// constructor-only assignment, class-level defaults, nullable with null default,
/// and that reading then assigning via `$user->email` produces the expected output.
#[test]
fn test_typed_properties_defaults_constructor_assignment_and_nullable() {
    let out = compile_and_run(
        r#"<?php
class User {
    public int $id;
    public string $name = "Ada";
    public ?string $email = null;

    public function __construct($id) {
        $this->id = $id;
    }

    public function label() {
        return $this->name . ":" . $this->id;
    }
}

$user = new User(42);
echo $user->label();
echo ":";
echo is_null($user->email);
$user->email = "ada@example.test";
echo ":";
echo $user->email;
"#,
    );
    assert_eq!(out, "Ada:42:1:ada@example.test");
}

/// Verifies that accessing a typed instance property before it is initialized
/// throws a catchable `Error` that `catch(\Error $e)` can observe.
#[test]
fn test_uninitialized_typed_instance_property_is_fatal() {
    let out = compile_and_run(
        r#"<?php
class Box {
    public int $value;
}

$box = new Box();
try {
    echo $box->value;
} catch (\Error $e) {
    echo $e->getMessage();
}
"#,
    );
    assert!(
        out.contains("Typed property Box::$value must not be accessed before initialization"),
        "{out}"
    );
}

/// Verifies that explicitly assigning `0` to a typed instance property constitutes
/// valid initialization and the property reads back as `0`.
#[test]
fn test_typed_instance_property_initialized_to_zero_reads_normally() {
    let out = compile_and_run(
        r#"<?php
class Box {
    public int $value;
}

$box = new Box();
$box->value = 0;
echo $box->value;
"#,
    );
    assert_eq!(out, "0");
}

/// Verifies that an inherited private typed property without a default starts
/// uninitialized on instances of the child class, so `isset()` can initialize it.
#[test]
fn test_inherited_private_typed_property_without_default_starts_uninitialized() {
    let out = compile_and_run(
        r#"<?php
class ParentBox {
    private string $path;

    public function path(): string {
        if (!isset($this->path)) {
            $this->path = "parent";
        }

        return $this->path;
    }
}

class ChildBox extends ParentBox {}

echo (new ChildBox())->path();
"#,
    );
    assert_eq!(out, "parent");
}

/// Verifies that cloning a child preserves the uninitialized marker of a
/// private typed property declared by an ancestor.
#[test]
fn test_clone_preserves_transitively_inherited_private_typed_property_marker() {
    let out = compile_and_run(
        r#"<?php
class CloneRoot {
    private string $path;

    public function path(): string {
        if (!isset($this->path)) {
            $this->path = "cloned";
        }

        return $this->path;
    }
}

class CloneMiddle extends CloneRoot {}
class CloneChild extends CloneMiddle {}

$copy = clone new CloneChild();
echo $copy->path();
"#,
    );
    assert_eq!(out, "cloned");
}

/// Verifies that a dynamic-evaluation context preserves uninitialized markers
/// when it constructs an AOT class with a private typed ancestor property.
#[test]
fn test_eval_new_preserves_transitively_inherited_private_typed_property_marker() {
    let out = compile_and_run(
        r#"<?php
class EvalRoot {
    private string $path;

    public function path(): string {
        if (!isset($this->path)) {
            $this->path = "evaluated";
        }

        return $this->path;
    }
}

class EvalMiddle extends EvalRoot {}
class EvalChild extends EvalMiddle {}

eval('$value = new EvalChild(); echo $value->path();');
"#,
    );
    assert_eq!(out, "evaluated");
}

/// Verifies that a private typed property without a default remains
/// uninitialized through multiple inheritance levels until its declaring class initializes it.
#[test]
fn test_transitively_inherited_private_typed_property_without_default_starts_uninitialized() {
    let out = compile_and_run(
        r#"<?php
class RootBox {
    private string $path;

    public function path(): string {
        if (!isset($this->path)) {
            $this->path = "root";
        }

        return $this->path;
    }
}

class MiddleBox extends RootBox {}
class ChildBox extends MiddleBox {}

echo (new ChildBox())->path();
"#,
    );
    assert_eq!(out, "root");
}

/// Verifies that a closure-created child instance preserves transitive private
/// typed-property initialization metadata through the dynamic callable path.
#[test]
fn test_closure_created_transitive_private_typed_property_starts_uninitialized() {
    let out = compile_and_run(
        r#"<?php
class RootBox {
    private string $path;

    public function path(): string {
        if (!isset($this->path)) {
            $this->path = "closure";
        }

        return $this->path;
    }
}

class MiddleBox extends RootBox {}
class ChildBox extends MiddleBox {}

$factory = static fn (): ChildBox => new ChildBox();
echo $factory()->path();
"#,
    );
    assert_eq!(out, "closure");
}

/// Verifies lazy source-directory discovery across transitive inheritance,
/// reflection, and a closure-created object without depending on a fixture path.
#[test]
fn test_transitive_private_typed_property_lazy_reflection_directory_initialization() {
    let out = compile_cli_files_and_run(
        &[
            ("project-root.marker", "root\n"),
            (
                "src/RootBox.php",
                r#"<?php
namespace App;

function pass_through(mixed $value): mixed {
    return $value;
}

trait ParentStorage {
    private array $entries = [];
}

trait ChildStorage {
    private array $entries = [];
}

class RootBox {
    protected array $bundles = [];
    protected mixed $container = null;
    protected bool $booted = false;
    protected ?float $startTime = null;
    private string $directory;

    public function __construct(
        protected string $environment,
        protected bool $debug,
    ) {}

    public function directory(): string {
        if (!isset($this->directory)) {
            $file = pass_through((new \ReflectionObject($this))->getFileName());
            if (!is_file($file)) {
                return "missing";
            }
            $directory = dirname($file);
            while (!is_file($directory . "/project-root.marker")) {
                if ($directory === dirname($directory)) {
                    return "root";
                }
                $directory = dirname($directory);
            }
            $this->directory = $directory;
        }

        return $this->directory;
    }
}
"#,
            ),
            (
                "src/MiddleBox.php",
                "<?php\nnamespace App;\nclass MiddleBox extends RootBox { use ParentStorage; private ?string $warmup = null; private int $counter = 0; private bool $reset = false; }\n",
            ),
            (
                "src/ChildBox.php",
                "<?php\nnamespace App;\nclass ChildBox extends MiddleBox { use ChildStorage; }\n",
            ),
            (
                "entry/main.php",
                "<?php\nrequire __DIR__ . '/../src/RootBox.php';\nrequire __DIR__ . '/../src/MiddleBox.php';\nrequire __DIR__ . '/../src/ChildBox.php';\n$factory = static fn (): \\App\\ChildBox => new \\App\\ChildBox('dev', true);\n$directory = $factory()->directory();\necho is_file($directory . '/project-root.marker') ? 'ready' : 'wrong';\n",
            ),
        ],
        "entry/main.php",
    );
    assert_eq!(out, "ready");
}

/// Verifies a dynamic include preserves the lexical class scope for a private method bridge.
#[test]
fn test_dynamic_include_preserves_private_method_scope() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
function scope_identity(mixed $value): mixed { return $value; }

trait PrivateScopeAfterDynamicInclude {
    private string $includePath;

    private function privateLabel(array &$visited, array &$visiting = []): string {
        $visited['called'] = true;
        $visiting['entered'] = true;
        return 'private-ok';
    }

    private function includeThenCallPrivate(): string {
        $path = $this->includePath;
        require $path;
        $self = scope_identity($this);
        $visited = [];
        $label = $self->privateLabel($visited);
        $copy = $visited;
        $copy['copied'] = true;
        return $label . '-' . $visited['called'] . '-' . $copy['copied'];
    }
}

class ScopeCarrier {
    use PrivateScopeAfterDynamicInclude;

    public function __construct(string $path) { $this->includePath = $path; }
    public function run(): string { return $this->includeThenCallPrivate(); }
}

final class ScopeCarrierChild extends ScopeCarrier {}

echo (new ScopeCarrierChild(__DIR__.'/payload.php'))->run();
"#,
            ),
            ("payload.php", "<?php $included = true;\n"),
        ],
        "entry.php",
    );
    assert_eq!(out, "private-ok-1-1");
}

/// Verifies a dynamically included class retains constructor parameters from a protected method.
#[test]
fn test_dynamic_include_class_constructor_keeps_associative_parameters() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
$path = __DIR__ . '/definition.php';
require $path;
$class = 'ParameterCarrier';
$carrier = new $class();
echo (($carrier->defaults()['kernel.debug'] ?? false) ? 'd' : 'x') . ($carrier->parameter('kernel.debug') ? 'p' : 'q');
"#,
            ),
            (
                "definition.php",
                r#"<?php
class ParameterCarrier {
    protected array $parameters = [];

    public function __construct() {
        $this->parameters = $this->getDefaultParameters();
    }

    protected function getDefaultParameters(): array {
        return ['kernel.debug' => true];
    }

    public function defaults(): array {
        return $this->getDefaultParameters();
    }

    public function parameter(string $name): mixed {
        if (array_key_exists($name, $this->parameters) && '.' !== ($name[0] ?? '')) {
            return $this->parameters[$name];
        }

        return null;
    }
}
"#,
            ),
        ],
        "entry.php",
    );
    assert_eq!(out, "dp");
}

/// Verifies an associative map returned by a dynamic require reaches typed AOT method arguments.
///
/// The method receives a PHP `array` contract while the runtime value is hash-backed. Its nested
/// traversal and by-reference accumulator must retain the associative storage rather than
/// reinterpreting it as an indexed array.
#[test]
fn test_dynamic_require_assoc_map_survives_typed_method_arguments() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
class BundleResolver {
    public function resolve(array $bundles): array {
        $resolved = [];
        foreach ($bundles as $class => $envs) {
            $this->append($class, $envs, $bundles, $resolved);
        }

        return $resolved;
    }

    private function append(string $class, array $envs, array $bundles, array &$resolved, array &$visiting = []): void {
        $visiting[$class] = true;
        if (!isset($bundles[$class])) {
            return;
        }
        $resolved[$class] = $envs;
    }
}

$path = __DIR__ . '/bundles.php';
$bundles = is_file($path) ? require $path : [];
$resolved = (new BundleResolver())->resolve($bundles);
echo $resolved['Fixture\\Bundle']['all'] ? 'resolved' : 'missing';
"#,
            ),
            (
                "bundles.php",
                "<?php\nreturn ['Fixture\\\\Bundle' => ['all' => true]];\n",
            ),
        ],
        "entry.php",
    );
    assert_eq!(out, "resolved");
}

/// Verifies a dynamic subclass updates a protected property used by an AOT parent method.
#[test]
fn test_dynamic_subclass_syncs_redeclared_protected_parent_property() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
class ParameterBase {
    protected array $parameters = [];

    public function parameter(string $name): mixed {
        return $this->parameters[$name] ?? null;
    }
}

$path = __DIR__ . '/definition.php';
require $path;
$class = 'ParameterCarrier';
echo (new $class())->parameter('kernel.debug') ? 'ok' : 'wrong';
"#,
            ),
            (
                "definition.php",
                r#"<?php
class ParameterCarrier extends ParameterBase {
    protected array $parameters = [];

    public function __construct() {
        $this->parameters = ['kernel.debug' => true];
    }
}
"#,
            ),
        ],
        "entry.php",
    );
    assert_eq!(out, "ok");
}

/// Verifies an eval barrier preserves an inherited boolean `$this` property.
///
/// `$this` is not assignable by PHP code, including dynamically included code. The following
/// trait method must therefore keep its concrete object representation across the eval barrier
/// and read the inherited boolean without routing the raw scalar through `__rt_mixed_cast_bool`.
#[test]
fn test_dynamic_include_preserves_inherited_bool_this_property_truthiness() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
trait DynamicBarrierBoolTrait {
    protected function checkDynamicBarrierBool(): void {
        $path = str_replace('/fixture/', '/fixture/', __DIR__ . '/fixture/declaration.php');
        require $path;

        if ($this->debug && !defined('DYNAMIC_BARRIER_BOOL_DISABLED')) {
            echo 'ok';
        }
    }
}

class DynamicBarrierBoolBase {
    public function __construct(protected bool $debug) {}
}

class DynamicBarrierBoolChild extends DynamicBarrierBoolBase {
    use DynamicBarrierBoolTrait;

    public function run(): void {
        $this->checkDynamicBarrierBool();
    }
}

(new DynamicBarrierBoolChild(true))->run();
"#,
            ),
            ("fixture/declaration.php", "<?php\nfunction dynamic_barrier_marker(): void {}\n"),
        ],
        "entry.php",
    );
    assert_eq!(out, "ok");
}

/// Verifies a namespaced dynamically included class reads its protected associative property.
#[test]
fn test_namespaced_dynamic_class_reads_protected_associative_property() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
$path = __DIR__ . '/definition.php';
require $path;
$class = 'Generated\\ParameterCarrier';
echo (new $class())->parameter('kernel.debug') ? 'ok' : 'wrong';
"#,
            ),
            (
                "definition.php",
                r#"<?php
namespace Generated;

class ParameterCarrier {
    protected array $parameters = [];

    public function __construct() {
        $this->parameters = ['kernel.debug' => true];
    }

    public function parameter(string $name): mixed {
        if (array_key_exists($name, $this->parameters) && '.' !== ($name[0] ?? '')) {
            return $this->parameters[$name];
        }

        return null;
    }
}
"#,
            ),
        ],
        "entry.php",
    );
    assert_eq!(out, "ok");
}

/// Verifies a generated-style dynamic getter finds values in its protected parameter map.
#[test]
fn test_dynamic_generated_style_parameter_getter_reads_protected_map() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
$path = __DIR__ . '/definition.php';
require $path;
$class = 'Generated\\ParameterCarrier';
echo (new $class())->getParameter('kernel.debug') ? 'ok' : 'wrong';
"#,
            ),
            (
                "definition.php",
                r#"<?php
namespace Generated;

class ParameterCarrier {
    private const DEPRECATED_PARAMETERS = [];
    private const NONEMPTY_PARAMETERS = [];
    private array $loadedDynamicParameters = [];
    private array $dynamicParameters = [];
    protected array $parameters = [];

    public function __construct(private array $buildParameters = [], protected string $containerDir = __DIR__) {
        $this->parameters = $this->getDefaultParameters();
    }

    protected function getDefaultParameters(): array {
        return [
            'kernel.project_dir' => dirname(__DIR__, 2),
            'kernel.environment' => 'dev',
            'kernel.debug' => true,
            'kernel.bundles' => [
                'FrameworkBundle' => 'FrameworkBundle',
            ],
            'kernel.bundles_metadata' => [
                'FrameworkBundle' => [
                    'path' => dirname(__DIR__),
                    'namespace' => 'Framework',
                ],
            ],
            '.known_environments' => ['prod', 'dev', 'test'],
            'kernel.logs_dir' => dirname(__DIR__) . '/log',
            'kernel.http_method_override' => false,
            'session.storage.options' => [
                'cookie_secure' => 'auto',
                'cookie_httponly' => true,
            ],
        ];
    }

    public function getParameter(string $name): mixed {
        if (isset(self::DEPRECATED_PARAMETERS[$name])) {
            return null;
        }

        if (array_key_exists($name, $this->buildParameters)) {
            return $this->buildParameters[$name];
        }

        if (isset($this->loadedDynamicParameters[$name])) {
            return $this->loadedDynamicParameters[$name]
                ? $this->dynamicParameters[$name]
                : null;
        }

        if (array_key_exists($name, $this->parameters) && '.' !== ($name[0] ?? '')) {
            return $this->parameters[$name];
        }

        return null;
    }
}
"#,
            ),
        ],
        "entry.php",
    );
    assert_eq!(out, "ok");
}

/// Verifies a statically typed AOT call dispatches to a runtime-defined subclass override.
#[test]
fn test_typed_aot_method_call_dispatches_to_dynamic_subclass_override() {
    let out = compile_and_run(
        r#"<?php
class BaseMessage {
    public function label(): string {
        return 'base';
    }
}

function label_for(BaseMessage $message): string {
    return $message->label();
}

$definition = 'namespace Generated; class Message extends \\BaseMessage { public function label(): string { return \'dynamic\'; } }';
eval($definition);
$class = 'Generated\\Message';
echo label_for(new $class()) . label_for(new BaseMessage());
"#,
    );
    assert_eq!(out, "dynamicbase");
}

/// Verifies that accessing an uninitialized typed static property throws a
/// catchable `Error` that `catch(\Error $e)` can observe.
#[test]
fn test_uninitialized_typed_static_property_is_fatal() {
    let out = compile_and_run(
        r#"<?php
class Box {
    public static int $value;
}

try {
    echo Box::$value;
} catch (\Error $e) {
    echo $e->getMessage();
}
"#,
    );
    assert!(
        out.contains("Typed static property Box::$value must not be accessed before initialization"),
        "{out}"
    );
}

/// Verifies that accessing an uninitialized typed property and catching `Error`
/// allows continued execution after the property is initialized (issue #339).
#[test]
fn test_uninitialized_property_catch_then_continue() {
    let out = compile_and_run(
        r#"<?php
class C { public int $x; }
$c = new C();
try { echo $c->x; } catch (\Error $e) { echo "e"; }
$c->x = 5;
echo $c->x;
"#,
    );
    assert_eq!(out, "e5");
}

/// Verifies that catching `Exception` (not `Error`) does NOT catch the
/// uninitialized typed property access (issue #339).
#[test]
fn test_uninitialized_property_catch_exception_does_not_match() {
    let err = compile_and_run_expect_failure(
        r#"<?php
class C { public int $x; }
$c = new C();
try { echo $c->x; } catch (\Exception $e) { echo "caught"; }
echo "end";
"#,
    );
    // Asserting the CLASS is what this test is actually about: the `catch (\Exception $e)` must
    // not match because an uninitialized typed property raises an `Error`. The previous
    // assertion only looked for the word "uncaught", so a fatal of any class would have passed —
    // including the very `Exception` the test exists to rule out.
    //
    // Byte-identical to reference PHP 8.5.6 up to its ` in <file>:<line>` suffix, which elephc
    // cannot emit (see issue #660).
    assert!(
        err.contains(
            "Fatal error: Uncaught Error: Typed property C::$x must not be accessed before initialization"
        ),
        "{err}"
    );
}
/// Verifies a typed static property assigned zero remains distinguishable from uninitialized.
#[test]
fn test_typed_static_property_initialized_to_zero_reads_normally() {
    let out = compile_and_run(
        r#"<?php
class Box {
    public static int $value;
}

Box::$value = 0;
echo Box::$value;
"#,
    );
    assert_eq!(out, "0");
}

/// Verifies typed instance and static property defaults resolve class-like constants only after
/// class, parent, and interface metadata is complete.
#[test]
fn test_typed_property_defaults_resolve_scoped_constants() {
    let out = compile_and_run(
        r#"<?php
interface Defaults {
    public const VALUE = 41;
}

class ParentBox {
    protected const OFFSET = 1;
}

class Box extends ParentBox implements Defaults {
    private const LOCAL = 40;

    public int $interfaceValue = Defaults::VALUE;
    public int $selfValue = self::LOCAL;
    public int $parentValue = parent::OFFSET;
    public static int $staticValue = Defaults::VALUE;
}

$box = new Box();
echo $box->interfaceValue, ":", $box->selfValue, ":", $box->parentValue, ":", Box::$staticValue;
"#,
    );
    assert_eq!(out, "41:40:1:41");
}

/// Verifies scoped constants nested in literal property defaults are resolved for instance and
/// static storage, including an instance selected by a runtime class string.
#[test]
fn test_array_property_defaults_resolve_nested_scoped_constants() {
    let out = compile_and_run(
        r#"<?php
class DefaultMap {
    private const CREATED = 'c';
    private const ALIGN = 'left';

    public array $values = [self::CREATED => 1, 'align' => self::ALIGN];
    public static array $staticValues = [self::CREATED => 2, 'align' => self::ALIGN];
}

function instantiate(string $class): object {
    return new $class();
}

$direct = new DefaultMap();
$dynamic = instantiate('DefaultMap');
echo $direct->values['c'], ':', $dynamic->values['align'], ':';
echo DefaultMap::$staticValues['c'], ':', DefaultMap::$staticValues['align'];
"#,
    );
    assert_eq!(out, "1:left:2:left");
}

/// Verifies that a nullable typed static property with an explicit `= null` default
/// is considered initialized (`is_null()` returns true), and that a typed static
/// property without a default remains uninitialized and throws a catchable Error;
/// without a try/catch the no-handler fast path still reports the specific
/// fatal diagnostic (issue #339).
#[test]
fn test_nullable_static_property_default_null_is_initialized() {
    let out = compile_and_run(
        r#"<?php
class WithDefault {
    public static ?int $value = null;
}
echo is_null(WithDefault::$value);
"#,
    );
    assert_eq!(out, "1");

    let err = compile_and_run_expect_failure(
        r#"<?php
class WithoutDefault {
    public static ?int $value;
}

echo WithoutDefault::$value;
"#,
    );
    assert!(
        err.contains("Fatal error: Typed static property WithoutDefault::$value must not be accessed before initialization"),
        "{err}"
    );
}

/// Verifies a nullable integer narrowed by a null check can populate an integer static property.
#[test]
fn test_narrowed_nullable_int_static_property_assignment() {
    let out = compile_and_run(
        r#"<?php
class StaticIntTarget {
    public static int $value = 0;

    public static function assign(?int $value): void {
        if (null !== $value) {
            self::$value = $value;
        }
    }
}
StaticIntTarget::assign(7);
echo StaticIntTarget::$value;
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies that an untyped instance property with a `= null` default is strictly
/// null (`=== null` is true, `== null` is true) and that `var_dump` emits `NULL`.
#[test]
fn test_untyped_null_property_default_is_strictly_null() {
    let out = compile_and_run(
        r#"<?php
class A { public $x = null; }
$a = new A();
var_dump($a->x);
echo is_null($a->x) ? "y" : "n", "\n";
echo ($a->x === null) ? "y" : "n", "\n";
echo ($a->x !== null) ? "y" : "n", "\n";
echo ($a->x == null) ? "y" : "n", "\n";
"#,
    );
    assert_eq!(out, "NULL\ny\ny\nn\ny\n");
}

/// Verifies that an untyped static property with a `= null` default is strictly
/// null (`=== null` is true, `== null` is true) and that `var_dump` emits `NULL`.
#[test]
fn test_untyped_static_null_property_default_is_strictly_null() {
    let out = compile_and_run(
        r#"<?php
class A { public static $x = null; }
var_dump(A::$x);
echo is_null(A::$x) ? "y" : "n", "\n";
echo (A::$x === null) ? "y" : "n", "\n";
echo (A::$x !== null) ? "y" : "n", "\n";
echo (A::$x == null) ? "y" : "n", "\n";
"#,
    );
    assert_eq!(out, "NULL\ny\ny\nn\ny\n");
}

/// Verifies that assigning `null` to an untyped instance or static property that
/// previously held a non-null value results in a strictly-null value
/// (`=== null` is true).
#[test]
fn test_untyped_property_assignment_to_null_is_strictly_null() {
    let out = compile_and_run(
        r#"<?php
class A {
    public $x = 1;
    public static $y = 1;
}
$a = new A();
$a->x = null;
A::$y = null;
echo is_null($a->x) ? "y" : "n", "\n";
echo ($a->x === null) ? "y" : "n", "\n";
echo is_null(A::$y) ? "y" : "n", "\n";
echo (A::$y === null) ? "y" : "n", "\n";
"#,
    );
    assert_eq!(out, "y\ny\ny\ny\n");
}

/// Verifies that a static property on a `readonly` class is mutable
/// (readonly only affects instance properties, not static ones).
#[test]
fn test_readonly_class_static_property_is_mutable() {
    let out = compile_and_run(
        r#"<?php
readonly class Counter {
    public static int $count = 0;
}
Counter::$count = 5;
echo Counter::$count;
Counter::$count = Counter::$count + 1;
echo ":";
echo Counter::$count;
"#,
    );
    assert_eq!(out, "5:6");
}

/// Verifies that a static property on an `abstract readonly` class is mutable,
/// matching the behaviour of plain `readonly` classes.
#[test]
fn test_readonly_abstract_class_static_property_is_mutable() {
    let out = compile_and_run(
        r#"<?php
abstract readonly class Counter {
    public static int $count = 0;
}
Counter::$count = 7;
echo Counter::$count;
Counter::$count = Counter::$count + 1;
echo ":";
echo Counter::$count;
"#,
    );
    assert_eq!(out, "7:8");
}

/// Verifies that a static property inherited through a `readonly` child class
/// remains mutable and that both base and child share the same static slot.
#[test]
fn test_readonly_inherited_static_property_remains_mutable() {
    let out = compile_and_run(
        r#"<?php
readonly class Base {
    public static int $shared = 1;
}
readonly class Child extends Base {
}
Child::$shared = 42;
echo Base::$shared;
echo ":";
echo Child::$shared;
"#,
    );
    assert_eq!(out, "42:42");
}

/// End-to-end smoke test using the checked-in `examples/final-classes/main.php`
/// fixture. Verifies the example compiles, runs, and produces `"invoice:42\n"`.
#[test]
fn test_example_final_classes_compiles_and_runs() {
    let out = compile_and_run(include_str!("../../../examples/final-classes/main.php"));
    assert_eq!(out, "invoice:42\n");
}

/// End-to-end smoke test using the checked-in `examples/typed-properties/main.php`
/// fixture. Verifies the example compiles, runs, and produces `"Ada:42\nmissing email\n"`.
#[test]
fn test_example_typed_properties_compiles_and_runs() {
    let out = compile_and_run(include_str!("../../../examples/typed-properties/main.php"));
    assert_eq!(out, "Ada:42\nmissing email\n");
}

/// Verifies PHP 8.4 asymmetric visibility at runtime: a `public private(set)` property is
/// writable from inside the class and readable from outside.
#[test]
fn test_asymmetric_visibility_internal_write_external_read() {
    let out = compile_and_run(
        "<?php
        class Counter {
            public private(set) int $value = 0;
            public function increment(): void { $this->value = $this->value + 1; }
        }
        $c = new Counter();
        $c->increment();
        $c->increment();
        echo $c->value;
        ",
    );
    assert_eq!(out, "2");
}

/// Verifies that a subclass may write a `protected(set)` property inherited from its parent.
#[test]
fn test_asymmetric_visibility_protected_set_subclass_write() {
    let out = compile_and_run(
        "<?php
        class Base { public protected(set) string $name = \"base\"; }
        class Derived extends Base {
            public function rename(string $n): void { $this->name = $n; }
        }
        $d = new Derived();
        $d->rename(\"derived\");
        echo $d->name;
        ",
    );
    assert_eq!(out, "derived");
}

/// Compiles and runs the checked-in `examples/asymmetric-visibility/main.php` fixture, which
/// models an account whose balance is publicly readable but only privately writable.
#[test]
fn test_example_asymmetric_visibility_compiles_and_runs() {
    let out = compile_and_run(include_str!("../../../examples/asymmetric-visibility/main.php"));
    assert_eq!(out, "balance: 120\ninsufficient funds\nbalance: 120\n");
}
