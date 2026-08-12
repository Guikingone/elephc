//! Purpose:
//! Declares the Magician registry entry for `bcscale()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Missing or null scale reads state; an integer sets it and returns the previous value.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bcscale",
    area: Math,
    params: [scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
