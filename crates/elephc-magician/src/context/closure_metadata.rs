//! Purpose:
//! Defines normalized closure targets, capture bindings, and eval closure metadata.
//!
//! Called from:
//! - Closure construction, binding, Reflection, and callable dispatch.
//!
//! Key details:
//! - Bound receivers/scopes and by-reference captures remain explicit runtime metadata.

use super::*;

/// Callable target represented by a PHP-visible eval `Closure` object.
#[derive(Clone)]
pub enum EvalClosureObjectTarget {
    Named(String),
    BoundNamed {
        name: String,
        bound_this: Option<RuntimeCellHandle>,
        bound_scope: Option<String>,
    },
    InvokableObject {
        object: RuntimeCellHandle,
    },
    ObjectMethod {
        object: RuntimeCellHandle,
        method: String,
        called_class: Option<String>,
        native_class: Option<String>,
        bridge_scope: Option<String>,
    },
    StaticMethod {
        class_name: String,
        method: String,
        called_class: Option<String>,
        native_class: Option<String>,
        bridge_scope: Option<String>,
    },
}

impl EvalClosureObjectTarget {
    /// Returns the receiver cell owned by this closure target, when it has one.
    pub(crate) fn receiver(&self) -> Option<RuntimeCellHandle> {
        match self {
            Self::BoundNamed { bound_this, .. } => *bound_this,
            Self::InvokableObject { object } | Self::ObjectMethod { object, .. } => Some(*object),
            Self::Named(_) | Self::StaticMethod { .. } => None,
        }
    }

    /// Allows closure construction to replace a borrowed receiver with its retained cell.
    pub(crate) fn receiver_mut(&mut self) -> Option<&mut RuntimeCellHandle> {
        match self {
            Self::BoundNamed { bound_this, .. } => bound_this.as_mut(),
            Self::InvokableObject { object } | Self::ObjectMethod { object, .. } => Some(object),
            Self::Named(_) | Self::StaticMethod { .. } => None,
        }
    }
}

/// Runtime value captured by an eval closure literal.
#[derive(Clone)]
pub struct EvalClosureCaptureBinding {
    pub(super) name: String,
    pub(super) value: RuntimeCellHandle,
    pub(super) by_ref_target: Option<EvalReferenceTarget>,
}

impl EvalClosureCaptureBinding {
    /// Creates one captured runtime value with optional caller-side by-reference storage.
    pub fn new(
        name: impl Into<String>,
        value: RuntimeCellHandle,
        by_ref_target: Option<EvalReferenceTarget>,
    ) -> Self {
        Self {
            name: name.into(),
            value,
            by_ref_target,
        }
    }

    /// Returns the captured variable name without the leading `$`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the runtime cell captured by the closure.
    pub const fn value(&self) -> RuntimeCellHandle {
        self.value
    }

    /// Returns caller-side writeback metadata for by-reference captures.
    pub fn by_ref_target(&self) -> Option<&EvalReferenceTarget> {
        self.by_ref_target.as_ref()
    }
}

/// One eval closure instance retained by a synthetic callable name.
#[derive(Clone)]
pub struct EvalClosure {
    pub(super) function: EvalFunction,
    pub(super) captures: Vec<EvalClosureCaptureBinding>,
    pub(super) is_static: bool,
    /// Class scope at the closure's declaration site, retained for deferred invocation.
    pub(super) declaring_class_scope: Option<String>,
    /// Late-static class at the declaration site, retained independently from `self` scope.
    pub(super) declaring_called_class_scope: Option<String>,
    /// File, directory, line, and `__FILE__` override active at the declaration site.
    pub(super) declaring_call_site: Option<(String, String, i64, Option<String>)>,
    /// The unique name `define_closure` minted for THIS closure, used to key its `static` slots.
    ///
    /// php gives each closure OBJECT its own static storage: two closures produced by calling the
    /// same factory twice count separately. `EvalFunction::name()` is the parser's name for the
    /// closure LITERAL and is shared by every object made from it, so keying statics on it made
    /// the two share one slot.
    pub(super) slot_key: String,
}

impl EvalClosure {
    /// Returns the per-object key this closure's `static` slots hang from.
    pub fn slot_key(&self) -> &str {
        &self.slot_key
    }

    /// Records the unique name this closure was defined under.
    pub fn set_slot_key(&mut self, slot_key: impl Into<String>) {
        self.slot_key = slot_key.into();
    }

    /// Creates one closure instance from its function body and captured values.
    pub fn new(
        function: EvalFunction,
        captures: Vec<EvalClosureCaptureBinding>,
        is_static: bool,
    ) -> Self {
        Self {
            slot_key: String::new(),
            function,
            captures,
            is_static,
            declaring_class_scope: None,
            declaring_called_class_scope: None,
            declaring_call_site: None,
        }
    }

    /// Returns the executable eval function payload for this closure.
    pub fn function(&self) -> &EvalFunction {
        &self.function
    }

    /// Returns the captured runtime values attached to this closure instance.
    pub fn captures(&self) -> &[EvalClosureCaptureBinding] {
        &self.captures
    }

    /// Returns whether this closure was declared with PHP's `static function` form.
    pub const fn is_static(&self) -> bool {
        self.is_static
    }

    /// Records the lexical and late-static scopes active where this closure was declared.
    pub(crate) fn set_declaring_class_scopes(
        &mut self,
        class_scope: Option<String>,
        called_class_scope: Option<String>,
    ) {
        self.declaring_class_scope = class_scope;
        self.declaring_called_class_scope = called_class_scope;
    }

    /// Returns the lexical class whose non-public members this closure may access.
    pub(crate) fn declaring_class_scope(&self) -> Option<&str> {
        self.declaring_class_scope.as_deref()
    }

    /// Returns the late-static class captured at the declaration site.
    pub(crate) fn declaring_called_class_scope(&self) -> Option<&str> {
        self.declaring_called_class_scope.as_deref()
    }

    /// Records the file-local execution frame that declared this closure.
    pub(crate) fn set_declaring_call_site(
        &mut self,
        call_site: (String, String, i64, Option<String>),
    ) {
        self.declaring_call_site = Some(call_site);
    }

    /// Returns the declaration frame used for file-relative code inside the closure body.
    pub(crate) fn declaring_call_site(&self) -> Option<&(String, String, i64, Option<String>)> {
        self.declaring_call_site.as_ref()
    }
}
