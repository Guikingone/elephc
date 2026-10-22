//! Purpose:
//! Maintains the allocator's exact live payload-base bitmap.
//!
//! Called from:
//! - Heap allocation/free emitters and the web arena reset emitter.
//!
//! Key details:
//! - One bit represents each 8-byte heap granule, outside user-controlled payloads.
//! - Allocation, free and bulk reset keep candidate-base checks constant-time.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Marks or retires the payload in x0/rax, preserving the allocator header in x10/r10.
pub(crate) fn emit_update(emitter: &mut Emitter, live: bool) {
    if emitter.target.arch == Arch::X86_64 {
        // -- locate and update the exact payload-base bit --
        abi::emit_symbol_address(emitter, "r8", "_heap_buf");
        emitter.instruction("mov rdx, rax");                                    // copy the payload before deriving its heap-relative offset
        emitter.instruction("sub rdx, r8");                                     // derive the payload's heap-relative byte offset
        if !live {
            emitter.instruction("cmp rdx, 16");                                 // reject pointers below the first managed payload
            emitter.instruction("jb __rt_heap_free_base_retired");              // leave foreign pointers outside the side table
            abi::emit_symbol_address(emitter, "r8", "_heap_off");
            emitter.instruction("cmp rdx, QWORD PTR [r8]");                     // require a payload inside the current heap extent
            emitter.instruction("jae __rt_heap_free_base_retired");             // avoid indexing the bitmap with a foreign pointer
        }
        emitter.instruction("mov rcx, rdx");                                    // preserve the offset for the granule-bit calculation
        emitter.instruction("shr rdx, 6");                                      // select one bitmap byte for each 64 heap bytes
        emitter.instruction("shr rcx, 3");                                      // convert the byte offset to its 8-byte granule index
        emitter.instruction("and ecx, 7");                                      // select the payload-base bit within that bitmap byte
        emitter.instruction("mov r9d, 1");                                      // begin with the low bit set
        emitter.instruction("shl r9d, cl");                                     // move the marker to this payload's granule bit
        abi::emit_symbol_address(emitter, "r8", "_heap_live_bases");
        if live {
            emitter.instruction("or BYTE PTR [r8 + rdx], r9b");                 // publish this exact live payload base
        } else {
            emitter.instruction("not r9d");                                     // invert the one-bit mask for clearing
            emitter.instruction("and BYTE PTR [r8 + rdx], r9b");                // retire this base before allocator reuse
            emitter.label("__rt_heap_free_base_retired");
        }
        return;
    }
    // -- locate and update the exact payload-base bit --
    abi::emit_symbol_address(emitter, "x12", "_heap_buf");
    emitter.instruction("sub x13, x0, x12");                                    // derive the payload's heap-relative byte offset
    emitter.instruction("lsr x14, x13, #6");                                    // select one bitmap byte for each 64 heap bytes
    emitter.instruction("lsr x13, x13, #3");                                    // convert the byte offset to its 8-byte granule index
    emitter.instruction("and x13, x13, #7");                                    // select the payload-base bit within that bitmap byte
    emitter.instruction("mov w15, #1");                                         // begin with the low bit set
    emitter.instruction("lsl w15, w15, w13");                                   // move the marker to this payload's granule bit
    abi::emit_symbol_address(emitter, "x12", "_heap_live_bases");
    emitter.instruction("ldrb w16, [x12, x14]");                                // load the bitmap byte that owns this payload base
    if live {
        emitter.instruction("orr w16, w16, w15");                               // mark the exact payload base as live
    } else {
        emitter.instruction("bic w16, w16, w15");                               // clear the marker before the storage can be reused
    }
    emitter.instruction("strb w16, [x12, x14]");                                // publish the updated allocator side-table byte
}

/// Clears bits for the used arena before the web request resets its bump extent.
pub(crate) fn emit_reset(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        // -- clear every bitmap byte covered by the previous request's arena --
        abi::emit_load_symbol_to_reg(emitter, "rcx", "_heap_off", 0);
        emitter.instruction("add rcx, 63");                                     // round the used heap extent to bitmap bytes
        emitter.instruction("shr rcx, 6");                                      // derive the number of bitmap bytes to clear
        abi::emit_symbol_address(emitter, "r8", "_heap_live_bases");
        emitter.label("__rt_web_reset_live_base_loop");
        emitter.instruction("test rcx, rcx");                                   // have all used bitmap bytes been cleared?
        emitter.instruction("jz __rt_web_reset_live_base_done");                // finish before touching an empty bitmap range
        emitter.instruction("sub rcx, 1");                                      // move to the next used byte from the end
        emitter.instruction("mov BYTE PTR [r8 + rcx], 0");                      // retire every base in this arena granule group
        emitter.instruction("jmp __rt_web_reset_live_base_loop");               // clear the remaining used bitmap bytes
    } else {
        // -- clear every bitmap byte covered by the previous request's arena --
        abi::emit_load_symbol_to_reg(emitter, "x9", "_heap_off", 0);
        emitter.instruction("add x9, x9, #63");                                 // round the used heap extent to bitmap bytes
        emitter.instruction("lsr x9, x9, #6");                                  // derive the number of bitmap bytes to clear
        abi::emit_symbol_address(emitter, "x10", "_heap_live_bases");
        emitter.label("__rt_web_reset_live_base_loop");
        emitter.instruction("cbz x9, __rt_web_reset_live_base_done");           // finish before touching an empty bitmap range
        emitter.instruction("sub x9, x9, #1");                                  // move to the next used byte from the end
        emitter.instruction("strb wzr, [x10, x9]");                             // retire every base in this arena granule group
        emitter.instruction("b __rt_web_reset_live_base_loop");                 // clear the remaining used bitmap bytes
    }
    emitter.label("__rt_web_reset_live_base_done");
}
