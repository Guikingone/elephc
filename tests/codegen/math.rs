//! Purpose:
//! Groups the math integration test submodules into the parent suite.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Submodules group focused fixtures for functions and PHP float->int conversion.

#[path = "math/functions.rs"]
mod functions;
#[path = "math/php_float_to_int.rs"]
mod php_float_to_int;
