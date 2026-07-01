//! Purpose:
//! Defines the lightweight structural "skeleton" of a program used by tree-shaking.
//! Records the class hierarchy, per-class method-name tables, and free functions
//! WITHOUT resolving any type hints, so it never errors on absent/optional-dep types.
//!
//! Called from:
//! - `crate::tree_shake::harvest` builds these records from the resolved AST.
//! - `crate::tree_shake::query` answers subtype/implementer/resolution questions over them.
//!
//! Key details:
//! - Class keys are canonical FQNs with no leading `\`; method keys are lowercase
//!   (PHP method lookup is case-insensitive). No resolved `PhpType` is stored here:
//!   later stages read parameter/return types from the AST directly.

use std::collections::HashMap;

use crate::parser::ast::{ClassMethod, Visibility};

/// Structural skeleton of a whole program: the closed class world plus free functions.
///
/// `classes` is keyed by canonical class FQN (no leading `\`); `functions` by canonical
/// function FQN. It carries no resolved type information — only structure needed for the
/// reachability and dispatch queries later tree-shaking stages perform.
#[derive(Debug, Clone, PartialEq)]
pub struct Skeleton {
    /// Every declared class-like (class/interface/trait/enum), keyed by canonical FQN.
    pub classes: HashMap<String, ClassSkel>,
    /// Every declared free function, keyed by canonical FQN.
    pub functions: HashMap<String, FnSkel>,
}

impl Skeleton {
    /// Creates an empty skeleton with no classes or functions.
    pub fn new() -> Self {
        Skeleton { classes: HashMap::new(), functions: HashMap::new() }
    }
}

impl Default for Skeleton {
    /// Returns an empty skeleton (delegates to [`Skeleton::new`]).
    fn default() -> Self {
        Skeleton::new()
    }
}

/// Category of a declared class-like entity, so queries can distinguish instantiable
/// concrete classes from interfaces, traits, and enums.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassKind {
    /// A `class` declaration (may still be abstract — see `ClassSkel::is_abstract`).
    Class,
    /// An `interface` declaration.
    Interface,
    /// A `trait` declaration.
    Trait,
    /// An `enum` declaration.
    Enum,
}

/// Structural record for one class-like declaration: its place in the hierarchy,
/// modifier flags, kind, and case-folded method-name table.
///
/// `parent`/`interfaces` hold canonical super-type FQNs exactly as written (post
/// name resolution); a super-type absent from `Skeleton::classes` is external/unknown
/// and marks a hierarchy boundary the queries stop at.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassSkel {
    /// Canonical FQN of this declaration (no leading `\`).
    pub fqn: String,
    /// Single parent class for a `class` (`extends`); `None` for interfaces/traits/enums.
    pub parent: Option<String>,
    /// Implemented/extended interfaces: `implements` for classes/enums, `extends` for interfaces.
    pub interfaces: Vec<String>,
    /// `true` when the declaration is `abstract` (only meaningful for classes).
    pub is_abstract: bool,
    /// `true` when the declaration is `final`.
    pub is_final: bool,
    /// Which kind of class-like this is.
    pub kind: ClassKind,
    /// Methods declared on this type (plus folded trait methods), keyed by lowercase name.
    pub methods: HashMap<String, MethodSkel>,
}

/// Structural record for one method: original-case name plus the modifier flags that
/// dispatch/reachability need. Intentionally carries no resolved parameter/return type —
/// later stages read those from the method body in the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodSkel {
    /// Original-case method name as written in source.
    pub name: String,
    /// Declared visibility (`public`/`protected`/`private`).
    pub visibility: Visibility,
    /// `true` for a `static` method.
    pub is_static: bool,
    /// `true` for a `final` method.
    pub is_final: bool,
    /// `true` for an `abstract` method (or an interface method with no body).
    pub is_abstract: bool,
}

impl MethodSkel {
    /// Builds a `MethodSkel` from a parsed `ClassMethod`, copying its name and modifier
    /// flags. No type resolution happens; only structural fields are captured.
    pub fn from_ast(method: &ClassMethod) -> Self {
        MethodSkel {
            name: method.name.clone(),
            visibility: method.visibility.clone(),
            is_static: method.is_static,
            is_final: method.is_final,
            is_abstract: method.is_abstract,
        }
    }
}

/// Structural record for one free function: just its canonical FQN. Later stages read
/// the parameter/return shape and body from the AST when the function is reachable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnSkel {
    /// Canonical FQN of the function (no leading `\`).
    pub fqn: String,
}

/// Strips a single/leading root separator so an FQN written as `\Foo\Bar` compares
/// equal to the canonical `Foo\Bar` key form used throughout the skeleton.
pub(super) fn strip_root(name: &str) -> &str {
    name.trim_start_matches('\\')
}
