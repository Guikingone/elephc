//! Purpose:
//! Groups the object property access integration test submodules into the parent suite.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Submodules group focused fixtures for nullsafe property and method access, mutations, deep chains, null-capable int property storage, and properties passed as by-reference arguments to mutating array builtins.

use super::*;

mod by_ref_builtin_args;
mod nullsafe;
mod nullsafe_side_effects;
mod mutations;
mod deep_chains;
mod nullable_int_defaults;
