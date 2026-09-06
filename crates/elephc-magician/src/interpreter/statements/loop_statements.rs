//! Purpose:
//! Executes static-local declarations, switch, and loop statement families.
//!
//! Called from:
//! - `crate::interpreter::statements::execute_stmt()`.
//!
//! Key details:
//! - Break/continue control, foreach array/object/iterator traversal, and key materialization are preserved.

use super::*;

/// Executes a PHP `static $name = expr;` declaration in the current eval scope.
pub(in crate::interpreter) fn execute_static_var_stmt(
    name: &str,
    init: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let Some(function_name) = context.current_function().map(str::to_string) else {
        let value = eval_expr(init, context, scope, values)?;
        if let Some(replaced) = scope.set(name.to_string(), value, ScopeCellOwnership::Owned) {
            values.release(replaced)?;
        }
        return Ok(());
    };
    if scope.contains_visible(name) {
        return Ok(());
    }
    let value = if let Some(value) = context.static_local(&function_name, name) {
        value
    } else {
        let value = eval_expr(init, context, scope, values)?;
        let _ = context.set_static_local(function_name.clone(), name.to_string(), value);
        value
    };
    if let Some(replaced) = scope.set(name.to_string(), value, ScopeCellOwnership::Borrowed) {
        values.release(replaced)?;
    }
    Ok(())
}

/// Executes a PHP switch with loose case matching, default fallback, and fallthrough.
pub(in crate::interpreter) fn execute_switch_stmt(
    expr: &EvalExpr,
    cases: &[EvalSwitchCase],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let subject = eval_expr(expr, context, scope, values)?;
    let mut default_index = None;
    let mut matched_index = None;
    for (index, case) in cases.iter().enumerate() {
        let Some(condition) = &case.condition else {
            if default_index.is_none() {
                default_index = Some(index);
            }
            continue;
        };
        let condition = eval_expr(condition, context, scope, values)?;
        let matches = values.compare(EvalBinOp::LooseEq, subject, condition)?;
        if values.truthy(matches)? {
            matched_index = Some(index);
            break;
        }
    }
    let Some(start_index) = matched_index.or(default_index) else {
        return Ok(EvalControl::None);
    };
    for case in &cases[start_index..] {
        match execute_statements(&case.body, context, scope, values)? {
            EvalControl::None => {}
            EvalControl::Break(1) | EvalControl::Continue(1) => break,
            EvalControl::Break(level) => return Ok(EvalControl::Break(level - 1)),
            EvalControl::Continue(level) => return Ok(EvalControl::Continue(level - 1)),
            EvalControl::Throw(result) => return Ok(EvalControl::Throw(result)),
            EvalControl::ReturnVoid => return Ok(EvalControl::ReturnVoid),
            EvalControl::Return(result) => return Ok(EvalControl::Return(result)),
            EvalControl::Goto(label) => return Ok(EvalControl::Goto(label)),
        }
    }
    Ok(EvalControl::None)
}

/// Executes a PHP `do/while` loop, evaluating the condition after every body run.
pub(in crate::interpreter) fn execute_do_while_stmt(
    body: &[EvalStmt],
    condition: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    loop {
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
        let condition = eval_expr(condition, context, scope, values)?;
        if !values.truthy(condition)? {
            break;
        }
    }
    Ok(EvalControl::None)
}

/// Executes a PHP `for` loop while preserving update-on-continue semantics.
pub(in crate::interpreter) fn execute_for_stmt(
    init: &[EvalStmt],
    condition: Option<&EvalExpr>,
    update: &[EvalStmt],
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    match execute_statements(init, context, scope, values)? {
        EvalControl::None | EvalControl::Continue(1) => {}
        EvalControl::Break(1) => return Ok(EvalControl::None),
        EvalControl::Break(level) => return Ok(EvalControl::Break(level - 1)),
        EvalControl::Continue(level) => return Ok(EvalControl::Continue(level - 1)),
        EvalControl::Throw(result) => return Ok(EvalControl::Throw(result)),
        EvalControl::ReturnVoid => return Ok(EvalControl::ReturnVoid),
        EvalControl::Return(result) => return Ok(EvalControl::Return(result)),
        EvalControl::Goto(label) => return Ok(EvalControl::Goto(label)),
    }
    loop {
        if let Some(condition) = condition {
            let condition = eval_expr(condition, context, scope, values)?;
            if !values.truthy(condition)? {
                break;
            }
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
        match execute_statements(update, context, scope, values)? {
            EvalControl::None | EvalControl::Continue(1) => {}
            EvalControl::Break(1) => break,
            EvalControl::Break(level) => return Ok(EvalControl::Break(level - 1)),
            EvalControl::Continue(level) => return Ok(EvalControl::Continue(level - 1)),
            EvalControl::Throw(result) => return Ok(EvalControl::Throw(result)),
            EvalControl::ReturnVoid => return Ok(EvalControl::ReturnVoid),
            EvalControl::Return(result) => return Ok(EvalControl::Return(result)),
            EvalControl::Goto(label) => return Ok(EvalControl::Goto(label)),
        }
    }
    Ok(EvalControl::None)
}

/// Executes a PHP `foreach` loop over eval array and Traversable object values.
pub(in crate::interpreter) fn execute_foreach_stmt(
    array: &EvalExpr,
    key_name: Option<&str>,
    value_name: &str,
    value_by_ref: bool,
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    // A subject the loop ALLOCATED belongs to the loop. Nothing else will ever release it: the
    // value never reaches a scope name, so no activation teardown sees it, and PHP's destruction
    // is observable — `foreach (new Bag() as $x)` runs `__destruct`, and breaking out of
    // `foreach (gen() as $v)` runs the generator's `finally`.
    let subject_owned = !value_by_ref && eval_foreach_owns_subject(array);
    let (array, array_target) = if value_by_ref {
        eval_call_arg_value(array, context, scope, values)?
    } else {
        (eval_expr(array, context, scope, values)?, None)
    };
    match values.type_tag(array)? {
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => {
            let iteration_array = if value_by_ref {
                values.retain(array)?
            } else {
                array
            };
            let result = execute_foreach_array_stmt(
                iteration_array,
                array_target.as_ref(),
                key_name,
                value_name,
                value_by_ref,
                body,
                context,
                scope,
                values,
            );
            if value_by_ref {
                values.release(iteration_array)?;
            }
            // Released on EVERY exit edge, which is why the loop's result is bound rather than
            // propagated with `?`: completion, `break`, `return` and a throw passing through all
            // arrive here.
            let released = if subject_owned {
                eval_release_value(context, values, array)
            } else {
                Ok(())
            };
            released.and(result)
        }
        EVAL_TAG_OBJECT => execute_foreach_object_stmt(
            array,
            key_name,
            value_name,
            value_by_ref,
            subject_owned,
            body,
            context,
            scope,
            values,
        ),
        _ => {
            if subject_owned {
                eval_release_value(context, values, array)?;
            }
            Err(EvalStatus::RuntimeFatal)
        }
    }
}

/// Returns whether the loop owns the value its subject expression produced.
///
/// This is the ownership rule the call machinery already uses for an argument allocated for the
/// call, widened by the call shapes: an interpreted call hands its return value to the caller —
/// `release_activation_scope` excludes exactly that handle from the activation's teardown — and
/// the statement-expression path already releases any expression result on that basis. A read of
/// existing storage such as `LoadVar` hands back a BORROWED cell and must not be released here.
fn eval_foreach_owns_subject(expr: &EvalExpr) -> bool {
    eval_expr_is_owning_temporary(expr)
        || matches!(
            expr,
            EvalExpr::Call { .. }
                | EvalExpr::NamespacedCall { .. }
                | EvalExpr::DynamicCall { .. }
                | EvalExpr::MethodCall { .. }
                | EvalExpr::NullsafeMethodCall { .. }
                | EvalExpr::DynamicMethodCall { .. }
                | EvalExpr::NullsafeDynamicMethodCall { .. }
                | EvalExpr::StaticMethodCall { .. }
                | EvalExpr::DynamicStaticMethodCall { .. }
        )
}

/// Executes `foreach` over a PHP array value using insertion-order runtime hooks.
pub(super) fn execute_foreach_array_stmt(
    array: RuntimeCellHandle,
    array_target: Option<&EvalReferenceTarget>,
    key_name: Option<&str>,
    value_name: &str,
    value_by_ref: bool,
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let len = values.array_len(array)?;
    for index in 0..len {
        let key = values.array_iter_key(array, index)?;
        let value = values.array_get(array, key)?;
        if let Some(key_name) = key_name {
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
        let replaced = if value_by_ref {
            let reference_key =
                eval_array_reference_key(key, values)?.ok_or(EvalStatus::RuntimeFatal)?;
            let target = array_target.cloned().map_or_else(
                || EvalReferenceTarget::Cell { cell: value },
                |array_target| EvalReferenceTarget::NestedArrayElement {
                    array_target: Box::new(array_target),
                    index: reference_key,
                },
            );
            let replaced = scope
                .rebind_reference(value_name.to_string(), value, ScopeCellOwnership::Borrowed)
                .into_iter()
                .collect();
            scope.set_reference_target(value_name.to_string(), target);
            replaced
        } else {
            set_scope_cell(
                context,
                scope,
                value_name.to_string(),
                value,
                ScopeCellOwnership::Owned,
            )?
        };
        for replaced in replaced {
            values.release(replaced)?;
        }
        if key_name.is_none() && !value_by_ref {
            values.release(key)?;
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
    }
    Ok(EvalControl::None)
}

/// Executes `foreach` over an Iterator or IteratorAggregate object.
pub(super) fn execute_foreach_object_stmt(
    object: RuntimeCellHandle,
    key_name: Option<&str>,
    value_name: &str,
    value_by_ref: bool,
    subject_owned: bool,
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    // The aggregate arm gives the subject back early, on purpose — PHP destroys an
    // `IteratorAggregate` temporary as soon as `getIterator()` has returned, so
    // `foreach (new Bag("temp") as $x) { echo $x; }` prints `destruct:temp;12`. Every other arm
    // holds the subject for the whole loop and lets go here.
    let mut subject_released = false;
    let result = execute_foreach_object_body(
        object,
        key_name,
        value_name,
        value_by_ref,
        subject_owned,
        &mut subject_released,
        body,
        context,
        scope,
        values,
    );
    let released = if subject_owned && !subject_released {
        eval_release_value(context, values, object)
    } else {
        Ok(())
    };
    released.and(result)
}

/// Runs the arm that matches the object's iteration protocol.
#[allow(clippy::too_many_arguments)]
fn execute_foreach_object_body(
    object: RuntimeCellHandle,
    key_name: Option<&str>,
    value_name: &str,
    value_by_ref: bool,
    subject_owned: bool,
    subject_released: &mut bool,
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let identity = values.object_identity(object)?;
    if context.has_eval_generator(identity) {
        if value_by_ref {
            // A by-reference `foreach` over a generator needs the yielded value to be an alias
            // into the generator's own frame, which suspension makes unsound to hold.
            return Err(EvalStatus::UnsupportedConstruct);
        }
        return execute_foreach_generator_stmt(
            identity, key_name, value_name, body, context, scope, values,
        );
    }
    if eval_foreach_object_is_a(object, "Iterator", context, values)? {
        return execute_foreach_iterator_stmt(
            object, key_name, value_name, body, context, scope, values,
        );
    }
    if eval_foreach_object_is_a(object, "IteratorAggregate", context, values)? {
        let iterator = eval_method_call_result(object, "getIterator", Vec::new(), context, values)?;
        // PHP's ordering: the aggregate's last reference dies with `getIterator()`, so a
        // temporary is destroyed BEFORE the first body run. An iterator that keeps the aggregate
        // alive — a generator method holding `$this` — keeps it alive here too.
        if subject_owned {
            *subject_released = true;
            eval_release_value(context, values, object)?;
        }
        let iterator_identity = values.object_identity(iterator).ok();
        let result = match values.type_tag(iterator)? {
            EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => execute_foreach_array_stmt(
                iterator,
                None,
                key_name,
                value_name,
                false,
                body,
                context,
                scope,
                values,
            ),
            EVAL_TAG_OBJECT
                if iterator_identity.is_some_and(|identity| context.has_eval_generator(identity)) =>
            {
                execute_foreach_generator_stmt(
                    iterator_identity.expect("checked just above"),
                    key_name,
                    value_name,
                    body,
                    context,
                    scope,
                    values,
                )
            }
            EVAL_TAG_OBJECT if eval_foreach_object_is_a(iterator, "Iterator", context, values)? => {
                execute_foreach_iterator_stmt(
                    iterator, key_name, value_name, body, context, scope, values,
                )
            }
            // PHP accepts an aggregate whose `getIterator()` returns ANOTHER aggregate and asks
            // again until it reaches something it can walk: measured with `php -n` 8.5.6, a
            // two-level chain yields `k0=v0,k1=v1,` rather than raising. The subject of the inner
            // round is this iterator, which the caller already owns, so it is not owned again.
            EVAL_TAG_OBJECT
                if eval_foreach_object_is_a(iterator, "IteratorAggregate", context, values)? =>
            {
                let mut inner_released = false;
                execute_foreach_object_body(
                    iterator,
                    key_name,
                    value_name,
                    false,
                    false,
                    &mut inner_released,
                    body,
                    context,
                    scope,
                    values,
                )
            }
            _ => Err(EvalStatus::RuntimeFatal),
        };
        // `getIterator()` handed its return value over, so the loop owns it whatever the subject
        // was: the aggregate may be a plain variable and the iterator still a fresh object.
        let released = eval_release_value(context, values, iterator);
        return released.and(result);
    }
    execute_foreach_plain_object_stmt(
        object,
        key_name,
        value_name,
        value_by_ref,
        body,
        context,
        scope,
        values,
    )
}

/// Iterates the properties of an object that implements neither Iterator interface.
///
/// PHP walks the properties VISIBLE FROM THE CALLING SCOPE: from outside the class only the
/// public ones, from inside a method the private and protected ones as well. That is the same
/// rule `get_object_vars()` follows, so this asks that builtin for the set rather than
/// re-deriving it — the two cannot then disagree about the same object, and the ordering,
/// visibility and dynamic-property handling are all decided in one place.
///
/// Measured against `php -n` 8.5.6 on a class with a private, a protected and a public property
/// plus three dynamic ones added out of alphabetical order: from outside the loop yields
/// `pub=u;zeta=z;alpha=a;mid=m;` and from inside a method `secret=s;prot=p;pub=u;zeta=z;alpha=a;mid=m;`.
pub(super) fn execute_foreach_plain_object_stmt(
    object: RuntimeCellHandle,
    key_name: Option<&str>,
    value_name: &str,
    value_by_ref: bool,
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let properties = eval_get_object_vars_result(&[object], context, values)?;
    if !value_by_ref {
        let result = execute_foreach_array_stmt(
            properties, None, key_name, value_name, false, body, context, scope, values,
        );
        values.release(properties)?;
        return result;
    }
    let result = execute_foreach_object_by_ref_stmt(
        object, properties, key_name, value_name, body, context, scope, values,
    );
    values.release(properties)?;
    result
}

/// Runs `foreach ($object as $k => &$v)`, aliasing the loop variable to each property.
///
/// The names come from the same visible-property snapshot the by-value arm iterates, so the two
/// forms cannot disagree about which properties a scope sees. Each binding then targets the
/// PROPERTY rather than the snapshot's copy, which is what makes an assignment inside the body
/// reach the object: `php -n` 8.5.6 leaves `{"a":10,"b":20,"c":30}` after a loop that multiplies
/// each value by ten.
fn execute_foreach_object_by_ref_stmt(
    object: RuntimeCellHandle,
    properties: RuntimeCellHandle,
    key_name: Option<&str>,
    value_name: &str,
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let access_scope = context.execution_scope();
    let len = values.array_len(properties)?;
    for index in 0..len {
        let key = values.array_iter_key(properties, index)?;
        let property = String::from_utf8(values.string_bytes(key)?)
            .map_err(|_| EvalStatus::RuntimeFatal)?;
        let value = values.array_get(properties, key)?;
        if let Some(key_name) = key_name {
            for replaced in set_scope_cell(
                context,
                scope,
                key_name.to_string(),
                key,
                ScopeCellOwnership::Owned,
            )? {
                values.release(replaced)?;
            }
        } else {
            values.release(key)?;
        }
        let target = EvalReferenceTarget::ObjectProperty {
            object,
            property,
            access_scope: access_scope.clone(),
        };
        let replaced: Vec<RuntimeCellHandle> = scope
            .rebind_reference(value_name.to_string(), value, ScopeCellOwnership::Borrowed)
            .into_iter()
            .collect();
        scope.set_reference_target(value_name.to_string(), target);
        for replaced in replaced {
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
    }
    Ok(EvalControl::None)
}

/// Drives one Iterator object through PHP's `foreach` method-call sequence.
pub(super) fn execute_foreach_iterator_stmt(
    iterator: RuntimeCellHandle,
    key_name: Option<&str>,
    value_name: &str,
    body: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let result = eval_method_call_result(iterator, "rewind", Vec::new(), context, values)?;
    values.release(result)?;
    loop {
        let valid = eval_method_call_result(iterator, "valid", Vec::new(), context, values)?;
        let is_valid = values.truthy(valid)?;
        values.release(valid)?;
        if !is_valid {
            return Ok(EvalControl::None);
        }

        let value = eval_method_call_result(iterator, "current", Vec::new(), context, values)?;
        let key = if key_name.is_some() {
            Some(eval_method_call_result(
                iterator,
                "key",
                Vec::new(),
                context,
                values,
            )?)
        } else {
            None
        };
        if let Some((key_name, key)) = key_name.zip(key) {
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
            EvalControl::None | EvalControl::Continue(1) => {
                let result =
                    eval_method_call_result(iterator, "next", Vec::new(), context, values)?;
                values.release(result)?;
            }
            EvalControl::Break(1) => return Ok(EvalControl::None),
            EvalControl::Break(level) => return Ok(EvalControl::Break(level - 1)),
            EvalControl::Continue(level) => return Ok(EvalControl::Continue(level - 1)),
            EvalControl::Throw(result) => return Ok(EvalControl::Throw(result)),
            EvalControl::ReturnVoid => return Ok(EvalControl::ReturnVoid),
            EvalControl::Return(result) => return Ok(EvalControl::Return(result)),
            EvalControl::Goto(label) => return Ok(EvalControl::Goto(label)),
        }
    }
}

/// Returns whether a foreach object satisfies one iterator interface.
pub(super) fn eval_foreach_object_is_a(
    object: RuntimeCellHandle,
    target: &str,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    dynamic_object_is_a(object, target, false, context, values)?
        .map_or_else(|| values.object_is_a(object, target, false), Ok)
}

/// Returns PHP's next automatic integer key for `$array[]` append writes.
pub(in crate::interpreter) fn eval_array_append_key(
    array: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let len = values.array_len(array)?;
    let mut next_key = None;
    for position in 0..len {
        let key = values.array_iter_key(array, position)?;
        if values.type_tag(key)? != EVAL_TAG_INT {
            continue;
        }
        let one = values.int(1)?;
        let candidate = values.add(key, one)?;
        let replace = if let Some(current) = next_key {
            let is_greater = values.compare(EvalBinOp::Gt, candidate, current)?;
            values.truthy(is_greater)?
        } else {
            true
        };
        if replace {
            next_key = Some(candidate);
        }
    }
    next_key.map_or_else(|| values.int(0), Ok)
}
