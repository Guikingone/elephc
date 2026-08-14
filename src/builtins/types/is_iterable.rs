//! Purpose:
//! Home of the PHP `is_iterable` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Uses the shared typed EIR predicate, including Traversable checks for object values.


builtin! {
    contract: "is_iterable",
    semantics: crate::builtins::semantics::type_predicate_semantics(
        crate::ir::PhpTypePredicate::Iterable,
    ),
}
