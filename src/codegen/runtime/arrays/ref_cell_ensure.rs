//! Purpose:
//! Emits `__rt_ref_cell_ensure`: the runtime-idempotent "get-or-promote" of a local's persistent
//! kind-6 reference cell. Given the local's current slot word, it returns that same word when it is
//! ALREADY a kind-6 reference cell (reuse), otherwise it allocates a fresh cell owning the value.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::arrays`.
//!
//! Key details:
//! - Backs the promote step of `&$var`: `$var` binds its ONE persistent cell, shared across every bind
//!   (Zend semantics). Because a loop body's single `&$var` statement runs many times, the promote MUST
//!   be idempotent at runtime — the first iteration promotes (`$var`'s slot then holds the cell), later
//!   iterations reuse the same cell. The compile-time ref-bound flag cannot express the loop back-edge,
//!   so the reuse-vs-promote decision is made here by inspecting the slot word's heap kind.
//! - Reuse check mirrors `__rt_incref`'s heap-range guard (`_heap_buf` .. `_heap_buf + _heap_off`) so a
//!   scalar/static word is never dereferenced; then it reads the uniform kind word at `[ptr-8]` and
//!   tests kind 6. A match returns the cell unchanged; otherwise it tail-calls `__rt_ref_cell_alloc`,
//!   which MOVES the value word into a fresh cell (the caller then stores the cell back into the slot,
//!   transferring ownership from the slot to the cell without an incref/decref).

use crate::codegen::abi::emit_symbol_address;
use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;

/// Low byte of the uniform kind word that marks a refcounted reference cell (heap kind 6). On x86_64
/// the kind word additionally carries the `"ELEPH"` heap marker in its high 32 bits, so only the low
/// 32 bits are compared here (the heap-range guard already proved the pointer is elephc-owned).
const REF_CELL_KIND: u64 = 6;

/// Emits the `__rt_ref_cell_ensure` runtime helper.
///
/// # ABI
/// - ARM64: in `x0` = slot value word, `x1` = inner value-tag; out `x0` = kind-6 reference cell pointer.
/// - x86_64: in `rdi` = slot value word, `rsi` = inner value-tag; out `rax` = kind-6 reference cell pointer.
pub fn emit_ref_cell_ensure(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ref_cell_ensure_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ref_cell_ensure ---");
    emitter.label_global("__rt_ref_cell_ensure");

    // -- heap range check: values outside the managed heap are never a cell → promote --
    emitter.instruction("cbz x0, __rt_ref_cell_ensure_alloc");                  // a null/zero word cannot be a cell; allocate one owning it
    emit_symbol_address(emitter, "x9", "_heap_buf");
    emitter.instruction("cmp x0, x9");                                          // is the slot word below the managed heap start?
    emitter.instruction("b.lo __rt_ref_cell_ensure_alloc");                     // below the heap → scalar/static value, promote it
    emit_symbol_address(emitter, "x10", "_heap_off");
    emitter.instruction("ldr x10, [x10]");                                      // load the current heap bump extent
    emitter.instruction("add x10, x9, x10");                                    // heap_end = _heap_buf + _heap_off
    emitter.instruction("cmp x0, x10");                                         // is the slot word at or beyond the heap end?
    emitter.instruction("b.hs __rt_ref_cell_ensure_alloc");                     // outside the heap window → promote it

    // -- reuse an existing kind-6 cell in place --
    emitter.instruction("ldr x9, [x0, #-8]");                                   // load the uniform heap kind word
    emitter.instruction(&format!("cmp x9, #{}", REF_CELL_KIND));                // is the slot word already a kind-6 reference cell?
    emitter.instruction("b.ne __rt_ref_cell_ensure_alloc");                     // a different heap kind (array/hash/object) → promote it
    emitter.instruction("ret");                                                 // reuse: x0 already holds the shared reference cell

    emitter.label("__rt_ref_cell_ensure_alloc");
    emitter.instruction("b __rt_ref_cell_alloc");                               // move the value word into a fresh cell (x0=value, x1=tag → x0=cell)
}

/// Emits the x86_64 Linux variant of `__rt_ref_cell_ensure`.
fn emit_ref_cell_ensure_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ref_cell_ensure ---");
    emitter.label_global("__rt_ref_cell_ensure");

    // -- heap range check: values outside the managed heap are never a cell → promote --
    emitter.instruction("test rdi, rdi");                                       // a null/zero word cannot be a cell
    emitter.instruction("jz __rt_ref_cell_ensure_alloc");                       // allocate a fresh cell owning the zero value
    emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("cmp rdi, r10");                                        // is the slot word below the managed heap start?
    emitter.instruction("jb __rt_ref_cell_ensure_alloc");                       // below the heap → scalar/static value, promote it
    emit_symbol_address(emitter, "r11", "_heap_off");
    emitter.instruction("mov r11, QWORD PTR [r11]");                            // load the current heap bump extent
    emitter.instruction("add r11, r10");                                        // heap_end = _heap_buf + _heap_off
    emitter.instruction("cmp rdi, r11");                                        // is the slot word at or beyond the heap end?
    emitter.instruction("jae __rt_ref_cell_ensure_alloc");                      // outside the heap window → promote it

    // -- reuse an existing kind-6 cell in place --
    emitter.instruction("mov r10d, DWORD PTR [rdi - 8]");                       // load the low 32 bits of the uniform kind word
    emitter.instruction(&format!("cmp r10d, {}", REF_CELL_KIND));               // is the slot word already a kind-6 reference cell?
    emitter.instruction("jne __rt_ref_cell_ensure_alloc");                      // a different heap kind (array/hash/object) → promote it
    emitter.instruction("mov rax, rdi");                                        // reuse: return the shared reference cell unchanged
    emitter.instruction("ret");                                                 // done — no allocation

    emitter.label("__rt_ref_cell_ensure_alloc");
    emitter.instruction("jmp __rt_ref_cell_alloc");                             // move the value word into a fresh cell (rdi=value, rsi=tag → rax=cell)
}
