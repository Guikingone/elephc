//! Purpose:
//! Emits the `__rt_ucwords`, `__rt_strcopy` runtime helper assembly for ucwords.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.
//! - PHP's `$separators` is a byte SET, so the helper takes a bounded separator string and
//!   tests each subject byte for membership. The caller always supplies one: the backend
//!   passes `_ucwords_default_seps` (`" \t\r\n\f\v"`) when the argument is omitted, so the
//!   default and an explicit set share this single scan. The previous hard-coded
//!   space/tab/newline test also silently failed PHP's `\r`, `\f`, and `\v` defaults.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_ucwords` runtime helper for ARM64.
///
/// Uppercases the first character of each word in a PHP byte-string, where a word starts at
/// the subject's first byte and after every byte that appears in the separator set.
///
/// Input registers (ARM64):
///   - x1: pointer to the input string
///   - x2: length of the input string
///   - x3: pointer to the separator byte set
///   - x4: length of the separator byte set
/// Output registers:
///   - x1: pointer to the result (concat-backed mutable copy via `__rt_strcopy`)
///   - x2: length of the result string
///
/// Clobbers: x9, x10, x11, x12, x13, x14, x15.
pub fn emit_ucwords(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ucwords_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ucwords ---");
    emitter.label_global("__rt_ucwords");
    emitter.instruction("sub sp, sp, #32");                                     // allocate a frame with room for the borrowed separator set
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // set frame pointer
    emitter.instruction("stp x3, x4, [sp]");                                    // preserve the separator set across the copy, which clobbers x6-x12
    emitter.instruction("bl __rt_strcopy");                                     // copy string to mutable concat_buf
    emitter.instruction("ldp x3, x4, [sp]");                                    // restore the separator pointer and length after the copy
    emitter.instruction("cbz x2, __rt_ucwords_done");                           // empty string → nothing to do
    emitter.instruction("mov x9, x1");                                          // cursor pointer
    emitter.instruction("mov x10, x2");                                         // remaining length
    emitter.instruction("mov x11, #1");                                         // word_start flag (1 = next char starts a word)

    emitter.label("__rt_ucwords_loop");
    emitter.instruction("cbz x10, __rt_ucwords_done");                          // no bytes left → done
    emitter.instruction("ldrb w12, [x9]");                                      // load current byte

    // -- membership test against the separator byte set --
    emitter.instruction("mov x13, x3");                                         // restart the separator cursor for this subject byte
    emitter.instruction("mov x14, x4");                                         // restart the separator counter for this subject byte
    emitter.label("__rt_ucwords_sep_loop");
    emitter.instruction("cbz x14, __rt_ucwords_word");                          // the byte is not a separator, so it may begin or continue a word
    emitter.instruction("ldrb w15, [x13], #1");                                 // load the next separator byte and advance the cursor
    emitter.instruction("sub x14, x14, #1");                                    // record that one separator byte has been compared
    emitter.instruction("cmp w15, w12");                                        // does the subject byte appear in the separator set?
    emitter.instruction("b.ne __rt_ucwords_sep_loop");                          // keep scanning the remaining separator bytes
    emitter.instruction("mov x11, #1");                                         // set word_start flag for the byte after this separator
    emitter.instruction("b __rt_ucwords_next");                                 // advance past the separator byte

    // -- not a separator: uppercase if word_start --
    emitter.label("__rt_ucwords_word");
    emitter.instruction("cbz x11, __rt_ucwords_next");                          // not word start → skip uppercasing
    emitter.instruction("cmp w12, #97");                                        // check if char >= 'a'
    emitter.instruction("b.lt __rt_ucwords_clear");                             // not lowercase → just clear flag
    emitter.instruction("cmp w12, #122");                                       // check if char <= 'z'
    emitter.instruction("b.gt __rt_ucwords_clear");                             // not lowercase → just clear flag
    emitter.instruction("sub w12, w12, #32");                                   // convert a-z to A-Z
    emitter.instruction("strb w12, [x9]");                                      // store uppercased byte
    emitter.label("__rt_ucwords_clear");
    emitter.instruction("mov x11, #0");                                         // clear word_start flag

    emitter.label("__rt_ucwords_next");
    emitter.instruction("add x9, x9, #1");                                      // advance cursor
    emitter.instruction("sub x10, x10, #1");                                    // decrement remaining
    emitter.instruction("b __rt_ucwords_loop");                                 // process next byte

    emitter.label("__rt_ucwords_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return with x1/x2 from strcopy
}

/// Emits the x86_64 Linux variant of the `__rt_ucwords` runtime helper.
///
/// Uppercases the first character of each word in a PHP byte-string, where a word starts at
/// the subject's first byte and after every byte that appears in the separator set.
///
/// Input registers (x86_64 System V ABI):
///   - rdi: pointer to the input string
///   - rsi: length of the input string
///   - rdx: pointer to the separator byte set
///   - rcx: length of the separator byte set
/// Output registers:
///   - rax: pointer to the result (concat-backed mutable copy via `__rt_strcopy`)
///   - rdx: length of the result string
///
/// The copied string's pointer and length are spilled rather than kept in registers, because
/// the separator membership scan needs both `rax` and `rdx` as scratch.
///
/// Clobbers: rax, rcx, rdx, rdi, rsi, r8, r9, r10, r11.
fn emit_ucwords_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ucwords ---");
    emitter.label_global("__rt_ucwords");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the string copy
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the spilled pointers
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the separator set and the copied string
    emitter.instruction("mov QWORD PTR [rsp], rdx");                            // preserve the separator pointer across the copy
    emitter.instruction("mov QWORD PTR [rsp + 8], rcx");                        // preserve the separator length across the copy
    emitter.instruction("mov rax, rdi");                                        // __rt_strcopy reads its source pointer from rax, not from the SysV first argument register
    emitter.instruction("mov rdx, rsi");                                        // __rt_strcopy reads its source length from rdx, not from the SysV second argument register
    emitter.instruction("call __rt_strcopy");                                   // copy the source string into concat storage so ucwords() can mutate bytes in place without touching borrowed input
    emitter.instruction("mov QWORD PTR [rsp + 16], rax");                       // spill the copied string pointer that must be returned
    emitter.instruction("mov QWORD PTR [rsp + 24], rdx");                       // spill the copied string length that must be returned
    emitter.instruction("test rdx, rdx");                                       // skip the word-start scan when ucwords() receives an empty string
    emitter.instruction("jz __rt_ucwords_done_linux_x86_64");                   // return immediately when there are no bytes to uppercase
    emitter.instruction("mov r8, rax");                                         // seed the mutable string cursor with the concat-backed copy returned by __rt_strcopy
    emitter.instruction("mov rcx, rdx");                                        // seed the remaining-length counter from the copied string length returned by __rt_strcopy
    emitter.instruction("mov r11, QWORD PTR [rsp]");                            // reload the separator set pointer for the membership scans
    emitter.instruction("mov rdi, QWORD PTR [rsp + 8]");                        // reload the separator set length for the membership scans
    emitter.instruction("mov r9, 1");                                           // start in word-start mode so the first non-separator byte can be uppercased when appropriate

    emitter.label("__rt_ucwords_loop_linux_x86_64");
    emitter.instruction("test rcx, rcx");                                       // stop once every byte of the concat-backed copy has been classified
    emitter.instruction("jz __rt_ucwords_done_linux_x86_64");                   // finish once the full copied string has been processed
    emitter.instruction("movzx r10d, BYTE PTR [r8]");                           // load the current byte from the mutable concat-backed copy before classifying separators and ASCII case

    emitter.instruction("mov rsi, r11");                                        // restart the separator cursor for this subject byte
    emitter.instruction("mov rax, rdi");                                        // restart the separator counter for this subject byte
    emitter.label("__rt_ucwords_sep_loop_linux_x86_64");
    emitter.instruction("test rax, rax");                                       // has the whole separator set been compared without a match?
    emitter.instruction("jz __rt_ucwords_word_linux_x86_64");                   // the byte is not a separator, so it may begin or continue a word
    emitter.instruction("movzx edx, BYTE PTR [rsi]");                           // load the next separator byte for the membership comparison
    emitter.instruction("add rsi, 1");                                          // advance the separator cursor past the compared byte
    emitter.instruction("sub rax, 1");                                          // record that one separator byte has been compared
    emitter.instruction("cmp dl, r10b");                                        // does the subject byte appear in the separator set?
    emitter.instruction("jne __rt_ucwords_sep_loop_linux_x86_64");              // keep scanning the remaining separator bytes
    emitter.instruction("mov r9, 1");                                           // mark the next byte as the start of a new word after a separator
    emitter.instruction("jmp __rt_ucwords_next_linux_x86_64");                  // advance past the separator byte

    emitter.label("__rt_ucwords_word_linux_x86_64");
    emitter.instruction("test r9, r9");                                         // should ucwords() try to uppercase the current non-separator byte?
    emitter.instruction("jz __rt_ucwords_next_linux_x86_64");                   // skip the ASCII-case conversion when the current byte is inside an existing word
    emitter.instruction("cmp r10b, 97");                                        // compare the current byte against 'a' to detect lowercase ASCII letters
    emitter.instruction("jb __rt_ucwords_clear_linux_x86_64");                  // clear word-start mode without mutating bytes below 'a'
    emitter.instruction("cmp r10b, 122");                                       // compare the current byte against 'z' to bound the lowercase ASCII range
    emitter.instruction("ja __rt_ucwords_clear_linux_x86_64");                  // clear word-start mode without mutating bytes above 'z'
    emitter.instruction("sub r10b, 32");                                        // convert the first lowercase ASCII letter of the word to uppercase
    emitter.instruction("mov BYTE PTR [r8], r10b");                             // store the uppercased first letter back into the mutable concat-backed copy

    emitter.label("__rt_ucwords_clear_linux_x86_64");
    emitter.instruction("mov r9, 0");                                           // clear word-start mode after the first byte of the current word has been handled

    emitter.label("__rt_ucwords_next_linux_x86_64");
    emitter.instruction("add r8, 1");                                           // advance the mutable string cursor after classifying the current byte
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining byte count after processing one byte from the copied string
    emitter.instruction("jmp __rt_ucwords_loop_linux_x86_64");                  // continue processing bytes until the full copied string has been classified

    emitter.label("__rt_ucwords_done_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rsp + 16]");                       // reload the copied string pointer into the standard x86_64 string result register
    emitter.instruction("mov rdx, QWORD PTR [rsp + 24]");                       // reload the copied string length into the standard x86_64 string length register
    emitter.instruction("add rsp, 32");                                         // release the ucwords spill slots before returning the mutated copy
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the mutated copy
    emitter.instruction("ret");                                                 // return the mutated concat-backed copy in the standard x86_64 string result registers
}
