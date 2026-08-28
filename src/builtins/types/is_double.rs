//! Purpose:
//! Registers PHP's `is_double` alias with the shared float-predicate semantics.
//!
//! Called from:
//! - The builtin registry through `crate::builtins::types`.
//!
//! Key details:
//! - The alias uses the same typed EIR target as `is_float`.

builtin! {
    contract: "is_double",
    semantics: crate::builtins::semantics::type_predicate_semantics(
        crate::ir::PhpTypePredicate::Float,
    ),
}
