//! Purpose:
//! Indexes source declarations and computes their conservative fixed-point reachability.
//! Separates executable roots from dependency edges held by declaration bodies.
//!
//! Called from:
//! - `crate::optimize::reachability::prune_unreachable_declarations()`.
//!
//! Key details:
//! - Dynamic hazards widen roots only after their behaviorally reachable declaration is scanned.
//! - Method-name dispatch is deliberately conservative across all live class-like declarations.
//! - Scoped parent edges stay class-specific while preserving matching slots on the whole vtable lineage.
//! - Checker-injected interface contracts retain implementations even without source interface AST.

use std::collections::{HashMap, HashSet};

use crate::names::php_symbol_key;
use crate::parser::ast::Stmt;
use crate::types::CheckResult;

use super::usage::{self, Hazards, Usage};
use super::PruneOptions;

mod index;

/// Final declaration keep-sets produced by the fixed-point graph walk.
#[derive(Clone, Debug, Default)]
pub struct Reachability {
    pub functions: HashSet<String>,
    pub classes: HashSet<String>,
    pub methods: HashSet<(String, String, bool)>,
    pub externs: HashSet<String>,
    pub hazards: Hazards,
}

/// Indexed declarations and their deferred dependency summaries.
#[derive(Clone, Debug, Default)]
pub(crate) struct DeclarationIndex {
    pub(crate) functions: HashMap<String, Usage>,
    pub(crate) classes: HashMap<String, ClassNode>,
    pub(crate) checker_methods: HashMap<(String, String, bool), Usage>,
    pub(crate) externs: HashSet<String>,
    pub(crate) packed_classes: HashSet<String>,
    pub(crate) extern_classes: HashSet<String>,
    pub(crate) function_variants: HashMap<String, Vec<String>>,
}

/// Indexed metadata for one source class, enum, interface, or trait.
#[derive(Clone, Debug)]
pub(crate) struct ClassNode {
    pub(crate) kind: ClassKind,
    pub(crate) usage: Usage,
    pub(crate) methods: HashMap<(String, bool), Usage>,
    pub(crate) parent: Option<String>,
    pub(crate) interfaces: Vec<String>,
    pub(crate) traits: Vec<String>,
}

/// Source class-like category used for conservative interface/trait method roots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClassKind {
    Class,
    Enum,
    Interface,
    Trait,
}

/// Declarations whose bodies can execute, rather than being retained only for structural metadata.
#[derive(Clone, Debug, Default)]
struct BehavioralReachability {
    functions: HashSet<String>,
    classes: HashSet<String>,
    methods: HashSet<(String, String, bool)>,
    referenced_methods: HashSet<(String, bool)>,
    instantiated_classes: HashSet<String>,
}

/// Computes reachable declarations from executable, export, forced-prelude, and hazard roots.
pub fn compute(
    program: &[Stmt],
    check_result: &CheckResult,
    options: &PruneOptions<'_>,
) -> Reachability {
    let call_signatures = usage::CallSignatureIndex::from_check_result(check_result);
    let index = DeclarationIndex::build_with_signatures(
        program,
        check_result,
        &call_signatures,
    );
    let static_class_alias_targets = index.class_alias_targets();
    let executable_usage = usage::scan_executable_program(program, &call_signatures);
    let mut state = GraphState::new(
        index,
        executable_usage.hazards,
        options.inventory.internal_callable_methods(),
        check_result,
    );
    state.apply_usage(executable_usage, true);
    for target in static_class_alias_targets {
        state.keep_class_alias_target_methods(&target, true);
    }
    for name in options.exported_functions {
        let name = php_symbol_key(name);
        state.reach.functions.insert(name.clone());
        state.behavioral.functions.insert(name);
    }
    for group in options
        .inventory
        .groups
        .values()
        .filter(|group| options.forced_groups.contains(&group.id))
    {
        state.reach.functions.extend(group.functions.iter().cloned());
        state.reach.classes.extend(group.classes.iter().cloned());
        state.reach.methods.extend(group.methods.iter().cloned());
        state.reach.externs.extend(group.externs.iter().cloned());
        state.behavioral.functions.extend(group.functions.iter().cloned());
        state.behavioral.classes.extend(group.classes.iter().cloned());
        state.behavioral.methods.extend(group.methods.iter().cloned());
    }
    if options.eval_forced {
        state.keep_everything();
    }
    state.apply_global_hazards();
    state.fixed_point();
    state.reach
}

struct GraphState {
    index: DeclarationIndex,
    reach: Reachability,
    behavioral: BehavioralReachability,
    structural_referenced_methods: HashSet<(String, bool)>,
    scoped_methods: HashSet<(String, String, bool)>,
    behavioral_scoped_methods: HashSet<(String, String, bool)>,
    instantiated_classes: HashSet<String>,
    scanned_functions: HashSet<String>,
    behaviorally_scanned_functions: HashSet<String>,
    scanned_classes: HashSet<String>,
    behaviorally_scanned_classes: HashSet<String>,
    scanned_methods: HashSet<(String, String, bool)>,
    behaviorally_scanned_methods: HashSet<(String, String, bool)>,
    opaque_variables: HashSet<String>,
    all_variables_opaque: bool,
    behavioral_variable_methods: HashMap<String, HashSet<(String, bool)>>,
    internal_callable_methods: HashSet<(String, String, bool)>,
    checker_interface_methods: HashMap<String, HashSet<(String, bool)>>,
    checker_method_implementations: HashMap<(String, String, bool), String>,
    vtable_slots: HashSet<(String, String, bool)>,
}

impl GraphState {
    /// Creates graph state with hazards found in executable roots.
    fn new(
        index: DeclarationIndex,
        hazards: Hazards,
        internal_callable_methods: HashSet<(String, String, bool)>,
        check_result: &CheckResult,
    ) -> Self {
        let checker_interface_methods = check_result
            .interfaces
            .iter()
            .map(|(name, info)| {
                let methods = info
                    .methods
                    .keys()
                    .map(|method| (method.clone(), false))
                    .chain(
                        info.static_methods
                            .keys()
                            .map(|method| (method.clone(), true)),
                    )
                    .collect();
                (php_symbol_key(name), methods)
            })
            .collect();
        let checker_method_implementations = check_result
            .classes
            .iter()
            .flat_map(|(class, info)| {
                let class = php_symbol_key(class);
                info.method_impl_classes
                    .iter()
                    .map({
                        let class = class.clone();
                        move |(method, owner)| {
                            (
                                (class.clone(), php_symbol_key(method), false),
                                php_symbol_key(owner),
                            )
                        }
                    })
                    .chain(info.static_method_impl_classes.iter().map({
                        let class = class.clone();
                        move |(method, owner)| {
                            (
                                (class.clone(), php_symbol_key(method), true),
                                php_symbol_key(owner),
                            )
                        }
                    }))
            })
            .collect();
        let vtable_slots = check_result
            .classes
            .iter()
            .flat_map(|(class, info)| {
                let class = php_symbol_key(class);
                info.vtable_methods
                    .iter()
                    .map({
                        let class = class.clone();
                        move |method| (class.clone(), php_symbol_key(method), false)
                    })
                    .chain(info.static_vtable_methods.iter().map({
                        let class = class.clone();
                        move |method| (class.clone(), php_symbol_key(method), true)
                    }))
            })
            .collect();
        Self {
            index,
            reach: Reachability {
                hazards,
                ..Reachability::default()
            },
            behavioral: BehavioralReachability::default(),
            structural_referenced_methods: HashSet::new(),
            scoped_methods: HashSet::new(),
            behavioral_scoped_methods: HashSet::new(),
            instantiated_classes: HashSet::new(),
            scanned_functions: HashSet::new(),
            behaviorally_scanned_functions: HashSet::new(),
            scanned_classes: HashSet::new(),
            behaviorally_scanned_classes: HashSet::new(),
            scanned_methods: HashSet::new(),
            behaviorally_scanned_methods: HashSet::new(),
            opaque_variables: HashSet::new(),
            all_variables_opaque: false,
            behavioral_variable_methods: HashMap::new(),
            internal_callable_methods,
            checker_interface_methods,
            checker_method_implementations,
            vtable_slots,
        }
    }

    /// Marks every indexed declaration reachable for explicit eval force-keep.
    fn keep_everything(&mut self) {
        self.reach.functions.extend(self.index.functions.keys().cloned());
        self.reach.classes.extend(self.index.classes.keys().cloned());
        self.reach.classes.extend(self.index.packed_classes.iter().cloned());
        self.reach.classes.extend(self.index.extern_classes.iter().cloned());
        self.reach.externs.extend(self.index.externs.iter().cloned());
        self.behavioral
            .functions
            .extend(self.index.functions.keys().cloned());
        self.behavioral
            .classes
            .extend(self.index.classes.keys().cloned());
        for (class, node) in &self.index.classes {
            for (method, is_static) in node.methods.keys() {
                let method = (class.clone(), method.clone(), *is_static);
                self.reach.methods.insert(method.clone());
                self.behavioral.methods.insert(method);
            }
        }
    }

    /// Applies dynamic hazard widening accumulated from executable and reachable bodies.
    fn apply_global_hazards(&mut self) {
        if self.reach.hazards.dynamic_function {
            self.reach.functions.extend(self.index.functions.keys().cloned());
            self.reach.externs.extend(self.index.externs.iter().cloned());
            self.behavioral
                .functions
                .extend(self.index.functions.keys().cloned());
        }
        if self.reach.hazards.dynamic_class {
            self.reach.classes.extend(self.index.classes.keys().cloned());
            self.reach.classes.extend(self.index.packed_classes.iter().cloned());
            self.reach.classes.extend(self.index.extern_classes.iter().cloned());
            self.instantiated_classes
                .extend(self.index.classes.keys().cloned());
            self.behavioral
                .classes
                .extend(self.index.classes.keys().cloned());
            self.behavioral
                .instantiated_classes
                .extend(self.index.classes.keys().cloned());
        }
    }

    /// Repeats declaration-body scans until no keep-set grows.
    fn fixed_point(&mut self) {
        loop {
            let before = self.size();
            self.expand_function_variants();
            self.scan_new_functions();
            self.scan_new_classes();
            self.seed_live_methods();
            self.scan_new_methods();
            if self.size() == before {
                break;
            }
        }
    }

    /// Expands reachable public function variant groups to their concrete declarations.
    fn expand_function_variants(&mut self) {
        for (group, variants) in &self.index.function_variants {
            if self.reach.functions.contains(group) {
                self.reach.functions.extend(variants.iter().cloned());
                if self.behavioral.functions.contains(group) {
                    self.behavioral.functions.extend(variants.iter().cloned());
                }
            }
        }
    }

    /// Scans new free functions and rescans structural survivors upgraded to behavioral roots.
    fn scan_new_functions(&mut self) {
        let names: Vec<_> = self
            .reach
            .functions
            .iter()
            .filter(|name| {
                !self.scanned_functions.contains(*name)
                    || (self.behavioral.functions.contains(*name)
                        && !self.behaviorally_scanned_functions.contains(*name))
            })
            .cloned()
            .collect();
        for name in names {
            let behavioral = self.behavioral.functions.contains(&name);
            self.scanned_functions.insert(name.clone());
            if behavioral {
                self.behaviorally_scanned_functions.insert(name.clone());
            }
            if let Some(usage) = self.index.functions.get(&name).cloned() {
                self.apply_usage(usage, behavioral);
            }
        }
    }

    /// Scans class shells and rescans structural survivors upgraded to behavioral roots.
    fn scan_new_classes(&mut self) {
        let names: Vec<_> = self
            .reach
            .classes
            .iter()
            .filter(|name| {
                !self.scanned_classes.contains(*name)
                    || (self.behavioral.classes.contains(*name)
                        && !self.behaviorally_scanned_classes.contains(*name))
            })
            .cloned()
            .collect();
        for name in names {
            let behavioral = self.behavioral.classes.contains(&name);
            self.scanned_classes.insert(name.clone());
            if behavioral {
                self.behaviorally_scanned_classes.insert(name.clone());
            }
            let Some(node) = self.index.classes.get(&name).cloned() else {
                continue;
            };
            self.apply_usage(node.usage.clone(), behavioral);
            if let Some(parent) = &node.parent {
                self.reach.classes.insert(parent.clone());
                if behavioral {
                    self.behavioral.classes.insert(parent.clone());
                }
            }
            self.reach.classes.extend(node.interfaces.iter().cloned());
            self.reach.classes.extend(node.traits.iter().cloned());
            if matches!(node.kind, ClassKind::Interface) {
                self.reference_all_declared_methods(&node);
            }
            for interface in &node.interfaces {
                if let Some(contract) = self.index.classes.get(interface).cloned() {
                    self.reference_all_declared_methods(&contract);
                } else if let Some(methods) = self.checker_interface_methods.get(interface) {
                    self.structural_referenced_methods
                        .extend(methods.iter().cloned());
                }
            }
        }
    }

    /// Seeds methods on live classes from direct names, hazards, magic hooks, and contracts.
    fn seed_live_methods(&mut self) {
        self.seed_instantiated_magic_methods();
        let live_classes: Vec<_> = self.reach.classes.iter().cloned().collect();
        for class in live_classes {
            let Some(node) = self.index.classes.get(&class) else {
                continue;
            };
            let has_runtime_owned_parent = node
                .parent
                .as_ref()
                .is_some_and(|parent| !self.index.classes.contains_key(parent));
            for (method, is_static) in node.methods.keys() {
                let key = (class.clone(), method.clone(), *is_static);
                let reference = (method.clone(), *is_static);
                if self.reach.hazards.dynamic_method
                    || has_runtime_owned_parent
                    || matches!(method.as_str(), "__call" | "__callstatic")
                    || self.behavioral.referenced_methods.contains(&reference)
                {
                    self.reach.methods.insert(key.clone());
                    self.behavioral.methods.insert(key);
                } else if self.structural_referenced_methods.contains(&reference) {
                    self.reach.methods.insert(key);
                }
            }
        }
        self.seed_vtable_slot_families();
        self.seed_explicit_method_implementations();
        self.seed_inherited_implementations();
    }

    /// Keeps shared virtual slots on every live class in a lineage once any occupant survives.
    ///
    /// The inner test used to be `class_is_or_descends_from(candidate, lineage_root)` run for
    /// EVERY (kept method, live class) pair — O(kept_methods x live_classes x depth) — and each
    /// call allocated a fresh `String` per parent hop plus a `HashSet` for its cycle guard. That
    /// product is the prune phase's dominant cost on a large program, and nothing in the
    /// predicate depends on the method: it is a pure function of the declaration index's parent
    /// chains. So the chains are walked ONCE per fixed-point round here and inverted into
    /// `lineage root -> live classes that descend from it`.
    ///
    /// The retained set cannot change:
    /// * the inversion and the predicate share ONE walk (`walk_class_ancestors`), so they cut the
    ///   chain at the same first repeated name and compare a name before looking its node up —
    ///   "root is among the names visited from candidate" is exactly what the predicate answered;
    /// * `live_classes` is deduplicated (it comes from a `HashSet`) and a name is visited at most
    ///   once per chain, so each root lists each candidate at most once;
    /// * the lists are built in ascending `live_classes` position, so the inner loop still visits
    ///   candidates in the same order it did, inserting the same pairs;
    /// * every other test in the loop (`has_vtable_slot`, the behavioral lookup) is untouched.
    fn seed_vtable_slot_families(&mut self) {
        let live_classes: Vec<_> = self.reach.classes.iter().cloned().collect();
        let kept_methods: Vec<_> = self.reach.methods.iter().cloned().collect();
        let descendants_by_root = live_descendants_by_root(&self.index, &live_classes);
        for (class, method, is_static) in kept_methods {
            if !self.has_vtable_slot(&class, &method, is_static) {
                continue;
            }
            let lineage_root = self.vtable_lineage_root(&class, &method, is_static);
            let behavioral = self.behavioral.methods.contains(&(
                class.clone(),
                method.clone(),
                is_static,
            ));
            let Some(descendants) = descendants_by_root.get(&lineage_root) else {
                continue;
            };
            for position in descendants {
                let candidate = &live_classes[*position];
                let visible_method = (candidate.clone(), method.clone(), is_static);
                if !self.has_vtable_slot(candidate, &method, is_static) {
                    continue;
                }
                self.reach.methods.insert(visible_method.clone());
                if behavioral {
                    self.behavioral.methods.insert(visible_method);
                }
            }
        }
    }

    /// Resolves class-qualified method edges to the checker-selected implementation body.
    fn seed_explicit_method_implementations(&mut self) {
        let methods: Vec<_> = self.reach.methods.iter().cloned().collect();
        for (class, method, is_static) in methods {
            let visible_method = (class.clone(), method.clone(), is_static);
            let behavioral = self.behavioral.methods.contains(&visible_method);
            let owner = self
                .checker_method_implementations
                .get(&visible_method)
                .cloned()
                .or_else(|| {
                    let mut current = Some(class.clone());
                    let mut seen = HashSet::new();
                    while let Some(candidate) = current {
                        if !seen.insert(candidate.clone()) {
                            return None;
                        }
                        let node = self.index.classes.get(&candidate)?;
                        if node.methods.contains_key(&(method.clone(), is_static)) {
                            return Some(candidate);
                        }
                        current = node.parent.clone();
                    }
                    None
                });
            let Some(owner) = owner else {
                continue;
            };
            let owner_method = (owner, method.clone(), is_static);
            self.reach.methods.insert(owner_method.clone());
            if behavioral {
                self.behavioral.methods.insert(owner_method);
            }
        }
    }

    /// Keeps the first concrete implementation of each implicit magic hook on instantiated classes.
    fn seed_instantiated_magic_methods(&mut self) {
        let instantiated: Vec<_> = self.instantiated_classes.iter().cloned().collect();
        for class in instantiated {
            let behavioral = self.behavioral.instantiated_classes.contains(&class);
            let checker_magic: Vec<_> = self
                .checker_method_implementations
                .iter()
                .filter_map(|((visible_class, method, is_static), owner)| {
                    (visible_class == &class && is_magic_method(method)).then(|| {
                        (
                            (visible_class.clone(), method.clone(), *is_static),
                            (owner.clone(), method.clone(), *is_static),
                        )
                    })
                })
                .collect();
            for (visible_method, owner_method) in checker_magic {
                self.reach.methods.insert(visible_method.clone());
                self.reach.methods.insert(owner_method.clone());
                if behavioral {
                    self.behavioral.methods.insert(visible_method);
                    self.behavioral.methods.insert(owner_method);
                }
            }
            let mut current = Some(class.clone());
            let mut seen_classes = HashSet::new();
            let mut found_methods = HashSet::new();
            while let Some(owner) = current {
                if !seen_classes.insert(owner.clone()) {
                    break;
                }
                let Some(node) = self.index.classes.get(&owner) else {
                    break;
                };
                for (method, is_static) in node.methods.keys() {
                    if is_magic_method(method) && found_methods.insert(method.clone()) {
                        let owner_method = (owner.clone(), method.clone(), *is_static);
                        let visible_method = (class.clone(), method.clone(), *is_static);
                        self.reach.methods.insert(owner_method.clone());
                        self.reach.methods.insert(visible_method.clone());
                        if behavioral {
                            self.behavioral.methods.insert(owner_method);
                            self.behavioral.methods.insert(visible_method);
                        }
                    }
                }
                current = node.parent.clone();
            }
        }
    }

    /// Keeps parent implementations and descendant vtable entries for every referenced method.
    fn seed_inherited_implementations(&mut self) {
        let live_classes: Vec<_> = self.reach.classes.iter().cloned().collect();
        let referenced: Vec<_> = self
            .structural_referenced_methods
            .iter()
            .cloned()
            .map(|method| (method, false))
            .chain(
                self.behavioral
                    .referenced_methods
                    .iter()
                    .cloned()
                    .map(|method| (method, true)),
            )
            .collect();
        for class in live_classes {
            for ((method, is_static), behavioral) in &referenced {
                let visible_method = (class.clone(), method.clone(), *is_static);
                if let Some(owner) = self
                    .checker_method_implementations
                    .get(&visible_method)
                    .cloned()
                {
                    let owner_method = (owner, method.clone(), *is_static);
                    self.reach.methods.insert(owner_method.clone());
                    self.reach.methods.insert(visible_method.clone());
                    if *behavioral {
                        self.behavioral.methods.insert(owner_method);
                        self.behavioral.methods.insert(visible_method);
                    }
                    continue;
                }
                let mut current = Some(class.clone());
                let mut seen = HashSet::new();
                while let Some(owner) = current {
                    if !seen.insert(owner.clone()) {
                        break;
                    }
                    let Some(node) = self.index.classes.get(&owner) else {
                        break;
                    };
                    if node.methods.contains_key(&(method.clone(), *is_static)) {
                        let owner_method = (owner.clone(), method.clone(), *is_static);
                        let visible_method = (class.clone(), method.clone(), *is_static);
                        self.reach.methods.insert(owner_method.clone());
                        self.reach.methods.insert(visible_method.clone());
                        if *behavioral {
                            self.behavioral.methods.insert(owner_method);
                            self.behavioral.methods.insert(visible_method);
                        }
                        break;
                    }
                    current = node.parent.clone();
                }
            }
        }
    }

    /// Scans new methods and rescans structural survivors upgraded to behavioral roots.
    fn scan_new_methods(&mut self) {
        let methods: Vec<_> = self
            .reach
            .methods
            .iter()
            .filter(|method| {
                !self.scanned_methods.contains(*method)
                    || (self.behavioral.methods.contains(*method)
                        && !self.behaviorally_scanned_methods.contains(*method))
            })
            .cloned()
            .collect();
        for (class, method, is_static) in methods {
            let key = (class.clone(), method.clone(), is_static);
            let behavioral = self.behavioral.methods.contains(&key);
            self.scanned_methods.insert(key.clone());
            if behavioral {
                self.behaviorally_scanned_methods.insert(key.clone());
            }
            let mut usage = self
                .index
                .classes
                .get(&class)
                .and_then(|node| node.methods.get(&(method.clone(), is_static)))
                .cloned()
                .or_else(|| {
                    self.index
                        .checker_methods
                        .get(&(class.clone(), method.clone(), is_static))
                        .cloned()
                });
            if let Some(usage) = usage.as_mut() {
                if behavioral && self.internal_callable_methods.contains(&key) {
                    usage.hazards.dynamic_function = false;
                    usage.hazards.dynamic_method = false;
                }
            }
            if let Some(usage) = usage {
                self.apply_usage(usage, behavioral);
            }
        }
    }

    /// Adds interface methods as structural roots until an executable edge reaches them.
    fn reference_all_declared_methods(&mut self, node: &ClassNode) {
        self.structural_referenced_methods
            .extend(node.methods.keys().cloned());
    }

    /// Applies one usage summary, propagating hazards only from behaviorally reachable bodies.
    fn apply_usage(&mut self, usage: Usage, behavioral: bool) {
        for target in &usage.class_alias_targets {
            self.keep_class_alias_target_methods(target, behavioral);
        }
        for root in &usage.instantiated_subclass_roots {
            self.keep_instantiable_subclasses(root, behavioral);
        }
        if behavioral {
            self.reach.hazards.dynamic_function |= usage.hazards.dynamic_function;
            self.reach.hazards.dynamic_method |= usage.hazards.dynamic_method;
            self.reach.hazards.dynamic_class |= usage.hazards.dynamic_class;
            self.behavioral
                .functions
                .extend(usage.functions.iter().cloned());
            self.behavioral
                .classes
                .extend(usage.classes.iter().cloned());
            self.behavioral
                .instantiated_classes
                .extend(usage.instantiated_classes.iter().cloned());
            self.opaque_variables
                .extend(usage.global_aliases.iter().cloned());
            self.all_variables_opaque |= usage.dynamic_global_alias;
            for (variable, methods) in &usage.variable_methods {
                self.behavioral_variable_methods
                    .entry(variable.clone())
                    .or_default()
                    .extend(methods.iter().cloned());
            }
            self.promote_opaque_variable_methods();
        }
        self.reach.functions.extend(usage.functions);
        self.reach.classes.extend(usage.classes);
        self.reach.externs.extend(usage.externs);
        self.instantiated_classes
            .extend(usage.instantiated_classes.iter().cloned());
        for (class, method, is_static) in usage.methods {
            self.reach.classes.insert(class.clone());
            let key = (class.clone(), method.clone(), is_static);
            self.reach.methods.insert(key.clone());
            if behavioral {
                self.behavioral.classes.insert(class);
                self.behavioral.methods.insert(key);
                self.behavioral
                    .referenced_methods
                    .insert((method, is_static));
            } else {
                self.structural_referenced_methods
                    .insert((method, is_static));
            }
        }
        for (class, method, is_static) in usage.scoped_methods {
            self.reach.classes.insert(class.clone());
            let key = (class.clone(), method, is_static);
            self.reach.methods.insert(key.clone());
            self.scoped_methods.insert(key.clone());
            if behavioral {
                self.behavioral.classes.insert(class);
                self.behavioral.methods.insert(key.clone());
                self.behavioral_scoped_methods.insert(key);
            }
        }
        if behavioral {
            self.behavioral
                .referenced_methods
                .extend(usage.wildcard_methods);
        } else {
            self.structural_referenced_methods
                .extend(usage.wildcard_methods);
        }
        self.apply_global_hazards();
    }

    /// Retains the complete callable hierarchy exposed through one PHP class alias.
    ///
    /// A class alias is an alternate class name, not a newly restricted subtype. Opaque runtime
    /// code can therefore invoke any inherited instance or static method through the alias, even
    /// when no statically visible call names that member. Keep the target hierarchy narrowly here
    /// instead of promoting the global dynamic-method hazard and retaining unrelated classes.
    fn keep_class_alias_target_methods(&mut self, target: &str, behavioral: bool) {
        let mut current = php_symbol_key(target);
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(current.clone()) {
                return;
            }
            let Some(node) = self.index.classes.get(&current) else {
                return;
            };
            let methods = node.methods.keys().cloned().collect::<Vec<_>>();
            let parent = node.parent.clone();
            self.reach.classes.insert(current.clone());
            if behavioral {
                self.behavioral.classes.insert(current.clone());
            }
            for (method, is_static) in methods {
                let method = (current.clone(), method, is_static);
                self.reach.methods.insert(method.clone());
                if behavioral {
                    self.behavioral.methods.insert(method);
                }
            }
            let Some(parent) = parent else {
                return;
            };
            current = parent;
        }
    }

    /// Retains every indexed class that is equal to or descends from one runtime-selected base.
    fn keep_instantiable_subclasses(&mut self, root: &str, behavioral: bool) {
        let root = php_symbol_key(root);
        let classes: Vec<_> = self
            .index
            .classes
            .keys()
            .filter(|class| self.class_is_or_descends_from(class, &root))
            .cloned()
            .collect();
        self.reach.classes.extend(classes.iter().cloned());
        self.instantiated_classes.extend(classes.iter().cloned());
        if behavioral {
            self.behavioral.classes.extend(classes.iter().cloned());
            self.behavioral.instantiated_classes.extend(classes);
        }
    }

    /// Returns whether the checker assigned a virtual slot for this visible method.
    fn has_vtable_slot(&self, class: &str, method: &str, is_static: bool) -> bool {
        self.vtable_slots
            .contains(&(class.to_string(), method.to_string(), is_static))
    }

    /// Returns the oldest ancestor that still occupies the same virtual slot.
    fn vtable_lineage_root(&self, class: &str, method: &str, is_static: bool) -> String {
        let mut current = class.to_string();
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(current.clone()) {
                return current;
            }
            let Some(parent) = self
                .index
                .classes
                .get(&current)
                .and_then(|node| node.parent.clone())
            else {
                return current;
            };
            if !self.has_vtable_slot(&parent, method, is_static) {
                return current;
            }
            current = parent;
        }
    }

    /// Returns whether one indexed class is the named root or inherits from it.
    ///
    /// Delegates to `walk_class_ancestors` so this predicate and the `root -> descendants`
    /// inversion `seed_vtable_slot_families` builds can never disagree about what a chain is.
    fn class_is_or_descends_from(&self, class: &str, root: &str) -> bool {
        let mut found = false;
        walk_class_ancestors(&self.index, class, |candidate| {
            if candidate == root {
                found = true;
                return false;
            }
            true
        });
        found
    }

    /// Turns method calls on interprocedurally aliased variable names into wildcard edges.
    fn promote_opaque_variable_methods(&mut self) {
        if self.all_variables_opaque {
            self.behavioral.referenced_methods.extend(
                self.behavioral_variable_methods
                    .values()
                    .flatten()
                    .cloned(),
            );
            return;
        }
        for variable in &self.opaque_variables {
            if let Some(methods) = self.behavioral_variable_methods.get(variable) {
                self.behavioral
                    .referenced_methods
                    .extend(methods.iter().cloned());
            }
        }
    }

    /// Returns the total cardinality used to detect fixed-point convergence.
    fn size(&self) -> usize {
        self.reach.functions.len()
            + self.reach.classes.len()
            + self.reach.methods.len()
            + self.reach.externs.len()
            + self.behavioral.functions.len()
            + self.behavioral.classes.len()
            + self.behavioral.methods.len()
            + self.behavioral.referenced_methods.len()
            + self.behavioral.instantiated_classes.len()
            + self.structural_referenced_methods.len()
            + self.scoped_methods.len()
            + self.behavioral_scoped_methods.len()
            + self.instantiated_classes.len()
            + self.opaque_variables.len()
            + usize::from(self.all_variables_opaque)
            + self
                .behavioral_variable_methods
                .values()
                .map(HashSet::len)
                .sum::<usize>()
            + usize::from(self.reach.hazards.dynamic_function)
            + usize::from(self.reach.hazards.dynamic_method)
            + usize::from(self.reach.hazards.dynamic_class)
    }
}

/// Returns whether an instantiated class must retain the declared PHP magic method.
fn is_magic_method(method: &str) -> bool {
    matches!(
        method,
        "__construct"
            | "__destruct"
            | "__clone"
            | "__tostring"
            | "__get"
            | "__set"
            | "__isset"
            | "__unset"
            | "__invoke"
            | "__serialize"
            | "__unserialize"
            | "__sleep"
            | "__wakeup"
            | "__debuginfo"
            | "__set_state"
            | "__call"
            | "__callstatic"
    )
}

/// Walks one class's ancestor chain and hands every name it reaches to `visit`, stopping when
/// `visit` returns `false` or when a name repeats.
///
/// This IS the walk `class_is_or_descends_from` performed, factored out so the single-root
/// predicate and the root-to-descendants inversion cannot drift apart: the cycle guard cuts the
/// chain at the first repeated name (a class in an `extends` cycle therefore descends only from
/// the names before the repeat), and a name is handed to `visit` BEFORE its node is looked up, so
/// a parent with no indexed node is still part of the chain and is its own lineage root.
fn walk_class_ancestors(
    index: &DeclarationIndex,
    class: &str,
    mut visit: impl FnMut(&str) -> bool,
) {
    let mut current = Some(class.to_string());
    let mut seen = HashSet::new();
    while let Some(candidate) = current {
        if !seen.insert(candidate.clone()) {
            return;
        }
        if !visit(&candidate) {
            return;
        }
        current = index
            .classes
            .get(&candidate)
            .and_then(|node| node.parent.clone());
    }
}

/// Inverts the live classes' ancestor chains into `root -> positions in live_classes`.
///
/// `descendants[root]` holds the position of every live class for which
/// `class_is_or_descends_from(class, root)` answers `true`, and no other: both come from
/// `walk_class_ancestors`. Positions are pushed in ascending order, so iterating one root's list
/// visits candidates in the same order a scan of `live_classes` did.
fn live_descendants_by_root(
    index: &DeclarationIndex,
    live_classes: &[String],
) -> HashMap<String, Vec<usize>> {
    let mut descendants: HashMap<String, Vec<usize>> = HashMap::new();
    for (position, class) in live_classes.iter().enumerate() {
        walk_class_ancestors(index, class, |candidate| {
            descendants
                .entry(candidate.to_string())
                .or_default()
                .push(position);
            true
        });
    }
    descendants
}

#[cfg(test)]
mod vtable_lineage_index_tests {
    use super::*;

    /// The pre-patch predicate, copied verbatim, so the index below is compared against the code
    /// it replaced rather than against a refactoring of itself.
    fn reference_class_is_or_descends_from(
        index: &DeclarationIndex,
        class: &str,
        root: &str,
    ) -> bool {
        let mut current = Some(class.to_string());
        let mut seen = HashSet::new();
        while let Some(candidate) = current {
            if !seen.insert(candidate.clone()) {
                return false;
            }
            if candidate == root {
                return true;
            }
            current = index
                .classes
                .get(&candidate)
                .and_then(|node| node.parent.clone());
        }
        false
    }

    /// Builds one indexed class node.
    fn node(
        kind: ClassKind,
        parent: Option<&str>,
        interfaces: &[&str],
        traits: &[&str],
    ) -> ClassNode {
        ClassNode {
            kind,
            usage: Usage::default(),
            methods: HashMap::new(),
            parent: parent.map(str::to_string),
            interfaces: interfaces.iter().map(|name| (*name).to_string()).collect(),
            traits: traits.iter().map(|name| (*name).to_string()).collect(),
        }
    }

    /// A hierarchy with an interface, a trait, an abstract-shaped parent, a class whose parent is
    /// not indexed at all, and an `extends` cycle.
    fn hierarchy() -> DeclarationIndex {
        let mut index = DeclarationIndex::default();
        for (name, class_node) in [
            ("contract", node(ClassKind::Interface, None, &[], &[])),
            ("helper", node(ClassKind::Trait, None, &[], &[])),
            (
                "base",
                node(ClassKind::Class, None, &["contract"], &["helper"]),
            ),
            (
                "middle",
                node(ClassKind::Class, Some("base"), &[], &["helper"]),
            ),
            ("leaf", node(ClassKind::Class, Some("middle"), &[], &[])),
            ("sibling", node(ClassKind::Class, Some("base"), &[], &[])),
            (
                "orphan",
                node(ClassKind::Class, Some("runtime_owned"), &[], &[]),
            ),
            ("cycle_a", node(ClassKind::Class, Some("cycle_b"), &[], &[])),
            ("cycle_b", node(ClassKind::Class, Some("cycle_a"), &[], &[])),
        ] {
            index.classes.insert(name.to_string(), class_node);
        }
        index
    }

    /// The live-class list the seeding loop would scan, plus names that only ever appear as
    /// parents, so roots outside the list are exercised too.
    fn live_classes() -> Vec<String> {
        [
            "contract",
            "helper",
            "base",
            "middle",
            "leaf",
            "sibling",
            "orphan",
            "runtime_owned",
            "cycle_a",
            "cycle_b",
        ]
        .iter()
        .map(|name| (*name).to_string())
        .collect()
    }

    /// The inverted index answers, for every root, exactly the set the chain predicate answers.
    #[test]
    fn descendant_index_agrees_with_the_chain_predicate() {
        let index = hierarchy();
        let live = live_classes();
        let descendants = live_descendants_by_root(&index, &live);
        let mut roots = live.clone();
        roots.push("never_declared".to_string());
        for root in roots {
            let from_index: HashSet<usize> = descendants
                .get(&root)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect();
            let from_predicate: HashSet<usize> = live
                .iter()
                .enumerate()
                .filter(|(_, class)| reference_class_is_or_descends_from(&index, class, &root))
                .map(|(position, _)| position)
                .collect();
            assert_eq!(from_index, from_predicate, "root {root}");
        }
    }

    /// Each root's list is in ascending `live_classes` position and free of duplicates, which is
    /// what keeps the seeding loop's visit order identical to the scan it replaced.
    #[test]
    fn descendant_lists_preserve_scan_order() {
        let index = hierarchy();
        for (root, positions) in live_descendants_by_root(&index, &live_classes()) {
            let mut sorted = positions.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(
                positions, sorted,
                "root {root} must list positions in scan order"
            );
        }
    }

    /// Interfaces and traits are their own roots: `base` implements `contract` and uses `helper`,
    /// but neither is on its `extends` chain, so neither lists `base` as a descendant. This pins
    /// the relation the inversion inherits from the predicate instead of inventing a new one.
    #[test]
    fn interfaces_and_traits_are_not_extends_roots() {
        let index = hierarchy();
        let live = live_classes();
        let descendants = live_descendants_by_root(&index, &live);
        let at = |name: &str| live.iter().position(|entry| entry == name).unwrap();
        assert_eq!(descendants["contract"], vec![at("contract")]);
        assert_eq!(descendants["helper"], vec![at("helper")]);
        assert_eq!(
            descendants["base"],
            vec![at("base"), at("middle"), at("leaf"), at("sibling")],
        );
        assert_eq!(descendants["middle"], vec![at("middle"), at("leaf")]);
        // A parent with no indexed node is still a root, and is its own descendant when live.
        assert_eq!(
            descendants["runtime_owned"],
            vec![at("orphan"), at("runtime_owned")],
        );
        // An `extends` cycle stops at the first repeat instead of looping.
        assert_eq!(descendants["cycle_a"], vec![at("cycle_a"), at("cycle_b")]);
        assert_eq!(descendants["cycle_b"], vec![at("cycle_a"), at("cycle_b")]);
        // A name no live class reaches has no entry at all.
        assert!(!descendants.contains_key("never_declared"));
    }
}
