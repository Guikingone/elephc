//! Purpose:
//! The position-agnostic EMITTER for elephc's checked downcasts. When a value is statically only
//! known to be an ANCESTOR (base class or implemented interface) of the type a slot declares —
//! the base→derived relaxation the checker admits through
//! `crate::types::checker::type_compat::object_types::Checker::checked_downcast_guardable` — this
//! inserts a chain of runtime checks that passes the value through on a match and throws a
//! catchable `\TypeError` (naming the ACTUAL runtime type) when every declared arm mismatches.
//!
//! Called from:
//! - `crate::ir_lower::stmt::return_type_guard::emit_checked_downcast_return_guard` (return
//!   position).
//! - `crate::ir_lower::expr::coerce_operands_to_params` (call-argument position).
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
//!   be reached on an edge where the payload is provably an object. Which shapes need those tests
//!   at all is recorded, with its proof, in `emit_guard_chain`.
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
//!   per-catch block delta is 0, not merely bounded. A future position whose throw abandons
//!   already-lowered operands must re-measure this rather than inherit the number.

use std::collections::HashSet;

use crate::ir::{IrType, Immediate, Op, Terminator};
use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::span::Span;
use crate::types::PhpType;
use crate::types::checked_downcast::{GuardShape, declared_guard_arms, downcast_guard_shape};

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
        }
    }

    /// Builds the compile-time message prefix — everything up to the runtime type name.
    ///
    /// php-8.5.6 wording, verified against `php -n`:
    /// - return: `F(): Return value must be of type D, ` + `<runtime type>` + ` returned`
    /// - argument: `F(): Argument #N ($p) must be of type D, ` + `<runtime type>` + ` given`
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
        }
    }

    /// Returns the fixed message tail this position appends after the runtime type name.
    fn message_suffix(&self) -> &'static str {
        match self {
            DowncastPosition::Return { .. } => " returned",
            DowncastPosition::Argument { .. } => " given",
        }
    }
}

/// Returns whether this lowering position emits a guard for `shape` yet.
///
/// The shape/position matrix is staged deliberately, and this is the ONE place that records it:
/// - `RawObject`/`RawObjectToBoxed` at the RETURN position reproduce exactly what the return
///   guard did before this module existed.
/// - `BoxedToRawObject` at an ARGUMENT position is the shape that turns a boxed value silently
///   read as a raw object pointer (garbage, and a segfault as soon as the callee dispatches on
///   it) into PHP's own catchable `TypeError`.
/// - `BoxedToBoxed` is NOT emitted anywhere yet, and it must not be enabled without pairing it
///   with the checker's own admission decision: a boxed value reaching a declared `string|array`
///   slot is WEAK-MODE COERCED by PHP, not rejected, so a bare tag-test chain would throw where
///   PHP converts. That pairing is the union-source slice of this campaign.
/// - `RawObjectToBoxed` at an ARGUMENT position is withheld for the same reason: a `Stringable`
///   object reaching a declared `string|D` slot is coerced by PHP, not rejected.
fn shape_is_emittable_at(shape: GuardShape, position: &DowncastPosition<'_>) -> bool {
    match (shape, position) {
        (
            GuardShape::RawObject | GuardShape::RawObjectToBoxed,
            DowncastPosition::Return { .. },
        ) => true,
        (
            GuardShape::RawObject | GuardShape::BoxedToRawObject,
            DowncastPosition::Argument { .. },
        ) => true,
        _ => false,
    }
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
/// - 0c. the value's static class is already a proven subtype-or-equal of a declared arm.
///
/// PHASE 1 — runtime tag tests for the declared NON-object arms, in declared order. Only a boxed
/// source can carry a non-object payload, so these are emitted only for a boxed source.
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
    if !shape_is_emittable_at(shape, &position) {
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
    if let PhpType::Object(actual_name) = &actual {
        if candidates
            .iter()
            .any(|target| class_is_subtype_or_equal(ctx, actual_name, target))
        {
            return value;
        }
    }
    emit_guard_chain(ctx, value, declared, &candidates, shape, position, span)
}

/// Emits the runtime chain: the tag tests, the `Op::InstanceOf` checks, the throwing fail path,
/// and the ok-edge repr fixup. The first matching test continues into the ok block; falling
/// through every test throws a catchable `\TypeError`.
fn emit_guard_chain(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    declared: &PhpType,
    candidates: &[String],
    shape: GuardShape,
    position: DowncastPosition<'_>,
    span: Option<Span>,
) -> LoweredValue {
    let prefix = position.block_prefix();
    let ok_block = ctx.builder.create_named_block(&format!("{}.ok", prefix), Vec::new());
    let fail_block = ctx.builder.create_named_block(&format!("{}.fail", prefix), Vec::new());

    // -- PHASE 1 is VACUOUS for every shape/position `shape_is_emittable_at` admits today, and
    //    that is a proof, not an omission:
    //    * `RawObject`/`RawObjectToBoxed` carry a raw object POINTER, so the value is an object
    //      at runtime by construction and every declared non-object arm is statically excluded.
    //    * `BoxedToRawObject` has a declared `codegen_repr()` of `Object(_)`, which only a plain
    //      `PhpType::Object` produces (a union's repr is `Mixed`), so the declared type has
    //      exactly one arm and it is an object arm.
    //    Enabling `BoxedToBoxed`, or `RawObjectToBoxed` at an argument position, breaks both
    //    legs and MUST add the tag tests first — see `shape_is_emittable_at`.
    debug_assert!(
        declared_has_no_testable_non_object_arm(declared)
            || matches!(shape, GuardShape::RawObject | GuardShape::RawObjectToBoxed),
        "a boxed source into a slot with non-object arms needs Phase-1 tag tests"
    );

    let total = candidates.len();
    let mut emitted = 0usize;
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

/// Returns whether `declared` has no arm a Phase-1 runtime tag test would have to cover.
///
/// Used only by the debug assertion in `emit_guard_chain` that pins the vacuous-Phase-1 proof:
/// if a future shape lets a BOXED payload reach a slot that declares a `null`, array, or scalar
/// arm, the guard would send a legitimate value down the fail path.
fn declared_has_no_testable_non_object_arm(declared: &PhpType) -> bool {
    use crate::types::checked_downcast::GuardArm;
    declared_guard_arms(declared)
        .iter()
        .all(|arm| matches!(arm, GuardArm::AnyObject | GuardArm::Class(_)))
}

/// Emits the position-appropriate throw for a value that matched no declared arm.
///
/// The return position uses `Op::ThrowCheckedReturnTypeError`, which RELEASES the mismatched
/// value — sound there and only there, because a return value the caller never receives has no
/// other owner. Every other position uses `Op::ThrowCheckedTypeError`, which does not: the
/// caller's own local still owns the argument, and releasing it here is a double free.
fn emit_mismatch_throw(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    declared: &PhpType,
    position: &DowncastPosition<'_>,
    span: Option<Span>,
) {
    let prefix_text = position.message_prefix(declared);
    let prefix = ctx.intern_string(&prefix_text);
    match position {
        DowncastPosition::Return { .. } => {
            ctx.emit_void(
                Op::ThrowCheckedReturnTypeError,
                vec![value.value],
                Some(Immediate::Data(prefix)),
                Op::ThrowCheckedReturnTypeError.default_effects(),
                span,
            );
        }
        DowncastPosition::Argument { .. } => {
            let suffix_data = ctx.intern_string(position.message_suffix());
            let suffix = ctx.emit_value(
                Op::ConstStr,
                Vec::new(),
                Some(Immediate::Data(suffix_data)),
                PhpType::Str,
                Op::ConstStr.default_effects(),
                span,
            );
            ctx.emit_void(
                Op::ThrowCheckedTypeError,
                vec![value.value, suffix.value],
                Some(Immediate::Data(prefix)),
                Op::ThrowCheckedTypeError.default_effects(),
                span,
            );
        }
    }
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
/// declarations normalize to) renders as `?T`; other unions join arms with `|` in source order;
/// everything else renders as a single type name.
fn format_declared_type_for_type_error(ty: &PhpType) -> String {
    if let PhpType::Union(members) = ty {
        if members.len() == 2 && members.iter().any(|member| matches!(member, PhpType::Void)) {
            if let Some(non_void) = members.iter().find(|member| !matches!(member, PhpType::Void)) {
                return format!("?{}", format_type_member(non_void));
            }
        }
        return members
            .iter()
            .map(format_type_member)
            .collect::<Vec<_>>()
            .join("|");
    }
    format_type_member(ty)
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
