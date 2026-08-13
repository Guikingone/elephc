//! Purpose:
//! Declares PHP `bcscale()` and its shared BCMath process-state contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Null or omission reads the scale; an integer sets it and returns the previous scale.

builtin! {
    contract: "bcscale",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcScale,
    ),
}
