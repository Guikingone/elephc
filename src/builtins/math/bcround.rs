//! Purpose:
//! Declares PHP `bcround()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Elephc accepts the existing integer rounding-mode enumeration `1..=8`.

builtin! {
    contract: "bcround",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcRound,
    ),
}
