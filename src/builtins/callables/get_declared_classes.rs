//! Purpose:
//! Home of the PHP `get_declared_classes` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Check hook returns `Array<Str>` unconditionally (zero-arg builtin).


builtin! {
    contract: "get_declared_classes",
    check: crate::builtins::callables::support::check_declared_names,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::GetDeclaredClasses,
    ),
}
