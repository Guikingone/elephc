//! Purpose:
//! The ONE source of truth for the checked-downcast relaxation's REPRESENTATION rule: given a
//! slot declared `expected` and a value statically typed `actual`, decide what shape a runtime
//! guard would have to have to bridge the two (`downcast_guard_shape`), and which runtime tests
//! such a guard may legitimately build out of the declared type alone (`declared_guard_arms`).
//!
//! Called from:
//! - `crate::types::checker::type_compat::object_types::Checker::checked_downcast_guardable` —
//!   the CHECKER half, which decides whether to ACCEPT such a flow.
//! - `crate::ir_lower::checked_downcast::emit_checked_downcast` — the LOWERING half, which
//!   decides what to EMIT for it.
//!
//! Key details:
//! - Deliberately free of `Checker` state and of `ir_lower`'s class tables, so both halves call
//!   literally the same function. The class-hierarchy walk stays duplicated (each side owns its
//!   own tables), but the representation rule — the new permissiveness — lives here exactly once:
//!   were acceptance and emission ever to disagree about the shape, the disagreement would
//!   surface as a wrong-representation read (a segfault), not as a compile error.
//! - The rule is expressed over `PhpType::codegen_repr()`, never over the declared spelling,
//!   because what the guard has to DO is decided by the STORAGE each side uses: a raw object
//!   pointer and a boxed `Mixed` cell are not interchangeable, and handing one where the other
//!   is expected is precisely how a checked downcast turns into a by-offset read of a box.

use crate::ir::PhpTypePredicate;
use crate::types::PhpType;

/// The storage-level shape of a checked downcast: how the guarded value is represented on the
/// way in, and how the guarded slot expects it on the way out.
///
/// Which shapes a given lowering position actually emits a guard for is decided by
/// `crate::ir_lower::checked_downcast::shape_is_emittable_at`, not here — this enum only
/// classifies the representation pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GuardShape {
    /// Raw object pointer into a raw object pointer slot. The guard is a pure `Op::InstanceOf`
    /// chain and the value that reaches the slot is the one that came in, unchanged.
    RawObject,
    /// Raw object pointer into a boxed (`Mixed`) slot. Still a pure `Op::InstanceOf` chain: the
    /// caller's existing boxing coercion runs after the guard and performs the repr change.
    RawObjectToBoxed,
    /// Boxed `Mixed` into a boxed `Mixed` slot. Runtime tag tests (and possibly an instanceof on
    /// the boxed payload); the box pointer itself is copied into the slot as-is.
    BoxedToBoxed,
    /// Boxed `Mixed` into a raw object pointer slot. The guard must additionally UNBOX on its
    /// ok-edge, or the consumer performs by-offset property access on a `Mixed` box.
    BoxedToRawObject,
}

/// Returns the guard shape for a value of static type `actual` reaching a slot declared
/// `expected`, or `None` when no runtime guard could bridge the two representations (e.g. a
/// scalar source, or a scalar target).
pub(crate) fn downcast_guard_shape(expected: &PhpType, actual: &PhpType) -> Option<GuardShape> {
    match (expected.codegen_repr(), actual.codegen_repr()) {
        (PhpType::Object(_), PhpType::Object(_)) => Some(GuardShape::RawObject),
        (PhpType::Mixed, PhpType::Object(_)) => Some(GuardShape::RawObjectToBoxed),
        (PhpType::Mixed, PhpType::Mixed) => Some(GuardShape::BoxedToBoxed),
        (PhpType::Object(_), PhpType::Mixed) => Some(GuardShape::BoxedToRawObject),
        _ => None,
    }
}

/// One runtime test a checked-downcast guard is allowed to build out of a declared type arm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GuardArm {
    /// PHP `null` — the arm a `?T` declaration normalizes to.
    Null,
    /// An array-family arm (`array`, a keyed array, `iterable`), tested by runtime tag.
    Array,
    /// A scalar arm, tested by runtime tag.
    Scalar(PhpTypePredicate),
    /// PHP's bare `object` pseudo-type. Matches EVERY object and is never an `instanceof`
    /// target: `Op::InstanceOf` against the empty class name can never match.
    AnyObject,
    /// A named class or interface — the `instanceof` targets.
    Class(String),
}

/// Returns the runtime tests a guard may build for `ty`, in DECLARED (source) order, deduped.
///
/// Arms a runtime guard cannot decide from the declared type alone (`callable`, `mixed`,
/// `never`, resources, raw pointers) are omitted: they contribute no test, so a caller that
/// needs every arm of `ty` to be decidable must check that the returned arm count matches the
/// declared arm count rather than assuming this is total.
pub(crate) fn declared_guard_arms(ty: &PhpType) -> Vec<GuardArm> {
    /// Appends the guard arms of `ty` to `out` in source order, skipping duplicates.
    fn walk(ty: &PhpType, out: &mut Vec<GuardArm>) {
        let arm = match ty {
            PhpType::Void => GuardArm::Null,
            PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => GuardArm::Array,
            PhpType::Str => GuardArm::Scalar(PhpTypePredicate::String),
            PhpType::Int => GuardArm::Scalar(PhpTypePredicate::Int),
            PhpType::Float => GuardArm::Scalar(PhpTypePredicate::Float),
            PhpType::Bool | PhpType::False => GuardArm::Scalar(PhpTypePredicate::Bool),
            PhpType::Object(name) if name.is_empty() => GuardArm::AnyObject,
            PhpType::Object(name) => GuardArm::Class(name.clone()),
            PhpType::Union(members) => {
                for member in members {
                    walk(member, out);
                }
                return;
            }
            _ => return,
        };
        if !out.contains(&arm) {
            out.push(arm);
        }
    }
    let mut out = Vec::new();
    walk(ty, &mut out);
    out
}

/// Returns the `Object(name)` arms of `ty` with a NON-EMPTY name, in declared order, deduped —
/// the classes/interfaces a guard can spell as `Op::InstanceOf` targets.
pub(crate) fn declared_instanceof_targets(ty: &PhpType) -> Vec<String> {
    declared_guard_arms(ty)
        .into_iter()
        .filter_map(|arm| match arm {
            GuardArm::Class(name) => Some(name),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A concrete object source into a concrete object slot is the raw-pointer shape.
    #[test]
    fn object_into_object_is_raw_object_shape() {
        let expected = PhpType::Object("D".to_string());
        let actual = PhpType::Object("B".to_string());
        assert_eq!(
            downcast_guard_shape(&expected, &actual),
            Some(GuardShape::RawObject)
        );
    }

    /// A union declaration boxes, so a raw object flowing into it is the box-on-the-way-out shape.
    #[test]
    fn object_into_nullable_object_is_raw_object_to_boxed_shape() {
        let expected = PhpType::Union(vec![PhpType::Object("D".to_string()), PhpType::Void]);
        let actual = PhpType::Object("B".to_string());
        assert_eq!(
            downcast_guard_shape(&expected, &actual),
            Some(GuardShape::RawObjectToBoxed)
        );
    }

    /// A boxed source into a concrete object slot is the shape that MUST unbox on its ok-edge.
    #[test]
    fn mixed_into_object_is_boxed_to_raw_object_shape() {
        let expected = PhpType::Object("D".to_string());
        assert_eq!(
            downcast_guard_shape(&expected, &PhpType::Mixed),
            Some(GuardShape::BoxedToRawObject)
        );
    }

    /// Scalar boundaries are not downcasts and get no shape at all.
    #[test]
    fn scalar_boundaries_have_no_guard_shape() {
        assert_eq!(
            downcast_guard_shape(&PhpType::Str, &PhpType::Object("B".to_string())),
            None
        );
        assert_eq!(
            downcast_guard_shape(&PhpType::Object("B".to_string()), &PhpType::Str),
            None
        );
        assert_eq!(downcast_guard_shape(&PhpType::Int, &PhpType::Mixed), None);
    }

    /// Declared arms come back in source order, with the null arm first for a `null|D` spelling.
    #[test]
    fn declared_arms_follow_source_order() {
        let ty = PhpType::Union(vec![
            PhpType::Void,
            PhpType::Array(Box::new(PhpType::Mixed)),
            PhpType::Str,
            PhpType::Object("D".to_string()),
        ]);
        assert_eq!(
            declared_guard_arms(&ty),
            vec![
                GuardArm::Null,
                GuardArm::Array,
                GuardArm::Scalar(PhpTypePredicate::String),
                GuardArm::Class("D".to_string()),
            ]
        );
    }

    /// The bare `object` pseudo-type is an arm, but never an `instanceof` target.
    #[test]
    fn bare_object_is_any_object_and_not_an_instanceof_target() {
        let ty = PhpType::Object(String::new());
        assert_eq!(declared_guard_arms(&ty), vec![GuardArm::AnyObject]);
        assert!(declared_instanceof_targets(&ty).is_empty());
    }

    /// A `callable` arm is undecidable from the declared type and contributes no test.
    #[test]
    fn callable_arm_contributes_no_test() {
        let ty = PhpType::Union(vec![PhpType::Callable, PhpType::Object("D".to_string())]);
        assert_eq!(declared_guard_arms(&ty), vec![GuardArm::Class("D".to_string())]);
    }

    /// Repeated arms are deduped so a guard emits one test per distinct check.
    #[test]
    fn repeated_arms_are_deduped() {
        let ty = PhpType::Union(vec![
            PhpType::Object("D".to_string()),
            PhpType::Object("D".to_string()),
            PhpType::Array(Box::new(PhpType::Mixed)),
            PhpType::AssocArray {
                key: Box::new(PhpType::Str),
                value: Box::new(PhpType::Mixed),
            },
        ]);
        assert_eq!(
            declared_guard_arms(&ty),
            vec![GuardArm::Class("D".to_string()), GuardArm::Array]
        );
    }
}
