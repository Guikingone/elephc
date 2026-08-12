//! Purpose:
//! Declares PHP `bcmul()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Multiplication truncates or pads the exact decimal product to the selected scale.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcmul",
    area: Math,
    params: [num1: Str, num2: Str, scale: Int = DefaultSpec::Null],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcMul,
    ),
    summary: "Multiplies two arbitrary-precision decimal numbers.",
    php_manual: "https://www.php.net/manual/en/function.bcmul.php",
}
