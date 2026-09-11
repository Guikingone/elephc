//! Purpose:
//! Regression coverage for native methods first reached from eval-owned code.
//!
//! Called from:
//! - `tests/codegen/mod.rs` through the codegen integration suite.

use crate::support::*;

/// A public method called solely by an eval fragment remains executable in the
/// compiled class instead of being metadata-only.
#[test]
fn test_eval_can_invoke_a_native_method_without_a_static_call_site() {
    let out = compile_and_run(
        r#"<?php
class EvalNativeReachability {
    public function dispatch(object $event): object {
        $event->answer = "ok";
        return $event;
    }
}

$dispatcher = new EvalNativeReachability();
echo eval('$event = $dispatcher->dispatch(new stdClass()); return $event->answer;');
"#,
    );

    assert_eq!(out, "ok");
}

/// A closure declared by eval retains its declaring class scope after another eval class invokes
/// it. Without that scope, the protected read is checked against the invoker rather than the
/// factory class that declared the closure.
#[test]
fn test_eval_closure_keeps_declaring_scope_for_protected_property_access() {
    let out = compile_and_run(
        r#"<?php
$source = <<<'PHP'
class ProtectedScopeBase {
    protected string $value = 'ok';
}

class ProtectedScopeFactory extends ProtectedScopeBase {
    public function callback() {
        $owner = $this;

        return static function () use ($owner): string {
            return $owner->value;
        };
    }
}

class ProtectedScopeInvoker {
    public function invoke($callback): string {
        return $callback();
    }
}

$factory = new ProtectedScopeFactory();
$callback = $factory->callback();

return (new ProtectedScopeInvoker())->invoke($callback);
PHP;

echo eval($source);
"#,
    );

    assert_eq!(out, "ok");
}

/// A closure retains its declaration file after a runtime include returns. `__DIR__` must name
/// the included file's directory rather than the caller that invokes the closure later.
#[test]
fn test_eval_closure_keeps_declaring_file_for_magic_dir() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
$path = $argc > 99 ? 'missing.php' : __DIR__.'/nested/factory.php';
include $path;

$callback = (new ClosureSourceFactory())->callback();
echo $callback();
"#,
            ),
            (
                "nested/factory.php",
                r#"<?php
class ClosureSourceFactory {
    public function callback() {
        return static fn (): string => basename(__DIR__);
    }
}
"#,
            ),
        ],
        "main.php",
    );

    assert_eq!(out, "nested");
}

/// Nested by-reference foreach loops keep their element targets while a closure is appended by
/// reference to another property-backed array. This is the minimal dynamic form of a listener
/// registry that compacts callbacks after priority sorting.
#[test]
fn test_eval_nested_foreach_references_survive_property_append_closure() {
    let out = compile_and_run(
        r#"<?php
$source = <<<'PHP'
class DynamicListenerRegistry {
    private array $listeners = [];
    private array $optimized = [];

    public function add(string $event, int $priority, mixed $listener): void {
        $this->listeners[$event][$priority][] = $listener;
    }

    public function optimize(string $event): int {
        krsort($this->listeners[$event]);
        $this->optimized[$event] = [];

        foreach ($this->listeners[$event] as &$listeners) {
            foreach ($listeners as &$listener) {
                $closure = &$this->optimized[$event][];
                $closure = static function () use (&$listener, &$closure): void {};
            }
        }

        return count($this->optimized[$event]);
    }
}

$registry = new DynamicListenerRegistry();
$registry->add('event', 0, 1);
$registry->add('event', 100, 2);

return $registry->optimize('event');
PHP;

echo eval($source);
"#,
    );

    assert_eq!(out, "2");
}
