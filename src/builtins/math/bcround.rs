//! Purpose:
//! Declares PHP `bcround()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Elephc accepts the existing integer rounding-mode enumeration `1..=8`.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcround",
    area: Math,
    params: [
        num: Str,
        precision: Int = DefaultSpec::Int(0),
        mode: Int = DefaultSpec::Int(1)
    ],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcRound,
    ),
    summary: "Rounds an arbitrary-precision decimal number.",
    php_manual: "https://www.php.net/manual/en/function.bcround.php",
}
