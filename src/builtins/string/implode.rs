//! Purpose:
//! Home of the PHP `implode` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Reference PHP declares `implode(string|array $separator, ?array $array = null)`, which is
//!   what makes BOTH accepted call forms work: `implode($separator, $array)` and the
//!   single-argument `implode($array)` that joins with an empty separator. `separator` is
//!   therefore declared `Mixed`, not `Str`, and the backend picks the operand roles from the
//!   argument count (`lower_implode` in `crate::codegen::lower_inst::builtins::strings`).
//! - The legacy reversed `implode($array, $separator)` order was REMOVED in PHP 8.0 and is
//!   deliberately not accepted.
//! - This declaration mirrors the `join` alias exactly; the registry's alias arity gate requires
//!   the two to agree on their enforced bounds (one required parameter, at most two).
//! - No `check` hook narrows the arity: `returns: Str` is authoritative for the checker.


builtin! {
    contract: "implode",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Implode,
    ),
}
