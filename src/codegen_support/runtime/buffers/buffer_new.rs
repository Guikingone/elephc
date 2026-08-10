//! Purpose:
//! Emits the `__rt_buffer_new`, `__rt_heap_alloc` runtime helper assembly for buffer allocation.
//! Keeps compiler buffer extension checks and fatal paths aligned with generated pointer operations.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::buffers`.
//!
//! Key details:
//! - Buffer helpers enforce extension ownership rules, including live headers, bounds checks, and fatal paths before unsafe access.
//! - `len * stride` is validated before allocating: an unchecked product wraps to a tiny block while
//!   the header still advertises the pre-overflow length, and `buffer[i]` bounds checks trust that
//!   header, so every in-"bounds" index past the real block would read and write foreign memory.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::runtime::data::BUFFER_ALLOC_SIZE_MSG;

/// Emits the `__rt_buffer_new` runtime helper for the current target.
///
/// Allocates a zero-initialized buffer with a 16-byte header followed by a contiguous
/// payload, then returns a pointer to the header. Calls `__rt_heap_alloc` which fatal-aborts
/// on allocation failure — callers do not need a separate null check.
///
/// Inputs (platform ABI):
///   - ARM64: x0 = logical element count, x1 = element stride in bytes
///   - x86_64 Linux: rdi = logical element count, rsi = element stride in bytes
///
/// Output (platform ABI):
///   - ARM64: x0 = pointer to buffer header
///   - x86_64: rax = pointer to buffer header
///
/// Buffer header layout (both targets):
///   - header[0..8]   = logical element count (set from input x0/rdi)
///   - header[8..16]  = element stride in bytes (set from input x1/rsi)
///   - header[16..]   = zero-initialized payload region (len * stride bytes)
///
/// Rejects negative lengths and any `len * stride + 16` that does not fit in a non-negative
/// machine word by terminating through `__rt_buffer_new_size_fail`, so the stored header length
/// always describes memory the buffer actually owns.
pub fn emit_buffer_new(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_buffer_new_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: buffer_new ---");
    emitter.label_global("__rt_buffer_new");

    // -- save len/stride across heap allocation --
    emitter.instruction("sub sp, sp, #32");                                     // allocate a small stack frame for saved arguments
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the temporary frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save requested logical length
    emitter.instruction("str x1, [sp, #8]");                                    // save requested element stride

    // -- reject negative lengths and unrepresentable payload sizes --
    emitter.instruction("cmp x0, #0");                                          // is the requested logical length negative?
    emitter.instruction("b.lt __rt_buffer_new_size_fail");                      // reject negative buffer lengths outright
    emitter.instruction("umulh x9, x0, x1");                                    // x9 = high 64 bits of len * stride
    emitter.instruction("cbnz x9, __rt_buffer_new_size_fail");                  // reject payload sizes that do not fit in one machine word
    emitter.instruction("mul x9, x0, x1");                                      // x9 = low 64 bits of len * stride
    emitter.instruction("adds x9, x9, #16");                                    // x9 = payload size plus the 16-byte buffer header
    emitter.instruction("b.hs __rt_buffer_new_size_fail");                      // reject totals that carried out of the machine word
    emitter.instruction("tbnz x9, #63, __rt_buffer_new_size_fail");             // reject totals the signed heap-size check would read as negative

    // -- allocate header + contiguous payload --
    emitter.instruction("mov x0, x9");                                          // x0 = validated payload byte count plus the buffer header
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the full buffer payload on the shared heap

    // -- initialize header fields --
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the logical length after allocation
    emitter.instruction("str x9, [x0]");                                        // header[0] = logical element count
    emitter.instruction("ldr x9, [sp, #8]");                                    // reload the element stride after allocation
    emitter.instruction("str x9, [x0, #8]");                                    // header[8] = element stride in bytes

    // -- zero-initialize the contiguous payload --
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the logical length for payload size calculation
    emitter.instruction("ldr x10, [sp, #8]");                                   // reload the stride for payload size calculation
    emitter.instruction("mul x12, x9, x10");                                    // compute payload byte count = len * stride
    emitter.instruction("add x11, x0, #16");                                    // x11 = first payload byte after the 16-byte header
    emitter.instruction("add x12, x11, x12");                                   // x12 = end pointer one past the payload
    emitter.label("__rt_buffer_new_zero_loop");
    emitter.instruction("cmp x11, x12");                                        // have we cleared the whole payload yet?
    emitter.instruction("b.eq __rt_buffer_new_zero_done");                      // yes — skip the zero-fill loop
    emitter.instruction("str xzr, [x11], #8");                                  // store one zeroed machine word and advance
    emitter.instruction("b __rt_buffer_new_zero_loop");                         // continue zeroing until the end pointer is reached
    emitter.label("__rt_buffer_new_zero_done");

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the temporary frame
    emitter.instruction("ret");                                                 // return x0 = buffer header pointer

    // -- fatal error: requested buffer length cannot be represented --
    emitter.label("__rt_buffer_new_size_fail");
    emitter.instruction("mov x0, #2");                                          // fd = stderr
    abi::emit_symbol_address(emitter, "x1", "_buffer_alloc_size_msg");
    emitter.instruction(&format!("mov x2, #{}", BUFFER_ALLOC_SIZE_MSG.len()));  // pass the exact buffer-length diagnostic byte count
    emitter.syscall(4);
    emitter.instruction("mov x0, #1");                                          // exit code 1
    emitter.syscall(1);
}

/// Emits the `__rt_buffer_new` runtime helper for the x86_64 Linux target.
/// Private; dispatcher lives in `emit_buffer_new`.
///
/// Applies the same length validation as the ARM64 path: negative lengths and any
/// `len * stride + 16` that does not fit in a non-negative machine word terminate the process
/// through `__rt_buffer_new_size_fail`.
fn emit_buffer_new_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: buffer_new ---");
    emitter.label_global("__rt_buffer_new");

    // -- save len/stride across heap allocation --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving temporary spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame pointer for the saved buffer_new arguments
    emitter.instruction("sub rsp, 32");                                         // reserve spill slots for the logical length, stride, and returned buffer pointer
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the requested logical length across the nested heap allocation call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // save the requested element stride across the nested heap allocation call

    // -- reject negative lengths and unrepresentable payload sizes --
    emitter.instruction("test rax, rax");                                       // is the requested logical length negative?
    emitter.instruction("js __rt_buffer_new_size_fail");                        // reject negative buffer lengths outright

    // -- allocate header + contiguous payload --
    emitter.instruction("imul rax, rdi");                                       // compute payload byte count = len * stride in the x86_64 heap-allocation size register
    emitter.instruction("jo __rt_buffer_new_size_fail");                        // reject payload sizes that do not fit in one machine word
    emitter.instruction("add rax, 16");                                         // add the 16-byte buffer header before requesting the backing allocation
    emitter.instruction("jo __rt_buffer_new_size_fail");                        // reject totals the signed heap-size accounting would read as negative
    emitter.instruction("call __rt_heap_alloc");                                // allocate the buffer header plus contiguous payload through the shared x86_64 heap wrapper
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // preserve the allocated buffer header pointer while materializing the header fields

    // -- initialize header fields --
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the logical length after the nested heap allocation clobbered caller-saved registers
    emitter.instruction("mov QWORD PTR [rax], r10");                            // header[0] = logical element count
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                       // reload the element stride after the nested heap allocation clobbered caller-saved registers
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // header[8] = element stride in bytes

    // -- zero-initialize the contiguous payload --
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the logical length for the payload byte-count calculation
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload the element stride for the payload byte-count calculation
    emitter.instruction("imul r10, rcx");                                       // compute payload byte count = len * stride
    emitter.instruction("lea r11, [rax + 16]");                                 // r11 = first payload byte after the 16-byte buffer header
    emitter.instruction("add r10, r11");                                        // r10 = end pointer one past the contiguous payload
    emitter.label("__rt_buffer_new_zero_loop");
    emitter.instruction("cmp r11, r10");                                        // have we cleared the whole payload yet?
    emitter.instruction("je __rt_buffer_new_zero_done");                        // stop once the whole payload range has been zero-filled
    emitter.instruction("mov QWORD PTR [r11], 0");                              // store one zeroed machine word into the contiguous payload
    emitter.instruction("add r11, 8");                                          // advance the payload cursor by one machine word after the zero store
    emitter.instruction("jmp __rt_buffer_new_zero_loop");                       // continue zero-filling until the payload cursor reaches the end pointer
    emitter.label("__rt_buffer_new_zero_done");

    // -- restore frame and return --
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // restore the allocated buffer header pointer as the return value after the zero-fill loop
    emitter.instruction("add rsp, 32");                                         // release the temporary spill slots reserved for buffer_new
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning to generated code
    emitter.instruction("ret");                                                 // return rax = buffer header pointer

    // -- fatal error: requested buffer length cannot be represented --
    emitter.label("__rt_buffer_new_size_fail");
    emitter.instruction("mov edi, 2");                                          // fd = stderr for the buffer-length fatal error message
    abi::emit_symbol_address(emitter, "rsi", "_buffer_alloc_size_msg");
    emitter.instruction(&format!("mov edx, {}", BUFFER_ALLOC_SIZE_MSG.len()));  // pass the exact buffer-length diagnostic byte count
    emitter.instruction("mov eax, 1");                                          // Linux x86_64 syscall 1 = write
    emitter.instruction("syscall");                                             // print the fatal buffer-length message to stderr
    emitter.instruction("mov edi, 1");                                          // exit code 1 for an unrepresentable buffer length
    emitter.instruction("mov eax, 60");                                         // Linux x86_64 syscall 60 = exit
    emitter.instruction("syscall");                                             // terminate the process after reporting the buffer-length failure
}
