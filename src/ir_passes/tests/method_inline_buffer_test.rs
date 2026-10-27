//! Purpose:
//! Checks whole checked-buffer accessor chains after generic specialization.
//!
//! Called from:
//! - The parent method-inlining EIR test module.
//!
//! Key details:
//! - Runtime loop bounds prevent constant folding; every supported target emits assembly.

use super::*;

/// Scalar generic accessors and their private guard chain splice into the actual loop.
#[test]
fn checked_managed_buffer_accessors_inline_on_every_target() {
    let source = include_str!("../../../examples/checked-buffer/main.php");
    for target in TARGETS {
        let module = optimized(source, target);
        let update = function(&module, "updateChecked");
        let calls = called_methods(&module, update);
        assert!(calls.is_empty(), "{target}: checked accessor calls survived: {calls:?}\n{}",
            crate::ir::print_function(update));
        assert!(placed(update).any(|inst| matches!(inst.op, Op::BufferGet | Op::BufferGetResolved)));
        assert!(placed(update).any(|inst| matches!(inst.op, Op::BufferSet | Op::BufferSetResolved)));
        let loops = crate::ir_passes::compute_loops(update, &crate::ir_passes::compute_dominance(update));
        let inner = loops.loops().iter().max_by_key(|loop_ref| loop_ref.depth).unwrap();
        for &block in &inner.blocks {
            for &id in &update.blocks[block.as_raw() as usize].instructions {
                let inst = &update.instructions[id.as_raw() as usize];
                assert!(!matches!(inst.op, Op::MethodCall | Op::PropGet | Op::BufferResolve | Op::BufferGet | Op::BufferSet),
                    "{target}: hot-loop {:?} survived\n{}", inst.op, crate::ir::print_function(update));
            }
        }
        crate::codegen::generate_user_asm_from_ir(&module, false, false)
            .unwrap_or_else(|error| panic!("{target}: {error:?}"));
    }
}
