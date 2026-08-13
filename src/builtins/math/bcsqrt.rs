//! Purpose:
//! Declares PHP `bcsqrt()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Negative inputs throw and non-negative roots truncate to the selected scale.

builtin! {
    contract: "bcsqrt",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcSqrt,
    ),
}
