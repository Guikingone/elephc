//! Purpose:
//! Emits the `__rt_concat`, `__rt_concat_cl` runtime helper assembly for runtime string concatenation.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.
//! - A result that would not fit in the concat scratch buffer is built in an owned heap string
//!   instead (like `str_repeat`), so concatenation never writes past `_concat_buf`.

use crate::codegen_support::runtime::data::CONCAT_BUF_SIZE;
use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits the `__rt_concat` runtime helper for concatenating two byte-strings.
/// Writes the concatenated result into the global `_concat_buf` at the current `_concat_off`
/// offset, then advances `_concat_off` by the total bytes written.
/// Dispatches to `emit_concat_linux_x86_64` on x86_64; uses the ARM64 path otherwise.
///
/// Input:  x1=left_ptr, x2=left_len, x3=right_ptr, x4=right_len
/// Output: x1=result_ptr (start of written region in _concat_buf), x2=result_len
pub fn emit_concat(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_concat_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: concat ---");
    emitter.label_global("__rt_concat");

    // -- set up stack frame (64 bytes) --
    emitter.instruction("sub sp, sp, #64");                                     // allocate 64 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish new frame pointer

    // -- save input arguments to stack --
    emitter.instruction("stp x1, x2, [sp, #0]");                                // save left string ptr and length
    emitter.instruction("stp x3, x4, [sp, #16]");                               // save right string ptr and length
    emitter.instruction("add x5, x2, x4");                                      // compute total result length
    emitter.instruction("str x5, [sp, #32]");                                   // save total length on stack

    // -- choose the destination: concat scratch when the result fits, else an owned heap string --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x6", "_concat_off");
    emitter.instruction("ldr x8, [x6]");                                        // load current write offset
    emitter.instruction("add x9, x8, x5");                                      // scratch end offset after this append
    emitter.instruction(&format!("mov x12, #{}", CONCAT_BUF_SIZE));              // concat scratch capacity
    emitter.instruction("cmp x9, x12");                                         // does the result fit in concat scratch?
    emitter.instruction("b.hi __rt_concat_heap");                               // no: never write past the scratch buffer
    crate::codegen_support::abi::emit_symbol_address(emitter, "x7", "_concat_buf");
    emitter.instruction("add x9, x7, x8");                                      // compute destination pointer: buf + offset
    emitter.instruction("str x9, [sp, #40]");                                   // save result start pointer on stack
    emitter.instruction("mov x13, #0");                                         // scratch-backed result
    emitter.instruction("b __rt_concat_copy");                                  // copy both operands
    emitter.label("__rt_concat_heap");
    emitter.instruction("mov x0, x5");                                          // payload size for the heap allocator
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate an owned string payload
    emitter.instruction("mov x6, #1");                                          // heap kind 1 = owned elephc string
    emitter.instruction("str x6, [x0, #-8]");                                   // stamp the heap allocation as a string payload
    emitter.instruction("mov x9, x0");                                          // destination is the heap payload
    emitter.instruction("str x9, [sp, #40]");                                   // save result start pointer on stack
    emitter.instruction("mov x13, #1");                                         // heap-backed result: the scratch offset stays unchanged

    emitter.label("__rt_concat_copy");
    emitter.instruction("mov x10, x9");                                         // set dest cursor to start of output
    emitter.instruction("ldp x1, x2, [sp, #0]");                                // reload left ptr and length
    emit_copy_aarch64(emitter, "x1", "x2", "__rt_concat_cl");
    emitter.instruction("ldp x3, x4, [sp, #16]");                               // reload right ptr and length
    emit_copy_aarch64(emitter, "x3", "x4", "__rt_concat_cr");

    // -- publish the new scratch offset for scratch-backed results --
    emitter.label("__rt_concat_done");
    emitter.instruction("cbnz x13, __rt_concat_return");                        // heap results leave the scratch offset alone
    emitter.instruction("ldr x5, [sp, #32]");                                   // reload total result length
    crate::codegen_support::abi::emit_symbol_address(emitter, "x6", "_concat_off");
    emitter.instruction("ldr x8, [x6]");                                        // load current offset
    emitter.instruction("add x8, x8, x5");                                      // advance offset by total length written
    emitter.instruction("str x8, [x6]");                                        // store updated offset
    emitter.label("__rt_concat_return");

    // -- set return values and restore frame --
    emitter.instruction("ldr x1, [sp, #40]");                                   // return result pointer (start of output)
    emitter.instruction("ldr x2, [sp, #32]");                                   // return result length
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the x86_64 Linux variant of `__rt_concat`.
/// Uses the System V AMD64 ABI: left string in rax/rdx, right string in rdi/rsi.
/// Result returned in rax (pointer) and rdx (length).
/// The global `_concat_buf` / `_concat_off` machinery is shared with the ARM64 path.
fn emit_concat_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: concat ---");
    emitter.label_global("__rt_concat");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while concat uses stack locals
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for concat bookkeeping
    emitter.instruction("sub rsp, 64");                                         // reserve local slots for input strings, total length, result pointer, and storage kind

    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save left string pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save left string length
    emitter.instruction("mov QWORD PTR [rbp - 24], rdi");                       // save right string pointer
    emitter.instruction("mov QWORD PTR [rbp - 32], rsi");                       // save right string length
    emitter.instruction("mov r8, rdx");                                         // seed total length from the left string length
    emitter.instruction("add r8, rsi");                                         // total length = left length + right length
    emitter.instruction("mov QWORD PTR [rbp - 40], r8");                        // save total length for offset update and return

    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_concat_off");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // load current concat write offset
    emitter.instruction("lea r11, [r9 + rdx]");                                 // scratch end offset ...
    emitter.instruction("add r11, rsi");                                        // ... after this append
    emitter.instruction(&format!("cmp r11, {}", CONCAT_BUF_SIZE));               // does the result fit in concat scratch?
    emitter.instruction("ja __rt_concat_heap_linux_x86_64");                    // no: never write past the scratch buffer
    crate::codegen_support::abi::emit_symbol_address(emitter, "r10", "_concat_buf");
    emitter.instruction("lea r10, [r10 + r9]");                                 // compute destination pointer: concat_buf + offset
    emitter.instruction("mov QWORD PTR [rbp - 48], r10");                       // save result start pointer for return
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // scratch-backed result
    emitter.instruction("jmp __rt_concat_copy_linux_x86_64");                   // copy both operands
    emitter.label("__rt_concat_heap_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // payload size for the heap allocator
    emitter.instruction("call __rt_heap_alloc");                                // allocate an owned string payload
    emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(1))); // owned-string heap kind word
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the heap allocation as a string payload
    emitter.instruction("mov r10, rax");                                        // destination is the heap payload
    emitter.instruction("mov QWORD PTR [rbp - 48], r10");                       // save result start pointer for return
    emitter.instruction("mov QWORD PTR [rbp - 56], 1");                         // heap-backed result: the scratch offset stays unchanged

    emitter.label("__rt_concat_copy_linux_x86_64");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // load left source pointer
    emitter.instruction("mov r9, QWORD PTR [rbp - 16]");                        // load remaining left byte count
    emit_copy_x86_64(emitter, "__rt_concat_cl");
    emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                        // load right source pointer
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // load remaining right byte count
    emit_copy_x86_64(emitter, "__rt_concat_cr");

    emitter.label("__rt_concat_done");
    emitter.instruction("cmp QWORD PTR [rbp - 56], 0");                         // heap results leave the scratch offset alone
    emitter.instruction("jne __rt_concat_return_linux_x86_64");                 // skip the offset update
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_concat_off");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // reload current concat write offset
    emitter.instruction("add r9, QWORD PTR [rbp - 40]");                        // advance offset by total bytes written
    emitter.instruction("mov QWORD PTR [r8], r9");                              // store updated concat write offset
    emitter.label("__rt_concat_return_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // return result pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // return result length
    emitter.instruction("add rsp, 64");                                         // release concat local slots
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return concatenated string in rax/rdx
}

/// Copies `len` bytes from `src` to the destination cursor x10, eight bytes at a time and then
/// byte by byte; `src`, `len`, x10 and x11 are clobbered.
fn emit_copy_aarch64(emitter: &mut Emitter, src: &str, len: &str, label: &str) {
    emitter.label(&format!("{}_words", label));
    emitter.instruction(&format!("cmp {}, #8", len));                         // at least one whole word left?
    emitter.instruction(&format!("b.lo {}", label));                          // no: finish byte by byte
    emitter.instruction(&format!("ldr x11, [{}], #8", src));                  // load eight source bytes
    emitter.instruction("str x11, [x10], #8");                                  // store them at the destination cursor
    emitter.instruction(&format!("sub {}, {}, #8", len, len));                // eight bytes fewer
    emitter.instruction(&format!("b {}_words", label));                       // continue with the next word
    emitter.label(label);
    emitter.instruction(&format!("cbz {}, {}_end", len, label));              // every byte copied
    emitter.instruction(&format!("ldrb w11, [{}], #1", src));                 // load one source byte
    emitter.instruction("strb w11, [x10], #1");                                 // store it at the destination cursor
    emitter.instruction(&format!("sub {}, {}, #1", len, len));                // one byte fewer
    emitter.instruction(&format!("b {}", label));                             // continue copying bytes
    emitter.label(&format!("{}_end", label));
}

/// Copies r9 bytes from r8 to the destination cursor r10, eight bytes at a time and then byte
/// by byte; r8, r9, r10 and r11 are clobbered.
fn emit_copy_x86_64(emitter: &mut Emitter, label: &str) {
    emitter.label(&format!("{}_words", label));
    emitter.instruction("cmp r9, 8");                                           // at least one whole word left?
    emitter.instruction(&format!("jb {}", label));                            // no: finish byte by byte
    emitter.instruction("mov r11, QWORD PTR [r8]");                             // load eight source bytes
    emitter.instruction("mov QWORD PTR [r10], r11");                            // store them at the destination cursor
    emitter.instruction("add r8, 8");                                           // advance the source pointer
    emitter.instruction("add r10, 8");                                          // advance the destination cursor
    emitter.instruction("sub r9, 8");                                           // eight bytes fewer
    emitter.instruction(&format!("jmp {}_words", label));                     // continue with the next word
    emitter.label(label);
    emitter.instruction("test r9, r9");                                         // every byte copied?
    emitter.instruction(&format!("je {}_end", label));                        // yes
    emitter.instruction("mov r11b, BYTE PTR [r8]");                             // load one source byte
    emitter.instruction("mov BYTE PTR [r10], r11b");                            // store it at the destination cursor
    emitter.instruction("add r8, 1");                                           // advance the source pointer
    emitter.instruction("add r10, 1");                                          // advance the destination cursor
    emitter.instruction("sub r9, 1");                                           // one byte fewer
    emitter.instruction(&format!("jmp {}", label));                           // continue copying bytes
    emitter.label(&format!("{}_end", label));
}

#[cfg(test)]
mod tests {
    use crate::codegen_support::platform::{Arch, Platform, Target};

    use super::*;

    #[test]
    /// Verifies that the x86_64 Linux concat path uses native byte-copy loops and returns
    /// the result pointer/length in rax/rdx per the AMD64 ABI convention.
    fn test_emit_concat_linux_x86_64_uses_native_copy_loop() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_concat(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_concat:\n"));
        assert!(asm.contains("mov QWORD PTR [rbp - 8], rax\n"));
        assert!(asm.contains("mov r11b, BYTE PTR [r8]\n"));
        assert!(asm.contains("mov rax, QWORD PTR [rbp - 48]\n"));
    }
}
