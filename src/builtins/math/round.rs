//! Purpose:
//! Home of the PHP `round` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `round` is a pure-data builtin whose return type
//!   (`Float`) is fully determined by its declaration.
//! - The second parameter `precision` and the third parameter `mode` are optional with
//!   defaults of `0` and `PHP_ROUND_HALF_UP` (`1`), matching PHP 8.4's
//!   `round(num, precision = 0, mode = RoundingMode::HalfAwayFromZero)` signature. The
//!   registry enforces 1-3 args.
//! - `$mode` is NOT validated here: PHP raises a catchable `ValueError` at runtime for an
//!   out-of-range mode, and the mode can be a runtime value, so the guard lives in the
//!   backend (`codegen::lower_inst::builtins::round_mode`) next to the ABI materialization.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "round",
    area: Math,
    params: [
        num: Float,
        precision: Int = DefaultSpec::Int(0),
        mode: Int = DefaultSpec::Int(1)
    ],
    returns: Float,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Round,
    ),
    summary: "Rounds a float.",
    php_manual: "https://www.php.net/manual/en/function.round.php",
}
