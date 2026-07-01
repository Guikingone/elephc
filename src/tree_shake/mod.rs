//! Purpose:
//! Entry point for the tree-shaking skeleton: harvests a lightweight structural view of
//! the program (class hierarchy + method-name tables + free functions) and answers the
//! subtype/implementer/instantiability/method-resolution queries later stages need.
//!
//! Called from:
//! - `crate::pipeline::compile`, gated behind `--tree-shake` (Stage 1 discards the result,
//!   so the flag has ZERO effect on diagnostics or codegen yet).
//!
//! Key details:
//! - The harvest deliberately does NOT resolve type hints, so it never errors on an
//!   absent/optional-dependency type. See `harvest.rs` for the walk and `skeleton.rs`
//!   for the data shapes; `query.rs` holds the closed-world queries.

mod harvest;
mod query;
mod skeleton;

#[cfg(test)]
mod tests;

pub use harvest::harvest_skeleton;
pub use skeleton::{ClassKind, ClassSkel, FnSkel, MethodSkel, Skeleton};
