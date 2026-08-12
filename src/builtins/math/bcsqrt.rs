//! Purpose:
//! Declares PHP `bcsqrt()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Negative inputs throw and non-negative roots truncate to the selected scale.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcsqrt",
    area: Math,
    params: [num: Str, scale: Int = DefaultSpec::Null],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcSqrt,
    ),
    summary: "Returns the square root of an arbitrary-precision decimal number.",
    php_manual: "https://www.php.net/manual/en/function.bcsqrt.php",
}
