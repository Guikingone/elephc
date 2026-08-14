//! Purpose:
//! The canonical set of HTTP-request superglobals exposed under `--web`, and the
//! shared PhpType for them. Single source of truth consumed by the type checker,
//! the IR lowering global-storage path, and `__rt_web_reset`.
//!
//! Called from:
//! - `crate::types::checker` (seeding), `crate::ir_lower::context` (global
//!   storage), `crate::codegen::web` (per-request reset).
//!
//! Key details:
//! - These names use `_eir_global_*` symbol storage in EVERY scope (true
//!   superglobals), unlike `$argc`/`$argv` which are top-level only.

use crate::ir::{Immediate, Module, Op};
use crate::types::PhpType;

/// PHP request superglobals visible in every scope under `--web`.
pub const SUPERGLOBALS: &[&str] =
    &["_SERVER", "_GET", "_POST", "_COOKIE", "_REQUEST", "_ENV", "_FILES", "_SESSION"];

/// Returns true when `name` (without leading `$`) is a request superglobal.
pub fn is_superglobal(name: &str) -> bool {
    SUPERGLOBALS.contains(&name)
}

/// The shared type of every request superglobal: a string-keyed associative
/// array of heterogeneous (Mixed) values.
pub fn superglobal_type() -> PhpType {
    PhpType::AssocArray {
        key: Box::new(PhpType::Str),
        value: Box::new(PhpType::Mixed),
    }
}

/// Returns true when EIR contains an owning array-reference marker for this superglobal.
///
/// Only those symbols change from direct hash storage to shared ref-cell storage, keeping the
/// ordinary fast path and eval/pointer ABI unchanged for every other request superglobal.
pub(crate) fn uses_shared_ref_cell(module: &Module, name: &str) -> bool {
    module
        .functions
        .iter()
        .chain(module.closures.iter())
        .flat_map(|function| function.instructions.iter())
        .any(|inst| {
            if inst.op != Op::InvokerRefArg {
                return false;
            }
            let Some(Immediate::GlobalName(data)) = inst.immediate else {
                return false;
            };
            module
                .data
                .global_names
                .get(data.as_raw() as usize)
                .is_some_and(|candidate| candidate == name)
        })
}
