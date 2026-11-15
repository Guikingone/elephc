//! Purpose:
//! Emits an inline forward byte copy for the x86_64 string runtime helpers: eight bytes at a
//! time, then a 4/2/1-byte tail, instead of one byte per iteration.
//!
//! Called from:
//! - String runtime emitters (`implode`, ...) that append a source string into a buffer.
//!
//! Key details:
//! - `src` and `dst` advance past the copied bytes; `len` is consumed. The source must end at or
//!   before the destination start whenever the two can share a buffer (the callers append after
//!   their sources), so no block read overlaps a block write.
//! - `tmp` must be one of `r8`..`r11` (its 32/16/8-bit names are used for the tail); `src`,
//!   `dst` and `len` may be any 64-bit registers.

use crate::codegen_support::emit::Emitter;

/// Emits the copy of `len` bytes from `src` to `dst`, labels prefixed by `prefix`.
pub(crate) fn emit_copy_bytes_x86_64(emitter: &mut Emitter, src: &str, dst: &str, len: &str, tmp: &str, prefix: &str) {
    let (tmp32, tmp16, tmp8) = (format!("{}d", tmp), format!("{}w", tmp), format!("{}b", tmp));
    emitter.instruction(&format!("cmp {}, 8", len));                            // at least one full 8-byte block?
    emitter.instruction(&format!("jb {}_tail", prefix));
    emitter.label(&format!("{}_qword", prefix));
    emitter.instruction(&format!("mov {}, QWORD PTR [{}]", tmp, src));          // load eight source bytes
    emitter.instruction(&format!("mov QWORD PTR [{}], {}", dst, tmp));          // store them
    emitter.instruction(&format!("add {}, 8", src));                            // advance the source
    emitter.instruction(&format!("add {}, 8", dst));                            // advance the destination
    emitter.instruction(&format!("sub {}, 8", len));                            // eight fewer bytes left
    emitter.instruction(&format!("cmp {}, 8", len));                            // another full block?
    emitter.instruction(&format!("jae {}_qword", prefix));
    emitter.label(&format!("{}_tail", prefix));
    emitter.instruction(&format!("test {}, 4", len));                           // four-byte tail?
    emitter.instruction(&format!("jz {}_tail2", prefix));
    emitter.instruction(&format!("mov {}, DWORD PTR [{}]", tmp32, src));        // load four source bytes
    emitter.instruction(&format!("mov DWORD PTR [{}], {}", dst, tmp32));        // store them
    emitter.instruction(&format!("add {}, 4", src));                            // advance the source
    emitter.instruction(&format!("add {}, 4", dst));                            // advance the destination
    emitter.label(&format!("{}_tail2", prefix));
    emitter.instruction(&format!("test {}, 2", len));                           // two-byte tail?
    emitter.instruction(&format!("jz {}_tail1", prefix));
    emitter.instruction(&format!("mov {}, WORD PTR [{}]", tmp16, src));         // load two source bytes
    emitter.instruction(&format!("mov WORD PTR [{}], {}", dst, tmp16));         // store them
    emitter.instruction(&format!("add {}, 2", src));                            // advance the source
    emitter.instruction(&format!("add {}, 2", dst));                            // advance the destination
    emitter.label(&format!("{}_tail1", prefix));
    emitter.instruction(&format!("test {}, 1", len));                           // last odd byte?
    emitter.instruction(&format!("jz {}_done", prefix));
    emitter.instruction(&format!("mov {}, BYTE PTR [{}]", tmp8, src));          // load the last source byte
    emitter.instruction(&format!("mov BYTE PTR [{}], {}", dst, tmp8));          // store it
    emitter.instruction(&format!("add {}, 1", src));                            // advance the source
    emitter.instruction(&format!("add {}, 1", dst));                            // advance the destination
    emitter.label(&format!("{}_done", prefix));
}
