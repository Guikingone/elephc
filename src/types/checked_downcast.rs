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
    /// A write against a declared instance-property type.
    PropertyStore,
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
        // The property store guards only the RAW-pointer shapes. Its two boxed shapes are held
        // back deliberately: `BoxedToRawObject`'s ok-edge UNBOXES, which would hand the store a
        // value the surrounding `PropSet`/release pair did not lower, and the store's own
        // ownership handling (`release_property_assignment_source_after_retaining_store`) is
        // written against the value the caller produced. Neither is a hierarchy question, so
        // neither belongs in the `RawObject` row.
        GuardPosition::PropertyStore => matches!(
            shape,
            GuardShape::RawObject | GuardShape::RawObjectToBoxed
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

/// A declared type PHP's weak mode treats as a CONVERSION site rather than a match site: one
/// scalar arm, or one `array` arm, optionally joined by `null`.
///
/// `guard_is_php_faithful` keeps such a declaration off the arm-chain guard precisely because no
/// arm test can see a conversion. That is the right call for the arms PHP CONVERTS — but PHP also
/// REFUSES payloads at the very same boundary, and those refusals are invisible to the arm chain
/// for the same reason. This type names the boundary so `weak_boundary_refusals` can state which
/// payloads it rejects and `weak_boundary_is_tag_decidable` can state when that rejection list is
/// COMPLETE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WeakTargetKind {
    /// `string`: converts every scalar and a `Stringable` object; refuses array, null, plain object.
    Str,
    /// `int` or `float`: converts int/float/bool and a NUMERIC string; refuses array, null, object.
    /// A non-numeric string is refused too, which no runtime tag test can see — see
    /// `weak_boundary_is_tag_decidable`.
    Number,
    /// `bool`: converts every scalar including ANY string; refuses array, null, object.
    Bool,
    /// `array`: matches an array and nothing else. The only kind whose verdict is total.
    Array,
}

/// A declared weak-conversion boundary: its target kind plus whether it also admits `null`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WeakCoercionTarget {
    /// What the declaration converts INTO.
    pub kind: WeakTargetKind,
    /// Whether the declaration also carries a `null` arm (`?string`, `string|null`).
    pub accepts_null: bool,
}

/// A runtime payload PHP REFUSES at a weak-conversion boundary, raising a catchable `TypeError`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoercionRefusal {
    /// A `null` payload reaching a NON-nullable declaration.
    Null,
    /// An array payload, refused by every scalar declaration.
    Array,
    /// ANY object payload — the `int`/`float`/`bool` targets, which never accept one.
    Object,
    /// An object payload that is not `Stringable` — the `string` target. PHP 8 makes every class
    /// declaring `__toString()` an implicit `Stringable`, so `instanceof Stringable` IS this
    /// verdict rather than an approximation of it (verified: elephc answers `N`/`Y` exactly as
    /// `php -n` for a plain and a `__toString`-bearing class).
    ObjectUnlessStringable,
    /// Anything that is NOT an array and NOT null — the `array` target's complement, expressed as
    /// one refusal because an accept-chain also catches the tags no other refusal names (a closure,
    /// a resource, a future tag).
    NotArray,
}

/// Returns the weak-conversion boundary `declared` describes, or `None` when it is not one.
///
/// Accepts `string`/`int`/`float`/`bool`/`array` and their `?T` spellings, and nothing else. A
/// declaration with a second non-null arm (`string|array`, `string|D`) is deliberately excluded: an
/// array reaching `string|array` is MATCHED, not refused, so the verdict table below does not
/// describe it and it needs its own proof.
pub(crate) fn weak_coercion_target(declared: &PhpType) -> Option<WeakCoercionTarget> {
    /// Maps one declared arm to its target kind, or `None` when it is not a conversion arm.
    fn arm_kind(member: &PhpType) -> Option<WeakTargetKind> {
        match member {
            PhpType::Str => Some(WeakTargetKind::Str),
            PhpType::Int | PhpType::Float => Some(WeakTargetKind::Number),
            PhpType::Bool => Some(WeakTargetKind::Bool),
            // `iterable` is excluded on purpose: it also admits a `Traversable` OBJECT, so the
            // array tag is not its whole accept set.
            PhpType::Array(_) | PhpType::AssocArray { .. } => Some(WeakTargetKind::Array),
            _ => None,
        }
    }
    match declared {
        PhpType::Union(members) => {
            let mut kind = None;
            let mut accepts_null = false;
            for member in members {
                match member {
                    PhpType::Void => accepts_null = true,
                    other => match (arm_kind(other), kind) {
                        (Some(found), None) => kind = Some(found),
                        // A second non-null arm: not this shape.
                        _ => return None,
                    },
                }
            }
            kind.map(|kind| WeakCoercionTarget { kind, accepts_null })
        }
        single => arm_kind(single).map(|kind| WeakCoercionTarget {
            kind,
            accepts_null: false,
        }),
    }
}

/// PHP's verdict for ONE payload kind at a weak-conversion boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WeakVerdict {
    /// PHP converts it (or matches it outright); the boundary's existing coercion is correct.
    Converts,
    /// PHP raises a catchable `TypeError`; the named refusal test decides it at runtime.
    Refuses(CoercionRefusal),
    /// PHP's verdict depends on something no runtime TAG test can read — today only the
    /// numeric-ness of a string reaching `int`/`float`.
    Undecidable,
}

/// Returns PHP's verdict for a source union MEMBER at `target`.
///
/// MEASURED against `php -n` (php-8.5.6): every scalar target crossed with a boxed `null`, `int`,
/// `float`, `bool`, numeric string, non-numeric string, array, plain object and `Stringable`
/// object. The two rows that are NOT uniform across the scalar targets are the reason this is a
/// per-kind table rather than one list: a non-numeric string is refused by `int`/`float` but
/// converted by `string`/`bool`, and a plain object is refused by all four while a `Stringable` one
/// is converted by `string` alone.
pub(crate) fn weak_member_verdict(target: WeakCoercionTarget, member: &PhpType) -> WeakVerdict {
    if target.kind == WeakTargetKind::Array {
        // The only TOTAL row: an array matches, null matches when declared, everything else —
        // including tags this module does not enumerate — is refused by the accept-chain.
        return match member {
            PhpType::Array(_) | PhpType::AssocArray { .. } => WeakVerdict::Converts,
            PhpType::Void | PhpType::Never if target.accepts_null => WeakVerdict::Converts,
            _ => WeakVerdict::Refuses(CoercionRefusal::NotArray),
        };
    }
    match member {
        PhpType::Void | PhpType::Never => {
            if target.accepts_null {
                WeakVerdict::Converts
            } else {
                WeakVerdict::Refuses(CoercionRefusal::Null)
            }
        }
        PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::False => WeakVerdict::Converts,
        PhpType::Str => match target.kind {
            WeakTargetKind::Str | WeakTargetKind::Bool => WeakVerdict::Converts,
            // `fi("7")` converts, `fi("abc")` throws — a tag test sees only "string".
            WeakTargetKind::Number => WeakVerdict::Undecidable,
            WeakTargetKind::Array => unreachable!("handled above"),
        },
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => {
            WeakVerdict::Refuses(CoercionRefusal::Array)
        }
        PhpType::Object(_) => match target.kind {
            WeakTargetKind::Str => WeakVerdict::Refuses(CoercionRefusal::ObjectUnlessStringable),
            WeakTargetKind::Number | WeakTargetKind::Bool => {
                WeakVerdict::Refuses(CoercionRefusal::Object)
            }
            WeakTargetKind::Array => unreachable!("handled above"),
        },
        // A closure boxes under its own tag, which none of the tests above reads, and `Mixed`
        // absorbs every kind at once.
        _ => WeakVerdict::Undecidable,
    }
}

/// Returns the source's union members, or the source itself when it is not a union.
fn source_members(source: &PhpType) -> &[PhpType] {
    match source {
        PhpType::Union(members) => members,
        single => std::slice::from_ref(single),
    }
}

/// Returns the refusal tests a guard must emit for `source` at `target`, deduped, in the fixed
/// order `null`, array, object, not-array.
///
/// Only the refusals the SOURCE can actually reach are returned, which is what keeps a `?string`
/// argument from paying for an array test it can never fail. A `Mixed` source can reach every
/// refusal, since it absorbs every kind.
pub(crate) fn weak_boundary_refusals(
    target: WeakCoercionTarget,
    source: &PhpType,
) -> Vec<CoercionRefusal> {
    /// Ranks a refusal for the emitted test order. `null` must settle before anything reads a
    /// payload, and the array tag before the object tag so neither reaches the other's test.
    fn rank(refusal: CoercionRefusal) -> u8 {
        match refusal {
            CoercionRefusal::Null => 0,
            CoercionRefusal::Array => 1,
            CoercionRefusal::Object | CoercionRefusal::ObjectUnlessStringable => 2,
            CoercionRefusal::NotArray => 3,
        }
    }
    let mut refusals: Vec<CoercionRefusal> = Vec::new();
    let members: Vec<&PhpType> = if matches!(source, PhpType::Mixed) {
        // `Mixed` is every kind at once; enumerate the representatives so each reachable refusal
        // is named exactly as a union source would name it.
        vec![
            &PhpType::Void,
            &PhpType::Int,
            &PhpType::Str,
            &MIXED_ARRAY_REPRESENTATIVE,
            &MIXED_OBJECT_REPRESENTATIVE,
        ]
    } else {
        source_members(source).iter().collect()
    };
    for member in members {
        if let WeakVerdict::Refuses(refusal) = weak_member_verdict(target, member) {
            if !refusals.contains(&refusal) {
                refusals.push(refusal);
            }
        }
    }
    refusals.sort_by_key(|refusal| rank(*refusal));
    refusals
}

/// The `array` member a bare `Mixed` source stands in for when enumerating reachable refusals.
const MIXED_ARRAY_REPRESENTATIVE: PhpType = PhpType::Iterable;
/// The object member a bare `Mixed` source stands in for when enumerating reachable refusals.
static MIXED_OBJECT_REPRESENTATIVE: PhpType = PhpType::Object(String::new());

/// Returns whether every payload `source` can carry has a verdict the emitted refusal tests
/// DECIDE — the gate a CHECKER relaxation must consult before admitting the flow.
///
/// This is the difference between the guard's two jobs. Emitting refusals is always safe and
/// always an improvement: each one turns a silent miscompile into PHP's own `TypeError`. ACCEPTING
/// a flow the checker used to reject is only safe when the refusal list is COMPLETE, because any
/// payload left undecided reaches the boundary's unguarded coercion — which is exactly the silent
/// divergence this family exists to remove.
pub(crate) fn weak_boundary_is_tag_decidable(
    target: WeakCoercionTarget,
    source: &PhpType,
) -> bool {
    if matches!(source, PhpType::Mixed) {
        // A bare `mixed` can hold a closure, and (at a numeric target) a non-numeric string.
        return false;
    }
    source_members(source)
        .iter()
        .all(|member| weak_member_verdict(target, member) != WeakVerdict::Undecidable)
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

    /// The four scalars, `array`, and their `?T` spellings are conversion boundaries; a declaration
    /// with a second non-null arm is not, because an array reaching `string|array` is MATCHED.
    #[test]
    fn weak_coercion_targets_are_the_single_arm_conversion_declarations() {
        for (declared, kind) in [
            (PhpType::Str, WeakTargetKind::Str),
            (PhpType::Int, WeakTargetKind::Number),
            (PhpType::Float, WeakTargetKind::Number),
            (PhpType::Bool, WeakTargetKind::Bool),
            (PhpType::Array(Box::new(PhpType::Mixed)), WeakTargetKind::Array),
        ] {
            let bare = weak_coercion_target(&declared).expect("a single conversion arm");
            assert_eq!(bare.kind, kind);
            assert!(!bare.accepts_null);

            let nullable = PhpType::Union(vec![declared.clone(), PhpType::Void]);
            let nullable = weak_coercion_target(&nullable).expect("`?T` is a boundary");
            assert_eq!(nullable.kind, kind);
            assert!(nullable.accepts_null);
        }
        assert!(weak_coercion_target(&PhpType::Mixed).is_none());
        assert!(weak_coercion_target(&PhpType::Object("D".to_string())).is_none());
        assert!(
            weak_coercion_target(&PhpType::Iterable).is_none(),
            "`iterable` also admits a Traversable OBJECT, so the array tag is not its accept set"
        );
        assert!(
            weak_coercion_target(&PhpType::Union(vec![
                PhpType::Str,
                PhpType::Array(Box::new(PhpType::Mixed)),
                PhpType::Void,
            ]))
            .is_none(),
            "an array arm MATCHES an array payload, so the verdict table does not describe it"
        );
    }

    /// The measured php-8.5.6 verdict table, per target kind.
    #[test]
    fn weak_member_verdicts_follow_the_measured_php_matrix() {
        let string_target = weak_coercion_target(&PhpType::Str).expect("boundary");
        let int_target = weak_coercion_target(&PhpType::Int).expect("boundary");
        let bool_target = weak_coercion_target(&PhpType::Bool).expect("boundary");
        let array = PhpType::Array(Box::new(PhpType::Mixed));
        let array_target = weak_coercion_target(&array).expect("boundary");
        let object = PhpType::Object("D".to_string());

        // Every scalar converts into every scalar target, except a string into a numeric one.
        for member in [&PhpType::Int, &PhpType::Float, &PhpType::Bool] {
            for target in [string_target, int_target, bool_target] {
                assert_eq!(weak_member_verdict(target, member), WeakVerdict::Converts);
            }
        }
        assert_eq!(
            weak_member_verdict(string_target, &PhpType::Str),
            WeakVerdict::Converts
        );
        assert_eq!(
            weak_member_verdict(bool_target, &PhpType::Str),
            WeakVerdict::Converts
        );
        assert_eq!(
            weak_member_verdict(int_target, &PhpType::Str),
            WeakVerdict::Undecidable,
            "`fi(\"7\")` converts and `fi(\"abc\")` throws — a tag test sees only \"string\""
        );

        // An array is refused by every scalar target; an object is refused by all four, but the
        // `string` one has to consult `Stringable` at runtime to say so.
        for target in [string_target, int_target, bool_target] {
            assert_eq!(
                weak_member_verdict(target, &array),
                WeakVerdict::Refuses(CoercionRefusal::Array)
            );
        }
        assert_eq!(
            weak_member_verdict(string_target, &object),
            WeakVerdict::Refuses(CoercionRefusal::ObjectUnlessStringable)
        );
        assert_eq!(
            weak_member_verdict(int_target, &object),
            WeakVerdict::Refuses(CoercionRefusal::Object)
        );

        // The `array` target is the one TOTAL row: array matches, everything else is refused.
        assert_eq!(weak_member_verdict(array_target, &array), WeakVerdict::Converts);
        for member in [&PhpType::Int, &PhpType::Str, &object, &PhpType::Void] {
            assert_eq!(
                weak_member_verdict(array_target, member),
                WeakVerdict::Refuses(CoercionRefusal::NotArray)
            );
        }
    }

    /// Null is refused only by a non-nullable declaration, at every target kind.
    #[test]
    fn null_is_refused_only_by_a_non_nullable_declaration() {
        let non_nullable = weak_coercion_target(&PhpType::Str).expect("boundary");
        assert_eq!(
            weak_member_verdict(non_nullable, &PhpType::Void),
            WeakVerdict::Refuses(CoercionRefusal::Null)
        );
        let nullable =
            weak_coercion_target(&PhpType::Union(vec![PhpType::Str, PhpType::Void])).expect("b");
        assert_eq!(
            weak_member_verdict(nullable, &PhpType::Void),
            WeakVerdict::Converts
        );
    }

    /// A union source pays only for the tests its own members can fail, and the emitted order is
    /// `null`, array, object — null before anything reads a payload, array before the object tag.
    #[test]
    fn weak_boundary_refusals_are_scoped_to_the_source_and_ordered() {
        let string_target = weak_coercion_target(&PhpType::Str).expect("boundary");
        let wide = PhpType::Union(vec![
            PhpType::Array(Box::new(PhpType::Mixed)),
            PhpType::Str,
            PhpType::Object("UnitEnum".to_string()),
            PhpType::Void,
        ]);
        assert_eq!(
            weak_boundary_refusals(string_target, &wide),
            vec![
                CoercionRefusal::Null,
                CoercionRefusal::Array,
                CoercionRefusal::ObjectUnlessStringable
            ]
        );

        let nullable_string =
            weak_coercion_target(&PhpType::Union(vec![PhpType::Str, PhpType::Void])).expect("b");
        assert_eq!(
            weak_boundary_refusals(
                nullable_string,
                &PhpType::Union(vec![PhpType::Str, PhpType::Void])
            ),
            Vec::new(),
            "a `?string` source at a `?string` target can fail no test, so it pays for none"
        );
    }

    /// The checker gate is STRICTER than the emitter: emitting refusals is always an improvement,
    /// but ACCEPTING a previously-rejected flow requires the refusal list to be COMPLETE.
    #[test]
    fn tag_decidability_gates_the_checker_not_the_emitter() {
        let string_target = weak_coercion_target(&PhpType::Str).expect("boundary");
        let int_target = weak_coercion_target(&PhpType::Int).expect("boundary");
        let wide = PhpType::Union(vec![
            PhpType::Array(Box::new(PhpType::Mixed)),
            PhpType::Bool,
            PhpType::Str,
            PhpType::Int,
            PhpType::Float,
            PhpType::Object("UnitEnum".to_string()),
            PhpType::Void,
        ]);
        assert!(
            weak_boundary_is_tag_decidable(string_target, &wide),
            "every member of Symfony's parameter-bag union has a tag-decidable verdict at `string`"
        );
        assert!(
            !weak_boundary_is_tag_decidable(int_target, &wide),
            "the `string` member's verdict at `int` needs a numeric probe"
        );
        assert!(
            !weak_boundary_is_tag_decidable(string_target, &PhpType::Mixed),
            "a bare `mixed` can hold a closure, whose tag no refusal test reads"
        );
        // ... yet the emitter still guards a bare `mixed`, which is what closes the silent family.
        assert!(!weak_boundary_refusals(string_target, &PhpType::Mixed).is_empty());
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
