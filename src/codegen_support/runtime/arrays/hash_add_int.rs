//! Purpose:
//! Emits the `__rt_hash_add_int` runtime helper: the in-place fast path for the PHP counter
//! idiom `$h[$k] = $h[$k] + $n` (and `$h[$k] += $n`, `$h[$k]++`) on associative arrays.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - Generated code for `Op::HashAddInt` (`crate::codegen::lower_inst::hashes`).
//!
//! Key details:
//! - The helper only handles the case where the update is observably identical to a plain
//!   integer add: the key exists, the hash is not shared (refcount <= 1, the same test as
//!   `__rt_hash_ensure_unique`), and the entry holds an integer either inline (tag 0) or in an
//!   unshared Mixed box (tag 7, box refcount 1, inner tag 0), and the sum does not overflow.
//! - Every other case (missing key, shared hash, reference element, non-int value, overflow to
//!   float) returns 0 without touching memory so generated code runs the generic PHP path.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_hash_add_int`.
/// Input:  x0=hash_table_ptr, x1=key_lo, x2=key_hi (-1 means integer key), x3=delta
///         (x86_64: rdi, rsi, rdx, rcx)
/// Output: x0=1 when the entry was updated in place, 0 when the caller must take the generic path
///         (x86_64: rax)
pub fn emit_hash_add_int(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_hash_add_int_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hash_add_int ---");
    emitter.label_global("__rt_hash_add_int");
    emitter.instruction("sub sp, sp, #32");                                     // reserve slots for the hash pointer and the delta
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // set up the helper frame
    emitter.instruction("str x0, [sp, #0]");                                    // keep the hash pointer for the sharing check
    emitter.instruction("str x3, [sp, #8]");                                    // keep the delta across the lookup
    emitter.instruction("bl __rt_hash_get");                                    // x0=found, x3=value tag, x4=entry address
    emitter.instruction("cbz x0, 90f");                                         // missing key (or null table): generic path warns and inserts
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the hash pointer
    emitter.instruction("ldr w10, [x9, #-12]");                                 // hash refcount from the uniform heap header
    emitter.instruction("cmp w10, #1");                                         // is the table shared with another owner?
    emitter.instruction("b.hi 90f");                                            // shared tables are split by the generic copy-on-write path
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload the delta
    emitter.instruction("cbz x3, 10f");                                         // tag 0: the integer is stored inline in the entry
    emitter.instruction("cmp x3, #7");                                          // tag 7: a boxed Mixed value
    emitter.instruction("b.ne 90f");                                            // any other tag (float, string, reference, ...) is generic
    emitter.instruction("ldr x12, [x4, #24]");                                  // the Mixed box pointer
    emitter.instruction("cbz x12, 90f");                                        // defensive: a null box is generic
    emitter.instruction("ldr w10, [x12, #-12]");                                // box refcount
    emitter.instruction("cmp w10, #1");                                         // only a box owned solely by this entry may change in place
    emitter.instruction("b.ne 90f");                                            // shared boxes are replaced by the generic path
    emitter.instruction("ldr x13, [x12]");                                      // the boxed runtime tag
    emitter.instruction("cbnz x13, 90f");                                       // only boxed integers take the fast path
    emitter.instruction("ldr x14, [x12, #8]");                                  // the boxed integer
    emitter.instruction("adds x14, x14, x11");                                  // add the delta, setting the overflow flag
    emitter.instruction("b.vs 90f");                                            // overflow promotes to float on the generic path
    emitter.instruction("str x14, [x12, #8]");                                  // store the sum back into the box
    emitter.instruction("b 80f");                                               // report success
    emitter.label("10");
    emitter.instruction("ldr x14, [x4, #24]");                                  // the inline integer
    emitter.instruction("adds x14, x14, x11");                                  // add the delta, setting the overflow flag
    emitter.instruction("b.vs 90f");                                            // overflow promotes to float on the generic path
    emitter.instruction("str x14, [x4, #24]");                                  // store the sum back into the entry
    emitter.label("80");
    emitter.instruction("mov x0, #1");                                          // updated in place
    emitter.instruction("b 99f");                                               // return
    emitter.label("90");
    emitter.instruction("mov x0, #0");                                          // caller takes the generic path
    emitter.label("99");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return the status
}

/// Emits the x86_64 variant of `__rt_hash_add_int` (SysV: rdi, rsi, rdx, rcx → rax).
fn emit_hash_add_int_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_add_int ---");
    emitter.label_global("__rt_hash_add_int");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("sub rsp, 16");                                         // reserve slots for the hash pointer and the delta
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // keep the hash pointer for the sharing check
    emitter.instruction("mov QWORD PTR [rbp - 16], rcx");                       // keep the delta across the lookup
    emitter.instruction("call __rt_hash_get");                                  // rax=found, rcx=value tag, r8=entry address
    emitter.instruction("test rax, rax");                                       // was the key found?
    emitter.instruction("jz 90f");                                              // missing key (or null table): generic path warns and inserts
    emitter.instruction("mov r9, QWORD PTR [rbp - 8]");                         // reload the hash pointer
    emitter.instruction("cmp DWORD PTR [r9 - 12], 1");                          // is the table shared with another owner?
    emitter.instruction("ja 90f");                                              // shared tables are split by the generic copy-on-write path
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                       // reload the delta
    emitter.instruction("test rcx, rcx");                                       // tag 0: the integer is stored inline in the entry
    emitter.instruction("jz 10f");                                              // update the inline integer
    emitter.instruction("cmp rcx, 7");                                          // tag 7: a boxed Mixed value
    emitter.instruction("jne 90f");                                             // any other tag (float, string, reference, ...) is generic
    emitter.instruction("mov r10, QWORD PTR [r8 + 24]");                        // the Mixed box pointer
    emitter.instruction("test r10, r10");                                       // defensive: a null box is generic
    emitter.instruction("jz 90f");                                              // take the generic path
    emitter.instruction("cmp DWORD PTR [r10 - 12], 1");                         // only a box owned solely by this entry may change in place
    emitter.instruction("jne 90f");                                             // shared boxes are replaced by the generic path
    emitter.instruction("cmp QWORD PTR [r10], 0");                              // only boxed integers take the fast path
    emitter.instruction("jne 90f");                                             // other boxed payloads are generic
    emitter.instruction("mov rax, QWORD PTR [r10 + 8]");                        // the boxed integer
    emitter.instruction("add rax, r11");                                        // add the delta, setting the overflow flag
    emitter.instruction("jo 90f");                                              // overflow promotes to float on the generic path
    emitter.instruction("mov QWORD PTR [r10 + 8], rax");                        // store the sum back into the box
    emitter.instruction("jmp 80f");                                             // report success
    emitter.label("10");
    emitter.instruction("mov rax, QWORD PTR [r8 + 24]");                        // the inline integer
    emitter.instruction("add rax, r11");                                        // add the delta, setting the overflow flag
    emitter.instruction("jo 90f");                                              // overflow promotes to float on the generic path
    emitter.instruction("mov QWORD PTR [r8 + 24], rax");                        // store the sum back into the entry
    emitter.label("80");
    emitter.instruction("mov eax, 1");                                          // updated in place
    emitter.instruction("jmp 99f");                                             // return
    emitter.label("90");
    emitter.instruction("xor eax, eax");                                        // caller takes the generic path
    emitter.label("99");
    emitter.instruction("leave");                                               // release the helper frame
    emitter.instruction("ret");                                                 // return the status
}
