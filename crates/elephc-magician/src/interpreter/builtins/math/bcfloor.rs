//! Purpose:
//! Declares the Magician registry entry for `bcfloor()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Exact decimal floor returns a PHP string.

eval_builtin! {
    name: "bcfloor",
    area: Math,
    params: [num],
    direct: Bcmath,
    values: Bcmath,
}
