//! Purpose:
//! Emits `__rt_array_fetch_dim_w`: fetch an indexed-array element that is itself an indexed array
//! for writing (`$m[$i][$j] = $v`), separating the parent and the child only when they are shared.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - Generated code for `Op::ArrayFetchDimW` (`crate::codegen::lower_inst::arrays`).
//!
//! Key details:
//! - The parent is made unique first (`__rt_array_ensure_unique`), so its reference to the child
//!   is the only one it holds; a shared child is then split the same way and the separated copy is
//!   stored straight into the parent slot (the split transfers the parent's reference to the copy,
//!   so the raw store is balanced).
//! - The child is returned borrowed: the caller writes into it in place and stores the (possibly
//!   relocated by growth) pointer back with a raw element store, never touching refcounts.
//! - A missing index, a non-pointer-sized payload, or an element that is not an indexed array
//!   returns 0 so generated code takes the generic nested-write path instead.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_array_fetch_dim_w`.
/// Input:  x0=parent indexed array, x1=index (x86_64: rdi, rsi)
/// Output: x0=child indexed array or 0, x1=parent after copy-on-write separation (x86_64: rax, rdx)
pub fn emit_array_fetch_dim_w(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_fetch_dim_w_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_fetch_dim_w ---");
    emitter.label_global("__rt_array_fetch_dim_w");
    emitter.instruction("sub sp, sp, #48");                                     // reserve slots for parent, index, slot address and child
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // set up the helper frame
    emitter.instruction("str x1, [sp, #8]");                                    // keep the index across helper calls
    emitter.instruction("str x0, [sp, #0]");                                    // keep the parent in case it is not a live array
    emitter.instruction("cbz x0, 90f");                                         // a null parent has nothing to fetch
    crate::codegen_support::abi::emit_load_int_immediate(
        emitter,
        "x9",
        crate::codegen_support::sentinels::NULL_SENTINEL,
    );
    emitter.instruction("cmp x0, x9");                                          // the in-band null-container sentinel?
    emitter.instruction("b.eq 90f");                                            // generic path handles writes into null
    emitter.instruction("bl __rt_array_ensure_unique");                         // split a shared parent before handing out its element
    emitter.instruction("str x0, [sp, #0]");                                    // keep the unique parent
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload the index
    emitter.instruction("ldr x9, [x0]");                                        // parent logical length
    emitter.instruction("cmp x1, x9");                                          // index within [0, length)? (unsigned: negative fails too)
    emitter.instruction("b.hs 90f");                                            // missing element: generic path
    emitter.instruction("ldr x9, [x0, #16]");                                   // parent element size
    emitter.instruction("cmp x9, #8");                                          // pointer-sized slots?
    emitter.instruction("b.ne 90f");                                            // other layouts are generic
    emitter.instruction("add x10, x0, #24");                                    // parent payload base
    emitter.instruction("add x10, x10, x1, lsl #3");                            // address of the element slot
    emitter.instruction("str x10, [sp, #16]");                                  // keep the slot address
    emitter.instruction("ldr x0, [x10]");                                       // the child pointer
    emitter.instruction("str x0, [sp, #24]");                                   // keep the child across the kind probe
    emitter.instruction("bl __rt_heap_kind");                                   // x0 = heap kind of the child (0 for non-heap values)
    emitter.instruction("cmp x0, #2");                                          // an indexed array?
    emitter.instruction("b.ne 90f");                                            // hashes, boxes and scalars are generic
    emitter.instruction("ldr x0, [sp, #24]");                                   // reload the child
    emitter.instruction("ldr w9, [x0, #-12]");                                  // child refcount
    emitter.instruction("cmp w9, #1");                                          // owned by the parent alone?
    emitter.instruction("b.ls 80f");                                            // yes: write in place
    emitter.instruction("bl __rt_array_ensure_unique");                         // split the shared child (moves the parent's reference to the copy)
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the slot address
    emitter.instruction("str x0, [x10]");                                       // the parent now points at its private copy
    emitter.label("80");
    emitter.instruction("ldr x1, [sp, #0]");                                    // return the unique parent
    emitter.instruction("b 99f");                                               // return the child
    emitter.label("90");
    emitter.instruction("mov x0, #0");                                          // not fetchable: caller takes the generic path
    emitter.instruction("ldr x1, [sp, #0]");                                    // return the (possibly separated) parent
    emitter.label("99");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return child and parent
}

/// Emits the x86_64 variant of `__rt_array_fetch_dim_w` (rdi, rsi → rax, rdx).
fn emit_array_fetch_dim_w_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_fetch_dim_w ---");
    emitter.label_global("__rt_array_fetch_dim_w");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("sub rsp, 32");                                         // reserve slots for parent, index, slot address and child
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // keep the index across helper calls
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // keep the parent in case it is not a live array
    emitter.instruction("test rdi, rdi");                                       // a null parent has nothing to fetch
    emitter.instruction("jz 90f");                                              // generic path
    crate::codegen_support::abi::emit_load_int_immediate(
        emitter,
        "r9",
        crate::codegen_support::sentinels::NULL_SENTINEL,
    );
    emitter.instruction("cmp rdi, r9");                                         // the in-band null-container sentinel?
    emitter.instruction("je 90f");                                              // generic path handles writes into null
    emitter.instruction("call __rt_array_ensure_unique");                       // split a shared parent before handing out its element
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // keep the unique parent
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // reload the index
    emitter.instruction("cmp rsi, QWORD PTR [rax]");                            // index within [0, length)? (unsigned: negative fails too)
    emitter.instruction("jae 90f");                                             // missing element: generic path
    emitter.instruction("cmp QWORD PTR [rax + 16], 8");                         // pointer-sized slots?
    emitter.instruction("jne 90f");                                             // other layouts are generic
    emitter.instruction("lea r10, [rax + rsi * 8 + 24]");                       // address of the element slot
    emitter.instruction("mov QWORD PTR [rbp - 24], r10");                       // keep the slot address
    emitter.instruction("mov rax, QWORD PTR [r10]");                            // the child pointer
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // keep the child across the kind probe
    emitter.instruction("call __rt_heap_kind");                                 // eax = heap kind of the child (0 for non-heap values)
    emitter.instruction("cmp eax, 2");                                          // an indexed array?
    emitter.instruction("jne 90f");                                             // hashes, boxes and scalars are generic
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // reload the child
    emitter.instruction("cmp DWORD PTR [rax - 12], 1");                         // owned by the parent alone?
    emitter.instruction("jbe 80f");                                             // yes: write in place
    emitter.instruction("mov rdi, rax");                                        // split the shared child
    emitter.instruction("call __rt_array_ensure_unique");                       // moves the parent's reference to the private copy
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the slot address
    emitter.instruction("mov QWORD PTR [r10], rax");                            // the parent now points at its private copy
    emitter.label("80");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // return the unique parent
    emitter.instruction("leave");                                               // release the helper frame
    emitter.instruction("ret");                                                 // return the child in rax
    emitter.label("90");
    emitter.instruction("xor eax, eax");                                        // not fetchable: caller takes the generic path
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // return the (possibly separated) parent
    emitter.instruction("leave");                                               // release the helper frame
    emitter.instruction("ret");                                                 // return
}
