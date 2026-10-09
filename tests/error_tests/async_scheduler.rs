//! Purpose:
//! Compile-time diagnostics for the Elephc Async structured scheduler surface.
//!
//! Called from:
//! - `cargo test --test error_tests async_scheduler`.
//!
//! Key details:
//! - The prelude is injected before name resolution so diagnostics exercise its
//!   real public signature rather than failing as an undefined function.

use super::*;

#[test]
fn async_run_requires_a_body() {
    expect_error(
        r#"<?php
use function Elephc\Async\run;
run();
"#,
        "expects 1 arguments, got 0",
    );
}

#[test]
fn async_run_requires_a_callable_body() {
    expect_error(
        r#"<?php
use function Elephc\Async\run;
run(42);
"#,
        "expects Callable, got Int",
    );
}

#[test]
fn parallel_join_in_async_root_task_warns_about_blocking_the_scheduler() {
    expect_warning(
        r#"<?php
\Elephc\Parallel\run(static function (\Elephc\Parallel\TaskGroup $parallel): void {
    $future = $parallel->spawn(static fn (): int => 42);
    \Elephc\Async\run(static function (\Elephc\Async\TaskGroup $tasks) use ($future): void {
        $future->join();
    });
});
"#,
        "Future::join() inside an Elephc Async task blocks the whole cooperative scheduler thread",
    );
}

#[test]
fn parallel_run_in_async_root_task_warns_about_blocking_the_scheduler() {
    expect_warning(
        r#"<?php
\Elephc\Async\run(static function (\Elephc\Async\TaskGroup $tasks): void {
    \Elephc\Parallel\run(static function (\Elephc\Parallel\TaskGroup $parallel): void {
        $parallel->spawn(static fn (): int => 42);
    });
});
"#,
        "Parallel\\run() inside an Elephc Async task blocks the whole cooperative scheduler thread until the Parallel scope drains",
    );
}

#[test]
fn parallel_join_in_spawned_async_task_warns_once() {
    let result = check_source_full(
        r#"<?php
\Elephc\Parallel\run(static function (\Elephc\Parallel\TaskGroup $parallel): void {
    $future = $parallel->spawn(static fn (): int => 42);
    \Elephc\Async\run(static function (\Elephc\Async\TaskGroup $tasks) use ($future): void {
        $tasks->spawn(static function () use ($future): void {
            $future->join();
        });
    });
});
"#,
    )
    .expect("expected source to type-check");
    let warnings = result
        .warnings
        .iter()
        .filter(|warning| warning.message.contains("blocks the whole cooperative scheduler thread"))
        .count();
    assert_eq!(warnings, 1, "expected one blocking-join warning: {:?}", result.warnings);
}

#[test]
fn parallel_join_outside_async_task_does_not_warn() {
    expect_no_warning(
        r#"<?php
\Elephc\Parallel\run(static function (\Elephc\Parallel\TaskGroup $parallel): void {
    $future = $parallel->spawn(static fn (): int => 42);
    $future->join();
});
"#,
        "blocks the whole cooperative scheduler thread",
    );
}

#[test]
fn parallel_join_in_unexecuted_nested_closure_does_not_warn() {
    expect_no_warning(
        r#"<?php
\Elephc\Parallel\run(static function (\Elephc\Parallel\TaskGroup $parallel): void {
    $future = $parallel->spawn(static fn (): int => 42);
    \Elephc\Async\run(static function (\Elephc\Async\TaskGroup $tasks) use ($future): void {
        $later = static function () use ($future): void {
            $future->join();
        };
    });
});
"#,
        "blocks the whole cooperative scheduler thread",
    );
}
