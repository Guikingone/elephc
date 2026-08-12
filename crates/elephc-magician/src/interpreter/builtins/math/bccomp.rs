//! Purpose:
//! Declares the Magician registry entry for `bccomp()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - The shared hook returns PHP integer `-1`, `0`, or `1`.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bccomp",
    area: Math,
    params: [num1, num2, scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
