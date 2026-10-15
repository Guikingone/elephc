//! Purpose:
//! Tests whether a pending compound append still has a live PHP array owner.
//!
//! Called from:
//! - Typed runtime-call dispatch for MixedArrayCellHasLiveOwner.
//!
//! Key details:
//! - The operand borrows one ordinary cell lease, not a PHP reference wrapper.
//! - A stored cell or independently boxed payload copy keeps the operation active.
//! - Compiler-only ancestor leases must be retired before this non-retaining query.

use crate::codegen::context::FunctionContext;
use crate::codegen::platform::Arch;
use crate::codegen::Result;
use crate::ir::Instruction;

/// Reads cell and array/hash ownership without callbacks or PHP-visible comparisons.
pub(super) fn lower(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ctx.load_value_to_result(super::expect_operand(inst, 0)?)?;
    let payload = ctx.next_label("append_live_array_payload");
    let live = ctx.next_label("append_array_live");
    let done = ctx.next_label("append_array_liveness_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x0, {live}"));                // non-array absent cells keep their existing update semantics
            ctx.emitter.instruction("ldr x9, [x0]");                            // inspect the canonical PHP cell tag
            ctx.emitter.instruction("cmp x9, #4");                              // tag four owns an indexed array payload
            ctx.emitter.instruction(&format!("b.eq {payload}"));                // indexed arrays use the shared payload owner check
            ctx.emitter.instruction("cmp x9, #5");                              // tag five owns an associative array payload
            ctx.emitter.instruction(&format!("b.ne {live}"));                   // do not cancel object or scalar offset behavior here
            ctx.emitter.label(&payload);
            ctx.emitter.instruction("ldr w9, [x0, #-12]");                      // one cell owner is the pending compiler lease
            ctx.emitter.instruction("cmp w9, #1");                              // another cell owner keeps its stored array reachable
            ctx.emitter.instruction(&format!("b.hi {live}"));                   // a stored or copied cell preserves the pending operation
            ctx.emitter.instruction("ldr x9, [x0, #8]");                        // separately boxed copies can retain this array payload
            ctx.emitter.instruction("ldr w9, [x9, #-12]");                      // inspect the payload independent owner count
            ctx.emitter.instruction("cmp w9, #1");                              // the pending cell itself owns one payload reference
            ctx.emitter.instruction("cset x0, hi");                             // a separate payload owner also preserves the operation
            ctx.emitter.instruction(&format!("b {done}"));                      // keep the complete liveness result
            ctx.emitter.label(&live);
            ctx.emitter.instruction("mov x0, #1");                              // preserve live arrays and non-array update semantics
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // recognize a non-array absent cell
            ctx.emitter.instruction(&format!("jz {live}"));                     // absent cells retain their existing update semantics
            ctx.emitter.instruction("cmp QWORD PTR [rax], 4");                  // tag four owns an indexed array payload
            ctx.emitter.instruction(&format!("je {payload}"));                  // indexed arrays share the payload owner check
            ctx.emitter.instruction("cmp QWORD PTR [rax], 5");                  // tag five owns an associative array payload
            ctx.emitter.instruction(&format!("jne {live}"));                    // do not cancel object or scalar offset behavior here
            ctx.emitter.label(&payload);
            ctx.emitter.instruction("cmp DWORD PTR [rax - 12], 1");             // one cell owner is the pending compiler lease
            ctx.emitter.instruction(&format!("ja {live}"));                     // another cell owner preserves the pending operation
            ctx.emitter.instruction("mov r10, QWORD PTR [rax + 8]");            // a separately boxed copy can own the same payload
            ctx.emitter.instruction("cmp DWORD PTR [r10 - 12], 1");             // discount the payload reference owned by the pending cell
            ctx.emitter.instruction("seta al");                                 // another payload owner preserves the operation
            ctx.emitter.instruction("movzx eax, al");                           // materialize the complete boolean result
            ctx.emitter.instruction(&format!("jmp {done}"));                    // keep the complete liveness result
            ctx.emitter.label(&live);
            ctx.emitter.instruction("mov eax, 1");                              // preserve live arrays and non-array update semantics
        }
    }
    ctx.emitter.label(&done);
    super::store_if_result(ctx, inst)
}
