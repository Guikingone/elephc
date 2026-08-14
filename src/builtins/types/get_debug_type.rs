//! Purpose:
//! Home of PHP's `get_debug_type` builtin and its runtime-aware type-name semantics.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The target-aware backend distinguishes boxed runtime tags and dynamic object class names.

builtin! {
    name: "get_debug_type",
    area: Types,
    params: [value: Mixed],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::GetDebugType,
    ),
    summary: "Returns a debug-oriented PHP type or class name.",
    php_manual: "function.get-debug-type",
}
