//! Purpose:
//! Materializes original non-scalar offsets until a Mixed receiver selects array or object rules.
//!
//! Called from:
//! - `super::mixed_array_runtime` reads, writes and fetch-for-write operations.
//!
//! Key details:
//! - The deferred pair borrows the existing payload and creates no cleanup owner.
//! - ArrayAccess callbacks are emitted before this ordinary runtime fallback.

use super::*;
use crate::codegen_support::deferred_array_keys::deferred_key_marker;

/// Preserves original object/array/Closure types in the normalized-key pair's invalid domain.
pub(super) fn materialize_key(
    ctx: &mut FunctionContext<'_>, key: ValueId, normalization: hashes::HashKeyNormalization,
) -> Result<()> {
    let ty = ctx.value_php_type(key)?.codegen_repr();
    let tag = match &ty {
        PhpType::Object(_) => Some(6),
        PhpType::Array(_) => Some(4),
        PhpType::AssocArray { .. } => Some(5),
        PhpType::Callable => Some(10),
        _ => None,
    };
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            if let Some(tag) = tag {
                ctx.load_value_to_reg(key, "x1")?;
                abi::emit_load_int_immediate(ctx.emitter, "x2", deferred_key_marker(tag));
                Ok(())
            } else if matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
                hashes::materialize_mixed_hash_key_aarch64(ctx, key, normalization, true)
            } else {
                hashes::materialize_hash_key_aarch64_with(ctx, key, normalization)
            }
        }
        Arch::X86_64 => {
            if let Some(tag) = tag {
                ctx.load_value_to_reg(key, "rsi")?;
                abi::emit_load_int_immediate(ctx.emitter, "rdx", deferred_key_marker(tag));
                Ok(())
            } else if matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
                hashes::materialize_mixed_hash_key_x86_64(ctx, key, normalization, true)
            } else {
                hashes::materialize_hash_key_x86_64_with(ctx, key, normalization)
            }
        }
    }
}
