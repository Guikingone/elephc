//! Purpose:
//! Home of the PHP `is_float` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Uses the shared typed EIR predicate; dynamic values are inspected by target-aware codegen.

builtin! {
    contract: "is_float",
    semantics: crate::builtins::semantics::type_predicate_semantics(
        crate::ir::PhpTypePredicate::Float,
    ),
}
