//! Purpose:
//! Declares the Magician registry entry for `bcdivmod()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - The shared hook returns a two-element quotient/remainder array.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bcdivmod",
    area: Math,
    params: [num1, num2, scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
