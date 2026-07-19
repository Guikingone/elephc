//! Purpose:
//! Groups the callables integration test submodules into the parent suite.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Submodules group focused fixtures for closures, expr calls, func_num_args/func_get_args/func_get_arg, language features, constants and system, state and variadics.

mod closures;
mod expr_calls;
mod func_args_intrinsics;
mod language_features;
mod constants_and_system;
mod shutdown_functions;
mod state_and_variadics;
mod pipe;
