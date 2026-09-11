//! Purpose:
//! Defines the stack layout constants for EIR exception-handler slots.
//! Keeps try/catch frame offsets available to codegen and runtime emitters.
//!
//! Called from:
//! - `crate::codegen::frame` when reserving handler slots.
//! - `crate::codegen::lower_inst` when writing handler metadata.
//!
//! Key details:
//! - Offsets must stay synchronized with the runtime exception handler ABI.

/// Size of the pre-allocated try handler slot, aligned for every supported ABI.
/// glibc AArch64 needs 312 bytes for jmp_buf: 176 bytes of register storage,
/// an aligned mask flag, and a 128-byte signal set. Darwin arm64 needs 192 and
/// glibc x86_64 needs 200. Reserve the full largest object, not only the register
/// prefix currently written by a particular libc implementation.
pub(crate) const TRY_HANDLER_SLOT_SIZE: usize = (TRY_HANDLER_JMP_BUF_OFFSET + 312 + 15) & !15;

/// Offset within the try handler slot for the diagnostic depth field.
pub(crate) const TRY_HANDLER_DIAG_DEPTH_OFFSET: usize = 16;

/// Offset within the try handler slot for the `jmp_buf` field.
pub(crate) const TRY_HANDLER_JMP_BUF_OFFSET: usize = 24;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_handler_slot_fits_supported_libc_jump_buffers() {
        // glibc 2.39: sysdeps/aarch64/bits/setjmp.h defines 22 u64 words;
        // setjmp/bits/types/struct___jmp_buf_tag.h adds a mask flag and
        // the 128-byte signal set, with the C ABI's alignment padding.
        #[repr(C)]
        struct LinuxAarch64JmpBuf {
            registers: [u64; 22],
            mask_was_saved: i32,
            saved_mask: [u64; 16],
        }
        let linux_arm64_size = std::mem::size_of::<LinuxAarch64JmpBuf>();
        assert_eq!(linux_arm64_size, 312);
        for (target, size) in [
            ("macos-aarch64", 192),
            ("ios-arm64", 192),
            ("ios-sim-arm64", 192),
            ("linux-aarch64", linux_arm64_size),
            ("linux-x86_64", 200),
        ] {
            assert!(TRY_HANDLER_JMP_BUF_OFFSET + size <= TRY_HANDLER_SLOT_SIZE,
                "{target}: jump buffer exceeds its handler slot");
        }
        assert_eq!(TRY_HANDLER_SLOT_SIZE % 16, 0);
        assert!(TRY_HANDLER_DIAG_DEPTH_OFFSET + 8 <= TRY_HANDLER_JMP_BUF_OFFSET);
    }
}
