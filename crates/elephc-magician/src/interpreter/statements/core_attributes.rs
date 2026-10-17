//! Purpose:
//! Applies shared Core attribute targets and NoDiscard diagnostics to interpreted declarations.
//!
//! Called from:
//! - Declaration validators and expression-statement execution.
//!
//! Key details:
//! - DelayedTargetValidation affects targets only; repeated non-repeatable markers still fail.
//! - Warning handlers run while the discarded return value remains owned by its caller.

use super::*;

/// Validates predefined attribute targets using the same masks as the native checker.
pub(super) fn validate_eval_core_attribute_targets(attributes: &[EvalAttribute], target: i64) -> Result<(), EvalStatus> {
    let delayed = eval_attributes_have_global_builtin_attribute(attributes, "DelayedTargetValidation");
    for (index, attribute) in attributes.iter().enumerate() {
        let Some(flags) = elephc_builtin_contract::core_attribute_flags(attribute.name()) else { continue };
        if flags & 64 == 0 && attributes[..index].iter().any(|other| other.name().eq_ignore_ascii_case(attribute.name())) {
            return Err(EvalStatus::RuntimeFatal);
        }
        if !delayed && flags & target == 0 { return Err(EvalStatus::RuntimeFatal); }
    }
    Ok(())
}

/// Emits the PHP warning for an ignored result from an interpreted marked declaration.
pub(super) fn warn_eval_discarded_call(
    expression: &EvalExpr,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let selected = match expression {
        EvalExpr::Call { name, .. } => context.function(name).map(|function| (name.clone(), function.attributes().to_vec())),
        EvalExpr::NamespacedCall { name, fallback_name, .. } => context.function(name).or_else(|| context.function(fallback_name))
            .map(|function| (function.name().to_string(), function.attributes().to_vec())),
        EvalExpr::StaticMethodCall { class_name, method, .. } => context.class_method(class_name, method)
            .map(|(_, method)| (format!("{class_name}::{}", method.name()), method.attributes().to_vec())),
        _ => None,
    };
    if let Some((name, attributes)) = selected {
        if eval_attributes_have_global_builtin_attribute(&attributes, "NoDiscard") {
            values.warning(&format!("Warning: The return value of {name}() should either be used or explicitly ignored"))?;
        }
    }
    Ok(())
}
