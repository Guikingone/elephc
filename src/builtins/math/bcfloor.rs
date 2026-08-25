//! Purpose:
//! Declares PHP `bcfloor()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - The result is a freshly allocated scale-zero decimal string.

builtin! {
    contract: "bcfloor",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcFloor,
    ),
}
