//! Purpose:
//! Declares PHP `bcmod()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Remainders retain the dividend sign and honor the selected output scale.

builtin! {
    contract: "bcmod",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcMod,
    ),
}
