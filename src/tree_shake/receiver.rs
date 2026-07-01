//! Purpose:
//! The Stage-2 receiver-typing helper: given a method-call receiver expression, decide the set of
//! classes it can hold using ONLY cheap, local, syntactic facts (no dataflow, no full inference).
//!
//! Called from:
//! - `crate::tree_shake::edges` when lowering a `MethodCall`/`NullsafeMethodCall` into edges.
//!
//! Key details:
//! - Every shape we cannot pin down cheaply returns `Receiver::Any`, which the fixpoint turns into
//!   a sound over-approximation (keep the method on every instantiated class). We NEVER guess a
//!   narrower set than the syntax guarantees — under-approximation would miscompile later stages.
//! - Five shapes are pinned: `$this`, a type-hinted parameter variable, an inline `new`, a
//!   `$this->prop` with a declared property type, and (Stage 2.5) a local variable whose per-body
//!   `LocalType` is `Known` (computed in `locals.rs`). Reassignments are folded into that join, so
//!   a reassigned local widens to `Any`; branch-sensitivity is deliberately not modeled.

use std::collections::HashMap;

use crate::parser::ast::{Expr, ExprKind, StaticReceiver, TypeExpr};

use super::locals::{LocalType, LocalTypes};
use super::reach::{Analyzer, Receiver};

impl<'a> Analyzer<'a> {
    /// Determines the candidate class set for a method-call receiver.
    ///
    /// `enclosing` is the class whose method body we are scanning (for `$this`/`$this->prop`);
    /// `params` maps in-scope parameter names to their declared type hints; `locals` is the per-body
    /// local-variable type map from `locals.rs`. Returns a `Bound` closed set for the five
    /// cheaply-typeable shapes, or `Any` for everything else.
    pub(super) fn receiver_of(
        &self,
        object: &Expr,
        enclosing: Option<&str>,
        params: &HashMap<&'a str, &'a TypeExpr>,
        locals: &LocalTypes,
    ) -> Receiver {
        match &object.kind {
            // `$this` is an instance of the enclosing class or any subtype that could be the
            // runtime object; the fixpoint later intersects this with the instantiated set.
            ExprKind::This => match enclosing {
                Some(class) => Receiver::Bound {
                    key: format!("this@{class}"),
                    classes: self.skeleton.subtypes(class).into_iter().collect(),
                },
                None => Receiver::Any,
            },
            // A local variable's per-body `LocalType` is authoritative when present: it already
            // folds the parameter hint and EVERY reassignment, so a widened `Any` must NOT fall
            // back to the (now stale) parameter hint. Only an absent local — the map carries no
            // information for it — consults the parameter hint, then the sound `Any` fallback.
            ExprKind::Variable(name) => match locals.get(name.as_str()) {
                Some(LocalType::Known(classes)) if !classes.is_empty() => Receiver::Bound {
                    key: format!("local:{name}"),
                    classes: classes.clone(),
                },
                Some(_) => Receiver::Any,
                None => match params.get(name.as_str()) {
                    Some(ty) => self.type_to_receiver(format!("param:{name}"), ty),
                    None => Receiver::Any,
                },
            },
            // A freshly constructed object is exactly its class.
            ExprKind::NewObject { class_name, .. } => {
                let key = class_name.as_str().trim_start_matches('\\');
                if self.skeleton.classes.contains_key(key) {
                    Receiver::Bound {
                        key: format!("new:{key}"),
                        classes: std::iter::once(key.to_string()).collect(),
                    }
                } else {
                    Receiver::Any
                }
            }
            // `new self()`/`new static()`/`new parent()` resolve against the enclosing class.
            ExprKind::NewScopedObject { receiver, .. } => {
                match self.scoped_receiver_class(receiver, enclosing) {
                    Some(class) => Receiver::Bound {
                        key: format!("newscoped:{class}"),
                        classes: self.skeleton.subtypes(&class).into_iter().collect(),
                    },
                    None => Receiver::Any,
                }
            }
            // `$this->prop` with a declared property type pins to that type's subtypes.
            ExprKind::PropertyAccess { object: inner, property }
            | ExprKind::NullsafePropertyAccess { object: inner, property } => {
                if matches!(inner.kind, ExprKind::This) {
                    if let Some(class) = enclosing {
                        if let Some(ty) = self.index.property_type(class, property) {
                            return self
                                .type_to_receiver(format!("prop:{class}::{property}"), ty);
                        }
                    }
                }
                Receiver::Any
            }
            // Chained calls, array elements, static-property reads, casts, ternaries, and anything
            // else are not cheaply typeable here → sound fallback.
            _ => Receiver::Any,
        }
    }

    /// Resolves the concrete enclosing class named by a `self`/`static`/`parent` receiver, or a
    /// literal class name, returning `None` when there is no enclosing class or no parent.
    pub(super) fn scoped_receiver_class(
        &self,
        receiver: &StaticReceiver,
        enclosing: Option<&str>,
    ) -> Option<String> {
        match receiver {
            StaticReceiver::Named(name) => Some(name.as_str().trim_start_matches('\\').to_string()),
            StaticReceiver::Self_ | StaticReceiver::Static => enclosing.map(|c| c.to_string()),
            StaticReceiver::Parent => {
                let class = enclosing?;
                self.skeleton.classes.get(class).and_then(|c| c.parent.clone())
            }
        }
    }
}
