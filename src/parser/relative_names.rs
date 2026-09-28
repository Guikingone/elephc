//! Purpose:
//! Resolves PHP relative names (`namespace\foo`) where the parser meets them, against the
//! namespace the parser is currently inside.
//!
//! Called from:
//! - `crate::parser::parse_with_recovery_inner()` (one scope per parsed file).
//! - `crate::parser::stmt::namespace_use` (namespace declarations and blocks), and the name,
//!   expression, statement, type and attribute parsers that accept a name.
//!
//! Key details:
//! - `namespace\foo` means "foo in the current namespace", so it is exactly `\Current\Ns\foo`,
//!   and plain `\foo` in the global namespace. Resolving it to that fully qualified `Name` at
//!   parse time keeps every later pass on the path it already has for `\Current\Ns\foo`.
//! - The namespace is tracked structurally by the namespace-statement parser rather than by a
//!   token scan: `namespace X;` sets it until the next declaration, and a braced
//!   `namespace X { ... }` sets it for its body and restores the previous one at its `}`, as the
//!   name resolver does. An enum case or method named `namespace` never reaches that parser.
//! - Only a `namespace` token directly followed by `\` is a relative prefix. A `namespace`
//!   segment after a separator (`\Demo\Namespace\Foo`) is an ordinary segment.

use std::cell::RefCell;

use crate::lexer::{SpannedToken, Token};

thread_local! {
    /// Segments of the namespace the parser is currently inside; empty for the global one.
    static CURRENT_NAMESPACE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Runs `parse` with the parser in the global namespace, restoring the enclosing parse's
/// namespace afterwards.
///
/// Every parsed file starts in the global namespace (an included file does not inherit its
/// includer's), and a nested parse must not leak its namespace into the parse that started it.
pub(super) fn with_global_namespace_scope<R>(parse: impl FnOnce() -> R) -> R {
    let previous = CURRENT_NAMESPACE.with(|current| std::mem::take(&mut *current.borrow_mut()));
    let result = parse();
    CURRENT_NAMESPACE.with(|current| *current.borrow_mut() = previous);
    result
}

/// Makes `parts` the current namespace and returns the one it replaces, for a braced block to
/// hand back to [`restore_namespace`] at its closing brace.
pub(crate) fn enter_namespace(parts: Vec<String>) -> Vec<String> {
    CURRENT_NAMESPACE.with(|current| std::mem::replace(&mut *current.borrow_mut(), parts))
}

/// Restores the namespace a braced namespace block replaced.
pub(crate) fn restore_namespace(previous: Vec<String>) {
    CURRENT_NAMESPACE.with(|current| *current.borrow_mut() = previous);
}

/// Returns the segments of the namespace a relative name at this point resolves against.
pub(crate) fn current_namespace_parts() -> Vec<String> {
    CURRENT_NAMESPACE.with(|current| current.borrow().clone())
}

/// Returns true when the tokens at `pos` are the relative-name prefix `namespace\`.
pub(crate) fn relative_name_starts_at(tokens: &[SpannedToken], pos: usize) -> bool {
    matches!(tokens.get(pos), Some((Token::Namespace, _)))
        && matches!(tokens.get(pos + 1), Some((Token::Backslash, _)))
}
