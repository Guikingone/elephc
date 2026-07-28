//! Purpose:
//! Common scalar conversion and checksum helpers.
//!
//! Called from:
//! - `crate::interpreter::builtins::scalars` re-exports.
//!
//! Key details:
//! - Runtime cells remain opaque and all PHP coercions flow through `RuntimeValueOps`.

use super::super::super::*;

/// Returns the standard zlib/PHP CRC-32 checksum for a byte slice.
pub(in crate::interpreter) fn eval_crc32_bytes(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

/// Returns the eval-local native payload carried by a runtime resource cell.
///
/// Reads the tag-9 payload word straight out of the cell instead of casting the
/// cell to PHP int and undoing a `+ 1`. Those are two different numbers: `(int)`
/// on a resource yields the PHP RESOURCE ID, which the runtime mints from its own
/// counter (`runtime::resource_ids`) precisely so that a displayed id never
/// depends on a native payload. The id therefore cannot be inverted back into the
/// zero-based key of `EvalStreamResources`, and any attempt to do so silently
/// resolves the wrong stream. The raw word is the only faithful source, and it is
/// the same accessor the native-argument and by-reference writeback paths already
/// use for tag-9 slots.
pub(in crate::interpreter) fn eval_resource_payload(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<i64, EvalStatus> {
    if values.type_tag(value)? != EVAL_TAG_RESOURCE {
        return Err(EvalStatus::RuntimeFatal);
    }
    i64::try_from(values.raw_value_word(value)?).map_err(|_| EvalStatus::RuntimeFatal)
}

/// Casts one eval value to PHP int and returns the scalar payload.
pub(in crate::interpreter) fn eval_int_value(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<i64, EvalStatus> {
    let value = values.cast_int(value)?;
    let bytes = values.string_bytes(value)?;
    std::str::from_utf8(&bytes)
        .map_err(|_| EvalStatus::RuntimeFatal)?
        .parse::<i64>()
        .map_err(|_| EvalStatus::RuntimeFatal)
}
