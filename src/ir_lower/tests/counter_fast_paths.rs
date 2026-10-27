//! Purpose:
//! Verifies guarded counter copies, dead seed overflow and arithmetic through index locals.
//!
//! Called from:
//! - The AST-to-EIR library regression suite.
//!
//! Key details:
//! - Every assertion inspects the named user function, avoiding injected prelude counts.

use crate::ir::{Function, Op};

/// Lowers and optimizes one fixture for a supported target, returning its user function.
fn optimized(source: &str, name: &str, target: &str) -> Function {
    let mut module = super::lower_source_at_for_target(source, std::path::Path::new("main.php"),
        std::path::Path::new("."), crate::codegen::platform::Target::parse(target).unwrap());
    crate::ir_passes::optimize_module(&mut module);
    module.functions.into_iter().find(|f| f.name == name).unwrap()
}

/// Inclusive integer bounds get both a scalar loop and the original overflow-capable fallback.
#[test]
fn counter_fast_path_inclusive_loop_has_integer_version_on_all_targets() {
    let source = r#"<?php
function inclusiveCounter(int $start, int $end): int {
    $count = 0;
    for ($i = $start; $i <= $end; $i++) { $count += 1; }
    return $count;
}
echo inclusiveCounter($argc, $argc + 4);
"#;
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let function = optimized(source, "inclusiveCounter", target);
        let text = crate::ir::print_function(&function);
        assert!(text.contains(".counter.fast"), "{target}: {text}");
        assert!(function.instructions.iter().any(|i| i.op == Op::StrIncDec), "{target}: {text}");
        assert!(function.instructions.iter().any(|i| i.op == Op::IAdd), "{target}: {text}");
    }
}

/// A checked product used as a stable loop bound is guarded by its concrete runtime tag.
#[test]
fn counter_fast_path_checked_bound_has_integer_version() {
    let source = r#"<?php
function productCounter(int $a, int $b): int {
    $cells = $a * $b;
    $count = 0;
    for ($cell = 0; $cell < $cells; $cell++) { $count += 1; }
    return $count;
}
echo productCounter($argc, $argc + 4);
"#;
    let function = optimized(source, "productCounter", "linux-x86_64");
    assert!(crate::ir::print_function(&function).contains(".counter.fast"), "{}", crate::ir::print_function(&function));
}

/// A seed below PHP_INT_MIN is unobservable when a dead-at-exit descending loop never enters.
#[test]
fn counter_fast_path_descending_seed_skips_overflow_without_loop_copy() {
    let source = r#"<?php
function descendingCounter(int $n): int {
    $count = 0;
    for ($box = $n - 1; $box >= 0; $box--) { $count += 1; }
    return $count;
}
echo descendingCounter($argc + 4);
"#;
    let function = optimized(source, "descendingCounter", "linux-x86_64");
    let text = crate::ir::print_function(&function);
    assert!(text.contains("counter.seed.inrange"), "{text}");
    assert!(!function.instructions.iter().any(|i| i.op == Op::StrIncDec), "{text}");
}

/// An index's suffix remains inside the original float-promoting arithmetic chain.
#[test]
fn counter_fast_path_index_chain_crosses_private_local() {
    let source = r#"<?php
function cellIndex(int $row, int $columns, int $column): int {
    $cell = $row * $columns + $column;
    return (int)($cell + 1);
}
echo cellIndex($argc, $argc + 4, $argc);
"#;
    let function = optimized(source, "cellIndex", "linux-x86_64");
    let text = crate::ir::print_function(&function);
    assert!(text.contains("ichecked_numeric_chain_to_int"), "{text}");
    assert!(!function.instructions.iter().any(|i| i.op == Op::MixedNumericBinop), "{text}");
}

/// A changing bound cannot be frozen at loop entry by integer loop versioning.
#[test]
fn counter_fast_path_mutable_bound_keeps_original_loop() {
    let source = r#"<?php
function changingCounter(int $end): int {
    $count = 0;
    for ($i = 0; $i <= $end; $i++) { $end -= 1; $count += 1; }
    return $count;
}
echo changingCounter($argc + 4);
"#;
    let function = optimized(source, "changingCounter", "linux-x86_64");
    assert!(!crate::ir::print_function(&function).contains(".counter.fast"));
}

/// A data slot shared by both loop copies can fuse its integer writer while retaining the fallback.
#[test]
fn counter_fast_path_index_inside_versioned_loop_fuses_integer_copy() {
    let source = r#"<?php
function guardedIndex(int $end, int $columns, int $column): int {
    for ($row = 0; $row <= $end; $row++) {
        $cell = $row * $columns + $column;
        echo (int)($cell + 1);
    }
    return 0;
}
echo guardedIndex($argc, 31, 7);
"#;
    let function = optimized(source, "guardedIndex", "linux-x86_64");
    let text = crate::ir::print_function(&function);
    assert!(text.contains(".counter.fast"), "{text}");
    assert!(function.instructions.iter().any(|i| i.op == Op::ICheckedNumericChainToInt), "{text}");
    assert!(function.instructions.iter().any(|i| i.op == Op::MixedNumericBinop), "{text}");
}
