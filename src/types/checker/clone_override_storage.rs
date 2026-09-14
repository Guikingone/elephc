//! Purpose:
//! Gives every property a PHP 8.5 `clone($object, $withProperties)` override can reach the
//! runtime-shaped storage PHP's untyped-property semantics require.
//!
//! Called from:
//! - `crate::builtins::callables::clone`'s check hook, which records the destination classes.
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
//! - Reference slots and packed/extern classes are left alone: their slot holds a cell pointer or
//!   a packed field rather than a value, so a Mixed stamp would describe the wrong storage.

use std::collections::BTreeSet;

use crate::types::PhpType;

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
        widen_class(checker, &class_name);
    }
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
        if info
            .property_reference_slots
            .get(slot)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
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
