//! Purpose:
//! Declares PHP `bcpow()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - The exponent is received as a decimal string and validated as integral at runtime.

builtin! {
    contract: "bcpow",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcPow,
    ),
}
