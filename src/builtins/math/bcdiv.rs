//! Purpose:
//! Declares PHP `bcdiv()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Division truncates and can throw a catchable `DivisionByZeroError`.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcdiv",
    area: Math,
    params: [num1: Str, num2: Str, scale: Int = DefaultSpec::Null],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcDiv,
    ),
    summary: "Divides two arbitrary-precision decimal numbers.",
    php_manual: "https://www.php.net/manual/en/function.bcdiv.php",
}
