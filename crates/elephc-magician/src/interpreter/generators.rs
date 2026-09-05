//! Purpose:
//! Implements PHP generators for interpreted code: the `Generator` object a function containing
//! `yield` returns, its resumption protocol, and `yield from` delegation.
//!
//! Called from:
//! - `crate::interpreter::dynamic_functions` when a called body contains a yield.
//! - `crate::interpreter::statements::method_dispatch` for the Generator methods.
//! - `crate::interpreter::statements::loop_statements` for `foreach` over a generator.
//!
//! Key details:
//! - ARCHITECTURE: a per-generator LINEAR STEP LIST (see `lowering`) plus a frame holding a
//!   program counter, the generator's own scope and its iteration state. Resuming is then just
//!   continuing at the stored index. A resumable tree walker was rejected because it must reify
//!   every loop's position anyway and then re-descend the tree on every resume; OS threads were
//!   rejected because the runtime handles and scope pointers are not `Send` and a generator
//!   outlives the scope that created it.
//! - Only constructs CONTAINING a yield are flattened. Everything else stays a statement subtree
//!   run atomically by the ordinary evaluator, so this module owns control flow and nothing else.
//! - PHP does not run a generator's body until it is first asked for a value, so creation is
//!   just building the frame.

mod lowering;

use super::*;
use crate::context::{
    EvalGeneratorActivation, EvalGeneratorDelegate, EvalGeneratorFrame, EvalGeneratorMagicScope,
    EvalGeneratorState, EvalGeneratorStep,
};

pub(in crate::interpreter) use lowering::eval_body_is_generator;
use lowering::lower_generator_body;

/// Builds the `Generator` object that one call to a generator function returns.
pub(in crate::interpreter) fn eval_generator_new(
    body: &[EvalStmt],
    scope: ElephcEvalScope,
    activation: EvalGeneratorActivation,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let program = lower_generator_body(body)?;
    let foreach_slots = vec![None; program.foreach_slots];
    let object = values.new_object("Generator")?;
    let identity = values.object_identity(object)?;
    context.register_eval_generator(
        identity,
        EvalGeneratorFrame {
            program,
            activation,
            step: 0,
            scope: Box::new(scope),
            state: EvalGeneratorState::NotStarted,
            auto_key: 0,
            current_key: None,
            current_value: None,
            return_value: None,
            foreach_slots,
            delegate: None,
            pending_send_slot: None,
            advanced: false,
        },
    );
    Ok(object)
}

/// Builds the activation of a generator produced by a plain function.
///
/// A plain function pushes no class scope and no magic frame, so neither does its generator.
pub(in crate::interpreter) fn eval_plain_function_activation(
    function_name: &str,
) -> EvalGeneratorActivation {
    EvalGeneratorActivation {
        function_name: function_name.to_string(),
        class_scope: None,
        called_class: None,
        magic: None,
    }
}

/// Builds the activation of a generator produced by a closure.
///
/// A bound closure pushes the class scopes but still no magic frame, exactly as the ordinary
/// closure call path does.
pub(in crate::interpreter) fn eval_closure_activation(
    function_name: &str,
    class_scope: Option<String>,
    called_class: Option<String>,
) -> EvalGeneratorActivation {
    EvalGeneratorActivation {
        function_name: function_name.to_string(),
        class_scope,
        called_class,
        magic: None,
    }
}

/// Builds the activation of a generator produced by a class method.
///
/// This is the case the activation exists for: the resumer's class scope must not leak into the
/// body, so the declaring class, the late-static-bound class and the magic frame the ordinary
/// method path pushes are all recorded and re-pushed on every resumption.
pub(in crate::interpreter) fn eval_method_activation(
    qualified_method_name: String,
    class_name: &str,
    called_class_name: &str,
    method: &EvalClassMethod,
) -> EvalGeneratorActivation {
    EvalGeneratorActivation {
        function_name: qualified_method_name,
        class_scope: Some(class_name.to_string()),
        called_class: Some(called_class_name.to_string()),
        magic: Some(EvalGeneratorMagicScope {
            function_name: method.magic_function_name().to_string(),
            method_name: method.magic_method_name(class_name),
            class_name: Some(class_name.trim_start_matches('\\').to_string()),
            trait_name: method
                .trait_origin()
                .map(|trait_name| trait_name.trim_start_matches('\\').to_string()),
        }),
    }
}

/// Runs the generator up to its first yield when it has not started yet.
fn eval_generator_prime(
    identity: u64,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let not_started = context
        .eval_generator(identity, |frame| frame.state == EvalGeneratorState::NotStarted)
        .unwrap_or(false);
    if not_started {
        eval_generator_step(identity, None, false, context, values)?;
    }
    Ok(())
}

/// Advances one generator, optionally delivering a value to the yield it is suspended on.
///
/// The frame is taken out of the context for the duration, because the body it runs needs the
/// context mutably and the frame lives inside it.
fn eval_generator_step(
    identity: u64,
    sent: Option<RuntimeCellHandle>,
    resuming: bool,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let Some(mut frame) = context.take_eval_generator(identity) else {
        return Err(EvalStatus::RuntimeFatal);
    };
    eval_generator_enter_activation(&frame, context);
    let result = eval_generator_run(&mut frame, sent, resuming, context, values);
    eval_generator_leave_activation(&frame, context);
    context.restore_eval_generator(identity, frame);
    result
}

/// Re-establishes the generator's own naming scope for the duration of one resumption.
///
/// The caller that resumes a generator is arbitrary, so the body must not inherit the resumer's
/// class scope: `self::`, `static::`, `__CLASS__` and private-member access inside a generator
/// method all read these stacks.
fn eval_generator_enter_activation(frame: &EvalGeneratorFrame, context: &mut ElephcEvalContext) {
    let activation = &frame.activation;
    context.push_function(activation.function_name.clone());
    if let Some(class_scope) = &activation.class_scope {
        context.push_class_scope(class_scope.clone());
    }
    if let Some(called_class) = &activation.called_class {
        context.push_called_class_scope(called_class.clone());
    }
    if let Some(magic) = &activation.magic {
        context.push_callable_magic_scope(
            &magic.function_name,
            &magic.method_name,
            magic.class_name.as_deref(),
            magic.trait_name.as_deref(),
        );
    }
}

/// Undoes `eval_generator_enter_activation` in the reverse order.
fn eval_generator_leave_activation(frame: &EvalGeneratorFrame, context: &mut ElephcEvalContext) {
    let activation = &frame.activation;
    if activation.magic.is_some() {
        context.pop_magic_scope();
    }
    if activation.called_class.is_some() {
        context.pop_called_class_scope();
    }
    if activation.class_scope.is_some() {
        context.pop_class_scope();
    }
    context.pop_function();
}

/// Executes steps until the generator suspends on a yield or finishes.
fn eval_generator_run(
    frame: &mut EvalGeneratorFrame,
    sent: Option<RuntimeCellHandle>,
    resuming: bool,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if frame.state == EvalGeneratorState::Finished {
        return Ok(());
    }
    if resuming {
        eval_generator_deliver_sent(frame, sent, values)?;
        if frame.delegate.is_some() {
            eval_generator_advance_delegate(frame, sent, context, values)?;
            if eval_generator_produce_from_delegate(frame, context, values)? {
                return Ok(());
            }
            eval_generator_end_delegation(frame, context, values)?;
        }
    }
    frame.state = EvalGeneratorState::Suspended;
    loop {
        let Some(step) = frame.program.steps.get(frame.step).cloned() else {
            return eval_generator_finish(frame, None, values);
        };
        frame.step += 1;
        match step {
            EvalGeneratorStep::Run(body) => {
                match execute_statements(&body, context, &mut frame.scope, values)? {
                    EvalControl::None => {}
                    EvalControl::Return(value) => {
                        return eval_generator_finish(frame, Some(value), values)
                    }
                    EvalControl::ReturnVoid => return eval_generator_finish(frame, None, values),
                    EvalControl::Throw(value) => {
                        frame.state = EvalGeneratorState::Finished;
                        context.set_pending_throw(value);
                        return Err(EvalStatus::UncaughtThrowable);
                    }
                    // A break or continue reaching here escaped the chunk it was batched into,
                    // which the lowering prevents by lowering such a statement itself.
                    _ => return Err(EvalStatus::UnsupportedConstruct),
                }
            }
            EvalGeneratorStep::Yield { key, value, into } => {
                let value = eval_expr(&value, context, &mut frame.scope, values)?;
                let key = match key {
                    Some(key) => eval_expr(&key, context, &mut frame.scope, values)?,
                    None => {
                        let key = values.int(frame.auto_key)?;
                        frame.auto_key += 1;
                        key
                    }
                };
                eval_generator_set_current(frame, key, value, values)?;
                frame.pending_send_slot = into;
                return Ok(());
            }
            EvalGeneratorStep::YieldFrom { source, into } => {
                let source = eval_expr(&source, context, &mut frame.scope, values)?;
                frame.pending_send_slot = into;
                eval_generator_begin_delegation(frame, source, context, values)?;
                if eval_generator_produce_from_delegate(frame, context, values)? {
                    return Ok(());
                }
                eval_generator_end_delegation(frame, context, values)?;
            }
            EvalGeneratorStep::JumpIfFalse { condition, target } => {
                let condition = eval_expr(&condition, context, &mut frame.scope, values)?;
                if !values.truthy(condition)? {
                    frame.step = target;
                }
            }
            EvalGeneratorStep::Jump(target) => frame.step = target,
            EvalGeneratorStep::Return(value) => {
                let value = match value {
                    Some(expr) => Some(eval_expr(&expr, context, &mut frame.scope, values)?),
                    None => None,
                };
                return eval_generator_finish(frame, value, values);
            }
            EvalGeneratorStep::ForeachInit { subject, slot } => {
                let subject = eval_expr(&subject, context, &mut frame.scope, values)?;
                frame.foreach_slots[slot] = Some((subject, 0));
            }
            EvalGeneratorStep::ForeachNext {
                slot,
                key_name,
                value_name,
                exit,
            } => {
                let Some((array, position)) = frame.foreach_slots[slot] else {
                    return Err(EvalStatus::RuntimeFatal);
                };
                if position >= values.array_len(array)? {
                    frame.step = exit;
                    continue;
                }
                let key = values.array_iter_key(array, position)?;
                let value = values.array_get(array, key)?;
                frame.foreach_slots[slot] = Some((array, position + 1));
                match key_name {
                    Some(key_name) => {
                        if let Some(replaced) =
                            frame.scope.set(key_name, key, ScopeCellOwnership::Owned)
                        {
                            values.release(replaced)?;
                        }
                    }
                    None => values.release(key)?,
                }
                if let Some(replaced) =
                    frame.scope.set(value_name, value, ScopeCellOwnership::Owned)
                {
                    values.release(replaced)?;
                }
            }
        }
    }
}

/// Stores what `send()` passed in under the slot the suspended yield named.
fn eval_generator_deliver_sent(
    frame: &mut EvalGeneratorFrame,
    sent: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let Some(name) = frame.pending_send_slot.take() else {
        return Ok(());
    };
    let value = match sent {
        Some(sent) => values.retain(sent)?,
        None => values.null()?,
    };
    if let Some(replaced) = frame.scope.set(name, value, ScopeCellOwnership::Owned) {
        values.release(replaced)?;
    }
    Ok(())
}

/// Starts delegating to an array or another generator.
fn eval_generator_begin_delegation(
    frame: &mut EvalGeneratorFrame,
    source: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    match values.type_tag(source)? {
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => {
            frame.delegate = Some(EvalGeneratorDelegate::Array {
                array: source,
                position: 0,
            });
            Ok(())
        }
        EVAL_TAG_OBJECT => {
            let identity = values.object_identity(source)?;
            if !context.has_eval_generator(identity) {
                return Err(EvalStatus::UnsupportedConstruct);
            }
            eval_generator_prime(identity, context, values)?;
            frame.delegate = Some(EvalGeneratorDelegate::Generator { identity });
            Ok(())
        }
        _ => eval_throw_error(
            "Can use \"yield from\" only with arrays and Traversables",
            context,
            values,
        ),
    }
}

/// Produces the delegate's current pair, or reports that it is exhausted.
///
/// The delegated keys are the INNER ones and the outer generator's auto-increment counter is
/// left alone: `php -n` 8.5.6 gives `yield 0; yield from [10, 20]; yield 99;` the keys 0, 0, 1, 1.
fn eval_generator_produce_from_delegate(
    frame: &mut EvalGeneratorFrame,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    match frame.delegate {
        None => Ok(false),
        Some(EvalGeneratorDelegate::Array { array, position }) => {
            if position >= values.array_len(array)? {
                return Ok(false);
            }
            let key = values.array_iter_key(array, position)?;
            let value = values.array_get(array, key)?;
            eval_generator_set_current(frame, key, value, values)?;
            Ok(true)
        }
        Some(EvalGeneratorDelegate::Generator { identity }) => {
            let valid = context
                .eval_generator(identity, |inner| {
                    inner.state != EvalGeneratorState::Finished && inner.current_value.is_some()
                })
                .unwrap_or(false);
            if !valid {
                return Ok(false);
            }
            let Some((key, value)) = context.eval_generator(identity, |inner| {
                inner.current_key.zip(inner.current_value)
            }).flatten() else {
                return Ok(false);
            };
            let key = values.retain(key)?;
            let value = values.retain(value)?;
            eval_generator_set_current(frame, key, value, values)?;
            Ok(true)
        }
    }
}

/// Moves the delegate on by one before its next value is read.
fn eval_generator_advance_delegate(
    frame: &mut EvalGeneratorFrame,
    sent: Option<RuntimeCellHandle>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    match frame.delegate {
        None => Ok(()),
        Some(EvalGeneratorDelegate::Array {
            array,
            ref mut position,
        }) => {
            let _ = array;
            *position += 1;
            Ok(())
        }
        Some(EvalGeneratorDelegate::Generator { identity }) => {
            eval_generator_step(identity, sent, true, context, values)
        }
    }
}

/// Clears an exhausted delegation and hands its return value to the waiting `into` slot.
fn eval_generator_end_delegation(
    frame: &mut EvalGeneratorFrame,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let delegate = frame.delegate.take();
    let produced = match delegate {
        Some(EvalGeneratorDelegate::Array { array, .. }) => {
            values.release(array)?;
            None
        }
        Some(EvalGeneratorDelegate::Generator { identity }) => context
            .eval_generator(identity, |inner| inner.return_value)
            .flatten(),
        None => None,
    };
    let Some(name) = frame.pending_send_slot.take() else {
        return Ok(());
    };
    let value = match produced {
        Some(value) => values.retain(value)?,
        None => values.null()?,
    };
    if let Some(replaced) = frame.scope.set(name, value, ScopeCellOwnership::Owned) {
        values.release(replaced)?;
    }
    Ok(())
}

/// Marks the generator finished and stores what `getReturn()` will hand back.
fn eval_generator_finish(
    frame: &mut EvalGeneratorFrame,
    value: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    frame.state = EvalGeneratorState::Finished;
    eval_generator_clear_current(frame, values)?;
    frame.return_value = value;
    Ok(())
}

/// Replaces the currently yielded pair, releasing the previous one.
fn eval_generator_set_current(
    frame: &mut EvalGeneratorFrame,
    key: RuntimeCellHandle,
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    eval_generator_clear_current(frame, values)?;
    frame.current_key = Some(key);
    frame.current_value = Some(value);
    Ok(())
}

/// Drops the currently yielded pair.
fn eval_generator_clear_current(
    frame: &mut EvalGeneratorFrame,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if let Some(key) = frame.current_key.take() {
        values.release(key)?;
    }
    if let Some(value) = frame.current_value.take() {
        values.release(value)?;
    }
    Ok(())
}

/// Dispatches the `Generator` methods PHP exposes.
pub(in crate::interpreter) fn eval_generator_method_result(
    identity: u64,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<RuntimeCellHandle>, EvalStatus> {
    if !context.has_eval_generator(identity) {
        return Ok(None);
    }
    match method_name.to_ascii_lowercase().as_str() {
        "current" => {
            eval_generator_prime(identity, context, values)?;
            match context
                .eval_generator(identity, |frame| frame.current_value)
                .flatten()
            {
                Some(value) => values.retain(value).map(Some),
                None => values.null().map(Some),
            }
        }
        "key" => {
            eval_generator_prime(identity, context, values)?;
            match context
                .eval_generator(identity, |frame| frame.current_key)
                .flatten()
            {
                Some(key) => values.retain(key).map(Some),
                None => values.null().map(Some),
            }
        }
        "valid" => {
            eval_generator_prime(identity, context, values)?;
            let valid = context
                .eval_generator(identity, |frame| frame.current_value.is_some())
                .unwrap_or(false);
            values.bool_value(valid).map(Some)
        }
        "next" => {
            eval_generator_prime(identity, context, values)?;
            eval_generator_mark_advanced(identity, context);
            eval_generator_step(identity, None, true, context, values)?;
            values.null().map(Some)
        }
        "send" => {
            let sent = evaluated_args.first().map(|arg| arg.value);
            // PHP runs the body up to the first yield before a `send()` can deliver anything,
            // and that first yield's value is discarded by the send itself.
            eval_generator_prime(identity, context, values)?;
            eval_generator_mark_advanced(identity, context);
            eval_generator_step(identity, sent, true, context, values)?;
            match context
                .eval_generator(identity, |frame| frame.current_value)
                .flatten()
            {
                Some(value) => values.retain(value).map(Some),
                None => values.null().map(Some),
            }
        }
        "rewind" => {
            eval_generator_prime(identity, context, values)?;
            let advanced = context
                .eval_generator(identity, |frame| frame.advanced)
                .unwrap_or(false);
            if advanced {
                return eval_throw_exception_message(
                    "Cannot rewind a generator that was already run",
                    context,
                    values,
                );
            }
            values.null().map(Some)
        }
        "getreturn" => {
            let finished = context
                .eval_generator(identity, |frame| frame.state == EvalGeneratorState::Finished)
                .unwrap_or(false);
            if !finished {
                return eval_throw_exception_message(
                    "Cannot get return value of a generator that hasn't returned",
                    context,
                    values,
                );
            }
            match context
                .eval_generator(identity, |frame| frame.return_value)
                .flatten()
            {
                Some(value) => values.retain(value).map(Some),
                None => values.null().map(Some),
            }
        }
        _ => Ok(None),
    }
}

/// Records that a generator has moved past its first yield, which `rewind()` refuses after.
fn eval_generator_mark_advanced(identity: u64, context: &mut ElephcEvalContext) {
    if let Some(mut frame) = context.take_eval_generator(identity) {
        frame.advanced = true;
        context.restore_eval_generator(identity, frame);
    }
}

/// Creates and schedules a plain `Exception`, which is what the generator errors are.
fn eval_throw_exception_message<T>(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let exception = values.new_object("Exception")?;
    let message = values.string(message)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}

/// Drives `foreach` over one generator, which PHP refuses once the generator has finished.
pub(in crate::interpreter) fn execute_foreach_generator_stmt(
    identity: u64,
    key_name: Option<&str>,
    value_name: &str,
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let already_closed = context
        .eval_generator(identity, |frame| {
            frame.state == EvalGeneratorState::Finished && frame.advanced
        })
        .unwrap_or(false);
    if already_closed {
        return eval_throw_exception_message(
            "Cannot traverse an already closed generator",
            context,
            values,
        );
    }
    eval_generator_prime(identity, context, values)?;
    loop {
        let Some((key, value)) = context
            .eval_generator(identity, |frame| frame.current_key.zip(frame.current_value))
            .flatten()
        else {
            return Ok(EvalControl::None);
        };
        let key = values.retain(key)?;
        let value = values.retain(value)?;
        match key_name {
            Some(key_name) => {
                for replaced in set_scope_cell(
                    context,
                    scope,
                    key_name.to_string(),
                    key,
                    ScopeCellOwnership::Owned,
                )? {
                    values.release(replaced)?;
                }
            }
            None => values.release(key)?,
        }
        for replaced in set_scope_cell(
            context,
            scope,
            value_name.to_string(),
            value,
            ScopeCellOwnership::Owned,
        )? {
            values.release(replaced)?;
        }
        match execute_statements(body, context, scope, values)? {
            EvalControl::None | EvalControl::Continue(1) => {}
            EvalControl::Break(1) => break,
            EvalControl::Break(level) => return Ok(EvalControl::Break(level - 1)),
            EvalControl::Continue(level) => return Ok(EvalControl::Continue(level - 1)),
            EvalControl::Throw(result) => return Ok(EvalControl::Throw(result)),
            EvalControl::ReturnVoid => return Ok(EvalControl::ReturnVoid),
            EvalControl::Return(result) => return Ok(EvalControl::Return(result)),
            EvalControl::Goto(label) => return Ok(EvalControl::Goto(label)),
        }
        eval_generator_mark_advanced(identity, context);
        eval_generator_step(identity, None, true, context, values)?;
    }
    Ok(EvalControl::None)
}
