//! Purpose:
//! Provides shared array-type predicates used by registry builtin checker hooks.
//!
//! Called from:
//! - `crate::builtins::array::count` while validating countable union members.
//!
//! Key details:
//! - `Mixed` remains countable because runtime tags decide the concrete container shape.

use crate::types::PhpType;

/// Returns `true` if a `PhpType` is a countable array type for Union membership checks.
///
/// Used by `crate::builtins::array::count` to test whether every branch of a Union type
/// is countable, in which case `count()` returns `Int` for the whole union.
pub(crate) fn union_member_is_countable_array(ty: &PhpType) -> bool {
    matches!(
        ty,
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed
    )
}

/// Returns whether an array-taking builtin may accept this type at a gradual boundary.
///
/// Concrete arrays and `Mixed` are accepted directly. A union is accepted when at least one
/// member can be an array; lowering retains the runtime tag guard for the non-array branches.
pub(crate) fn array_arg_is_gradually_acceptable(ty: &PhpType) -> bool {
    match ty {
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed => true,
        PhpType::Union(members) => members
            .iter()
            .any(|member| matches!(member, PhpType::Array(_) | PhpType::AssocArray { .. })),
        _ => false,
    }
}
