//! Purpose:
//! Emits the `__rt_str_append` runtime helper behind `$s .= $x` on plain string locals.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//! - `crate::codegen::lower_inst` (`str_append_local`).
//!
//! Key details:
//! - The destination is the string a local slot owns. When it is a heap string (kind 1) whose
//!   only reference is that slot and whose block capacity (`[ptr-16]`) fits the new length,
//!   the appended bytes are written right after the current length: no copy of the prefix.
//!   Holders of the old `(ptr, len)` pair still see their original bytes, because only bytes
//!   past `len` change.
//! - Otherwise a new heap string with doubled capacity (min 32 bytes) receives both parts and
//!   the old string is released through `__rt_decref_any`, so repeated appends are amortized
//!   O(1) per byte instead of O(n) per append.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_str_append`.
///
/// - AArch64: x1/x2 = current string, x3/x4 = appended string → x1/x2 = owned result.
/// - x86_64: rdi/rsi = current string, rdx/rcx = appended string → rax/rdx = owned result.
pub fn emit_str_append(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_str_append_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: str_append ---");
    emitter.label_global("__rt_str_append");
    emitter.instruction("sub sp, sp, #64");                                     // frame: current, appended, new length, new pointer
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish new frame pointer
    emitter.instruction("stp x1, x2, [sp, #0]");                                // save current string
    emitter.instruction("stp x3, x4, [sp, #16]");                               // save appended string
    emitter.instruction("add x5, x2, x4");                                      // new length
    emitter.instruction("str x5, [sp, #32]");                                   // save new length

    // -- in place only for a uniquely owned heap string with enough capacity --
    emitter.instruction("cbz x1, __rt_str_append_grow");                        // no current payload
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_buf");
    emitter.instruction("cmp x1, x9");                                          // below the managed heap?
    emitter.instruction("b.lo __rt_str_append_grow");                           // literal or static data: copy
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_heap_off");
    emitter.instruction("ldr x10, [x10]");                                      // heap bump offset
    emitter.instruction("add x10, x9, x10");                                    // heap end
    emitter.instruction("cmp x1, x10");                                         // beyond the managed heap?
    emitter.instruction("b.hs __rt_str_append_grow");                           // foreign pointer: copy
    emitter.instruction("ldr x11, [x1, #-8]");                                  // kind word
    emitter.instruction("and x11, x11, #0xff");                                 // heap kind tag
    emitter.instruction("cmp x11, #1");                                         // persisted string?
    emitter.instruction("b.ne __rt_str_append_grow");                           // not a string block: copy
    emitter.instruction("ldr w12, [x1, #-12]");                                 // refcount
    emitter.instruction("cmp w12, #1");                                         // only the local owns it?
    emitter.instruction("b.ne __rt_str_append_grow");                           // shared: copy-on-write
    emitter.instruction("ldr w13, [x1, #-16]");                                 // block capacity
    emitter.instruction("cmp x5, x13");                                         // does the new length fit?
    emitter.instruction("b.hi __rt_str_append_grow");                           // no: reallocate
    emitter.instruction("add x0, x1, x2");                                      // write position = current end
    emitter.instruction("mov x6, x3");                                          // source = appended bytes
    emitter.instruction("mov x7, x4");                                          // count = appended length
    emitter.instruction("bl __rt_str_append_copy");                             // append in place
    emitter.instruction("ldr x1, [sp, #0]");                                    // same pointer
    emitter.instruction("ldr x2, [sp, #32]");                                   // new length
    emitter.instruction("b __rt_str_append_ret");                               // done

    // -- reallocate with doubled capacity, copy both parts, release the old string --
    emitter.label("__rt_str_append_grow");
    emitter.instruction("lsl x0, x5, #1");                                      // capacity = 2 * new length
    emitter.instruction("mov x9, #32");                                         // minimum capacity
    emitter.instruction("cmp x0, x9");                                          // below the minimum?
    emitter.instruction("csel x0, x9, x0, lo");                                 // clamp to 32 bytes
    emitter.instruction("bl __rt_heap_alloc");                                  // x0 = new block
    emitter.instruction("mov x9, #1");                                          // heap kind 1 = persisted string
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the string kind
    emitter.instruction("str x0, [sp, #40]");                                   // save the new pointer
    emitter.instruction("ldp x6, x7, [sp, #0]");                                // source = current string
    emitter.instruction("bl __rt_str_append_copy");                             // copy the prefix
    emitter.instruction("ldp x6, x7, [sp, #16]");                               // source = appended string
    emitter.instruction("bl __rt_str_append_copy");                             // x0 already points past the prefix
    emitter.instruction("ldr x0, [sp, #0]");                                    // old string
    emitter.instruction("bl __rt_decref_any");                                  // release the slot's old reference
    emitter.instruction("ldr x1, [sp, #40]");                                   // new pointer
    emitter.instruction("ldr x2, [sp, #32]");                                   // new length

    emitter.label("__rt_str_append_ret");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate frame
    emitter.instruction("ret");                                                 // return the owned string

    // -- leaf copy: x0 = destination (advanced), x6 = source, x7 = byte count --
    emitter.label("__rt_str_append_copy");
    emitter.label("__rt_str_append_copy_words");
    emitter.instruction("cmp x7, #8");                                          // at least one word left?
    emitter.instruction("b.lo __rt_str_append_copy_bytes");                     // no: finish bytewise
    emitter.instruction("ldr x8, [x6], #8");                                    // load 8 bytes
    emitter.instruction("str x8, [x0], #8");                                    // store 8 bytes
    emitter.instruction("sub x7, x7, #8");                                      // 8 fewer bytes
    emitter.instruction("b __rt_str_append_copy_words");                        // continue
    emitter.label("__rt_str_append_copy_bytes");
    emitter.instruction("cbz x7, __rt_str_append_copy_done");                   // all copied
    emitter.instruction("ldrb w8, [x6], #1");                                   // load a byte
    emitter.instruction("strb w8, [x0], #1");                                   // store a byte
    emitter.instruction("sub x7, x7, #1");                                      // one fewer byte
    emitter.instruction("b __rt_str_append_copy_bytes");                        // continue
    emitter.label("__rt_str_append_copy_done");
    emitter.instruction("ret");                                                 // return to the helper
}

/// Emits the x86_64 `__rt_str_append` (see [`emit_str_append`]).
fn emit_str_append_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: str_append ---");
    emitter.label_global("__rt_str_append");
    emitter.instruction("push rbp");                                            // frame for the helper calls
    emitter.instruction("mov rbp, rsp");                                        // stable frame base
    emitter.instruction("sub rsp, 48");                                         // current, appended, new length, new pointer
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // current pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // current length
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // appended pointer
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // appended length
    emitter.instruction("lea rax, [rsi + rcx]");                                // new length
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save new length

    // -- in place only for a uniquely owned heap string with enough capacity --
    emitter.instruction("test rdi, rdi");                                       // no current payload?
    emitter.instruction("jz __rt_str_append_grow");                             // copy
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_buf");
    emitter.instruction("cmp rdi, r8");                                         // below the managed heap?
    emitter.instruction("jb __rt_str_append_grow");                             // literal or static data: copy
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r9", "_heap_off", 0);
    emitter.instruction("add r9, r8");                                          // heap end
    emitter.instruction("cmp rdi, r9");                                         // beyond the managed heap?
    emitter.instruction("jae __rt_str_append_grow");                            // foreign pointer: copy
    emitter.instruction("mov r10, QWORD PTR [rdi - 8]");                        // kind word
    emitter.instruction("shr r10, 32");                                         // high-word heap marker
    emitter.instruction(&format!("cmp r10d, 0x{:x}", crate::codegen_support::sentinels::X86_64_HEAP_MAGIC_HI32)); // elephc-owned block?
    emitter.instruction("jne __rt_str_append_grow");                            // foreign block: copy
    emitter.instruction("movzx r10d, BYTE PTR [rdi - 8]");                      // heap kind tag
    emitter.instruction("cmp r10d, 1");                                         // persisted string?
    emitter.instruction("jne __rt_str_append_grow");                            // not a string block: copy
    emitter.instruction("cmp DWORD PTR [rdi - 12], 1");                         // only the local owns it?
    emitter.instruction("jne __rt_str_append_grow");                            // shared: copy-on-write
    emitter.instruction("mov r10d, DWORD PTR [rdi - 16]");                      // block capacity
    emitter.instruction("cmp rax, r10");                                        // does the new length fit?
    emitter.instruction("ja __rt_str_append_grow");                             // no: reallocate
    emitter.instruction("add rdi, rsi");                                        // write position = current end
    emitter.instruction("mov rsi, rdx");                                        // source = appended bytes
    emitter.instruction("rep movsb");                                           // append in place (rcx = appended length)
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // same pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // new length
    emitter.instruction("jmp __rt_str_append_ret");                             // done

    // -- reallocate with doubled capacity, copy both parts, release the old string --
    emitter.label("__rt_str_append_grow");
    emitter.instruction("add rax, rax");                                        // capacity = 2 * new length
    emitter.instruction("mov r8, 32");                                          // minimum capacity
    emitter.instruction("cmp rax, r8");                                         // below the minimum?
    emitter.instruction("cmovb rax, r8");                                       // clamp to 32 bytes
    emitter.instruction("call __rt_heap_alloc");                                // rax = new block
    emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(1))); // persisted-string kind word
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the string kind
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // save the new pointer
    emitter.instruction("mov rdi, rax");                                        // destination
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // source = current string
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // prefix length
    emitter.instruction("rep movsb");                                           // copy the prefix (rdi advances)
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // source = appended string
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // appended length
    emitter.instruction("rep movsb");                                           // copy the appended bytes
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // old string
    emitter.instruction("call __rt_decref_any");                                // release the slot's old reference
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // new pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // new length

    emitter.label("__rt_str_append_ret");
    emitter.instruction("add rsp, 48");                                         // release the frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the owned string
}
