//! Purpose:
//! Tests whether a pending compound append still has a live PHP array owner.
//!
//! Called from:
//! - Typed runtime-call dispatch for MixedArrayCellHasLiveOwner.
//!
//! Key details:
//! - The operand borrows one ordinary cell lease, not a PHP reference wrapper.
//! - Exactly one surviving storage route keeps the operation active. A stored cell plus an
//!   independently boxed payload means COW already invalidated the pending target.
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
            ctx.emitter.instruction("cmp w9, #1");                              // detect a stored or copied owner of that exact cell
            ctx.emitter.instruction("cset x10, hi");                            // remember whether the cell itself survives elsewhere
            ctx.emitter.instruction("ldr x9, [x0, #8]");                        // separately boxed copies can retain this array payload
            ctx.emitter.instruction("ldr w9, [x9, #-12]");                      // inspect the payload independent owner count
            ctx.emitter.instruction("cmp w9, #1");                              // the pending cell itself owns one payload reference
            ctx.emitter.instruction("cset x11, hi");                            // remember whether the payload survives through another box
            ctx.emitter.instruction("eor x0, x10, x11");                        // exactly one survivor identifies PHP's still-selected storage
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
            ctx.emitter.instruction("seta r11b");                               // detect a stored or copied owner of that exact cell
            ctx.emitter.instruction("movzx r11, r11b");                         // materialize the complete cell-survivor predicate
            ctx.emitter.instruction("mov r10, QWORD PTR [rax + 8]");            // a separately boxed copy can own the same payload
            ctx.emitter.instruction("cmp DWORD PTR [r10 - 12], 1");             // discount the payload reference owned by the pending cell
            ctx.emitter.instruction("seta r10b");                               // detect an independently boxed payload survivor
            ctx.emitter.instruction("movzx r10, r10b");                         // materialize the complete payload-survivor predicate
            ctx.emitter.instruction("xor r11, r10");                            // exactly one survivor identifies PHP's still-selected storage
            ctx.emitter.instruction("mov rax, r11");                            // return the complete boolean result
            ctx.emitter.instruction(&format!("jmp {done}"));                    // keep the complete liveness result
            ctx.emitter.label(&live);
            ctx.emitter.instruction("mov eax, 1");                              // preserve live arrays and non-array update semantics
        }
    }
    ctx.emitter.label(&done);
    super::store_if_result(ctx, inst)
}
