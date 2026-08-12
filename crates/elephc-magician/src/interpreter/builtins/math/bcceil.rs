//! Purpose:
//! Declares the Magician registry entry for `bcceil()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Exact decimal ceiling returns a PHP string.

eval_builtin! {
    name: "bcceil",
    area: Math,
    params: [num],
    direct: Bcmath,
    values: Bcmath,
}
