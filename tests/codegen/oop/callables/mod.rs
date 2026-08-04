//! Purpose:
//! Groups the object-oriented PHP callables integration test submodules into the parent suite.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Submodules group focused fixtures for functions and builtins, methods, variadics, and the
//!   `is_object`/`is_callable` answers a closure gets, and the declared `: callable` return slot.

use super::*;

mod closure_is_object;
mod declared_callable_return;
mod functions_and_builtins;
mod methods;
mod variadics;
