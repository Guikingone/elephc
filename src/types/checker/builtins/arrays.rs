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
    // `False` belongs here because php compiles `count($x)` where `$x` may be an array or `false`
    // and decides at run time — `scandir()`, `fgetcsv()` and every other `array|false` builtin
    // produce exactly that union. Accepting it was a wrong answer waiting to happen until the
    // runtime raised php's TypeError for the false payload rather than answering 0; it does now,
    // so the refusal no longer protects anything.
    matches!(
        ty,
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed | PhpType::False
    )
}
