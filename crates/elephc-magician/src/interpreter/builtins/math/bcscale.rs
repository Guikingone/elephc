//! Purpose:
//! Declares the Magician registry entry for `bcscale()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Missing or null scale reads state; an integer sets it and returns the previous value.

eval_builtin! {
    contract: "bcscale",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
