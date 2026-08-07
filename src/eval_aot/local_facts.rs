//! Purpose:
//! Defines deterministic hashing and definite-local facts for AOT analysis.
//!
//! Called from:
//! - The eval AOT facade and sibling analysis modules.
//!
//! Key details:
//! - Local facts track values, array kinds, and scalar representations conservatively.

use super::*;

/// Hashes a fragment with a stable FNV-1a variant for deterministic symbol names.
pub(super) fn stable_fragment_hash(fragment: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in fragment.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Definite local facts tracked while classifying an eval fragment for EIR AOT.
#[derive(Clone, Default)]
pub(super) struct EirLocalFacts {
    pub(super) assigned: BTreeSet<String>,
    pub(super) int_locals: BTreeSet<String>,
    pub(super) float_locals: BTreeSet<String>,
    pub(super) array_locals: BTreeSet<String>,
}

impl EirLocalFacts {
    /// Creates empty local facts for a fresh eval fragment block.
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Returns true when a variable is definitely assigned in this control-flow path.
    pub(super) fn is_assigned(&self, name: &str) -> bool {
        self.assigned.contains(name)
    }

    /// Returns true when a variable is definitely assigned as an integer value.
    pub(super) fn is_int_local(&self, name: &str) -> bool {
        self.int_locals.contains(name)
    }

    /// Returns true when a variable is definitely assigned as a floating value.
    pub(super) fn is_float_local(&self, name: &str) -> bool {
        self.float_locals.contains(name)
    }

    /// Returns true when a variable is definitely assigned from a static array literal.
    pub(super) fn is_array_local(&self, name: &str) -> bool {
        self.array_locals.contains(name)
    }

    /// Records an assignment and updates scalar/array local facts for that variable.
    pub(super) fn assign<S>(
        &mut self,
        name: &str,
        value: &Expr,
        support: &S,
        scope_reads: Option<&BTreeSet<String>>,
    ) where
        S: EirStaticCallSupport,
    {
        self.assigned.insert(name.to_string());
        if expr_is_eir_int_value_safe(value, support, self, scope_reads) {
            self.int_locals.insert(name.to_string());
        } else {
            self.int_locals.remove(name);
        }
        if expr_is_eir_float_value_safe(value, support, self, scope_reads) {
            self.float_locals.insert(name.to_string());
        } else {
            self.float_locals.remove(name);
        }
        if expr_is_eir_static_array_literal_source_safe(value, support, self, scope_reads) {
            self.array_locals.insert(name.to_string());
        } else {
            self.array_locals.remove(name);
        }
    }

    /// Records that a variable is definitely assigned, but with no narrower local fact.
    pub(super) fn assign_unknown(&mut self, name: &str) {
        self.assigned.insert(name.to_string());
        self.int_locals.remove(name);
        self.array_locals.remove(name);
    }
}
