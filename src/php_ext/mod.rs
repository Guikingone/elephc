//! Purpose:
//! Hosting support for real PHP extensions: deriving their callable surface and
//! deciding whether they can be hosted at all.
//!
//! Called from:
//! - Not yet wired into the pipeline. This module is self-contained and tested
//!   in isolation while the surrounding plumbing (recipe, manifest section,
//!   codegen) is built.
//!
//! Key details:
//! - `stub` reads an extension's own `*.stub.php` rather than a hand-maintained
//!   catalogue, so signatures cannot drift from the extension they describe.
//! - `admission` refuses, at declaration time, extensions that would link
//!   cleanly and then silently do nothing — the failure static linking misses.

//! - `manifest` owns the `[php-ext]` section, mirroring `native_deps::manifest`
//!   so declaring an extension never disturbs `[native]` or hand-written TOML.

//! - `recipe` decides how an extension is compiled, as an inspectable value kept
//!   separate from running the commands.

//! - `shim` carries the Zend stand-in as embedded C, following the
//!   `pcre2_shim.c` precedent rather than the pure-Rust bridge-crate pattern.

//! - `call` describes the ABI-level call sequence for a hosted function, kept
//!   separate from emitting it.

//! - `build` executes a plan with a real toolchain, archiving the shim once
//!   rather than folding a copy into every extension.

pub mod admission;
pub mod build;
pub mod call;
pub mod manifest;
pub mod recipe;
pub mod shim;
pub mod stub;
