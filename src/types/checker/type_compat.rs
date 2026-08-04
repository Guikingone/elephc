//! Purpose:
//! Implements assignability and compatibility rules for `PhpType` values.
//! Delegates object, declaration, pointer, and union-specific checks to focused helpers.
//!
//! Called from:
//! - `crate::types::checker::Checker`
//! - `crate::types::traits`
//!
//! Key details:
//! - Compatibility must be conservative for Mixed, unions, nullable values, inheritance, and pointer-like extensions.

pub(crate) mod declarations;
mod object_types;
mod pointers;
mod unions;

pub(crate) use object_types::type_is_gradual_object_family;
