//! Purpose:
//! Gives every property a PHP 8.5 `clone($object, $withProperties)` override can reach the
//! runtime-shaped storage PHP's untyped-property semantics require.
//!
//! Called from:
//! - `crate::builtins::callables::clone`'s check hook, which records the destination classes.
//! - `Checker::infer_closure_call_type`, for a runtime string or boxed callable whose callee may
//!   be `clone` and whose destination is therefore unknowable.
//! - `crate::types::checker::check_types_with_options`, which applies the widening once every
//!   body has been checked.
//!
//! Key details:
//! - A PHP property WITHOUT a declared type is `mixed`: it accepts an int, a string, `null`, an
//!   array and an object with no coercion and no type error. This compiler instead infers such a
//!   slot from its default (`public $u = 0;` becomes `int`, `public $u;` becomes `null`), which
//!   is sound only because `refine_object_property_type` re-widens the slot the moment an
//!   ordinary `$o->u = $value;` assigns something else. A `clone()` override is exactly such an
//!   assignment, but it is SYNTHESIZED after checking, so nothing ever widened the slot for it:
//!   an inferred `int` slot silently coerced `"hello"` to `int(0)`, and an inferred `null` slot
//!   failed the whole build with `prop_set assigning PHP type Mixed to U::$u with PHP type Void`.
//! - The widening is applied to DESTINATION classes only, never to every class in the program.
//!   The `clone` check hook knows the first argument's inferred type, so `clone($item, [...])`
//!   widens `Item` and its subclasses and leaves every unrelated class alone. Only a call whose
//!   object type is runtime-shaped widens every user class.
//! - Declared slots keep their declared type. PHP applies weak-mode property typing to them, and
//!   `mixed_property_type_guard` already implements it for runtime-shaped values.
//! - Packed/extern classes are left alone: their slot holds a packed field rather than a value, so
//!   a Mixed stamp would describe the wrong storage.
//! - A reference slot IS widened when its property is undeclared. `property_reference_slots`
//!   records that the slot physically holds a shared cell; `properties[slot].1` records that
//!   cell's PAYLOAD, and an undeclared property's payload is `mixed` with or without an alias.
//! - The same destination set also reserves the per-instance property HASH an override key whose
//!   name matches no declared slot needs. It is a storage capability only: creating the property
//!   still emits php 8.5's `Creation of dynamic property C::$n is deprecated`, and the checker
//!   never consults `has_property_hash_storage()`, so `$ordinary->undeclared = 1` keeps its
//!   compile-time refusal.

use std::collections::BTreeSet;

use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::{PhpType, TypeEnv};

use super::Checker;

/// The clone-with destination classes one program can reach.
#[derive(Debug, Default, Clone)]
pub struct CloneOverrideDestinations {
    /// A site passes an object whose class the checker could not name.
    pub any_class: bool,
    /// Statically known destination class names, before subclass expansion.
    pub classes: BTreeSet<String>,
}

impl CloneOverrideDestinations {
    /// Records one two-argument `clone()` site from its inferred first-argument type.
    pub fn record(&mut self, object_ty: &PhpType) {
        match object_ty.codegen_repr() {
            PhpType::Object(class) if !class.is_empty() => {
                self.classes.insert(class);
            }
            _ => self.any_class = true,
        }
    }

    /// Returns whether this program reaches a two-argument `clone()` at all.
    fn is_empty(&self) -> bool {
        !self.any_class && self.classes.is_empty()
    }
}

/// Records the widening destination a first-class `clone()` callable call reaches.
///
/// `clone(...)` is checked through its callable SIGNATURE, not through the builtin check hook, so
/// nothing recorded the destination for `$f = clone(...); $f($object, [...])`. An untyped
/// `public $u = 0;` slot then coerced an override's string back to `int(0)`, and an untyped
/// `public $u;` slot failed the build with `prop_set assigning PHP type Mixed to U::$u with PHP
/// type Void`, exactly the two failures `widen_clone_override_property_storage` exists to prevent.
///
/// Only a site that can actually carry the optional `withProperties` argument records anything,
/// and a statically named first argument records only its own class, so unrelated classes are
/// left alone.
pub(in crate::types::checker) fn record_callable_clone_override_destination(
    checker: &mut Checker,
    args: &[Expr],
    env: &TypeEnv,
) -> Result<(), CompileError> {
    if !clone_call_may_carry_overrides(args) {
        return Ok(());
    }
    match args.first() {
        // An unpack or a named argument does not pin which expression is the object, so the
        // destination really is unknowable and every user class has to be widened.
        Some(object)
            if !matches!(object.kind, ExprKind::Spread(_) | ExprKind::NamedArg { .. }) =>
        {
            let object_ty = checker.infer_type(object, env)?;
            checker.clone_override_destinations.record(&object_ty);
        }
        _ => checker.clone_override_destinations.any_class = true,
    }
    Ok(())
}

/// Records the widening a runtime string or boxed callable invocation can reach.
///
/// `$f = 'clone'; $f($object, [...])` resolves its callee at runtime, so neither the builtin check
/// hook nor the callable-signature path ever sees a `clone` call. The callable set behind such a
/// variable is not tracked as an exact string, so it MAY be `clone`, and the only sound record is
/// the runtime-shaped one: every user class this program declares is widened, exactly like a
/// `clone($object, [...])` whose first argument type is boxed.
///
/// The widening it requests is PHP-correct on its own terms: it only re-stamps property slots that
/// carry no declared type, and a PHP property without a declared type IS `mixed`. A declared slot,
/// a packed class and a checker-injected class are all left alone by `widen_class`.
///
/// A one-argument invocation can never write a property, so it records nothing.
pub(in crate::types::checker) fn record_runtime_callable_clone_override_destination(
    checker: &mut Checker,
    args: &[Expr],
) {
    if !clone_call_may_carry_overrides(args) {
        return;
    }
    checker.clone_override_destinations.any_class = true;
}

/// Returns whether a `clone()` call site can carry the optional `withProperties` overrides.
///
/// A one-argument `clone($object)` never writes a property, so it must not widen anything.
fn clone_call_may_carry_overrides(args: &[Expr]) -> bool {
    args.len() >= 2
        || args.iter().any(|arg| match &arg.kind {
            // An unpack carries an unknown number of values, so the overrides may be inside it.
            ExprKind::Spread(_) => true,
            ExprKind::NamedArg { name, .. } => name == "withProperties",
            _ => false,
        })
}

/// Widens every undeclared property slot a `clone()` override can write to `mixed`.
///
/// Runs after body checking, next to `apply_reference_property_promotions`, because both change
/// property STORAGE metadata that EIR lowering re-reads from `Checker::classes` rather than from
/// a per-span record.
pub(super) fn widen_clone_override_property_storage(checker: &mut Checker) {
    let destinations = std::mem::take(&mut checker.clone_override_destinations);
    if destinations.is_empty() {
        return;
    }
    let sel = destination_classes(checker, &destinations);
    for class_name in sel {
        reserve_property_hash_storage(checker, &class_name);
        widen_class(checker, &class_name);
    }
}

/// Reserves the per-instance property hash one class needs for an UNKNOWN override key.
///
/// php 8.5 stores `clone($object, ["zz" => "x"])` on an ordinary class and deprecates the
/// creation; without a hash there is nowhere to put it, and the applicator could only refuse the
/// write. Reserving it here rather than on every object keeps the cost to the destination classes
/// this program's own `clone()` sites can actually reach: one trailing pointer per instance, and
/// the clone/free/GC traversal `#[\AllowDynamicProperties]` instances already pay.
///
/// The flag is a STORAGE capability, never PHP's permission. `allow_dynamic_properties` stays
/// exactly as the attribute left it, so `ClassInfo::dynamic_property_creation_is_deprecated()`
/// still separates the exempt classes from the deprecated ones, and `__set()` keeps its
/// precedence in `crate::ir_lower::clone_overrides::arms`.
fn reserve_property_hash_storage(checker: &mut Checker, class_name: &str) {
    // Same exclusions as `widen_class`: a checker-injected builtin, a packed class and an enum
    // all own their physical layout together with the code that reads it.
    if !checker.declared_classes.contains(class_name)
        || checker.packed_classes.contains_key(class_name)
        || checker.enums.contains_key(class_name)
        || super::builtin_stdclass::is_stdclass(class_name)
    {
        return;
    }
    let Some(info) = checker.classes.get_mut(class_name) else {
        return;
    };
    if info.has_property_hash_storage() {
        return;
    }
    info.clone_override_property_storage = true;
}

/// Expands the recorded destinations into the exact set of classes to widen.
fn destination_classes(checker: &Checker, destinations: &CloneOverrideDestinations) -> Vec<String> {
    let mut names = BTreeSet::new();
    for (candidate, _) in checker.classes.iter() {
        if destinations.any_class
            || destinations.classes.contains(candidate)
            || destinations
                .classes
                .iter()
                .any(|destination| checker.is_subclass_of(candidate, destination))
        {
            names.insert(candidate.clone());
        }
    }
    names.into_iter().collect()
}

/// Stamps one class's undeclared, non-reference instance property slots as `mixed`.
fn widen_class(checker: &mut Checker, class_name: &str) {
    // Only a class this program DECLARES is widened. A checker-injected builtin (SPL, Reflection,
    // DateTime, the throwables) owns its slot representation together with the code that reads it,
    // and `clone()` on one of those is refused by the applicator planner anyway.
    if !checker.declared_classes.contains(class_name)
        || checker.packed_classes.contains_key(class_name)
        || checker.enums.contains_key(class_name)
    {
        return;
    }
    let Some(info) = checker.classes.get_mut(class_name) else {
        return;
    };
    for slot in 0..info.properties.len() {
        let declared = info
            .property_declared_slots
            .get(slot)
            .copied()
            .unwrap_or_else(|| info.declared_properties.contains(&info.properties[slot].0));
        if declared {
            continue;
        }
        // A reference slot is NOT skipped. `property_reference_slots` says the slot physically
        // holds a shared cell, while `properties[slot].1` says what that cell's PAYLOAD is, and an
        // undeclared property's payload is `mixed` whether or not an alias exists. Skipping it made
        // `$alias = &$o->u; clone($o, ["u" => "hello"]);` coerce the override back to the payload
        // type the default happened to infer. Declared slots already left this loop above, so a
        // declared typed reference property keeps its declared payload type and its PHP weak-mode
        // coercion and type-error behavior.
        // A hooked property has no backing value of its own to widen, and its accessors carry
        // their own declared types.
        if info
            .property_hooks
            .get(&info.properties[slot].0)
            .is_some_and(|hooks| hooks.any())
        {
            continue;
        }
        info.properties[slot].1 = PhpType::Mixed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    /// Wraps one argument expression in `...$arg`.
    fn spread(arg: Expr) -> Expr {
        Expr::new(ExprKind::Spread(Box::new(arg)), Span::dummy())
    }

    /// Wraps one argument expression in `name: $arg`.
    fn named(name: &str, arg: Expr) -> Expr {
        Expr::new(
            ExprKind::NamedArg {
                name: name.to_string(),
                value: Box::new(arg),
            },
            Span::dummy(),
        )
    }

    /// A one-argument invocation writes no property, so it must never widen any storage.
    ///
    /// This is the gate every recording path shares, including the runtime string callable one
    /// where the callee is not knowable and the widening would otherwise reach every user class.
    #[test]
    fn one_argument_clone_invocations_never_widen_property_storage() {
        assert!(!clone_call_may_carry_overrides(&[]));
        assert!(!clone_call_may_carry_overrides(&[Expr::var("object")]));
        assert!(!clone_call_may_carry_overrides(&[named(
            "object",
            Expr::var("object")
        )]));
    }

    /// Every shape that can carry `withProperties` records a widening destination.
    #[test]
    fn clone_invocations_that_can_carry_overrides_are_recorded() {
        assert!(clone_call_may_carry_overrides(&[
            Expr::var("object"),
            Expr::var("overrides"),
        ]));
        assert!(clone_call_may_carry_overrides(&[spread(Expr::var("args"))]));
        assert!(clone_call_may_carry_overrides(&[named(
            "withProperties",
            Expr::var("overrides")
        )]));
    }
}
