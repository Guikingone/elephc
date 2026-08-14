//! Purpose:
//! Declares the Magician registry entry for `bcpowmod()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Decimal operands stay strings until the crate validates integral values.

eval_builtin! {
    contract: "bcpowmod",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
