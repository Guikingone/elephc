//! Purpose:
//! Home of the internal `__elephc_object_prop_value` builtin: the value held by an
//! object's Nth renderable property, boxed as a `Mixed` cell.
//!
//! Called from:
//! - The injected `var_export` prelude (`src/var_export_prelude.rs`), which feeds
//!   the value straight back into its recursive renderer.
//!
//! Key details:
//! - `internal: true`: never PHP-visible.
//! - OWNERSHIP: the result is `Fresh`. Every property slot is re-boxed through
//!   `__rt_mixed_from_value`, which persists a string payload and increfs a
//!   container/object payload, so the returned cell is independently owned and the
//!   caller's ordinary release cannot damage the object it came from. Handing back
//!   a property's own `Mixed` cell instead would alias object storage into a
//!   caller-released temporary.
//! - A missing index, a non-object value, an uninitialized typed property, or a
//!   slot holding the in-band null sentinel all box canonical PHP `null`.

builtin! {
    contract: "__elephc_object_prop_value",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ElephcObjectPropValue,
    ),
}
