//! Purpose:
//! Declares PHP `bcpowmod()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Base, exponent, and modulus are validated as integral decimal strings at runtime.

builtin! {
    contract: "bcpowmod",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcPowmod,
    ),
}
