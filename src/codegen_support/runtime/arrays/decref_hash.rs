//! Purpose:
//! Emits the `__rt_decref_hash`, `__rt_decref_hash_skip` runtime helper assembly for decref hash.
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


/// Emits the `__rt_decref_hash` runtime helper.
///
/// Decrements the refcount of a heap-allocated PHP hash table.
/// Input: hash pointer in `x0` (ARM64) or `rax` (x86_64).
/// Behavior:
/// - Skips null, non-heap, and sentinel pointers immediately.
/// - Decrements refcount; if zero, tail-calls `__rt_hash_free_deep` for deep release.
/// - If refcount remains non-zero, returns without invoking cycle collection.
/// - On x86_64, additionally validates the heap magic marker before any operation.
///
/// ABI: preserves all caller-saved registers except `x0`/`rax` used for the optional
/// collector return value when a collection pass runs.
pub fn emit_decref_hash(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_decref_hash_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: decref_hash ---");
    emitter.label_global("__rt_decref_hash");

    // -- null check --
    emitter.instruction("cbz x0, __rt_decref_hash_skip");                       // skip if null pointer

    // -- heap range check: x0 >= _heap_buf --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_buf");
    emitter.instruction("cmp x0, x9");                                          // is pointer below heap start?
    emitter.instruction("b.lo __rt_decref_hash_skip");                          // yes — not a heap pointer, skip

    // -- heap range check: x0 < _heap_buf + _heap_off --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_heap_off");
    emitter.instruction("ldr x10, [x10]");                                      // x10 = current heap offset
    emitter.instruction("add x10, x9, x10");                                    // x10 = heap_buf + heap_off = heap end
    emitter.instruction("cmp x0, x10");                                         // is pointer at or beyond heap end?
    emitter.instruction("b.hs __rt_decref_hash_skip");                          // yes — not a valid heap pointer, skip

    // -- debug mode: reject decref on freed storage --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_debug_enabled");
    emitter.instruction("ldr x9, [x9]");                                        // load the heap-debug enabled flag
    emitter.instruction("cbz x9, __rt_decref_hash_checked");                    // skip debug validation when heap-debug mode is disabled
    emitter.instruction("str x30, [sp, #-16]!");                                // preserve the caller return address before nested validation
    emitter.instruction("bl __rt_heap_debug_check_live");                       // ensure the hash block still has a live refcount
    emitter.instruction("ldr x30, [sp], #16");                                  // restore the caller return address after validation
    emitter.label("__rt_decref_hash_checked");

    // -- decrement refcount and check for zero --
    emitter.instruction("ldr w9, [x0, #-12]");                                  // load 32-bit refcount from the uniform heap header
    emitter.instruction("subs w9, w9, #1");                                     // decrement refcount, set flags
    emitter.instruction("str w9, [x0, #-12]");                                  // store decremented refcount
    emitter.instruction("b.eq __rt_decref_hash_free");                          // zero refcount means the hash can be freed immediately

    emitter.instruction("b __rt_decref_hash_skip");                             // non-zero refcount stays alive until an explicit GC safe point

    // -- refcount reached zero: deep free the hash table and its owned entries --
    emitter.label("__rt_decref_hash_free");
    emitter.instruction("b __rt_hash_free_deep");                               // tail-call to deep free hash keys and heap-backed values

    emitter.label("__rt_decref_hash_skip");
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the x86_64 Linux-specific path of `__rt_decref_hash`.
///
/// Target-specific lowering for x86_64; called from `emit_decref_hash` when
/// `emitter.target.arch == Arch::X86_64`.
/// Input: hash pointer in `rax`.
/// Differences from ARM64:
/// - Uses the x86_64 heap magic marker in the high 32 bits of the header word to
///   distinguish foreign pointers from eligphc-owned hash tables.
/// - Does not use a frame pointer or callee-saved registers; preserves `rbx`, `r12`–`r15`.
/// - `__rt_hash_free_deep` is reached through a tail jump.
fn emit_decref_hash_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: decref_hash ---");
    emitter.label_global("__rt_decref_hash");

    super::emit_x86_64_heap_owner_guard(emitter, "__rt_decref_hash_skip", 16);
    emitter.instruction("sub DWORD PTR [rax - 12], 1");                         // drop one owner from the 32-bit refcount in the uniform heap header
    emitter.instruction("jz __rt_decref_hash_free");                            // zero refcount means the hash and its owned entries can be released now
    emitter.instruction("jmp __rt_decref_hash_skip");                           // non-zero refcount stays alive until an explicit GC safe point
    emitter.label("__rt_decref_hash_skip");
    emitter.instruction("ret");                                                 // nothing else needs to happen for non-zero refcounts or foreign pointers

    emitter.label("__rt_decref_hash_free");
    emitter.instruction("jmp __rt_hash_free_deep");                             // tail-call to deep free the hash table once the last owner is gone
}
