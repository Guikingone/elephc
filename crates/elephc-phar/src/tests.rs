//! Purpose:
//! Coordinates focused unit-test modules for the PHAR bridge.
//!
//! Called from:
//! - `cargo test -p elephc-phar` through Rust's test harness.
//!
//! Key details:
//! - Common archive fixture builders are shared through this parent module.

use super::*;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::Write;

mod extraction;
mod fixtures;
mod metadata;
mod mutation;
mod signatures;
mod zip_features;

#[allow(unused_imports)]
use fixtures::*;
