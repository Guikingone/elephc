//! Purpose:
//! Collects the runtime helpers that implement PHP's `==` (loose equality) for
//! values whose shape is only known at run time: boxed `Mixed` cells, arrays and
//! hashes compared pair-wise, and objects compared class-then-property.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()`.
//!
//! Key details:
//! - `__rt_mixed_loose_eq` is the single entry point every backend loose-equality
//!   fallback funnels through; the array and object walkers call back into it for
//!   element/property comparison, so all three share one PHP comparison table.
//! - Recursion carries an explicit depth argument instead of global state, so the
//!   helpers stay reentrant; the cap keeps a cyclic structure from overflowing the
//!   stack (see `MAX_LOOSE_EQ_DEPTH`).

mod array_loose_eq;
mod mixed_loose_eq;
mod obj_loose_eq;

pub use array_loose_eq::emit_mixed_array_loose_eq;
pub use mixed_loose_eq::emit_mixed_loose_eq;
pub use obj_loose_eq::emit_obj_loose_eq;

/// Maximum nesting the loose-equality walkers follow before reporting "not equal".
///
/// PHP raises `Fatal error: Nesting level too deep - recursive dependency?` when
/// `==` meets a cyclic array/object graph. elephc has no comparable unwind path in
/// a leaf runtime helper, so the walkers stop at this depth and report inequality
/// instead of recursing until the stack dies. Real data nests far below the cap;
/// only a genuine cycle can reach it.
pub(crate) const MAX_LOOSE_EQ_DEPTH: i64 = 256;
