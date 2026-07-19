//! Purpose:
//! Emits the `__rt_closure_bind` runtime helper that implements PHP's
//! `Closure::bind` / `Closure::bindTo` / `Closure::call` for any closure capture
//! shape: captureless closures, closures with any number of by-value and
//! by-reference captures, and (among them) an implicit `$this` capture.
//!
//! Called from:
//! - `crate::codegen_ir::lower_inst::builtins::lower_closure_bind`.
//!
//! Key details:
//! - The descriptor environment record (`crate::codegen::callable_descriptor::
//!   environment_record`) carries a per-capture binding table (name, type tag,
//!   by-ref flag) plus a fifth `is_static` word; this helper walks that table by
//!   runtime capture count instead of assuming a single `$this` capture.
//! - Rebinding copies the 64-byte static header plus every capture slot into a
//!   fresh `64 + count*16`-byte descriptor. The capture NAMED `this` (PHP auto-
//!   captures it for any non-static closure that references `$this`, appended by
//!   `crate::ir_lower::function::lower_closure_function_with_signature`) is
//!   overwritten with the new receiver instead of copied; every other capture is
//!   copied by the same by-value/by-ref rule its ORIGINAL creation used:
//!     - by-ref (`use (&$x)`): the raw pointer word is copied verbatim with no
//!       retain, so the bound closure SHARES the same ref-cell as the source —
//!       matching `crate::codegen::runtime::callables::descriptor_release`, which
//!       likewise never releases by-ref capture slots (the cell's owner is the
//!       promoting local scope, not the descriptor).
//!     - string (tag 1): re-persisted via `__rt_str_persist` into an independently
//!       owned heap copy, mirroring how descriptor release frees an owned string
//!       capture directly (`__rt_heap_free_safe`) rather than through refcounting.
//!     - every other by-value tag (int/float/bool/array/hash/object/mixed/
//!       callable/iterable/pointer/…): the word(s) are copied verbatim and then
//!       unconditionally passed to `__rt_incref`, which self-guards on non-heap
//!       and null pointers, so scalars pass through as a no-op while heap values
//!       get a second independent owner — the source and bound descriptors can
//!       both be released without a double free or an early free of a value the
//!       other still references.
//! - A `static` closure (the environment record's `is_static` word) rejects a
//!   non-null new `$this` exactly like PHP: no fatal, no silent rebind — it emits
//!   PHP's own `E_WARNING` text and returns a null descriptor (`0`), matching
//!   `Closure::bind`'s `?Closure` return type (php-verified against PHP 8.5:
//!   `Closure::bind(static fn(){}, null, C::class)` succeeds; the same call with
//!   a non-null `$newThis` warns and returns `NULL`). Binding `null` onto a
//!   static closure (to only rebind scope) still succeeds.
//! - Verified on aarch64 (macOS/Linux) and x86_64 (Linux).

use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;
use crate::codegen::runtime::data::CLOSURE_BIND_STATIC_THIS_WARNING_MSG;

/// Emits the `__rt_closure_bind` runtime helper for the active target.
///
/// Input: `x0`/`rdi` = source closure descriptor pointer, `x1`/`rsi` = the new
/// `$this` object pointer, or `0` for no rebind (keep the closure's current
/// receiver, or rebind only the compile-time-erased scope). Output: `x0`/`rax` =
/// a freshly heap-allocated descriptor copy, or `0` when the source is a
/// `static` closure and the new `$this` is non-null (PHP's own null-return
/// divergence; see the module doc comment).
pub(crate) fn emit_closure_bind(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_closure_bind_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: closure bind (N captures) ---");
    emitter.label_global("__rt_closure_bind");

    // -- frame and argument save --
    emitter.instruction("sub sp, sp, #80");                                     // reserve closure-bind spill slots (src/this/dst/count/table/index)
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save frame pointer and return address across helper calls
    emitter.instruction("add x29, sp, #64");                                    // establish a frame pointer for the helper
    emitter.instruction("str x0, [sp, #0]");                                    // save the source descriptor pointer
    emitter.instruction("str x1, [sp, #8]");                                    // save the new $this receiver (0 = no rebind)

    // -- load environment metadata: capture count, capture table, is_static --
    emitter.instruction("ldr x9, [x0, #40]");                                   // x9 = descriptor environment record pointer (0 if none)
    emitter.instruction("cbz x9, __rt_closure_bind_no_env");                    // no captures/hidden/static flag at all
    emitter.instruction("ldr x10, [x9]");                                       // x10 = capture count
    emitter.instruction("str x10, [sp, #24]");                                  // save capture count for the copy loop
    emitter.instruction("ldr x11, [x9, #16]");                                  // x11 = capture binding metadata table
    emitter.instruction("str x11, [sp, #32]");                                  // save capture table pointer for the copy loop
    emitter.instruction("ldr x12, [x9, #32]");                                  // x12 = is_static flag (fifth environment word)
    emitter.instruction("b __rt_closure_bind_static_check");                    // continue to the static-closure rejection check
    emitter.label("__rt_closure_bind_no_env");
    emitter.instruction("str xzr, [sp, #24]");                                  // no environment record means zero captures
    emitter.instruction("str xzr, [sp, #32]");                                  // and no capture table to walk
    emitter.instruction("mov x12, #0");                                         // and never a static closure

    // -- reject binding a non-null $this onto a static closure (PHP's own divergence) --
    //
    // A literal `null` argument lowers through `Op::ConstNull` at `PhpType::Void`, which
    // (`crate::codegen_ir::lower_inst::lower_const_null`) always materializes as the in-band
    // `NULL_SENTINEL`, not a plain zero — so "no new $this" must be recognized in EITHER form.
    emitter.label("__rt_closure_bind_static_check");
    emitter.instruction("cbz x12, __rt_closure_bind_alloc");                    // not static: proceed normally
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload the new $this receiver
    emitter.instruction("cbz x1, __rt_closure_bind_alloc");                     // plain-zero $this is always allowed
    crate::codegen::abi::emit_load_int_immediate(
        emitter,
        "x2",
        crate::codegen::sentinels::NULL_SENTINEL,
    );
    emitter.instruction("cmp x1, x2");                                          // is $this the in-band null sentinel?
    emitter.instruction("b.eq __rt_closure_bind_alloc");                        // sentinel-null $this on a static closure is always allowed
    emitter.instruction("b __rt_closure_bind_reject");                          // static closure + non-null $this: warn and return null

    // -- allocate a fresh descriptor: 64-byte header + one 16-byte slot per capture --
    emitter.label("__rt_closure_bind_alloc");
    emitter.instruction("ldr x10, [sp, #24]");                                  // x10 = capture count
    emitter.instruction("mov x11, #16");                                        // each runtime capture slot is 16 bytes
    emitter.instruction("mul x10, x10, x11");                                   // x10 = capture payload bytes
    emitter.instruction("add x0, x10, #64");                                    // total size = static header + capture payload
    emitter.instruction("bl __rt_heap_alloc");                                  // x0 = fresh descriptor block
    emitter.instruction("str x0, [sp, #16]");                                   // save the new descriptor pointer

    // -- copy the 64-byte static header --
    emitter.instruction("ldr x1, [sp, #0]");                                    // x1 = source descriptor
    emitter.instruction("ldp x2, x3, [x1, #0]");                                // copy header words 0-1 (kind, entry)
    emitter.instruction("stp x2, x3, [x0, #0]");                                // store header words 0-1
    emitter.instruction("ldp x2, x3, [x1, #16]");                               // copy header words 2-3 (name, name_len)
    emitter.instruction("stp x2, x3, [x0, #16]");                               // store header words 2-3
    emitter.instruction("ldp x2, x3, [x1, #32]");                               // copy header words 4-5 (signature, environment)
    emitter.instruction("stp x2, x3, [x0, #32]");                               // store header words 4-5
    emitter.instruction("ldp x2, x3, [x1, #48]");                               // copy header words 6-7 (invocation, invoker)
    emitter.instruction("stp x2, x3, [x0, #48]");                               // store header words 6-7
    emitter.instruction("str xzr, [sp, #40]");                                  // loop index i = 0

    // -- walk capture metadata and materialize each slot into the new descriptor --
    emitter.label("__rt_closure_bind_loop");
    emitter.instruction("ldr x12, [sp, #40]");                                  // reload current capture index
    emitter.instruction("ldr x13, [sp, #24]");                                  // reload total capture count
    emitter.instruction("cmp x12, x13");                                        // have all capture slots been processed?
    emitter.instruction("b.hs __rt_closure_bind_done");                         // yes — the bound descriptor is complete
    emitter.instruction("ldr x14, [sp, #32]");                                  // reload capture binding table pointer
    emitter.instruction("mov x15, #32");                                        // each capture binding entry is four 8-byte words
    emitter.instruction("mul x15, x12, x15");                                   // compute byte offset for this capture metadata entry
    emitter.instruction("add x14, x14, x15");                                   // x14 = capture metadata entry pointer
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload source descriptor pointer
    emitter.instruction("mov x10, #16");                                        // each runtime capture slot is 16 bytes
    emitter.instruction("mul x10, x12, x10");                                   // compute this capture slot's byte offset
    emitter.instruction("add x10, x10, #64");                                   // skip the 64-byte static descriptor header
    emitter.instruction("add x9, x9, x10");                                     // x9 = source capture slot address
    emitter.instruction("ldr x15, [x14, #24]");                                 // load by-ref flag for this capture
    emitter.instruction("cbnz x15, __rt_closure_bind_ref_copy");                // by-ref captures borrow an external cell: copy the pointer, no retain
    emitter.instruction("ldr x15, [x14, #8]");                                  // load capture name length
    emitter.instruction("cmp x15, #4");                                         // "this" is four bytes long
    emitter.instruction("b.ne __rt_closure_bind_check_string");                 // a different-length name cannot be $this
    emitter.instruction("ldr x16, [x14, #0]");                                  // x16 = capture name byte pointer
    emitter.instruction("ldr w17, [x16]");                                      // load the first four name bytes
    emitter.instruction("movz w0, #0x6874");                                    // low half of "this" little-endian ("th")
    emitter.instruction("movk w0, #0x7369, lsl #16");                           // high half of "this" little-endian ("is")
    emitter.instruction("cmp w17, w0");                                         // is this capture named "this"?
    emitter.instruction("b.eq __rt_closure_bind_this_capture");                 // rebind the $this capture to the new receiver
    emitter.label("__rt_closure_bind_check_string");
    emitter.instruction("ldr x15, [x14, #16]");                                 // load this capture's descriptor type tag
    emitter.instruction("cmp x15, #1");                                         // is this a string capture?
    emitter.instruction("b.eq __rt_closure_bind_string_capture");               // strings need an independently owned heap copy
    emitter.instruction("b __rt_closure_bind_generic_capture");                 // every other by-value tag copies verbatim then increfs

    // -- by-reference capture: share the same cell, no retain (see module doc comment) --
    emitter.label("__rt_closure_bind_ref_copy");
    emitter.instruction("ldp x0, x1, [x9]");                                    // load the source capture slot words (a ref-cell pointer)
    emitter.instruction("bl __rt_closure_bind_store_slot");                     // store both words verbatim into the new descriptor's slot
    emitter.instruction("b __rt_closure_bind_next");

    // -- generic by-value capture: copy verbatim, then retain (no-op for non-heap words) --
    emitter.label("__rt_closure_bind_generic_capture");
    emitter.instruction("ldp x0, x1, [x9]");                                    // load the source capture slot words
    emitter.instruction("bl __rt_closure_bind_store_slot");                     // store both words into the new descriptor's slot
    emitter.instruction("ldr x0, [sp, #48]");                                   // reload the stored low word for the retain call
    emitter.instruction("bl __rt_incref");                                      // retain a shared heap value; safely skips scalars/null
    emitter.instruction("b __rt_closure_bind_next");

    // -- string capture: persist an independently owned copy --
    emitter.label("__rt_closure_bind_string_capture");
    emitter.instruction("ldp x1, x2, [x9]");                                    // x1 = source string pointer, x2 = length
    emitter.instruction("bl __rt_str_persist");                                 // x1 = owned heap copy, x2 = length (unchanged)
    emitter.instruction("mov x0, x1");                                          // move persisted pointer into the generic slot-store input
    emitter.instruction("mov x1, x2");                                          // move length into the generic slot-store input
    emitter.instruction("bl __rt_closure_bind_store_slot");                     // store the persisted pointer/length into the new descriptor's slot
    emitter.instruction("b __rt_closure_bind_next");

    // -- $this capture: overwrite with the new receiver instead of copying --
    emitter.label("__rt_closure_bind_this_capture");
    emitter.instruction("ldr x15, [x14, #16]");                                 // reload this capture's descriptor type tag
    emitter.instruction("cmp x15, #7");                                         // a Mixed capture stores a boxed cell, not a raw object
    emitter.instruction("b.eq __rt_closure_bind_this_mixed");                   // top-level closures use a Mixed $this receiver
    emitter.instruction("ldr x0, [sp, #8]");                                    // x0 = new $this receiver
    emitter.instruction("bl __rt_incref");                                      // the bound descriptor now owns a reference to $this (no-op if null)
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the new $this receiver after the retain call
    emitter.instruction("mov x1, #0");                                          // object captures leave the slot's second word unused
    emitter.instruction("bl __rt_closure_bind_store_slot");                     // store the new receiver into the new descriptor's slot
    emitter.instruction("b __rt_closure_bind_next");
    emitter.label("__rt_closure_bind_this_mixed");
    emitter.instruction("mov x0, #6");                                          // boxed payload tag 6 = object
    emitter.instruction("ldr x1, [sp, #8]");                                    // payload low word = new $this object pointer
    emitter.instruction("mov x2, #0");                                          // payload high word is unused for objects
    emitter.instruction("bl __rt_mixed_from_value");                            // box (and retain) the receiver into a Mixed cell
    emitter.instruction("mov x1, #0");                                          // boxed captures leave the slot's second word unused
    emitter.instruction("bl __rt_closure_bind_store_slot");                     // store the boxed Mixed receiver into the new descriptor's slot

    emitter.label("__rt_closure_bind_next");
    emitter.instruction("ldr x12, [sp, #40]");                                  // reload current capture index after any nested helper call
    emitter.instruction("add x12, x12, #1");                                    // advance to the next capture slot
    emitter.instruction("str x12, [sp, #40]");                                  // persist the updated capture index
    emitter.instruction("b __rt_closure_bind_loop");                            // continue walking capture metadata

    // -- return the new descriptor --
    emitter.label("__rt_closure_bind_done");
    emitter.instruction("ldr x0, [sp, #16]");                                   // x0 = bound descriptor result
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // tear down the closure-bind frame
    emitter.instruction("ret");                                                 // return the rebound closure descriptor

    // -- static closure rejected a non-null $this: warn and return null --
    emitter.label("__rt_closure_bind_reject");
    crate::codegen::abi::emit_symbol_address(emitter, "x1", "_diag_closure_bind_static_this_msg");
    emitter.instruction(&format!("mov x2, #{}", CLOSURE_BIND_STATIC_THIS_WARNING_MSG.len())); // byte length of PHP's own warning text
    emitter.instruction("bl __rt_diag_warning");                                // emit or suppress the static-closure-bind warning
    emitter.instruction("mov x0, #0");                                          // null result, matching Closure::bind's ?Closure return type
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // tear down the closure-bind frame
    emitter.instruction("ret");                                                 // return null to the caller

    emit_closure_bind_store_slot(emitter);
}

/// Emits the shared `__rt_closure_bind_store_slot` leaf helper: stores `x0`/`x1` into the
/// new descriptor's current capture slot (`[sp, #16]` base, `[sp, #40]` index), and also
/// leaves the stored low word in `[sp, #48]` for callers that need to retain it afterward
/// (`__rt_closure_bind_generic_capture`). A tiny `bl`-callable leaf keeps the four call sites
/// above from duplicating this address computation inline.
fn emit_closure_bind_store_slot(emitter: &mut Emitter) {
    emitter.label("__rt_closure_bind_store_slot");
    emitter.instruction("str x0, [sp, #48]");                                   // save the low word for callers that retain it afterward
    emitter.instruction("ldr x9, [sp, #16]");                                   // x9 = new descriptor pointer
    emitter.instruction("ldr x10, [sp, #40]");                                  // x10 = current capture index
    emitter.instruction("mov x11, #16");                                        // each runtime capture slot is 16 bytes
    emitter.instruction("mul x10, x10, x11");                                   // compute this capture slot's byte offset
    emitter.instruction("add x10, x10, #64");                                   // skip the 64-byte static descriptor header
    emitter.instruction("add x9, x9, x10");                                     // x9 = destination capture slot address
    emitter.instruction("stp x0, x1, [x9]");                                    // store both capture slot words
    emitter.instruction("ret");                                                 // return to the capture-dispatch call site
}

/// Emits the Linux x86_64 `__rt_closure_bind` helper (mirror of the aarch64 path).
fn emit_closure_bind_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: closure bind (N captures, x86_64) ---");
    emitter.label_global("__rt_closure_bind");

    // -- frame and argument save --
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish this helper's frame
    emitter.instruction("sub rsp, 64");                                         // reserve spill slots (src/this/dst/count/table/index, 16-byte aligned)
    emitter.instruction("mov [rsp+0], rdi");                                    // save the source descriptor pointer
    emitter.instruction("mov [rsp+8], rsi");                                    // save the new $this receiver (0 = no rebind)

    // -- load environment metadata: capture count, capture table, is_static --
    emitter.instruction("mov r8, [rdi+40]");                                    // r8 = descriptor environment record pointer (0 if none)
    emitter.instruction("test r8, r8");                                         // does the descriptor carry an environment record?
    emitter.instruction("jz __rt_closure_bind_no_env");                         // no captures/hidden/static flag at all
    emitter.instruction("mov r9, [r8]");                                        // r9 = capture count
    emitter.instruction("mov [rsp+24], r9");                                    // save capture count for the copy loop
    emitter.instruction("mov r9, [r8+16]");                                     // r9 = capture binding metadata table
    emitter.instruction("mov [rsp+32], r9");                                    // save capture table pointer for the copy loop
    emitter.instruction("mov r9, [r8+32]");                                     // r9 = is_static flag (fifth environment word)
    emitter.instruction("jmp __rt_closure_bind_static_check");                  // continue to the static-closure rejection check
    emitter.label("__rt_closure_bind_no_env");
    emitter.instruction("mov QWORD PTR [rsp+24], 0");                           // no environment record means zero captures
    emitter.instruction("mov QWORD PTR [rsp+32], 0");                           // and no capture table to walk
    emitter.instruction("xor r9, r9");                                          // and never a static closure

    // -- reject binding a non-null $this onto a static closure (PHP's own divergence) --
    //
    // A literal `null` argument lowers through `Op::ConstNull` at `PhpType::Void`, which
    // (`crate::codegen_ir::lower_inst::lower_const_null`) always materializes as the in-band
    // `NULL_SENTINEL`, not a plain zero — so "no new $this" must be recognized in EITHER form.
    emitter.label("__rt_closure_bind_static_check");
    emitter.instruction("test r9, r9");                                         // is the source closure static?
    emitter.instruction("jz __rt_closure_bind_alloc");                          // not static: proceed normally
    emitter.instruction("mov r9, [rsp+8]");                                     // reload the new $this receiver
    emitter.instruction("test r9, r9");                                         // is the new $this plain-zero?
    emitter.instruction("jz __rt_closure_bind_alloc");                          // plain-zero $this is always allowed
    crate::codegen::abi::emit_load_int_immediate(
        emitter,
        "r10",
        crate::codegen::sentinels::NULL_SENTINEL,
    );
    emitter.instruction("cmp r9, r10");                                         // is $this the in-band null sentinel?
    emitter.instruction("je __rt_closure_bind_alloc");                          // sentinel-null $this on a static closure is always allowed
    emitter.instruction("jmp __rt_closure_bind_reject");                        // static closure + non-null $this: warn and return null

    // -- allocate a fresh descriptor: 64-byte header + one 16-byte slot per capture --
    emitter.label("__rt_closure_bind_alloc");
    emitter.instruction("mov rax, [rsp+24]");                                   // rax = capture count
    emitter.instruction("shl rax, 4");                                          // rax = capture payload bytes (count * 16)
    emitter.instruction("add rax, 64");                                         // total size = static header + capture payload
    emitter.instruction("call __rt_heap_alloc");                                // rax = fresh descriptor block
    emitter.instruction("mov [rsp+16], rax");                                   // save the new descriptor pointer

    // -- copy the 64-byte static header --
    emitter.instruction("mov rsi, [rsp+0]");                                    // rsi = source descriptor
    emitter.instruction("mov rdi, rax");                                        // rdi = destination descriptor
    emitter.instruction("mov rcx, 8");                                          // 64 bytes = eight 8-byte words
    emitter.instruction("cld");                                                 // copy forward
    emitter.instruction("rep movsq");                                           // copy the static descriptor header word by word
    emitter.instruction("mov QWORD PTR [rsp+40], 0");                           // loop index i = 0

    // -- walk capture metadata and materialize each slot into the new descriptor --
    emitter.label("__rt_closure_bind_loop");
    emitter.instruction("mov r10, [rsp+40]");                                   // reload current capture index
    emitter.instruction("cmp r10, [rsp+24]");                                   // have all capture slots been processed?
    emitter.instruction("jae __rt_closure_bind_done");                          // yes — the bound descriptor is complete
    emitter.instruction("mov r11, [rsp+32]");                                   // reload capture binding table pointer
    emitter.instruction("mov rcx, r10");                                        // copy index before scaling the metadata offset
    emitter.instruction("shl rcx, 5");                                          // each capture binding entry is 32 bytes
    emitter.instruction("add r11, rcx");                                        // r11 = capture metadata entry pointer
    emitter.instruction("mov r8, [rsp+0]");                                     // reload source descriptor pointer
    emitter.instruction("mov rcx, r10");                                        // copy index before scaling the capture slot offset
    emitter.instruction("shl rcx, 4");                                          // each runtime capture slot is 16 bytes
    emitter.instruction("add r8, rcx");                                         // skip to this capture slot's byte offset
    emitter.instruction("add r8, 64");                                          // skip the 64-byte static descriptor header
    emitter.instruction("mov [rsp+48], r8");                                    // save source capture slot address for this iteration
    emitter.instruction("mov rax, [r11+24]");                                   // load by-ref flag for this capture
    emitter.instruction("test rax, rax");                                       // does this slot borrow an external reference cell?
    emitter.instruction("jnz __rt_closure_bind_ref_copy");                      // by-ref captures borrow an external cell: copy the pointer, no retain
    emitter.instruction("mov rax, [r11+8]");                                    // load capture name length
    emitter.instruction("cmp rax, 4");                                          // "this" is four bytes long
    emitter.instruction("jne __rt_closure_bind_check_string");                  // a different-length name cannot be $this
    emitter.instruction("mov rax, [r11]");                                      // rax = capture name byte pointer
    emitter.instruction("mov eax, [rax]");                                      // load the first four name bytes
    emitter.instruction("cmp eax, 0x73696874");                                 // compare against "this" little-endian
    emitter.instruction("je __rt_closure_bind_this_capture");                   // rebind the $this capture to the new receiver
    emitter.label("__rt_closure_bind_check_string");
    emitter.instruction("mov rax, [r11+16]");                                   // load this capture's descriptor type tag
    emitter.instruction("cmp rax, 1");                                          // is this a string capture?
    emitter.instruction("je __rt_closure_bind_string_capture");                 // strings need an independently owned heap copy
    emitter.instruction("jmp __rt_closure_bind_generic_capture");               // every other by-value tag copies verbatim then increfs

    // -- by-reference capture: share the same cell, no retain (see module doc comment) --
    emitter.label("__rt_closure_bind_ref_copy");
    emitter.instruction("mov r8, [rsp+48]");                                    // reload source capture slot address
    emitter.instruction("mov rax, [r8]");                                       // rax = source slot low word (a ref-cell pointer)
    emitter.instruction("mov rdx, [r8+8]");                                     // rdx = source slot high word
    emitter.instruction("call __rt_closure_bind_store_slot");                   // store both words verbatim into the new descriptor's slot
    emitter.instruction("jmp __rt_closure_bind_next");

    // -- generic by-value capture: copy verbatim, then retain (no-op for non-heap words) --
    emitter.label("__rt_closure_bind_generic_capture");
    emitter.instruction("mov r8, [rsp+48]");                                    // reload source capture slot address
    emitter.instruction("mov rax, [r8]");                                       // rax = source slot low word
    emitter.instruction("mov rdx, [r8+8]");                                     // rdx = source slot high word
    emitter.instruction("call __rt_closure_bind_store_slot");                   // store both words into the new descriptor's slot
    emitter.instruction("mov rax, [rsp+56]");                                   // reload the stored low word for the retain call
    emitter.instruction("call __rt_incref");                                    // retain a shared heap value; safely skips scalars/null
    emitter.instruction("jmp __rt_closure_bind_next");

    // -- string capture: persist an independently owned copy --
    emitter.label("__rt_closure_bind_string_capture");
    emitter.instruction("mov r8, [rsp+48]");                                    // reload source capture slot address
    emitter.instruction("mov rax, [r8]");                                       // rax = source string pointer
    emitter.instruction("mov rdx, [r8+8]");                                     // rdx = source string length
    emitter.instruction("call __rt_str_persist");                               // rax = owned heap copy, rdx = length (unchanged)
    emitter.instruction("call __rt_closure_bind_store_slot");                   // store the persisted pointer/length into the new descriptor's slot
    emitter.instruction("jmp __rt_closure_bind_next");

    // -- $this capture: overwrite with the new receiver instead of copying --
    emitter.label("__rt_closure_bind_this_capture");
    emitter.instruction("mov r11, [rsp+32]");                                   // reload capture binding table pointer
    emitter.instruction("mov r10, [rsp+40]");                                   // reload current capture index
    emitter.instruction("mov rcx, r10");                                        // copy index before scaling the metadata offset
    emitter.instruction("shl rcx, 5");                                          // each capture binding entry is 32 bytes
    emitter.instruction("add r11, rcx");                                        // r11 = capture metadata entry pointer
    emitter.instruction("mov rax, [r11+16]");                                   // reload this capture's descriptor type tag
    emitter.instruction("cmp rax, 7");                                          // a Mixed capture stores a boxed cell, not a raw object
    emitter.instruction("je __rt_closure_bind_this_mixed");                     // top-level closures use a Mixed $this receiver
    emitter.instruction("mov rax, [rsp+8]");                                    // rax = new $this receiver
    emitter.instruction("call __rt_incref");                                    // the bound descriptor now owns a reference to $this (no-op if null)
    emitter.instruction("mov rax, [rsp+8]");                                    // reload the new $this receiver after the retain call
    emitter.instruction("xor edx, edx");                                        // object captures leave the slot's second word unused
    emitter.instruction("call __rt_closure_bind_store_slot");                   // store the new receiver into the new descriptor's slot
    emitter.instruction("jmp __rt_closure_bind_next");
    emitter.label("__rt_closure_bind_this_mixed");
    emitter.instruction("mov rax, 6");                                          // boxed payload tag 6 = object
    emitter.instruction("mov rdi, [rsp+8]");                                    // payload low word = new $this object pointer
    emitter.instruction("xor esi, esi");                                        // payload high word is unused for objects
    emitter.instruction("call __rt_mixed_from_value");                          // box (and retain) the receiver into a Mixed cell
    emitter.instruction("xor edx, edx");                                        // boxed captures leave the slot's second word unused
    emitter.instruction("call __rt_closure_bind_store_slot");                   // store the boxed Mixed receiver into the new descriptor's slot

    emitter.label("__rt_closure_bind_next");
    emitter.instruction("mov r10, [rsp+40]");                                   // reload current capture index after any nested helper call
    emitter.instruction("add r10, 1");                                          // advance to the next capture slot
    emitter.instruction("mov [rsp+40], r10");                                   // persist the updated capture index
    emitter.instruction("jmp __rt_closure_bind_loop");                          // continue walking capture metadata

    // -- return the new descriptor --
    emitter.label("__rt_closure_bind_done");
    emitter.instruction("mov rax, [rsp+16]");                                   // rax = bound descriptor result
    emitter.instruction("mov rsp, rbp");                                        // tear down the closure-bind frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the rebound closure descriptor

    // -- static closure rejected a non-null $this: warn and return null --
    emitter.label("__rt_closure_bind_reject");
    crate::codegen::abi::emit_symbol_address(emitter, "rdi", "_diag_closure_bind_static_this_msg");
    emitter.instruction(&format!("mov esi, {}", CLOSURE_BIND_STATIC_THIS_WARNING_MSG.len())); // byte length of PHP's own warning text
    emitter.instruction("call __rt_diag_warning");                              // emit or suppress the static-closure-bind warning
    emitter.instruction("xor eax, eax");                                        // null result, matching Closure::bind's ?Closure return type
    emitter.instruction("mov rsp, rbp");                                        // tear down the closure-bind frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return null to the caller

    emit_closure_bind_store_slot_x86_64(emitter);
}

/// Emits the shared `__rt_closure_bind_store_slot` leaf helper (x86_64): stores `rax`/`rdx`
/// into the new descriptor's current capture slot (`[rsp+16]` base, `[rsp+40]` index), and
/// also leaves the stored low word in `[rsp+56]` for callers that need to retain it
/// afterward (`__rt_closure_bind_generic_capture`).
fn emit_closure_bind_store_slot_x86_64(emitter: &mut Emitter) {
    emitter.label("__rt_closure_bind_store_slot");
    emitter.instruction("mov [rsp+56], rax");                                   // save the low word for callers that retain it afterward
    emitter.instruction("mov r8, [rsp+16]");                                    // r8 = new descriptor pointer
    emitter.instruction("mov r9, [rsp+40]");                                    // r9 = current capture index
    emitter.instruction("shl r9, 4");                                           // each runtime capture slot is 16 bytes
    emitter.instruction("add r8, r9");                                          // skip to this capture slot's byte offset
    emitter.instruction("add r8, 64");                                          // skip the 64-byte static descriptor header
    emitter.instruction("mov [r8], rax");                                       // store the capture slot low word
    emitter.instruction("mov [r8+8], rdx");                                     // store the capture slot high word
    emitter.instruction("ret");                                                 // return to the capture-dispatch call site
}
