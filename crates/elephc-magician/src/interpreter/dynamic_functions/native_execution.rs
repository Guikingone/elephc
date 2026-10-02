//! Purpose:
//! Executes registered native functions and balances temporary argument arrays.
//!
//! Called from:
//! - Dynamic function dispatch after native signature binding.
//!
//! Key details:
//! - By-reference writeback and temporary runtime-cell ownership are handled together.

use super::*;

thread_local! {
    /// The eval contexts currently inside a call to a registered AOT function, innermost last.
    ///
    /// The generated invoker converts a boxed argument into a `callable` descriptor when the
    /// parameter is typed `callable`/`Closure`, and an interpreted callback that is not a Closure
    /// object (a function name, an array callable) has no owner context of its own. The context
    /// that made the call is the one php would resolve that name in.
    static NATIVE_CALL_CONTEXTS: std::cell::RefCell<Vec<usize>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Returns the eval context of the innermost interpreted call into a registered AOT function.
pub(crate) fn current_native_call_context() -> *mut ElephcEvalContext {
    NATIVE_CALL_CONTEXTS.with(|contexts| {
        contexts
            .borrow()
            .last()
            .map_or(std::ptr::null_mut(), |context| *context as *mut ElephcEvalContext)
    })
}

/// Evaluates a registered AOT function through its descriptor-compatible invoker.
///
/// `name` is the PHP name the call was written with (`{closure}` for an anonymous function). The
/// call is recorded as a live frame while the compiled body runs, so a compiled callee that reads
/// `debug_backtrace()` or builds a throwable sees the interpreted call site that entered it.
pub(in crate::interpreter) fn eval_native_function(
    name: &str,
    function: NativeFunction,
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    caller_scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let evaluated_args =
        eval_native_function_call_args(&function, args, context, caller_scope, values)?;
    if !crate::context::native_call_frames_enabled() {
        return eval_native_function_with_values(function, evaluated_args, context, values);
    }
    let frame_args = evaluated_args.values.clone();
    let frame = EvalCallFrame::function(name, Some(frame_args), context);
    context.push_call_frame(frame);
    let result = eval_native_function_with_values(function, evaluated_args, context, values);
    context.pop_call_frame();
    result
}

/// Invokes a registered AOT function after its arguments have been bound and staged.
pub(in crate::interpreter) fn eval_native_function_with_values(
    function: NativeFunction,
    bound_args: BoundNativeFunctionArgs,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !function.bridge_supported() {
        return Err(EvalStatus::RuntimeFatal);
    }
    let variadic_index = native_function_variadic_index(&function);
    if variadic_index.is_none() && bound_args.values.len() != function.param_count() {
        return Err(EvalStatus::RuntimeFatal);
    }
    if let Some(variadic_index) = variadic_index {
        if bound_args.values.len() < function.required_param_count().min(variadic_index) {
            return Err(EvalStatus::RuntimeFatal);
        }
    }
    let arg_array = match build_native_function_arg_array(&bound_args, values) {
        Ok(arg_array) => arg_array,
        Err(status) => {
            cleanup_native_function_ref_args(&bound_args, values)?;
            release_native_function_owned_temporaries(&bound_args, None, context, values)?;
            return Err(status);
        }
    };
    // Copy the hooks and scope pointer before invoking generated code: nested
    // native/eval calls may reenter the context. Synchronize before observing
    // the native result or performing reference writeback, including errors.
    let global_sync = context.native_global_sync.zip(context.global_scope_ptr());
    if let Some((hooks, scope)) = global_sync {
        unsafe { (hooks.eval_to_native)(scope) };
    }
    NATIVE_CALL_CONTEXTS.with(|contexts| {
        contexts
            .borrow_mut()
            .push(context as *mut ElephcEvalContext as usize)
    });
    let result = unsafe { function.call(arg_array) };
    NATIVE_CALL_CONTEXTS.with(|contexts| {
        contexts.borrow_mut().pop();
    });
    if let Some((hooks, scope)) = global_sync {
        unsafe { (hooks.native_to_eval)(scope) };
    }
    if let Err(status) = values.release(arg_array) {
        cleanup_native_function_ref_args(&bound_args, values)?;
        release_native_function_owned_temporaries(&bound_args, None, context, values)?;
        return Err(status);
    }
    let result = values.native_call_result(result);
    let writeback = write_back_native_function_ref_args(&bound_args, context, values);
    let released = release_native_function_owned_temporaries(
        &bound_args,
        result.as_ref().ok().copied(),
        context,
        values,
    );
    match (result, writeback, released) {
        (Err(status), _, _) | (_, Err(status), _) | (_, _, Err(status)) => Err(status),
        (Ok(result), Ok(()), Ok(())) => {
            eval_declared_native_return_value(function.return_type(), None, None, result, context, values)
        }
    }
}

/// Releases the argument cells the caller still owed once the native invoker has returned.
///
/// A spread element arrives from `array_get` with a reference the interpreter owns
/// (`src/codegen_support/runtime/objects/mixed_array_get.rs`) while the invoker only borrows it:
/// the temporary argument array took its own reference and gave it back above. The invoked
/// function is allowed to hand one of those very cells back as its result, which transfers the
/// reference to the caller instead, so a returned handle is never released here.
fn release_native_function_owned_temporaries(
    bound_args: &BoundNativeFunctionArgs,
    returned: Option<RuntimeCellHandle>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for value in &bound_args.owned_temporaries {
        if Some(*value) == returned {
            continue;
        }
        eval_release_value(context, values, *value)?;
    }
    Ok(())
}

/// Builds the positional runtime array passed to descriptor-compatible native invokers.
fn build_native_function_arg_array(
    bound_args: &BoundNativeFunctionArgs,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let arg_array = values.array_new(bound_args.values.len())?;
    for (index, value) in bound_args.values.iter().copied().enumerate() {
        let index = match values.int(index as i64) {
            Ok(index) => index,
            Err(status) => {
                values.release(arg_array)?;
                return Err(status);
            }
        };
        if let Err(status) = values.array_set(arg_array, index, value) {
            values.release(arg_array)?;
            return Err(status);
        }
    }
    Ok(arg_array)
}

/// Releases retained raw native-function by-reference staging slots without writeback.
fn cleanup_native_function_ref_args(
    bound_args: &BoundNativeFunctionArgs,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for ref_slot in &bound_args.ref_slots {
        match ref_slot {
            BoundNativeFunctionRefSlot::RawString { original, slot, .. } => {
                let words = **slot;
                values.release_raw_string_words(words[0], words[1])?;
                if words[0] != original[0] {
                    values.release_raw_string_words(original[0], original[1])?;
                }
            }
            BoundNativeFunctionRefSlot::OwnedRawWord { original, slot, .. } => {
                let word = **slot;
                values.release_raw_heap_word(word)?;
                if word != *original {
                    values.release_raw_heap_word(*original)?;
                }
            }
            BoundNativeFunctionRefSlot::Mixed { slot, .. } => {
                values.release(**slot)?;
            }
            BoundNativeFunctionRefSlot::RawWord { .. } => {}
        }
    }
    Ok(())
}

/// Writes changed staged native-function by-reference slots back to eval caller targets.
fn write_back_native_function_ref_args(
    bound_args: &BoundNativeFunctionArgs,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for ref_slot in &bound_args.ref_slots {
        match ref_slot {
            BoundNativeFunctionRefSlot::Mixed {
                original,
                slot,
                target,
            } => {
                let value = **slot;
                if value == *original {
                    values.release(value)?;
                    continue;
                }
                let Some(target) = target else {
                    values.release(value)?;
                    continue;
                };
                let current = match eval_reference_target_value(target, context, values) {
                    Ok(current) => current,
                    Err(status) => {
                        values.release(value)?;
                        return Err(status);
                    }
                };
                if current == value {
                    values.release(value)?;
                    continue;
                }
                if let Err(status) = eval_write_direct_ref_target(
                    target,
                    value,
                    context,
                    values,
                    Some(ScopeCellOwnership::Owned),
                ) {
                    values.release(value)?;
                    return Err(status);
                }
            }
            BoundNativeFunctionRefSlot::RawWord {
                tag,
                original,
                slot,
                target,
            } => {
                let word = **slot;
                if word == *original {
                    continue;
                }
                let Some(target) = target else {
                    continue;
                };
                let value = raw_slot_word_value(*tag, word, values)?;
                eval_write_direct_ref_target(
                    target,
                    value,
                    context,
                    values,
                    Some(ScopeCellOwnership::Owned),
                )?;
            }
            BoundNativeFunctionRefSlot::RawString {
                original,
                slot,
                target,
            } => {
                let words = **slot;
                if target.is_none() {
                    values.release_raw_string_words(words[0], words[1])?;
                    if words[0] != original[0] {
                        values.release_raw_string_words(original[0], original[1])?;
                    }
                    continue;
                }
                let Some(target) = target else {
                    return Err(EvalStatus::RuntimeFatal);
                };
                if words == *original {
                    values.release_raw_string_words(words[0], words[1])?;
                    continue;
                }
                let value = values.raw_string_value(words[0], words[1]);
                values.release_raw_string_words(words[0], words[1])?;
                if words[0] != original[0] {
                    values.release_raw_string_words(original[0], original[1])?;
                }
                let value = value?;
                eval_write_direct_ref_target(
                    target,
                    value,
                    context,
                    values,
                    Some(ScopeCellOwnership::Owned),
                )?;
            }
            BoundNativeFunctionRefSlot::OwnedRawWord {
                original,
                slot,
                target,
            } => {
                let word = **slot;
                if target.is_none() {
                    values.release_raw_heap_word(word)?;
                    if word != *original {
                        values.release_raw_heap_word(*original)?;
                    }
                    continue;
                }
                let Some(target) = target else {
                    return Err(EvalStatus::RuntimeFatal);
                };
                if word == *original {
                    values.release_raw_heap_word(word)?;
                    continue;
                }
                let value = values.raw_heap_word_value(word);
                values.release_raw_heap_word(word)?;
                values.release_raw_heap_word(*original)?;
                let value = value?;
                eval_write_direct_ref_target(
                    target,
                    value,
                    context,
                    values,
                    Some(ScopeCellOwnership::Owned),
                )?;
            }
        }
    }
    Ok(())
}
