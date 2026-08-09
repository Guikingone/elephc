//! Purpose:
//! Defines the stack layout constants for EIR exception-handler slots.
//! Keeps try/catch frame offsets available to codegen and runtime emitters.
//!
//! Called from:
//! - `crate::codegen::frame` when reserving handler slots.
//! - `crate::codegen::block_emit` when restoring state at catch entry.
//! - `crate::codegen::lower_inst` when writing handler metadata.
//!
//! Key details:
//! - Offsets must stay synchronized with the runtime exception handler ABI.
//! - Scalar metadata precedes the complete opaque `jmp_buf`; every handler
//!   producer and consumer must use these offsets instead of hard-coded values.

/// Size of the exception-handler record through the complete `jmp_buf`.
pub(crate) const TRY_HANDLER_SLOT_SIZE: usize = 240;

/// Offset within the try handler slot for the diagnostic depth field.
pub(crate) const TRY_HANDLER_DIAG_DEPTH_OFFSET: usize = 16;

/// Offset within the try handler slot for the `jmp_buf` field.
pub(crate) const TRY_HANDLER_JMP_BUF_OFFSET: usize = 32;

/// Offset within a handler record holding the user-stack byte-budget snapshot.
/// `longjmp` skips normal epilogues, so catch entry restores this value before
/// executing PHP code in the handler block.
pub(crate) const TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET: usize = 24;

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies scalar snapshots precede the opaque `jmp_buf` and the complete
    /// handler record preserves native stack alignment.
    #[test]
    fn recursion_snapshot_does_not_overlap_jmp_buf() {
        assert!(
            TRY_HANDLER_JMP_BUF_OFFSET < TRY_HANDLER_SLOT_SIZE,
            "the complete jmp_buf region must remain inside its handler slot"
        );
        assert!(
            TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET < TRY_HANDLER_JMP_BUF_OFFSET,
            "recursion metadata must precede the opaque jmp_buf region"
        );
        assert!(
            TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET + std::mem::size_of::<u64>()
                <= TRY_HANDLER_JMP_BUF_OFFSET,
            "recursion metadata must not overlap the opaque jmp_buf region"
        );
        assert_eq!(
            TRY_HANDLER_SLOT_SIZE % 16,
            0,
            "the handler ABI must preserve native stack alignment"
        );
    }
}
