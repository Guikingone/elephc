//! Purpose:
//! Declares PHP `bcpow()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - The exponent is received as a decimal string and validated as integral at runtime.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcpow",
    area: Math,
    params: [num: Str, exponent: Str, scale: Int = DefaultSpec::Null],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcPow,
    ),
    summary: "Raises an arbitrary-precision decimal number to an integral power.",
    php_manual: "https://www.php.net/manual/en/function.bcpow.php",
}
