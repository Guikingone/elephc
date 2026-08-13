//! Purpose:
//! Declares the Magician registry entry for `bccomp()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - The shared hook returns PHP integer `-1`, `0`, or `1`.

eval_builtin! {
    contract: "bccomp",
    area: Math,
    direct: Bcmath,
    values: Bcmath,
}
