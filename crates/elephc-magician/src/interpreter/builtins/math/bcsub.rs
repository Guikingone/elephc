//! Purpose:
//! Declares the Magician registry entry for `bcsub()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Null scale selects the crate-owned process scale.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bcsub",
    area: Math,
    params: [num1, num2, scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
