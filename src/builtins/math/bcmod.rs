//! Purpose:
//! Declares PHP `bcmod()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Remainders retain the dividend sign and honor the selected output scale.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcmod",
    area: Math,
    params: [num1: Str, num2: Str, scale: Int = DefaultSpec::Null],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcMod,
    ),
    summary: "Returns the remainder of arbitrary-precision decimal division.",
    php_manual: "https://www.php.net/manual/en/function.bcmod.php",
}
