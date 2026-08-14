//! Purpose:
//! Emits ownership helpers for request-global reference cells shared with PHP arrays.
//!
//! Called from:
//! - global load/store lowering, web reset, and tag-12 Mixed marker destruction.
//!
//! Key details:
//! - A cell owns one associative-array payload and uses the allocator header refcount for sharing.
//! - The final release decrefs the payload before freeing the cell allocation.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits global reference-cell incref/decref helpers for the current target.
pub fn emit_global_ref_cell(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_global_ref_cell_aarch64(emitter),
        Arch::X86_64 => emit_global_ref_cell_x86_64(emitter),
    }
}

/// Emits the AArch64 request-global reference-cell ownership helpers.
fn emit_global_ref_cell_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: request-global reference cell ownership ---");
    emitter.label_global("__rt_global_ref_cell_incref");
    emitter.instruction("cbz x0, __rt_global_ref_cell_incref_done");            // a missing cell has no ownership share to retain
    emitter.instruction("ldr w9, [x0, #-12]");                                 // load the allocator-header refcount
    emitter.instruction("add w9, w9, #1");                                     // retain one array-marker share
    emitter.instruction("str w9, [x0, #-12]");                                 // publish the incremented cell refcount
    emitter.label("__rt_global_ref_cell_incref_done");
    emitter.instruction("ret");                                                // return the unchanged cell pointer in x0

    emitter.blank();
    emitter.label_global("__rt_global_ref_cell_decref");
    emitter.instruction("cbz x0, __rt_global_ref_cell_decref_done");            // null globals need no release
    emitter.instruction("ldr w9, [x0, #-12]");                                 // load the shared cell refcount
    emitter.instruction("subs w9, w9, #1");                                    // release one global or marker owner
    emitter.instruction("str w9, [x0, #-12]");                                 // publish the remaining owner count
    emitter.instruction("b.ne __rt_global_ref_cell_decref_done");              // another owner keeps both cell and payload alive
    emitter.instruction("str x30, [sp, #-16]!");                               // preserve the return address across nested releases
    emitter.instruction("str x0, [sp, #8]");                                   // preserve the final cell pointer for heap_free
    emitter.instruction("ldr x0, [x0]");                                       // load the associative-array payload owned by the cell
    emitter.instruction("bl __rt_decref_hash");                                // release the final payload owner
    emitter.instruction("ldr x0, [sp, #8]");                                   // restore the cell pointer
    emitter.instruction("bl __rt_heap_free");                                  // free the cell allocation itself
    emitter.instruction("ldr x30, [sp], #16");                                 // restore the caller return address and stack
    emitter.label("__rt_global_ref_cell_decref_done");
    emitter.instruction("ret");                                                // finish the ownership release
}

/// Emits the Linux x86_64 request-global reference-cell ownership helpers.
fn emit_global_ref_cell_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: request-global reference cell ownership ---");
    emitter.label_global("__rt_global_ref_cell_incref");
    emitter.instruction("test rax, rax");                                      // a missing cell has no ownership share to retain
    emitter.instruction("jz __rt_global_ref_cell_incref_done");
    emitter.instruction("add DWORD PTR [rax - 12], 1");                        // retain one array-marker share in the allocator header
    emitter.label("__rt_global_ref_cell_incref_done");
    emitter.instruction("ret");                                                // return the unchanged cell pointer in rax

    emitter.blank();
    emitter.label_global("__rt_global_ref_cell_decref");
    emitter.instruction("test rax, rax");                                      // null globals need no release
    emitter.instruction("jz __rt_global_ref_cell_decref_done");
    emitter.instruction("sub DWORD PTR [rax - 12], 1");                        // release one global or marker owner
    emitter.instruction("jnz __rt_global_ref_cell_decref_done");               // another owner keeps both cell and payload alive
    emitter.instruction("push rbp");                                           // establish an aligned frame for nested helper calls
    emitter.instruction("mov rbp, rsp");
    emitter.instruction("sub rsp, 16");
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                       // preserve the final cell pointer for heap_free
    emitter.instruction("mov rax, QWORD PTR [rax]");                           // load the associative-array payload owned by the cell
    emitter.instruction("call __rt_decref_hash");                              // release the final payload owner
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                       // restore the cell pointer
    emitter.instruction("call __rt_heap_free");                                // free the cell allocation itself
    emitter.instruction("leave");                                              // restore the caller frame and stack
    emitter.label("__rt_global_ref_cell_decref_done");
    emitter.instruction("ret");                                                // finish the ownership release
}
