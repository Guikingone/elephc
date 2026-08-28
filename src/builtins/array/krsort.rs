//! Purpose:
//! Home of the PHP `krsort` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The golden signature is `first_param_ref(fixed(["array"]))`: exactly 1 argument,
//!   the `array` param is by-reference. The `ref` marker is mandatory — it is what makes
//!   by-reference mutation lower correctly (ir_lower reads `ref_params` from the registry sig).
//! - The shared key-sort checker accepts concrete arrays and integer-indexed cells of
//!   `array<mixed>` locals or supported property places; the latter are checked by nested
//!   lowering before mutation.

builtin! {
    contract: "krsort",
    check: super::key_sort::check,
    semantics: crate::builtins::semantics::with_argument_lowering(
        crate::builtins::semantics::runtime_fn_semantics(crate::ir::RuntimeFnId::Krsort),
        crate::builtins::semantics::BuiltinArgumentLowering::ReverseKeySort,
    ),
}
