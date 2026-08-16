//! Purpose:
//! Home of the PHP `class_implements` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `lazy_check: true` so the hook infers each argument exactly once in source order,
//!   matching the legacy arm.
//! - The check hook validates that the first argument is an object or string literal
//!   and that the optional autoload arg is a literal bool or int.


builtin! {
    contract: "class_implements",
    check: crate::builtins::callables::support::check_class_relation,
    lazy_check: true,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ClassImplements,
    ),
}
