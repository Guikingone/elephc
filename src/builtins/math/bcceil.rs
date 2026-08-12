//! Purpose:
//! Declares PHP `bcceil()` and its shared BCMath runtime contract.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - The result is a freshly allocated scale-zero decimal string.

builtin! {
    name: "bcceil",
    area: Math,
    params: [num: Str],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcCeil,
    ),
    summary: "Rounds an arbitrary-precision decimal number up to an integer.",
    php_manual: "https://www.php.net/manual/en/function.bcceil.php",
}
