//! Purpose:
//! Defines stable dependency-neutral identities for PHP builtin contracts.
//!
//! Called from:
//! - Shared catalog declarations and backend implementation registries.
//!
//! Key details:
//! - IDs use a fixed FNV-1a hash of the canonical lowercase PHP name.
//! - Catalog initialization must reject the theoretically possible hash collision.

/// Stable identity shared by the AOT and Magician bindings of one PHP builtin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BuiltinId(u64);

impl BuiltinId {
    /// Derives an ID from a canonical lowercase PHP builtin name.
    pub const fn from_canonical_name(name: &str) -> Self {
        let bytes = name.as_bytes();
        let mut hash = 0xcbf29ce484222325u64;
        let mut index = 0usize;
        while index < bytes.len() {
            hash ^= bytes[index] as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            index += 1;
        }
        Self(hash)
    }

    /// Returns the stable integer representation used by typed registries and ABIs.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}
#[cfg(test)]
mod tests {
    use super::BuiltinId;

    /// Verifies builtin identities are deterministic and name-sensitive.
    #[test]
    fn builtin_ids_are_stable_for_canonical_names() {
        assert_eq!(
            BuiltinId::from_canonical_name("strlen"),
            BuiltinId::from_canonical_name("strlen")
        );
        assert_ne!(
            BuiltinId::from_canonical_name("strlen"),
            BuiltinId::from_canonical_name("substr")
        );
    }
}
