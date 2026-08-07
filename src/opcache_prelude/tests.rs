//! Purpose:
//! Coordinates focused tests for OPcache prelude rendering and manifest behavior.
//!
//! Called from:
//! - cargo test through Rust's test harness.
//!
//! Key details:
//! - Shared fixtures are re-exported privately so sibling test modules can reuse them.

use super::*;

mod basics;
mod env_restrict;
mod manifest_bake;
mod manifest_ini;
mod preload;

#[allow(unused_imports)]
use basics::*;
#[allow(unused_imports)]
use env_restrict::*;
#[allow(unused_imports)]
use manifest_bake::*;
#[allow(unused_imports)]
use manifest_ini::*;
#[allow(unused_imports)]
use preload::*;
