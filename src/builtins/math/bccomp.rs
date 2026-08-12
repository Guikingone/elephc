//! Purpose:
//! Declares PHP `bccomp()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Comparison truncates both operands to the explicit or process-default scale.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bccomp",
    area: Math,
    params: [num1: Str, num2: Str, scale: Int = DefaultSpec::Null],
    returns: Int,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcComp,
    ),
    summary: "Compares two arbitrary-precision decimal numbers.",
    php_manual: "https://www.php.net/manual/en/function.bccomp.php",
}
