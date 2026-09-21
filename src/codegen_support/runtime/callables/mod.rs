//! Purpose:
//! Wires callable-introspection runtime helpers used by type builtins.
//! Keeps the helper surface narrow while data tables live in runtime/data.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()`.
//!
//! Key details:
//! - Helpers must match callable metadata tables for builtin functions, user functions, and class methods.

mod closure_bind;
mod function_exists;
mod is_callable;
mod descriptor_method_name;
mod spread_array;
mod descriptor_release;
mod lookup;

pub(crate) use closure_bind::emit_closure_bind;
pub(crate) use descriptor_method_name::emit_callable_descriptor_method_name;
pub(crate) use spread_array::emit_mixed_spread_array;
pub(crate) use descriptor_release::emit_callable_descriptor_release;
pub(crate) use function_exists::emit_function_exists_lookup;
pub(crate) use is_callable::emit_is_callable_runtime;
pub(crate) use lookup::emit_callable_lookup;
