//! Purpose:
//! Home of the internal `__elephc_object_prop_count` builtin: the number of
//! properties an object renders, i.e. the row count of its display descriptor.
//!
//! Called from:
//! - The injected `var_export` prelude (`src/var_export_prelude.rs`), as the bound
//!   of the loop that walks an object's properties.
//!
//! Key details:
//! - `internal: true`: never PHP-visible. PHP's `get_object_vars()` is the closest
//!   equivalent, but it returns an array of `mixed` and elephc has no
//!   object-to-array conversion; a count plus per-index accessors keeps the prelude
//!   in ordinary PHP control flow with no new container type.
//! - Reads `_class_prop_desc_ptrs[class_id]`, the SAME rows `var_dump` and
//!   `print_r` walk, so the three renderers cannot disagree about which properties
//!   an object has. A non-object value reports 0.

builtin! {
    contract: "__elephc_object_prop_count",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ElephcObjectPropCount,
    ),
}
