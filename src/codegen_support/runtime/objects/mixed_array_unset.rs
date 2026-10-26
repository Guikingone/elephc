//! Purpose:
//! Emits the `__rt_mixed_array_unset` runtime helper for removing an offset from a boxed
//! `Mixed` container the lowering cannot statically type.
//!
//! Called from:
//! - `crate::codegen_support::runtime::objects::emit_mixed_array_unset()`.
//!
//! Key details:
//! - The key tuple matches `emit_normalized_hash_key`: int keys use `key_hi = -1`.
//! - Arrays promote to sparse storage (PHP `unset()` never renumbers surviving keys) and remove
//!   through `__rt_hash_unset`; the cell's payload is published back after the promotion.
//! - `ArrayAccess` objects resolve in two steps like `__rt_mixed_array_get`: the runtime's OWN
//!   containers by class id, then any PHP class declaring `ArrayAccess` through the dense
//!   `_class_offsetunset_ptrs` table. A PHP method BORROWS its argument, so this frame frees
//!   the boxed key.
//! - php's refusals are php's: a non-ArrayAccess object raises `Cannot use object of type … as
//!   array`, a scalar raises `Cannot unset offset in a non-array variable`, and a null cell or
//!   null payload is a silent no-op.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::runtime::data::CANNOT_UNSET_NON_ARRAY_MSG;
use crate::codegen_support::sentinels::{
    emit_branch_if_null_container, emit_throwable_creation_line_unknown, x86_64_heap_kind_word,
};

/// Dispatches to the target-specific `__rt_mixed_array_unset` emitter.
///
/// Checks `emitter.target.arch` and routes to either `emit_mixed_array_unset_x86_64`
/// (SysV ABI) or `emit_mixed_array_unset_aarch64` (AAPCS64). Both implementations share the
/// dispatch ladder, the sparse-key contract, and php's refusal behavior.
pub fn emit_mixed_array_unset(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_array_unset_x86_64(emitter);
        emit_throw_cannot_unset_non_array_x86_64(emitter);
        return;
    }
    emit_mixed_array_unset_aarch64(emitter);
    emit_throw_cannot_unset_non_array_aarch64(emitter);
}

/// ARM64 implementation of the `__rt_mixed_array_unset` runtime helper.
///
/// Input registers:
/// - `x0`: pointer to the boxed `Mixed` container cell (nullable)
/// - `x1`: key low word (array index or normalized hash low word)
/// - `x2`: key high word (integer-key sentinel `0xFFFF_FFFF` or hash high word)
///
/// The helper returns the (possibly mutated) cell pointer in `x0` for the EIR-level storeback,
/// and keeps the caller's frame pointers intact across the throw tails.
fn emit_mixed_array_unset_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_unset ---");
    emitter.label_global("__rt_mixed_array_unset");

    // Inputs: x0 = mixed_ptr, x1 = key_lo, x2 = key_hi.
    emitter.instruction("sub sp, sp, #64");                                     // reserve frame for inputs and saved frame registers
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the target Mixed cell
    emitter.instruction("str x1, [sp, #8]");                                    // save key_lo
    emitter.instruction("str x2, [sp, #16]");                                   // save key_hi so integer keys can be distinguished
    emitter.instruction("bl __rt_mixed_deref");                                 // follow a reference receiver to its current value
    emitter.instruction("str x0, [sp, #0]");                                    // save the concrete receiver

    emitter.instruction("cbz x0, __rt_mixed_array_unset_done");                 // a null cell has no offset to remove
    emitter.instruction("ldr x9, [x0]");                                        // load the boxed payload tag
    emitter.instruction("cmp x9, #4");                                          // is the Mixed payload an indexed array?
    emitter.instruction("b.eq __rt_mixed_array_unset_promote");                 // sparse removal promotes to hash storage
    emitter.instruction("cmp x9, #5");                                          // is the Mixed payload an associative array?
    emitter.instruction("b.eq __rt_mixed_array_unset_assoc");                   // route hash arrays to key-based removal
    emitter.instruction("cmp x9, #6");                                          // is the Mixed payload an object?
    emitter.instruction("b.eq __rt_mixed_array_unset_object");                  // route ArrayAccess objects to offsetUnset
    emitter.instruction("cmp x9, #8");                                          // is the target a canonical Mixed null?
    emitter.instruction("b.eq __rt_mixed_array_unset_done");                    // php's unset() on null is a silent no-op
    emitter.instruction("b __rt_mixed_array_unset_non_array");                  // scalars raise php's refusal

    emitter.label("__rt_mixed_array_unset_promote");
    emitter.instruction("ldr x0, [sp, #0]");                                    // pass the indexed Mixed cell for the promotion helper
    emitter.instruction("bl __rt_mixed_cell_promote_to_hash");                  // convert to hash storage in the cell (COW-safe)
    emitter.instruction("cbz x0, __rt_mixed_array_unset_non_array");            // an unconvertible receiver raises php's refusal
    emitter.label("__rt_mixed_array_unset_assoc");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the owning Mixed cell
    emitter.instruction("ldr x0, [x0, #8]");                                    // pass the hash payload to its COW boundary
    emitter.instruction("bl __rt_hash_ensure_unique");                          // detach shared hash storage before the removal
    emitter.instruction("ldr x9, [sp, #0]");                                    // recover the concrete owning cell
    emitter.instruction("str x0, [x9, #8]");                                    // publish the unique hash back into the cell
    emitter.instruction("ldr x1, [sp, #8]");                                    // pass the normalized key low word
    emitter.instruction("ldr x2, [sp, #16]");                                   // pass the normalized key high word
    emitter.instruction("bl __rt_hash_unset");                                  // remove the entry, leaving surviving keys alone
    emitter.instruction("b __rt_mixed_array_unset_done");                       // return the updated array cell after associative removal

    // The object arm mirrors `__rt_mixed_array_get`: the runtime's own containers by class id,
    // then any PHP class declaring `ArrayAccess` through the dense method table.
    emitter.label("__rt_mixed_array_unset_object");
    emitter.instruction("ldr x10, [x0, #8]");                                   // load object payload from the Mixed cell
    emit_branch_if_null_container(
        emitter,
        "x10",
        "x9",
        "__rt_mixed_array_unset_done",
    );                                                                          // a null-container payload has no offset to remove
    emitter.instruction("ldr x11, [x10]");                                      // load object class id
    abi::emit_symbol_address(emitter, "x12", "_spl_fixed_array_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // load SplFixedArray class id
    emitter.instruction("cmp x11, x12");                                        // is this a SplFixedArray object?
    emitter.instruction("b.eq __rt_mixed_array_unset_spl_fixed");               // dispatch to SplFixedArray::offsetUnset
    abi::emit_symbol_address(emitter, "x12", "_spl_dll_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // load SplDoublyLinkedList class id
    emitter.instruction("cmp x11, x12");                                        // is this a SplDoublyLinkedList object?
    emitter.instruction("b.eq __rt_mixed_array_unset_spl_dll");                 // dispatch to the shared list offsetUnset helper
    abi::emit_symbol_address(emitter, "x12", "_spl_stack_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // load SplStack class id
    emitter.instruction("cmp x11, x12");                                        // is this a SplStack object?
    emitter.instruction("b.eq __rt_mixed_array_unset_spl_dll");                 // SplStack shares the list helper
    abi::emit_symbol_address(emitter, "x12", "_spl_queue_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // load SplQueue class id
    emitter.instruction("cmp x11, x12");                                        // is this a SplQueue object?
    emitter.instruction("b.eq __rt_mixed_array_unset_spl_dll");                 // SplQueue shares the list helper
    emitter.instruction("b __rt_mixed_array_unset_php_offset_unset");           // every other class goes through PHP ArrayAccess or raises

    emitter.label("__rt_mixed_array_unset_spl_fixed");
    emitter.instruction("str x10, [sp, #24]");                                  // save unboxed receiver while boxing the key
    emitter.instruction("str xzr, [sp, #40]");                                  // mark the fixed-array dispatch target
    emitter.instruction("b __rt_mixed_array_unset_box_key");                    // share the offsetUnset call after key boxing

    emitter.label("__rt_mixed_array_unset_spl_dll");
    emitter.instruction("str x10, [sp, #24]");                                  // save unboxed receiver while boxing the key
    emitter.instruction("mov x13, #1");                                         // mark the shared-list dispatch target
    emitter.instruction("str x13, [sp, #40]");                                  // remember which SPL helper to call
    emitter.instruction("b __rt_mixed_array_unset_box_key");                    // box the key before dispatching to the shared list helper

    emitter.label("__rt_mixed_array_unset_box_key");
    emitter.instruction("ldr x11, [sp, #16]");                                  // reload normalized key high word
    emitter.instruction("cmn x11, #1");                                         // does key_hi carry the integer-key sentinel?
    emitter.instruction("b.eq __rt_mixed_array_unset_int_key");                 // integer keys box as Mixed int
    emitter.instruction("mov x0, #1");                                          // tag = string for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key string pointer
    emitter.instruction("ldr x2, [sp, #16]");                                   // key string length
    emitter.instruction("b __rt_mixed_array_unset_call");                       // share the offsetUnset call after key boxing
    emitter.label("__rt_mixed_array_unset_int_key");
    emitter.instruction("mov x0, #0");                                          // tag = int for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key integer payload
    emitter.instruction("mov x2, #0");                                          // integer keys have no high payload
    emitter.label("__rt_mixed_array_unset_call");
    emitter.instruction("bl __rt_mixed_from_value");                            // allocate the boxed ArrayAccess offset
    emitter.instruction("mov x1, x0");                                          // pass boxed offset as argument 1
    emitter.instruction("ldr x0, [sp, #24]");                                   // pass unboxed receiver as argument 0
    emitter.instruction("ldr x13, [sp, #40]");                                  // reload the SPL dispatch marker
    emitter.instruction("cbz x13, __rt_mixed_array_unset_spl_fixed_call");      // the marker is zero for the fixed-array target
    emitter.instruction("bl __rt_spl_dll_offset_unset");                        // remove through the shared SPL list offsetUnset helper
    emitter.instruction("b __rt_mixed_array_unset_done");                       // return the cell after shared list removal
    emitter.label("__rt_mixed_array_unset_spl_fixed_call");
    emitter.instruction("bl __rt_spl_fixed_offset_unset");                      // remove through SplFixedArray::offsetUnset
    emitter.instruction("b __rt_mixed_array_unset_done");                       // return the cell after fixed-array removal

    // Classes above are the runtime's OWN containers, recognised by class id. Everything else is
    // a PHP class, and its `ArrayAccess::offsetUnset` is reachable only through the dense method
    // table — without this an `unset()` through a `mixed` receiver was a compile-time refusal.
    emitter.label("__rt_mixed_array_unset_php_offset_unset");
    emitter.instruction("tbnz x11, #63, __rt_mixed_array_unset_not_indexable"); // synthetic negative ids cannot index metadata
    abi::emit_load_symbol_to_reg(emitter, "x12", "_class_iface_method_count", 0);
    emitter.instruction("cmp x11, x12");                                        // is the id within the dense table?
    emitter.instruction("b.hs __rt_mixed_array_unset_not_indexable");           // out-of-range ids have no entry
    abi::emit_symbol_address(emitter, "x12", "_class_offsetunset_ptrs");
    emitter.instruction("ldr x12, [x12, x11, lsl #3]");                         // resolve the concrete or inherited offsetUnset
    emitter.instruction("cbz x12, __rt_mixed_array_unset_not_indexable");       // 0 means the class is not ArrayAccess at all
    emitter.instruction("str x10, [sp, #24]");                                  // save unboxed receiver while boxing the key
    emitter.instruction("ldr x11, [sp, #16]");                                  // reload normalized key high word
    emitter.instruction("cmn x11, #1");                                         // does key_hi carry the integer-key sentinel?
    emitter.instruction("b.eq __rt_mixed_array_unset_php_int_key");             // integer keys box as Mixed int
    emitter.instruction("mov x0, #1");                                          // tag = string for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key string pointer
    emitter.instruction("ldr x2, [sp, #16]");                                   // key string length
    emitter.instruction("b __rt_mixed_array_unset_php_box_key");                // share the offsetUnset call after key boxing
    emitter.label("__rt_mixed_array_unset_php_int_key");
    emitter.instruction("mov x0, #0");                                          // tag = int for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key integer payload
    emitter.instruction("mov x2, #0");                                          // integer keys have no high payload
    emitter.label("__rt_mixed_array_unset_php_box_key");
    emitter.instruction("bl __rt_mixed_from_value");                            // allocate the boxed ArrayAccess offset
    emitter.instruction("str x0, [sp, #32]");                                   // stash the box for the borrowed-argument release
    emitter.instruction("mov x1, x0");                                          // pass boxed offset as argument 1
    emitter.instruction("ldr x0, [sp, #24]");                                   // pass unboxed receiver as argument 0
    emitter.instruction("ldr x11, [x0]");                                       // reload class id, clobbered across the boxing call
    abi::emit_symbol_address(emitter, "x12", "_class_offsetunset_ptrs");
    emitter.instruction("ldr x12, [x12, x11, lsl #3]");                         // re-resolve offsetUnset for the same class
    emitter.instruction("blr x12");                                             // remove through PHP's ArrayAccess::offsetUnset
    emitter.instruction("ldr x0, [sp, #32]");                                   // reload the boxed offset
    emitter.instruction("bl __rt_decref_mixed");                                // a PHP method BORROWS its argument, so this frame frees the box
    emitter.instruction("b __rt_mixed_array_unset_done");                       // return after releasing the borrowed PHP method key argument

    // PHP does not answer for an object it cannot index. `$o["k"]` on anything that is not
    // ArrayAccess raises, in every context including unset, so this leaves through the Error.
    emitter.label("__rt_mixed_array_unset_not_indexable");
    emitter.instruction("mov x0, x10");                                         // pass the unboxed receiver to name its class
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the helper frame before the tail-call
    emitter.instruction("b __rt_throw_object_not_array");                       // never returns

    // php raises `Cannot unset offset in a non-array variable` for a scalar receiver.
    emitter.label("__rt_mixed_array_unset_non_array");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the helper frame before the tail-call
    emitter.instruction("b __rt_throw_cannot_unset_non_array");                 // never returns

    emitter.label("__rt_mixed_array_unset_done");
    emitter.instruction("ldr x0, [sp, #0]");                                    // the container cell is the op's result
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return Mixed* in x0
}

/// Emits `__rt_throw_cannot_unset_non_array` for ARM64.
///
/// Takes no input. Raises `Cannot unset offset in a non-array variable`, php's refusal of
/// `unset($scalar['k'])`. Never returns.
fn emit_throw_cannot_unset_non_array_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: throw cannot-unset-non-array Error ---");
    emitter.label_global("__rt_throw_cannot_unset_non_array");

    // Stack (64 bytes): [sp, #0] the borrowed message pair.
    emitter.instruction("sub sp, sp, #64");                                     // reserve message state and frame linkage
    emitter.instruction("stp x29, x30, [sp, #48]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #48");                                    // establish a stable Error-construction frame

    // A constant lives for the whole process, so the Throwable can borrow those bytes directly.
    abi::emit_symbol_address(emitter, "x1", "_cannot_unset_non_array_msg");     // the whole message pointer
    emitter.instruction(&format!(
        "mov x2, #{}",
        CANNOT_UNSET_NON_ARRAY_MSG.len()
    ));                                                                         // the whole message byte length
    emitter.instruction("stp x1, x2, [sp]");                                    // preserve the message pair across the allocation

    emitter.instruction("mov x0, #56");                                         // canonical Throwable payload size
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the Error object payload
    emitter.instruction("mov x9, #6");                                          // heap kind 6 identifies a throwable object
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the allocation as a runtime object
    emitter.instruction("bl __rt_object_handle_acquire");                       // bind the Error to its PHP object handle
    abi::emit_load_symbol_to_reg(emitter, "x9", "_spl_error_class_id", 0);
    emitter.instruction("str x9, [x0]");                                        // stamp the per-program Error class id
    emitter.instruction("ldp x10, x11, [sp]");                                  // recover the message pair
    emitter.instruction("str x10, [x0, #8]");                                   // message pointer
    emitter.instruction("str x11, [x0, #16]");                                  // message byte length
    // __rt_heap_alloc recycles blocks without zeroing, so every remaining slot is written here.
    emitter.instruction("str xzr, [x0, #24]");                                  // code = 0
    emit_throwable_creation_line_unknown(emitter, "x0");
    emitter.instruction("str xzr, [x0, #40]");                                  // previous = null
    abi::emit_store_reg_to_symbol(emitter, "x0", "_exc_value", 0);              // publish the Throwable for the unwinder
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the local frame
    emitter.instruction("b __rt_throw_current");                                // unwind, or report it uncaught and exit like PHP
}

/// x86_64 implementation of the `__rt_mixed_array_unset` runtime helper.
///
/// Input registers (SysV ABI):
/// - `rdi`: pointer to the boxed `Mixed` container cell (nullable)
/// - `rsi`: key low word (array index or normalized hash low word)
/// - `rdx`: key high word (integer-key sentinel `-1` or hash high word)
///
/// Behavior mirrors the ARM64 version with x86_64-specific register and instruction encoding.
/// The helper returns the (possibly mutated) cell pointer in `rax` for the EIR-level storeback.
fn emit_mixed_array_unset_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_unset ---");
    emitter.label_global("__rt_mixed_array_unset");

    // Inputs (SysV): rdi = mixed_ptr, rsi = key_lo, rdx = key_hi.
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame
    emitter.instruction("sub rsp, 64");                                         // reserve slots for inputs and dispatch state
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the target Mixed cell
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save key_lo
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save key_hi so integer keys can be distinguished
    emitter.instruction("mov rax, rdi");                                        // translate the SysV receiver argument to the native Mixed dereference register
    emitter.instruction("call __rt_mixed_deref");                               // follow a reference receiver to its current value
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the concrete receiver

    emitter.instruction("test rax, rax");                                       // a null cell has no offset to remove
    emitter.instruction("jz __rt_mixed_array_unset_done");                      // a missing receiver has no offset to remove
    emitter.instruction("mov r9, QWORD PTR [rax]");                             // load the boxed payload tag
    emitter.instruction("cmp r9, 4");                                           // is the Mixed payload an indexed array?
    emitter.instruction("je __rt_mixed_array_unset_promote");                   // sparse removal promotes to hash storage
    emitter.instruction("cmp r9, 5");                                           // is the Mixed payload an associative array?
    emitter.instruction("je __rt_mixed_array_unset_assoc");                     // route hash arrays to key-based removal
    emitter.instruction("cmp r9, 6");                                           // is the Mixed payload an object?
    emitter.instruction("je __rt_mixed_array_unset_object");                    // route ArrayAccess objects to offsetUnset
    emitter.instruction("cmp r9, 8");                                           // is the target a canonical Mixed null?
    emitter.instruction("je __rt_mixed_array_unset_done");                      // php's unset() on null is a silent no-op
    emitter.instruction("jmp __rt_mixed_array_unset_non_array");                // scalars raise php's refusal

    emitter.label("__rt_mixed_array_unset_promote");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the indexed Mixed cell for the promotion helper
    emitter.instruction("call __rt_mixed_cell_promote_to_hash");                // convert to hash storage in the cell (COW-safe)
    emitter.instruction("test rax, rax");                                       // an unconvertible receiver raises php's refusal
    emitter.instruction("jz __rt_mixed_array_unset_non_array");                 // refuse a receiver whose payload cannot be promoted to an array
    emitter.label("__rt_mixed_array_unset_assoc");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the owning Mixed cell
    emitter.instruction("mov rdi, QWORD PTR [rax + 8]");                        // pass the hash payload to its COW boundary
    emitter.instruction("call __rt_hash_ensure_unique");                        // detach shared hash storage before the removal
    emitter.instruction("mov rcx, QWORD PTR [rbp - 8]");                        // recover the concrete owning cell
    emitter.instruction("mov QWORD PTR [rcx + 8], rax");                        // publish the unique hash back into the cell
    emitter.instruction("mov rdi, rax");                                        // pass the unique hash table
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // pass the normalized key low word
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // pass the normalized key high word
    emitter.instruction("call __rt_hash_unset");                                // remove the entry, leaving surviving keys alone
    emitter.instruction("jmp __rt_mixed_array_unset_done");                     // return the updated array cell after associative removal

    // The object arm mirrors `__rt_mixed_array_get`: the runtime's own containers by class id,
    // then any PHP class declaring `ArrayAccess` through the dense method table.
    emitter.label("__rt_mixed_array_unset_object");
    emitter.instruction("mov r10, QWORD PTR [rax + 8]");                        // load object payload from the Mixed cell
    emit_branch_if_null_container(
        emitter,
        "r10",
        "r9",
        "__rt_mixed_array_unset_done",
    );                                                                          // a null-container payload has no offset to remove
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // load object class id
    abi::emit_symbol_address(emitter, "r12", "_spl_fixed_array_class_id");
    emitter.instruction("mov r12, QWORD PTR [r12]");                            // load SplFixedArray class id
    emitter.instruction("cmp r11, r12");                                        // is this a SplFixedArray object?
    emitter.instruction("je __rt_mixed_array_unset_spl_fixed");                 // dispatch to SplFixedArray::offsetUnset
    abi::emit_symbol_address(emitter, "r12", "_spl_dll_class_id");
    emitter.instruction("mov r12, QWORD PTR [r12]");                            // load SplDoublyLinkedList class id
    emitter.instruction("cmp r11, r12");                                        // is this a SplDoublyLinkedList object?
    emitter.instruction("je __rt_mixed_array_unset_spl_dll");                   // dispatch to the shared list offsetUnset helper
    abi::emit_symbol_address(emitter, "r12", "_spl_stack_class_id");
    emitter.instruction("mov r12, QWORD PTR [r12]");                            // load SplStack class id
    emitter.instruction("cmp r11, r12");                                        // is this a SplStack object?
    emitter.instruction("je __rt_mixed_array_unset_spl_dll");                   // SplStack shares the list helper
    abi::emit_symbol_address(emitter, "r12", "_spl_queue_class_id");
    emitter.instruction("mov r12, QWORD PTR [r12]");                            // load SplQueue class id
    emitter.instruction("cmp r11, r12");                                        // is this a SplQueue object?
    emitter.instruction("je __rt_mixed_array_unset_spl_dll");                   // SplQueue shares the list helper
    emitter.instruction("jmp __rt_mixed_array_unset_php_offset_unset");         // every other class goes through PHP ArrayAccess or raises

    emitter.label("__rt_mixed_array_unset_spl_fixed");
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // save unboxed receiver while boxing the key
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // mark the fixed-array dispatch target
    emitter.instruction("jmp __rt_mixed_array_unset_box_key");                  // share the offsetUnset call after key boxing

    emitter.label("__rt_mixed_array_unset_spl_dll");
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // save unboxed receiver while boxing the key
    emitter.instruction("mov QWORD PTR [rbp - 48], 1");                         // mark the shared-list dispatch target
    emitter.instruction("jmp __rt_mixed_array_unset_box_key");                  // box the key before dispatching to the shared list helper


    emitter.label("__rt_mixed_array_unset_box_key");
    emitter.instruction("cmp QWORD PTR [rbp - 24], -1");                        // does key_hi carry the integer-key sentinel?
    emitter.instruction("je __rt_mixed_array_unset_int_key");                   // integer keys box as Mixed int
    emitter.instruction("mov edi, 1");                                          // tag = string for mixed_from_value
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // key string pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // key string length
    emitter.instruction("jmp __rt_mixed_array_unset_call");                     // share the offsetUnset call after key boxing
    emitter.label("__rt_mixed_array_unset_int_key");
    emitter.instruction("mov edi, 0");                                          // tag = int for mixed_from_value
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // key integer payload
    emitter.instruction("mov rdx, 0");                                          // integer keys have no high payload
    emitter.label("__rt_mixed_array_unset_call");
    emitter.instruction("call __rt_mixed_from_value");                          // allocate the boxed ArrayAccess offset
    emitter.instruction("mov rsi, rax");                                        // pass boxed offset as argument 1
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // pass unboxed receiver as argument 0
    emitter.instruction("cmp QWORD PTR [rbp - 48], 0");                         // which SPL helper does this dispatch to?
    emitter.instruction("je __rt_mixed_array_unset_spl_fixed_call");            // route the zero dispatch marker to fixed-array removal
    emitter.instruction("call __rt_spl_dll_offset_unset");                      // remove through the shared SPL list offsetUnset helper
    emitter.instruction("jmp __rt_mixed_array_unset_done");                     // return the cell after shared list removal
    emitter.label("__rt_mixed_array_unset_spl_fixed_call");
    emitter.instruction("call __rt_spl_fixed_offset_unset");                    // remove through SplFixedArray::offsetUnset
    emitter.instruction("jmp __rt_mixed_array_unset_done");                     // return the cell after fixed-array removal

    // Classes above are the runtime's OWN containers, recognised by class id. Everything else is
    // a PHP class, and its `ArrayAccess::offsetUnset` is reachable only through the dense method
    // table — without this an `unset()` through a `mixed` receiver was a compile-time refusal.
    emitter.label("__rt_mixed_array_unset_php_offset_unset");
    emitter.instruction("test r11, r11");                                       // synthetic negative ids cannot index metadata
    emitter.instruction("js __rt_mixed_array_unset_not_indexable");             // reject synthetic class ids before interface metadata lookup
    abi::emit_load_symbol_to_reg(emitter, "r12", "_class_iface_method_count", 0);
    emitter.instruction("cmp r11, r12");                                        // is the id within the dense table?
    emitter.instruction("jae __rt_mixed_array_unset_not_indexable");            // out-of-range ids have no entry
    abi::emit_symbol_address(emitter, "r12", "_class_offsetunset_ptrs");
    emitter.instruction("mov r12, QWORD PTR [r12 + r11 * 8]");                  // resolve the concrete or inherited offsetUnset
    emitter.instruction("test r12, r12");                                       // 0 means the class is not ArrayAccess at all
    emitter.instruction("jz __rt_mixed_array_unset_not_indexable");             // reject objects without an ArrayAccess method table
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // save unboxed receiver while boxing the key
    emitter.instruction("cmp QWORD PTR [rbp - 24], -1");                        // does key_hi carry the integer-key sentinel?
    emitter.instruction("je __rt_mixed_array_unset_php_int_key");               // integer keys box as Mixed int
    emitter.instruction("mov edi, 1");                                          // tag = string for mixed_from_value
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // key string pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // key string length
    emitter.instruction("jmp __rt_mixed_array_unset_php_box_key");              // share the offsetUnset call after key boxing
    emitter.label("__rt_mixed_array_unset_php_int_key");
    emitter.instruction("mov edi, 0");                                          // tag = int for mixed_from_value
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // key integer payload
    emitter.instruction("mov rdx, 0");                                          // integer keys have no high payload
    emitter.label("__rt_mixed_array_unset_php_box_key");
    emitter.instruction("call __rt_mixed_from_value");                          // allocate the boxed ArrayAccess offset
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // stash the box for the borrowed-argument release
    emitter.instruction("mov rsi, rax");                                        // pass boxed offset as argument 1
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // pass unboxed receiver as argument 0
    emitter.instruction("mov r11, QWORD PTR [rdi]");                            // reload class id, clobbered across the boxing call
    abi::emit_symbol_address(emitter, "r12", "_class_offsetunset_ptrs");
    emitter.instruction("mov r12, QWORD PTR [r12 + r11 * 8]");                  // re-resolve offsetUnset for the same class
    emitter.instruction("call r12");                                            // remove through PHP's ArrayAccess::offsetUnset
    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // reload the boxed offset
    emitter.instruction("call __rt_decref_mixed");                              // a PHP method BORROWS its argument, so this frame frees the box
    emitter.instruction("jmp __rt_mixed_array_unset_done");                     // return after releasing the borrowed PHP method key argument

    // PHP does not answer for an object it cannot index. `$o["k"]` on anything that is not
    // ArrayAccess raises, in every context including unset, so this leaves through the Error.
    emitter.label("__rt_mixed_array_unset_not_indexable");
    emitter.instruction("mov rdi, r10");                                        // pass the unboxed receiver to name its class
    emitter.instruction("mov rsp, rbp");                                        // release the helper frame before the tail-call
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("jmp __rt_throw_object_not_array");                     // never returns

    // php raises `Cannot unset offset in a non-array variable` for a scalar receiver.
    emitter.label("__rt_mixed_array_unset_non_array");
    emitter.instruction("mov rsp, rbp");                                        // release the helper frame before the tail-call
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("jmp __rt_throw_cannot_unset_non_array");               // never returns

    emitter.label("__rt_mixed_array_unset_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // the container cell is the op's result
    emitter.instruction("mov rsp, rbp");                                        // release the helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax
}

/// Emits `__rt_throw_cannot_unset_non_array` for x86_64 (SysV ABI).
///
/// Takes no input. Raises `Cannot unset offset in a non-array variable`, php's refusal of
/// `unset($scalar['k'])`. Never returns.
fn emit_throw_cannot_unset_non_array_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: throw cannot-unset-non-array Error ---");
    emitter.label_global("__rt_throw_cannot_unset_non_array");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish an Error-construction frame
    emitter.instruction("sub rsp, 48");                                         // reserve the message pair with aligned calls

    // A constant lives for the whole process, so the Throwable can borrow those bytes directly.
    emitter.instruction("lea rax, [rip + _cannot_unset_non_array_msg]");        // the whole message pointer
    emitter.instruction(&format!(
        "mov rdx, {}",
        CANNOT_UNSET_NON_ARRAY_MSG.len()
    ));                                                                         // the whole message byte length
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve the message pointer across the allocation
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // preserve the message byte length

    emitter.instruction("mov rax, 56");                                         // canonical Throwable payload size
    abi::emit_call_label(emitter, "__rt_heap_alloc");                           // allocate the Error object payload (rax = payload)
    emitter.instruction(&format!("mov r10, 0x{:x}", x86_64_heap_kind_word(6))); // magic + kind 6 identifies a throwable object
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the uniform heap header
    abi::emit_call_label(emitter, "__rt_object_handle_acquire");                // bind the Error to its PHP object handle
    abi::emit_load_symbol_to_reg(emitter, "r10", "_spl_error_class_id", 0);
    emitter.instruction("mov QWORD PTR [rax], r10");                            // stamp the per-program Error class id
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // recover the message pointer
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                       // recover the message byte length
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // message pointer
    emitter.instruction("mov QWORD PTR [rax + 16], r11");                       // message byte length
    // __rt_heap_alloc recycles blocks without zeroing, so every remaining slot is written here.
    emitter.instruction("mov QWORD PTR [rax + 24], 0");                         // code = 0
    emit_throwable_creation_line_unknown(emitter, "rax");
    emitter.instruction("mov QWORD PTR [rax + 40], 0");                         // previous = null
    abi::emit_store_reg_to_symbol(emitter, "rax", "_exc_value", 0);             // publish the Throwable for the unwinder
    emitter.instruction("mov rsp, rbp");                                        // release the local frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("jmp __rt_throw_current");                              // unwind, or report it uncaught and exit like PHP
}
