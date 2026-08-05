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

pub mod stub;
