//! Purpose:
//! Declares the Magician registry entry for `bcadd()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Null scale selects the crate-owned process scale.

eval_builtin! {
    contract: "bcadd",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
