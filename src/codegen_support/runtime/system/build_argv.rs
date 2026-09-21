//! Purpose:
//! Emits the `__rt_build_argv`, `__rt_array_new` runtime helper assembly for build argv.
//! Keeps PHP builtin semantics, libc/syscall boundaries, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - The helper constructs PHP $argv arrays from OS argc/argv without taking ownership of OS-provided storage.
//! - Both targets allocate through `__rt_array_new`, which writes the packed kind word at
//!   `[array - 8]` (heap kind, string value_type tag, copy-on-write flag). x86_64 used to
//!   `malloc` the block by hand and fill the 24-byte header only, so `[array - 8]` held
//!   malloc's own metadata: boxing `$argv` into a `mixed` cell then read its elements with
//!   the wrong stride and answered a pointer as `int` (issue #984).

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits the `__rt_build_argv` runtime helper for the current target.
/// Reads `_global_argc` and `_global_argv` populated by the entry point, iterates over
/// each OS argument string, computes its length via null-terminator scan, and stores a
/// `ptr+len` slot in a runtime array. Returns the array pointer in the function result
/// register (`x0` on ARM64, `rax` on x86_64).
///
/// On ARM64 this function uses `__rt_array_new` and `__rt_array_push_str` for allocation.
/// On x86_64 Linux it calls `malloc` directly with a precomputed layout of `(argc * 16) + 24`
/// bytes to hold the 24-byte array header plus `argc` entries of 16 bytes each (ptr + len).
/// Callee-saved registers are preserved across the helper call sequence.
pub fn emit_build_argv(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_build_argv_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: build_argv ---");
    emitter.label_global("__rt_build_argv");

    // -- set up stack frame (48 bytes for locals + saved registers) --
    emitter.instruction("sub sp, sp, #48");                                     // allocate 48 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // set up new frame pointer

    // -- load argc from the global variable --
    abi::emit_load_symbol_to_reg(emitter, "x19", "_global_argc", 0);

    // -- load argv pointer from the global variable --
    abi::emit_load_symbol_to_reg(emitter, "x20", "_global_argv", 0);

    // -- save callee-saved registers we're about to use --
    emitter.instruction("stp x19, x20, [sp, #0]");                              // save x19 (argc) and x20 (argv) to stack
    emitter.instruction("str x21, [sp, #16]");                                  // save x21 (will hold array pointer)

    // -- create a new string array with capacity = argc --
    emitter.instruction("mov x0, x19");                                         // arg0: capacity = argc
    emitter.instruction("mov x1, #16");                                         // arg1: elem_size = 16 (ptr + len per string)
    emitter.instruction("bl __rt_array_new");                                   // allocate the array, x0 = array pointer
    emitter.instruction("mov x21, x0");                                         // x21 = array pointer (save in callee-saved reg)

    // -- initialize loop counter i = 0 --
    emitter.instruction("mov x22, #0");                                         // x22 = 0 (loop counter)
    emitter.instruction("str x22, [sp, #24]");                                  // store i on stack (survives function calls)

    // -- loop: for i = 0..argc, convert each C string and push to array --
    emitter.label("__rt_build_argv_loop");
    emitter.instruction("ldr x22, [sp, #24]");                                  // reload i from stack
    emitter.instruction("cmp x22, x19");                                        // compare i with argc
    emitter.instruction("b.ge __rt_build_argv_done");                           // if i >= argc, exit loop

    // -- get pointer to argv[i] (C string) --
    emitter.instruction("ldr x1, [x20, x22, lsl #3]");                          // x1 = argv[i] (load pointer at argv + i*8)

    // -- compute string length by scanning for null terminator --
    emitter.instruction("mov x2, #0");                                          // x2 = 0 (length counter)
    emitter.label("__rt_build_argv_strlen");
    emitter.instruction("ldrb w3, [x1, x2]");                                   // w3 = byte at str[length] (load single byte)
    emitter.instruction("cbz w3, __rt_build_argv_push");                        // if byte == 0 (null terminator), done counting
    emitter.instruction("add x2, x2, #1");                                      // length += 1
    emitter.instruction("b __rt_build_argv_strlen");                            // continue scanning

    // -- push the string (ptr in x1, len in x2) to the array --
    emitter.label("__rt_build_argv_push");
    emitter.instruction("mov x0, x21");                                         // arg0: array pointer
    emitter.instruction("bl __rt_array_push_str");                              // push string element to array
    emitter.instruction("mov x21, x0");                                         // update array pointer after possible realloc

    // -- increment loop counter and continue --
    emitter.instruction("ldr x22, [sp, #24]");                                  // reload i from stack (may have been clobbered)
    emitter.instruction("add x22, x22, #1");                                    // i += 1
    emitter.instruction("str x22, [sp, #24]");                                  // save updated i back to stack
    emitter.instruction("b __rt_build_argv_loop");                              // continue loop

    // -- loop complete, return the array pointer --
    emitter.label("__rt_build_argv_done");
    emitter.instruction("mov x0, x21");                                         // return value: array pointer in x0

    // -- restore callee-saved registers and tear down stack frame --
    emitter.instruction("ldp x19, x20, [sp, #0]");                              // restore x19 (argc) and x20 (argv)
    emitter.instruction("ldr x21, [sp, #16]");                                  // restore x21
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the x86_64 Linux variant of `__rt_build_argv` using the System V AMD64 ABI.
/// Uses callee-saved registers `r8` (argc), `r9` (argv), `r10` (array pointer), `r11`
/// (current string pointer), and `rcx` (loop index). The loop counter lives at `[rbp - 32]`
/// and the array pointer at `[rbp - 24]`. String length is accumulated in `rdx` via null-terminator
/// scan. Allocates through `__rt_array_new(argc, 16)` — the same helper the ARM64 path uses — and
/// stores the resulting array pointer in `rax` before returning.
fn emit_build_argv_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: build_argv ---");
    emitter.label_global("__rt_build_argv");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving local scratch space
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for argc/argv bookkeeping
    emitter.instruction("sub rsp, 32");                                         // reserve local slots for argc, argv, result pointer, and loop index

    abi::emit_load_symbol_to_reg(emitter, "r8", "_global_argc", 0);
    abi::emit_load_symbol_to_reg(emitter, "r9", "_global_argv", 0);
    emitter.instruction("mov QWORD PTR [rbp - 8], r8");                         // save argc across the malloc call and later loop iterations
    emitter.instruction("mov QWORD PTR [rbp - 16], r9");                        // save the OS argv pointer array across the malloc call

    emitter.instruction("mov rdi, r8");                                         // arg0: capacity = argc, one 16-byte slot per OS argument
    emitter.instruction("mov rsi, 16");                                         // arg1: elem_size = 16 (ptr + len per string)
    emitter.instruction("call __rt_array_new");                                 // allocate through the runtime so the kind word at [rax-8] is written
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the allocated array pointer for the loop body and final return

    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload argc after the call may have clobbered caller-saved registers
    emitter.instruction("mov QWORD PTR [rax], r8");                             // header[0] = logical argv length; capacity and elem_size are already set
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // initialize the argv loop counter to zero

    emitter.label("__rt_build_argv_loop");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // reload the current argv element index
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 8]");                        // compare the loop index against argc
    emitter.instruction("jae __rt_build_argv_done");                            // stop once every OS argv entry has been materialized

    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                       // reload the OS argv pointer table base
    emitter.instruction("mov r11, QWORD PTR [r10 + rcx * 8]");                  // load argv[i] as a raw C string pointer
    emitter.instruction("xor rdx, rdx");                                        // reset the byte-count accumulator before scanning for the null terminator

    emitter.label("__rt_build_argv_strlen");
    emitter.instruction("mov al, BYTE PTR [r11 + rdx]");                        // read the next byte from argv[i] while measuring its PHP string length
    emitter.instruction("test al, al");                                         // check whether the current byte is the terminating NUL
    emitter.instruction("je __rt_build_argv_store");                            // stop scanning once the C string terminator is reached
    emitter.instruction("add rdx, 1");                                          // advance the measured argv[i] length by one byte
    emitter.instruction("jmp __rt_build_argv_strlen");                          // continue scanning the current argv[i] C string

    emitter.label("__rt_build_argv_store");
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the destination argv array pointer
    emitter.instruction("mov rsi, rcx");                                        // copy the element index into a scratch register for 16-byte slot addressing
    emitter.instruction("shl rsi, 4");                                          // convert the argv element index into a byte offset
    emitter.instruction("add r10, rsi");                                        // advance to the selected argv element slot
    emitter.instruction("add r10, 24");                                         // skip the array header to reach element storage
    emitter.instruction("mov QWORD PTR [r10], r11");                            // store argv[i]'s raw pointer as the PHP string payload pointer
    emitter.instruction("mov QWORD PTR [r10 + 8], rdx");                        // store argv[i]'s measured length beside the pointer

    emitter.instruction("add rcx, 1");                                          // increment the argv loop counter
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // persist the updated loop counter for the next iteration
    emitter.instruction("jmp __rt_build_argv_loop");                            // continue materializing the remaining argv entries

    emitter.label("__rt_build_argv_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // return the argv array pointer in the integer result register
    emitter.instruction("add rsp, 32");                                         // release the local argc/argv scratch slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning
    emitter.instruction("ret");                                                 // return the materialized argv array header pointer
}

#[cfg(test)]
mod tests {
    use crate::codegen_support::platform::{Arch, Platform, Target};

    use super::*;

    /// Verifies that the x86_64 Linux path allocates through `__rt_array_new`, publishes the
    /// logical length, and stores each string's length beside its pointer.
    ///
    /// `malloc` is what it used to call, and the reason issue #984 reproduced only here: the
    /// packed kind word lives at `[array - 8]` and only `__rt_array_new` writes it, so a
    /// hand-rolled block left malloc's own metadata where the boxed-`mixed` element reader
    /// looks for the slot stride. The negative assertion is the regression guard.
    #[test]
    fn test_emit_build_argv_linux_x86_64_allocates_through_the_runtime() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_build_argv(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_build_argv:\n"));
        assert!(asm.contains("call __rt_array_new\n"));
        assert!(!asm.contains("call malloc\n"));
        assert!(asm.contains("mov QWORD PTR [rax], r8\n"));
        assert!(asm.contains("mov QWORD PTR [r10 + 8], rdx\n"));
    }

    /// Both targets build `$argv` the same way: `__rt_array_new` with a 16-byte element size.
    ///
    /// AGENTS.md forbids landing runtime work ARM64-first, and this helper was exactly that —
    /// ARM64 went through the runtime allocator from the start while x86_64 open-coded the
    /// header. Deriving the expectation from both targets is what keeps them together.
    #[test]
    fn test_emit_build_argv_uses_the_runtime_allocator_on_every_target() {
        for arch in [Arch::AArch64, Arch::X86_64] {
            let mut emitter = Emitter::new(Target::new(Platform::Linux, arch));
            emit_build_argv(&mut emitter);
            let asm = emitter.output();
            assert!(
                asm.contains("__rt_array_new"),
                "{arch:?} must allocate $argv through the runtime: {asm}"
            );
            assert!(
                !asm.contains("call malloc") && !asm.contains("bl malloc"),
                "{arch:?} must not hand-roll the $argv block: {asm}"
            );
        }
    }
}
