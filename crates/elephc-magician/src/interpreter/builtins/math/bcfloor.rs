//! Purpose:
//! Declares the Magician registry entry for `bcfloor()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Exact decimal floor returns a PHP string.

eval_builtin! {
    contract: "bcfloor",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
