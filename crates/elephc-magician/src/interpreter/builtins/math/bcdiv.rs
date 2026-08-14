//! Purpose:
//! Declares the Magician registry entry for `bcdiv()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Division failures become catchable `DivisionByZeroError` objects.

eval_builtin! {
    contract: "bcdiv",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
