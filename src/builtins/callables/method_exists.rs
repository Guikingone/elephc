//! Purpose:
//! Registers PHP's `method_exists` metadata lookup as a typed builtin operation.
//!
//! Called from:
//! - The builtin registry through `crate::builtins::callables`.
//!
//! Key details:
//! - Static class metadata and eval-aware lookup remain backend implementation details.

builtin! {
    contract: "method_exists",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::MethodExists,
    ),
}
