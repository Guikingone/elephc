//! Purpose:
//! Emits the `__rt_heap_alloc`, `__rt_heap_alloc_start` runtime helper assembly for heap alloc.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Heap helpers own allocator metadata, debug accounting, and free-list invariants used by all refcounted runtime values.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::sentinels::HEAP_FREE_REFCOUNT_MARK;

/// Largest payload, in bytes, that a freed block is parked for by exact size class.
///
/// Payloads are 16-byte multiples, so the classes are 16, 32, …, this value. A request up to it
/// is answered by popping its class; only larger requests, and requests whose class is empty,
/// walk the ordered free list.
pub(crate) const HEAP_SIZE_CLASS_MAX: usize = 1024;

/// Number of exact size classes, one head pointer each in `_heap_small_bins`.
pub(crate) const HEAP_SIZE_CLASS_COUNT: usize = HEAP_SIZE_CLASS_MAX / 16;

/// Emits the `__rt_heap_alloc` runtime helper: a free-list allocator with exact size-class
/// caching and a bump-pointer fallback.
///
/// Allocation path: size class (≤`HEAP_SIZE_CLASS_MAX` bytes) → ordered free list → bump
/// pointer → defragment and retry → exhaustion. x86_64 follows the same steps in
/// `emit_heap_alloc_linux_x86_64`.
///
/// Each block carries a 16-byte header `[size:4][refcount:4][kind:8]` before the user pointer.
/// Free blocks reuse the same header layout plus a `next_ptr:8` word for list chaining.
///
/// WHY CLASSES. The ordered list was the only home for a freed block above 64 bytes, and
/// first-fit walks it from the lowest address, where the small leftovers collect. On a Symfony
/// request that walk was the hottest code in the process: `__rt_heap_alloc` 13.6% of all
/// samples, most of them on the per-node bounds reload inside the loop. An exact class makes the
/// common request one pop, and keeps small blocks out of the list the large ones are walked in.
///
/// Parked class blocks do not coalesce, so they can hold memory a larger request needs. Before
/// the allocator reports exhaustion it therefore defragments: one pass over the arena merges
/// every run of adjacent parked blocks into the ordered list and empties the classes, and the
/// request is retried. Caching can delay coalescing, never prevent it.
///
/// A class pop validates its block with the same discipline as the general free list: the
/// header must lie inside the live heap window (else the chain is dropped, since its next link
/// cannot be trusted), and a parked block must carry the free mark, no retained `kind`, and a
/// size its class admits (else it is unlinked as poison). Under `--heap-debug` the same poison
/// is caught loudly at alloc entry by `__rt_heap_debug_validate_free_list`.
///
/// Input: `x0` (ARM) / `rax` (x86_64) = requested payload bytes (rounded up to 16).
/// Output: `x0` / `rax` = user pointer (header + 16).
///
/// Updates `_gc_allocs`, `_gc_live`, and `_gc_peak` counters on every allocation.
/// On heap exhaustion, a cdylib call with an active native boundary unwinds
/// with `STATUS_ALLOCATION_FAILURE`; ordinary executables retain the fatal exit.
pub fn emit_heap_alloc(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_heap_alloc_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: heap_alloc (free-list + bump) ---");
    emitter.label_global("__rt_heap_alloc");

    // -- normalize every payload to the runtime's 16-byte allocation alignment --
    emitter.instruction("cmp x0, #8");                                          // is requested size < 8?
    emitter.instruction("b.ge __rt_heap_alloc_start");                          // skip if already >= 8
    emitter.instruction("mov x0, #8");                                          // round up to minimum 8 bytes
    emitter.label("__rt_heap_alloc_start");
    emitter.instruction("add x0, x0, #15");                                     // reserve room to round the payload up to the next 16-byte boundary
    emitter.instruction("lsr x0, x0, #4");                                      // divide by the 16-byte runtime allocation alignment
    emitter.instruction("lsl x0, x0, #4");                                      // restore the aligned payload byte count before any heap pointer arithmetic
    emitter.instruction("lsr x9, x0, #32");                                     // inspect bits the 32-bit block-size header cannot represent
    emitter.instruction("cbnz x9, __rt_heap_alloc_size_overflow");              // reject unrepresentable payload sizes before truncating metadata

    // -- debug mode: validate the free list before consuming it --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_debug_enabled");
    emitter.instruction("ldr x9, [x9]");                                        // load the heap-debug enabled flag
    emitter.instruction("cbz x9, __rt_heap_alloc_debug_checked");               // skip validation when heap-debug mode is disabled
    emitter.instruction("stp x0, x30, [sp, #-16]!");                            // preserve allocation size and return address across validation
    emitter.instruction("bl __rt_heap_debug_validate_free_list");               // verify the ordered free list before searching it
    emitter.instruction("ldp x0, x30, [sp], #16");                              // restore allocation size and return address after validation
    emitter.label("__rt_heap_alloc_debug_checked");

    // -- an exact size class answers most requests with one pop --
    // Class k holds parked blocks whose payload is at least 16*(k+1) and below 16*(k+2), so its
    // head fits any request that rounds to 16*(k+1) and no search is needed.
    emitter.instruction(&format!("cmp x0, #{}", HEAP_SIZE_CLASS_MAX));          // is the request small enough for an exact size class?
    emitter.instruction("b.hi __rt_heap_alloc_fl_start");                       // larger requests use the general free list
    emitter.instruction("lsr x13, x0, #4");                                     // x13 = request in 16-byte units, at least 1 after rounding
    emitter.instruction("sub x13, x13, #1");                                    // x13 = the request's size-class index
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_small_bins");
    emitter.instruction("add x9, x9, x13, lsl #3");                             // x9 = address of this class's head slot
    emitter.label("__rt_heap_alloc_class_pop");
    emitter.instruction("ldr x10, [x9]");                                       // x10 = the class's first parked block, or null when it is empty
    emitter.instruction("cbz x10, __rt_heap_alloc_fl_start");                   // nothing parked in this class: fall back to the general free list
    // -- reject a head that escaped the live heap window before dereferencing it --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_heap_buf");
    emitter.instruction("cmp x10, x12");                                        // does the cached block point below the heap buffer base?
    emitter.instruction("b.lo __rt_heap_alloc_class_drop");                     // wild pointer: drop the chain, its next link cannot be trusted
    crate::codegen_support::abi::emit_symbol_address(emitter, "x14", "_heap_off");
    emitter.instruction("ldr x14, [x14]");                                      // load the current heap bump offset before deriving the live heap end
    emitter.instruction("add x14, x12, x14");                                   // x14 = current live heap end
    emitter.instruction("cmp x10, x14");                                        // does the cached block point at or beyond the live heap end?
    emitter.instruction("b.hs __rt_heap_alloc_class_drop");                     // wild pointer: drop the chain past the live heap window
    // -- a parked block carries the free mark, no retained heap kind, and a size of its class --
    emitter.instruction("ldr w15, [x10, #4]");                                  // load the cached block refcount from its header
    super::heap_free::emit_load_free_mark(emitter, "w11");                      // materialize the parked-block refcount marker
    emitter.instruction("cmp w15, w11");                                        // does this entry still claim to be parked?
    emitter.instruction("b.ne __rt_heap_alloc_class_unlink_invalid");           // anything else marks a poisoned entry, unlink it
    emitter.instruction("ldr x15, [x10, #8]");                                  // load the cached block heap kind from its header
    emitter.instruction("cbnz x15, __rt_heap_alloc_class_unlink_invalid");      // a retained live kind marks a poisoned entry, unlink it
    emitter.instruction("ldr w11, [x10]");                                      // load the cached block payload size before reusing it
    emitter.instruction("cmp x11, x0");                                         // is the cached block at least as large as the request?
    emitter.instruction("b.lo __rt_heap_alloc_class_unlink_invalid");           // a block too small for its own class is poison, unlink it
    emitter.instruction("add x15, x10, #16");                                   // x15 = start of this cached block payload
    emitter.instruction("add x15, x15, x11");                                   // x15 = cached block claimed end address
    emitter.instruction("cmp x15, x14");                                        // does the cached block stay inside the live heap window?
    emitter.instruction("b.hi __rt_heap_alloc_class_unlink_invalid");           // unlink cached entries whose recorded size overruns the live heap
    emitter.instruction("ldr x11, [x10, #16]");                                 // x11 = the next parked block of this class
    emitter.instruction("str x11, [x9]");                                       // pop the head off its size class
    emitter.instruction("mov w13, #1");                                         // initial refcount = 1 for the reused block
    emitter.instruction("str w13, [x10, #4]");                                  // restore the live refcount in the reused header
    emitter.instruction("str xzr, [x10, #8]");                                  // reset heap kind to raw until a typed constructor overwrites it
    emitter.instruction("add x0, x10, #16");                                    // return user pointer = header + 16
    emitter.instruction("b __rt_heap_alloc_count");                             // reuse the shared allocation-accounting path

    emitter.label("__rt_heap_alloc_class_drop");
    emitter.instruction("str xzr, [x9]");                                       // drop the unreachable chain so no later pop follows the wild pointer
    emitter.instruction("b __rt_heap_alloc_fl_start");                          // this class is empty from here on, use the general free list
    emitter.label("__rt_heap_alloc_class_unlink_invalid");
    emitter.instruction("ldr x15, [x10, #16]");                                 // the poisoned block is in-heap, so its next link is a safe load
    emitter.instruction("str x15, [x9]");                                       // unlink the poisoned block from its size class
    emitter.instruction("b __rt_heap_alloc_class_pop");                         // try the class's next parked block

    // -- walk the general free list looking for first-fit block --
    // x0 = requested size, x9 = prev_next_addr, x10 = current block header,
    // x12 = heap buffer base, x13 = live heap end (neither moves during the walk)
    emitter.label("__rt_heap_alloc_fl_start");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_heap_buf");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x13", "_heap_off");
    emitter.instruction("ldr x13, [x13]");                                      // load the current heap bump offset once for the whole walk
    emitter.instruction("add x13, x12, x13");                                   // x13 = current live heap end
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_free_list");
    emitter.instruction("ldr x10, [x9]");                                       // x10 = first free block header (0 if empty)

    // -- walk the free list looking for first-fit block --
    emitter.label("__rt_heap_alloc_fl_loop");
    emitter.instruction("cbz x10, __rt_heap_alloc_bump");                       // no free block found, fall through to bump
    emitter.instruction("cmp x10, x12");                                        // reject free-list pointers that point before the heap buffer
    emitter.instruction("b.lo __rt_heap_alloc_fl_drop_tail");                   // drop the rest of a chain once it leaves the heap buffer
    emitter.instruction("cmp x10, x13");                                        // reject free-list pointers at or beyond the live heap end
    emitter.instruction("b.hs __rt_heap_alloc_fl_drop_tail");                   // truncate a chain that has escaped the live heap window
    emitter.instruction("ldr w11, [x10]");                                      // x11 = block size (32-bit, zero-extends)
    emitter.instruction("cmp x11, #8");                                         // is the free block large enough to carry allocator metadata?
    emitter.instruction("b.lo __rt_heap_alloc_fl_unlink_invalid");              // unlink in-heap nodes with impossible payload sizes
    emitter.instruction("ldr w14, [x10, #4]");                                  // load the candidate refcount from the free block header
    super::heap_free::emit_load_free_mark(emitter, "w15");                      // materialize the parked-block refcount marker
    emitter.instruction("cmp w14, w15");                                        // does this node still claim to be parked on the free list?
    emitter.instruction("b.ne __rt_heap_alloc_fl_unlink_invalid");              // live or corrupt blocks must never be reused through the free list
    emitter.instruction("ldr x14, [x10, #8]");                                  // load the candidate heap kind from the free block header
    emitter.instruction("cbnz x14, __rt_heap_alloc_fl_unlink_invalid");         // free-list blocks must not retain a live heap kind
    emitter.instruction("add x14, x10, #16");                                   // compute the start of this candidate payload
    emitter.instruction("add x14, x14, x11");                                   // compute the candidate block end address
    emitter.instruction("cmp x14, x13");                                        // does the candidate block stay inside the live heap window?
    emitter.instruction("b.hi __rt_heap_alloc_fl_unlink_invalid");              // unlink in-heap nodes whose recorded size overruns live heap
    emitter.instruction("cmp x11, x0");                                         // does this block fit the request?
    emitter.instruction("b.ge __rt_heap_alloc_fl_found");                       // yes — use this block

    // -- advance to next free block --
    emitter.instruction("add x9, x10, #16");                                    // prev_next_addr = &current->next after the 16-byte free-block header
    emitter.instruction("ldr x10, [x10, #16]");                                 // current = current->next
    emitter.instruction("b __rt_heap_alloc_fl_loop");                           // continue searching

    // -- unlink malformed in-heap nodes instead of allocating from stale state --
    emitter.label("__rt_heap_alloc_fl_unlink_invalid");
    emitter.instruction("ldr x12, [x10, #16]");                                 // preserve the next free-list pointer before removing this malformed node
    emitter.instruction("str x12, [x9]");                                       // unlink the malformed in-heap node from the free-list chain
    emitter.instruction("mov x10, x12");                                        // continue scanning from the successor node
    emitter.instruction("b __rt_heap_alloc_fl_loop");                           // keep searching for a valid reusable free block

    // -- truncate chains that have escaped the heap address range --
    emitter.label("__rt_heap_alloc_fl_drop_tail");
    emitter.instruction("str xzr, [x9]");                                       // drop the unreachable tail without dereferencing an out-of-heap pointer
    emitter.instruction("mov x10, xzr");                                        // force the next loop iteration to use bump allocation
    emitter.instruction("b __rt_heap_alloc_fl_loop");                           // restart the loop with the truncated chain

    // -- found a suitable free block, either split it or unlink it whole --
    emitter.label("__rt_heap_alloc_fl_found");
    emitter.instruction("sub x12, x11, x0");                                    // x12 = free block payload minus requested payload
    emitter.instruction("cmp x12, #24");                                        // is there room for a new 16-byte header plus minimum payload?
    emitter.instruction("b.lt __rt_heap_alloc_fl_take_whole");                  // no — consume the whole free block
    emitter.instruction("add x13, x10, x0");                                    // x13 = current header + requested payload
    emitter.instruction("add x13, x13, #16");                                   // x13 = split remainder header address
    emitter.instruction("sub x12, x12, #16");                                   // x12 = remainder payload size after carving out a new header
    emitter.instruction("str w12, [x13]");                                      // write split remainder size into its header
    super::heap_free::emit_load_free_mark(emitter, "w15");                      // materialize the parked-block refcount marker
    emitter.instruction("str w15, [x13, #4]");                                  // the split remainder stays parked, so it carries the free mark
    emitter.instruction("str xzr, [x13, #8]");                                  // free remainder has no heap kind while on the free list
    emitter.instruction("ldr x14, [x10, #16]");                                 // x14 = current->next before splitting
    // A remainder small enough for a size class is parked there: left in the ordered list it
    // would sit in front of every later walk, which is the cost the classes exist to remove.
    emitter.instruction("cmp x12, #16");                                        // is the remainder below the smallest size class?
    emitter.instruction("b.lo __rt_heap_alloc_split_to_list");                  // yes — keep it in the ordered free list
    emitter.instruction(&format!("cmp x12, #{}", HEAP_SIZE_CLASS_MAX));         // is the remainder above the largest size class?
    emitter.instruction("b.hi __rt_heap_alloc_split_to_list");                  // yes — keep it in the ordered free list
    emitter.instruction("str x14, [x9]");                                       // unlink the matched block: prev->next = current->next
    emitter.instruction("lsr x14, x12, #4");                                    // x14 = remainder payload in 16-byte units
    emitter.instruction("sub x14, x14, #1");                                    // x14 = the remainder's size-class index
    crate::codegen_support::abi::emit_symbol_address(emitter, "x15", "_heap_small_bins");
    emitter.instruction("add x15, x15, x14, lsl #3");                           // x15 = address of the remainder's class head slot
    emitter.instruction("ldr x14, [x15]");                                      // x14 = the class's current head
    emitter.instruction("str x14, [x13, #16]");                                 // remainder->next = previous class head
    emitter.instruction("str x13, [x15]");                                      // park the remainder at the head of its class
    emitter.instruction("b __rt_heap_alloc_split_parked");                      // the remainder is parked; finish the reused block
    emitter.label("__rt_heap_alloc_split_to_list");
    emitter.instruction("str x14, [x13, #16]");                                 // remainder->next = current->next
    emitter.instruction("str x13, [x9]");                                       // prev->next = remainder header
    emitter.label("__rt_heap_alloc_split_parked");
    emitter.instruction("str w0, [x10]");                                       // shrink allocated block header size to the requested payload
    emitter.instruction("mov w13, #1");                                         // initial refcount = 1
    emitter.instruction("str w13, [x10, #4]");                                  // reset refcount in reused header
    emitter.instruction("str xzr, [x10, #8]");                                  // reset heap kind to raw until a typed constructor overwrites it
    emitter.instruction("add x0, x10, #16");                                    // return user pointer = header + 16
    emitter.instruction("b __rt_heap_alloc_count");                             // count allocation and return

    emitter.label("__rt_heap_alloc_fl_take_whole");
    emitter.instruction("ldr x12, [x10, #16]");                                 // x12 = current->next (rest of list)
    emitter.instruction("str x12, [x9]");                                       // prev->next = current->next (unlink current)
    emitter.instruction("mov w13, #1");                                         // initial refcount = 1
    emitter.instruction("str w13, [x10, #4]");                                  // reset refcount in reused header
    emitter.instruction("str xzr, [x10, #8]");                                  // reset heap kind to raw until a typed constructor overwrites it
    emitter.instruction("add x0, x10, #16");                                    // return user pointer = header + 16

    emitter.label("__rt_heap_alloc_count");
    // -- increment gc_allocs counter --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_gc_allocs");
    emitter.instruction("ldr x13, [x12]");                                      // load current count
    emitter.instruction("add x13, x13, #1");                                    // increment
    emitter.instruction("str x13, [x12]");                                      // store back
    // -- update current/peak live heap footprint --
    emitter.instruction("ldr w14, [x10]");                                      // load the allocated payload size from the finalized header
    emitter.instruction("add x14, x14, #16");                                   // include the 16-byte header in the live-footprint accounting
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_gc_live");
    emitter.instruction("ldr x13, [x12]");                                      // load current live bytes
    emitter.instruction("add x13, x13, x14");                                   // add this block's total footprint to live bytes
    emitter.instruction("str x13, [x12]");                                      // store updated live bytes
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_gc_peak");
    emitter.instruction("ldr x15, [x12]");                                      // load the previous live-byte high watermark
    emitter.instruction("cmp x13, x15");                                        // did this allocation raise the live-byte peak?
    emitter.instruction("csel x15, x13, x15, hi");                              // keep the larger of current live bytes and the previous peak
    emitter.instruction("str x15, [x12]");                                      // store the updated peak-live-bytes counter
    emitter.instruction("ret");                                                 // return to caller

    // -- no free block found, bump allocate with header --
    emitter.label("__rt_heap_alloc_bump");

    // -- load current heap offset --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_off");
    emitter.instruction("ldr x10, [x9]");                                       // x10 = current heap offset

    // -- bounds check: offset + 16 + requested <= heap_max --
    emitter.instruction("add x12, x10, x0");                                    // x12 = offset + requested size
    emitter.instruction("add x12, x12, #16");                                   // x12 = offset + requested + header (16 bytes)
    crate::codegen_support::abi::emit_symbol_address(emitter, "x13", "_heap_max");
    emitter.instruction("ldr x13, [x13]");                                      // x13 = heap max size in bytes
    emitter.instruction("cmp x12, x13");                                        // does the allocation fit (unsigned, so a wrapped size stays above the limit)?
    emitter.instruction("b.hi __rt_heap_alloc_defragment");                     // no — reclaim parked class blocks before giving up

    // -- compute base address of heap buffer --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_heap_buf");

    // -- write header and bump offset --
    emitter.instruction("add x14, x11, x10");                                   // x14 = buf + offset (header location)
    emitter.instruction("str w0, [x14]");                                       // write block size to header (32-bit)
    emitter.instruction("mov w15, #1");                                         // initial refcount = 1
    emitter.instruction("str w15, [x14, #4]");                                  // write refcount to header upper half
    emitter.instruction("str xzr, [x14, #8]");                                  // initialize heap kind to raw until a typed constructor overwrites it
    emitter.instruction("add x10, x10, x0");                                    // advance offset by requested size
    emitter.instruction("add x10, x10, #16");                                   // advance offset by header size
    emitter.instruction("str x10, [x9]");                                       // store updated offset to _heap_off
    emitter.instruction("add x0, x14, #16");                                    // return user pointer = header + 16
    emitter.instruction("mov x10, x14");                                        // reuse the common allocation-accounting path with the new block header pointer
    emitter.instruction("b __rt_heap_alloc_count");                             // count alloc/live/peak stats and return

    emit_heap_defragment_aarch64(emitter);

    // -- fatal error: heap memory exhausted --
    // `__rt_cstr`/`__rt_cstr2` are SEPARATE atoms and have to reach this block by name. Under
    // macOS dead stripping an ordinary `label()` is renamed to an `L`-local, which is not a
    // symbol at all, so that cross-atom branch is not a relocation the linker can follow: the
    // atom is collectable and the branch lands wherever the linker put the next one. The
    // `.alt_entry` alias keeps the block inside `__rt_heap_alloc`'s atom while giving the
    // cross-atom callers a real symbol; the `L`-local name stays for the intra-atom CONDITIONAL
    // branches, which older assemblers refuse to point at an `.alt_entry` label.
    emitter.label_shared("__rt_heap_exhausted_entry");
    emitter.label("__rt_heap_exhausted");
    if emitter.cdylib_boundary {
        crate::codegen_support::abi::emit_symbol_address(
            emitter,
            "x9",
            crate::codegen_support::cdylib::BOUNDARY_ACTIVE,
        );
        emitter.instruction("ldr x9, [x9]");                                    // read whether a native recovery boundary is active
        emitter.instruction("cbz x9, __rt_heap_exhausted_fatal");               // retain executable fatal behavior without a boundary
        crate::codegen_support::abi::emit_store_imm_to_symbol(
            emitter,
            crate::codegen_support::cdylib::BOUNDARY_STATUS,
            0,
            crate::codegen_support::cdylib::STATUS_ALLOCATION_FAILURE as i64,
        );
        crate::codegen_support::abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
        emitter.instruction("b __rt_throw_current");                            // unwind to the cdylib handler with allocation status recorded
        emitter.label("__rt_heap_exhausted_fatal");
    }
    emitter.instruction("stp x0, x30, [sp, #-16]!");                            // preserve the failed request and immediate caller return address across fatal writes
    emitter.instruction("mov x0, #2");                                          // fd = stderr
    crate::codegen_support::abi::emit_symbol_address(emitter, "x1", "_heap_err_msg");
    emitter.instruction("mov x2, #35");                                         // message length: "Fatal error: heap memory exhausted\n"
    emitter.syscall(4);
    emit_heap_exhaustion_stats(emitter);
    emitter.instruction("mov x0, #1");                                          // exit code 1
    emitter.syscall(1);

    emitter.label("__rt_heap_alloc_size_overflow");
    emitter.instruction("b __rt_heap_exhausted");                               // report an impossible header size through the established fatal path
}

/// Emits `__rt_heap_alloc_defragment`, the AArch64 allocator's last resort before exhaustion.
///
/// Reached with `x0` = the aligned request when the bump pointer cannot fit it. Parked
/// size-class blocks never coalesce, so a heap full of them can refuse a request that the same
/// memory would satisfy once merged. When any class holds a block, this walks the arena once
/// from `_heap_buf` to the live end, block by block through the headers (the same walk the cycle
/// collector makes), and rebuilds the ordered free list from scratch: every run of adjacent
/// parked blocks becomes ONE list node, in address order, and every class is emptied. A run that
/// reaches the live end goes back to the bump pointer. The request is then retried from the
/// general free list.
///
/// It runs at most once per request: a retry refills a class only by splitting a block, which
/// means the retry succeeded. With every class empty it goes straight to exhaustion.
fn emit_heap_defragment_aarch64(emitter: &mut Emitter) {
    emitter.label("__rt_heap_alloc_defragment");
    // -- only worth a pass when some class holds a parked block --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_small_bins");
    emitter.instruction("mov x10, #0");                                         // x10 = size-class index being probed
    emitter.label("__rt_heap_alloc_defrag_probe");
    emitter.instruction("ldr x11, [x9, x10, lsl #3]");                          // x11 = this class's head
    emitter.instruction("cbnz x11, __rt_heap_alloc_defrag_clear_start");        // a parked class block exists: defragmenting can help
    emitter.instruction("add x10, x10, #1");                                    // move on to the next size class
    emitter.instruction(&format!("cmp x10, #{}", HEAP_SIZE_CLASS_COUNT));       // have all size classes been probed?
    emitter.instruction("b.lo __rt_heap_alloc_defrag_probe");                   // no — keep probing
    emitter.instruction("b __rt_heap_exhausted");                               // every class is empty: the heap really is full

    // -- the arena walk rediscovers every parked block, so the classes start empty --
    emitter.label("__rt_heap_alloc_defrag_clear_start");
    emitter.instruction("mov x10, #0");                                         // x10 = size-class index being cleared
    emitter.label("__rt_heap_alloc_defrag_clear");
    emitter.instruction("str xzr, [x9, x10, lsl #3]");                          // empty this class
    emitter.instruction("add x10, x10, #1");                                    // move on to the next size class
    emitter.instruction(&format!("cmp x10, #{}", HEAP_SIZE_CLASS_COUNT));       // have all size classes been emptied?
    emitter.instruction("b.lo __rt_heap_alloc_defrag_clear");                   // no — keep clearing

    // -- walk the arena header by header, rebuilding the ordered list from its runs --
    // x10 = header being scanned, x13 = live heap end, x14 = link slot the next run is stored
    // through, x15 = header of the run the previous block extended (0 after a live block),
    // x16 = link slot that points at that run, w17 = the free mark
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_heap_buf");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x13", "_heap_off");
    emitter.instruction("ldr x13, [x13]");                                      // load the current heap bump offset
    emitter.instruction("add x13, x10, x13");                                   // x13 = live heap end
    crate::codegen_support::abi::emit_symbol_address(emitter, "x14", "_heap_free_list");
    emitter.instruction("str xzr, [x14]");                                      // the rebuilt list starts empty
    emitter.instruction("mov x16, x14");                                        // no run yet: its link slot is the list head
    emitter.instruction("mov x15, #0");                                         // no run is open before the first block
    super::heap_free::emit_load_free_mark(emitter, "w17");                      // materialize the parked-block refcount marker
    emitter.label("__rt_heap_alloc_defrag_scan");
    emitter.instruction("cmp x10, x13");                                        // has the walk reached the live heap end?
    emitter.instruction("b.hs __rt_heap_alloc_defrag_done");                    // yes — every block has been classified
    emitter.instruction("ldr w11, [x10]");                                      // x11 = this block's payload size
    emitter.instruction("add x9, x10, #16");                                    // x9 = this block's payload start
    emitter.instruction("add x9, x9, x11");                                     // x9 = the next block's header
    emitter.instruction("cmp x9, x13");                                         // does the recorded size stay inside the live heap?
    emitter.instruction("b.hi __rt_heap_alloc_defrag_done");                    // no — a corrupt header ends the walk; what was rebuilt stands
    emitter.instruction("ldr w12, [x10, #4]");                                  // load this block's refcount word
    emitter.instruction("cmp w12, w17");                                        // is the block parked?
    emitter.instruction("b.ne __rt_heap_alloc_defrag_live");                    // no — a live block closes any open run
    emitter.instruction("ldr x12, [x10, #8]");                                  // load this block's heap kind
    emitter.instruction("cbnz x12, __rt_heap_alloc_defrag_live");               // a parked block retains no kind: anything else is live
    emitter.instruction("cbz x15, __rt_heap_alloc_defrag_open_run");            // no run open: this block starts one
    emitter.instruction("sub x12, x9, x15");                                    // x12 = bytes from the run's header to this block's end
    emitter.instruction("sub x12, x12, #16");                                   // x12 = the run's payload once it absorbs this block
    emitter.instruction("str w12, [x15]");                                      // grow the open run over this block
    emitter.instruction("b __rt_heap_alloc_defrag_next");                       // continue with the next block
    emitter.label("__rt_heap_alloc_defrag_open_run");
    emitter.instruction("str x10, [x14]");                                      // link this run after the previous one, in address order
    emitter.instruction("mov x16, x14");                                        // remember the slot that points at this run
    emitter.instruction("add x14, x10, #16");                                   // later runs link through this run's next field
    emitter.instruction("str xzr, [x14]");                                      // this run ends the list until another one opens
    emitter.instruction("mov x15, x10");                                        // this block is now the open run
    emitter.instruction("b __rt_heap_alloc_defrag_next");                       // continue with the next block
    emitter.label("__rt_heap_alloc_defrag_live");
    emitter.instruction("mov x15, #0");                                         // a live block closes the open run
    emitter.label("__rt_heap_alloc_defrag_next");
    emitter.instruction("mov x10, x9");                                         // advance to the next block's header
    emitter.instruction("b __rt_heap_alloc_defrag_scan");                       // keep walking the arena

    // -- a run that reaches the live end is returned to the bump pointer --
    emitter.label("__rt_heap_alloc_defrag_done");
    emitter.instruction("cbz x15, __rt_heap_alloc_fl_start");                   // the arena ends in a live block: retry from the rebuilt list
    emitter.instruction("cmp x10, x13");                                        // did the walk end exactly at the live end, inside the run?
    emitter.instruction("b.ne __rt_heap_alloc_fl_start");                       // no — it stopped at a corrupt header, keep the run listed
    emitter.instruction("str xzr, [x16]");                                      // unlink the tail run from the rebuilt list
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_heap_buf");
    emitter.instruction("sub x12, x15, x12");                                   // x12 = the tail run's offset in the arena
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_heap_off");
    emitter.instruction("str x12, [x11]");                                      // shrink the bump pointer back to the run's header
    emitter.instruction("b __rt_heap_alloc_fl_start");                          // retry the request against the defragmented heap
}

/// Emits `__rt_heap_alloc_defragment` for x86_64; see [`emit_heap_defragment_aarch64`].
///
/// Reached with `rax` = the aligned request, which it preserves. It touches only the registers
/// the allocator already clobbers (`r8`–`r11`, `rcx`, `rdx`, `rsi`).
fn emit_heap_defragment_x86_64(emitter: &mut Emitter) {
    emitter.label("__rt_heap_alloc_defragment");
    // -- only worth a pass when some class holds a parked block --
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_small_bins");
    emitter.instruction("xor r9d, r9d");                                        // r9 = size-class index being probed
    emitter.label("__rt_heap_alloc_defrag_probe");
    emitter.instruction("cmp QWORD PTR [r8 + r9*8], 0");                        // is this class empty?
    emitter.instruction("jne __rt_heap_alloc_defrag_clear_start");              // no — a parked class block exists, defragmenting can help
    emitter.instruction("inc r9");                                              // move on to the next size class
    emitter.instruction(&format!("cmp r9, {}", HEAP_SIZE_CLASS_COUNT));         // have all size classes been probed?
    emitter.instruction("jb __rt_heap_alloc_defrag_probe");                     // no — keep probing
    emitter.instruction("jmp __rt_heap_exhausted");                             // every class is empty: the heap really is full

    // -- the arena walk rediscovers every parked block, so the classes start empty --
    emitter.label("__rt_heap_alloc_defrag_clear_start");
    emitter.instruction("xor r9d, r9d");                                        // r9 = size-class index being cleared
    emitter.label("__rt_heap_alloc_defrag_clear");
    emitter.instruction("mov QWORD PTR [r8 + r9*8], 0");                        // empty this class
    emitter.instruction("inc r9");                                              // move on to the next size class
    emitter.instruction(&format!("cmp r9, {}", HEAP_SIZE_CLASS_COUNT));         // have all size classes been emptied?
    emitter.instruction("jb __rt_heap_alloc_defrag_clear");                     // no — keep clearing

    // -- walk the arena header by header, rebuilding the ordered list from its runs --
    // r10 = header being scanned, rcx = live heap end, r9 = link slot the next run is stored
    // through, r11 = header of the run the previous block extended (0 after a live block),
    // rsi = link slot that points at that run, rdx = next header, r8 = scratch
    crate::codegen_support::abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    crate::codegen_support::abi::emit_symbol_address(emitter, "rcx", "_heap_off");
    emitter.instruction("mov rcx, QWORD PTR [rcx]");                            // load the current heap bump offset
    emitter.instruction("add rcx, r10");                                        // rcx = live heap end
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_free_list");
    emitter.instruction("mov QWORD PTR [r9], 0");                               // the rebuilt list starts empty
    emitter.instruction("mov rsi, r9");                                         // no run yet: its link slot is the list head
    emitter.instruction("xor r11d, r11d");                                      // no run is open before the first block
    emitter.label("__rt_heap_alloc_defrag_scan");
    emitter.instruction("cmp r10, rcx");                                        // has the walk reached the live heap end?
    emitter.instruction("jae __rt_heap_alloc_defrag_done");                     // yes — every block has been classified
    emitter.instruction("mov r8d, DWORD PTR [r10]");                            // r8 = this block's payload size
    emitter.instruction("lea rdx, [r10 + r8 + 16]");                            // rdx = the next block's header
    emitter.instruction("cmp rdx, rcx");                                        // does the recorded size stay inside the live heap?
    emitter.instruction("ja __rt_heap_alloc_defrag_done");                      // no — a corrupt header ends the walk; what was rebuilt stands
    emitter.instruction(&format!(
        "cmp DWORD PTR [r10 + 4], {:#x}",
        HEAP_FREE_REFCOUNT_MARK
    ));                                                                         // is the block parked?
    emitter.instruction("jne __rt_heap_alloc_defrag_live");                     // no — a live block closes any open run
    emitter.instruction("cmp QWORD PTR [r10 + 8], 0");                          // a parked block retains no kind
    emitter.instruction("jne __rt_heap_alloc_defrag_live");                     // anything else is live
    emitter.instruction("test r11, r11");                                       // is a run open?
    emitter.instruction("jz __rt_heap_alloc_defrag_open_run");                  // no — this block starts one
    emitter.instruction("mov r8, rdx");                                         // r8 = this block's end
    emitter.instruction("sub r8, r11");                                         // r8 = bytes from the run's header to this block's end
    emitter.instruction("sub r8, 16");                                          // r8 = the run's payload once it absorbs this block
    emitter.instruction("mov DWORD PTR [r11], r8d");                            // grow the open run over this block
    emitter.instruction("jmp __rt_heap_alloc_defrag_next");                     // continue with the next block
    emitter.label("__rt_heap_alloc_defrag_open_run");
    emitter.instruction("mov QWORD PTR [r9], r10");                             // link this run after the previous one, in address order
    emitter.instruction("mov rsi, r9");                                         // remember the slot that points at this run
    emitter.instruction("lea r9, [r10 + 16]");                                  // later runs link through this run's next field
    emitter.instruction("mov QWORD PTR [r9], 0");                               // this run ends the list until another one opens
    emitter.instruction("mov r11, r10");                                        // this block is now the open run
    emitter.instruction("jmp __rt_heap_alloc_defrag_next");                     // continue with the next block
    emitter.label("__rt_heap_alloc_defrag_live");
    emitter.instruction("xor r11d, r11d");                                      // a live block closes the open run
    emitter.label("__rt_heap_alloc_defrag_next");
    emitter.instruction("mov r10, rdx");                                        // advance to the next block's header
    emitter.instruction("jmp __rt_heap_alloc_defrag_scan");                     // keep walking the arena

    // -- a run that reaches the live end is returned to the bump pointer --
    emitter.label("__rt_heap_alloc_defrag_done");
    emitter.instruction("test r11, r11");                                       // does the arena end inside an open run?
    emitter.instruction("jz __rt_heap_alloc_fl_start");                         // no — retry from the rebuilt list
    emitter.instruction("cmp r10, rcx");                                        // did the walk end exactly at the live end?
    emitter.instruction("jne __rt_heap_alloc_fl_start");                        // no — it stopped at a corrupt header, keep the run listed
    emitter.instruction("mov QWORD PTR [rsi], 0");                              // unlink the tail run from the rebuilt list
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_buf");
    emitter.instruction("sub r11, r8");                                         // r11 = the tail run's offset in the arena
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_off");
    emitter.instruction("mov QWORD PTR [r9], r11");                             // shrink the bump pointer back to the run's header
    emitter.instruction("jmp __rt_heap_alloc_fl_start");                        // retry the request against the defragmented heap
}

/// Emits the x86_64 Linux variant of `__rt_heap_alloc`.
///
/// Identical allocation strategy to the ARM64 path but uses System V AMD64 ABI
/// registers (`rax` = size/return, `r8–r15` = temporaries) and Linux syscalls
/// for error reporting (`syscall` instead of `svc #0x80`).
///
/// Header stamping uses `crate::codegen_support::sentinels::X86_64_HEAP_MAGIC_HI32` in the high 32 bits of the kind field
/// to distinguish owned heap blocks from other runtime values.
fn emit_heap_alloc_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: heap_alloc (free-list + bump) ---");
    emitter.label_global("__rt_heap_alloc");

    // -- normalize every payload to the runtime's 16-byte allocation alignment --
    emitter.instruction("cmp rax, 8");                                          // is the requested payload smaller than the minimum reusable block size?
    emitter.instruction("jge __rt_heap_alloc_start");                           // keep the original request when it already satisfies the minimum payload size
    emitter.instruction("mov rax, 8");                                          // round tiny allocations up so free blocks can still carry a next pointer
    emitter.label("__rt_heap_alloc_start");
    emitter.instruction("add rax, 15");                                         // reserve room to round the payload up to the next 16-byte boundary
    emitter.instruction("and rax, -16");                                        // keep the aligned payload size before any heap pointer arithmetic
    emitter.instruction("mov r10d, 0xffffffff");                                // materialize u32::MAX with zero-extension to a 64-bit comparison operand
    emitter.instruction("cmp rax, r10");                                        // verify the request fits the 32-bit block-size header
    emitter.instruction("ja __rt_heap_alloc_size_overflow");                    // reject before a narrowing metadata store can truncate the size

    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_debug_enabled");
    emitter.instruction("mov r8, QWORD PTR [r8]");                              // load the heap-debug enabled flag before consuming cached free-list state
    emitter.instruction("test r8, r8");                                         // is heap-debug validation enabled for this allocation path?
    emitter.instruction("jz __rt_heap_alloc_debug_checked");                    // skip the validator when heap-debug mode is disabled
    emitter.instruction("sub rsp, 16");                                         // reserve one aligned stack slot to preserve the requested allocation size
    emitter.instruction("mov QWORD PTR [rsp], rax");                            // save the requested allocation size across the nested validator call
    emitter.instruction("call __rt_heap_debug_validate_free_list");             // verify the ordered free list and cached small bins before consuming them
    emitter.instruction("mov rax, QWORD PTR [rsp]");                            // restore the requested allocation size after the nested validator call
    emitter.instruction("add rsp, 16");                                         // release the temporary validator spill slot
    emitter.label("__rt_heap_alloc_debug_checked");

    // -- an exact size class answers most requests with one pop (see the AArch64 arm) --
    emitter.instruction(&format!("cmp rax, {}", HEAP_SIZE_CLASS_MAX));          // is the request small enough for an exact size class?
    emitter.instruction("ja __rt_heap_alloc_fl_start");                         // larger requests use the general free list
    emitter.instruction("mov r9, rax");                                         // r9 = request, to derive its class index
    emitter.instruction("shr r9, 4");                                           // r9 = request in 16-byte units, at least 1 after rounding
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_small_bins");
    emitter.instruction("lea r9, [r8 + r9*8 - 8]");                             // r9 = address of this class's head slot (index = units - 1)
    emitter.label("__rt_heap_alloc_class_pop");
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // r10 = the class's first parked block, or null when it is empty
    emitter.instruction("test r10, r10");                                       // is anything parked in this class?
    emitter.instruction("jz __rt_heap_alloc_fl_start");                         // no — fall back to the general free list
    // -- reject a head that escaped the live heap window before dereferencing it --
    crate::codegen_support::abi::emit_symbol_address(emitter, "rdx", "_heap_buf");
    emitter.instruction("cmp r10, rdx");                                        // does the cached block point below the heap buffer base?
    emitter.instruction("jb __rt_heap_alloc_class_drop");                       // wild pointer: drop the chain, its next link cannot be trusted
    crate::codegen_support::abi::emit_symbol_address(emitter, "rsi", "_heap_off");
    emitter.instruction("mov rsi, QWORD PTR [rsi]");                            // load the current heap bump offset before deriving the live heap end
    emitter.instruction("add rsi, rdx");                                        // rsi = current live heap end
    emitter.instruction("cmp r10, rsi");                                        // does the cached block point at or beyond the live heap end?
    emitter.instruction("jae __rt_heap_alloc_class_drop");                      // wild pointer: drop the chain past the live heap window
    // -- a parked block carries the free mark, no retained heap kind, and a size of its class --
    emitter.instruction("mov edx, DWORD PTR [r10 + 4]");                        // load the cached block refcount from its header
    emitter.instruction(&format!("cmp edx, {:#x}", HEAP_FREE_REFCOUNT_MARK));   // does the cached block still claim to be parked?
    emitter.instruction("jne __rt_heap_alloc_class_unlink_invalid");            // anything else marks a poisoned entry, unlink it
    emitter.instruction("mov rdx, QWORD PTR [r10 + 8]");                        // load the cached block heap kind from its header
    emitter.instruction("test rdx, rdx");                                       // does the cached block retain a live heap kind?
    emitter.instruction("jnz __rt_heap_alloc_class_unlink_invalid");            // a retained live kind marks a poisoned entry, unlink it
    emitter.instruction("mov r11d, DWORD PTR [r10]");                           // load the cached block payload size before reusing it
    emitter.instruction("cmp r11, rax");                                        // is the cached block at least as large as the request?
    emitter.instruction("jb __rt_heap_alloc_class_unlink_invalid");             // a block too small for its own class is poison, unlink it
    emitter.instruction("lea rdx, [r10 + r11 + 16]");                           // rdx = cached block claimed end address
    emitter.instruction("cmp rdx, rsi");                                        // does the cached block stay inside the live heap window?
    emitter.instruction("ja __rt_heap_alloc_class_unlink_invalid");             // unlink cached entries whose recorded size overruns the live heap
    emitter.instruction("mov r11, QWORD PTR [r10 + 16]");                       // load the next parked block of this class
    emitter.instruction("mov QWORD PTR [r9], r11");                             // pop the head off its size class
    emitter.instruction("mov DWORD PTR [r10 + 4], 1");                          // restore a live refcount of one in the reused heap header
    emitter.instruction(&format!("mov r11, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(0))); // materialize the x86_64 heap marker while leaving the low kind bits clear
    emitter.instruction("mov QWORD PTR [r10 + 8], r11");                        // stamp the reused heap header as an owned raw heap allocation
    emitter.instruction("lea rax, [r10 + 16]");                                 // return the user payload pointer instead of the internal header address
    emitter.instruction("jmp __rt_heap_alloc_count");                           // reuse the shared allocation-accounting path for cached blocks

    emitter.label("__rt_heap_alloc_class_drop");
    emitter.instruction("mov QWORD PTR [r9], 0");                               // drop the unreachable chain so no later pop follows the wild pointer
    emitter.instruction("jmp __rt_heap_alloc_fl_start");                        // this class is empty from here on, use the general free list
    emitter.label("__rt_heap_alloc_class_unlink_invalid");
    emitter.instruction("mov rdx, QWORD PTR [r10 + 16]");                       // the poisoned block is in-heap, so its next link is a safe load
    emitter.instruction("mov QWORD PTR [r9], rdx");                             // unlink the poisoned block from its size class
    emitter.instruction("jmp __rt_heap_alloc_class_pop");                       // try the class's next parked block

    // -- walk the general free list looking for a first-fit block --
    // r8 = heap buffer base and rcx = live heap end: neither moves during the walk
    emitter.label("__rt_heap_alloc_fl_start");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_buf");
    crate::codegen_support::abi::emit_symbol_address(emitter, "rcx", "_heap_off");
    emitter.instruction("mov rcx, QWORD PTR [rcx]");                            // load the current heap bump offset once for the whole walk
    emitter.instruction("add rcx, r8");                                         // rcx = current live heap end
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_free_list");
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // r10 = current free-list block header or null if the list is empty
    emitter.label("__rt_heap_alloc_fl_loop");
    emitter.instruction("test r10, r10");                                       // did the free-list walk run out of blocks?
    emitter.instruction("jz __rt_heap_alloc_bump");                             // yes — fall back to bump allocation from the heap buffer
    emitter.instruction("cmp r10, r8");                                         // reject free-list pointers that point before the heap buffer
    emitter.instruction("jb __rt_heap_alloc_fl_drop_tail");                     // drop the rest of a chain once it leaves the heap buffer
    emitter.instruction("cmp r10, rcx");                                        // reject free-list pointers at or beyond the live heap end
    emitter.instruction("jae __rt_heap_alloc_fl_drop_tail");                    // truncate a chain that has escaped the live heap window
    emitter.instruction("mov r11d, DWORD PTR [r10]");                           // load this free block payload size from its header
    emitter.instruction("cmp r11, 8");                                          // is the free block large enough to carry allocator metadata?
    emitter.instruction("jb __rt_heap_alloc_fl_unlink_invalid");                // unlink in-heap nodes with impossible payload sizes
    emitter.instruction("mov edx, DWORD PTR [r10 + 4]");                        // load the candidate refcount from the free block header
    emitter.instruction(&format!("cmp edx, {:#x}", HEAP_FREE_REFCOUNT_MARK));   // does the candidate still claim to be parked on the free list?
    emitter.instruction("jne __rt_heap_alloc_fl_unlink_invalid");               // live or corrupt blocks must never be reused through the free list
    emitter.instruction("mov rdx, QWORD PTR [r10 + 8]");                        // load the candidate heap kind from the free block header
    emitter.instruction("test rdx, rdx");                                       // does this free-list node retain a live heap kind?
    emitter.instruction("jnz __rt_heap_alloc_fl_unlink_invalid");               // free-list blocks must not retain live heap kind metadata
    emitter.instruction("lea rdx, [r10 + r11 + 16]");                           // compute the candidate block end address
    emitter.instruction("cmp rdx, rcx");                                        // does the candidate block stay inside the live heap window?
    emitter.instruction("ja __rt_heap_alloc_fl_unlink_invalid");                // unlink in-heap nodes whose recorded size overruns live heap
    emitter.instruction("cmp r11, rax");                                        // does this free block fit the requested payload size?
    emitter.instruction("jae __rt_heap_alloc_fl_found");                        // yes — reuse this free block
    emitter.instruction("lea r9, [r10 + 16]");                                  // advance prev_next_addr to the current block's next field
    emitter.instruction("mov r10, QWORD PTR [r10 + 16]");                       // move on to the next free block in the ordered list
    emitter.instruction("jmp __rt_heap_alloc_fl_loop");                         // continue searching for a large-enough free block

    // -- unlink malformed in-heap nodes instead of allocating from stale state --
    emitter.label("__rt_heap_alloc_fl_unlink_invalid");
    emitter.instruction("mov rdx, QWORD PTR [r10 + 16]");                       // preserve the next free-list pointer before removing this malformed node
    emitter.instruction("mov QWORD PTR [r9], rdx");                             // unlink the malformed in-heap node from the free-list chain
    emitter.instruction("mov r10, rdx");                                        // continue scanning from the successor node
    emitter.instruction("jmp __rt_heap_alloc_fl_loop");                         // keep searching for a valid reusable free block

    // -- truncate chains that have escaped the heap address range --
    emitter.label("__rt_heap_alloc_fl_drop_tail");
    emitter.instruction("mov QWORD PTR [r9], 0");                               // drop the unreachable tail without dereferencing an out-of-heap pointer
    emitter.instruction("xor r10, r10");                                        // force the next loop iteration to use bump allocation
    emitter.instruction("jmp __rt_heap_alloc_fl_loop");                         // restart the loop with the truncated chain

    // -- found a suitable free block; either split it or consume it whole --
    emitter.label("__rt_heap_alloc_fl_found");
    emitter.instruction("mov rcx, r11");                                        // preserve the matched free block payload size for split calculations
    emitter.instruction("sub rcx, rax");                                        // compute the payload bytes left over after satisfying this allocation
    emitter.instruction("cmp rcx, 24");                                         // is there room for a new 16-byte header plus minimum reusable payload?
    emitter.instruction("jb __rt_heap_alloc_fl_take_whole");                    // no — consume the entire free block instead of creating an unusable tail
    emitter.instruction("lea r8, [r10 + rax + 16]");                            // compute the header address for the split remainder block
    emitter.instruction("sub rcx, 16");                                         // remove the new header size so rcx becomes the remainder payload size
    emitter.instruction("mov DWORD PTR [r8], ecx");                             // write the split remainder payload size into its free-block header
    emitter.instruction(&format!(
        "mov DWORD PTR [r8 + 4], {:#x}",
        HEAP_FREE_REFCOUNT_MARK
    ));                                                                         // the split remainder stays parked, so it carries the free mark
    emitter.instruction("mov QWORD PTR [r8 + 8], 0");                           // free-list blocks clear the heap kind until a typed allocation reuses them
    emitter.instruction("mov rdx, QWORD PTR [r10 + 16]");                       // preserve the original successor before rewriting the free-list links
    // A remainder small enough for a size class is parked there (see the AArch64 arm).
    emitter.instruction("cmp rcx, 16");                                         // is the remainder below the smallest size class?
    emitter.instruction("jb __rt_heap_alloc_split_to_list");                    // yes — keep it in the ordered free list
    emitter.instruction(&format!("cmp rcx, {}", HEAP_SIZE_CLASS_MAX));          // is the remainder above the largest size class?
    emitter.instruction("ja __rt_heap_alloc_split_to_list");                    // yes — keep it in the ordered free list
    emitter.instruction("mov QWORD PTR [r9], rdx");                             // unlink the matched block: prev->next = current->next
    emitter.instruction("shr rcx, 4");                                          // rcx = remainder payload in 16-byte units
    crate::codegen_support::abi::emit_symbol_address(emitter, "rsi", "_heap_small_bins");
    emitter.instruction("lea rsi, [rsi + rcx*8 - 8]");                          // rsi = address of the remainder's class head slot
    emitter.instruction("mov rdx, QWORD PTR [rsi]");                            // rdx = the class's current head
    emitter.instruction("mov QWORD PTR [r8 + 16], rdx");                        // remainder->next = previous class head
    emitter.instruction("mov QWORD PTR [rsi], r8");                             // park the remainder at the head of its class
    emitter.instruction("jmp __rt_heap_alloc_split_parked");                    // the remainder is parked; finish the reused block
    emitter.label("__rt_heap_alloc_split_to_list");
    emitter.instruction("mov QWORD PTR [r8 + 16], rdx");                        // splice the split remainder to the original successor
    emitter.instruction("mov QWORD PTR [r9], r8");                              // replace the matched free block with the split remainder in the free list
    emitter.label("__rt_heap_alloc_split_parked");
    emitter.instruction("mov DWORD PTR [r10], eax");                            // shrink the reused block header down to the requested payload size
    emitter.instruction("mov DWORD PTR [r10 + 4], 1");                          // restore a live refcount of one in the reused heap header
    emitter.instruction(&format!("mov r8, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(0))); // materialize the x86_64 heap marker for the reused block header
    emitter.instruction("mov QWORD PTR [r10 + 8], r8");                         // stamp the reused block as an owned raw heap allocation
    emitter.instruction("lea rax, [r10 + 16]");                                 // return the user payload pointer for the reused block
    emitter.instruction("jmp __rt_heap_alloc_count");                           // reuse the common allocation-accounting path

    emitter.label("__rt_heap_alloc_fl_take_whole");
    emitter.instruction("mov rcx, QWORD PTR [r10 + 16]");                       // load the matched free block successor before unlinking it
    emitter.instruction("mov QWORD PTR [r9], rcx");                             // unlink the matched free block from the ordered free list
    emitter.instruction("mov DWORD PTR [r10 + 4], 1");                          // restore a live refcount of one in the reused whole block
    emitter.instruction(&format!("mov r8, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(0))); // materialize the x86_64 heap marker for the reused whole block
    emitter.instruction("mov QWORD PTR [r10 + 8], r8");                         // stamp the whole reused block as an owned raw heap allocation
    emitter.instruction("lea rax, [r10 + 16]");                                 // return the user payload pointer for the reused free block

    emitter.label("__rt_heap_alloc_count");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_gc_allocs");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // load the current allocation counter before recording this heap allocation
    emitter.instruction("add r9, 1");                                           // count the newly allocated or reused heap block
    emitter.instruction("mov QWORD PTR [r8], r9");                              // store the updated allocation counter back into runtime state
    emitter.instruction("mov r11d, DWORD PTR [r10]");                           // load the finalized payload size from the allocated block header
    emitter.instruction("add r11, 16");                                         // include the uniform 16-byte header in the live-footprint accounting
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_gc_live");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // load the current live-byte count before adding this block footprint
    emitter.instruction("add r9, r11");                                         // add this block's payload-plus-header footprint to the live-byte count
    emitter.instruction("mov QWORD PTR [r8], r9");                              // store the updated live-byte count after the allocation
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_gc_peak");
    emitter.instruction("mov rcx, QWORD PTR [r8]");                             // load the previous live-byte peak watermark before comparing against the new total
    emitter.instruction("cmp r9, rcx");                                         // did this allocation raise the peak live-byte watermark?
    emitter.instruction("cmova rcx, r9");                                       // keep the larger of the current live total and the previous peak watermark
    emitter.instruction("mov QWORD PTR [r8], rcx");                             // store the updated peak live-byte watermark back into runtime state
    emitter.instruction("ret");                                                 // return the owned user payload pointer in rax

    // -- no reusable block found; bump allocate from the heap buffer --
    emitter.label("__rt_heap_alloc_bump");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_off");
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // load the current bump offset from the heap state
    emitter.instruction("mov rcx, r10");                                        // preserve the current bump offset while computing the tentative allocation end
    emitter.instruction("add rcx, rax");                                        // add the requested payload size to the current bump offset
    emitter.instruction("add rcx, 16");                                         // include the uniform 16-byte header in the tentative allocation end
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_max");
    emitter.instruction("mov r8, QWORD PTR [r8]");                              // load the configured heap capacity in bytes
    emitter.instruction("cmp rcx, r8");                                         // does the bump allocation still fit inside the configured heap capacity?
    emitter.instruction("ja __rt_heap_alloc_defragment");                       // no — reclaim parked class blocks before giving up
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_heap_buf");
    emitter.instruction("lea r10, [r11 + r10]");                                // compute the new block header address inside the heap buffer
    emitter.instruction("mov DWORD PTR [r10], eax");                            // write the requested payload size into the new block header
    emitter.instruction("mov DWORD PTR [r10 + 4], 1");                          // initialize the new block refcount to one
    emitter.instruction(&format!("mov r8, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(0))); // materialize the x86_64 heap marker for the freshly bumped block
    emitter.instruction("mov QWORD PTR [r10 + 8], r8");                         // stamp the new block as an owned raw heap allocation
    emitter.instruction("mov QWORD PTR [r9], rcx");                             // persist the advanced bump offset after carving out this block
    emitter.instruction("lea rax, [r10 + 16]");                                 // return the user payload pointer instead of the header address
    emitter.instruction("jmp __rt_heap_alloc_count");                           // reuse the shared allocation-accounting path for bumped blocks

    emit_heap_defragment_x86_64(emitter);

    // -- fatal error: heap memory exhausted --
    // `__rt_cstr`/`__rt_cstr2` are SEPARATE atoms and have to reach this block by name. Under
    // macOS dead stripping an ordinary `label()` is renamed to an `L`-local, which is not a
    // symbol at all, so that cross-atom branch is not a relocation the linker can follow: the
    // atom is collectable and the branch lands wherever the linker put the next one. The
    // `.alt_entry` alias keeps the block inside `__rt_heap_alloc`'s atom while giving the
    // cross-atom callers a real symbol; the `L`-local name stays for the intra-atom CONDITIONAL
    // branches, which older assemblers refuse to point at an `.alt_entry` label.
    emitter.label_shared("__rt_heap_exhausted_entry");
    emitter.label("__rt_heap_exhausted");
    if emitter.cdylib_boundary {
        crate::codegen_support::abi::emit_symbol_address(
            emitter,
            "r8",
            crate::codegen_support::cdylib::BOUNDARY_ACTIVE,
        );
        emitter.instruction("mov r8, QWORD PTR [r8]");                          // read whether a native recovery boundary is active
        emitter.instruction("test r8, r8");                                     // distinguish cdylib recovery from executable fatal handling
        emitter.instruction("jz __rt_heap_exhausted_fatal");                    // retain executable fatal behavior without a boundary
        emitter.instruction(&format!(                                           // materialize the boundary allocation-failure status
            "mov r8, {}",
            crate::codegen_support::cdylib::STATUS_ALLOCATION_FAILURE
        ));
        crate::codegen_support::abi::emit_store_reg_to_symbol(
            emitter,
            "r8",
            crate::codegen_support::cdylib::BOUNDARY_STATUS,
            0,
        );
        crate::codegen_support::abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
        emitter.instruction("jmp __rt_throw_current");                          // unwind to the cdylib handler with allocation status recorded
        emitter.label("__rt_heap_exhausted_fatal");
    }
    emitter.instruction("mov r11, QWORD PTR [rsp]");                            // capture the immediate caller return address before reserving fatal-report storage
    emitter.instruction("sub rsp, 16");                                         // reserve aligned storage for the failed allocation request and return address
    emitter.instruction("mov QWORD PTR [rsp], rax");                            // preserve the aligned failed allocation request across fatal writes
    emitter.instruction("mov QWORD PTR [rsp + 8], r11");                        // preserve the immediate caller return address beside the failed request
    emitter.instruction("mov edi, 2");                                          // fd = stderr for the heap exhaustion fatal error message
    crate::codegen_support::abi::emit_symbol_address(emitter, "rsi", "_heap_err_msg");
    emitter.instruction("mov edx, 35");                                         // pass the exact heap exhaustion message length to the Linux write syscall
    emitter.instruction("mov eax, 1");                                          // Linux x86_64 syscall 1 = write
    emitter.instruction("syscall");                                             // print the fatal heap exhaustion message to stderr
    emit_heap_exhaustion_stats(emitter);
    emitter.instruction("mov edi, 1");                                          // exit code 1 for heap exhaustion
    emitter.instruction("mov eax, 231");                                        // Linux x86_64 syscall 231 = exit_group
    emitter.instruction("syscall");                                             // terminate the process after reporting heap exhaustion

    emitter.label("__rt_heap_alloc_size_overflow");
    emitter.instruction("jmp __rt_heap_exhausted");                             // report an impossible header size through the established fatal path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::{Platform, Target};

    /// Verifies the AArch64 allocator rejects payload sizes that cannot be
    /// represented by its 32-bit block-size header before any metadata write.
    #[test]
    fn aarch64_heap_allocator_guards_32_bit_header_size() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::AArch64));
        emit_heap_alloc(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("lsr x9, x0, #32\n"));
        assert!(asm.contains("cbnz x9, __rt_heap_alloc_size_overflow\n"));
        assert!(asm.contains("__rt_heap_alloc_size_overflow:\n"));
    }

    /// Verifies PIC allocators recover through the cdylib boundary instead of exiting.
    #[test]
    fn pic_heap_allocator_routes_exhaustion_to_cdylib_boundary() {
        let mut arm = Emitter::new_cdylib(Target::new(Platform::MacOS, Arch::AArch64));
        emit_heap_alloc(&mut arm);
        let arm_asm = arm.output();
        assert!(arm_asm.contains(crate::codegen_support::cdylib::BOUNDARY_ACTIVE));
        assert!(arm_asm.contains("b __rt_throw_current"));

        let mut x86 = Emitter::new_cdylib(Target::new(Platform::Linux, Arch::X86_64));
        emit_heap_alloc(&mut x86);
        let x86_asm = x86.output();
        assert!(x86_asm.contains(crate::codegen_support::cdylib::BOUNDARY_STATUS));
        assert!(x86_asm.contains("jmp __rt_throw_current"));
    }

    /// Verifies the x86_64 allocator rejects payload sizes that cannot be
    /// represented by its 32-bit block-size header before truncating `rax`.
    #[test]
    fn x86_64_heap_allocator_guards_32_bit_header_size() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_heap_alloc(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("mov r10d, 0xffffffff\n"));
        assert!(asm.contains("cmp rax, r10\n"));
        assert!(asm.contains("ja __rt_heap_alloc_size_overflow\n"));
        assert!(asm.contains("__rt_heap_alloc_size_overflow:\n"));
    }

    /// Verifies both allocator targets report the counters that explain terminal exhaustion.
    #[test]
    fn heap_exhaustion_reports_allocation_counters() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            emit_heap_alloc(&mut emitter);
            let asm = emitter.output();
            for label in [
                "_heap_stats_allocs_msg",
                "_heap_stats_frees_msg",
                "_heap_stats_live_msg",
                "_heap_stats_peak_msg",
                "_heap_stats_bump_msg",
                "_heap_stats_max_msg",
                "_heap_stats_hash_origin_msg",
                "_heap_stats_return_msg",
                "_heap_stats_request_msg",
                "_heap_stats_nl",
            ] {
                assert!(asm.contains(label), "{target:?}: {label} missing");
            }
            assert!(asm.contains("__rt_itoa"), "{target:?}: no counter formatting");
        }
    }
}

/// Emits allocator counters immediately before a terminal heap-exhaustion exit.
fn emit_heap_exhaustion_stats(emitter: &mut Emitter) {
    emit_heap_exhaustion_hash_origin(emitter);
    emit_heap_exhaustion_caller_return_address(emitter);
    emit_heap_exhaustion_requested_bytes(emitter);
    for (label, value) in [
        ("_heap_stats_allocs_msg", "_gc_allocs"),
        ("_heap_stats_frees_msg", "_gc_frees"),
        ("_heap_stats_live_msg", "_gc_live"),
        ("_heap_stats_peak_msg", "_gc_peak"),
        ("_heap_stats_bump_msg", "_heap_off"),
        ("_heap_stats_max_msg", "_heap_max"),
    ] {
        let len = match label {
            "_heap_stats_allocs_msg" => b"Heap stats: allocs=".len(),
            "_heap_stats_frees_msg" => b" frees=".len(),
            "_heap_stats_live_msg" => b" live=".len(),
            "_heap_stats_peak_msg" => b" peak=".len(),
            "_heap_stats_bump_msg" => b" bump=".len(),
            "_heap_stats_max_msg" => b" max=".len(),
            _ => unreachable!("heap stat label table is exhaustive"),
        };
        crate::codegen_support::emit_write_literal_stderr(emitter, label, len);
        match emitter.target.arch {
            Arch::AArch64 => {
                crate::codegen_support::abi::emit_symbol_address(emitter, "x0", value);
                emitter.instruction("ldr x0, [x0]");                            // load the selected allocator counter for decimal formatting
            }
            Arch::X86_64 => {
                crate::codegen_support::abi::emit_symbol_address(emitter, "rax", value);
                emitter.instruction("mov rax, QWORD PTR [rax]");                // load the selected allocator counter for decimal formatting
            }
        }
        crate::codegen_support::abi::emit_call_label(emitter, "__rt_itoa");
        crate::codegen_support::emit_write_current_string_stderr(emitter);
    }
    crate::codegen_support::emit_write_literal_stderr(emitter, "_heap_stats_nl", 1);
}

/// Emits the `__rt_hash_new` caller captured before that helper invokes the allocator.
fn emit_heap_exhaustion_hash_origin(emitter: &mut Emitter) {
    crate::codegen_support::emit_write_literal_stderr(
        emitter,
        "_heap_stats_hash_origin_msg",
        b"Heap stats: hash_new_origin=".len(),
    );
    match emitter.target.arch {
        Arch::AArch64 => {
            crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_stats_hash_origin");
            emitter.instruction("ldr x0, [x9]");                                // load the hash-construction caller captured before its allocator call
            crate::codegen_support::abi::emit_call_label(emitter, "__rt_itoa");
            crate::codegen_support::emit_write_current_string_stderr(emitter);
        }
        Arch::X86_64 => {
            crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_heap_stats_hash_origin");
            emitter.instruction("mov rax, QWORD PTR [r11]");                    // load the hash-construction caller captured before its allocator call
            crate::codegen_support::abi::emit_call_label(emitter, "__rt_itoa");
            crate::codegen_support::emit_write_current_string_stderr(emitter);
        }
    }
}

/// Emits the immediate runtime caller address captured at terminal heap exhaustion.
fn emit_heap_exhaustion_caller_return_address(emitter: &mut Emitter) {
    crate::codegen_support::emit_write_literal_stderr(
        emitter,
        "_heap_stats_return_msg",
        b"Heap stats: caller_return=".len(),
    );
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x0, [sp, #8]");                            // reload the call-site return address captured before fatal reporting clobbers link register state
            crate::codegen_support::abi::emit_call_label(emitter, "__rt_itoa");
            crate::codegen_support::emit_write_current_string_stderr(emitter);
        }
        Arch::X86_64 => {
            emitter.instruction("mov rax, QWORD PTR [rsp + 8]");                // reload the call-site return address captured before fatal reporting clobbers caller-saved state
            crate::codegen_support::abi::emit_call_label(emitter, "__rt_itoa");
            crate::codegen_support::emit_write_current_string_stderr(emitter);
        }
    }
}

/// Emits the normalized request that the allocator could not fit in the heap arena.
fn emit_heap_exhaustion_requested_bytes(emitter: &mut Emitter) {
    crate::codegen_support::emit_write_literal_stderr(
        emitter,
        "_heap_stats_request_msg",
        b"Heap stats: request=".len(),
    );
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x0, [sp]");                                // reload the failed allocation request for decimal formatting
            crate::codegen_support::abi::emit_call_label(emitter, "__rt_itoa");
            crate::codegen_support::emit_write_current_string_stderr(emitter);
            emitter.instruction("add sp, sp, #16");                             // release the request preservation slot after reporting it
        }
        Arch::X86_64 => {
            emitter.instruction("mov rax, QWORD PTR [rsp]");                    // reload the failed allocation request for decimal formatting
            crate::codegen_support::abi::emit_call_label(emitter, "__rt_itoa");
            crate::codegen_support::emit_write_current_string_stderr(emitter);
            emitter.instruction("add rsp, 16");                                 // release the request preservation slot after reporting it
        }
    }
}
