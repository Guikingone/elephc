//! Purpose:
//! Home of the PHP `is_scalar` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Uses the shared typed EIR predicate across scalar and boxed dynamic values.


builtin! {
    contract: "is_scalar",
    semantics: crate::builtins::semantics::type_predicate_semantics(
        crate::ir::PhpTypePredicate::Scalar,
    ),
}
