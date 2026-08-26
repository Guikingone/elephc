//! Purpose:
//! Declares PHP `bcdiv()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Division truncates and can throw a catchable `DivisionByZeroError`.

builtin! {
    contract: "bcdiv",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcDiv,
    ),
}
