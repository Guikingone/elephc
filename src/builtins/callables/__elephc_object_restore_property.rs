//! Purpose:
//! Restores a serialized property through the shared typed property writer.
//!
//! Called from:
//! - Injected SPL hydration methods, after validating their serialized data.
//!
//! Key details:
//! - PHP mangled keys select physical private/protected slots, including shadowed slots.
//! - The writer borrows inputs, retains the replacement and releases the previous value.

builtin! {
    contract: "__elephc_object_restore_property",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ElephcObjectRestoreProperty,
    ),
}
