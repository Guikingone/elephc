//! Purpose:
//! Declares the Magician registry entry for `bcdiv()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Division failures become catchable `DivisionByZeroError` objects.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bcdiv",
    area: Math,
    params: [num1, num2, scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
