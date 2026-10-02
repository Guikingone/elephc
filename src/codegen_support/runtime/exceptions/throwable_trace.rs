//! Purpose:
//! Emits `__rt_throwable_capture_trace`, which records the interpreter frames live at a compiled
//! `new` into the new Throwable's declared `trace` slot.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via the exception runtime section.
//! - Throwable allocation in `crate::codegen::lower_inst::objects::throwable_new`, only in modules
//!   that link the eval bridge.
//!
//! Key details:
//! - The frames come from `_elephc_eval_throwable_trace_fn`, which the interpreter installs when
//!   it creates its first context. A zero slot or a null answer leaves the slot untouched, so a
//!   Throwable created with no interpreter frame live keeps its declared empty trace.
//! - The callback hands back an OWNED boxed Mixed array. The helper keeps an owned reference to
//!   the array payload, releases the box, releases whatever the slot held before (the empty array
//!   a class-declared default stores, or nothing in a zeroed compact payload), and then stores the
//!   array. `__rt_object_free_deep` releases it with the object, since the slot is declared
//!   `array`.
//! - Input is the object pointer in `x0`/`rax`; every caller-saved register is clobbered.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::throwable_layout::TRACE_OFFSET;

/// Emits the `__rt_throwable_capture_trace` runtime helper for the target architecture.
pub fn emit_throwable_capture_trace(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_throwable_capture_trace_aarch64(emitter),
        Arch::X86_64 => emit_throwable_capture_trace_x86_64(emitter),
    }
}

/// ARM64 body. Frame: `[sp]` object, `[sp+8]` returned box, `[sp+16]` array payload.
fn emit_throwable_capture_trace_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: throwable_capture_trace ---");
    emitter.label_global("__rt_throwable_capture_trace");
    emitter.instruction("sub sp, sp, #48");                                     // reserve the helper frame
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // keep the new Throwable across the callback
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_throwable_trace_fn");
    emitter.instruction("ldr x9, [x9]");                                        // load the interpreter's trace callback, if installed
    emitter.instruction("cbz x9, __rt_throwable_capture_trace_done");           // no interpreter: keep the declared empty trace
    emitter.instruction("blr x9");                                              // ask for the interpreter frames live right now
    emitter.instruction("cbz x0, __rt_throwable_capture_trace_done");           // no live frame: keep the declared empty trace
    emitter.instruction("str x0, [sp, #8]");                                    // keep the owned box for its release
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0 = tag, x1 = array payload
    emitter.instruction("cmp x0, #4");                                          // an indexed array is the expected shape
    emitter.instruction("b.eq __rt_throwable_capture_trace_store");             // take the array
    emitter.instruction("cmp x0, #5");                                          // an associative array is accepted as well
    emitter.instruction("b.ne __rt_throwable_capture_trace_release_box");       // anything else is dropped
    emitter.label("__rt_throwable_capture_trace_store");
    emitter.instruction("str x1, [sp, #16]");                                   // keep the array payload
    emitter.instruction("mov x0, x1");                                          // the slot takes its own reference to the array
    emitter.instruction("bl __rt_incref");                                      // retain the array for the Throwable
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the returned box
    emitter.instruction("bl __rt_decref_mixed");                                // release the box; the array survives on the slot's reference
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the Throwable
    emitter.instruction(&format!("ldr x0, [x9, #{}]", TRACE_OFFSET));            // load whatever the trace slot held before
    emitter.instruction("bl __rt_decref_any");                                  // release it (a declared empty array, or nothing)
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the Throwable
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the retained array payload
    emitter.instruction(&format!("str x10, [x9, #{}]", TRACE_OFFSET));           // publish the frames as the Throwable's trace
    emitter.instruction("b __rt_throwable_capture_trace_done");                 // finish
    emitter.label("__rt_throwable_capture_trace_release_box");
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the unexpected box
    emitter.instruction("bl __rt_decref_mixed");                                // release it without touching the Throwable
    emitter.label("__rt_throwable_capture_trace_done");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return to the allocating site
}

/// x86_64 body. Frame: `[rbp-8]` object, `[rbp-16]` returned box, `[rbp-24]` array payload.
fn emit_throwable_capture_trace_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: throwable_capture_trace ---");
    emitter.label_global("__rt_throwable_capture_trace");
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    emitter.instruction("sub rsp, 32");                                         // reserve the helper slots
    emitter.instruction("and rsp, -16");                                        // align for the Rust callback whatever the caller's alignment
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // keep the new Throwable across the callback
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_throwable_trace_fn");
    emitter.instruction("mov r10, QWORD PTR [r10]");                            // load the interpreter's trace callback, if installed
    emitter.instruction("test r10, r10");                                       // is an interpreter linked and initialized?
    emitter.instruction("jz __rt_throwable_capture_trace_done");                // no interpreter: keep the declared empty trace
    emitter.instruction("call r10");                                            // ask for the interpreter frames live right now
    emitter.instruction("test rax, rax");                                       // did the interpreter have any frame to report?
    emitter.instruction("jz __rt_throwable_capture_trace_done");                // no live frame: keep the declared empty trace
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // keep the owned box for its release
    emitter.instruction("call __rt_mixed_unbox");                               // rax = tag, rdi = array payload
    emitter.instruction("cmp rax, 4");                                          // an indexed array is the expected shape
    emitter.instruction("je __rt_throwable_capture_trace_store");               // take the array
    emitter.instruction("cmp rax, 5");                                          // an associative array is accepted as well
    emitter.instruction("jne __rt_throwable_capture_trace_release_box");        // anything else is dropped
    emitter.label("__rt_throwable_capture_trace_store");
    emitter.instruction("mov QWORD PTR [rbp - 24], rdi");                       // keep the array payload
    emitter.instruction("mov rax, rdi");                                        // the slot takes its own reference to the array
    emitter.instruction("call __rt_incref");                                    // retain the array for the Throwable
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the returned box
    emitter.instruction("call __rt_decref_mixed");                              // release the box; the array survives on the slot's reference
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the Throwable
    emitter.instruction(&format!("mov rax, QWORD PTR [r10 + {}]", TRACE_OFFSET)); // load whatever the trace slot held before
    emitter.instruction("call __rt_decref_any");                                // release it (a declared empty array, or nothing)
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the Throwable
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // reload the retained array payload
    emitter.instruction(&format!("mov QWORD PTR [r10 + {}], r11", TRACE_OFFSET)); // publish the frames as the Throwable's trace
    emitter.instruction("jmp __rt_throwable_capture_trace_done");               // finish
    emitter.label("__rt_throwable_capture_trace_release_box");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the unexpected box
    emitter.instruction("call __rt_decref_mixed");                              // release it without touching the Throwable
    emitter.label("__rt_throwable_capture_trace_done");
    emitter.instruction("mov rsp, rbp");                                        // drop the helper slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the allocating site
}
