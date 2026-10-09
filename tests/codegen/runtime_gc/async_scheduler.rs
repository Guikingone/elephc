//! Purpose:
//! Heap-debug acceptance tests for Async scheduler task and reactor ownership.
//!
//! Called from:
//! - `tests/codegen/runtime_gc.rs`.
//!
//! Key details:
//! - Success, cancellation, failure, and descriptor readiness must release every
//!   task, Fiber, outcome, registration, captured value, and scheduler array.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(out: crate::support::ProgramOutput, expected: &str) {
    assert_eq!(out.stdout, expected, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

#[test]
fn async_success_releases_task_results_and_captures() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

$value = run(function (TaskGroup $tasks): string {
    $payload = ["answer" => 42];
    $child = $tasks->spawn(function () use ($payload): string {
        return "value:" . $payload["answer"];
    });
    return $child->await();
});
echo $value;
unset($value);
"#,
    );
    assert_clean(out, "value:42");
}

#[test]
fn async_empty_scope_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

$i = 0;
while ($i < 2) {
    $result = run(function (TaskGroup $tasks): void {});
    unset($result);
    $i = $i + 1;
}
\__elephc_async_gc_collect();
echo "empty";
"#,
    );
    assert_clean(out, "empty");
}

#[test]
fn async_prelude_without_runtime_entry_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
echo "unused";
"#,
    );
    assert_clean(out, "unused");
}

#[test]
fn async_internal_objects_without_a_fiber_are_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__Scheduler;
use Elephc\Async\__ScopeState;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$state = new __CancellationState();
$scheduler = new __Scheduler($state);
$scope = new __ScopeState($scheduler);
$group = new TaskGroup($scope, $state);
$scope->detach();
unset($group);
unset($scope);
unset($scheduler);
unset($state);
\__elephc_async_gc_collect();
echo "objects";
"#,
    );
    assert_clean(out, "objects");
}

#[test]
fn fiber_wrapper_shape_used_by_async_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class FiberArgumentHolder {}

function driveFiber(callable $body, array $args): void {
    $fiber = new Fiber(function () use ($body, $args): mixed {
        return $body(...$args);
    });
    $signal = $fiber->start();
    unset($signal);
    unset($fiber);
    unset($body);
    unset($args);
}

$holder = new FiberArgumentHolder();
driveFiber(function (FiberArgumentHolder $value): void {}, [$holder]);
unset($holder);
echo "fiber";
"#,
    );
    assert_clean(out, "fiber");
}

#[test]
fn fiber_start_argument_cycle_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class FiberOwner {
    public array $fibers = [];
}
final class FiberArgument {
    public function __construct(public FiberOwner $owner) {}
}

$owner = new FiberOwner();
$argument = new FiberArgument($owner);
$fiber = new Fiber(function (FiberArgument $value): void {});
$owner->fibers[] = $fiber;
$signal = $fiber->start($argument);
unset($signal);
$owner->fibers[0] = null;
unset($fiber);
unset($argument);
unset($owner);
echo "cycle";
"#,
    );
    assert_clean(out, "cycle");
}

#[test]
fn rejected_second_fiber_start_releases_mixed_arguments() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class FiberStartPayload {}

function start_terminated_fiber(Fiber $fiber, mixed $payload): void {
    try {
        $fiber->start($payload);
    } catch (FiberError $error) {
    }
}

$fiber = new Fiber(static function (mixed $payload): void {});
$fiber->start(new FiberStartPayload());
$payload = new FiberStartPayload();
start_terminated_fiber($fiber, $payload);
unset($payload);
unset($fiber);
echo "fiber";
"#,
    );
    assert_clean(out, "fiber");
}

#[test]
fn rejected_second_fiber_start_releases_mixed_call_result() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class FiberStartCallPayload {}
function make_mixed_fiber_start_payload(): mixed {
    return new FiberStartCallPayload();
}
function make_typed_fiber_start_payload(): FiberStartCallPayload {
    return new FiberStartCallPayload();
}
$fiber = new Fiber(static function (mixed $payload): void {});
$fiber->start(new FiberStartCallPayload());
try {
    $fiber->start(make_mixed_fiber_start_payload());
} catch (FiberError $error) {
    echo "start-error";
}
try {
    $fiber->start(make_typed_fiber_start_payload());
} catch (FiberError $error) {
    echo "-start-error";
}
unset($fiber);
"#,
    );
    assert_clean(out, "start-error-start-error");
}

#[test]
fn temporary_fiber_resume_state_error_releases_receiver() {
    let source = r#"<?php
try {
    (new Fiber(static function (): void {}))->resume();
} catch (FiberError $error) {
    echo "resume-error";
}
"#;
    let out = compile_and_run_with_heap_debug(source);
    assert_clean(out, "resume-error");
}

#[test]
fn rejected_fiber_resume_releases_boxed_argument() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class ResumePayload {}
$fiber = new Fiber(static function (): void {});
$payload = new ResumePayload();
try {
    $fiber->resume($payload);
} catch (FiberError $error) {
    echo "resume-error";
}
unset($payload);
unset($fiber);
"#,
    );
    assert_clean(out, "resume-error");
}

#[test]
fn rejected_fiber_resume_releases_retained_mixed_argument() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function mixed_resume_payload(): mixed {
    return "payload";
}
$fiber = new Fiber(static function (): void {});
$payload = mixed_resume_payload();
try {
    $fiber->resume($payload);
} catch (FiberError $error) {
    echo "resume-error";
}
unset($payload);
unset($fiber);
"#,
    );
    assert_clean(out, "resume-error");
}

#[test]
fn rejected_fiber_resume_releases_mixed_call_result() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class ResumeCallPayload {}
function make_mixed_resume_payload(): mixed {
    return new ResumeCallPayload();
}
$fiber = new Fiber(static function (): void {});
try {
    $fiber->resume(make_mixed_resume_payload());
} catch (FiberError $error) {
    echo "resume-error";
}
unset($fiber);
"#,
    );
    assert_clean(out, "resume-error");
}

#[test]
fn rejected_fiber_resume_releases_typed_call_result() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class ResumeObjectPayload {}
$fiber = new Fiber(static function (): void {});
try {
    $fiber->resume(new ResumeObjectPayload());
} catch (FiberError $error) {
    echo "resume-error";
}
unset($fiber);
"#,
    );
    assert_clean(out, "resume-error");
}

#[test]
fn rejected_temporary_fiber_throw_releases_receiver_and_throwable() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
try {
    (new Fiber(static function (): void {}))->throw(new RuntimeException("rejected"));
} catch (FiberError $error) {
    echo "throw-error";
}
"#,
    );
    assert_clean(out, "throw-error");
}

#[test]
fn escaped_temporary_fiber_throw_releases_receiver_and_throwable() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function started_suspended_fiber(): Fiber {
    $fiber = new Fiber(static function (): void {
        Fiber::suspend();
    });
    $fiber->start();
    return $fiber;
}
try {
    started_suspended_fiber()->throw(new RuntimeException("escape"));
} catch (RuntimeException $error) {
    echo "escaped";
}
"#,
    );
    assert_clean(out, "escaped");
}

#[test]
fn escaped_borrowed_fiber_throw_releases_injected_exception() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function started_suspended_fiber(): Fiber {
    $fiber = new Fiber(static function (): void {
        Fiber::suspend();
    });
    $fiber->start();
    return $fiber;
}
$fiber = started_suspended_fiber();
$thrown = new RuntimeException("escape");
try {
    $fiber->throw($thrown);
} catch (RuntimeException $error) {
    echo "escaped";
}
unset($fiber, $thrown, $error);
echo "done";
"#,
    );
    assert_clean(out, "escapeddone");
}

#[test]
fn escaped_temporary_fiber_resume_releases_receiver() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(static function (): void {
    Fiber::suspend();
    throw new Exception();
});
$fiber->start();
try {
    $fiber->resume();
} catch (Exception $error) {
    echo "escaped";
}
unset($fiber, $error);
echo "done";
"#,
    );
    assert_clean(out, "escapeddone");
}

#[test]
fn suspended_fiber_local_object_is_released_when_the_fiber_is_dropped() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class FiberLocalPayload { public function __destruct() { echo "Y"; } }
$fiber = new Fiber(static function (): void {
    $payload = new FiberLocalPayload();
    Fiber::suspend();
});
$fiber->start();
echo "drop";
unset($fiber);
echo "done";
"#,
    );
    assert_clean(out, "dropYdone");
}

#[test]
fn returned_temporary_fiber_start_state_error_releases_receiver() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function already_started_fiber(): Fiber {
    $fiber = new Fiber(static function (): void {});
    $fiber->start();
    return $fiber;
}
try {
    already_started_fiber()->start();
} catch (FiberError $error) {
    echo "start-error";
}
"#,
    );
    assert_clean(out, "start-error");
}

#[test]
fn mixed_property_array_releases_object_elements() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class MixedOwner {
    public array $items = [];
}
final class MixedPayload {}

$owner = new MixedOwner();
$payload = new MixedPayload();
$owner->items[] = $payload;
unset($owner);
unset($payload);
echo "array";
"#,
    );
    assert_clean(out, "array");
}

#[test]
fn private_typed_array_releases_terminated_fiber() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__ScopeState;
use Elephc\Async\__Scheduler;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
final class FiberOwner {
    private array $items = [];
    public function add(mixed $value): void { $this->items[] = $value; }
    public function clear(): void { $this->items[0] = null; }
}

$state = new __CancellationState();
$scheduler = new __Scheduler($state);
$scope = new __ScopeState($scheduler);
$group = new TaskGroup($scope, $state);
$owner = new FiberOwner();
$fiber = new Fiber(function (TaskGroup $tasks): void {});
$signal = $fiber->start($group);
unset($signal);
$owner->add($fiber);
$owner->clear();
$scope->detach();
unset($fiber);
unset($group);
unset($scope);
unset($scheduler);
unset($state);
unset($owner);
echo "private";
"#,
    );
    assert_clean(out, "private");
}

#[test]
fn private_typed_array_releases_ordinary_object() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class ObjectOwner {
    private array $items = [];
    public function add(mixed $value): void { $this->items[] = $value; }
    public function clear(): void { $this->items = []; }
}
final class ObjectPayload {}

$owner = new ObjectOwner();
$payload = new ObjectPayload();
$owner->add($payload);
$owner->clear();
unset($payload);
unset($owner);
echo "object";
"#,
    );
    assert_clean(out, "object");
}

#[test]
fn bare_fiber_releases_task_group_argument() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__ScopeState;
use Elephc\Async\__Scheduler;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$state = new __CancellationState();
$scheduler = new __Scheduler($state);
$scope = new __ScopeState($scheduler);
$group = new TaskGroup($scope, $state);
$fiber = new Fiber(function (TaskGroup $tasks): void {});
$signal = $fiber->start($group);
unset($signal);
$scope->detach();
unset($fiber);
unset($group);
unset($scope);
unset($scheduler);
unset($state);
echo "bare";
"#,
    );
    assert_clean(out, "bare");
}

#[test]
fn bare_terminated_fiber_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(function (): void {});
$signal = $fiber->start();
unset($signal);
unset($fiber);
echo "fiber";
"#,
    );
    assert_clean(out, "fiber");
}

#[test]
fn bare_terminated_fiber_releases_object_argument() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class FiberPayload {}
$payload = new FiberPayload();
$fiber = new Fiber(function (FiberPayload $value): void {});
$signal = $fiber->start($payload);
unset($signal);
unset($fiber);
unset($payload);
echo "payload";
"#,
    );
    assert_clean(out, "payload");
}

#[test]
fn bare_terminated_fiber_releases_return_value() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(function (): string { return "value"; });
$signal = $fiber->start();
$result = $fiber->getReturn();
unset($signal);
unset($result);
unset($fiber);
echo "return";
"#,
    );
    assert_clean(out, "return");
}

#[test]
fn suspended_then_resumed_fiber_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(function (): string {
    Fiber::suspend();
    return "done";
});
$signal = $fiber->start();
unset($signal);
$signal = $fiber->resume();
unset($signal);
$result = $fiber->getReturn();
unset($result);
unset($fiber);
echo "resume";
"#,
    );
    assert_clean(out, "resume");
}

#[test]
fn suspended_then_resumed_fiber_releases_capture() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class FiberCapture {}
$capture = new FiberCapture();
$fiber = new Fiber(function () use ($capture): string {
    Fiber::suspend();
    return "done";
});
$signal = $fiber->start();
unset($signal);
$signal = $fiber->resume();
unset($signal);
$result = $fiber->getReturn();
unset($result);
unset($fiber);
unset($capture);
echo "capture";
"#,
    );
    assert_clean(out, "capture");
}

#[test]
fn async_unawaited_child_releases_result_at_scope_exit() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $payload = ["answer" => 42];
    $tasks->spawn(function () use ($payload): string {
        return "value:" . $payload["answer"];
    });
    unset($payload);
});
echo "scope";
"#,
    );
    assert_clean(out, "scope");
}

#[test]
fn async_child_capturing_task_group_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $tasks->spawn(function () use ($tasks): string { return "done"; });
});
echo "group-capture";
"#,
    );
    assert_clean(out, "group-capture");
}

#[test]
fn async_sleeping_child_releases_state_after_normal_wake() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $child = $tasks->spawn(function () use ($tasks): string {
        $tasks->sleep(0.001);
        return "awake";
    });
    echo $child->await();
});
"#,
    );
    assert_clean(out, "awake");
}

#[test]
fn async_root_sleep_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $tasks->sleep(0.001);
});
echo "root-sleep";
"#,
    );
    assert_clean(out, "root-sleep");
}

#[test]
fn cancelled_exception_is_heap_clean_after_catch() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
try {
    throw new CancelledException();
} catch (CancelledException $error) {
    echo "caught";
}
"#,
    );
    assert_clean(out, "caught");
}

#[test]
fn fiber_releases_dynamic_captured_callable() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$payload = ["answer" => 42];
$body = function () use ($payload): void {
    $payload["answer"];
};
$fiber = new Fiber($body);
$signal = $fiber->start();
unset($signal);
unset($fiber);
unset($body);
unset($payload);
echo "dynamic";
"#,
    );
    assert_clean(out, "dynamic");
}

#[test]
fn function_frame_releases_fiber_start_argument() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class FramePayload {}
function startInFrame(Fiber $fiber, FramePayload $payload): void {
    $signal = $fiber->start($payload);
    unset($signal);
}

$payload = new FramePayload();
$fiber = new Fiber(function (FramePayload $value): void {});
startInFrame($fiber, $payload);
unset($fiber);
unset($payload);
echo "frame";
"#,
    );
    assert_clean(out, "frame");
}

#[test]
fn mixed_fiber_receiver_releases_start_argument() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class MixedFiberPayload {}
$payload = new MixedFiberPayload();
$fiber = new Fiber(function (MixedFiberPayload $value): void {});
$items = [$fiber];
$live = $items[0];
$signal = $live->start($payload);
unset($signal);
$items[0] = null;
unset($live);
unset($fiber);
unset($payload);
unset($items);
echo "mixed";
"#,
    );
    assert_clean(out, "mixed");
}

#[test]
fn captured_closure_releases_its_descriptor_and_capture() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$payload = ["answer" => 42];
$closure = function () use ($payload): string {
    return "value:" . $payload["answer"];
};
unset($closure);
unset($payload);
echo "closure";
"#,
    );
    assert_clean(out, "closure");
}

#[test]
fn descriptor_invoker_releases_captured_closure_arguments() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$payload = ["answer" => 42];
$closure = function () use ($payload): void {
    $payload["answer"];
};
call_user_func($closure);
unset($closure);
unset($payload);
echo "invoker";
"#,
    );
    assert_clean(out, "invoker");
}

#[test]
fn descriptor_invoker_releases_capture_free_closure() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$closure = function (): void {};
call_user_func($closure);
unset($closure);
echo "invoker-free";
"#,
    );
    assert_clean(out, "invoker-free");
}

#[test]
fn async_task_start_capsule_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__Scheduler;
use Elephc\Async\__TaskStart;
use Elephc\Async\__ScopeState;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$state = new __CancellationState();
$scheduler = new __Scheduler($state);
$scope = new __ScopeState($scheduler);
$group = new TaskGroup($scope, $state);
$fiber = new Fiber(function (TaskGroup $tasks): void {});
$start = __TaskStart::fromArguments($fiber, [$group]);
$owner = new class {
    public array $fibers = [];
    public array $starts = [];
};
$owner->fibers[] = $fiber;
$owner->starts[] = $start;
$live = $owner->fibers[0];
$before = $live->isStarted();
$liveStart = $owner->starts[0];
$owner->starts[0] = null;
$signal = $liveStart->start();
unset($signal);
$after = $live->isTerminated();
$return = $live->getReturn();
$owner->fibers[0] = null;
unset($after);
unset($return);
unset($before);
unset($live);
unset($liveStart);
unset($start);
unset($fiber);
$scope->detach();
unset($group);
unset($scope);
unset($scheduler);
unset($state);
unset($owner);
echo "capsule";
"#,
    );
    assert_clean(out, "capsule");
}

#[test]
fn async_task_start_capsule_rethrows_root_failure_without_refcount_loss() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__TaskStart;
use Elephc\Async\__ScopeState;
use Elephc\Async\__Scheduler;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$state = new __CancellationState();
$scheduler = new __Scheduler($state);
$scope = new __ScopeState($scheduler);
$group = new TaskGroup($scope, $state);
$fiber = new Fiber(function (TaskGroup $tasks): void {
    throw new RuntimeException("capsule-failure");
});
$start = __TaskStart::fromArguments($fiber, [$group]);
try {
    $start->start();
} catch (RuntimeException $caught) {
    echo $caught->getMessage();
    unset($caught);
}
$scope->detach();
unset($start);
unset($fiber);
unset($group);
unset($scope);
unset($scheduler);
unset($state);
"#,
    );
    assert_clean(out, "capsule-failure");
}

#[test]
fn async_task_start_dynamic_callable_failure_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__TaskStart;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$body = function (): void {
    throw new RuntimeException("task-start-dynamic");
};
$fiber = new Fiber($body);
$start = __TaskStart::fromNoArgs($fiber);
try {
    $start->start();
} catch (RuntimeException $caught) {
    echo $caught->getMessage();
    unset($caught);
}
unset($start);
unset($fiber);
unset($body);
"#,
    );
    assert_clean(out, "task-start-dynamic");
}

#[test]
fn async_caught_fiber_failure_can_be_stored_and_rethrown_once() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__TaskOutcome;
use Elephc\Async\__TaskStart;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$state = new __CancellationState();
$outcome = new __TaskOutcome();
$body = function (): void {
    throw new RuntimeException("caught-fiber");
};
$fiber = new Fiber($body);
$start = __TaskStart::fromNoArgs($fiber);
try {
    $start->start();
} catch (Throwable $error) {
    $outcome->recordFailure($error);
    $state->request($error);
}
unset($error);
try {
    $outcome->throwIfFailed();
} catch (RuntimeException $caught) {
    echo $caught->getMessage();
    unset($caught);
}
$state->__clearReason();
unset($start);
unset($fiber);
unset($body);
unset($outcome);
unset($state);
"#,
    );
    assert_clean(out, "caught-fiber");
}

#[test]
fn terminal_fiber_status_after_start_failure_keeps_caught_error_live() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(function (): void {
    throw new RuntimeException("terminal-status");
});
try {
    $fiber->start();
} catch (RuntimeException $caught) {
    $terminal = $fiber->isTerminated();
    echo $terminal ? $caught->getMessage() : "not-terminal";
    unset($terminal);
    unset($caught);
}
unset($fiber);
"#,
    );
    assert_clean(out, "terminal-status");
}

#[test]
fn async_empty_root_scope_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {});
echo "root";
"#,
    );
    assert_clean(out, "root");
}

#[test]
fn async_root_failure_is_heap_clean_through_public_scope() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

$body = function (TaskGroup $tasks): void {
    throw new RuntimeException("root-direct");
};
try {
    run($body);
} catch (RuntimeException $caught) {
    echo $caught->getMessage();
    unset($caught);
}
unset($body);
"#,
    );
    assert_clean(out, "root-direct");
}

#[test]
fn async_cancellation_releases_sleeping_task_state() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $child = $tasks->spawn(function () use ($tasks): string {
        try {
            $tasks->sleep(60.0);
        } catch (CancelledException $error) {
            return "cleaned";
        }
        return "unexpected";
    });
    $tasks->reschedule();
    $tasks->cancel();
    echo $child->await();
});
"#,
    );
    assert_clean(out, "cleaned");
}

#[test]
fn async_cancelled_sleeping_child_without_catch_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $tasks->spawn(function () use ($tasks): void {
        $tasks->sleep(60.0);
    });
    $tasks->reschedule();
    $tasks->cancel();
});
echo "cancelled";
"#,
    );
    assert_clean(out, "cancelled");
}

#[test]
fn async_root_reschedule_then_cancel_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $tasks->reschedule();
    $tasks->cancel();
});
echo "root-cancel";
"#,
    );
    assert_clean(out, "root-cancel");
}

#[test]
fn async_cancelled_suspended_child_without_task_group_capture_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $tasks->spawn(function (): void {
        Fiber::suspend(2);
    });
    $tasks->reschedule();
    $tasks->cancel();
});
echo "suspended";
"#,
    );
    assert_clean(out, "suspended");
}

#[test]
fn async_cancelled_suspended_child_with_task_group_capture_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $tasks->spawn(function () use ($tasks): void {
        if ($tasks->cancellation()->isRequested()) {
            echo "";
        }
        Fiber::suspend(2);
    });
    $tasks->reschedule();
    $tasks->cancel();
});
echo "captured";
"#,
    );
    assert_clean(out, "captured");
}

#[test]
fn scheduler_root_cancellation_releases_sleeping_child_state() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $tasks->spawn(function () use ($tasks): void {
        $tasks->sleep(60.0);
    });
    $tasks->reschedule();
    $tasks->cancel();
});
echo "cancelled";
"#,
    );
    assert_clean(out, "cancelled");
}

#[test]
fn suspended_fiber_escaping_exception_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(function (): void {
    Fiber::suspend();
    throw new RuntimeException("boom");
});
$fiber->start();
try {
    $fiber->resume();
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
unset($fiber);
"#,
    );
    assert_clean(out, "boom");
}

#[test]
fn suspended_fiber_escaping_exception_releases_capture() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class EscapingFiberCapture {}
$capture = new EscapingFiberCapture();
$fiber = new Fiber(function () use ($capture): void {
    Fiber::suspend();
    throw new RuntimeException("boom");
});
$fiber->start();
try {
    $fiber->resume();
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
unset($fiber);
unset($capture);
"#,
    );
    assert_clean(out, "boom");
}

#[test]
fn suspended_fiber_escaping_nested_exception_releases_capture() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
final class NestedEscapingFiberCapture {}
function nestedFiberFailure(): void {
    throw new RuntimeException("boom");
}
$capture = new NestedEscapingFiberCapture();
$fiber = new Fiber(function () use ($capture): void {
    Fiber::suspend();
    nestedFiberFailure();
});
$fiber->start();
try {
    $fiber->resume();
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
unset($fiber);
unset($capture);
"#,
    );
    assert_clean(out, "boom");
}

#[test]
fn suspended_fiber_escaping_exception_releases_task_group_capture() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__Scheduler;
use Elephc\Async\__ScopeState;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$state = new __CancellationState();
$scheduler = new __Scheduler($state);
$scope = new __ScopeState($scheduler);
$group = new TaskGroup($scope, $state);
$fiber = new Fiber(function () use ($group): void {
    Fiber::suspend();
    throw new RuntimeException("boom");
});
$fiber->start();
try {
    $fiber->resume();
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
$scope->detach();
unset($fiber);
unset($group);
unset($scope);
unset($scheduler);
unset($state);
"#,
    );
    assert_clean(out, "boom");
}

#[test]
fn suspended_fiber_escaping_cancelled_exception_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$fiber = new Fiber(function (): void {
    Fiber::suspend();
    throw new CancelledException();
});
$fiber->start();
try {
    $fiber->resume();
} catch (CancelledException $error) {
    echo "cancelled";
}
unset($fiber);
"#,
    );
    assert_clean(out, "cancelled");
}

#[test]
fn array_stored_suspended_fiber_escaping_cancelled_exception_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$fiber = new Fiber(function (): void {
    Fiber::suspend();
    throw new CancelledException();
});
$fibers = [$fiber];
$fiber->start();
try {
    $fibers[0]->resume();
} catch (CancelledException $error) {
    echo "cancelled";
}
$fibers = [];
unset($fiber);
"#,
    );
    assert_clean(out, "cancelled");
}

#[test]
fn array_stored_started_fiber_escaping_exception_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$fiber = new Fiber(function (): void {
    throw new CancelledException();
});
$fibers = [$fiber];
try {
    $fibers[0]->start();
} catch (CancelledException $error) {
    echo "started";
}
$fibers = [];
unset($fiber);
"#,
    );
    assert_clean(out, "started");
}

#[test]
fn array_stored_fiber_throw_releases_receiver_after_caught_delivery() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(function (): string {
    try {
        Fiber::suspend();
    } catch (RuntimeException $error) {
        return "caught";
    }
    return "missed";
});
$fibers = [$fiber];
$fiber->start();
$injected = new RuntimeException("injected");
$fibers[0]->throw($injected);
echo $fiber->getReturn();
$fibers = [];
unset($injected);
unset($fiber);
"#,
    );
    assert_clean(out, "caught");
}

#[test]
fn array_stored_fiber_throw_releases_receiver_before_escaped_exception() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(function (): void {
    Fiber::suspend();
});
$fibers = [$fiber];
$fiber->start();
$injected = new RuntimeException("injected");
try {
    $fibers[0]->throw($injected);
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
$fibers = [];
unset($injected);
unset($fiber);
"#,
    );
    assert_clean(out, "injected");
}

#[test]
fn array_stored_fiber_throw_releases_receiver_on_state_error() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$fiber = new Fiber(function (): void {});
$fibers = [$fiber];
$injected = new RuntimeException("injected");
try {
    $fibers[0]->throw($injected);
} catch (FiberError $error) {
    echo "invalid";
}
$fibers = [];
unset($injected);
unset($fiber);
"#,
    );
    assert_clean(out, "invalid");
}

#[test]
fn array_stored_fiber_loaded_into_local_before_exception_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$fiber = new Fiber(function (): void {
    Fiber::suspend();
    throw new CancelledException();
});
$fibers = [$fiber];
$fiber->start();
$stored = $fibers[0];
try {
    $stored->resume();
} catch (CancelledException $error) {
    echo "cancelled";
}
unset($stored);
$fibers = [];
unset($fiber);
"#,
    );
    assert_clean(out, "cancelled");
}

#[test]
fn method_catch_releases_terminated_fiber_loaded_from_property_array() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
final class FiberHolder {
    public array $fibers = [];
    public function drive(): void {
        $fiber = $this->fibers[0];
        try {
            $fiber->resume();
        } catch (CancelledException $error) {
            echo "method-catch";
        }
        unset($fiber);
        $this->fibers = [];
    }
}
$fiber = new Fiber(function (): void {
    Fiber::suspend();
    throw new CancelledException();
});
$holder = new FiberHolder();
$holder->fibers = [$fiber];
$fiber->start();
$holder->drive();
unset($fiber);
unset($holder);
"#,
    );
    assert_clean(out, "method-catch");
}

#[test]
fn fiber_catching_cancelled_exception_releases_task_group_capture() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__Scheduler;
use Elephc\Async\__ScopeState;
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) { run(function (TaskGroup $tasks): void {}); }
$state = new __CancellationState();
$scheduler = new __Scheduler($state);
$scope = new __ScopeState($scheduler);
$group = new TaskGroup($scope, $state);
$fiber = new Fiber(function () use ($group): void {
    Fiber::suspend();
    try { throw new CancelledException(); } catch (CancelledException $error) { echo "caught"; }
});
$fiber->start();
$fiber->resume();
$scope->detach();
unset($fiber);
unset($group);
unset($scope);
unset($scheduler);
unset($state);
"#,
    );
    assert_clean(out, "caught");
}

#[test]
fn cancellation_state_throw_if_requested_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\CancelledException;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) { run(function (TaskGroup $tasks): void {}); }
$state = new __CancellationState();
$state->request();
try { $state->throwIfRequested(); } catch (CancelledException $error) { echo "state"; }
unset($state);
"#,
    );
    assert_clean(out, "state");
}

#[test]
fn dynamic_fiber_descriptor_catch_preserves_string_return() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function eraseSuspendingFiberCallable(callable $callback): callable {
    return $callback;
}
$source = function (): string {
    try {
        Fiber::suspend();
        throw new RuntimeException("inside");
    } catch (RuntimeException $error) {
        return "cleaned";
    }
};
$callback = eraseSuspendingFiberCallable($source);
$fiber = new Fiber($callback);
unset($callback);
unset($source);
$fiber->start();
$fiber->resume();
echo $fiber->getReturn();
unset($fiber);
"#,
    );
    assert_clean(out, "cleaned");
}

#[test]
fn dynamic_fiber_descriptor_escape_releases_argument_cell() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function eraseThrowingFiberCallable(callable $callback): callable {
    return $callback;
}
$source = function (int $value): void {
    if ($value === 42) {
        throw new RuntimeException("dynamic");
    }
};
$callback = eraseThrowingFiberCallable($source);
$fiber = new Fiber($callback);
unset($callback);
unset($source);
try {
    $fiber->start(42);
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
unset($fiber);
"#,
    );
    assert_clean(out, "dynamic");
}

#[test]
fn suspended_dynamic_fiber_releases_parked_argument_container_on_destroy() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function eraseParkedFiberCallable(callable $callback): callable {
    return $callback;
}
$source = function (int $value): void {
    if ($value === 42) {
        Fiber::suspend();
    }
};
$callback = eraseParkedFiberCallable($source);
$fiber = new Fiber($callback);
unset($callback);
unset($source);
$fiber->start(42);
unset($fiber);
echo "parked";
"#,
    );
    assert_clean(out, "parked");
}

#[test]
fn async_failure_releases_task_outcomes_after_propagation() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

try {
    run(function (TaskGroup $tasks): void {
        $tasks->spawn(function (): void {
            throw new RuntimeException("failed");
        });
    });
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
"#,
    );
    assert_clean(out, "failed");
}

#[test]
fn async_root_failure_releases_root_task_state() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

try {
    run(function (TaskGroup $tasks): void {
        throw new RuntimeException("root");
    });
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
"#,
    );
    assert_clean(out, "root");
}

#[test]
fn async_task_outcome_retains_a_failure_through_explicit_rethrow() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__TaskOutcome;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$outcome = new __TaskOutcome();
$failure = new RuntimeException("outcome");
$outcome->recordFailure($failure);
unset($failure);
try {
    $outcome->throwIfFailed();
} catch (RuntimeException $caught) {
    echo $caught->getMessage();
    unset($caught);
}
unset($outcome);
"#,
    );
    assert_clean(out, "outcome");
}

#[test]
fn async_mixed_outcome_rethrow_keeps_failure_live_after_table_owner_releases() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__TaskOutcome;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$outcome = new __TaskOutcome();
$failure = new RuntimeException("mixed-outcome");
$outcome->recordFailure($failure);
unset($failure);
$table = [$outcome];
unset($outcome);
$dynamic = $table[0];
$table[0] = null;
try {
    $dynamic->throwIfFailed();
} catch (RuntimeException $caught) {
    echo $caught->getMessage();
    unset($caught);
}
unset($dynamic);
unset($table);
"#,
    );
    assert_clean(out, "mixed-outcome");
}

#[test]
fn clearing_an_outcome_table_before_rethrow_preserves_the_acquired_failure() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__TaskOutcome;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
final class OutcomeTable {
    public array $outcomes = [];
    public function throwFirst(): void {
        $outcome = $this->outcomes[0];
        $this->outcomes = [];
        $outcome->throwIfFailed();
    }
}
$outcome = new __TaskOutcome();
$failure = new RuntimeException("table");
$outcome->recordFailure($failure);
unset($failure);
$table = new OutcomeTable();
$table->outcomes[] = $outcome;
unset($outcome);
try {
    $table->throwFirst();
} catch (RuntimeException $caught) {
    echo $caught->getMessage();
    unset($caught);
}
unset($table);
"#,
    );
    assert_clean(out, "table");
}

#[test]
fn async_outcome_and_cancellation_reason_share_failure_without_refcount_loss() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\__CancellationState;
use Elephc\Async\__TaskOutcome;
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

if ($argc < 0) {
    run(function (TaskGroup $tasks): void {});
}
$state = new __CancellationState();
$outcome = new __TaskOutcome();
$failure = new RuntimeException("shared");
$outcome->recordFailure($failure);
$state->request($failure);
unset($failure);
try {
    $outcome->throwIfFailed();
} catch (RuntimeException $caught) {
    echo $caught->getMessage();
    unset($caught);
}
$state->__clearReason();
unset($outcome);
unset($state);
"#,
    );
    assert_clean(out, "shared");
}

#[test]
fn async_awaited_failure_releases_root_task_state() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

try {
    run(function (TaskGroup $tasks): void {
        $child = $tasks->spawn(function (): void {
            throw new RuntimeException("awaited");
        });
        $child->await();
    });
} catch (RuntimeException $error) {
    echo $error->getMessage();
}
"#,
    );
    assert_clean(out, "awaited");
}

#[test]
fn async_reactor_releases_registration_resources() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
use Elephc\Async\TaskGroup;
use function Elephc\Async\run;

run(function (TaskGroup $tasks): void {
    $pair = stream_socket_pair(1, 1, 0);
    $writer = $pair[0];
    $reader = $pair[1];
    stream_set_blocking($reader, false);
    $waiter = $tasks->spawn(function () use ($tasks, $reader): string {
        if (!$tasks->awaitReadable($reader, 1.0)) {
            return "timeout";
        }
        return fread($reader, 1);
    });
    $tasks->spawn(function () use ($writer): void {
        fwrite($writer, "x");
    });
    echo $waiter->await();
    fclose($writer);
    fclose($reader);
});
"#,
    );
    assert_clean(out, "x");
}
