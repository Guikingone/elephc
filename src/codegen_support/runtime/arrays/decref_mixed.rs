//! Purpose:
//! Emits the `__rt_decref_mixed`, `__rt_decref_mixed_skip` runtime helper assembly for decref mixed.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Decrement helpers are release paths for refcounted values; cycle collection
//!   runs only from explicit safe points after PHP-visible roots are updated.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;


/// Emits the `__rt_decref_mixed` runtime helper for the current target.
///
/// # Arguments
/// * `emitter` - The assembly emitter to write instructions into.
///
/// # Behavior
/// - On ARM64: validates the pointer lies within the managed heap, decrements the refcount
///   from the mixed cell header, and calls `__rt_mixed_free_deep` to release the payload
///   when the refcount reaches zero.
/// - On x86_64: validates the pointer is in-bounds with the correct heap magic header,
///   decrements the refcount, and tail-calls `__rt_mixed_free_deep` on zero refcount.
///
/// # ABI
/// - ARM64: input pointer in `x0`; clobbers `x9`, `x10`; preserves `x30` (link register).
/// - x86_64: input pointer in `rax`; clobbers `r10`, `r11`.
pub fn emit_decref_mixed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_decref_mixed_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: decref_mixed ---");
    emitter.label_global("__rt_decref_mixed");

    emitter.instruction("cbz x0, __rt_decref_mixed_skip");                      // skip null mixed pointers immediately
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_buf");
    emitter.instruction("cmp x0, x9");                                          // is pointer below heap start?
    emitter.instruction("b.lo __rt_decref_mixed_skip");                         // non-heap pointers need no mixed decref
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_heap_off");
    emitter.instruction("ldr x10, [x10]");                                      // x10 = current heap offset
    emitter.instruction("add x10, x9, x10");                                    // compute the current heap end
    emitter.instruction("cmp x0, x10");                                         // is pointer at or beyond heap end?
    emitter.instruction("b.hs __rt_decref_mixed_skip");                         // invalid heap pointers must be ignored here

    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_debug_enabled");
    emitter.instruction("ldr x9, [x9]");                                        // load the heap-debug enabled flag
    emitter.instruction("cbz x9, __rt_decref_mixed_checked");                   // skip debug validation when heap-debug mode is disabled
    emitter.instruction("str x30, [sp, #-16]!");                                // preserve return address before nested validation
    emitter.instruction("bl __rt_heap_debug_check_live");                       // ensure the mixed cell is still live
    emitter.instruction("ldr x30, [sp], #16");                                  // restore return address after validation
    emitter.label("__rt_decref_mixed_checked");

    emitter.instruction("ldr w9, [x0, #-12]");                                  // load the mixed cell refcount from the uniform header
    emitter.instruction("subs w9, w9, #1");                                     // decrement the mixed cell refcount and set flags
    emitter.instruction("str w9, [x0, #-12]");                                  // store the decremented mixed cell refcount
    emitter.instruction("b.eq __rt_decref_mixed_free");                         // zero refcount means the boxed payload can be released now

    emitter.instruction("b __rt_decref_mixed_skip");                            // non-zero refcount stays alive until an explicit GC safe point

    emitter.label("__rt_decref_mixed_free");
    emit_scalar_box_fast_free(emitter);
    emitter.instruction("b __rt_mixed_free_deep");                              // tail-call to deep free the mixed cell and its boxed child

    emitter.label("__rt_decref_mixed_skip");
    emitter.instruction("ret");                                                 // nothing to release
}

/// Emits the `__rt_decref_mixed` runtime helper for the x86_64 Linux ABI.
///
/// # Arguments
/// * `emitter` - The assembly emitter to write instructions into.
///
/// # Behavior
/// - Skips null pointers and values below the managed heap base.
/// - Validates the pointer is within the live heap window and carries the
///   x86_64 heap magic marker in the high 32 bits of the header word.
/// - Decrements the refcount and leaves cycle collection to explicit safe points.
/// - Tail-calls `__rt_mixed_free_deep` on zero refcount.
///
/// # ABI
/// - Input pointer in `rax`; clobbers `r10`, `r11`; returns via `ret`.
fn emit_decref_mixed_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: decref_mixed ---");
    emitter.label_global("__rt_decref_mixed");

    emitter.instruction("test rax, rax");                                       // skip null mixed pointers immediately because they do not own heap storage
    emitter.instruction("jz __rt_decref_mixed_skip");                           // null mixed values need no release work
    crate::codegen_support::abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("lea r10, [r10 + 16]");                                 // first valid user payload begins after the initial heap header
    emitter.instruction("cmp rax, r10");                                        // reject null sentinels, scalar values, and static pointers before reading a heap header
    emitter.instruction("jb __rt_decref_mixed_skip");                           // non-heap values below the managed heap do not own mixed-box storage
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_heap_off");
    emitter.instruction("mov r11, QWORD PTR [r11]");                            // load the current x86_64 heap bump extent before deriving the live heap end
    crate::codegen_support::abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("add r11, r10");                                        // compute the managed heap end address from the base and live offset
    emitter.instruction("cmp rax, r11");                                        // is the candidate mixed pointer outside the live heap window?
    emitter.instruction("jae __rt_decref_mixed_skip");                          // pointers above the live heap end are not refcounted mixed boxes
    emitter.instruction("mov r10, QWORD PTR [rax - 8]");                        // load the stamped x86_64 heap kind word from the uniform header
    emitter.instruction("shr r10, 32");                                         // isolate the high-word heap marker used by the x86_64 heap wrapper
    emitter.instruction(&format!("cmp r10d, 0x{:x}", crate::codegen_support::sentinels::X86_64_HEAP_MAGIC_HI32)); // ignore foreign pointers that do not carry the elephc x86_64 heap marker
    emitter.instruction("jne __rt_decref_mixed_skip");                          // only elephc-owned mixed boxes participate in x86_64 decref bookkeeping
    emitter.instruction("mov r10d, DWORD PTR [rax - 12]");                      // load the 32-bit mixed-box refcount from the uniform heap header
    emitter.instruction("sub r10d, 1");                                         // decrement the mixed-box refcount for the releasing x86_64 owner
    emitter.instruction("mov DWORD PTR [rax - 12], r10d");                      // store the decremented mixed-box refcount back into the uniform heap header
    emitter.instruction("jz __rt_decref_mixed_free");                           // zero refcount means the boxed payload can be released now
    emitter.instruction("jmp __rt_decref_mixed_skip");                          // non-zero refcount stays alive until an explicit GC safe point
    emitter.label("__rt_decref_mixed_skip");
    emitter.instruction("ret");                                                 // nothing else needs to happen for non-zero refcounts or foreign pointers

    emitter.label("__rt_decref_mixed_free");
    emit_scalar_box_fast_free(emitter);
    emitter.instruction("jmp __rt_mixed_free_deep");                            // tail-call to deep free the mixed box once the last owner is gone
}

/// Emits the release fast path for a dead box holding an int/float/bool/null (no child to
/// release) in a 24..=31-byte block, with heap debugging off: the block is accounted exactly
/// like `__rt_heap_free` does, then either shrinks the bump tail or joins the 24-byte class
/// bin. Every other box falls through to `__rt_mixed_free_deep`. Input: the box in x0/rax,
/// refcount already zero. Clobbers only registers `__rt_heap_free` clobbers.
fn emit_scalar_box_fast_free(emitter: &mut Emitter) {
    use crate::codegen_support::abi::emit_symbol_address;
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x10, [x0]");                               // runtime tag of the dead box
            emitter.instruction("cmp x10, #1");                                 // strings own a payload
            emitter.instruction("b.eq __rt_decref_mixed_deep");
            emitter.instruction("cmp x10, #4");                                 // int/float/bool own nothing
            emitter.instruction("b.lo __rt_decref_mixed_scalar");
            emitter.instruction("cmp x10, #8");                                 // null owns nothing either
            emitter.instruction("b.ne __rt_decref_mixed_deep");
            emitter.label("__rt_decref_mixed_scalar");
            emit_symbol_address(emitter, "x10", "_heap_debug_enabled");
            emitter.instruction("ldr x10, [x10]");                              // heap-debug mode poisons and validates frees
            emitter.instruction("cbnz x10, __rt_decref_mixed_deep");
            emitter.instruction("ldr w11, [x0, #-16]");                         // block payload size
            emitter.instruction("sub w10, w11, #24");                           // only 24..31-byte blocks use the 24-byte bin
            emitter.instruction("cmp w10, #7");
            emitter.instruction("b.hi __rt_decref_mixed_deep");
            emitter.instruction("sub x12, x0, #16");                            // block header
            emitter.instruction("str xzr, [x12, #8]");                          // clear the heap kind of the free block
            emitter.instruction("add x13, x11, #16");                           // footprint with the header
            emit_symbol_address(emitter, "x10", "_gc_live");
            emitter.instruction("ldr x14, [x10]");                              // live bytes
            emitter.instruction("sub x14, x14, x13");
            emitter.instruction("str x14, [x10]");
            emit_symbol_address(emitter, "x10", "_gc_frees");
            emitter.instruction("ldr x14, [x10]");                              // count the free
            emitter.instruction("add x14, x14, #1");
            emitter.instruction("str x14, [x10]");
            emit_symbol_address(emitter, "x10", "_heap_buf");
            emit_symbol_address(emitter, "x13", "_heap_off");
            emitter.instruction("ldr x14, [x13]");                              // bump offset
            emitter.instruction("add x14, x10, x14");                           // heap end
            emitter.instruction("add x15, x0, x11");                            // end of the freed block
            emitter.instruction("cmp x15, x14");                                // is it the tail block?
            emitter.instruction("b.ne __rt_decref_mixed_bin");
            emitter.instruction("sub x14, x12, x10");                           // shrink the bump tail to the block
            emitter.instruction("str x14, [x13]");
            emitter.instruction("ret");
            emitter.label("__rt_decref_mixed_bin");
            emit_symbol_address(emitter, "x10", "_heap_small_bins");
            emitter.instruction("ldr x14, [x10, #16]");                         // previous head of the 24-byte bin
            emitter.instruction("str x14, [x12, #16]");                         // link it behind the freed block
            emitter.instruction("str x12, [x10, #16]");                         // publish the freed block
            emitter.instruction("ret");
            emitter.label("__rt_decref_mixed_deep");
        }
        Arch::X86_64 => {
            emitter.instruction("mov r10, QWORD PTR [rax]");                    // runtime tag of the dead box
            emitter.instruction("cmp r10, 1");                                  // strings own a payload
            emitter.instruction("je __rt_decref_mixed_deep");
            emitter.instruction("cmp r10, 4");                                  // int/float/bool own nothing
            emitter.instruction("jb __rt_decref_mixed_scalar");
            emitter.instruction("cmp r10, 8");                                  // null owns nothing either
            emitter.instruction("jne __rt_decref_mixed_deep");
            emitter.label("__rt_decref_mixed_scalar");
            emit_symbol_address(emitter, "r8", "_heap_debug_enabled");
            emitter.instruction("cmp QWORD PTR [r8], 0");                       // heap-debug mode poisons and validates frees
            emitter.instruction("jne __rt_decref_mixed_deep");
            emitter.instruction("mov r11d, DWORD PTR [rax - 16]");              // block payload size
            emitter.instruction("lea r10, [r11 - 24]");                         // only 24..31-byte blocks use the 24-byte bin
            emitter.instruction("cmp r10, 7");
            emitter.instruction("ja __rt_decref_mixed_deep");
            emitter.instruction("lea r9, [rax - 16]");                          // block header
            emitter.instruction("mov QWORD PTR [r9 + 8], 0");                   // clear the heap kind of the free block
            emitter.instruction("lea rcx, [r11 + 16]");                         // footprint with the header
            emit_symbol_address(emitter, "r8", "_gc_live");
            emitter.instruction("sub QWORD PTR [r8], rcx");                     // live bytes
            emit_symbol_address(emitter, "r8", "_gc_frees");
            emitter.instruction("add QWORD PTR [r8], 1");                       // count the free
            emit_symbol_address(emitter, "r10", "_heap_buf");
            emit_symbol_address(emitter, "r8", "_heap_off");
            emitter.instruction("mov rcx, QWORD PTR [r8]");                     // bump offset
            emitter.instruction("add rcx, r10");                                // heap end
            emitter.instruction("lea rdx, [rax + r11]");                        // end of the freed block
            emitter.instruction("cmp rdx, rcx");                                // is it the tail block?
            emitter.instruction("jne __rt_decref_mixed_bin");
            emitter.instruction("mov rdx, r9");                                 // shrink the bump tail to the block
            emitter.instruction("sub rdx, r10");
            emitter.instruction("mov QWORD PTR [r8], rdx");
            emitter.instruction("ret");
            emitter.label("__rt_decref_mixed_bin");
            emit_symbol_address(emitter, "r10", "_heap_small_bins");
            emitter.instruction("mov rdx, QWORD PTR [r10 + 16]");               // previous head of the 24-byte bin
            emitter.instruction("mov QWORD PTR [r9 + 16], rdx");                // link it behind the freed block
            emitter.instruction("mov QWORD PTR [r10 + 16], r9");                // publish the freed block
            emitter.instruction("ret");
            emitter.label("__rt_decref_mixed_deep");
        }
    }
}
