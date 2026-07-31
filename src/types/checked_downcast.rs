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
/// Which shapes a given position guards is decided by `shape_is_guardable_at` — in THIS module,
/// so the checker's acceptance and the emitter's emission read one matrix rather than two.
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

/// The lowering position a checked downcast guards, reduced to what the shape matrix needs.
///
/// The emitter's own `crate::ir_lower::checked_downcast::DowncastPosition` carries the message
/// wording too; this is the part the CHECKER also has to know, so it lives here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GuardPosition {
    /// A `return` against the enclosing function's declared return type.
    Return,
    /// A call argument against the callee's declared parameter type.
    Argument,
}

/// Returns whether `position` guards `shape` — the ONE shape/position matrix, consulted by the
/// checker before it ACCEPTS such a flow and by the emitter before it EMITS one.
///
/// Keeping this out of `ir_lower` is what stops the two halves from disagreeing. A flow the
/// checker admits but the emitter declines is not a compile error, it is an unguarded
/// representation change at runtime: for `BoxedToRawObject` that is a by-offset read of a box.
///
/// The matrix, and why each entry is where it is:
/// - `RawObject` at both positions: a raw pointer into a raw pointer slot, a pure `Op::InstanceOf`
///   chain. This is what the return guard did before the positions were unified, and what the
///   argument position added on top of it.
/// - `BoxedToRawObject` at an ARGUMENT position: the shape that turns a boxed value silently read
///   as a raw object pointer (garbage, then a segfault as soon as the callee dispatches on it)
///   into PHP's own catchable `TypeError`. Its ok-edge MUST unbox.
/// - `RawObjectToBoxed` at a RETURN position: the return boundary's own boxing coercion performs
///   the representation change after the guard.
/// - `RawObjectToBoxed`/`BoxedToBoxed` at an ARGUMENT position: admitted, but only for a
///   declaration `guard_is_php_faithful` clears — see that function for the weak-mode coercion
///   this must not throw over.
/// - `BoxedToRawObject`/`BoxedToBoxed` at a RETURN position: NOT guarded. The return position's
///   throw resolves the mismatched value's name through `get_class` and RELEASES it as an object;
///   both are wrong for a box, and generalizing them is a separate change from this matrix.
pub(crate) const fn shape_is_guardable_at(shape: GuardShape, position: GuardPosition) -> bool {
    match (shape, position) {
        (GuardShape::RawObject, _) => true,
        (GuardShape::RawObjectToBoxed, _) => true,
        (GuardShape::BoxedToRawObject, GuardPosition::Argument) => true,
        (GuardShape::BoxedToBoxed, GuardPosition::Argument) => true,
        (GuardShape::BoxedToRawObject | GuardShape::BoxedToBoxed, GuardPosition::Return) => false,
    }
}

/// Returns whether a guard built out of `declared` alone reproduces PHP's own accept/reject
/// decision for a value of the given `shape` — i.e. whether every value PHP would let through is
/// one an arm test passes, and every value an arm test fails is one PHP would `TypeError` on.
///
/// The question exists because a declared type is where PHP stops MATCHING and starts CONVERTING,
/// and no arm test can see a conversion: a bare arm chain would throw exactly where PHP quietly
/// converts. Which conversions are reachable depends on what the source can hold, so the rule is
/// shape-aware:
/// - A RAW OBJECT source can only ever be an object, and PHP converts an object into exactly one
///   declared type: `string`, through `__toString`. Its `int`/`float`/`bool` arms therefore stay
///   exact-match-only (an object never weak-coerces to them), which is why a `D|int` return keeps
///   its guard.
/// - A BOXED source can carry any payload, so EVERY scalar arm becomes a conversion site: an int
///   reaching a declared `string|D` is stringified, a numeric string reaching `int|D` is
///   converted, an int reaching `float|D` is widened.
///
/// `iterable` is excluded for both: its object half is `Traversable`, which no runtime tag test
/// covers, so a legitimate `Traversable` would fall down the throw path. A class-free declaration
/// is excluded too — it has no `instanceof` target, and the scalar-free target (`?array`) needs
/// its own proof rather than this one.
///
/// A declaration this refuses is not a rejected program: it simply keeps flowing through the
/// checker's ordinary weak-boundary predicates, unguarded, exactly as it did before.
pub(crate) fn guard_is_php_faithful(declared: &PhpType, shape: GuardShape) -> bool {
    let source_is_raw_object = matches!(shape, GuardShape::RawObject | GuardShape::RawObjectToBoxed);
    /// Returns whether one declared arm is matched exactly by PHP, never converted into.
    fn arm_is_exact_match_only(member: &PhpType, source_is_raw_object: bool) -> bool {
        match member {
            PhpType::Void => true,
            PhpType::Array(_) | PhpType::AssocArray { .. } => true,
            PhpType::Object(name) => !name.is_empty(),
            PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::False => source_is_raw_object,
            _ => false,
        }
    }
    let members: &[PhpType] = match declared {
        PhpType::Union(members) => members,
        single => std::slice::from_ref(single),
    };
    members
        .iter()
        .all(|member| arm_is_exact_match_only(member, source_is_raw_object))
        && members
            .iter()
            .any(|member| matches!(member, PhpType::Object(name) if !name.is_empty()))
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

/// Returns whether a value whose static type is `member` — one member of a UNION SOURCE — is one
/// the guard chain routes the way PHP does.
///
/// Every listed kind has a runtime tag the chain either matches against a declared arm or fails
/// into the throw, and in both cases PHP agrees (given a declared type
/// `guard_is_php_faithful` has cleared). The exclusions are the kinds whose runtime
/// tag the chain cannot read faithfully:
/// - `Callable`: a closure boxes under its own tag, and `__rt_mixed_instanceof` answers 0 for any
///   tag that is not the object tag — so a closure would fail an `instanceof Closure` arm that PHP
///   satisfies.
/// - `Mixed`: not a member a normalized union carries (it absorbs the union), and it already
///   reaches these boundaries through the gradual top-type path.
/// - Pointers, resources and the tagged-scalar representation: not PHP-level values with a boxed
///   tag the chain tests.
pub(crate) fn source_member_is_guard_routable(member: &PhpType) -> bool {
    matches!(
        member,
        PhpType::Void
            | PhpType::Never
            | PhpType::Str
            | PhpType::Int
            | PhpType::Float
            | PhpType::Bool
            | PhpType::False
            | PhpType::Array(_)
            | PhpType::AssocArray { .. }
            | PhpType::Iterable
            | PhpType::Object(_)
    )
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

    /// A boxed source reaching a raw object slot is guarded at an argument, never at a return:
    /// the return throw resolves its name through `get_class` and releases it as an object.
    #[test]
    fn boxed_shapes_are_guarded_at_arguments_only() {
        assert!(shape_is_guardable_at(
            GuardShape::BoxedToRawObject,
            GuardPosition::Argument
        ));
        assert!(shape_is_guardable_at(
            GuardShape::BoxedToBoxed,
            GuardPosition::Argument
        ));
        assert!(!shape_is_guardable_at(
            GuardShape::BoxedToRawObject,
            GuardPosition::Return
        ));
        assert!(!shape_is_guardable_at(
            GuardShape::BoxedToBoxed,
            GuardPosition::Return
        ));
        assert!(shape_is_guardable_at(
            GuardShape::RawObject,
            GuardPosition::Return
        ));
        assert!(shape_is_guardable_at(
            GuardShape::RawObjectToBoxed,
            GuardPosition::Return
        ));
    }

    /// `?D`, `D|E` and `D|array` are matched exactly by PHP, so an arm chain reproduces it.
    #[test]
    fn exact_match_unions_are_php_faithful() {
        let nullable = PhpType::Union(vec![PhpType::Object("D".to_string()), PhpType::Void]);
        assert!(guard_is_php_faithful(&nullable, GuardShape::BoxedToBoxed));
        assert!(guard_is_php_faithful(
            &PhpType::Union(vec![
                PhpType::Object("D".to_string()),
                PhpType::Object("E".to_string()),
            ]),
            GuardShape::BoxedToBoxed
        ));
        assert!(guard_is_php_faithful(
            &PhpType::Union(vec![
                PhpType::Array(Box::new(PhpType::Mixed)),
                PhpType::Object("D".to_string()),
                PhpType::Void,
            ]),
            GuardShape::BoxedToBoxed
        ));
        assert!(guard_is_php_faithful(
            &PhpType::Object("D".to_string()),
            GuardShape::BoxedToRawObject
        ));
    }

    /// A declaration PHP CONVERTS INTO is not guardable: no arm test can see a weak-mode
    /// coercion, so a bare arm chain would throw exactly where PHP quietly converts.
    #[test]
    fn conversion_bearing_unions_are_not_php_faithful() {
        // `string|D` weak-coerces an int, a float, a bool and a `Stringable` object — the last of
        // which a RAW OBJECT source can be, so this one is refused for every shape.
        let string_or_d = PhpType::Union(vec![PhpType::Str, PhpType::Object("D".to_string())]);
        assert!(!guard_is_php_faithful(
            &string_or_d,
            GuardShape::BoxedToBoxed
        ));
        assert!(!guard_is_php_faithful(
            &string_or_d,
            GuardShape::RawObjectToBoxed
        ));
        // `iterable`'s object half is `Traversable`, which no tag test covers.
        assert!(!guard_is_php_faithful(
            &PhpType::Union(vec![PhpType::Iterable, PhpType::Object("D".to_string())]),
            GuardShape::RawObjectToBoxed
        ));
        // A class-free union has no `instanceof` target and belongs to the scalar-free slice.
        assert!(!guard_is_php_faithful(
            &PhpType::Union(vec![
                PhpType::Array(Box::new(PhpType::Mixed)),
                PhpType::Void
            ]),
            GuardShape::BoxedToBoxed
        ));
        // Bare `object` is not an `instanceof` target.
        assert!(!guard_is_php_faithful(
            &PhpType::Union(vec![PhpType::Object(String::new()), PhpType::Void]),
            GuardShape::RawObjectToBoxed
        ));
    }

    /// `D|int` is a conversion site for a BOXED source (an int payload is a legitimate member, and
    /// a numeric string weak-coerces into it) but NOT for a RAW OBJECT one: PHP never converts an
    /// object to `int`, so that guard stays exact and keeps firing.
    #[test]
    fn numeric_arms_are_faithful_only_for_a_raw_object_source() {
        let d_or_int = PhpType::Union(vec![PhpType::Object("D".to_string()), PhpType::Int]);
        assert!(guard_is_php_faithful(&d_or_int, GuardShape::RawObject));
        assert!(guard_is_php_faithful(
            &d_or_int,
            GuardShape::RawObjectToBoxed
        ));
        assert!(!guard_is_php_faithful(&d_or_int, GuardShape::BoxedToBoxed));
    }

    /// A closure member is not routable: it boxes under its own tag, which the boxed
    /// `instanceof` helper answers 0 for.
    #[test]
    fn callable_union_members_are_not_routable() {
        assert!(!source_member_is_guard_routable(&PhpType::Callable));
        assert!(!source_member_is_guard_routable(&PhpType::Mixed));
        assert!(source_member_is_guard_routable(&PhpType::Void));
        assert!(source_member_is_guard_routable(&PhpType::False));
        assert!(source_member_is_guard_routable(&PhpType::Object(
            "D".to_string()
        )));
        assert!(source_member_is_guard_routable(&PhpType::Array(Box::new(
            PhpType::Mixed
        ))));
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
