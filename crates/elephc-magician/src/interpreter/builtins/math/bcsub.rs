//! Purpose:
//! Declares the Magician registry entry for `bcsub()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Null scale selects the crate-owned process scale.

eval_builtin! {
    contract: "bcsub",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
