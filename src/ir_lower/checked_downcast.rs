//! Purpose:
//! The position-agnostic EMITTER for elephc's checked downcasts. When a value is statically only
//! known to be an ANCESTOR (base class or implemented interface) of the type a slot declares —
//! the base→derived relaxation the checker admits through
//! `crate::types::checker::type_compat::object_types::Checker::checked_downcast_guardable` — this
//! inserts a chain of runtime checks that passes the value through on a match and throws a
//! catchable `\TypeError` (naming the ACTUAL runtime type) when every declared arm mismatches.
//!
//! It also owns the boundary's REFUSAL half (`emit_scalar_coercion_refusal_guard`): a declared
//! SCALAR is a conversion site the arm chain deliberately never guards, but PHP still rejects a
//! `null` or array payload there, and those rejections need the same catchable `\TypeError`.
//!
//! Called from:
//! - `crate::ir_lower::stmt::return_type_guard::emit_checked_downcast_return_guard` (return
//!   position).
//! - `crate::ir_lower::expr::coerce_operands_to_params` (call-argument position, BOTH halves).
//!
//! Key details:
//! - ONE chain for every position. The message wording and the ownership policy of the throw
//!   differ per position; the tests, their order, and the fast paths do not. The argument and
//!   property-store positions land on this same chain, which is why it was lifted out of
//!   `crate::ir_lower::stmt::return_type_guard` rather than copied.
//! - TAG TESTS BEFORE INSTANCEOF, for three independent reasons: (i) SEMANTIC — a legitimate
//!   declared `null`/`array` arm must PASS THROUGH, not fall into the object branch; (ii) MESSAGE
//!   SOUNDNESS — the fail block names the ACTUAL runtime type, and taking the `get_class` route
//!   with a non-object payload yields garbage; (iii) DOMINANCE — an unbox on the ok-edge may only
//!   be reached on an edge where the payload is provably an object. See `phase_one_tag_tests`.
//! - A guard is only ever built out of arms PHP MATCHES rather than CONVERTS INTO. PHP's weak
//!   mode turns a union slot into a conversion site — an int, a float, a bool or a `Stringable`
//!   object reaching a declared `string|D` is coerced, not rejected — and no arm test can see a
//!   conversion. `crate::types::checked_downcast::guard_is_php_faithful` is the gate that keeps
//!   such a declaration off this chain entirely, on both halves.
//! - OWNERSHIP IS SPLIT BY OP, never by a flag on one op. `Op::ThrowCheckedReturnTypeError`
//!   RELEASES the mismatched value, which is sound ONLY at the return position where nothing else
//!   owns it. A position whose caller still owns the value needs its own non-releasing op: a
//!   single op with two ownership policies is how a double free comes back.
//! - The class-hierarchy walk here mirrors the checker's own predicate over `ir_lower`'s tables;
//!   the two must stay in lock-step. The REPRESENTATION rule they share — the part that decides
//!   what a guard would have to do — is not duplicated: both call
//!   `crate::types::checked_downcast::downcast_guard_shape`.
//! - Proven-safe flows emit ZERO ops (see `PHASE 0` in `emit_checked_downcast`). That fast path
//!   is what keeps a guard at every call boundary from taxing programs that were already correct.
//! - THROW-PATH COST, measured: a caught argument-position `TypeError` leaks NOTHING. Looping the
//!   mismatch 1/10/100/1000 times under `--heap-debug` gives allocs == frees at every count, zero
//!   live blocks, and a peak of 384 live bytes that stays CONSTANT from 10 iterations on — so the
//!   per-catch block delta is 0, not merely bounded. Re-measured for a UNION source (a boxed
//!   payload the fail path abandons) and it holds there too. A future position whose throw
//!   abandons already-lowered operands must re-measure this rather than inherit the number.

use std::collections::HashSet;

use crate::ir::{Immediate, IrType, Op, PhpTypePredicate, Terminator};
use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::span::Span;
use crate::types::PhpType;
use crate::types::checked_downcast::{
    CoercionRefusal, GuardArm, GuardPosition, GuardShape, declared_guard_arms,
    downcast_guard_shape, guard_is_php_faithful, shape_is_guardable_at,
};

/// How the callee of a guarded call argument is spelled in the `TypeError` message.
///
/// `FunctionSig` carries no name, so the label is threaded from the call site. Where a call site
/// has not been wired up yet the guard is STILL emitted and only the message degrades (it drops
/// the `F(): ` prefix) — skipping it would let checker acceptance and lowering emission diverge,
/// and that divergence surfaces as a wrong-representation read, not as a compile error.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CalleeLabel<'a> {
    /// A resolved callee: `f` for a free function, `C::m` for a method, `{closure}` for a closure.
    Named(&'a str),
    /// An unresolved callee: the message drops its `F(): ` prefix.
    Unknown,
}

/// The slot a checked downcast guards, carrying everything its `TypeError` wording needs.
#[derive(Clone, Copy, Debug)]
pub(crate) enum DowncastPosition<'a> {
    /// A `return` against the enclosing function's declared return type.
    Return {
        /// The enclosing function's PHP-visible name (`ctx.owner_name()`).
        owner: &'a str,
    },
    /// A call argument against the callee's declared parameter type.
    Argument {
        /// How to spell the callee in the message.
        callee: CalleeLabel<'a>,
        /// Zero-based parameter index; PHP's message numbers arguments from 1.
        index: usize,
        /// The declared parameter name, without its leading `$`.
        param: &'a str,
    },
    /// A write against a declared instance-property type.
    ///
    /// The odd one out in message SHAPE: PHP names the runtime type in the MIDDLE
    /// (`Cannot assign A to property C::$p of type B`), not at the end. The three-part
    /// prefix/runtime-type/suffix machinery already covers that — it is only the suffix that
    /// stops being a fixed word and starts carrying the property and its declared type.
    PropertyStore {
        /// The class DECLARING the property, as PHP spells it in the message.
        class: &'a str,
        /// The property name, without its leading `$`.
        property: &'a str,
    },
}

impl DowncastPosition<'_> {
    /// Returns the EIR block-name prefix for this position's guard blocks.
    ///
    /// The return position deliberately keeps the historical `return_type_guard` prefix: the
    /// extraction of this module out of `crate::ir_lower::stmt::return_type_guard` is required to
    /// be a byte-for-byte no-op on the `--emit-ir` output of every function that already had a
    /// return guard, and the block names are part of that output.
    fn block_prefix(&self) -> &'static str {
        match self {
            DowncastPosition::Return { .. } => "return_type_guard",
            DowncastPosition::Argument { .. } => "arg_type_guard",
            DowncastPosition::PropertyStore { .. } => "prop_type_guard",
        }
    }

    /// Builds the compile-time message prefix — everything up to the runtime type name.
    ///
    /// php-8.5.6 wording, verified against `php -n`:
    /// - return: `F(): Return value must be of type D, ` + `<runtime type>` + ` returned`
    /// - argument: `F(): Argument #N ($p) must be of type D, ` + `<runtime type>` + ` given`
    /// - property: `Cannot assign ` + `<runtime type>` + ` to property C::$p of type D`
    ///
    /// PHP additionally appends `, called in <file> on line <n>` to the ARGUMENT form when the
    /// callee is userland. elephc does not reproduce that tail: it names the call site's file and
    /// line, which an AOT binary would have to bake in from its compile-time path.
    fn message_prefix(&self, declared: &PhpType) -> String {
        let declared = format_declared_type_for_type_error(declared);
        match self {
            DowncastPosition::Return { owner } => {
                format!("{}(): Return value must be of type {}, ", owner, declared)
            }
            DowncastPosition::Argument {
                callee: CalleeLabel::Named(callee),
                index,
                param,
            } => format!(
                "{}(): Argument #{} (${}) must be of type {}, ",
                callee,
                index + 1,
                param,
                declared
            ),
            DowncastPosition::Argument {
                callee: CalleeLabel::Unknown,
                index,
                param,
            } => format!(
                "Argument #{} (${}) must be of type {}, ",
                index + 1,
                param,
                declared
            ),
            DowncastPosition::PropertyStore { .. } => "Cannot assign ".to_string(),
        }
    }

    /// Returns the message tail this position appends after the runtime type name.
    fn message_suffix(&self, declared: &PhpType) -> String {
        match self {
            DowncastPosition::Return { .. } => " returned".to_string(),
            DowncastPosition::Argument { .. } => " given".to_string(),
            DowncastPosition::PropertyStore { class, property } => format!(
                " to property {}::${} of type {}",
                class,
                property,
                format_declared_type_for_type_error(declared)
            ),
        }
    }

    /// Reduces this position to the form the shared shape matrix is expressed over.
    fn kind(&self) -> GuardPosition {
        match self {
            DowncastPosition::Return { .. } => GuardPosition::Return,
            DowncastPosition::Argument { .. } => GuardPosition::Argument,
            DowncastPosition::PropertyStore { .. } => GuardPosition::PropertyStore,
        }
    }
}

/// Returns whether this lowering position emits a guard for `shape` against `declared`.
///
/// Two conditions, and NEITHER of them lives here: the shape/position matrix is
/// `crate::types::checked_downcast::shape_is_guardable_at` and the declared-type admissibility is
/// `guard_is_php_faithful`, both in the shared module the CHECKER also consults. A guard the
/// checker admits but this declines is not a compile error — it is an unguarded representation
/// change at runtime — so the decision may not be re-derived here.
fn shape_is_emittable_at(
    shape: GuardShape,
    declared: &PhpType,
    position: &DowncastPosition<'_>,
) -> bool {
    shape_is_guardable_at(shape, position.kind()) && guard_is_php_faithful(declared, shape)
}

/// Inserts a checked-downcast guard around `value` for the slot declared `declared`, and returns
/// the value to continue lowering with.
///
/// A no-op (returns `value` unchanged) whenever no guard is needed or possible; otherwise emits
/// the guard chain and returns the value as it exists on the guard's ok-edge — which for a boxed
/// source flowing into a concrete object slot is a freshly UNBOXED raw object pointer, not the
/// incoming box.
///
/// PHASE 0 — compile-time short-circuits, in this order:
/// - 0a. no bridgeable representation pair, or a pair this position does not emit yet.
/// - 0b. the declared type has a bare `object` arm. PHP's `object` accepts every object, and
///   `Op::InstanceOf` against the empty class name can never match — emitting one is the exact
///   bug this guard family shipped with once already.
/// - 0c. every member of the value's static type is already covered by a declared arm, so no
///   runtime test could fail (`source_is_proven_covered`).
///
/// PHASE 1 — runtime tag tests for the declared NON-object arms, `null` then array then scalars,
/// each kind in declared order. A value matching one of them is a LEGITIMATE member of the
/// declared type and passes straight through.
///
/// PHASE 2 — one `Op::InstanceOf` per declared class arm, in declared order.
///
/// PHASE 3 — the fail block: build and throw, then `Terminator::Unreachable`.
///
/// PHASE 4 — the ok block: for `GuardShape::BoxedToRawObject` only, `Op::ObjectCast` re-materializes
/// an owned raw object pointer out of the box (`__rt_object_from_mixed`'s object arm increfs and
/// returns the SAME instance). Without it the consumer performs by-offset property access on a
/// `Mixed` box. The result rides the same owned-temporary contract the boundary's existing string
/// coercions already rely on.
pub(crate) fn emit_checked_downcast(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    declared: &PhpType,
    position: DowncastPosition<'_>,
    span: Option<Span>,
) -> LoweredValue {
    let actual = ctx.builder.value_php_type(value.value);
    // -- PHASE 0a --
    let Some(shape) = downcast_guard_shape(declared, &actual) else {
        return value;
    };
    if !shape_is_emittable_at(shape, declared, &position) {
        return value;
    }
    // -- PHASE 0b --
    if declared_accepts_any_object(declared) {
        return value;
    }
    let candidates = crate::types::checked_downcast::declared_instanceof_targets(declared);
    if candidates.is_empty() {
        return value;
    }
    // -- PHASE 0c --
    if source_is_proven_covered(ctx, &actual, declared, &candidates, shape) {
        return value;
    }
    emit_guard_chain(
        ctx, value, declared, &actual, &candidates, shape, position, span,
    )
}

/// Returns whether the value's STATIC type already proves it satisfies `declared`, so no runtime
/// test could ever fail and the guard may emit nothing at all.
///
/// This is the fast path that keeps a guard at every declared boundary from taxing programs that
/// were already correct — for a union source it is also what keeps the common `D|null` into `?D`
/// call from paying for two runtime tests it can never fail.
///
/// `GuardShape::BoxedToRawObject` is deliberately excluded whatever the static type says: its
/// ok-edge does not merely PASS the value, it re-materializes a raw object pointer out of the box
/// (`unbox_guarded_object`), and skipping the chain would hand the consumer the box.
fn source_is_proven_covered(
    ctx: &LoweringContext<'_, '_>,
    actual: &PhpType,
    declared: &PhpType,
    candidates: &[String],
    shape: GuardShape,
) -> bool {
    /// Returns whether one member of the source type is covered by a declared arm.
    fn member_is_covered(
        ctx: &LoweringContext<'_, '_>,
        member: &PhpType,
        arms: &[GuardArm],
        candidates: &[String],
    ) -> bool {
        match member {
            PhpType::Object(name) if !name.is_empty() => candidates
                .iter()
                .any(|target| class_is_subtype_or_equal(ctx, name, target)),
            PhpType::Void | PhpType::Never => arms.contains(&GuardArm::Null),
            PhpType::Array(_) | PhpType::AssocArray { .. } => arms.contains(&GuardArm::Array),
            _ => false,
        }
    }
    if shape == GuardShape::BoxedToRawObject {
        return false;
    }
    let arms = declared_guard_arms(declared);
    match actual {
        PhpType::Union(members) => members
            .iter()
            .all(|member| member_is_covered(ctx, member, &arms, candidates)),
        single => member_is_covered(ctx, single, &arms, candidates),
    }
}

/// Emits the runtime chain: the tag tests, the `Op::InstanceOf` checks, the throwing fail path,
/// and the ok-edge repr fixup. The first matching test continues into the ok block; falling
/// through every test throws a catchable `\TypeError`.
#[allow(clippy::too_many_arguments)]
fn emit_guard_chain(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    declared: &PhpType,
    actual: &PhpType,
    candidates: &[String],
    shape: GuardShape,
    position: DowncastPosition<'_>,
    span: Option<Span>,
) -> LoweredValue {
    let prefix = position.block_prefix();
    let ok_block = ctx.builder.create_named_block(&format!("{}.ok", prefix), Vec::new());
    let fail_block = ctx.builder.create_named_block(&format!("{}.fail", prefix), Vec::new());

    let tag_tests = phase_one_tag_tests(declared, actual);
    let total = tag_tests.len() + candidates.len();
    let mut emitted = 0usize;

    // -- PHASE 1 -- the declared NON-object arms, BEFORE any instanceof. Three independent
    //    reasons, all of which bite as soon as a BOXED value can reach here:
    //    (i)   SEMANTIC — a declared `null`/array/scalar arm is a legitimate member, and a value
    //          matching it must pass THROUGH. `?D` receiving a real null reaches the callee as
    //          null instead of dying on `instanceof D`.
    //    (ii)  MESSAGE SOUNDNESS — the fail block names the ACTUAL runtime type, and it may only
    //          take the `get_class` route once every non-object arm is excluded.
    //    (iii) DOMINANCE — Phase 4's unbox may only be reached where the payload is an object.
    //    This list is EMPTY for a raw-object source, which is why the guard shape a raw source
    //    produces is byte-for-byte what it was before Phase 1 existed — see `phase_one_tag_tests`.
    for predicate in tag_tests {
        let matched = emit_phase_one_test(ctx, value, predicate, span);
        emitted += 1;
        branch_to_ok_or_next(ctx, matched, ok_block, fail_block, prefix, emitted, total);
    }

    // -- PHASE 2 --
    for candidate in candidates {
        let class_data = ctx.intern_class_name(candidate);
        let matched = ctx.emit_value(
            Op::InstanceOf,
            vec![value.value],
            Some(Immediate::Data(class_data)),
            PhpType::Bool,
            Op::InstanceOf.default_effects(),
            span,
        );
        emitted += 1;
        branch_to_ok_or_next(ctx, matched.value, ok_block, fail_block, prefix, emitted, total);
    }

    // -- PHASE 3 --
    ctx.builder.position_at_end(fail_block);
    emit_mismatch_throw(ctx, value, declared, &position, span);
    ctx.builder.terminate(Terminator::Unreachable);

    // -- PHASE 4 --
    ctx.builder.position_at_end(ok_block);
    if shape == GuardShape::BoxedToRawObject {
        return unbox_guarded_object(ctx, value, declared, span);
    }
    value
}

/// Inserts the REFUSAL half of a weak-mode conversion boundary: the runtime payloads PHP rejects
/// outright at a declared `string`/`int`/`float`/`bool`/`array` slot, raising the same catchable
/// `\TypeError` the arm chain does.
///
/// This is the complement of `emit_checked_downcast`, not a variant of it. Such a declaration is
/// where PHP stops MATCHING and starts CONVERTING, so `guard_is_php_faithful` keeps it off the arm
/// chain entirely — an arm test cannot see a conversion and would throw where PHP quietly converts.
/// But the same boundary also REFUSES payloads, and those refusals were invisible for exactly the
/// same reason: nothing tested them, so an array reaching a declared `string` was handed to the
/// boundary's `IToStr` coercion and silently became `""`, and one reaching a declared `int` became
/// its element COUNT. Both continued with exit 0 where PHP throws.
///
/// Which payloads are refused is `crate::types::checked_downcast::weak_boundary_refusals`, over the
/// measured per-target verdict table; only the refusals the SOURCE can actually reach are emitted.
/// The object refusal at a `string` target is decided by `instanceof Stringable`, which is PHP's
/// OWN rule rather than an approximation of it: PHP 8 makes every class declaring `__toString()` an
/// implicit `Stringable`, and elephc's runtime answers that test exactly as `php -n` does.
///
/// The `array` target inverts the chain: it emits an ACCEPT chain (`null` when declared, then the
/// array tag) and throws on fall-through, because `array` matches an array and NOTHING else — so
/// one accept-chain also catches the tags no refusal names (a closure, a resource, a future tag).
///
/// Only a BOXED source is guarded: the tests read a runtime tag, and a raw scalar has none. That is
/// also exactly the family that was silent — a statically-typed array reaching a `string` parameter
/// is a compile error the checker still raises.
///
/// Emits nothing and leaves lowering positioned where it found it when the declaration is not a
/// weak-conversion boundary, when the source is not boxed, or when the source's static type proves
/// it cannot hold any refused payload.
pub(crate) fn emit_scalar_coercion_refusal_guard(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    declared: &PhpType,
    position: DowncastPosition<'_>,
    span: Option<Span>,
) {
    let Some(target) = crate::types::checked_downcast::weak_coercion_target(declared) else {
        return;
    };
    let actual = ctx.builder.value_php_type(value.value);
    if actual.codegen_repr() != PhpType::Mixed {
        return;
    }
    let refused = crate::types::checked_downcast::weak_boundary_refusals(target, &actual);
    if refused.is_empty() {
        return;
    }

    let prefix = "arg_coercion_guard";
    let ok_block = ctx.builder.create_named_block(&format!("{}.ok", prefix), Vec::new());
    let fail_block = ctx.builder.create_named_block(&format!("{}.fail", prefix), Vec::new());
    let total = refused.len();
    for (emitted, refusal) in refused.into_iter().enumerate() {
        let is_last = emitted + 1 == total;
        let next_block = if is_last {
            ok_block
        } else {
            ctx.builder
                .create_named_block(&format!("{}.check", prefix), Vec::new())
        };
        emit_refusal_test(
            ctx, value, refusal, target, fail_block, next_block, ok_block, span,
        );
        if !is_last {
            ctx.builder.position_at_end(next_block);
        }
    }

    ctx.builder.position_at_end(fail_block);
    emit_mismatch_throw(ctx, value, declared, &position, span);
    ctx.builder.terminate(Terminator::Unreachable);
    ctx.builder.position_at_end(ok_block);
}

/// Emits one refusal test and terminates the current block.
///
/// A REFUSAL test that MATCHES branches to `fail_block`; falling through every test is what
/// continues into the boundary's coercion. `CoercionRefusal::NotArray` inverts that — it is the
/// `array` target's ACCEPT test, so a match continues and a miss throws.
#[allow(clippy::too_many_arguments)]
fn emit_refusal_test(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    refusal: CoercionRefusal,
    target: crate::types::checked_downcast::WeakCoercionTarget,
    fail_block: crate::ir::BlockId,
    next_block: crate::ir::BlockId,
    ok_block: crate::ir::BlockId,
    span: Option<Span>,
) {
    // `ObjectUnlessStringable` needs TWO runtime reads (the object tag, then `instanceof
    // Stringable`), so it terminates its own blocks rather than sharing the single-test shape.
    if refusal == CoercionRefusal::ObjectUnlessStringable {
        let is_object = emit_tag_test(ctx, value, PhpTypePredicate::Object, span);
        let stringable_check = ctx
            .builder
            .create_named_block("arg_coercion_guard.stringable", Vec::new());
        ctx.builder.terminate(Terminator::CondBr {
            cond: is_object,
            then_target: stringable_check,
            then_args: Vec::new(),
            else_target: next_block,
            else_args: Vec::new(),
        });
        ctx.builder.position_at_end(stringable_check);
        let stringable = ctx.intern_class_name("Stringable");
        let matched = ctx.emit_value(
            Op::InstanceOf,
            vec![value.value],
            Some(Immediate::Data(stringable)),
            PhpType::Bool,
            Op::InstanceOf.default_effects(),
            span,
        );
        // A `Stringable` object is CONVERTED by `__toString`, so it rejoins the boundary directly:
        // the remaining refusal tests are all about non-object tags it cannot carry.
        ctx.builder.terminate(Terminator::CondBr {
            cond: matched.value,
            then_target: ok_block,
            then_args: Vec::new(),
            else_target: fail_block,
            else_args: Vec::new(),
        });
        return;
    }

    // `NotArray` is the `array` target's ACCEPT chain, so a NULLABLE declaration has to let a real
    // null through BEFORE the array tag is read — otherwise `?array` throws on the very null it
    // declares. The scalar targets never need this: their `null` handling is a REFUSAL test that
    // `weak_boundary_refusals` only lists for a non-nullable declaration.
    if refusal == CoercionRefusal::NotArray && target.accepts_null {
        let is_null = ctx
            .emit_value(
                Op::IsNull,
                vec![value.value],
                None,
                PhpType::Bool,
                Op::IsNull.default_effects(),
                span,
            )
            .value;
        let array_check = ctx
            .builder
            .create_named_block("arg_coercion_guard.array", Vec::new());
        ctx.builder.terminate(Terminator::CondBr {
            cond: is_null,
            then_target: ok_block,
            then_args: Vec::new(),
            else_target: array_check,
            else_args: Vec::new(),
        });
        ctx.builder.position_at_end(array_check);
    }

    let matched = match refusal {
        CoercionRefusal::Null => ctx
            .emit_value(
                Op::IsNull,
                vec![value.value],
                None,
                PhpType::Bool,
                Op::IsNull.default_effects(),
                span,
            )
            .value,
        CoercionRefusal::Array | CoercionRefusal::NotArray => {
            emit_tag_test(ctx, value, PhpTypePredicate::Array, span)
        }
        CoercionRefusal::Object => emit_tag_test(ctx, value, PhpTypePredicate::Object, span),
        CoercionRefusal::ObjectUnlessStringable => unreachable!("handled above"),
    };
    // The `array` target's accept-chain: matching the array tag CONTINUES, missing it throws.
    let (then_target, else_target) = if refusal == CoercionRefusal::NotArray {
        (ok_block, fail_block)
    } else {
        (fail_block, next_block)
    };
    ctx.builder.terminate(Terminator::CondBr {
        cond: matched,
        then_target,
        then_args: Vec::new(),
        else_target,
        else_args: Vec::new(),
    });
}

/// Emits one `Op::TypePredicate` runtime tag test and returns its boolean result.
fn emit_tag_test(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    predicate: PhpTypePredicate,
    span: Option<Span>,
) -> crate::ir::ValueId {
    ctx.emit_value(
        Op::TypePredicate,
        vec![value.value],
        Some(Immediate::TypePredicate(predicate)),
        PhpType::Bool,
        Op::TypePredicate.default_effects(),
        span,
    )
    .value
}

/// Terminates the current block with the guard's `matched ? ok : next-check` branch, creating the
/// next check block when more tests follow and falling into `fail_block` on the last one.
fn branch_to_ok_or_next(
    ctx: &mut LoweringContext<'_, '_>,
    matched: crate::ir::ValueId,
    ok_block: crate::ir::BlockId,
    fail_block: crate::ir::BlockId,
    prefix: &str,
    emitted: usize,
    total: usize,
) {
    let is_last = emitted == total;
    let else_target = if is_last {
        fail_block
    } else {
        ctx.builder
            .create_named_block(&format!("{}.check", prefix), Vec::new())
    };
    ctx.builder.terminate(Terminator::CondBr {
        cond: matched,
        then_target: ok_block,
        then_args: Vec::new(),
        else_target,
        else_args: Vec::new(),
    });
    if !is_last {
        ctx.builder.position_at_end(else_target);
    }
}

/// One Phase-1 runtime test — the declared arms a guard decides by TAG rather than by class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TagTest {
    /// PHP `null`. `Op::IsNull` reads tag 8 for a boxed source and probes the null container for a
    /// raw one, so it is correct for both representations.
    Null,
    /// An array or scalar arm, decided by the matching `Op::TypePredicate` runtime tag.
    Predicate(PhpTypePredicate),
}

/// Returns the Phase-1 tag tests for `declared`, in the fixed KIND order `null`, array, scalars —
/// each kind keeping its arms' declared (source) order.
///
/// The kind order is fixed rather than purely declared because the tests are not independent:
/// `null` has to be settled before anything reads a payload, and the array tag has to be settled
/// before the scalar tags so an array never reaches a scalar arm's comparison. Within a kind the
/// arms are mutually exclusive, so the declared order is preserved purely to keep the emitted
/// chain readable against the source declaration.
///
/// `GuardArm::AnyObject` produces no test here: a declared type carrying it never reaches the
/// chain at all (Phase 0b returns the value untouched, because PHP's bare `object` accepts every
/// object and `Op::InstanceOf` against the empty class name can never match).
///
/// A RAW OBJECT source produces NO Phase-1 test at all: the value is an object at runtime by
/// construction, so every non-object arm is statically excluded and `Op::TypePredicate` would fold
/// to a constant false. The `null` arm is excluded with them — a `null` reaching a declared `?D`
/// is typed `PhpType::Void`, whose representation pairs with nothing, so it never reaches this
/// chain in the first place. Phase 1 is exactly the part a boxed source added.
fn phase_one_tag_tests(declared: &PhpType, actual: &PhpType) -> Vec<TagTest> {
    if matches!(actual.codegen_repr(), PhpType::Object(_)) {
        return Vec::new();
    }
    let arms = declared_guard_arms(declared);
    let mut tests = Vec::new();
    if arms.contains(&GuardArm::Null) {
        tests.push(TagTest::Null);
    }
    if arms.contains(&GuardArm::Array) {
        tests.push(TagTest::Predicate(PhpTypePredicate::Array));
    }
    for arm in &arms {
        if let GuardArm::Scalar(predicate) = arm {
            tests.push(TagTest::Predicate(*predicate));
        }
    }
    tests
}

/// Emits one Phase-1 test and returns its boolean result.
fn emit_phase_one_test(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    test: TagTest,
    span: Option<Span>,
) -> crate::ir::ValueId {
    let (op, immediate) = match test {
        TagTest::Null => (Op::IsNull, None),
        TagTest::Predicate(predicate) => {
            (Op::TypePredicate, Some(Immediate::TypePredicate(predicate)))
        }
    };
    ctx.emit_value(
        op,
        vec![value.value],
        immediate,
        PhpType::Bool,
        op.default_effects(),
        span,
    )
    .value
}

/// Emits the position-appropriate throw for a value that matched no declared arm.
///
/// OWNERSHIP is the whole decision here, and it is NOT a property of the position alone:
/// - RETURN always releases. A return value the caller never receives has no other owner, whatever
///   produced it.
/// - ARGUMENT never releases. The caller's own local still owns the argument, and releasing here
///   is a double free.
/// - A PROPERTY STORE is decided per VALUE. `$this->p = $x` leaves the local owning `$x`, so
///   releasing would double-free it; `$this->p = f()` hands over an owning temporary whose only
///   other release — `release_property_assignment_source_after_retaining_store`, emitted AFTER the
///   store — the throw path never reaches, so not releasing leaks it.
///
/// That per-value test is `value_is_owning_temporary` MINUS `value_is_owned_unboxed_local_load`,
/// the exact pair `lower_throw` uses for the same question about a thrown value. The subtraction is
/// not cosmetic: `main` classifies a concrete-object LOCAL load as an owning temporary for its
/// provisional unbox-release tracking, and taking that at face value made `$h->slot = $x` release a
/// reference the local still holds — caught as `heap debug detected bad refcount` by a 300-iteration
/// probe, not by any type or validation check.
///
/// The releasing op is REUSED rather than duplicated for the third position. Its `" returned"`
/// tail used to be baked into codegen; it now takes an optional suffix operand, and the return
/// position keeps supplying none — which is what keeps that position's emitted assembly
/// byte-for-byte what it was. Releasing must stay INSIDE the throw op: the message names the
/// value's ACTUAL runtime class, read from the object header, so any release emitted before the
/// throw would be a use-after-free on exactly the path that needs the name.
fn emit_mismatch_throw(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    declared: &PhpType,
    position: &DowncastPosition<'_>,
    span: Option<Span>,
) {
    let prefix_text = position.message_prefix(declared);
    let prefix = ctx.intern_string(&prefix_text);
    let releases = match position {
        DowncastPosition::Return { .. } => true,
        DowncastPosition::Argument { .. } => false,
        DowncastPosition::PropertyStore { .. } => {
            ctx.value_is_owning_temporary(value)
                && !ctx.value_is_owned_unboxed_local_load(value.value)
        }
    };
    // The return position is the one that supplies no suffix operand.
    let suffix = match position {
        DowncastPosition::Return { .. } => None,
        other => {
            let suffix_data = ctx.intern_string(&other.message_suffix(declared));
            Some(
                ctx.emit_value(
                    Op::ConstStr,
                    Vec::new(),
                    Some(Immediate::Data(suffix_data)),
                    PhpType::Str,
                    Op::ConstStr.default_effects(),
                    span,
                )
                .value,
            )
        }
    };
    let mut operands = vec![value.value];
    operands.extend(suffix);
    let op = if releases {
        Op::ThrowCheckedReturnTypeError
    } else {
        Op::ThrowCheckedTypeError
    };
    ctx.emit_void(
        op,
        operands,
        Some(Immediate::Data(prefix)),
        op.default_effects(),
        span,
    );
}

/// Re-materializes an owned raw object pointer out of the guarded box on the ok-edge.
///
/// Reached only for `GuardShape::BoxedToRawObject`, and only on an edge the Phase-2 tests have
/// proven carries an object payload. `Op::ObjectCast` lowers to `__rt_object_from_mixed`, whose
/// object arm increfs and returns the SAME instance, so the result is an OWNED ALIAS of the boxed
/// object rather than a copy. That is what makes it safe for the consumer to keep, and it rides
/// the same owned-temporary contract the call boundary's existing string coercions already use.
fn unbox_guarded_object(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    declared: &PhpType,
    span: Option<Span>,
) -> LoweredValue {
    let unboxed = ctx.emit_value(
        Op::ObjectCast,
        vec![value.value],
        None,
        declared.clone(),
        Op::ObjectCast.default_effects(),
        span,
    );
    LoweredValue {
        value: unboxed.value,
        ir_type: IrType::Heap(crate::ir::IrHeapKind::Object),
    }
}

/// Returns whether the declared type has a bare `object` arm — PHP's `object` pseudo-type, which
/// the checker models as `PhpType::Object("")` (an EMPTY class name, not a class called `""`).
/// `object` accepts EVERY object, so such a slot needs no runtime downcast check at all.
///
/// Without this, `function mk(): object { return new A(); }` fell through to the guard chain with
/// the empty name as its sole candidate, emitting `Op::InstanceOf` against class `""` — a check
/// no runtime class can ever satisfy — so every such return died at runtime with
/// `TypeError: mk(): Return value must be of type , A returned`.
fn declared_accepts_any_object(ty: &PhpType) -> bool {
    match ty {
        PhpType::Object(name) => name.is_empty(),
        PhpType::Union(members) => members.iter().any(declared_accepts_any_object),
        _ => false,
    }
}

/// Formats a declared PHP type using PHP's own type-declaration syntax, matching how a
/// `TypeError` type-mismatch message renders it: a two-member `T|null` union (the shape nullable
/// declarations normalize to) renders as `?T`; other unions render in PHP's CANONICAL arm order;
/// everything else renders as a single type name.
///
/// PHP does not echo the declared order back. It renders class/interface arms first, keeping their
/// declared order among themselves, then the built-in arms in a FIXED order of its own
/// (`union_arm_rank`). php-8.5.6, verified against `php -n`:
/// `array|D|null`, `D|array|null` and `null|array|D` all render `D|array|null`; `array|E|D` renders
/// `E|D|array`; `null|bool|float|int|string|array|D` renders `D|array|string|int|float|bool|null`.
fn format_declared_type_for_type_error(ty: &PhpType) -> String {
    let PhpType::Union(members) = ty else {
        return format_type_member(ty);
    };
    if members.len() == 2 && members.iter().any(|member| matches!(member, PhpType::Void)) {
        if let Some(non_void) = members.iter().find(|member| !matches!(member, PhpType::Void)) {
            return format!("?{}", format_type_member(non_void));
        }
    }
    let mut ordered: Vec<&PhpType> = members.iter().collect();
    ordered.sort_by_key(|member| union_arm_rank(member));
    ordered
        .into_iter()
        .map(format_type_member)
        .collect::<Vec<_>>()
        .join("|")
}

/// Returns a member's sort rank in PHP's canonical union rendering. A stable sort on it keeps the
/// declared order within each rank, which is what makes the class arms come out in source order.
///
/// Only the class, array and `null` ranks are reachable from a guarded message for a BOXED source,
/// and only those plus `int`/`float`/`bool`/`false` for a raw-object one — every other declaration
/// is refused by `crate::types::checked_downcast::guard_is_php_faithful`. The remaining ranks are
/// carried so the ordering stays a faithful model of PHP's rather than a special case of it.
fn union_arm_rank(member: &PhpType) -> u8 {
    match member {
        PhpType::Object(name) if !name.is_empty() => 0,
        PhpType::Object(_) => 1,
        PhpType::Callable => 2,
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => 3,
        PhpType::Str => 4,
        PhpType::Int => 5,
        PhpType::Float => 6,
        PhpType::Bool => 7,
        PhpType::False => 8,
        PhpType::Void => 10,
        _ => 9,
    }
}

/// Formats a single (non-union) `PhpType` member using PHP's type-declaration spelling.
fn format_type_member(ty: &PhpType) -> String {
    match ty {
        // `Object("")` is PHP's bare `object` pseudo-type, spelled `object` in a declaration.
        PhpType::Object(name) if name.is_empty() => "object".to_string(),
        PhpType::Object(name) => name.clone(),
        PhpType::Int => "int".to_string(),
        PhpType::Float => "float".to_string(),
        PhpType::Str => "string".to_string(),
        PhpType::Bool => "bool".to_string(),
        PhpType::Void => "null".to_string(),
        PhpType::Mixed => "mixed".to_string(),
        PhpType::Iterable => "iterable".to_string(),
        PhpType::Callable => "callable".to_string(),
        PhpType::Array(_) | PhpType::AssocArray { .. } => "array".to_string(),
        other => format!("{:?}", other),
    }
}

/// True if `class_name` is provably `ancestor_name` or a subtype of it (class inheritance or
/// interface implementation). Mirrors `Checker::is_subclass_of`/`object_type_implements_interface`
/// (`src/types/checker/type_compat/object_types.rs`) over `ir_lower`'s own class/interface tables.
fn class_is_subtype_or_equal(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
    ancestor_name: &str,
) -> bool {
    class_name == ancestor_name
        || is_subclass_of(ctx, class_name, ancestor_name)
        || object_type_implements_interface(ctx, class_name, ancestor_name)
}

/// Returns true if `class_name` is or inherits from `ancestor_name` (excluding self equality).
fn is_subclass_of(ctx: &LoweringContext<'_, '_>, class_name: &str, ancestor_name: &str) -> bool {
    let mut current = ctx.classes.get(class_name).and_then(|class| class.parent.clone());
    while let Some(parent_name) = current {
        if parent_name == ancestor_name {
            return true;
        }
        current = ctx.classes.get(&parent_name).and_then(|class| class.parent.clone());
    }
    false
}

/// Returns true if `class_name` directly implements `interface_name` (not via inheritance).
fn class_implements_interface(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
    interface_name: &str,
) -> bool {
    ctx.classes.get(class_name).is_some_and(|class_info| {
        class_info.interfaces.iter().any(|name| name == interface_name)
    })
}

/// Returns true if `type_name` (a class or interface) implements `interface_name`, checking
/// direct implementation and interface inheritance chains.
fn object_type_implements_interface(
    ctx: &LoweringContext<'_, '_>,
    type_name: &str,
    interface_name: &str,
) -> bool {
    if ctx.classes.contains_key(type_name) {
        return class_implements_interface(ctx, type_name, interface_name);
    }
    if ctx.interfaces.contains_key(type_name) {
        return type_name == interface_name
            || interface_extends_interface(ctx, type_name, interface_name);
    }
    false
}

/// Returns true if `interface_name` is or transitively extends `ancestor_name`. DFS with cycle
/// detection over interface parent chains.
fn interface_extends_interface(
    ctx: &LoweringContext<'_, '_>,
    interface_name: &str,
    ancestor_name: &str,
) -> bool {
    if interface_name == ancestor_name {
        return true;
    }
    let mut stack = vec![interface_name.to_string()];
    let mut seen = HashSet::new();
    while let Some(current_name) = stack.pop() {
        if !seen.insert(current_name.clone()) {
            continue;
        }
        let Some(interface_info) = ctx.interfaces.get(&current_name) else {
            continue;
        };
        for parent_name in &interface_info.parents {
            if parent_name == ancestor_name {
                return true;
            }
            stack.push(parent_name.clone());
        }
    }
    false
}
