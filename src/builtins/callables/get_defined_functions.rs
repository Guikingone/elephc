//! Purpose:
//! Home of the PHP `get_defined_functions` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Check hook returns `array<string, array<string>>` unconditionally (zero-arg builtin).
//! - Lowering mirrors `get_declared_classes`, one level nested: two indexed string
//!   arrays (`'internal'` builtin names, `'user'` user-defined names) boxed into a
//!   two-entry assoc hash.


builtin! {
    name: "get_defined_functions",
    area: Callables,
    params: [],
    returns: Mixed,
    check: crate::builtins::callables::support::check_defined_functions,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::GetDefinedFunctions,
    ),
    summary: "Returns an array of all defined functions, split into 'internal' and 'user'.",
    php_manual: "function.get-defined-functions",
}
