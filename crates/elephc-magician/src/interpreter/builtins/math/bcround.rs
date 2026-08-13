//! Purpose:
//! Declares the Magician registry entry for `bcround()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Precision defaults to zero and integer mode one is half away from zero.

eval_builtin! {
    contract: "bcround",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
