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
}
