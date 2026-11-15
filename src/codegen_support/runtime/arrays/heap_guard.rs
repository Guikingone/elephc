//! Purpose:
//! Shared x86_64 guard used by the refcount helpers before they touch a heap header.
//!
//! Called from:
//! - `__rt_incref`, `__rt_decref_mixed`, `__rt_decref_hash`, `__rt_decref_array` emitters.
//!
//! Key details:
//! - One unsigned offset compare (`ptr - _heap_buf < _heap_off`) rejects null, the
//!   in-band null sentinel, static data and pointers past the live heap; the heap marker
//!   is then compared straight from the high word of the uniform header.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;

/// Branches to `skip` unless `rax` points at a live elephc heap payload whose offset from
/// `_heap_buf` is at least `min_offset`. Clobbers `r10`, `r11` and the flags.
pub(crate) fn emit_x86_64_heap_owner_guard(emitter: &mut Emitter, skip: &str, min_offset: u32) {
    abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("mov r11, rax");                                        // candidate heap payload pointer
    emitter.instruction("sub r11, r10");                                        // offset into the managed heap (null and low pointers wrap to huge values)
    abi::emit_load_symbol_to_reg(emitter, "r10", "_heap_off", 0);
    emitter.instruction("cmp r11, r10");                                        // is the pointer outside the live heap window?
    emitter.instruction(&format!("jae {}", skip));                              // non-heap values own no refcounted storage
    if min_offset > 0 {
        emitter.instruction(&format!("cmp r11, {}", min_offset));               // the first user payload begins after the initial heap header
        emitter.instruction(&format!("jb {}", skip));                           // header-adjacent pointers are not heap payloads
    }
    emitter.instruction(&format!("cmp DWORD PTR [rax - 4], 0x{:x}", crate::codegen_support::sentinels::X86_64_HEAP_MAGIC_HI32)); // high word of the uniform header carries the elephc heap marker
    emitter.instruction(&format!("jne {}", skip));                              // foreign or static pointers carry no elephc heap header
}
