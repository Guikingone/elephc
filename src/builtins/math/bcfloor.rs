//! Purpose:
//! Declares PHP `bcfloor()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - The result is a freshly allocated scale-zero decimal string.

builtin! {
    name: "bcfloor",
    area: Math,
    params: [num: Str],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcFloor,
    ),
    summary: "Rounds an arbitrary-precision decimal number down to an integer.",
    php_manual: "https://www.php.net/manual/en/function.bcfloor.php",
}
