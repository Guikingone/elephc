//! Purpose:
//! Stage-2 reachability fixpoint: from the program's top-level (entry) statements, compute the
//! set of functions, methods, and instantiated classes that can actually run, plus a list of
//! human-readable "fallback" notes recording every place precision was traded for soundness.
//!
//! Called from:
//! - `crate::tree_shake::compute_reachable` (re-exported), invoked behind `--tree-shake` in the
//!   pipeline. Stage 2 only *dumps* the result; it does not yet prune the checker or ir_lower.
//!
//! Key details:
//! - The algorithm is Rapid Type Analysis (RTA): a virtual/unknown-receiver call can only reach
//!   an object that was actually constructed, so dispatch is resolved against the growing
//!   `instantiated` set. This is sound (never under-approximates) and much tighter than resolving
//!   over the whole class hierarchy. `new $dynamic` conservatively instantiates every instantiable
//!   class, collapsing RTA to hierarchy analysis for that method name — recorded as a fallback.
//! - Soundness over precision: when a receiver's class set cannot be cheaply, locally determined
//!   the call becomes an `Any` fallback keyed only by method name, applied to every instantiated
//!   class that has that method. A dynamic method name keeps *all* methods of instantiated classes.
//! - The fixpoint is monotonic (sets only grow over a finite closed world) so it always terminates.

use std::collections::{HashSet, VecDeque};

use crate::parser::ast::{Program, TypeExpr};

use super::index::{build_index, AstIndex};
use super::skeleton::Skeleton;

/// The reachable slice of the program: which functions/methods can run and which classes are
/// constructed, plus diagnostics describing every soundness fallback that was triggered.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reachable {
    /// Canonical FQNs of every function that can be called from reachable code.
    pub functions: HashSet<String>,
    /// `(declaring-class FQN, lowercase method name)` for every method that can run.
    pub methods: HashSet<(String, String)>,
    /// Canonical FQNs of every class that is constructed (`new`) from reachable code.
    pub instantiated: HashSet<String>,
    /// One human-readable note per distinct soundness fallback, for diagnostics/dumps.
    pub fallback_sites: Vec<String>,
}

/// The class set a method-call receiver can hold, as determined by cheap local syntax.
///
/// `Any` means the receiver type could not be pinned down, so dispatch must fall back to every
/// instantiated class exposing the method. `Bound` names a closed set of candidate classes
/// (from a type hint, `$this`, an inline `new`, or a typed property).
pub(super) enum Receiver {
    /// Unknown receiver: fall back over instantiated classes by method name.
    Any,
    /// A closed candidate set, tagged with a stable `key` used only to de-duplicate call sites.
    Bound { key: String, classes: HashSet<String> },
}

/// A method call whose receiver was pinned to a closed candidate set. Stored so the RTA fixpoint
/// can re-resolve it whenever a matching class is later instantiated.
struct TypedSite {
    /// Candidate receiver classes (already expanded to their subtypes).
    classes: HashSet<String>,
    /// Lowercase name of the invoked method.
    method: String,
}

/// The mutable state of the reachability fixpoint over one program.
pub(super) struct Analyzer<'a> {
    /// Structural class world (hierarchy, method existence, instantiability queries).
    pub(super) skeleton: &'a Skeleton,
    /// Body-level index (function/method bodies + typed properties) to scan reachable code.
    pub(super) index: AstIndex<'a>,
    /// Reachable function FQNs discovered so far.
    functions: HashSet<String>,
    /// Reachable `(declaring class, lowercase method)` pairs discovered so far.
    methods: HashSet<(String, String)>,
    /// Classes constructed so far (RTA's set of "objects that can exist").
    instantiated: HashSet<String>,
    /// De-duplicated fallback notes, newest-insertion-order preserved in `fallback_notes`.
    fallback_seen: HashSet<String>,
    /// Ordered fallback notes for the final report.
    fallback_notes: Vec<String>,
    /// Functions still to scan.
    fn_worklist: VecDeque<String>,
    /// Methods still to scan, as `(declaring class, lowercase method)`.
    method_worklist: VecDeque<(String, String)>,
    /// Method names with an active `Any` (unknown-receiver) fallback.
    named_fallback: HashSet<String>,
    /// `true` once a dynamic method name forced "all methods of every instantiated class".
    dynamic_fallback: bool,
    /// Typed call sites recorded for RTA re-resolution on later instantiations.
    typed_sites: Vec<TypedSite>,
    /// De-dup guard for typed sites, keyed by `(receiver key, method)`.
    typed_seen: HashSet<(String, String)>,
}

/// Computes the reachable functions/methods/instantiated-classes for `program`, resolving
/// virtual dispatch against the constructed-class set (RTA) and falling back soundly whenever a
/// receiver type is unknown. Pure analysis — it neither mutates the AST nor affects codegen.
pub fn compute_reachable(program: &Program, skeleton: &Skeleton) -> Reachable {
    let analyzer = Analyzer::new(program, skeleton);
    analyzer.run(program)
}

/// Renders a deterministic, human-readable dump of a `Reachable` set for the `ELEPHC_TREE_SHAKE_DUMP`
/// debug path: total counts first, then sorted `Class::method`, sorted `function()`, sorted
/// instantiated classes, and sorted fallback notes. Sorting keeps the output stable across runs.
pub fn dump_reachable(reachable: &Reachable) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "[tree-shake] reachable summary");
    let _ = writeln!(
        out,
        "[tree-shake] counts: methods={} functions={} instantiated={} fallback_sites={}",
        reachable.methods.len(),
        reachable.functions.len(),
        reachable.instantiated.len(),
        reachable.fallback_sites.len(),
    );

    let mut methods: Vec<String> =
        reachable.methods.iter().map(|(class, method)| format!("{class}::{method}")).collect();
    methods.sort();
    for method in &methods {
        let _ = writeln!(out, "[tree-shake]   method {method}");
    }

    let mut functions: Vec<&String> = reachable.functions.iter().collect();
    functions.sort();
    for function in &functions {
        let _ = writeln!(out, "[tree-shake]   function {function}()");
    }

    let mut instantiated: Vec<&String> = reachable.instantiated.iter().collect();
    instantiated.sort();
    for class in &instantiated {
        let _ = writeln!(out, "[tree-shake]   instantiated {class}");
    }

    let mut fallbacks = reachable.fallback_sites.clone();
    fallbacks.sort();
    for note in &fallbacks {
        let _ = writeln!(out, "[tree-shake]   fallback {note}");
    }
    out
}

impl<'a> Analyzer<'a> {
    /// Creates an empty analyzer bound to a program's skeleton and a freshly built body index.
    fn new(program: &'a Program, skeleton: &'a Skeleton) -> Self {
        Analyzer {
            skeleton,
            index: build_index(program),
            functions: HashSet::new(),
            methods: HashSet::new(),
            instantiated: HashSet::new(),
            fallback_seen: HashSet::new(),
            fallback_notes: Vec::new(),
            fn_worklist: VecDeque::new(),
            method_worklist: VecDeque::new(),
            named_fallback: HashSet::new(),
            dynamic_fallback: false,
            typed_sites: Vec::new(),
            typed_seen: HashSet::new(),
        }
    }

    /// Seeds the fixpoint from the program's top-level statements (the entry "virtual body")
    /// and drains the function/method worklists until nothing new is discovered, then returns
    /// the accumulated reachable set.
    fn run(mut self, program: &'a Program) -> Reachable {
        self.scan_root(program);
        loop {
            if let Some(func) = self.fn_worklist.pop_front() {
                self.scan_function(&func);
                continue;
            }
            if let Some((class, method)) = self.method_worklist.pop_front() {
                self.scan_method(&class, &method);
                continue;
            }
            break;
        }
        Reachable {
            functions: self.functions,
            methods: self.methods,
            instantiated: self.instantiated,
            fallback_sites: self.fallback_notes,
        }
    }

    /// Marks `fqn` reachable and enqueues its body for scanning the first time it is seen. A
    /// function without an indexed body (builtin/extern) is still recorded but never scanned.
    pub(super) fn enqueue_function(&mut self, fqn: &str) {
        let key = fqn.trim_start_matches('\\').to_string();
        if self.functions.insert(key.clone()) && self.index.functions.contains_key(&key) {
            self.fn_worklist.push_back(key);
        }
    }

    /// Marks every indexed free function reachable. Used by the computed-string callable fallback:
    /// a runtime string callable could name ANY global function, so soundness requires keeping them
    /// all (they carry no absent-optional-dependency risk, unlike class methods). Idempotent.
    pub(super) fn enqueue_all_functions(&mut self) {
        let names: Vec<String> = self.index.functions.keys().cloned().collect();
        for name in names {
            self.enqueue_function(&name);
        }
    }

    /// Marks `(class, method)` reachable and enqueues it for scanning the first time it is seen.
    /// `class` is the declaring class returned by resolution; `method` is a lowercase key.
    pub(super) fn enqueue_method(&mut self, class: String, method: String) {
        if self.methods.insert((class.clone(), method.clone())) {
            self.method_worklist.push_back((class, method));
        }
    }

    /// Records `class` as constructed. On the first sighting it pulls in the destructor and
    /// resolved constructor, then applies every already-active fallback and typed call site to
    /// the new class (RTA: a fresh object can now receive those virtual calls).
    pub(super) fn instantiate(&mut self, class: &str) {
        let key = class.trim_start_matches('\\').to_string();
        if !self.instantiated.insert(key.clone()) {
            return;
        }
        // Constructor and destructor of any constructed class are always kept.
        for magic in ["__construct", "__destruct"] {
            if let Some((decl, _)) = self.skeleton.resolve_method(&key, magic) {
                self.enqueue_method(decl, magic.to_string());
            }
        }
        self.apply_fallbacks_to_class(&key);
    }

    /// Applies every currently-active fallback and typed call site to a newly constructed class,
    /// enqueuing any method the class exposes that a pending call site could dispatch to.
    fn apply_fallbacks_to_class(&mut self, class: &str) {
        let mut enqueue: Vec<(String, String)> = Vec::new();
        for method in &self.named_fallback {
            if let Some((decl, _)) = self.skeleton.resolve_method(class, method) {
                enqueue.push((decl, method.clone()));
            }
        }
        if self.dynamic_fallback {
            enqueue.extend(self.all_methods_of(class));
        }
        for site in &self.typed_sites {
            if site.classes.contains(class) {
                if let Some((decl, _)) = self.skeleton.resolve_method(class, &site.method) {
                    enqueue.push((decl, site.method.clone()));
                }
            }
        }
        for (decl, method) in enqueue {
            self.enqueue_method(decl, method);
        }
    }

    /// Registers an unknown-receiver ("Any") call on `method`: every instantiated class exposing
    /// `method` becomes reachable, and the name is remembered so future instantiations match too.
    pub(super) fn add_named_fallback(&mut self, method: &str, note: String) {
        self.record_fallback(note);
        if !self.named_fallback.insert(method.to_string()) {
            return;
        }
        let mut enqueue: Vec<(String, String)> = Vec::new();
        for class in &self.instantiated {
            if let Some((decl, _)) = self.skeleton.resolve_method(class, method) {
                enqueue.push((decl, method.to_string()));
            }
        }
        for (decl, m) in enqueue {
            self.enqueue_method(decl, m);
        }
    }

    /// Registers a dynamic-method-name fallback (`$o->{$n}()` / reflection invoke): keeps every
    /// method of every instantiated class, and arms the flag for future instantiations.
    pub(super) fn add_dynamic_fallback(&mut self, note: String) {
        self.record_fallback(note);
        if self.dynamic_fallback {
            return;
        }
        self.dynamic_fallback = true;
        let mut enqueue: Vec<(String, String)> = Vec::new();
        let classes: Vec<String> = self.instantiated.iter().cloned().collect();
        for class in &classes {
            enqueue.extend(self.all_methods_of(class));
        }
        for (decl, method) in enqueue {
            self.enqueue_method(decl, method);
        }
    }

    /// Registers a typed call site: enqueues `method` on the already-instantiated candidates now,
    /// and stores the site so later instantiations of a candidate re-resolve it. De-duplicated by
    /// `(key, method)` so identical receiver shapes are not re-scanned.
    pub(super) fn add_typed_site(&mut self, key: String, classes: HashSet<String>, method: &str) {
        if !self.typed_seen.insert((key, method.to_string())) {
            return;
        }
        let mut enqueue: Vec<(String, String)> = Vec::new();
        for class in &classes {
            if self.instantiated.contains(class) {
                if let Some((decl, _)) = self.skeleton.resolve_method(class, method) {
                    enqueue.push((decl, method.to_string()));
                }
            }
        }
        self.typed_sites.push(TypedSite { classes, method: method.to_string() });
        for (decl, m) in enqueue {
            self.enqueue_method(decl, m);
        }
    }

    /// Enqueues `method` resolved over every class in `classes` regardless of instantiation — used
    /// for static dispatch (`static::m`, class-string callables) where no object is required.
    pub(super) fn enqueue_over_classes(&mut self, classes: &HashSet<String>, method: &str) {
        let mut enqueue: Vec<(String, String)> = Vec::new();
        for class in classes {
            if let Some((decl, _)) = self.skeleton.resolve_method(class, method) {
                enqueue.push((decl, method.to_string()));
            }
        }
        for (decl, m) in enqueue {
            self.enqueue_method(decl, m);
        }
    }

    /// Instantiates every concrete class in the closed world. Used for `new $dynamic` and other
    /// construction channels whose runtime class is unknown; conservative but sound.
    pub(super) fn instantiate_all(&mut self, note: String) {
        self.record_fallback(note);
        let classes: Vec<String> = self
            .skeleton
            .classes
            .keys()
            .filter(|name| self.skeleton.is_instantiable(name))
            .cloned()
            .collect();
        for class in classes {
            self.instantiate(&class);
        }
    }

    /// Instantiates every concrete subtype of `bound` (plus `fallback`). Used for the synthetic
    /// `NewDynamicObject` factory, which constrains the runtime class to a known parent.
    pub(super) fn instantiate_subtypes(&mut self, bound: &str, fallback: &str, note: String) {
        self.record_fallback(note);
        let mut classes: Vec<String> = self
            .skeleton
            .subtypes(bound)
            .into_iter()
            .filter(|name| self.skeleton.is_instantiable(name))
            .collect();
        if self.skeleton.is_instantiable(fallback) {
            classes.push(fallback.trim_start_matches('\\').to_string());
        }
        for class in classes {
            self.instantiate(&class);
        }
    }

    /// Returns every `(declaring class, lowercase method)` the class exposes, walking its parent
    /// chain so inherited methods are included. Used by the dynamic-method-name fallback.
    fn all_methods_of(&self, class: &str) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        let mut seen_names: HashSet<String> = HashSet::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut current = Some(class.trim_start_matches('\\').to_string());
        while let Some(cur) = current {
            if !visited.insert(cur.clone()) {
                break;
            }
            let Some(class_skel) = self.skeleton.classes.get(&cur) else { break };
            for name in class_skel.methods.keys() {
                if seen_names.insert(name.clone()) {
                    out.push((cur.clone(), name.clone()));
                }
            }
            current = class_skel.parent.clone();
        }
        out
    }

    /// Records a fallback note once (de-duplicated), preserving first-seen order for the report.
    fn record_fallback(&mut self, note: String) {
        if self.fallback_seen.insert(note.clone()) {
            self.fallback_notes.push(note);
        }
    }

    /// Expands a receiver type expression into a `Bound` candidate set, or `Any` when the type is
    /// scalar/`mixed`/`object`/an unresolved external class. `key` tags the site for de-dup.
    pub(super) fn type_to_receiver(&self, key: String, ty: &TypeExpr) -> Receiver {
        match self.classes_for_type(ty) {
            Some(classes) if !classes.is_empty() => Receiver::Bound { key, classes },
            _ => Receiver::Any,
        }
    }

    /// Collects the candidate classes for a type expression: `Some(subtypes)` for a known
    /// class/interface (union members combined), `None` when any part is unknown/`Any`-inducing.
    pub(super) fn classes_for_type(&self, ty: &TypeExpr) -> Option<HashSet<String>> {
        match ty {
            TypeExpr::Named(name) => {
                let key = name.as_str().trim_start_matches('\\');
                // `self`/`parent`/`static` are substituted to concrete names before this pass;
                // an external class absent from the skeleton is unknown → fall back.
                if self.skeleton.classes.contains_key(key) {
                    Some(self.skeleton.subtypes(key).into_iter().collect())
                } else {
                    None
                }
            }
            // A nullable object is still that object when non-null; null carries no method.
            TypeExpr::Nullable(inner) => self.classes_for_type(inner),
            TypeExpr::Union(members) => {
                let mut classes = HashSet::new();
                for member in members {
                    match self.classes_for_type(member) {
                        Some(set) => classes.extend(set),
                        // A scalar arm contributes nothing, but an unknown class arm means the
                        // receiver could be an object we cannot name → fall back for the union.
                        None if is_scalarish(member) => {}
                        None => return None,
                    }
                }
                if classes.is_empty() {
                    None
                } else {
                    Some(classes)
                }
            }
            // Everything else (int/float/bool/string/array/iterable/ptr/buffer/intersection) is
            // either a scalar or too imprecise to pin to a closed object set → fall back.
            _ => None,
        }
    }
}

/// Returns `true` for type expressions that can never carry an object method (scalars, arrays,
/// void/never), so they contribute nothing to a union's candidate set instead of forcing `Any`.
fn is_scalarish(ty: &TypeExpr) -> bool {
    matches!(
        ty,
        TypeExpr::Int
            | TypeExpr::Float
            | TypeExpr::Bool
            | TypeExpr::Str
            | TypeExpr::Void
            | TypeExpr::Never
            | TypeExpr::Array(_)
            | TypeExpr::Ptr(_)
            | TypeExpr::Buffer(_)
    )
}
