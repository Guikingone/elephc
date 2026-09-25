//! Purpose:
//! Pins PHP object type identity when an eval-created object crosses into another eval frame.
//!
//! Called from:
//! - `tests/codegen/mod.rs` through the codegen integration suite.

use crate::support::*;

/// ReflectionClass::newInstance from evaluated code may instantiate a class compiled into the
/// surrounding binary; its native object factory and constructor bridge must agree on that class.
#[test]
fn test_eval_reflection_class_new_instance_constructs_aot_class() {
    let out = compile_and_run_capture(
        r#"<?php
class AotReflectionConstructionTarget {
    public string $value = 'ready';
    public function __construct() {}
}
echo eval('$r = new ReflectionClass("AotReflectionConstructionTarget"); '
    . '$instance = $r->newInstance(); return get_class($instance);');
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "AotReflectionConstructionTarget");
}

/// ReflectionClass::newInstance must also construct a class declared by an autoloaded file in a
/// previous eval frame, matching PHP's shared runtime class table across includes/eval scopes.
#[test]
fn test_eval_reflection_class_new_instance_constructs_autoloaded_class_across_frames() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
eval('require __DIR__ . "/ReflectionAutoloadedTarget.php";');
echo eval('$reflection = new ReflectionClass("ReflectionAutoloadedTarget"); '
    . '$instance = $reflection->newInstance(); return get_class($instance);');
"#,
            ),
            (
                "ReflectionAutoloadedTarget.php",
                "<?php\nclass ReflectionAutoloadedTarget { public string $value = 'ready'; }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "ReflectionAutoloadedTarget");
}

/// A compiled caller may construct a dynamically autoloaded class through ReflectionClass; the
/// Reflection AOT bridge must resolve that eval declaration in the declaring context.
#[test]
fn test_aot_reflection_new_instance_constructs_runtime_autoloaded_class() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
spl_autoload_register(static function (string $class): void {
    if ($class === 'ReflectionAotAutoloadedTarget') {
        require __DIR__ . '/ReflectionAotAutoloadedTarget.php';
    }
});
function instantiate_reflected(string $class): object {
    return (new ReflectionClass($class))->newInstance();
}
echo get_class(instantiate_reflected('ReflectionAotAutoloadedTarget'));
"#,
            ),
            (
                "ReflectionAotAutoloadedTarget.php",
                "<?php\ninterface ReflectionAotAutoloadedContract { public function configuration(): string; }\nclass ReflectionAotAutoloadedTarget implements ReflectionAotAutoloadedContract { public string $value = 'ready'; public function configuration(): string { return $this->value; } }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "ReflectionAotAutoloadedTarget");
}

/// ReflectionClass::newInstance from an AOT method must see a class declared by an earlier eval
/// include, not only when the Reflection call itself runs inside the declaring eval frame.
#[test]
fn test_aot_reflection_new_instance_constructs_class_declared_in_prior_eval_frame() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
eval('require __DIR__ . "/PriorEvalReflectionTarget.php";');
function instantiate_prior_eval_class(string $class): object {
    return (new ReflectionClass($class))->newInstance();
}
echo get_class(instantiate_prior_eval_class('PriorEvalReflectionTarget'));
"#,
            ),
            (
                "PriorEvalReflectionTarget.php",
                "<?php\nclass PriorEvalReflectionTarget { public string $value = 'ready'; }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "PriorEvalReflectionTarget");
}

/// Namespaced, runtime-autoloaded declarations are valid targets for ReflectionClass::newInstance
/// when the AOT caller and the autoload callback execute in different evaluation scopes.
#[test]
fn test_aot_reflection_new_instance_constructs_namespaced_runtime_class() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
spl_autoload_register(static function (string $class): void {
    if ($class === 'App\\Config\\ReflectionTarget') {
        require __DIR__ . '/ReflectionTarget.php';
    }
});
function instantiate_namespaced_reflection_target(string $class): object {
    return (new \ReflectionClass($class))->newInstance();
}
echo get_class(instantiate_namespaced_reflection_target('App\\Config\\ReflectionTarget'));
"#,
            ),
            (
                "ReflectionTarget.php",
                "<?php\nnamespace App\\Config; class ReflectionTarget { public function __construct() {} }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "App\\Config\\ReflectionTarget");
}

/// A namespaced eval-included class implementing an interface remains constructible by an AOT
/// caller through ReflectionClass, covering the cross-context shape used by extension metadata.
#[test]
fn test_aot_reflection_new_instance_constructs_namespaced_eval_interface_class() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
eval('require __DIR__ . "/NamespacedReflectionTarget.php";');
function instantiate_namespaced_eval_class(string $class): object {
    return (new ReflectionClass($class))->newInstance();
}
echo get_class(instantiate_namespaced_eval_class('App\\Config\\NamespacedReflectionTarget'));
"#,
            ),
            (
                "NamespacedReflectionTarget.php",
                "<?php\nnamespace App\\Config; interface ReflectionConfigContract { public function configuration(): string; } class NamespacedReflectionTarget implements ReflectionConfigContract { public function configuration(): string { return 'ok'; } }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "App\\Config\\NamespacedReflectionTarget");
}

/// A class known only to the process-wide eval declaration table can be instantiated by an AOT
/// ReflectionClass caller in a later scope, as PHP's global class table permits.
#[test]
fn test_aot_reflection_new_instance_constructs_eval_only_class() {
    let out = compile_and_run_capture(
        r#"<?php
$prefix = 'RuntimeOnly';
$definition = 'class ' . $prefix . 'ReflectionTarget {}';
eval($definition);
function instantiate_eval_only_class(string $class): object {
    return (new ReflectionClass($class))->newInstance();
}
echo get_class(instantiate_eval_only_class($prefix . 'ReflectionTarget'));
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "RuntimeOnlyReflectionTarget");
}

/// A ReflectionClass cached in an object property and fetched in a later method keeps its target
/// class identity through the array/property ownership boundary.
#[test]
fn test_aot_reflection_new_instance_constructs_cached_reflection_object() {
    let out = compile_and_run_capture(
        r#"<?php
class ReflectionCacheProbe {
    private array $classes = [];
    public function getReflectionClass(string $name): ReflectionClass {
        if (!isset($this->classes[$name])) {
            $this->classes[$name] = new ReflectionClass($name);
        }
        return $this->classes[$name];
    }
    public function instantiate(string $name): object {
        $reflection = $this->getReflectionClass($name);
        return $reflection->newInstance();
    }
}
class ReflectionCacheTarget { public string $value = 'ready'; }
$cache = new ReflectionCacheProbe();
echo get_class($cache->instantiate(ReflectionCacheTarget::class));
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "ReflectionCacheTarget");
}

/// ReflectionClass cached by an AOT method must preserve the identity of a class loaded later by
/// an autoloader, the same interaction used by native extension configuration discovery.
#[test]
fn test_aot_reflection_new_instance_constructs_cached_autoloaded_class() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
spl_autoload_register(static function (string $class): void {
    if ($class === 'CachedAutoloadedReflectionTarget') {
        require __DIR__ . '/CachedAutoloadedReflectionTarget.php';
    }
});
class ReflectionCacheProbe {
    private array $classes = [];
    public function getReflectionClass(string $name): ReflectionClass {
        if (!isset($this->classes[$name])) {
            $this->classes[$name] = new ReflectionClass($name);
        }
        return $this->classes[$name];
    }
    public function instantiate(string $name): object {
        return $this->getReflectionClass($name)->newInstance();
    }
}
$cache = new ReflectionCacheProbe();
echo get_class($cache->instantiate('CachedAutoloadedReflectionTarget'));
"#,
            ),
            (
                "CachedAutoloadedReflectionTarget.php",
                "<?php\nclass CachedAutoloadedReflectionTarget { public string $value = 'ready'; }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "CachedAutoloadedReflectionTarget");
}

/// A ReflectionClass object itself may cross eval frames; newInstance must still resolve the
/// class identity held by that object when a later frame invokes it.
#[test]
fn test_eval_reflection_class_object_new_instance_crosses_frames() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
$reflection = eval('require __DIR__ . "/CrossFrameReflectionTarget.php"; return new ReflectionClass("CrossFrameReflectionTarget");');
echo eval('$instance = $reflection->newInstance(); return get_class($instance);');
"#,
            ),
            (
                "CrossFrameReflectionTarget.php",
                "<?php\nclass CrossFrameReflectionTarget { public string $value = 'ready'; }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "CrossFrameReflectionTarget");
}

/// Reflection instantiates a runtime-loaded, namespaced class with an imported interface and no
/// explicit constructor, matching PHP's default constructor behavior across eval declarations.
#[test]
fn test_eval_reflection_instantiates_runtime_class_with_aliased_interface_and_no_constructor() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
echo eval('spl_autoload_register(static function (string $class): void { if ($class === "App\\\\Contracts\\\\RuntimeContract") { require __DIR__ . "/RuntimeContract.php"; } elseif ($class === "App\\\\Bundle\\\\RuntimeConfiguration") { require __DIR__ . "/RuntimeConfiguration.php"; } }); class RuntimeContainer { public function getReflectionClass(string $className): ?ReflectionClass { return new ReflectionClass($className); } } class RuntimeExtension { public function getConfiguration(string $className, RuntimeContainer $container): object { $reflection = $container->getReflectionClass($className); if (!$reflection->implementsInterface("App\\\\Contracts\\\\RuntimeContract")) { return null; } if (!($constructor = $reflection->getConstructor()) || !$constructor->getNumberOfRequiredParameters()) { return $reflection->newInstance(); } return null; } } class ConcreteRuntimeExtension extends RuntimeExtension {} return get_class((new ConcreteRuntimeExtension())->getConfiguration("App\\\\Bundle\\\\RuntimeConfiguration", new RuntimeContainer()));');
"#,
            ),
            (
                "RuntimeContract.php",
                "<?php\nnamespace App\\Contracts; interface RuntimeContract { public function configure(): string; }\n",
            ),
            (
                "RuntimeConfiguration.php",
                "<?php\nnamespace App\\Bundle; use App\\Contracts\\RuntimeContract as Contract; class RuntimeConfiguration implements Contract { public function configure(): string { return 'ready'; } }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "App\\Bundle\\RuntimeConfiguration");
}

/// An AOT method can receive a ReflectionClass for an eval/autoload class; the nullable receiver
/// guard must not turn the empty variadic ReflectionClass::newInstance() call into one array arg.
#[test]
fn test_aot_reflection_new_instance_on_nullable_eval_reflection_class_passes_no_constructor_args() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
eval('require __DIR__ . "/RuntimeContract.php"; require __DIR__ . "/RuntimeConfiguration.php";');
class RuntimeContainer {
    public function getReflectionClass(string $className): ?ReflectionClass {
        return new ReflectionClass($className);
    }
}
class RuntimeExtension {
    public function getConfiguration(string $className, RuntimeContainer $container): ?\App\Contracts\RuntimeContract {
        $reflection = $container->getReflectionClass($className);
        if (!$reflection) {
            return null;
        }
        if (!$reflection->implementsInterface('App\\Contracts\\RuntimeContract')) {
            return null;
        }
        if (!($constructor = $reflection->getConstructor()) || !$constructor->getNumberOfRequiredParameters()) {
            return $reflection->newInstance();
        }
        return null;
    }
}
$configuration = (new RuntimeExtension())->getConfiguration('App\\Bundle\\RuntimeConfiguration', new RuntimeContainer());
if (!$configuration) {
    echo 'missing';
} else {
    echo $configuration->configure();
}
"#,
            ),
            (
                "RuntimeContract.php",
                "<?php\nnamespace App\\Contracts; interface RuntimeContract { public function configure(): string; }\n",
            ),
            (
                "RuntimeConfiguration.php",
                "<?php\nnamespace App\\Bundle; use App\\Contracts\\RuntimeContract as Contract; class RuntimeConfiguration implements Contract { public function configure(): string { return 'ready'; } }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "ready");
}

/// A runtime-included object's class remains available to typed argument binding after its
/// creating eval frame returns.
#[test]
fn test_eval_returned_object_keeps_its_class_for_a_later_typed_call() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
$dependency = eval('require __DIR__ . "/RuntimeDependency.php"; return new RuntimeDependency();');
echo eval('require __DIR__ . "/RuntimeConsumer.php"; new RuntimeConsumer($dependency); return "ok";');
"#,
            ),
            (
                "RuntimeDependency.php",
                "<?php\nclass RuntimeDependency {}\n",
            ),
            (
                "RuntimeConsumer.php",
                "<?php\nclass RuntimeConsumer { public function __construct(RuntimeDependency $dependency) {} }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "ok");
}

/// Objects whose classes are declared by SPL autoload in one eval frame remain assignable to a
/// typed constructor in a later frame.
#[test]
fn test_eval_autoloaded_object_keeps_its_class_for_a_later_typed_call() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
spl_autoload_register(static function (string $class): void {
    if ($class === 'AutoloadedDependency') {
        require __DIR__ . '/AutoloadedDependency.php';
    } elseif ($class === 'AutoloadedConsumer') {
        require __DIR__ . '/AutoloadedConsumer.php';
    }
});
$dependency = eval('return new AutoloadedDependency();');
echo eval('new AutoloadedConsumer($dependency); return "ok";');
"#,
            ),
            (
                "AutoloadedDependency.php",
                "<?php\nclass AutoloadedDependency {}\n",
            ),
            (
                "AutoloadedConsumer.php",
                "<?php\nclass AutoloadedConsumer { public function __construct(AutoloadedDependency $dependency) {} }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "ok");
}

/// Objects returned by Reflection and round-tripped through an array retain their PHP class
/// identity when a later eval call binds a typed constructor argument.
#[test]
fn test_eval_lazy_ghost_keeps_its_class_through_array_for_typed_call() {
    let out = compile_and_run_capture(
        r#"<?php
class LazyGhostDependency { public string $value = 'new'; }
class LazyGhostConsumer { public function __construct(LazyGhostDependency $dependency) {} }
$reflection = new ReflectionClass(LazyGhostDependency::class);
$services = [];
$services['dependency'] = $reflection->newLazyGhost(static function ($object) {
    $object->value = 'initialized';
});
echo eval('new LazyGhostConsumer($services["dependency"]); return "ok";');
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "ok");
}

/// A Reflection-created object written into an array-valued object property keeps its class when
/// read back by a later eval call that binds a typed constructor argument.
#[test]
fn test_eval_lazy_ghost_keeps_its_class_through_object_property_array() {
    let out = compile_and_run_capture(
        r#"<?php
class PropertyArrayGhostDependency { public string $value = 'new'; }
class PropertyArrayGhostConsumer { public function __construct(PropertyArrayGhostDependency $dependency) {} }
class PropertyArrayGhostContainer { public array $privates = []; }
$container = new PropertyArrayGhostContainer();
$reflection = new ReflectionClass(PropertyArrayGhostDependency::class);
$container->privates['dependency'] = $reflection->newLazyGhost(static function ($object) {
    $object->value = 'initialized';
});
echo eval('new PropertyArrayGhostConsumer($container->privates["dependency"]); return "ok";');
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "ok");
}

/// A captured service container used by a lazy-ghost initializer does not erase the ghost's class.
#[test]
fn test_eval_lazy_ghost_keeps_its_class_through_capturing_initializer() {
    let out = compile_and_run_capture(
        r#"<?php
class CapturedGhostDependency { public string $value = 'new'; }
class CapturedGhostConsumer { public function __construct(CapturedGhostDependency $dependency) {} }
class CapturedGhostContainer { public array $privates = []; }
class CapturedGhostFactory {
    public static function get(CapturedGhostContainer $container, $lazyLoad = true) {
        if ($lazyLoad === true) {
            return $container->privates['dependency'] = (new ReflectionClass(CapturedGhostDependency::class))
                ->newLazyGhost(static function ($proxy) use ($container) {
                    self::get($container, $proxy);
                });
        }
        $lazyLoad->value = 'initialized';
        return $lazyLoad;
    }
}
$container = new CapturedGhostContainer();
CapturedGhostFactory::get($container);
echo eval('new CapturedGhostConsumer($container->privates["dependency"]); return "ok";');
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "ok");
}

/// ReflectionClass::newLazyGhost called through Magician keeps the AOT class identity.
#[test]
fn test_eval_dynamic_new_lazy_ghost_keeps_its_class_for_typed_call() {
    let out = compile_and_run_capture(
        r#"<?php
class DynamicGhostDependency { public string $value = 'new'; }
class DynamicGhostConsumer { public function __construct(DynamicGhostDependency $dependency) {} }
$services = [];
$setup = '$services["dependency"] = (new ReflectionClass("DynamicGhostDependency"))'
    . '->newLazyGhost(static function ($object) { $object->value = "initialized"; });';
eval($setup);
echo eval('new DynamicGhostConsumer($services["dependency"]); return "ok";');
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "ok");
}

/// Autoloaded AOT classes retain their runtime class id after constructorless allocation,
/// initialization, and a round-trip through a compiled service array into a later eval call.
#[test]
fn test_eval_autoloaded_lazy_ghost_keeps_its_class_for_typed_call() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
spl_autoload_register(static function (string $class): void {
    if ($class === 'App\\Ghost\\AutoloadedLazyGhostDependency') {
        require __DIR__ . '/AutoloadedLazyGhostDependency.php';
    } elseif ($class === 'App\\Ghost\\AutoloadedLazyGhostConsumer') {
        require __DIR__ . '/AutoloadedLazyGhostConsumer.php';
    }
});
class AutoloadedLazyGhostContainer { public array $privates = []; }
$container = new AutoloadedLazyGhostContainer();
$setup = '$container->privates["dependency"] = (new ReflectionClass("App\\Ghost\\AutoloadedLazyGhostDependency"))'
    . '->newLazyGhost(static function ($proxy) use ($container) { $proxy->__construct("initialized"); });';
eval($setup);
echo eval('new \\App\\Ghost\\AutoloadedLazyGhostConsumer($container->privates["dependency"]); return "ok";');
"#,
            ),
            (
                "AutoloadedLazyGhostDependency.php",
                "<?php\nnamespace App\\Ghost;\nclass AutoloadedLazyGhostDependency { public string $value = 'new'; public function __construct(string $value) { $this->value = $value; } }\n",
            ),
            (
                "AutoloadedLazyGhostConsumer.php",
                "<?php\nnamespace App\\Ghost;\nclass AutoloadedLazyGhostConsumer { public function __construct(AutoloadedLazyGhostDependency $dependency) {} }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "ok");
}

/// Discarding a dynamic method's return of its borrowed object parameter must not free the
/// constructorless object retained by the caller.
#[test]
fn test_eval_discarded_dynamic_object_return_preserves_caller_object() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
require __DIR__ . '/DynamicReturnDependency.php';
require __DIR__ . '/DynamicReturnConsumer.php';
class DynamicReturnContainer { protected array $privates = []; }
eval('class DynamicReturnFactory extends DynamicReturnContainer {
    public static function get($container, $lazyLoad = true) {
        if ($lazyLoad === true) {
            return $container->privates["dependency"] = (new ReflectionClass("DynamicReturnDependency"))
                ->newLazyGhost(static function ($proxy) use ($container) {
                    self::get($container, $proxy);
                });
        }
        return $lazyLoad;
    }
    public static function makeConsumer($container) {
        return new DynamicReturnConsumer($container->privates["dependency"]);
    }
}');
eval('class DynamicReturnAppContainer extends DynamicReturnContainer {
    public function load($id) { return DynamicReturnFactory::get($this); }
}');
$container = eval('return new DynamicReturnAppContainer();');
eval('$container->load("dependency");');
echo eval('DynamicReturnFactory::makeConsumer($container); return "ok";');
"#,
            ),
            (
                "DynamicReturnDependency.php",
                "<?php\nclass DynamicReturnDependency { public function __construct() {} }\n",
            ),
            (
                "DynamicReturnConsumer.php",
                "<?php\nclass DynamicReturnConsumer { public function __construct(DynamicReturnDependency $dependency) {} }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "ok");
}

/// A lazily initialized object declared only by runtime autoload remains type-identifiable after
/// storage and is accepted by a constructor in a later eval context.
#[test]
fn test_eval_runtime_autoloaded_ghost_keeps_class_identity_across_contexts() {
    let out = compile_cli_files_and_run(
        &[(
            "main.php",
            r#"<?php
$prefix = __DIR__;
spl_autoload_register(static function (string $class) use ($prefix): void {
    if ($class === 'RuntimeGhostDependency') {
        require $prefix . '/RuntimeGhostDependency.php';
    } elseif ($class === 'RuntimeGhostConsumer') {
        require $prefix . '/RuntimeGhostConsumer.php';
    }
});
class RuntimeGhostContainer { public array $privates = []; }
$container = new RuntimeGhostContainer();
$setup = '$container->privates["dependency"] = (new ReflectionClass("RuntimeGhostDependency"))'
    . '->newLazyGhost(static function ($proxy) { $proxy->__construct("secret"); });';
eval($setup);
echo eval('new RuntimeGhostConsumer($container->privates["dependency"]); return "ok";');
"#,
        ),
        (
            "RuntimeGhostDependency.php",
            "<?php\nclass RuntimeGhostDependency { public string $value; public function __construct(string $value) { $this->value = $value; } }\n",
        ),
        (
            "RuntimeGhostConsumer.php",
            "<?php\nclass RuntimeGhostConsumer { public function __construct(RuntimeGhostDependency $dependency) {} }\n",
        ),
        ],
        "main.php",
    );
    assert_eq!(out, "ok");
}

/// ReflectionClass::newLazyGhost retains an interpreter-owned instance across callback argument
/// cleanup and a later typed constructor call.
#[test]
fn test_eval_dynamic_class_lazy_ghost_survives_initializer_argument_cleanup() {
    let out = compile_cli_files_and_run(
        &[
            (
                "main.php",
                r#"<?php
$declarations = file_get_contents(__DIR__ . '/runtime-classes.txt');
eval($declarations);
$container = eval('return new InterpreterGhostContainer();');
eval('InterpreterGhostFactory::get($container);');
echo eval('InterpreterGhostFactory::makeConsumer($container); return "ok";');
"#,
            ),
            (
                "runtime-classes.txt",
                "class InterpreterGhostDependency { private string $secret; public function __construct(string $secret) { $this->secret = $secret; } }\nclass InterpreterGhostContainer { public array $privates = []; }\nclass InterpreterGhostFactory { public static function get($container, $lazyLoad = true) { if ($lazyLoad === true) { return $container->privates['dependency'] = (new ReflectionClass('InterpreterGhostDependency'))->newLazyGhost(static function ($proxy) use ($container) { self::get($container, $proxy); }); } return ($lazyLoad->__construct('secret') && false ?: $lazyLoad); } public static function makeConsumer($container) { return new InterpreterGhostConsumer($container->privates['dependency']); } }\nclass InterpreterGhostConsumer { public function __construct(InterpreterGhostDependency $dependency) {} }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "ok");
}
