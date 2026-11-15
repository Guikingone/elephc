//! Purpose:
//! Emits `__rt_mixed_array_unset`, which removes one key from the array or
//! hash payload of a boxed Mixed cell (`unset($mixed[$key])`).
//!
//! Called from:
//! - `RuntimeCallTarget::MixedArrayUnset` lowering in `crate::codegen::lower_inst`.
//!
//! Key details:
//! - Inputs: cell, normalized key (`key_hi == -1` marks an integer key). The
//!   cell is mutated in place; EIR lowering separates a shared local cell first
//!   (`__rt_mixed_separate`), so other owners never observe the removal.
//! - Hash payloads delegate to `__rt_hash_unset`, which copy-on-write splits a
//!   shared table, and the unique table is republished into the cell.
//! - Indexed payloads are left untouched for string or out-of-range keys (PHP
//!   keeps a list unchanged when the key does not exist). An existing integer
//!   key promotes the payload to hash storage first, because PHP keeps the
//!   surviving keys without renumbering.
//! - Null, scalar, and object payloads are a quiet no-op.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::sentinels::emit_branch_if_null_container;

/// Emits `__rt_mixed_array_unset` for the active target.
pub(crate) fn emit_mixed_array_unset(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_array_unset_x86_64(emitter);
        return;
    }
    emit_mixed_array_unset_aarch64(emitter);
}

/// Emits the AArch64 helper. Inputs: `x0` = cell, `x1` = key_lo, `x2` = key_hi.
fn emit_mixed_array_unset_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_unset ---");
    emitter.label_global("__rt_mixed_array_unset");

    emitter.instruction("sub sp, sp, #48");                                     // reserve slots for cell, key words, and saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the receiver cell
    emitter.instruction("stp x1, x2, [sp, #8]");                                // save the normalized key words
    emitter.instruction("cbz x0, __rt_mixed_array_unset_done");                 // null receivers hold no keys
    emitter.instruction("ldr x9, [x0]");                                        // load the boxed payload tag
    emitter.instruction("cmp x9, #5");                                          // is the payload an associative array?
    emitter.instruction("b.eq __rt_mixed_array_unset_hash");                    // remove the key from hash storage
    emitter.instruction("cmp x9, #4");                                          // is the payload an indexed array?
    emitter.instruction("b.ne __rt_mixed_array_unset_done");                    // scalars, null, and objects ignore the unset
    emitter.instruction("ldr x10, [x0, #8]");                                   // load the indexed-array payload
    emit_branch_if_null_container(emitter, "x10", "x9", "__rt_mixed_array_unset_done");
    emitter.instruction("cmn x2, #1");                                          // is the key an integer key?
    emitter.instruction("b.ne __rt_mixed_array_unset_done");                    // string keys never exist in a list payload
    emitter.instruction("cmp x1, #0");                                          // negative keys never exist in a list payload
    emitter.instruction("b.lt __rt_mixed_array_unset_done");                    // nothing to remove for negative keys
    emitter.instruction("ldr x11, [x10]");                                      // load the list length
    emitter.instruction("cmp x1, x11");                                         // is the key inside the list?
    emitter.instruction("b.hs __rt_mixed_array_unset_done");                    // missing keys leave the list unchanged
    emitter.instruction("bl __rt_mixed_cell_promote_to_hash");                  // keep surviving keys: convert the payload to hash storage
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the now-associative receiver cell

    emitter.label("__rt_mixed_array_unset_hash");
    emitter.instruction("ldr x0, [x0, #8]");                                    // load the hash payload
    emit_branch_if_null_container(emitter, "x0", "x9", "__rt_mixed_array_unset_done");
    emitter.instruction("ldp x1, x2, [sp, #8]");                                // reload the normalized key words
    emitter.instruction("bl __rt_hash_unset");                                  // remove the key (COW-splitting a shared table)
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the receiver cell
    emitter.instruction("str x0, [x9, #8]");                                    // publish the unique table back into the cell

    emitter.label("__rt_mixed_array_unset_done");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return to the caller
}

/// Emits the x86_64 helper. Inputs: `rdi` = cell, `rsi` = key_lo, `rdx` = key_hi.
fn emit_mixed_array_unset_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_unset ---");
    emitter.label_global("__rt_mixed_array_unset");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("sub rsp, 32");                                         // reserve slots for cell and key words
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the receiver cell
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save key_lo
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save key_hi
    emitter.instruction("test rdi, rdi");                                       // null receivers hold no keys
    emitter.instruction("jz __rt_mixed_array_unset_done");                      // nothing to remove
    emitter.instruction("mov r10, QWORD PTR [rdi]");                            // load the boxed payload tag
    emitter.instruction("cmp r10, 5");                                          // is the payload an associative array?
    emitter.instruction("je __rt_mixed_array_unset_hash");                      // remove the key from hash storage
    emitter.instruction("cmp r10, 4");                                          // is the payload an indexed array?
    emitter.instruction("jne __rt_mixed_array_unset_done");                     // scalars, null, and objects ignore the unset
    emitter.instruction("mov r10, QWORD PTR [rdi + 8]");                        // load the indexed-array payload
    emit_branch_if_null_container(emitter, "r10", "r11", "__rt_mixed_array_unset_done");
    emitter.instruction("cmp rdx, -1");                                         // is the key an integer key?
    emitter.instruction("jne __rt_mixed_array_unset_done");                     // string keys never exist in a list payload
    emitter.instruction("cmp rsi, 0");                                          // negative keys never exist in a list payload
    emitter.instruction("jl __rt_mixed_array_unset_done");                      // nothing to remove for negative keys
    emitter.instruction("cmp rsi, QWORD PTR [r10]");                            // is the key inside the list?
    emitter.instruction("jae __rt_mixed_array_unset_done");                     // missing keys leave the list unchanged
    emitter.instruction("call __rt_mixed_cell_promote_to_hash");                // keep surviving keys: convert the payload to hash storage
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the now-associative receiver cell

    emitter.label("__rt_mixed_array_unset_hash");
    emitter.instruction("mov rdi, QWORD PTR [rdi + 8]");                        // load the hash payload
    emit_branch_if_null_container(emitter, "rdi", "r11", "__rt_mixed_array_unset_done");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // reload key_lo
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // reload key_hi
    emitter.instruction("call __rt_hash_unset");                                // remove the key (COW-splitting a shared table)
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the receiver cell
    emitter.instruction("mov QWORD PTR [r10 + 8], rax");                        // publish the unique table back into the cell

    emitter.label("__rt_mixed_array_unset_done");
    emitter.instruction("mov rsp, rbp");                                        // release the helper slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the caller
}
