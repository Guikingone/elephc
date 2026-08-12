//! Purpose:
//! Declares the Magician registry entry for `bcpowmod()`.
//!
//! Called from:
//! - The eval builtin inventory and shared BCMath hook.
//!
//! Key details:
//! - Decimal operands stay strings until the crate validates integral values.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "bcpowmod",
    area: Math,
    params: [num, exponent, modulus, scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
