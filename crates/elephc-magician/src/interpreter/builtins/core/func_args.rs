//! Purpose:
//! Implements PHP's argument-introspection trio -- `func_num_args()`, `func_get_args()` and
//! `func_get_arg()` -- for the eval interpreter.
//!
//! Called from:
//! - `crate::interpreter::expressions::calls::eval_call`, ahead of the undefined-function refusal.
//!
//! Key details:
//! - These are NOT shared-catalog builtins and must not become any. The compiler answers them by
//!   DESUGARING at compile time (`src/func_args/`), rewriting each call into plain PHP before
//!   lowering, so there is no AOT registry binding for them and adding a contract would demand
//!   one that cannot exist. The interpreter never runs that pass, which is the whole gap.
//! - The values come from the innermost CALL FRAME, which already records what the caller passed,
//!   plus the running scope for anything the callee declared a parameter name for.
//! - Both halves are needed. `php -n` 8.5.6 prints `["changed"]` for
//!   `function f($a) { $a = "changed"; echo json_encode(func_get_args()); } f("original");` --
//!   php reports the CURRENT value of a declared parameter, not the one that arrived -- while an
//!   argument beyond the declared list has no variable to read and can only come from the frame.

use super::super::super::*;

/// Names the interpreter answers itself rather than looking up as functions.
pub(in crate::interpreter) fn eval_is_func_args_intrinsic(name: &str) -> bool {
    matches!(
        name.trim_start_matches('\\').to_ascii_lowercase().as_str(),
        "func_num_args" | "func_get_args" | "func_get_arg"
    )
}

/// Evaluates one of the argument-introspection trio.
pub(in crate::interpreter) fn eval_func_args_intrinsic(
    name: &str,
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let lowered = name.trim_start_matches('\\').to_ascii_lowercase();
    let current = eval_current_call_arguments(context, scope)?;
    match lowered.as_str() {
        "func_num_args" => {
            if !args.is_empty() {
                return Err(EvalStatus::RuntimeFatal);
            }
            values.int(current.len() as i64)
        }
        "func_get_args" => {
            if !args.is_empty() {
                return Err(EvalStatus::RuntimeFatal);
            }
            let mut array = values.array_new(current.len())?;
            for (index, value) in current.into_iter().enumerate() {
                let key = values.int(index as i64)?;
                let value = values.retain(value)?;
                array = values.array_set(array, key, value)?;
            }
            // php returns a LIST: `json_encode(func_get_args())` is `[1,2,3]`, never
            // `{"0":1,...}`. Built here from the frame's argument cells, the array carries the
            // right keys and the right values -- `var_dump` is byte-identical to php's -- but
            // `json_encode` still renders it as an object, while the same `array_new` +
            // `array_set` sequence in the array-literal path does not. The values are the
            // difference: a literal stores freshly evaluated cells, this stores the caller's.
            // Passing it through the same repack `array_values()` performs settles it, which is
            // measured rather than reasoned: `json_encode(array_values($a))` already printed
            // `[7,8]` for the very array whose direct encoding printed `{"0":7,"1":8}`.
            let packed = super::super::array::array_values::eval_array_values_result(
                array, values,
            )?;
            values.release(array)?;
            Ok(packed)
        }
        "func_get_arg" => {
            let [index] = args else {
                return Err(EvalStatus::RuntimeFatal);
            };
            if index.name().is_some() || index.is_spread() {
                return Err(EvalStatus::RuntimeFatal);
            }
            let index = eval_expr(index.value(), context, scope, values)?;
            let position = eval_int_value(index, values)?;
            values.release(index)?;
            if position < 0 {
                return Err(EvalStatus::RuntimeFatal);
            }
            let Some(value) = current.get(position as usize) else {
                return Err(EvalStatus::RuntimeFatal);
            };
            values.retain(*value)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Returns the arguments the running activation was called with, as php reports them now.
///
/// A declared parameter is read from the SCOPE, so a reassignment is visible; anything past the
/// declared list is read from the frame, where it is the only record of the value.
fn eval_current_call_arguments(
    context: &ElephcEvalContext,
    scope: &ElephcEvalScope,
) -> Result<Vec<RuntimeCellHandle>, EvalStatus> {
    let Some(passed) = context.with_call_frames(|frames| {
        frames
            .last()
            .map(|frame| frame.args.clone().unwrap_or_default())
    }) else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let names = eval_current_parameter_names(context);
    let mut current = Vec::with_capacity(passed.len());
    for (index, value) in passed.iter().enumerate() {
        let live = names
            .get(index)
            .and_then(|name| visible_scope_cell(context, scope, name));
        current.push(live.unwrap_or(*value));
    }
    Ok(current)
}

/// Returns the declared parameter names of the running function, method or closure.
fn eval_current_parameter_names(context: &ElephcEvalContext) -> Vec<String> {
    let Some(name) = context.current_function().map(str::to_string) else {
        return Vec::new();
    };
    if let Some(function) = context.function(&name) {
        return function.params().to_vec();
    }
    if let Some(closure) = context.closure(&name) {
        return closure.function().params().to_vec();
    }
    if let Some(class) = context.current_class_scope().map(str::to_string) {
        if let Some((_, method)) = context.class_method(&class, &name) {
            return method.params().to_vec();
        }
    }
    Vec::new()
}
