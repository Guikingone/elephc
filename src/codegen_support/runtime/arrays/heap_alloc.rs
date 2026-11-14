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

/// Number of segregated free-list bins: eight 8-byte-step classes (8..=64 bytes), then
/// 128, 256, 512 and 1024, then 1088 / 2112 (the 16- and 32-slot hash tables), 1536, 3072
/// and 4096.
pub(crate) const HEAP_BIN_COUNT: usize = 17;
/// Largest payload cached in a segregated bin (blocks above it use the ordered free list).
pub(crate) const HEAP_BIN_MAX_PAYLOAD: usize = 4096;
/// Byte offset of the last bin head inside `_heap_small_bins`.
pub(crate) const HEAP_BIN_LAST_OFFSET: usize = (HEAP_BIN_COUNT - 1) * 8;

/// Payload size of every bin class, plus one exclusive upper bound past the last class.
///
/// A binned request is rounded up to its class size, and a freed block is cached in the
/// largest class it can serve, so the head of a bin always satisfies any request mapped to
/// that bin. Every class above 64 bytes is a multiple of 64, which lets the class of a larger
/// size be read from a 64-byte-granular lookup table. Emitted as the `_heap_bin_sizes` table.
pub(crate) const HEAP_BIN_CLASS_SIZES: [usize; HEAP_BIN_COUNT + 1] =
    [8, 16, 24, 32, 40, 48, 56, 64, 128, 256, 512, 1024, 1088, 1536, 2112, 3072, 4096, 4097];

/// Entries of the 64-byte-granular class lookup tables (`size >> 6` reaches 64 at 4096).
pub(crate) const HEAP_BIN_LOOKUP_LEN: usize = HEAP_BIN_MAX_PAYLOAD / 64 + 2;

/// `_heap_bin_ceil_index[j]`: the smallest class holding `64 * (j + 1)` bytes, indexed by
/// `(size - 1) >> 6` for a request above 64 bytes.
pub(crate) fn heap_bin_ceil_index_table() -> Vec<u8> {
    (0..HEAP_BIN_LOOKUP_LEN)
        .map(|j| {
            let need = (64 * (j + 1)).min(HEAP_BIN_MAX_PAYLOAD);
            HEAP_BIN_CLASS_SIZES[..HEAP_BIN_COUNT]
                .iter()
                .position(|&size| size >= need)
                .unwrap_or(HEAP_BIN_COUNT - 1) as u8
        })
        .collect()
}

/// `_heap_bin_floor_index[k]`: the largest class not above `64 * k` bytes, indexed by
/// `size >> 6` for a freed block of at least 64 bytes.
pub(crate) fn heap_bin_floor_index_table() -> Vec<u8> {
    (0..HEAP_BIN_LOOKUP_LEN)
        .map(|k| {
            HEAP_BIN_CLASS_SIZES[..HEAP_BIN_COUNT]
                .iter()
                .rposition(|&size| size <= (64 * k).max(64))
                .unwrap_or(0) as u8
        })
        .collect()
}

/// Returns the 32-bit register name aliasing an AArch64 `xN` register.
fn aarch64_w(reg: &str) -> String {
    reg.replacen('x', "w", 1)
}

/// Emits AArch64 code computing the bin-head byte offset of the smallest class that holds a
/// request of `size` (8..=4096) bytes: `(size - 1) >> 3` up to 64, else the ceiling table.
pub(crate) fn emit_bin_offset_aarch64(emitter: &mut Emitter, size: &str, out: &str, scratch: &str, scratch2: &str) {
    emitter.instruction(&format!("sub {}, {}, #1", out, size));                     // size - 1 so exact class sizes stay in their own class
    emitter.instruction(&format!("lsr {}, {}, #6", scratch2, out));                  // 64-byte granule index of the request
    crate::codegen_support::abi::emit_symbol_address(emitter, scratch, "_heap_bin_ceil_index");
    emitter.instruction(&format!("ldrb {}, [{}, {}]", aarch64_w(scratch), scratch, scratch2)); // class index for requests above 64 bytes
    emitter.instruction(&format!("lsr {}, {}, #3", out, out));                       // 8-byte-step class index for requests up to 64 bytes
    emitter.instruction(&format!("cmp {}, #64", size));                              // is the request above the 8-byte-step classes?
    emitter.instruction(&format!("csel {}, {}, {}, hi", out, scratch, out));         // pick the table class for larger requests
    emitter.instruction(&format!("lsl {}, {}, #3", out, out));                       // bin-head byte offset = index * 8
}

/// Emits x86_64 code computing the bin-head byte offset of the smallest class that holds a
/// request of `size` (8..=4096) bytes: `(size - 1) >> 3` up to 64, else the ceiling table.
pub(crate) fn emit_bin_offset_x86_64(emitter: &mut Emitter, size: &str, out: &str, scratch: &str, scratch2: &str) {
    emitter.instruction(&format!("lea {}, [{} - 1]", out, size));                  // size - 1 so exact class sizes stay in their own class
    emitter.instruction(&format!("mov {}, {}", scratch2, out));                    // copy size - 1
    emitter.instruction(&format!("shr {}, 6", scratch2));                          // 64-byte granule index of the request
    crate::codegen_support::abi::emit_symbol_address(emitter, scratch, "_heap_bin_ceil_index");
    emitter.instruction(&format!("movzx {}, BYTE PTR [{} + {}]", scratch, scratch, scratch2)); // class index for requests above 64 bytes
    emitter.instruction(&format!("shr {}, 3", out));                               // 8-byte-step class index for requests up to 64 bytes
    emitter.instruction(&format!("cmp {}, 64", size));                             // is the request above the 8-byte-step classes?
    emitter.instruction(&format!("cmova {}, {}", out, scratch));                   // pick the table class for larger requests
    emitter.instruction(&format!("shl {}, 3", out));                               // bin-head byte offset = index * 8
}

/// Emits AArch64 code computing the bin-head byte offset of the largest class a freed block of
/// `size` (8..=4096) bytes can serve: `(size >> 3) - 1` below 64, else the floor table.
pub(crate) fn emit_free_bin_offset_aarch64(emitter: &mut Emitter, size: &str, out: &str, scratch: &str, scratch2: &str) {
    emitter.instruction(&format!("lsr {}, {}, #3", out, size));                      // size / 8
    emitter.instruction(&format!("sub {}, {}, #1", out, out));                       // 8-byte-step class index the block fully covers
    emitter.instruction(&format!("lsr {}, {}, #6", scratch2, size));                 // 64-byte granule count of the block
    crate::codegen_support::abi::emit_symbol_address(emitter, scratch, "_heap_bin_floor_index");
    emitter.instruction(&format!("ldrb {}, [{}, {}]", aarch64_w(scratch), scratch, scratch2)); // class index for blocks of 64+ bytes
    emitter.instruction(&format!("cmp {}, #64", size));                              // does the block reach the table classes?
    emitter.instruction(&format!("csel {}, {}, {}, hs", out, scratch, out));         // pick the table class for larger blocks
    emitter.instruction(&format!("lsl {}, {}, #3", out, out));                       // bin-head byte offset = index * 8
}

/// Emits x86_64 code computing the bin-head byte offset of the largest class a freed block of
/// `size` (8..=4096) bytes can serve: `(size >> 3) - 1` below 64, else the floor table.
pub(crate) fn emit_free_bin_offset_x86_64(emitter: &mut Emitter, size: &str, out: &str, scratch: &str, scratch2: &str) {
    emitter.instruction(&format!("mov {}, {}", out, size));                        // copy the freed block size
    emitter.instruction(&format!("shr {}, 3", out));                               // size / 8
    emitter.instruction(&format!("sub {}, 1", out));                               // 8-byte-step class index the block fully covers
    emitter.instruction(&format!("mov {}, {}", scratch2, size));                   // copy the freed block size
    emitter.instruction(&format!("shr {}, 6", scratch2));                          // 64-byte granule count of the block
    crate::codegen_support::abi::emit_symbol_address(emitter, scratch, "_heap_bin_floor_index");
    emitter.instruction(&format!("movzx {}, BYTE PTR [{} + {}]", scratch, scratch, scratch2)); // class index for blocks of 64+ bytes
    emitter.instruction(&format!("cmp {}, 64", size));                             // does the block reach the table classes?
    emitter.instruction(&format!("cmovae {}, {}", out, scratch));                  // pick the table class for larger blocks
    emitter.instruction(&format!("shl {}, 3", out));                               // bin-head byte offset = index * 8
}

/// Emits the `__rt_heap_alloc` runtime helper: a free-list allocator with size-segregated
/// small-bin caching and a bump-pointer fallback.
///
/// Allocation path (ARM64): small requests (≤1024 bytes) are rounded up to their class and pop
/// the class's bin head, else carve from the bump tail, else search larger classes and the
/// ordered free list; larger requests walk the ordered free list, then bump.
/// Allocation path (x86_64): identical logic via `emit_heap_alloc_linux_x86_64`.
///
/// Each block carries a 16-byte header `[size:4][refcount:4][kind:8]` before the user pointer.
/// Free blocks reuse the same header layout plus a `next_ptr:8` word for list chaining.
///
/// Input: `x0` (ARM) / `rax` (x86_64) = requested payload bytes (minimum 8 enforced).
/// Output: `x0` / `rax` = user pointer (header + 16).
///
/// Updates `_gc_allocs`, `_gc_live`, and `_gc_peak` counters on every allocation.
/// On heap exhaustion, prints a fatal message to stderr and exits with code 1.
pub fn emit_heap_alloc(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_heap_alloc_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: heap_alloc (free-list + bump) ---");
    emitter.label_global("__rt_heap_alloc");

    // -- enforce minimum allocation of 8 bytes (free payload needs space for next ptr) --
    emitter.instruction("cmp x0, #8");                                          // is requested size < 8?
    emitter.instruction("b.ge __rt_heap_alloc_start");                          // skip if already >= 8
    emitter.instruction("mov x0, #8");                                          // round up to minimum 8 bytes
    emitter.label("__rt_heap_alloc_start");

    // -- debug mode: validate the free list before consuming it --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_debug_enabled");
    emitter.instruction("ldr x9, [x9]");                                        // load the heap-debug enabled flag
    emitter.instruction("cbz x9, __rt_heap_alloc_debug_checked");               // skip validation when heap-debug mode is disabled
    emitter.instruction("stp x0, x30, [sp, #-16]!");                            // preserve allocation size and return address across validation
    emitter.instruction("bl __rt_heap_debug_validate_free_list");               // verify the ordered free list before searching it
    emitter.instruction("ldp x0, x30, [sp], #16");                              // restore allocation size and return address after validation
    emitter.label("__rt_heap_alloc_debug_checked");

    // -- try small segregated bins before walking the general free list --
    emitter.instruction(&format!("cmp x0, #{}", HEAP_BIN_MAX_PAYLOAD));         // do we fit in the segregated bin classes?
    emitter.instruction("b.hi __rt_heap_alloc_fl_start");                       // larger requests still use the general free list
    emit_bin_offset_aarch64(emitter, "x0", "x13", "x14", "x15");                // x13 = byte offset of the request's size-class bin
    crate::codegen_support::abi::emit_symbol_address(emitter, "x14", "_heap_bin_sizes");
    emitter.instruction("ldr x0, [x14, x13]");                                  // round the request up to its class size so every cached block in the bin fits
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_small_bins");
    emitter.instruction("add x9, x9, x13");                                     // x9 = address of the request's bin head
    emitter.instruction("ldr x10, [x9]");                                       // x10 = first cached block of this class, or null
    emitter.instruction("cbz x10, __rt_heap_alloc_small_bump");                 // empty class: carve from the bump tail
    emitter.instruction("mov x16, x9");                                         // the bin head is the previous next-pointer slot
    emitter.instruction("b __rt_heap_alloc_small_bin_found");                   // any cached block of this class satisfies the rounded request

    // -- bump tail full: search larger classes, then the general free list --
    emitter.label("__rt_heap_alloc_small_rescan");
    emit_bin_offset_aarch64(emitter, "x0", "x13", "x14", "x15");                // x13 = byte offset of the request's size-class bin
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_small_bins");
    emitter.instruction("add x9, x9, x13");                                     // x9 = address of the first candidate bin head
    emitter.label("__rt_heap_alloc_small_bin_loop");
    emitter.instruction("mov x16, x9");                                         // x16 tracks the previous next-pointer slot while scanning this bin
    emitter.label("__rt_heap_alloc_small_bin_scan");
    emitter.instruction("ldr x10, [x16]");                                      // x10 = current cached block header or null when this bin is exhausted
    emitter.instruction("cbz x10, __rt_heap_alloc_small_bin_next_class");       // try the next larger bin when this bin has no fitting block
    emitter.instruction("ldr w11, [x10]");                                      // load the cached block payload size before reusing it
    emitter.instruction("cmp x11, x0");                                         // does the cached block satisfy the requested payload size?
    emitter.instruction("b.hs __rt_heap_alloc_small_bin_found");                // yes — reuse this cached block safely
    emitter.instruction("add x16, x10, #16");                                   // advance the previous next-pointer slot to current->next
    emitter.instruction("b __rt_heap_alloc_small_bin_scan");                    // keep searching this bin for a large-enough cached block
    emitter.label("__rt_heap_alloc_small_bin_next_class");
    emitter.instruction(&format!("cmp x13, #{}", HEAP_BIN_LAST_OFFSET));        // have we already checked the largest bin?
    emitter.instruction("b.eq __rt_heap_alloc_fl_start");                       // yes — fall back to the general free list
    emitter.instruction("add x13, x13, #8");                                    // advance to the next larger small-bin class
    emitter.instruction("add x9, x9, #8");                                      // move to the next bin-head slot
    emitter.instruction("b __rt_heap_alloc_small_bin_loop");                    // keep searching the remaining small bins

    emitter.label("__rt_heap_alloc_small_bin_found");
    emitter.instruction("ldr x11, [x10, #16]");                                 // x11 = cached_small_block->next within this size class
    emitter.instruction("str x11, [x16]");                                      // unlink the cached block from its segregated small-bin chain
    emitter.instruction("mov w13, #1");                                         // initial refcount = 1 for the reused block
    emitter.instruction("str w13, [x10, #4]");                                  // restore the live refcount in the reused header
    emitter.instruction("str xzr, [x10, #8]");                                  // reset heap kind to raw until a typed constructor overwrites it
    emitter.instruction("add x0, x10, #16");                                    // return user pointer = header + 16
    emitter.instruction("b __rt_heap_alloc_count");                             // reuse the shared allocation-accounting path

    // -- walk the general free list looking for first-fit block --
    // x0 = requested size, x9 = prev_next_addr, x10 = current block header
    emitter.label("__rt_heap_alloc_fl_start");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_free_list");
    emitter.instruction("ldr x10, [x9]");                                       // x10 = first free block header (0 if empty)

    // -- walk the free list looking for first-fit block --
    emitter.label("__rt_heap_alloc_fl_loop");
    emitter.instruction("cbz x10, __rt_heap_alloc_bump");                       // no free block found, fall through to bump
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_heap_buf");
    emitter.instruction("cmp x10, x12");                                        // reject free-list pointers that point before the heap buffer
    emitter.instruction("b.lo __rt_heap_alloc_fl_drop_tail");                   // drop the rest of a chain once it leaves the heap buffer
    crate::codegen_support::abi::emit_symbol_address(emitter, "x13", "_heap_off");
    emitter.instruction("ldr x13, [x13]");                                      // load the current heap bump offset before deriving the live heap end
    emitter.instruction("add x13, x12, x13");                                   // x13 = current live heap end
    emitter.instruction("cmp x10, x13");                                        // reject free-list pointers at or beyond the live heap end
    emitter.instruction("b.hs __rt_heap_alloc_fl_drop_tail");                   // truncate a chain that has escaped the live heap window
    emitter.instruction("ldr w11, [x10]");                                      // x11 = block size (32-bit, zero-extends)
    emitter.instruction("cmp x11, #8");                                         // is the free block large enough to carry allocator metadata?
    emitter.instruction("b.lo __rt_heap_alloc_fl_unlink_invalid");              // unlink in-heap nodes with impossible payload sizes
    emitter.instruction("ldr w14, [x10, #4]");                                  // load the candidate refcount from the free block header
    emitter.instruction("cbnz w14, __rt_heap_alloc_fl_unlink_invalid");         // live blocks must never be reused through the free list
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
    emitter.instruction("str wzr, [x13, #4]");                                  // free remainder keeps refcount cleared while on the free list
    emitter.instruction("str xzr, [x13, #8]");                                  // free remainder has no heap kind while on the free list
    emitter.instruction("ldr x14, [x10, #16]");                                 // x14 = current->next before splitting
    emitter.instruction(&format!("cmp x12, #{}", HEAP_BIN_MAX_PAYLOAD));         // is the remainder small enough for the size-class bins?
    emitter.instruction("b.hi __rt_heap_alloc_fl_split_keep");                  // no — it stays in the ordered free list
    emitter.instruction("str x14, [x9]");                                       // unlink the matched block from the ordered free list
    emit_free_bin_offset_aarch64(emitter, "x12", "x11", "x16", "x15");          // x11 = byte offset of the largest class the remainder can serve
    crate::codegen_support::abi::emit_symbol_address(emitter, "x16", "_heap_small_bins");
    emitter.instruction("add x16, x16, x11");                                   // x16 = address of that bin head
    emitter.instruction("ldr x14, [x16]");                                      // x14 = previous bin head
    emitter.instruction("str x14, [x13, #16]");                                 // remainder->next = previous bin head
    emitter.instruction("str x13, [x16]");                                      // cache the remainder in its bin instead of the ordered list
    emitter.instruction("b __rt_heap_alloc_fl_split_done");                     // finish stamping the allocated block
    emitter.label("__rt_heap_alloc_fl_split_keep");
    emitter.instruction("str x14, [x13, #16]");                                 // remainder->next = current->next
    emitter.instruction("str x13, [x9]");                                       // prev->next = remainder header
    emitter.label("__rt_heap_alloc_fl_split_done");
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
    emitter.instruction("cmp x12, x13");                                        // does the allocation fit?
    emitter.instruction("b.gt __rt_heap_exhausted");                            // no — fatal error
    emitter.label("__rt_heap_alloc_bump_commit");

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

    // -- small requests with empty bins: O(1) bump carve, general free list only when full --
    // Walking the ordered free list for every boxed temporary made each allocation O(n) and
    // split large free blocks into tiny pieces; small blocks come back through the bins.
    emitter.label("__rt_heap_alloc_small_bump");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_heap_off");
    emitter.instruction("ldr x10, [x9]");                                       // x10 = current heap offset
    emitter.instruction("add x12, x10, x0");                                    // x12 = offset + requested size
    emitter.instruction("add x12, x12, #16");                                   // x12 = offset + requested + header (16 bytes)
    crate::codegen_support::abi::emit_symbol_address(emitter, "x13", "_heap_max");
    emitter.instruction("ldr x13, [x13]");                                      // x13 = heap max size in bytes
    emitter.instruction("cmp x12, x13");                                        // does the small block still fit at the bump tail?
    emitter.instruction("b.gt __rt_heap_alloc_small_rescan");                   // no — reuse a cached block of a larger class or the general free list
    emitter.instruction("b __rt_heap_alloc_bump_commit");                       // yes — carve it from the bump tail

    // -- fatal error: heap memory exhausted --
    emitter.label("__rt_heap_exhausted");
    emitter.instruction("mov x0, #2");                                          // fd = stderr
    crate::codegen_support::abi::emit_symbol_address(emitter, "x1", "_heap_err_msg");
    emitter.instruction("mov x2, #35");                                         // message length: "Fatal error: heap memory exhausted\n"
    emitter.syscall(4);
    emitter.instruction("mov x0, #1");                                          // exit code 1
    emitter.syscall(1);
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

    // -- enforce minimum allocation of 8 bytes (free payload needs space for next ptr) --
    emitter.instruction("cmp rax, 8");                                          // is the requested payload smaller than the minimum reusable block size?
    emitter.instruction("jge __rt_heap_alloc_start");                           // keep the original request when it already satisfies the minimum payload size
    emitter.instruction("mov rax, 8");                                          // round tiny allocations up so free blocks can still carry a next pointer
    emitter.label("__rt_heap_alloc_start");

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

    // -- try small segregated bins before walking the general free list --
    emitter.instruction(&format!("cmp rax, {}", HEAP_BIN_MAX_PAYLOAD));         // does this request fit within the segregated bin classes?
    emitter.instruction("ja __rt_heap_alloc_fl_start");                         // larger payloads still use the general ordered free list
    emit_bin_offset_x86_64(emitter, "rax", "r8", "r11", "r10");                 // r8 = byte offset of the request's size-class bin
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_bin_sizes");
    emitter.instruction("mov rax, QWORD PTR [r9 + r8]");                        // round the request up to its class size so every cached block in the bin fits
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_small_bins");
    emitter.instruction("add r9, r8");                                          // r9 = address of the request's bin head
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // r10 = first cached block of this class, or null
    emitter.instruction("test r10, r10");                                       // is this class empty?
    emitter.instruction("jz __rt_heap_alloc_small_bump");                       // empty class: carve from the bump tail
    emitter.instruction("mov r11, QWORD PTR [r10 + 16]");                       // any cached block of this class fits: load its next pointer
    emitter.instruction("mov QWORD PTR [r9], r11");                             // unlink the cached block from the head of its bin
    emitter.instruction("mov DWORD PTR [r10 + 4], 1");                          // restore a live refcount of one in the reused heap header
    emitter.instruction(&format!("mov r11, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(0))); // materialize the x86_64 heap marker while leaving the low kind bits clear
    emitter.instruction("mov QWORD PTR [r10 + 8], r11");                        // stamp the reused heap header as an owned raw heap allocation
    emitter.instruction("lea rax, [r10 + 16]");                                 // return the user payload pointer instead of the internal header address
    emitter.instruction("jmp __rt_heap_alloc_count");                           // shared allocation accounting

    // -- bump tail full: search larger classes, then the general free list --
    emitter.label("__rt_heap_alloc_small_rescan");
    emit_bin_offset_x86_64(emitter, "rax", "r8", "r11", "r10");                 // r8 = byte offset of the request's size-class bin
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_small_bins");
    emitter.instruction("add r9, r8");                                          // r9 = address of the first candidate small-bin head slot
    emitter.label("__rt_heap_alloc_small_bin_loop");
    emitter.instruction("mov rcx, r9");                                         // rcx tracks the previous next-pointer slot while scanning this bin
    emitter.label("__rt_heap_alloc_small_bin_scan");
    emitter.instruction("mov r10, QWORD PTR [rcx]");                            // r10 = current cached block header or null when this bin is exhausted
    emitter.instruction("test r10, r10");                                       // did this bin scan run out of cached blocks?
    emitter.instruction("jz __rt_heap_alloc_small_bin_next_class");             // try the next larger bin when this bin has no fitting block
    emitter.instruction("mov r11d, DWORD PTR [r10]");                           // load the cached block payload size before reusing it
    emitter.instruction("cmp r11, rax");                                        // does the cached block satisfy the requested payload size?
    emitter.instruction("jae __rt_heap_alloc_small_bin_found");                 // yes — reuse this cached block safely
    emitter.instruction("lea rcx, [r10 + 16]");                                 // advance the previous next-pointer slot to current->next
    emitter.instruction("jmp __rt_heap_alloc_small_bin_scan");                  // keep searching this bin for a large-enough cached block
    emitter.label("__rt_heap_alloc_small_bin_next_class");
    emitter.instruction(&format!("cmp r8, {}", HEAP_BIN_LAST_OFFSET));          // have we already checked the largest cache bin?
    emitter.instruction("je __rt_heap_alloc_fl_start");                         // yes — fall back to the general free list
    emitter.instruction("add r8, 8");                                           // advance to the next larger small-bin class offset
    emitter.instruction("add r9, 8");                                           // move to the next small-bin head slot
    emitter.instruction("jmp __rt_heap_alloc_small_bin_loop");                  // keep scanning the remaining small bins

    emitter.label("__rt_heap_alloc_small_bin_found");
    emitter.instruction("mov r11, QWORD PTR [r10 + 16]");                       // load the cached block's next pointer within this size class
    emitter.instruction("mov QWORD PTR [rcx], r11");                            // unlink the cached block from its segregated small-bin chain
    emitter.instruction("mov DWORD PTR [r10 + 4], 1");                          // restore a live refcount of one in the reused heap header
    emitter.instruction(&format!("mov r11, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(0))); // materialize the x86_64 heap marker while leaving the low kind bits clear
    emitter.instruction("mov QWORD PTR [r10 + 8], r11");                        // stamp the reused heap header as an owned raw heap allocation
    emitter.instruction("lea rax, [r10 + 16]");                                 // return the user payload pointer instead of the internal header address
    emitter.instruction("jmp __rt_heap_alloc_count");                           // reuse the shared allocation-accounting path for cached blocks

    // -- walk the general free list looking for a first-fit block --
    emitter.label("__rt_heap_alloc_fl_start");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_free_list");
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // r10 = current free-list block header or null if the list is empty
    emitter.label("__rt_heap_alloc_fl_loop");
    emitter.instruction("test r10, r10");                                       // did the free-list walk run out of blocks?
    emitter.instruction("jz __rt_heap_alloc_bump");                             // yes — fall back to bump allocation from the heap buffer
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_buf");
    emitter.instruction("cmp r10, r8");                                         // reject free-list pointers that point before the heap buffer
    emitter.instruction("jb __rt_heap_alloc_fl_drop_tail");                     // drop the rest of a chain once it leaves the heap buffer
    crate::codegen_support::abi::emit_symbol_address(emitter, "rcx", "_heap_off");
    emitter.instruction("mov rcx, QWORD PTR [rcx]");                            // load the current heap bump offset before deriving the live heap end
    emitter.instruction("add rcx, r8");                                         // rcx = current live heap end
    emitter.instruction("cmp r10, rcx");                                        // reject free-list pointers at or beyond the live heap end
    emitter.instruction("jae __rt_heap_alloc_fl_drop_tail");                    // truncate a chain that has escaped the live heap window
    emitter.instruction("mov r11d, DWORD PTR [r10]");                           // load this free block payload size from its header
    emitter.instruction("cmp r11, 8");                                          // is the free block large enough to carry allocator metadata?
    emitter.instruction("jb __rt_heap_alloc_fl_unlink_invalid");                // unlink in-heap nodes with impossible payload sizes
    emitter.instruction("mov edx, DWORD PTR [r10 + 4]");                        // load the candidate refcount from the free block header
    emitter.instruction("test edx, edx");                                       // is the candidate still marked live rather than free?
    emitter.instruction("jnz __rt_heap_alloc_fl_unlink_invalid");               // live blocks must never be reused through the free list
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
    emitter.instruction("mov DWORD PTR [r8 + 4], 0");                           // free-list blocks keep refcount zero while they remain unowned
    emitter.instruction("mov QWORD PTR [r8 + 8], 0");                           // free-list blocks clear the heap kind until a typed allocation reuses them
    emitter.instruction("mov rdx, QWORD PTR [r10 + 16]");                       // preserve the original successor before rewriting the free-list links
    emitter.instruction(&format!("cmp rcx, {}", HEAP_BIN_MAX_PAYLOAD));         // is the remainder small enough for the size-class bins?
    emitter.instruction("ja __rt_heap_alloc_fl_split_keep");                    // no — it stays in the ordered free list
    emitter.instruction("mov QWORD PTR [r9], rdx");                             // unlink the matched block from the ordered free list
    emit_free_bin_offset_x86_64(emitter, "rcx", "rdx", "r11", "r9");            // rdx = byte offset of the largest class the remainder can serve
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_heap_small_bins");
    emitter.instruction("add r11, rdx");                                        // r11 = address of that bin head
    emitter.instruction("mov rdx, QWORD PTR [r11]");                            // rdx = previous bin head
    emitter.instruction("mov QWORD PTR [r8 + 16], rdx");                        // remainder->next = previous bin head
    emitter.instruction("mov QWORD PTR [r11], r8");                             // cache the remainder in its bin instead of the ordered list
    emitter.instruction("jmp __rt_heap_alloc_fl_split_done");                   // finish stamping the allocated block
    emitter.label("__rt_heap_alloc_fl_split_keep");
    emitter.instruction("mov QWORD PTR [r8 + 16], rdx");                        // splice the split remainder to the original successor
    emitter.instruction("mov QWORD PTR [r9], r8");                              // replace the matched free block with the split remainder in the free list
    emitter.label("__rt_heap_alloc_fl_split_done");
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
    emitter.instruction("add QWORD PTR [r8], 1");                               // count the newly allocated or reused heap block
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
    emitter.instruction("ja __rt_heap_exhausted");                              // no — report heap exhaustion and terminate
    emitter.label("__rt_heap_alloc_bump_commit_x86");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_heap_buf");
    emitter.instruction("lea r10, [r11 + r10]");                                // compute the new block header address inside the heap buffer
    emitter.instruction("mov DWORD PTR [r10], eax");                            // write the requested payload size into the new block header
    emitter.instruction("mov DWORD PTR [r10 + 4], 1");                          // initialize the new block refcount to one
    emitter.instruction(&format!("mov r8, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(0))); // materialize the x86_64 heap marker for the freshly bumped block
    emitter.instruction("mov QWORD PTR [r10 + 8], r8");                         // stamp the new block as an owned raw heap allocation
    emitter.instruction("mov QWORD PTR [r9], rcx");                             // persist the advanced bump offset after carving out this block
    emitter.instruction("lea rax, [r10 + 16]");                                 // return the user payload pointer instead of the header address
    emitter.instruction("jmp __rt_heap_alloc_count");                           // reuse the shared allocation-accounting path for bumped blocks

    // -- small requests with empty bins: O(1) bump carve, general free list only when full --
    // Walking the ordered free list for every boxed temporary made each allocation O(n) and
    // split large free blocks into tiny pieces; small blocks come back through the bins.
    emitter.label("__rt_heap_alloc_small_bump");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_off");
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // load the current bump offset from the heap state
    emitter.instruction("mov rcx, r10");                                        // preserve the current bump offset while computing the tentative allocation end
    emitter.instruction("add rcx, rax");                                        // add the requested payload size to the current bump offset
    emitter.instruction("add rcx, 16");                                         // include the uniform 16-byte header in the tentative allocation end
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_max");
    emitter.instruction("mov r8, QWORD PTR [r8]");                              // load the configured heap capacity in bytes
    emitter.instruction("cmp rcx, r8");                                         // does the small block still fit at the bump tail?
    emitter.instruction("ja __rt_heap_alloc_small_rescan");                     // no — reuse a cached block of a larger class or the general free list
    emitter.instruction("jmp __rt_heap_alloc_bump_commit_x86");                 // yes — carve it from the bump tail

    // -- fatal error: heap memory exhausted --
    emitter.label("__rt_heap_exhausted");
    emitter.instruction("mov edi, 2");                                          // fd = stderr for the heap exhaustion fatal error message
    crate::codegen_support::abi::emit_symbol_address(emitter, "rsi", "_heap_err_msg");
    emitter.instruction("mov edx, 35");                                         // pass the exact heap exhaustion message length to the Linux write syscall
    emitter.instruction("mov eax, 1");                                          // Linux x86_64 syscall 1 = write
    emitter.instruction("syscall");                                             // print the fatal heap exhaustion message to stderr
    emitter.instruction("mov edi, 1");                                          // exit code 1 for heap exhaustion
    emitter.instruction("mov eax, 60");                                         // Linux x86_64 syscall 60 = exit
    emitter.instruction("syscall");                                             // terminate the process after reporting heap exhaustion
}
