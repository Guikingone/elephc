//! Purpose:
//! Declares the Magician registry entry for `bcround()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Precision defaults to zero and integer mode one is half away from zero.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bcround",
    area: Math,
    params: [
        num,
        precision = EvalBuiltinDefaultValue::Int(0),
        mode = EvalBuiltinDefaultValue::Int(1)
    ],
    direct: Bcmath,
    values: Bcmath,
}
