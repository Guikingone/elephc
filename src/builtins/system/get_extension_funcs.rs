//! Purpose:
//! Declares the bounded PHP `get_extension_funcs` registry builtin.
//!
//! Called from:
//! - The shared builtin registry through checker, EIR, and codegen consumers.
//!
//! Key details:
//! - The active backend exposes the frozen DOM/libxml/SimpleXML function registries only.

builtin! {
    contract: "get_extension_funcs",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::GetExtensionFuncs,
    ),
}
