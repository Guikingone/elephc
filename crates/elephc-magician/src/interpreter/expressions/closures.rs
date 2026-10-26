//! Purpose:
//! Constructs opaque eval closures and attaches captures to their native object lifetime.
//! Releases creation-time owners after every successful or failed construction.
//!
//! Called from:
//! - `crate::interpreter::expressions::eval_expr()` and callable materialization.
//! - Closure binding, cloning, and Reflection dispatch.
//!
//! Key details:
//! - Capture and receiver edges must be installed together because the setter replaces its list.
//! - Context metadata borrows cells; runtime objects own the actual GC graph edges.

use super::*;

/// Materializes one eval closure literal whose runtime object owns its capture cells.
pub(super) fn eval_closure_expr(
    function: &EvalFunction,
    captures: &[crate::eval_ir::EvalClosureCapture],
    is_static: bool,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut bindings = Vec::with_capacity(captures.len());
    for capture in captures {
        match eval_closure_capture(capture, context, scope, values) {
            Ok(binding) => bindings.push(binding),
            Err(status) => {
                let _ = release_closure_capture_owners(&bindings, context, values);
                return Err(status);
            }
        }
    }
    let mut borrowed = bindings.clone();
    for binding in &mut borrowed { binding.borrow_value(); }
    let name = context.define_closure(EvalClosure::new(function.clone(), borrowed, is_static));
    let result = eval_closure_object_expr(EvalClosureObjectTarget::Named(name.clone()), context, values);
    if result.is_err() { context.discard_unowned_closure(&name); }
    let cleanup = release_closure_capture_owners(&bindings, context, values);
    context.discard_unowned_closure(&name);
    match (result, cleanup) {
        (Err(status), _) => Err(status),
        (Ok(object), Err(status)) => {
            let _ = eval_release_value(context, values, object);
            Err(status)
        }
        (Ok(object), Ok(())) => Ok(object),
    }
}

/// Releases every creation-time capture owner, including after a later capture fails.
fn release_closure_capture_owners(
    bindings: &[EvalClosureCaptureBinding],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let mut result = Ok(());
    for binding in bindings {
        let cleanup = release_expr_result(binding.value(), context, values);
        if result.is_ok() { result = cleanup; }
    }
    result
}

/// Attaches all capture and receiver edges together; the native edge setter replaces its list.
pub(in crate::interpreter) fn eval_attach_closure_target_children(
    object: RuntimeCellHandle,
    target: &mut EvalClosureObjectTarget,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let mut children = context.closure_target_captures(target);
    if let Some(receiver) = target.receiver_mut() { children.push(*receiver); }
    values.retain_object_children(object, &children)?;
    context.retain_closure_target_metadata(target);
    if let Some(receiver) = target.receiver_mut() { *receiver = receiver.borrowed(); }
    Ok(())
}

/// Materializes one PHP-visible `Closure` object for an eval callable target.
pub(in crate::interpreter) fn eval_closure_object_expr(
    mut target: EvalClosureObjectTarget,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let object = values.new_object("stdClass")?;
    let attached = values.object_identity(object).and_then(|identity| {
        eval_attach_closure_target_children(object, &mut target, context, values)?;
        Ok(identity)
    });
    let identity = match attached {
        Ok(identity) => identity,
        Err(status) => {
            let _ = values.release(object);
            return Err(status);
        }
    };
    context.register_closure_object_target(identity, target);
    crate::ffi::dynamic_destructors::register_dynamic_object_context(identity, context);
    Ok(object)
}

/// Evaluates one closure capture from the defining scope.
fn eval_closure_capture(
    capture: &crate::eval_ir::EvalClosureCapture,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalClosureCaptureBinding, EvalStatus> {
    if capture.by_ref() {
        if let Some(reference) = eval_persistent_variable_reference(capture.name(), context, scope, values)? {
            let reference = values.retain(reference)?;
            return Ok(EvalClosureCaptureBinding::new(capture.name(), reference, Some(EvalReferenceTarget::Cell { cell: reference })));
        }
        let expr = EvalExpr::LoadVar(capture.name().to_string());
        let (value, target) = eval_call_arg_value(&expr, context, scope, values)?;
        return Ok(EvalClosureCaptureBinding::new(
            capture.name(),
            value,
            target,
        ));
    }
    let value = if let Some(value) = visible_scope_cell(context, scope, capture.name()) {
        if values.is_reference(value)? { values.copy_value(value)? } else { values.retain(value)? }
    } else {
        values.null()?
    };
    Ok(EvalClosureCaptureBinding::new(capture.name(), value, None))
}
