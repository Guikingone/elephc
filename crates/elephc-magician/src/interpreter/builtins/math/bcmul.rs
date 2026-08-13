//! Purpose:
//! Declares the Magician registry entry for `bcmul()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Null scale selects the crate-owned process scale.

eval_builtin! {
    contract: "bcmul",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
