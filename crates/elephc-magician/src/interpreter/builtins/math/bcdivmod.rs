//! Purpose:
//! Declares the Magician registry entry for `bcdivmod()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - The shared hook returns a two-element quotient/remainder array.

eval_builtin! {
    contract: "bcdivmod",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
