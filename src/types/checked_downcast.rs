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
/// - `BoxedToRawObject`/`BoxedToBoxed` at a RETURN position: admitted. These were held back while
///   the return position's throw could only resolve a name through `get_class` and could only
///   release as an object — both wrong for a box, and both since fixed in
///   `crate::codegen::lower_inst::objects::return_type_guard`, which now picks the name table and
///   the release helper from the operand's own representation. The RELEASE ITSELF stays
///   unconditional here, because it is a property of the POSITION (a return value the caller never
///   receives has no other owner), not of the shape.
///
/// Deliberately matched on the POSITION first, and exhaustively on both axes: every position that
/// is added has to state which shapes it guards, and can only do so by pointing at an emitter that
/// handles them. A position defaulting into someone else's row is the failure mode this exists to
/// prevent.
pub(crate) const fn shape_is_guardable_at(shape: GuardShape, position: GuardPosition) -> bool {
    match position {
        GuardPosition::Return | GuardPosition::Argument => matches!(
            shape,
            GuardShape::RawObject
                | GuardShape::RawObjectToBoxed
                | GuardShape::BoxedToRawObject
                | GuardShape::BoxedToBoxed
        ),
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

/// A declared type PHP's weak mode treats as a CONVERSION site rather than a match site: exactly
/// one scalar arm, optionally joined by `null`.
///
/// `guard_is_php_faithful` keeps such a declaration off the arm-chain guard precisely because no
/// arm test can see a conversion. That is the right call for the arms PHP CONVERTS — but PHP also
/// REFUSES payloads at the very same boundary, and those refusals are invisible to the arm chain
/// for the same reason. This type names the boundary so `scalar_coercion_refusals` can state which
/// payloads it rejects.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScalarCoercionTarget {
    /// The single scalar arm the declaration converts INTO (`Str`, `Int`, `Float` or `Bool`).
    pub scalar: PhpType,
    /// Whether the declaration also carries a `null` arm (`?string`, `string|null`).
    pub accepts_null: bool,
}

/// A runtime payload PHP REFUSES at a scalar conversion boundary, raising a catchable `TypeError`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoercionRefusal {
    /// A `null` payload reaching a NON-nullable scalar declaration.
    Null,
    /// An array payload, refused by every scalar declaration.
    Array,
}

/// Returns the scalar conversion boundary `declared` describes, or `None` when it is not one.
///
/// Accepts `string`/`int`/`float`/`bool` and their `?T` spellings, and nothing else. A declaration
/// with a second non-null arm (`string|array`, `string|D`) is deliberately excluded: an array
/// reaching `string|array` is MATCHED, not refused, so the uniform refusal table below does not
/// describe it and it needs its own proof.
pub(crate) fn scalar_coercion_target(declared: &PhpType) -> Option<ScalarCoercionTarget> {
    /// Returns whether a member is one of the four scalars PHP weak-converts into.
    fn is_coercion_scalar(member: &PhpType) -> bool {
        matches!(
            member,
            PhpType::Str | PhpType::Int | PhpType::Float | PhpType::Bool
        )
    }
    match declared {
        single if is_coercion_scalar(single) => Some(ScalarCoercionTarget {
            scalar: single.clone(),
            accepts_null: false,
        }),
        PhpType::Union(members) => {
            let mut scalar = None;
            let mut accepts_null = false;
            for member in members {
                match member {
                    PhpType::Void => accepts_null = true,
                    other if is_coercion_scalar(other) && scalar.is_none() => {
                        scalar = Some(other.clone());
                    }
                    // A second scalar arm, a class arm, an array arm: not this shape.
                    _ => return None,
                }
            }
            scalar.map(|scalar| ScalarCoercionTarget {
                scalar,
                accepts_null,
            })
        }
        _ => None,
    }
}

/// Returns the payloads `target` refuses, in the order a guard must test them.
///
/// MEASURED against `php -n` (php-8.5.6) for all four scalar targets crossed with a boxed `null`,
/// `int`, `float`, `bool`, numeric string, non-numeric string, array, plain object and `Stringable`
/// object source. Two refusals hold for EVERY target and are therefore stated once here:
/// - `null` into a non-nullable scalar — `fs(null)`, `fi(null)`, `ff(null)`, `fb(null)` all throw.
/// - an array into any scalar — `fs(arr)`, `fi(arr)`, `ff(arr)`, `fb(arr)` all throw.
///
/// The refusals that are NOT uniform are deliberately absent, because a guard that states them
/// here would have to state them per target: a non-numeric string is refused by `int`/`float` but
/// converted by `string`/`bool`; a plain object is refused by all four but a `Stringable` one is
/// converted by `string` alone. Those need the numeric-string and `__toString` probes a tag test
/// cannot express, so they stay out rather than be approximated.
pub(crate) fn scalar_coercion_refusals(target: &ScalarCoercionTarget) -> Vec<CoercionRefusal> {
    let mut refusals = Vec::new();
    if !target.accepts_null {
        refusals.push(CoercionRefusal::Null);
    }
    refusals.push(CoercionRefusal::Array);
    refusals
}

/// Returns whether a value statically typed `actual` could carry `refusal`'s payload at runtime.
///
/// Only a BOXED source is ever asked: the caller gates on `codegen_repr() == Mixed`, because a
/// refusal test reads a runtime tag and only a box has one. Within that, `Mixed` itself could hold
/// anything, while a union source is decided by its members — which is what keeps a `?string`
/// argument from paying for an array test it can never fail.
pub(crate) fn boxed_source_may_hold(actual: &PhpType, refusal: CoercionRefusal) -> bool {
    match actual {
        PhpType::Mixed => true,
        PhpType::Union(members) => members
            .iter()
            .any(|member| boxed_source_may_hold(member, refusal)),
        PhpType::Void | PhpType::Never => refusal == CoercionRefusal::Null,
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => {
            refusal == CoercionRefusal::Array
        }
        _ => false,
    }
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

    /// Both value positions guard all four shapes: the return throw resolves a BOXED value's name
    /// by runtime tag and releases it as a `Mixed` cell, so the two shapes it used to decline are
    /// emittable there now.
    #[test]
    fn both_value_positions_guard_every_shape() {
        for position in [GuardPosition::Return, GuardPosition::Argument] {
            for shape in [
                GuardShape::RawObject,
                GuardShape::RawObjectToBoxed,
                GuardShape::BoxedToRawObject,
                GuardShape::BoxedToBoxed,
            ] {
                assert!(
                    shape_is_guardable_at(shape, position),
                    "{:?} must be guardable at {:?}",
                    shape,
                    position
                );
            }
        }
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

    /// The four scalars and their `?T` spellings are conversion boundaries; a declaration with a
    /// second non-null arm is not, because an array reaching `string|array` is MATCHED there.
    #[test]
    fn scalar_coercion_targets_are_the_four_scalars_and_their_nullable_spellings() {
        for scalar in [PhpType::Str, PhpType::Int, PhpType::Float, PhpType::Bool] {
            let bare = scalar_coercion_target(&scalar).expect("a bare scalar is a boundary");
            assert_eq!(bare.scalar, scalar);
            assert!(!bare.accepts_null);

            let nullable = PhpType::Union(vec![scalar.clone(), PhpType::Void]);
            let nullable = scalar_coercion_target(&nullable).expect("`?T` is a boundary");
            assert_eq!(nullable.scalar, scalar);
            assert!(nullable.accepts_null);
        }
        assert!(scalar_coercion_target(&PhpType::Mixed).is_none());
        assert!(scalar_coercion_target(&PhpType::Object("D".to_string())).is_none());
        assert!(
            scalar_coercion_target(&PhpType::Union(vec![
                PhpType::Str,
                PhpType::Array(Box::new(PhpType::Mixed)),
                PhpType::Void,
            ]))
            .is_none(),
            "an array arm MATCHES an array payload, so the uniform refusal table does not apply"
        );
        assert!(
            scalar_coercion_target(&PhpType::Union(vec![PhpType::Str, PhpType::Int])).is_none(),
            "two scalar arms need a per-arm decision this shape does not carry"
        );
    }

    /// Null is refused only by a non-nullable declaration; an array is refused by every one.
    #[test]
    fn scalar_coercion_refusals_follow_the_measured_php_matrix() {
        let non_nullable = scalar_coercion_target(&PhpType::Str).expect("boundary");
        assert_eq!(
            scalar_coercion_refusals(&non_nullable),
            vec![CoercionRefusal::Null, CoercionRefusal::Array]
        );
        let nullable = scalar_coercion_target(&PhpType::Union(vec![PhpType::Int, PhpType::Void]))
            .expect("boundary");
        assert_eq!(
            scalar_coercion_refusals(&nullable),
            vec![CoercionRefusal::Array]
        );
    }

    /// A union source pays only for the tests its own members can fail.
    #[test]
    fn boxed_source_may_hold_is_decided_by_the_source_members() {
        assert!(boxed_source_may_hold(&PhpType::Mixed, CoercionRefusal::Null));
        assert!(boxed_source_may_hold(&PhpType::Mixed, CoercionRefusal::Array));

        let string_or_null = PhpType::Union(vec![PhpType::Str, PhpType::Void]);
        assert!(boxed_source_may_hold(
            &string_or_null,
            CoercionRefusal::Null
        ));
        assert!(
            !boxed_source_may_hold(&string_or_null, CoercionRefusal::Array),
            "a `?string` source can never be an array, so it must not pay for the array test"
        );

        let array_or_int = PhpType::Union(vec![
            PhpType::Array(Box::new(PhpType::Mixed)),
            PhpType::Int,
        ]);
        assert!(boxed_source_may_hold(&array_or_int, CoercionRefusal::Array));
        assert!(!boxed_source_may_hold(&array_or_int, CoercionRefusal::Null));
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
