//! Purpose:
//! Emits the `__rt_str_persist`, `__rt_str_persist_done` runtime helper assembly for heap-backed string persistence.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers scan or transform byte ranges and return target ABI pointer/length pairs for generated call sites.

use crate::codegen_support::{emit::Emitter, platform::Arch};


/// str_persist: copy a string to heap for permanent storage.
/// Used to persist strings that would otherwise outlive their current owner.
/// Input:  x1=ptr, x2=len
/// Output: x1=new_ptr (on heap), x2=len (unchanged)
pub fn emit_str_persist(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_str_persist_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: str_persist ---");
    emitter.label_global("__rt_str_persist");

    crate::codegen_support::abi::emit_load_int_immediate(
        emitter,
        "x9",
        crate::codegen_support::sentinels::NULL_SENTINEL,
    );
    emitter.instruction("cmp x1, x9");                                          // preserve the dedicated null-string sentinel across ownership stabilization
    emitter.instruction("b.eq __rt_str_persist_done");                          // a missing string has no payload to allocate or copy

    // -- zero-length strings still get an owned heap block so callers never alias a borrowed source pointer --
    // (the old early-return let explode()'s empty segment alias the subject string and double-free it on release)

    // -- set up stack frame (we call heap_alloc which may clobber regs) --
    emitter.instruction("sub sp, sp, #32");                                     // allocate 32 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish new frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save source pointer
    emitter.instruction("str x2, [sp, #8]");                                    // save string length

    // -- round up allocation to next power of 2 (min 32) for free-list reuse --
    // This reduces fragmentation for string .= loops: freed blocks can be
    // reused by subsequent allocations of similar or smaller size.
    emitter.instruction("mov x0, #32");                                         // minimum allocation size
    emitter.instruction("cmp x2, #32");                                         // is length <= 32?
    emitter.instruction("b.le __rt_str_persist_alloc");                         // yes, use 32
    // -- round up to next power of 2 --
    emitter.instruction("sub x0, x2, #1");                                      // x0 = len - 1
    emitter.instruction("clz x3, x0");                                          // count leading zeros
    emitter.instruction("mov x0, #1");                                          // start with 1
    emitter.instruction("mov x4, #64");                                         // 64 - clz = bit position
    emitter.instruction("sub x4, x4, x3");                                      // x4 = 64 - leading_zeros = ceil(log2)
    emitter.instruction("lsl x0, x0, x4");                                      // x0 = 1 << ceil(log2(len)) = next power of 2
    emitter.label("__rt_str_persist_alloc");
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate on heap, x0 = heap pointer
    emitter.instruction("mov x6, #1");                                          // heap kind 1 = persisted elephc string
    emitter.instruction("str x6, [x0, #-8]");                                   // store string kind in the uniform heap header

    // -- copy bytes from source to heap --
    emitter.instruction("ldr x1, [sp, #0]");                                    // x1 = source pointer (restored)
    emitter.instruction("ldr x2, [sp, #8]");                                    // x2 = length (restored)
    emitter.instruction("mov x3, x0");                                          // x3 = destination (heap pointer)
    emitter.instruction("mov x4, x2");                                          // x4 = byte count for loop

    emitter.label("__rt_str_persist_copy8");
    emitter.instruction("cmp x4, #8");                                          // at least one whole 8-byte word left?
    emitter.instruction("b.lo __rt_str_persist_copy");                          // no — finish byte by byte
    emitter.instruction("ldr x5, [x1], #8");                                    // load eight source bytes, advance
    emitter.instruction("str x5, [x3], #8");                                    // store eight bytes to the heap, advance
    emitter.instruction("sub x4, x4, #8");                                      // decrement remaining count by a word
    emitter.instruction("b __rt_str_persist_copy8");                            // continue copying whole words

    emitter.label("__rt_str_persist_copy");
    emitter.instruction("cbz x4, __rt_str_persist_ret");                        // all bytes copied
    emitter.instruction("ldrb w5, [x1], #1");                                   // load byte from source, advance
    emitter.instruction("strb w5, [x3], #1");                                   // store byte to heap, advance
    emitter.instruction("sub x4, x4, #1");                                      // decrement remaining count
    emitter.instruction("b __rt_str_persist_copy");                             // continue copying

    // -- return heap pointer and original length --
    emitter.label("__rt_str_persist_ret");
    emitter.instruction("mov x1, x0");                                          // x1 = heap pointer (new string location)
    emitter.instruction("ldr x2, [sp, #8]");                                    // x2 = original length (unchanged)

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate stack frame

    emitter.label("__rt_str_persist_done");
    emitter.instruction("ret");                                                 // return to caller
}

/// x86_64 Linux variant of `emit_str_persist`.
/// Allocates heap storage, stamps the header with the owned-string heap kind, and copies
/// the source bytes into the owned allocation.
/// Input:  rdi=ptr, rdx=len
/// Output: rax=heap_ptr (owned), rdx=len (unchanged)
fn emit_str_persist_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: str_persist ---");
    emitter.label_global("__rt_str_persist");

    crate::codegen_support::abi::emit_load_int_immediate(
        emitter,
        "r10",
        crate::codegen_support::sentinels::NULL_SENTINEL,
    );
    emitter.instruction("cmp rax, r10");                                        // preserve the dedicated null-string sentinel across ownership stabilization
    emitter.instruction("je __rt_str_persist_done");                            // a missing string has no payload to allocate or copy

    // -- zero-length strings still get an owned heap block so callers never alias a borrowed source pointer --

    // -- preserve the source payload across the heap allocation helper call --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame pointer for the saved source pointer and length
    emitter.instruction("sub rsp, 16");                                         // reserve local slots for the source pointer and source length across the allocator call
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the source pointer across the heap allocation helper call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the source length across the heap allocation helper call
    emitter.instruction("mov rax, rdx");                                        // move the byte length into the x86_64 heap helper input register
    emitter.instruction("call __rt_heap_alloc");                                // allocate owned string storage and return the payload pointer in rax
    emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(1))); // materialize the owned-string heap kind word with the x86_64 heap magic marker
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the allocated payload as a persisted elephc string in the uniform heap header
    emitter.instruction("mov r8, rax");                                         // preserve the destination heap pointer for the byte-copy loop and final return value
    emitter.instruction("mov r9, QWORD PTR [rbp - 8]");                         // reload the source pointer after the allocator helper returns
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload the source byte length after the allocator helper returns

    // -- copy the source bytes into the owned heap allocation --
    emitter.instruction("push rsi");                                            // preserve rsi, which callers may still hold live across this helper
    emitter.instruction("push rdi");                                            // preserve rdi, which callers may still hold live across this helper
    emitter.instruction("mov rdi, r8");                                         // rdi = destination payload
    emitter.instruction("mov rsi, r9");                                         // rsi = source payload
    emitter.instruction("rep movsb");                                           // copy rcx bytes into the owned allocation
    emitter.instruction("pop rdi");                                             // restore the caller's rdi
    emitter.instruction("pop rsi");                                             // restore the caller's rsi

    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // restore the original string length for the x86_64 string result pair
    emitter.instruction("mov rax, r8");                                         // return the owned string pointer in the x86_64 string result register
    emitter.instruction("add rsp, 16");                                         // release the temporary spill slots used by the persist helper
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning

    emitter.label("__rt_str_persist_done");
    emitter.instruction("ret");                                                 // return to the caller with rax=owned_ptr and rdx=length
}

#[cfg(test)]
mod tests {
    use crate::codegen_support::platform::{Arch, Platform, Target};

    use super::*;

    /// Verifies emit str persist linux x86_64 uses heap helper.
    #[test]
    fn test_emit_str_persist_linux_x86_64_uses_heap_helper() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_str_persist(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_str_persist:\n"));
        assert!(asm.contains("call __rt_heap_alloc\n"));
        assert!(asm.contains("mov QWORD PTR [rax - 8], r10\n"));
    }
}
