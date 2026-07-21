//! Purpose:
//! Emits the `__rt_ucwords`, `__rt_strcopy` runtime helper assembly for ucwords.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_ucwords` runtime helper for ARM64.
///
/// Uppercases the first character of each word in a PHP byte-string.
/// Whitespace characters (space ASCII 32, tab ASCII 9, newline ASCII 10) are word separators.
///
/// Input registers (ARM64):
///   - x1: pointer to the input string
///   - x2: length of the input string
/// Output registers:
///   - x1: pointer to the result (heap-allocated via `__rt_strcopy`, refcounted)
///   - x2: length of the result string
///
/// Clobbers: x9, x10, x11, x12.
pub fn emit_ucwords(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ucwords_linux_x86_64(emitter);
        emit_ucwords_sep_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ucwords ---");
    emitter.label_global("__rt_ucwords");
    emitter.instruction("sub sp, sp, #16");                                     // allocate stack frame
    emitter.instruction("stp x29, x30, [sp]");                                  // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // set frame pointer
    emitter.instruction("bl __rt_strcopy");                                     // copy string to mutable concat_buf
    emitter.instruction("cbz x2, __rt_ucwords_done");                           // empty string → nothing to do
    emitter.instruction("mov x9, x1");                                          // cursor pointer
    emitter.instruction("mov x10, x2");                                         // remaining length
    emitter.instruction("mov x11, #1");                                         // word_start flag (1 = next char starts a word)

    emitter.label("__rt_ucwords_loop");
    emitter.instruction("cbz x10, __rt_ucwords_done");                          // no bytes left → done
    emitter.instruction("ldrb w12, [x9]");                                      // load current byte
    // -- check if current char is whitespace --
    emitter.instruction("cmp w12, #32");                                        // space?
    emitter.instruction("b.eq __rt_ucwords_ws");                                // yes → mark next as word start
    emitter.instruction("cmp w12, #9");                                         // tab?
    emitter.instruction("b.eq __rt_ucwords_ws");                                // yes → mark next as word start
    emitter.instruction("cmp w12, #10");                                        // newline?
    emitter.instruction("b.eq __rt_ucwords_ws");                                // yes → mark next as word start
    // -- not whitespace: uppercase if word_start --
    emitter.instruction("cbz x11, __rt_ucwords_next");                          // not word start → skip uppercasing
    emitter.instruction("cmp w12, #97");                                        // check if char >= 'a'
    emitter.instruction("b.lt __rt_ucwords_clear");                             // not lowercase → just clear flag
    emitter.instruction("cmp w12, #122");                                       // check if char <= 'z'
    emitter.instruction("b.gt __rt_ucwords_clear");                             // not lowercase → just clear flag
    emitter.instruction("sub w12, w12, #32");                                   // convert a-z to A-Z
    emitter.instruction("strb w12, [x9]");                                      // store uppercased byte
    emitter.label("__rt_ucwords_clear");
    emitter.instruction("mov x11, #0");                                         // clear word_start flag
    emitter.instruction("b __rt_ucwords_next");                                 // advance to next char

    emitter.label("__rt_ucwords_ws");
    emitter.instruction("mov x11, #1");                                         // set word_start flag for next char

    emitter.label("__rt_ucwords_next");
    emitter.instruction("add x9, x9, #1");                                      // advance cursor
    emitter.instruction("sub x10, x10, #1");                                    // decrement remaining
    emitter.instruction("b __rt_ucwords_loop");                                 // process next byte

    emitter.label("__rt_ucwords_done");
    emitter.instruction("ldp x29, x30, [sp]");                                  // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return with x1/x2 from strcopy

    emit_ucwords_sep(emitter);
}

/// Emits the `__rt_ucwords_sep` runtime helper for ARM64.
///
/// Like `__rt_ucwords`, but word boundaries are the bytes of a caller-supplied
/// separators string rather than the fixed whitespace set. The first character of
/// the string is always a word start (PHP capitalizes it regardless of the
/// separator set).
///
/// Input registers (ARM64):
///   - x1/x2: source string pointer/length (copied via `__rt_strcopy`)
///   - x3/x4: separators string pointer/length
/// Output registers:
///   - x1/x2: pointer/length of the heap-allocated, capitalized result
///
/// Clobbers: x9, x10, x11, x12, x13, x14, x15.
fn emit_ucwords_sep(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ucwords with custom separators ---");
    emitter.label_global("__rt_ucwords_sep");
    emitter.instruction("sub sp, sp, #32");                                     // allocate stack frame with separators spill slots
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // set frame pointer
    emitter.instruction("str x3, [sp, #0]");                                    // preserve separators pointer across __rt_strcopy
    emitter.instruction("str x4, [sp, #8]");                                    // preserve separators length across __rt_strcopy
    emitter.instruction("bl __rt_strcopy");                                     // copy source string to mutable concat_buf → x1/x2
    emitter.instruction("ldr x3, [sp, #0]");                                    // restore separators pointer after the copy
    emitter.instruction("ldr x4, [sp, #8]");                                    // restore separators length after the copy
    emitter.instruction("cbz x2, __rt_ucwords_sep_done");                       // empty string → nothing to do
    emitter.instruction("mov x9, x1");                                          // cursor pointer over the copied string
    emitter.instruction("mov x10, x2");                                         // remaining length
    emitter.instruction("mov x11, #1");                                         // word_start flag (1 = next char starts a word)

    emitter.label("__rt_ucwords_sep_loop");
    emitter.instruction("cbz x10, __rt_ucwords_sep_done");                      // no bytes left → done
    emitter.instruction("ldrb w12, [x9]");                                      // load current byte

    // -- test membership of the current byte in the separators string --
    emitter.instruction("mov x13, x3");                                         // separator scan cursor
    emitter.instruction("mov x14, x4");                                         // separator scan remaining
    emitter.label("__rt_ucwords_sep_scan");
    emitter.instruction("cbz x14, __rt_ucwords_sep_notsep");                    // exhausted separators → current byte is not a separator
    emitter.instruction("ldrb w15, [x13]");                                     // load candidate separator byte
    emitter.instruction("cmp w12, w15");                                        // does the current byte match this separator?
    emitter.instruction("b.eq __rt_ucwords_sep_issep");                         // yes → mark next char as a word start
    emitter.instruction("add x13, x13, #1");                                    // advance separator scan cursor
    emitter.instruction("sub x14, x14, #1");                                    // decrement separator scan remaining
    emitter.instruction("b __rt_ucwords_sep_scan");                             // keep scanning the separators string

    emitter.label("__rt_ucwords_sep_issep");
    emitter.instruction("mov x11, #1");                                         // separator → next char starts a word
    emitter.instruction("b __rt_ucwords_sep_next");                             // advance to next char

    emitter.label("__rt_ucwords_sep_notsep");
    emitter.instruction("cbz x11, __rt_ucwords_sep_next");                      // not word start → leave byte unchanged
    emitter.instruction("cmp w12, #97");                                        // check if char >= 'a'
    emitter.instruction("b.lt __rt_ucwords_sep_clear");                         // not lowercase → just clear flag
    emitter.instruction("cmp w12, #122");                                       // check if char <= 'z'
    emitter.instruction("b.gt __rt_ucwords_sep_clear");                         // not lowercase → just clear flag
    emitter.instruction("sub w12, w12, #32");                                   // convert a-z to A-Z
    emitter.instruction("strb w12, [x9]");                                      // store uppercased byte
    emitter.label("__rt_ucwords_sep_clear");
    emitter.instruction("mov x11, #0");                                         // clear word_start flag

    emitter.label("__rt_ucwords_sep_next");
    emitter.instruction("add x9, x9, #1");                                      // advance cursor
    emitter.instruction("sub x10, x10, #1");                                    // decrement remaining
    emitter.instruction("b __rt_ucwords_sep_loop");                             // process next byte

    emitter.label("__rt_ucwords_sep_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return with x1/x2 from strcopy
}

/// Emits the x86_64 Linux variant of the `__rt_ucwords` runtime helper.
///
/// Uppercases the first character of each word in a PHP byte-string.
/// Whitespace characters (space ASCII 32, tab ASCII 9, newline ASCII 10) are word separators.
///
/// Input registers (x86_64 System V ABI):
///   - rdi: pointer to the input string
///   - rsi: length of the input string
/// Output registers:
///   - rax: pointer to the result (heap-allocated via `__rt_strcopy`, refcounted)
///   - rdx: length of the result string
///
/// Clobbers: r8, rcx, r9, r10.
fn emit_ucwords_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ucwords ---");
    emitter.label_global("__rt_ucwords");
    emitter.instruction("call __rt_strcopy");                                   // copy the source string into concat storage so ucwords() can mutate bytes in place without touching borrowed input
    emitter.instruction("test rdx, rdx");                                       // skip the word-start scan when ucwords() receives an empty string
    emitter.instruction("jz __rt_ucwords_done_linux_x86_64");                   // return immediately when there are no bytes to uppercase
    emitter.instruction("mov r8, rax");                                         // seed the mutable string cursor with the concat-backed copy returned by __rt_strcopy
    emitter.instruction("mov rcx, rdx");                                        // seed the remaining-length counter from the copied string length returned by __rt_strcopy
    emitter.instruction("mov r9, 1");                                           // start in word-start mode so the first non-whitespace byte can be uppercased when appropriate

    emitter.label("__rt_ucwords_loop_linux_x86_64");
    emitter.instruction("test rcx, rcx");                                       // stop once every byte of the concat-backed copy has been classified
    emitter.instruction("jz __rt_ucwords_done_linux_x86_64");                   // finish once the full copied string has been processed
    emitter.instruction("movzx r10d, BYTE PTR [r8]");                           // load the current byte from the mutable concat-backed copy before classifying whitespace and ASCII case
    emitter.instruction("cmp r10b, 32");                                        // is the current byte a space that marks the start of the next word?
    emitter.instruction("je __rt_ucwords_ws_linux_x86_64");                     // mark the next byte as a word start after a space separator
    emitter.instruction("cmp r10b, 9");                                         // is the current byte a tab that marks the start of the next word?
    emitter.instruction("je __rt_ucwords_ws_linux_x86_64");                     // mark the next byte as a word start after a tab separator
    emitter.instruction("cmp r10b, 10");                                        // is the current byte a newline that marks the start of the next word?
    emitter.instruction("je __rt_ucwords_ws_linux_x86_64");                     // mark the next byte as a word start after a newline separator
    emitter.instruction("test r9, r9");                                         // should ucwords() try to uppercase the current non-whitespace byte?
    emitter.instruction("jz __rt_ucwords_next_linux_x86_64");                   // skip the ASCII-case conversion when the current byte is inside an existing word
    emitter.instruction("cmp r10b, 97");                                        // compare the current byte against 'a' to detect lowercase ASCII letters
    emitter.instruction("jb __rt_ucwords_clear_linux_x86_64");                  // clear word-start mode without mutating bytes below 'a'
    emitter.instruction("cmp r10b, 122");                                       // compare the current byte against 'z' to bound the lowercase ASCII range
    emitter.instruction("ja __rt_ucwords_clear_linux_x86_64");                  // clear word-start mode without mutating bytes above 'z'
    emitter.instruction("sub r10b, 32");                                        // convert the first lowercase ASCII letter of the word to uppercase
    emitter.instruction("mov BYTE PTR [r8], r10b");                             // store the uppercased first letter back into the mutable concat-backed copy

    emitter.label("__rt_ucwords_clear_linux_x86_64");
    emitter.instruction("mov r9, 0");                                           // clear word-start mode after the first byte of the current word has been handled
    emitter.instruction("jmp __rt_ucwords_next_linux_x86_64");                  // advance to the next byte after handling the current word-start candidate

    emitter.label("__rt_ucwords_ws_linux_x86_64");
    emitter.instruction("mov r9, 1");                                           // mark the next non-whitespace byte as the start of a new word after a separator

    emitter.label("__rt_ucwords_next_linux_x86_64");
    emitter.instruction("add r8, 1");                                           // advance the mutable string cursor after classifying the current byte
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining byte count after processing one byte from the copied string
    emitter.instruction("jmp __rt_ucwords_loop_linux_x86_64");                  // continue processing bytes until the full copied string has been classified

    emitter.label("__rt_ucwords_done_linux_x86_64");
    emitter.instruction("ret");                                                 // return the mutated concat-backed copy in the standard x86_64 string result registers
}

/// Emits the x86_64 Linux variant of the `__rt_ucwords_sep` runtime helper.
///
/// Like `__rt_ucwords`, but the word boundaries are the bytes of a caller-supplied
/// separators string. The first character is always a word start.
///
/// Input registers (x86_64 System V ABI):
///   - rax/rdx: source string pointer/length (copied via `__rt_strcopy`)
///   - rdi/rsi: separators string pointer/length
/// Output registers:
///   - rax/rdx: pointer/length of the heap-allocated, capitalized result
///
/// Clobbers: r8, r9, r10, r11, rcx (rax/rdx are reloaded from spill slots on return).
fn emit_ucwords_sep_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ucwords with custom separators ---");
    emitter.label_global("__rt_ucwords_sep");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while ucwords_sep() uses spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the separators and copy spill slots
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the separators pointer/length and copied string pointer/length
    emitter.instruction("mov QWORD PTR [rsp + 0], rdi");                        // preserve the separators pointer across the __rt_strcopy call
    emitter.instruction("mov QWORD PTR [rsp + 8], rsi");                        // preserve the separators length across the __rt_strcopy call
    emitter.instruction("call __rt_strcopy");                                   // copy the source string into concat storage → rax/rdx
    emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");                        // restore the separators pointer after the copy
    emitter.instruction("mov rsi, QWORD PTR [rsp + 8]");                        // restore the separators length after the copy
    emitter.instruction("mov QWORD PTR [rsp + 16], rax");                       // preserve the copied string pointer for the return registers
    emitter.instruction("mov QWORD PTR [rsp + 24], rdx");                       // preserve the copied string length for the return registers
    emitter.instruction("test rdx, rdx");                                       // skip the scan when ucwords_sep() receives an empty string
    emitter.instruction("jz __rt_ucwords_sep_done_x86");                        // return immediately when there are no bytes to uppercase
    emitter.instruction("mov r8, rax");                                         // seed the mutable string cursor with the copied string
    emitter.instruction("mov rcx, rdx");                                        // seed the remaining-length counter from the copied string length
    emitter.instruction("mov r9, 1");                                           // start in word-start mode so the first byte can be uppercased

    emitter.label("__rt_ucwords_sep_loop_x86");
    emitter.instruction("test rcx, rcx");                                       // stop once every byte of the copied string has been classified
    emitter.instruction("jz __rt_ucwords_sep_done_x86");                        // finish once the full copied string has been processed
    emitter.instruction("movzx r10d, BYTE PTR [r8]");                           // load the current byte from the mutable copied string

    // -- test membership of the current byte in the separators string --
    emitter.instruction("mov r11, rdi");                                        // separator scan cursor
    emitter.instruction("mov rax, rsi");                                        // separator scan remaining (rax is free; copy pointer is spilled)
    emitter.label("__rt_ucwords_sep_scan_x86");
    emitter.instruction("test rax, rax");                                       // exhausted separators?
    emitter.instruction("jz __rt_ucwords_sep_notsep_x86");                      // current byte is not a separator
    emitter.instruction("movzx edx, BYTE PTR [r11]");                           // load candidate separator byte (rdx is free; copy length is spilled)
    emitter.instruction("cmp r10b, dl");                                        // does the current byte match this separator?
    emitter.instruction("je __rt_ucwords_sep_issep_x86");                       // yes → mark next char as a word start
    emitter.instruction("add r11, 1");                                          // advance separator scan cursor
    emitter.instruction("sub rax, 1");                                          // decrement separator scan remaining
    emitter.instruction("jmp __rt_ucwords_sep_scan_x86");                       // keep scanning the separators string

    emitter.label("__rt_ucwords_sep_issep_x86");
    emitter.instruction("mov r9, 1");                                           // separator → next char starts a word
    emitter.instruction("jmp __rt_ucwords_sep_next_x86");                       // advance to the next byte

    emitter.label("__rt_ucwords_sep_notsep_x86");
    emitter.instruction("test r9, r9");                                         // should ucwords_sep() try to uppercase the current byte?
    emitter.instruction("jz __rt_ucwords_sep_next_x86");                        // inside an existing word → leave byte unchanged
    emitter.instruction("cmp r10b, 97");                                        // compare the current byte against 'a'
    emitter.instruction("jb __rt_ucwords_sep_clear_x86");                       // clear word-start mode without mutating bytes below 'a'
    emitter.instruction("cmp r10b, 122");                                       // compare the current byte against 'z'
    emitter.instruction("ja __rt_ucwords_sep_clear_x86");                       // clear word-start mode without mutating bytes above 'z'
    emitter.instruction("sub r10b, 32");                                        // convert the first lowercase ASCII letter of the word to uppercase
    emitter.instruction("mov BYTE PTR [r8], r10b");                             // store the uppercased first letter back into the copied string
    emitter.label("__rt_ucwords_sep_clear_x86");
    emitter.instruction("mov r9, 0");                                           // clear word-start mode after handling the first byte of the word

    emitter.label("__rt_ucwords_sep_next_x86");
    emitter.instruction("add r8, 1");                                           // advance the mutable string cursor
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining byte count
    emitter.instruction("jmp __rt_ucwords_sep_loop_x86");                       // continue processing bytes

    emitter.label("__rt_ucwords_sep_done_x86");
    emitter.instruction("mov rax, QWORD PTR [rsp + 16]");                       // reload the copied string pointer into the string result register
    emitter.instruction("mov rdx, QWORD PTR [rsp + 24]");                       // reload the copied string length into the string result register
    emitter.instruction("add rsp, 32");                                         // release the spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the capitalized copy in the standard x86_64 string result registers
}
