//! Purpose:
//! Checks descriptor hoisting through direct and inlined buffer reads on invariant objects.
//!
//! Called from:
//! - The EIR pass library test harness.
//!
//! Key details:
//! - Real lowering retains method receiver guards and private-property lexical scopes.

use std::path::Path;

use crate::codegen::platform::Target;
use crate::ir::{Module, Op};
use crate::ir_passes::{compute_dominance, compute_loops};

/// Lowers the ECS example and applies the complete EIR optimization pipeline.
fn optimized(target: &str) -> Module {
    let mut module = crate::ir_lower::tests::lower_source_at_for_target(
        include_str!("../../../examples/buffer-object-loop/main.php"),
        Path::new("main.php"), Path::new("."), Target::parse(target).unwrap(),
    );
    crate::ir_passes::optimize_module(&mut module);
    module
}

/// Both accessor forms resolve every buffer before the loop on every supported backend.
#[test]
fn buffer_object_loop_direct_and_inlined_hoist_on_all_targets() {
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = optimized(target);
        for name in ["moveDirect", "moveInlined"] {
            let function = module.functions.iter().find(|function| function.name == name).unwrap();
            let dominance = compute_dominance(function);
            let loops = compute_loops(function, &dominance);
            assert_eq!(loops.loops().len(), 1, "{target}: {name}");
            let loop_ref = &loops.loops()[0];
            let resolves: Vec<_> = function.blocks.iter().flat_map(|block| {
                block.instructions.iter().filter_map(|id| {
                    let inst = &function.instructions[id.as_raw() as usize];
                    (inst.op == Op::BufferResolve).then_some(block.id)
                })
            }).collect();
            assert_eq!(resolves.len(), 2, "{target}: {name}");
            assert!(resolves.iter().all(|&block| !loop_ref.contains(block)), "{target}: {name}");
            for &block in &loop_ref.blocks {
                for &id in &function.blocks[block.as_raw() as usize].instructions {
                    let inst = &function.instructions[id.as_raw() as usize];
                    assert!(!matches!(inst.op, Op::BufferGet | Op::BufferSet | Op::MethodCall), "{target}: {name}: {:?}", inst.op);
                }
            }
        }
        crate::codegen::generate_user_asm_from_ir(&module, false, false)
            .expect("emit optimized object loop");
    }
}

/// A foreign property read that can warn and continue must remain a descriptor-hoisting barrier.
#[test]
fn buffer_object_loop_unconsumed_read_blocks_descriptor_motion() {
    let source = r#"<?php
final class Slots { public buffer<int> $slots; }
function scan(Slots $table, buffer<int> $positions): void {
    for ($index = 0; $index < buffer_len($positions); $index++) {
        $table->slots;
        $positions[$index] = 7;
    }
}
"#;
    let mut module = crate::ir_lower::tests::lower_source_at_for_target(
        source, Path::new("main.php"), Path::new("."), Target::parse("linux-x86_64").unwrap(),
    );
    crate::ir_passes::optimize_module(&mut module);
    let function = module.functions.iter().find(|function| function.name == "scan").unwrap();
    assert!(!function.instructions.iter().any(|inst| inst.op == Op::BufferResolve));
}

/// A runtime count parameter supports the task's original loop shape without a buffer-length bound.
#[test]
fn buffer_object_loop_runtime_count_parameter_hoists_resolution() {
    let source = include_str!("../../../examples/buffer-object-loop/main.php")
        .replace("buffer<Position> $positions): void", "buffer<Position> $positions, int $count): void")
        .replace("    $count = buffer_len($positions);\n", "")
        .replace("($table, $positions);", "($table, $positions, $argc + 3);");
    let mut module = crate::ir_lower::tests::lower_source_at_for_target(
        &source, Path::new("main.php"), Path::new("."), Target::parse("linux-x86_64").unwrap(),
    );
    crate::ir_passes::optimize_module(&mut module);
    for name in ["moveDirect", "moveInlined"] {
        let function = module.functions.iter().find(|function| function.name == name).unwrap();
        let dominance = compute_dominance(function);
        let loops = compute_loops(function, &dominance);
        assert_eq!(loops.loops().len(), 1);
        assert_eq!(function.instructions.iter().filter(|inst| inst.op == Op::BufferResolve).count(), 2, "{name}");
        for &block in &loops.loops()[0].blocks {
            assert!(function.blocks[block.as_raw() as usize].instructions.iter().all(|id| {
                !matches!(function.instructions[id.as_raw() as usize].op, Op::BufferGet | Op::BufferSet | Op::BufferResolve)
            }), "{name}");
        }
    }
}
