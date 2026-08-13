//! Purpose:
//! Home of the PHP `base_convert` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - PHP names the first parameter `$num`, not `$number`; named-argument calls depend on it.
//! - Both base arguments raise php-src's `ValueError` outside `2..=36`, so the runtime
//!   function is declared `MAY_THROW` and cannot be eliminated as dead code.
//! - Values past `PHP_INT_MAX` widen to `double` during the parse and render through
//!   php-src's lossy float loop; `crate::codegen_support::runtime::strings::base_convert`
//!   owns that contract.

builtin! {
    contract: "base_convert",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BaseConvert,
    ),
}
