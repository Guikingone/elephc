//! Purpose:
//! Small binary-format helpers shared by PHAR container parsers and writers.
//!
//! Called from:
//! - Native PHAR, tar, ZIP, ZipCrypto, and signature modules.
//!
//! Key details:
//! - Integer reads remain bounds-checked and CRC32 stays PHP-compatible.

/// Computes PHP-compatible reflected CRC32 for a PHAR entry payload.
pub(super) fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
/// Reads a little-endian `u16` from `data`.
pub(super) fn le16(data: &[u8], off: usize) -> Option<u16> {
    let b = data.get(off..off + 2)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

/// Reads a little-endian `u32` from `data`.
pub(super) fn le32(data: &[u8], off: usize) -> Option<u32> {
    let b = data.get(off..off + 4)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Reads a little-endian `u64` from `data` (used for ZIP64 fields).
pub(super) fn le64(data: &[u8], off: usize) -> Option<u64> {
    let b = data.get(off..off + 8)?;
    Some(u64::from_le_bytes(b.try_into().ok()?))
}

/// Returns the offset of `needle` in `hay`.
pub(super) fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}
