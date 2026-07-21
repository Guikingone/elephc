//! Purpose:
//! Emits `__rt_ref_cell_ensure`: the runtime-idempotent "get-or-promote" of a local's persistent
//! kind-7 reference cell. Given the local's current slot word, it returns that same word when it is
//! ALREADY a kind-7 reference cell (reuse), otherwise it allocates a fresh cell owning the value.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Backs the promote step of `&$var`: `$var` binds its ONE persistent cell, shared across every bind
//!   (Zend semantics). Because a loop body's single `&$var` statement runs many times, the promote MUST
//!   be idempotent at runtime — the first iteration promotes (`$var`'s slot then holds the cell), later
//!   iterations reuse the same cell. The compile-time ref-bound flag cannot express the loop back-edge,
//!   so the reuse-vs-promote decision is made here by inspecting the slot word's heap kind.
//! - Reuse check mirrors `__rt_incref`'s heap-range guard (`_heap_buf` .. `_heap_buf + _heap_off`) so a
//!   scalar/static word is never dereferenced; then it reads the uniform kind word at `[ptr-8]`,
//!   masks it to the low byte (bit 16 is the cycle collector's transient reachable mark, left set on
//!   surviving blocks after `__rt_gc_collect_cycles`; an unmasked compare would wrap a live cell in a
//!   second cell and corrupt every later load/store through the slot), and tests kind 7. A match
//!   returns the cell unchanged; otherwise it tail-calls `__rt_ref_cell_alloc`,
//!   which MOVES the value word into a fresh cell (the caller then stores the cell back into the slot,
//!   transferring ownership from the slot to the cell without an incref/decref).

use crate::codegen::abi::emit_symbol_address;
use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Low byte of the uniform kind word that marks a refcounted reference cell (heap kind 7). On x86_64
/// the kind word additionally carries the `"ELEPH"` heap marker in its high 32 bits, so only the low
/// 32 bits are compared here (the heap-range guard already proved the pointer is elephc-owned).
const REF_CELL_KIND: u64 = 7;

/// Emits the `__rt_ref_cell_ensure` runtime helper.
///
/// # ABI
/// - ARM64: in `x0` = slot value word, `x1` = inner value-tag; out `x0` = kind-7 reference cell pointer.
/// - x86_64: in `rdi` = slot value word, `rsi` = inner value-tag; out `rax` = kind-7 reference cell pointer.
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

    // -- reuse an existing kind-7 cell in place --
    emitter.instruction("ldr x9, [x0, #-8]");                                   // load the uniform heap kind word
    emitter.instruction("and x9, x9, #0xff");                                   // isolate the low-byte kind: bit 16 is the transient GC reachable mark
    emitter.instruction(&format!("cmp x9, #{}", REF_CELL_KIND));                // is the slot word already a kind-7 reference cell?
    emitter.instruction("b.ne __rt_ref_cell_ensure_probe");                     // a different heap kind → probe for Mixed-box tag override
    emitter.instruction("ret");                                                 // reuse: x0 already holds the shared reference cell

    // -- probe: when the value is a Mixed box (heap kind 5), the compile-time tag may be wrong --
    // store_local boxes adopted-ref-bound values as Mixed before local_ref_ensure, but the
    // instruction's result_php_type is still the declared type (e.g. int → tag 0). If the cell
    // is created with tag 0 but holds a Mixed box, __rt_ref_cell_store/free_deep will skip the
    // inner release (tag < 4 = scalar, no heap storage) and leak the Mixed box. Override tag to
    // 7 (Mixed) when the heap kind is 5 so the cell's inner_tag matches the actual value shape.
    emitter.label("__rt_ref_cell_ensure_probe");
    emitter.instruction("cmp x9, #5");                                          // is the value a Mixed box (heap kind 5)?
    emitter.instruction("b.ne __rt_ref_cell_ensure_alloc");                     // not a Mixed box → use the compile-time tag as-is
    emitter.instruction("mov x1, #7");                                          // override inner tag to Mixed (7) to match the boxed value
    emitter.instruction("b __rt_ref_cell_alloc");                               // move the value word into a fresh cell (x0=value, x1=7 → x0=cell)

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

    // -- reuse an existing kind-7 cell in place --
    emitter.instruction("mov r10d, DWORD PTR [rdi - 8]");                       // load the low 32 bits of the uniform kind word
    emitter.instruction("and r10d, 0xff");                                      // isolate the low-byte kind: bit 16 is the transient GC reachable mark
    emitter.instruction(&format!("cmp r10d, {}", REF_CELL_KIND));               // is the slot word already a kind-7 reference cell?
    emitter.instruction("jne __rt_ref_cell_ensure_probe");                      // a different heap kind → probe for Mixed-box tag override
    emitter.instruction("mov rax, rdi");                                        // reuse: return the shared reference cell unchanged
    emitter.instruction("ret");                                                 // done — no allocation

    // -- probe: when the value is a Mixed box (heap kind 5), override the inner tag to 7 (Mixed) --
    // store_local boxes adopted-ref-bound values as Mixed before local_ref_ensure, but the
    // instruction's result_php_type is still the declared type (e.g. int → tag 0). Without the
    // override, the cell's inner_tag would say Int while holding a Mixed box, causing leaks on
    // later __rt_ref_cell_store/free_deep (tag < 4 → skip inner release).
    emitter.label("__rt_ref_cell_ensure_probe");
    emitter.instruction("cmp r10d, 5");                                         // is the value a Mixed box (heap kind 5)?
    emitter.instruction("jne __rt_ref_cell_ensure_alloc");                      // not a Mixed box → use the compile-time tag as-is
    emitter.instruction("mov esi, 7");                                          // override inner tag to Mixed (7) to match the boxed value
    emitter.instruction("jmp __rt_ref_cell_alloc");                             // move the value word into a fresh cell (rdi=value, rsi=7 → rax=cell)

    emitter.label("__rt_ref_cell_ensure_alloc");
    emitter.instruction("jmp __rt_ref_cell_alloc");                             // move the value word into a fresh cell (rdi=value, rsi=tag → rax=cell)
}
