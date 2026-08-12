//! Purpose:
//! Declares PHP `bcsub()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - A null or omitted scale reads the process-wide BCMath scale.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcsub",
    area: Math,
    params: [num1: Str, num2: Str, scale: Int = DefaultSpec::Null],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcSub,
    ),
    summary: "Subtracts two arbitrary-precision decimal numbers.",
    php_manual: "https://www.php.net/manual/en/function.bcsub.php",
}
