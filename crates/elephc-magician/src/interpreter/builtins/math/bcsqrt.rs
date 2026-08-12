//! Purpose:
//! Declares the Magician registry entry for `bcsqrt()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Null scale selects the crate-owned process scale.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bcsqrt",
    area: Math,
    params: [num, scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
