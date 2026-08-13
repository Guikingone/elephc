//! Purpose:
//! Declares PHP `bcadd()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - A null or omitted scale reads the process-wide BCMath scale.

builtin! {
    contract: "bcadd",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcAdd,
    ),
}
