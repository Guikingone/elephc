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

/// An eval-declared class extending a native exception class (`\LogicException`) must declare
/// successfully and behave like PHP: the constructor accepts message/code, the subclass is
/// catchable by its native parent's type, and the inherited native accessors work.
///
/// Nothing in the AOT-compiled program calls `LogicException::getMessage()`/`getCode()`
/// directly -- only the eval fragment does -- so their EIR bodies are pruned as unreachable by
/// the same dead-code pass `test_eval_can_invoke_a_native_method_without_a_static_call_site`
/// above exercises for a user class. Reachability metadata used to double as the eval
/// reflection bridge's ABSTRACT bit for AOT class methods, so a native ancestor method missing
/// only its emitted symbol was reported abstract, and every native-based eval subclass failed
/// `class X could not be declared` at declaration time.
///
/// `php -n` 8.5.6 (`scratchpad/verify_native_ext.php`, this session): `caught:boom:42`.
#[test]
fn test_eval_class_can_extend_a_native_exception_and_be_caught_by_its_parent_type() {
    let out = compile_and_run(
        r#"<?php
$source = <<<'PHP'
class EvalLogicSub extends \LogicException {}

try {
    throw new EvalLogicSub("boom", 42);
} catch (\LogicException $e) {
    echo "caught:" . $e->getMessage() . ":" . $e->getCode();
}
PHP;
echo eval($source);
"#,
    );

    assert_eq!(out, "caught:boom:42");
}

/// An eval-declared class extending a native exception class and defining its OWN constructor
/// (constructor-promoted property included) must be able to call `parent::__construct(...)` on
/// the native ancestor. This is the exact shape of Symfony's
/// `Dotenv\Exception\FormatException extends \LogicException`, which gated the whole `--web`
/// campaign: `final class FormatException extends \LogicException implements ExceptionInterface`
/// with a promoted `FormatExceptionContext $context` parameter and a `parent::__construct(...)`
/// call built from `sprintf(...)`.
///
/// `parent::__construct()` static-call syntax has no entry in the eval reflection method index
/// when the resolved parent is native -- `collect_eval_native_instance_methods` (codegen) skips
/// `__construct` on purpose, since allocation-time `new X()` dispatch uses a separate constructor
/// registration table keyed for that purpose. Every generic static-dispatch branch therefore fell
/// through to "undefined method" for a native `parent::__construct()` call specifically.
///
/// `php -n` 8.5.6 (`scratchpad/verify_parent_construct.php`, this session): `bad in
/// "/tmp/x":0:is`.
#[test]
fn test_eval_class_extending_a_native_exception_can_call_native_parent_construct() {
    let out = compile_and_run(
        r#"<?php
$source = <<<'PHP'
class EvalCtx {
    public function p(): string { return '/tmp/x'; }
}

final class EvalFormatLike extends \LogicException {
    public function __construct(string $message, private EvalCtx $context, int $code = 0, ?\Throwable $previous = null) {
        parent::__construct(sprintf('%s in "%s"', $message, $context->p()), $code, $previous);
    }
}

$e = new EvalFormatLike('bad', new EvalCtx());
echo $e->getMessage(), ':', $e->getCode(), ':', ($e instanceof \LogicException ? 'is' : 'not'), "\n";
PHP;
echo eval($source);
"#,
    );

    assert_eq!(out, "bad in \"/tmp/x\":0:is\n");
}

/// The exact shape of Symfony's `Dotenv\Exception\FormatException`: an eval-declared class
/// extends a native exception AND implements an interface that itself extends `\Throwable`
/// (`Symfony\Component\Dotenv\Exception\ExceptionInterface extends \Throwable`).
///
/// PHP only rejects `implements Throwable` ("Class X cannot implement interface Throwable,
/// extend Exception or Error instead", `php -n` 8.5.6) when the class does not already inherit
/// Throwable from its parent. `validate_eval_class_does_not_implement_throwable_interfaces`
/// rejected this UNCONDITIONALLY whenever any implemented interface transitively reached
/// `Throwable`, without checking whether the class's own parent already supplied it -- so every
/// Symfony-shaped exception subclass (parent native, own interface re-stating Throwable) failed
/// to declare, one stage past the abstract-parent-requirements gate this file's other tests fix.
///
/// `php -n` 8.5.6 (`scratchpad/verify_full_symfony_shape.php`, this session):
/// `bad in "/tmp/x":/tmp/x:is:is`.
#[test]
fn test_eval_class_extending_native_and_implementing_a_throwable_interface_declares() {
    let out = compile_and_run(
        r#"<?php
$source = <<<'PHP'
interface MyDotenvExceptionInterface extends \Throwable {}

final class MyFormatException extends \LogicException implements MyDotenvExceptionInterface
{
    public function __construct(string $message, private string $context, int $code = 0, ?\Throwable $previous = null)
    {
        parent::__construct(sprintf('%s in "%s"', $message, $context), $code, $previous);
    }

    public function getContext(): string
    {
        return $this->context;
    }
}

$e = new MyFormatException('bad', '/tmp/x');
echo $e->getMessage(), ':', $e->getContext(), ':', ($e instanceof MyDotenvExceptionInterface ? 'is' : 'not'), ':', ($e instanceof \Throwable ? 'is' : 'not'), "\n";
PHP;
echo eval($source);
"#,
    );

    assert_eq!(out, "bad in \"/tmp/x\":/tmp/x:is:is\n");
}
