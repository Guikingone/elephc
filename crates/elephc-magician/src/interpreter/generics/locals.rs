//! Purpose:
//! Enforces substituted generic contracts on explicitly typed local storage.
//!
//! Called from:
//! - Variable assignment and array mutation statements.
//!
//! Key details:
//! - Validation happens after operand evaluation and before storage mutation.

use super::*;

/// Checks an initialized or reassigned typed local without coercing its stored value.
pub(in crate::interpreter) fn validate_generic_local(declared: &EvalParameterType, name: &str, value: RuntimeCellHandle, context: &mut ElephcEvalContext, values: &mut impl RuntimeValueOps) -> Result<(), EvalStatus> {
    if generic_value_accepts(&generic_type(declared), value, context, values)? { return Ok(()); }
    eval_throw_type_error(&format!("Typed local ${name} expects {}", generic_type(declared)), context, values)
}

/// Checks an array write against its key and element contracts before mutating storage.
pub(in crate::interpreter) fn validate_generic_local_element(declared: Option<&EvalParameterType>, index: RuntimeCellHandle, value: RuntimeCellHandle, context: &mut ElephcEvalContext, values: &mut impl RuntimeValueOps) -> Result<(), EvalStatus> {
    let Some(declared) = declared else { return Ok(()); };
    let ty = generic_type(declared);
    if ty == Type::atom("array") || ty == Type::atom("mixed") { return Ok(()); }
    if let Type::Apply(name, args) = &ty {
        if name == "array" && !args.is_empty() {
            let element = generic_value_accepts(args.last().unwrap(), value, context, values)?;
            let key = args.len() != 2 || generic_value_accepts(&args[0], index, context, values)?;
            if element && key { return Ok(()); }
        }
    }
    eval_throw_type_error(&format!("Array write violates typed local {ty}"), context, values)
}
