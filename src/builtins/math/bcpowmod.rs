//! Purpose:
//! Declares PHP `bcpowmod()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Base, exponent, and modulus are validated as integral decimal strings at runtime.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcpowmod",
    area: Math,
    params: [
        num: Str,
        exponent: Str,
        modulus: Str,
        scale: Int = DefaultSpec::Null
    ],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcPowmod,
    ),
    summary: "Returns an arbitrary-precision integral modular power.",
    php_manual: "https://www.php.net/manual/en/function.bcpowmod.php",
}
