//! Purpose:
//! Verifies in-process heap measurements for frame-phase profiling.
//!
//! Called from:
//! - The codegen integration suite through `runtime_gc`.
//!
//! Key details:
//! - Assertions compare allocation lifetimes rather than PHP-specific byte totals.
//! - Extension counters are sampled without constructing an allocating status array.

use crate::support::*;

/// Live bytes follow ownership, while reset starts the next peak at current usage.
#[test]
fn test_memory_metrics_live_peak_reset() {
    let source = r#"<?php
$base = memory_get_usage();
\MEMORY_RESET_PEAK_USAGE();
echo memory_get_peak_usage() === $base ? "reset|" : "bad|";
$values = [1, 2, 3];
$live = \MEMORY_GET_USAGE(real_usage: false);
$peak = \MEMORY_GET_PEAK_USAGE();
echo $live > $base && $peak >= $live ? "live|" : "bad|";
memory_reset_peak_usage();
echo memory_get_peak_usage() === $live ? "held|" : "bad|";
unset($values);
echo memory_get_usage() === $base ? "released|" : "bad|";
echo memory_get_peak_usage() >= $live ? "peak|" : "bad|";
memory_reset_peak_usage();
echo memory_get_peak_usage() === memory_get_usage() ? "reset" : "bad";
"#;
    assert_eq!(compile_and_run(source), "reset|live|held|released|peak|reset");
}

/// Reserved bytes include free arena space and remain fixed through allocation and reset.
#[test]
fn test_memory_metrics_reserved_arena() {
    let source = r#"<?php
$real = memory_get_usage(real_usage: $argc > 0);
echo $real, ":", memory_get_peak_usage(...[true]), "|";
echo memory_get_usage(0) === memory_get_usage("0") && memory_get_usage(0.0) === memory_get_usage(false) ? "coerced|" : "bad|";
$values = [1, 2, 3];
echo memory_get_usage() < $real ? "live|" : "bad|";
unset($values);
gc_mem_caches();
memory_reset_peak_usage();
echo memory_get_usage(true), ":", memory_get_peak_usage(real_usage: true);
"#;
    assert_eq!(compile_and_run_with_heap_size(source, 131072), "131072:131072|coerced|live|131072:131072");
}

/// Allocation/free counters include COW, growth and reuse, and reads never allocate.
#[test]
fn test_memory_metrics_allocation_counters() {
    let source = r#"<?php
$allocs = memory_get_allocations();
$frees = memory_get_frees();
echo memory_get_allocations() === $allocs && memory_get_frees() === $frees ? "cheap|" : "bad|";
$values = [1, 2, 3];
$copy = $values;
$copy[] = 4;
$copy[] = 5;
$after = memory_get_allocations();
echo $after > $allocs ? "allocated|" : "bad|";
unset($copy);
unset($values);
echo memory_get_frees() > $frees ? "freed|" : "bad|";
$beforeReuse = memory_get_allocations();
$again = [4, 5, 6];
echo memory_get_allocations() > $beforeReuse ? "reused|" : "bad|";
unset($again);
$beforeReset = memory_get_allocations();
$freeBeforeReset = memory_get_frees();
memory_reset_peak_usage();
echo memory_get_allocations() === $beforeReset && memory_get_frees() === $freeBeforeReset ? "stable" : "bad";
"#;
    assert_eq!(compile_and_run(source), "cheap|allocated|freed|reused|stable");
}

/// First-class and runtime-selected calls preserve optional flags and void reset results.
#[test]
fn test_memory_metrics_callables() {
    let source = r#"<?php
function sampleMemory(callable $read): int { return $read(true); }
$read = memory_get_usage(...);
echo sampleMemory($read) > 0 ? "first|" : "bad|";
$name = $argc > 0 ? "memory_get_peak_usage" : "memory_get_usage";
echo $name(real_usage: true) > 0 ? "dynamic|" : "bad|";
$reset = memory_reset_peak_usage(...);
echo $reset() === null ? "void|" : "bad|";
echo call_user_func("memory_get_allocations") >= call_user_func("memory_get_frees") ? "counts|" : "bad|";
echo function_exists("MEMORY_GET_USAGE") && function_exists("memory_reset_peak_usage") ? "exists" : "bad";
"#;
    assert_eq!(compile_and_run(source), "first|dynamic|void|counts|exists");
}

/// Opaque eval and eval callables share the native arena and published counter operations.
#[test]
fn test_memory_metrics_eval_bundle() {
    let source = r#"<?php
$code = 'echo memory_get_usage(real_usage: true) > 0 ? "arena|" : "bad|";
$held = [1, 2, 3];
memory_reset_peak_usage();
echo memory_get_peak_usage() > 0 ? "peak|" : "bad|";
echo call_user_func("memory_get_usage", true) === memory_get_peak_usage(true) ? "shared|" : "bad|";
$reader = memory_get_allocations(...);
echo $reader() > 0 && memory_get_frees() >= 0 ? "counts|" : "bad|";
$reset = memory_reset_peak_usage(...);
$reset();
echo function_exists("MEMORY_GET_USAGE") ? "exists" : "bad";' . ' // ' . $argc;
eval($code);
"#;
    let output = compile_and_run_with_heap_debug(source);
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, "arena|peak|shared|counts|exists");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}
