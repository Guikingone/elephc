//! Purpose:
//! Declares the Magician registry entry for `bcpow()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - The exponent remains a decimal string for exact integral validation.

eval_builtin! {
    contract: "bcpow",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
