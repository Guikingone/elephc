//! Purpose:
//! Declares PHP `bcmul()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Multiplication truncates or pads the exact decimal product to the selected scale.

builtin! {
    contract: "bcmul",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcMul,
    ),
}
