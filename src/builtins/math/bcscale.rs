//! Purpose:
//! Declares PHP `bcscale()` and its shared BCMath process-state contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - Null or omission reads the scale; an integer sets it and returns the previous scale.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "bcscale",
    area: Math,
    params: [scale: Int = DefaultSpec::Null],
    returns: Int,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcScale,
    ),
    summary: "Gets or sets the process-wide default BCMath scale.",
    php_manual: "https://www.php.net/manual/en/function.bcscale.php",
}
