//! Purpose:
//! Removes Linux AArch64 pointer tags and authentication bits during attach walks.
//!
//! Called from:
//! - `monitor::ptrace::registers()` and its frame-chain decoder.
//!
//! Key details:
//! - The target's instruction PAC mask preserves valid 48-bit and 52-bit VAs.
//! - Unsupported PAC regsets mean no PAC; unexpected read failures reject the sample.

use std::io;

/// Reads the instruction mask from the stopped target's `user_pac_mask` regset.
#[cfg(target_arch = "aarch64")]
pub(super) fn instruction_mask(tid: u32) -> io::Result<u64> {
    // Linux UAPI NT_ARM_PAC_MASK; user_pac_mask contains data_mask, insn_mask.
    const NT_ARM_PAC_MASK: libc::c_int = 0x406;
    let mut masks = [0u64; 2];
    let mut iov = libc::iovec {
        iov_base: masks.as_mut_ptr().cast(),
        iov_len: std::mem::size_of_val(&masks),
    };
    // SAFETY: the kernel writes at most iov_len bytes into the live masks array.
    let result = unsafe {
        libc::ptrace(
            libc::PTRACE_GETREGSET, tid as libc::pid_t, NT_ARM_PAC_MASK,
            &mut iov as *mut libc::iovec,
        )
    };
    if result == -1 {
        let error = io::Error::last_os_error();
        return match error.raw_os_error() {
            // Absent regset or absent hardware support, respectively.
            Some(libc::EINVAL) | Some(libc::EIO) | Some(libc::ENODEV) => Ok(0),
            _ => Err(error),
        };
    }
    if iov.iov_len != std::mem::size_of_val(&masks) {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "short PAC mask regset"));
    }
    Ok(masks[1])
}

/// Other Linux architectures have no AArch64 pointer-authentication regset.
#[cfg(not(target_arch = "aarch64"))]
pub(super) fn instruction_mask(_tid: u32) -> io::Result<u64> {
    Ok(0)
}

/// Clears the target's instruction PAC bits and top-byte tag only on AArch64.
pub(super) fn strip_pointer_auth(addr: u64, instruction_mask: u64) -> u64 {
    strip_pointer_auth_on(cfg!(target_arch = "aarch64"), addr, instruction_mask)
}

/// Applies the target mask independently of the test host's architecture.
fn strip_pointer_auth_on(aarch64: bool, addr: u64, instruction_mask: u64) -> u64 {
    if aarch64 {
        addr & !instruction_mask & 0x00ff_ffff_ffff_ffff
    } else {
        addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full 48-bit PAC, TBI alone and canonical 52-bit VAs use their target mask.
    #[test]
    fn pointer_authentication_bits_are_stripped_only_on_aarch64() {
        for (signed, mask, canonical) in [
            (0xAA7F_0010_0000_1234, 0x007f_0000_0000_0000, 0x0010_0000_1234),
            (0xAA00_0010_0000_1234, 0, 0x0010_0000_1234),
            (0xAA7F_0010_0000_1234, 0x0070_0000_0000_0000, 0x000f_0010_0000_1234),
            (0x000f_0010_0000_1234, 0, 0x000f_0010_0000_1234),
        ] {
            assert_eq!(strip_pointer_auth_on(true, signed, mask), canonical);
            assert_eq!(strip_pointer_auth_on(false, signed, mask), signed);
            assert_eq!(strip_pointer_auth_on(true, canonical, mask), canonical);
        }
    }
}
