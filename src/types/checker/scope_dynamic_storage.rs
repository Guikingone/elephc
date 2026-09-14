//! Purpose:
//! Reserves the per-instance property hash a class needs when a reachable MUTATION addresses a
//! strict ancestor's private property name from outside the class that declared it.
//!
//! Called from:
//! - `crate::types::checker::stmt_check::assignments::properties`, for `$o->p = v` and every
//!   form that desugars to it (compound assignment, pre/post increment and decrement).
//! - `crate::types::checker::builtins::language_constructs`, for `unset($o->p)`.
//! - `crate::types::checker::check_types_with_options`, which applies the reservation once every
//!   body has been checked.
//!
//! Key details:
//! - php 7.4 removed shadow properties. A strict ancestor's `private $p` lives under a mangled
//!   key, and the child's by-name table does not contain it at all, so `$child->p = 1` outside
//!   the declaring class CREATES a distinct dynamic property and leaves the ancestor's slot
//!   untouched. This compiler's `ClassInfo::properties` is the PHYSICAL slot table, which still
//!   carries that slot under its plain name, so without a hash the backend's by-name ladder falls
//!   through to `resolve_property_slot_for_class` and writes the ANCESTOR'S storage. Reserving
//!   the hash is what gives the distinct dynamic property somewhere to live.
//! - The reservation is PROGRAM-USAGE gated, exactly like
//!   [`super::clone_override_storage`]'s clone destinations. A class no reachable mutation
//!   addresses that way pays nothing: the trailing pointer per instance, and the clone/free/GC
//!   traversal `#[\AllowDynamicProperties]` instances already pay, are charged only to the
//!   classes a site in THIS program can actually reach.
//! - Recording is done on the STATIC receiver class and expanded over its subclasses, because a
//!   parameter typed `Child` can hold a `GrandChild` at run time and the write must find storage
//!   on whatever class the instance really is.
//! - The flag is a STORAGE capability, never php's permission.
//!   `allow_dynamic_properties` stays exactly as the attribute left it, so
//!   `ClassInfo::dynamic_property_creation_is_deprecated()` still separates php 8.5's exempt
//!   classes from the deprecated ones: an ordinary class keeps reporting
//!   `Creation of dynamic property C::$p is deprecated`, and an
//!   `#[\AllowDynamicProperties]` class keeps reporting nothing.

use std::collections::BTreeSet;

use crate::types::PhpType;

use super::Checker;

/// One recorded mutation site, kept whole so the subclass expansion can re-ask php's question.
///
/// The class alone is not enough. Expansion has to decide, PER runtime subclass, whether php
/// would really create a dynamic property there, and that answer depends on the NAME, on the
/// SCOPE the site was written in, and on which accessor the operation consults. A subclass that
/// redeclares the name as a property of its own, or that declares the accessor, stores nothing in
/// a hash and must not be charged one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ScopeDynamicMutationSite {
    /// The receiver class the site names, before expansion.
    class_name: String,
    /// The exact property name, or `None` when the site spells the name at run time.
    property: Option<String>,
    /// The accessor php consults FIRST: `__set` for a write, `__unset` for an `unset()`.
    magic_method: String,
    /// The lexical scope the site was written in, which is what makes the name dynamic at all.
    scope: Option<String>,
}

/// The mutation sites in this program that can address a strict ancestor's private name.
#[derive(Debug, Default, Clone)]
pub struct ScopeDynamicMutationTargets {
    sites: BTreeSet<ScopeDynamicMutationSite>,
}

impl ScopeDynamicMutationTargets {
    /// Returns whether this program reaches such a mutation at all.
    fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }
}

/// Returns whether php resolves `property` on `class_name` to a DYNAMIC property in this scope
/// while the physical layout still carries a slot of that name.
///
/// The single question the mutation checks ask before they decide anything: it is what separates
/// a write php turns into a distinct dynamic property from one it refuses or sends to a slot.
pub(in crate::types::checker) fn mutation_targets_scope_dynamic_name(
    checker: &Checker,
    class_name: &str,
    property: &str,
) -> bool {
    crate::types::property_name_shadows_ancestor_private_slot(
        &checker.classes,
        class_name,
        property,
        checker.current_class.as_deref(),
    )
}

/// Records one mutation site whose name php resolves to a dynamic property on `class_name`.
///
/// `magic_method` is the accessor php consults FIRST for this mutation: `__set` for a write,
/// `__unset` for an `unset`. A class that declares it never reaches the hash at all, because php
/// hands the name to the accessor instead of creating anything, so such a class is not charged
/// the storage. This is the same precedence `magic_set_receiver_has_method` applies in lowering.
///
/// The site is recorded WHOLE and that precedence is applied later, by `site_needs_storage_on_class`
/// during the subclass expansion. Applying it only here would have charged every subclass that
/// declares the accessor its parent lacks, which is the polymorphic case `crate::ir_lower` peels
/// off with an `instanceof` guard and never lets reach a hash at all.
pub(in crate::types::checker) fn record_scope_dynamic_mutation(
    checker: &mut Checker,
    class_name: &str,
    property: &str,
    magic_method: &str,
) {
    if !mutation_targets_scope_dynamic_name(checker, class_name, property) {
        return;
    }
    record_site(
        checker,
        class_name,
        Some(property.to_string()),
        magic_method,
    );
}

/// Inserts one site, normalized, with the scope it was written in.
fn record_site(
    checker: &mut Checker,
    class_name: &str,
    property: Option<String>,
    magic_method: &str,
) {
    let site = ScopeDynamicMutationSite {
        class_name: class_name.trim_start_matches('\\').to_string(),
        property,
        magic_method: magic_method.to_string(),
        scope: checker.current_class.clone(),
    };
    checker.scope_dynamic_mutation_targets.sites.insert(site);
}

/// Records one RUNTIME-name mutation site (`$o->{$name} = v`) on a statically known class.
///
/// The name is not known here, so the question is whether the class's layout carries ANY name
/// this scope resolves to a dynamic property. If it does, the write can land on one of them, and
/// the ladder's miss arm needs somewhere to put it. This is the same test
/// `class_runtime_name_read_can_miss` applies on the read side.
pub(in crate::types::checker) fn record_scope_dynamic_runtime_name_mutation(
    checker: &mut Checker,
    class_name: &str,
) {
    let normalized = class_name.trim_start_matches('\\');
    let Some(class_info) = checker.classes.get(normalized) else {
        return;
    };
    if class_info.methods.contains_key("__set") {
        return;
    }
    let carries_scope_dynamic_name = class_info
        .properties
        .iter()
        .any(|(name, _)| {
            crate::types::property_name_shadows_ancestor_private_slot(
                &checker.classes,
                normalized,
                name,
                checker.current_class.as_deref(),
            )
        });
    if !carries_scope_dynamic_name {
        return;
    }
    let normalized = normalized.to_string();
    record_site(checker, &normalized, None, "__set");
}

/// Records one RUNTIME-name mutation site for ANY receiver shape the checker admits.
///
/// `$o->{$k} = v` accepts an `Object`, a `Union` of them and a boxed `Mixed`, and each one has its
/// own backend ladder: `lower_runtime_object_prop_set` for the first and
/// `lower_runtime_mixed_prop_set` for the other two. All three dispatch on the RUNTIME class, so
/// the reservation has to cover the same set of classes the ladder can select, or an arm addresses
/// a hash the class never reserved.
pub(in crate::types::checker) fn record_scope_dynamic_runtime_name_receiver_mutation(
    checker: &mut Checker,
    receiver_ty: &PhpType,
) {
    match receiver_ty {
        PhpType::Object(class_name) => {
            let class_name = class_name.clone();
            record_scope_dynamic_runtime_name_mutation(checker, &class_name);
        }
        PhpType::Union(members) => {
            // A union that resolves to ONE object class is that class, and degrading it to "every
            // class in the program" would charge storage to classes this site can never reach.
            // `union_single_object_class` is the same authority `unset()`'s own receiver check
            // uses, so the two agree about what a union receiver really is.
            if let Some(class_name) = checker.union_single_object_class(receiver_ty) {
                record_scope_dynamic_runtime_name_mutation(checker, &class_name);
                return;
            }
            // Otherwise the union still NAMES its object members, so each of those is recorded on
            // its own. Only a `Mixed` member means the receiver can truly be any object.
            for member in members.clone() {
                match member {
                    PhpType::Object(_) | PhpType::Union(_) => {
                        record_scope_dynamic_runtime_name_receiver_mutation(checker, &member)
                    }
                    PhpType::Mixed => {
                        record_scope_dynamic_runtime_name_mixed_receiver_mutation(checker)
                    }
                    _ => {}
                }
            }
        }
        // A boxed `Mixed` names no class, so the sound record is every class whose layout carries
        // a name this scope resolves dynamically. That set is already narrow: a class with no
        // strict ancestor's private slot in its layout contributes nothing.
        PhpType::Mixed => record_scope_dynamic_runtime_name_mixed_receiver_mutation(checker),
        _ => {}
    }
}

/// Records every class a boxed `Mixed` runtime-name mutation can land on.
///
/// A class whose layout carries no name this scope resolves dynamically is skipped outright, so
/// this is never a blanket reservation over the program: it is exactly the classes whose OWN
/// layout makes the shape possible, and expansion then filters their subclasses the same way.
fn record_scope_dynamic_runtime_name_mixed_receiver_mutation(checker: &mut Checker) {
    let scope = checker.current_class.clone();
    let targets = checker
        .classes
        .iter()
        .filter(|(class_name, class_info)| {
            !class_info.methods.contains_key("__set")
                && class_info.properties.iter().any(|(property, _)| {
                    crate::types::property_name_shadows_ancestor_private_slot(
                        &checker.classes,
                        class_name,
                        property,
                        scope.as_deref(),
                    )
                })
        })
        .map(|(class_name, _)| class_name.clone())
        .collect::<Vec<_>>();
    for class_name in targets {
        record_site(checker, &class_name, None, "__set");
    }
}

/// Records one STATIC-name mutation site whose receiver is only known to be runtime-shaped.
///
/// A boxed `Mixed` receiver names no class, so the sound record is every class whose layout
/// carries THIS name as a strict ancestor's private slot. That is already a narrow set: an
/// ordinary undeclared name matches nothing at all, and a name only some classes shadow charges
/// only those. The Mixed write ladder dispatches on the runtime class id, so each recorded class
/// gets its own arm addressing its own hash.
pub(in crate::types::checker) fn record_scope_dynamic_mixed_receiver_mutation(
    checker: &mut Checker,
    property: &str,
) {
    let scope = checker.current_class.clone();
    let targets = checker
        .classes
        .iter()
        .filter(|(class_name, class_info)| {
            !class_info.methods.contains_key("__set")
                && crate::types::property_name_shadows_ancestor_private_slot(
                    &checker.classes,
                    class_name,
                    property,
                    scope.as_deref(),
                )
        })
        .map(|(class_name, _)| class_name.clone())
        .collect::<Vec<_>>();
    for class_name in targets {
        record_site(checker, &class_name, Some(property.to_string()), "__set");
    }
}

/// Reserves the per-instance property hash on every class a recorded mutation can reach.
///
/// Runs after every body has been checked, so the recorded set is complete and the class map has
/// its final parent links for the subclass expansion.
pub(in crate::types::checker) fn reserve_scope_dynamic_property_storage(checker: &mut Checker) {
    let targets = std::mem::take(&mut checker.scope_dynamic_mutation_targets);
    if targets.is_empty() {
        return;
    }
    for class_name in target_classes(checker, &targets) {
        reserve_property_hash_storage(checker, &class_name);
    }
}

/// Expands the recorded sites into the exact set of classes to reserve storage on.
///
/// A receiver typed `Child` can hold any subclass of `Child` at run time, so every subclass is
/// CONSIDERED. It is not automatically charged: expansion re-asks php's own question on each
/// candidate, because a subclass can answer the very same name differently. A subclass that
/// declares the accessor the operation consults hands the name to it and stores nothing, and a
/// subclass that redeclares the name as a property of its own writes that slot instead of a hash.
/// Both keep their instances free of the trailing pointer and of the clone and GC traversal that
/// comes with it, which is the difference between this and a blanket reservation.
fn target_classes(checker: &Checker, targets: &ScopeDynamicMutationTargets) -> Vec<String> {
    let mut names = BTreeSet::new();
    for site in &targets.sites {
        for (candidate, _) in checker.classes.iter() {
            if candidate != &site.class_name && !checker.is_subclass_of(candidate, &site.class_name)
            {
                continue;
            }
            if site_needs_storage_on_class(checker, site, candidate) {
                names.insert(candidate.clone());
            }
        }
    }
    names.into_iter().collect()
}

/// Returns whether php would really create a dynamic property for this SITE on this CLASS.
///
/// The three answers that need no hash are all checked here, in php's own order: the accessor
/// first, because php consults it before deciding anything else; then the name, because a class
/// that resolves it to a slot of its own writes that slot. A runtime-name site has no single
/// name, so it asks whether the class carries ANY name this scope resolves dynamically, which is
/// the same question its backend ladder's miss arm asks.
fn site_needs_storage_on_class(
    checker: &Checker,
    site: &ScopeDynamicMutationSite,
    class_name: &str,
) -> bool {
    let Some(class_info) = checker.classes.get(class_name) else {
        return false;
    };
    if class_info.methods.contains_key(site.magic_method.as_str()) {
        return false;
    }
    match &site.property {
        Some(property) => crate::types::property_name_shadows_ancestor_private_slot(
            &checker.classes,
            class_name,
            property,
            site.scope.as_deref(),
        ),
        None => class_info.properties.iter().any(|(property, _)| {
            crate::types::property_name_shadows_ancestor_private_slot(
                &checker.classes,
                class_name,
                property,
                site.scope.as_deref(),
            )
        }),
    }
}

/// Sets the storage flag on one class, skipping every class that owns its physical layout.
///
/// Same exclusions as `clone_override_storage::reserve_property_hash_storage`: a checker-injected
/// builtin, a packed class and an enum all own their slot representation together with the code
/// that reads it, and stdClass already stores every property in a hash.
///
/// The builtin catalog, NOT `declared_classes`, is the authority for the first of those. The
/// driver repopulates `declared_classes` from the class map AFTER builtin injection, so that set
/// contains the injected SPL, Reflection, DateTime and throwable classes too and would exclude
/// nothing here. A user subclass of a builtin is absent from the catalog and stays eligible.
///
/// A `readonly` class is excluded for a different reason, php's own: it carries the engine's
/// no-dynamic-properties flag, so no write may ever create an entry there and the write checks
/// refuse the creation outright. Reserving a hash it can never legally fill would charge every
/// instance a trailing pointer plus clone and GC traversal for nothing.
fn reserve_property_hash_storage(checker: &mut Checker, class_name: &str) {
    if elephc_builtin_contract::lookup_class(class_name).is_some()
        || !checker.declared_classes.contains(class_name)
        || checker.packed_classes.contains_key(class_name)
        || checker.enums.contains_key(class_name)
        || super::builtin_stdclass::is_stdclass(class_name)
    {
        return;
    }
    let Some(info) = checker.classes.get_mut(class_name) else {
        return;
    };
    if info.is_readonly_class {
        return;
    }
    if info.has_property_hash_storage() {
        return;
    }
    info.scope_dynamic_property_storage = true;
}
