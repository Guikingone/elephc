//! Purpose:
//! Emits the `__rt_object_from_hash` runtime helper used by the PHP `(object)` cast.
//! Iterates a source hash and copies every entry into a fresh `stdClass` property map.
//!
//! Called from:
//! - `crate::codegen_support::runtime::objects` via the top-level runtime emitter.
//! - `__rt_object_from_mixed`, which delegates array/hash payloads here.
//!
//! Key details:
//! - String keys are used verbatim; integer keys are converted to decimal-string
//!   property names. Values are boxed into owned `Mixed` cells (already-boxed
//!   entries are retained instead of re-wrapped) and stored via `__rt_stdclass_set`.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits `__rt_object_from_hash(hash_ptr) -> obj_ptr` for the active target.
///
/// Allocates a fresh `stdClass`, walks the source hash in insertion order via
/// `__rt_hash_iter_next`, boxes each value into an owned `Mixed` cell, normalizes
/// integer keys to decimal strings, and installs each pair as a dynamic property.
/// Returns the populated object; the source hash is only borrowed.
pub fn emit_object_from_hash(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_object_from_hash_x86_64(emitter);
        return;
    }
    emit_object_from_hash_aarch64(emitter);
}

/// ARM64 implementation of `__rt_object_from_hash` (input `x0`, result `x0`).
fn emit_object_from_hash_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: object_from_hash ---");
    emitter.label_global("__rt_object_from_hash");

    // Frame: [sp,#0]=hash [sp,#8]=obj [sp,#16]=cursor [sp,#24]=key_ptr
    //        [sp,#32]=key_len [sp,#40]=value mixed [sp,#48]=val_hi
    //        [sp,#56]=val_tag [sp,#64]=fp [sp,#72]=lr
    emitter.instruction("sub sp, sp, #80");                                     // reserve the iteration frame and saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #64");                                    // set the new frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the source hash pointer
    emitter.instruction("bl __rt_stdclass_new");                                // x0 = fresh empty stdClass
    emitter.instruction("str x0, [sp, #8]");                                    // save the destination object pointer
    emitter.instruction("str xzr, [sp, #16]");                                  // start hash iteration at cursor 0
    emitter.label("__rt_object_from_hash_loop");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the source hash pointer
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload the iteration cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // x0=cursor x1=key ptr x2=key len x3..x5=value
    emitter.instruction("cmn x0, #1");                                          // did iteration reach the end sentinel (-1)?
    emitter.instruction("b.eq __rt_object_from_hash_done");                     // stop once every entry is consumed
    emitter.instruction("str x0, [sp, #16]");                                   // save the next iteration cursor
    emitter.instruction("str x1, [sp, #24]");                                   // save the entry key pointer
    emitter.instruction("str x2, [sp, #32]");                                   // save the entry key length (-1 marks an int key)
    emitter.instruction("str x3, [sp, #40]");                                   // save the entry value low word
    emitter.instruction("str x4, [sp, #48]");                                   // save the entry value high word
    emitter.instruction("str x5, [sp, #56]");                                   // save the entry value runtime tag
    // -- box the entry value into an owned Mixed property cell --
    emitter.instruction("cmp x5, #7");                                          // is the entry already a boxed Mixed cell?
    emitter.instruction("b.eq __rt_object_from_hash_reuse");                    // reuse already-boxed values directly
    emitter.instruction("mov x0, x5");                                          // pass the runtime tag to the boxing helper
    emitter.instruction("mov x1, x3");                                          // pass the value low word to the boxing helper
    emitter.instruction("mov x2, x4");                                          // pass the value high word to the boxing helper
    emitter.instruction("bl __rt_mixed_from_value");                            // x0 = freshly owned Mixed cell (retains payloads)
    emitter.instruction("str x0, [sp, #40]");                                   // stash the property Mixed pointer
    emitter.instruction("b __rt_object_from_hash_key");                         // continue with key normalization
    emitter.label("__rt_object_from_hash_reuse");
    emitter.instruction("ldr x0, [sp, #40]");                                   // reload the inner boxed Mixed pointer
    emitter.instruction("bl __rt_incref");                                      // retain it for the new property owner
    emitter.label("__rt_object_from_hash_key");
    // -- normalize integer keys to decimal-string property names --
    emitter.instruction("ldr x2, [sp, #32]");                                   // reload the entry key length
    emitter.instruction("cmn x2, #1");                                          // is this an integer key (length -1)?
    emitter.instruction("b.ne __rt_object_from_hash_set");                      // string keys are used unchanged
    emitter.instruction("ldr x0, [sp, #24]");                                   // integer keys store the value in the key pointer slot
    emitter.instruction("bl __rt_itoa");                                        // x1=decimal digits ptr, x2=digit length
    emitter.instruction("str x1, [sp, #24]");                                   // overwrite the key pointer with the decimal string
    emitter.instruction("str x2, [sp, #32]");                                   // overwrite the key length with the decimal length
    emitter.label("__rt_object_from_hash_set");
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the destination object
    emitter.instruction("ldr x1, [sp, #24]");                                   // reload the property name pointer
    emitter.instruction("ldr x2, [sp, #32]");                                   // reload the property name length
    emitter.instruction("ldr x3, [sp, #40]");                                   // reload the property value Mixed pointer
    emitter.instruction("bl __rt_stdclass_set");                                // store the property (moves the Mixed into the hash)
    emitter.instruction("b __rt_object_from_hash_loop");                        // process the next entry
    emitter.label("__rt_object_from_hash_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // return the populated object
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the stdClass pointer in x0
}

/// x86_64 implementation of `__rt_object_from_hash` (input `rdi`, result `rax`).
fn emit_object_from_hash_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: object_from_hash ---");
    emitter.label_global("__rt_object_from_hash");

    // Frame: [rbp-8]=hash [rbp-16]=obj [rbp-24]=cursor [rbp-32]=key_ptr
    //        [rbp-40]=key_len [rbp-48]=value mixed [rbp-56]=val_hi [rbp-64]=val_tag
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 64");                                         // reserve the iteration spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the source hash pointer
    emitter.instruction("call __rt_stdclass_new");                              // rax = fresh empty stdClass
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the destination object pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // start hash iteration at cursor 0
    emitter.label("__rt_object_from_hash_loop");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the source hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // reload the iteration cursor
    emitter.instruction("call __rt_hash_iter_next");                            // rax=cursor rdi/rdx=key rcx/r8/r9=value
    emitter.instruction("cmp rax, -1");                                         // did iteration reach the end sentinel?
    emitter.instruction("je __rt_object_from_hash_done");                       // stop once every entry is consumed
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the next iteration cursor
    emitter.instruction("mov QWORD PTR [rbp - 32], rdi");                       // save the entry key pointer
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // save the entry key length
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // save the entry value low word
    emitter.instruction("mov QWORD PTR [rbp - 56], r8");                        // save the entry value high word
    emitter.instruction("mov QWORD PTR [rbp - 64], r9");                        // save the entry value runtime tag
    // -- box the entry value into an owned Mixed property cell --
    emitter.instruction("cmp r9, 7");                                           // is the entry already a boxed Mixed cell?
    emitter.instruction("je __rt_object_from_hash_reuse");                      // reuse already-boxed values directly
    emitter.instruction("mov rax, r9");                                         // pass the runtime tag to the boxing helper
    emitter.instruction("mov rdi, rcx");                                        // pass the value low word to the boxing helper
    emitter.instruction("mov rsi, r8");                                         // pass the value high word to the boxing helper
    emitter.instruction("call __rt_mixed_from_value");                          // rax = freshly owned Mixed cell
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // stash the property Mixed pointer
    emitter.instruction("jmp __rt_object_from_hash_key");                       // continue with key normalization
    emitter.label("__rt_object_from_hash_reuse");
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // reload the inner boxed Mixed pointer
    emitter.instruction("call __rt_incref");                                    // retain it for the new property owner
    emitter.label("__rt_object_from_hash_key");
    // -- normalize integer keys to decimal-string property names --
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // reload the entry key length
    emitter.instruction("cmp rdx, -1");                                         // is this an integer key?
    emitter.instruction("jne __rt_object_from_hash_set");                       // string keys are used unchanged
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // integer keys store the value in the key slot
    emitter.instruction("call __rt_itoa");                                      // rax=decimal digits ptr, rdx=digit length
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // overwrite the key pointer with the string
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // overwrite the key length with the length
    emitter.label("__rt_object_from_hash_set");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the destination object
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // reload the property name pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // reload the property name length
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the property value Mixed pointer
    emitter.instruction("call __rt_stdclass_set");                              // store the property into the object hash
    emitter.instruction("jmp __rt_object_from_hash_loop");                      // process the next entry
    emitter.label("__rt_object_from_hash_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return the populated object
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the stdClass pointer in rax
}
