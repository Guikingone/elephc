//! Purpose:
//! Declares PHP `bccomp()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Comparison truncates both operands to the explicit or process-default scale.

builtin! {
    contract: "bccomp",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcComp,
    ),
}
