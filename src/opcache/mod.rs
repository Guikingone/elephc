//! Purpose:
//! Compile-time OPcache introspection data shared across the native compiler and
//! the magician eval interpreter. This is the first increment of the OPcache core:
//! the `opcache.*` directive matrix that backs `opcache_get_configuration()`.
//!
//! Called from:
//! - `crate::opcache_prelude` (renders the directive table to a PHP array literal).
//! - `crates/elephc-magician` (shares `directives.rs` verbatim via a `#[path]`
//!   include to build the equivalent runtime array).
//!
//! Key details:
//! - `directives` is dependency-free on purpose so the exact same source file is the
//!   single source of truth in both crates, with no duplication or drift.
//! - `state` derives the cache-enabled boolean (the compile-time SAPI-gated state that
//!   governs `opcache_reset()`) from that same directive table, so the two stay in sync.

pub mod directives;
pub mod state;
