//! Purpose:
//! Home of the PHP `enum_exists` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The check hook accepts literal and runtime string names; literals may seed autoload
//!   discovery while runtime names query the emitted closed-world enum metadata.
//! - Arguments are pre-inferred by the registry common path before the hook runs.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "enum_exists",
    area: Callables,
    params: [enum: Str, autoload: Bool = DefaultSpec::Bool(true)],
    returns: Bool,
    check: crate::builtins::callables::support::check_class_like_exists,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::EnumExists,
    ),
    summary: "Checks if the enum has been defined.",
    php_manual: "function.enum-exists",
}
