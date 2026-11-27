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

    emitter.instruction("b __rt_gc_note_candidate");                            // non-zero refcount stays alive until an explicit GC safe point

    emitter.label("__rt_decref_mixed_free");
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
    emitter.instruction("jmp __rt_gc_note_candidate");                          // non-zero refcount stays alive until an explicit GC safe point
    emitter.label("__rt_decref_mixed_skip");
    emitter.instruction("ret");                                                 // nothing else needs to happen for non-zero refcounts or foreign pointers

    emitter.label("__rt_decref_mixed_free");
    emit_scalar_cell_cache_push_x86_64(emitter);
    emitter.instruction("jmp __rt_mixed_free_deep");                            // tail-call to deep free the mixed box once the last owner is gone
}

/// Parks a dead scalar Mixed cell in the small reuse cache instead of freeing it (x86_64).
///
/// Scalar cells (int, float, bool, null) own no child, so their final release is only the
/// block free; boxing loops pay a full allocator round trip per value. Up to
/// `MIXED_CELL_CACHE_LIMIT` such cells are kept on a singly linked list threaded through their
/// first payload word and handed back by `__rt_mixed_from_value`. A parked cell keeps a zero
/// owner count and gets a zero kind word, so a stale release of it is ignored like a release of
/// a freed block, and the collector's candidate validation rejects it. Parking and reuse update
/// `_gc_frees`/`_gc_allocs`/`_gc_live`/`_gc_peak` exactly like a free and an allocation, so
/// `--gc-stats` and `memory_get_usage()` see the same totals as without the cache. The cache is bypassed in
/// `--heap-debug` builds (exact alloc/free accounting), while eval tracks array-reference
/// metadata by address (`heap_free` must retire it) and while the collector frees an
/// unreachable graph; `--web` arena resets forget the cache with the arena; only full-size cells whose kind word carries no collector metadata qualify.
/// Input: rax = cell whose owner count just reached zero. Falls through when the cell is not cached.
fn emit_scalar_cell_cache_push_x86_64(emitter: &mut Emitter) {
    emitter.instruction("mov r10, QWORD PTR [rax]");                            // inspect the boxed runtime value tag
    emitter.instruction("cmp r10, 1");                                          // strings own a payload block
    emitter.instruction("je __rt_decref_mixed_cache_skip");                     // release them through the deep free
    emitter.instruction("cmp r10, 4");                                          // tags 0, 2 and 3 are int, float and bool
    emitter.instruction("jb __rt_decref_mixed_cache_scalar");                   // scalar payloads own nothing
    emitter.instruction("cmp r10, 8");                                          // tag 8 is null
    emitter.instruction("jne __rt_decref_mixed_cache_skip");                    // every other tag may own a child
    emitter.label("__rt_decref_mixed_cache_scalar");
    emitter.instruction("cmp DWORD PTR [rax - 16], 24");                        // only complete three-word cells are reusable
    emitter.instruction("jb __rt_decref_mixed_cache_skip");                     // short storage goes back to the allocator
    emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(5))); // the plain stamped Mixed kind word
    emitter.instruction("cmp QWORD PTR [rax - 8], r10");                        // any collector metadata bit set?
    emitter.instruction("jne __rt_decref_mixed_cache_skip");                    // leave marked cells to the ordinary free
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_heap_debug_enabled");
    emitter.instruction("cmp QWORD PTR [r11], 0");                              // heap-debug builds keep exact allocation accounting
    emitter.instruction("jne __rt_decref_mixed_cache_skip");                    // free normally under --heap-debug
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_elephc_eval_array_reference_retire_fn");
    emitter.instruction("cmp QWORD PTR [r11], 0");                              // eval may track array-reference metadata by cell address
    emitter.instruction("jne __rt_decref_mixed_cache_skip");                    // let heap_free run its retirement callback
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_gc_freeing_unreachable");
    emitter.instruction("cmp QWORD PTR [r11], 0");                              // is the collector freeing an unreachable graph?
    emitter.instruction("jne __rt_decref_mixed_cache_skip");                    // its sweep owns every block it frees
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_mixed_cell_cache_count");
    emitter.instruction(&format!("cmp QWORD PTR [r11], {MIXED_CELL_CACHE_LIMIT}")); // is the reuse cache full?
    emitter.instruction("jae __rt_decref_mixed_cache_skip");                    // a full cache frees the cell normally
    emitter.instruction("add QWORD PTR [r11], 1");                              // account for the parked cell
    // -- keep the allocator's accounting exact: a parked cell counts as freed --
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_gc_frees");
    emitter.instruction("add QWORD PTR [r11], 1");                              // count the release like __rt_heap_free does
    emitter.instruction("mov r10d, DWORD PTR [rax - 16]");                      // payload size of the parked block
    emitter.instruction("add r10, 16");                                         // include the uniform header in the live footprint
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_gc_live");
    emitter.instruction("sub QWORD PTR [r11], r10");                            // the parked block no longer counts as live
    emitter.instruction("mov QWORD PTR [rax - 8], 0");                          // unstamp it so stale releases treat it as foreign
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_mixed_cell_cache_head");
    emitter.instruction("mov r10, QWORD PTR [r11]");                            // current cache head
    emitter.instruction("mov QWORD PTR [rax], r10");                            // thread the list through the first payload word
    emitter.instruction("mov QWORD PTR [r11], rax");                            // publish the parked cell as the new head
    emitter.instruction("ret");                                                 // the cell is retired without an allocator call
    emitter.label("__rt_decref_mixed_cache_skip");
}

/// Upper bound on scalar Mixed cells parked for reuse (see `emit_scalar_cell_cache_push_x86_64`).
pub(crate) const MIXED_CELL_CACHE_LIMIT: u32 = 64;
