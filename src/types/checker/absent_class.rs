//! Purpose:
//! Shared tolerance helpers for references to classes that are absent from the closed world.
//! Lets every reference position (type hint, `new`, static access, `catch`) degrade an unknown
//! class to `PhpType::Mixed` plus a warning instead of hard-erroring.
//!
//! Called from:
//! - `crate::types::checker::type_compat::pointers` (type hints)
//! - `crate::types::checker::inference::objects` (`new`, static access)
//! - `crate::types::checker::stmt_check::control_flow` (`catch`)
//!
//! Key details:
//! - "Absent" means the name is not known anywhere in the closed world (class/interface/enum/
//!   extern/packed, declared or defined). Only that case degrades; genuine reserved-word,
//!   syntax, arity, and missing-member errors keep erroring so typo/contract detection stays.
//! - Modeling an uninstalled optional dependency as absent is PHP-faithful: at runtime the class
//!   does not exist, `class_exists()` returns false, and the guarding `throw` fires, so the
//!   opaque `Mixed` value is only ever produced by DCE-pruned guarded code.

use crate::span::Span;

use super::Checker;

impl Checker {
    /// Returns true when `name` names a class-like symbol that exists in the closed world:
    /// a defined or forward-declared class, a defined or forward-declared interface, an enum,
    /// an extern class, or a packed class. Names are matched exactly (callers pass canonical,
    /// namespace-resolved names), mirroring the existing per-site lookups.
    pub(crate) fn class_like_exists(&self, name: &str) -> bool {
        self.classes.contains_key(name)
            || self.declared_classes.contains(name)
            || self.interfaces.contains_key(name)
            || self.declared_interfaces.contains(name)
            || self.enums.contains_key(name)
            || self.extern_classes.contains_key(name)
            || self.packed_classes.contains_key(name)
    }

    /// Records a warning that an unresolved class reference is being treated as an absent
    /// optional dependency (its uses degrade to `PhpType::Mixed`). Used by every reference
    /// position that holds `&mut self` (`new`, static access, `catch`), so the softened
    /// typo/optional-dependency signal stays visible in diagnostics.
    pub(crate) fn warn_absent_class(&mut self, span: Span, name: &str) {
        let message = absent_class_message(name);
        if !warning_already_present(&self.warnings, span, &message) {
            self.warnings
                .push(crate::errors::CompileWarning::new(span, &message));
        }
    }

    /// Buffers an absent-class warning discovered while resolving a type hint. Type resolution
    /// runs behind `&self`, so the warning is interned into `absent_class_warnings` (deduplicated
    /// by span+message, since the same hint is resolved across several checker passes) and later
    /// merged into `warnings` by `drain_absent_class_warnings`.
    pub(crate) fn pending_absent_class_warning(&self, span: Span, name: &str) {
        let message = absent_class_message(name);
        let mut buffer = self.absent_class_warnings.borrow_mut();
        if !warning_already_present(&buffer, span, &message) {
            buffer.push(crate::errors::CompileWarning::new(span, &message));
        }
    }

    /// Moves the buffered type-hint absent-class warnings into `warnings`, skipping any that a
    /// `&mut self` site already emitted for the same span+message. Called once after checking.
    pub(crate) fn drain_absent_class_warnings(&mut self) {
        let buffered = std::mem::take(&mut *self.absent_class_warnings.borrow_mut());
        for warning in buffered {
            if !warning_already_present(&self.warnings, warning.span, &warning.message) {
                self.warnings.push(warning);
            }
        }
    }
}

/// Builds the canonical absent-class warning message for `name`.
fn absent_class_message(name: &str) -> String {
    format!(
        "reference to unknown class '{}' treated as an absent optional dependency",
        name
    )
}

/// Returns true when `warnings` already holds a warning at the same span with the same message.
/// `Span` is not `PartialEq`, so line/column are compared explicitly.
fn warning_already_present(
    warnings: &[crate::errors::CompileWarning],
    span: Span,
    message: &str,
) -> bool {
    warnings
        .iter()
        .any(|existing| existing.span.line == span.line && existing.span.col == span.col && existing.message == message)
}
