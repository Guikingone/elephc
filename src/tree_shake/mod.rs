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
//! - Stage 2 adds `compute_reachable` (a Rapid-Type-Analysis reachability fixpoint over the
//!   skeleton) in `reach.rs`, with the body index (`index.rs`), receiver typing (`receiver.rs`),
//!   and edge walk (`edges.rs`). It is pure analysis today: the pipeline only dumps the result.

mod edges;
mod harvest;
mod index;
mod query;
mod reach;
mod receiver;
mod skeleton;

#[cfg(test)]
mod reach_tests;
#[cfg(test)]
mod tests;

pub use harvest::harvest_skeleton;
pub use reach::{compute_reachable, dump_reachable, Reachable};
pub use skeleton::{ClassKind, ClassSkel, FnSkel, MethodSkel, Skeleton};
