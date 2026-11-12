//! Purpose:
//! php-src conformance suite: `.phpt` tests from the PHP source tree rewritten as Rust tests,
//! grouped by the php-src directory they come from.
//!
//! Called from:
//! - `cargo test` through Rust's test harness (`codegen::php_src`).
//!
//! Key details:
//! - Each test keeps the phpt `--FILE--` section verbatim and asserts PHP's exact stdout.
//! - Regenerate or extend with `scripts/php-src/phpt_triage.py` (runs php-src tests against
//!   elephc and the local `php`) and `scripts/php-src/phpt_migrate.py` (writes these files).

mod array;
mod json;
mod math;
mod strings;
