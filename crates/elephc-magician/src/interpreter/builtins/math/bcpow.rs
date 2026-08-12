//! Purpose:
//! Declares the Magician registry entry for `bcpow()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - The exponent remains a decimal string for exact integral validation.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bcpow",
    area: Math,
    params: [num, exponent, scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
