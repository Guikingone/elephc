//! Purpose:
//! Dispatches each EvalIR statement variant to its focused execution helper and
//! propagates structured loop, throw, and return control flow.
//!
//! Called from:
//! - `crate::interpreter::execute_program_outcome_with_context()`.
//! - Dynamic eval function and method execution.
//!
//! Key details:
//! - The exhaustive match remains centralized so every new `EvalStmt` variant
//!   must explicitly define its runtime control-flow behavior.

use super::*;

/// Executes statements in source order and propagates the first eval `return`.
pub(in crate::interpreter) fn execute_statements(
    statements: &[EvalStmt],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let mut position = 0;
    while let Some(stmt) = statements.get(position) {
        eval_pcntl_maybe_dispatch(context, values)?;
        match execute_stmt(stmt, context, scope, values) {
            Ok(EvalControl::None) => {
                eval_run_due_tick(context, values)?;
                position += 1;
            }
            Ok(EvalControl::Goto(label)) => {
                let Some(target) = statements.iter().position(
                    |statement| matches!(statement, EvalStmt::Label(candidate) if candidate == &label),
                ) else {
                    return Ok(EvalControl::Goto(label));
                };
                position = target + 1;
            }
            Ok(control) => return Ok(control),
            Err(status) => {
                if status == EvalStatus::RuntimeFatal {
                    // Same first-writer-wins rule as the expression dispatcher: this only lands
                    // for a statement whose failure nothing inside it described.
                    note_eval_runtime_failure(
                        format!("unsupported {} statement", eval_stmt_kind(stmt)),
                        context,
                    );
                }
                trace_failed_statement(stmt, status, context);
                return Err(status);
            }
        }
    }
    // Nothing is failing any more, so no description of a failure may survive here. An FFI entry
    // point can map an interpreter error onto a benign answer for its AOT caller — that is how
    // Symfony absorbs a missing autoload target — and the note that error left would otherwise
    // wait to be printed beside an unrelated fatal much later on.
    crate::errors::clear_eval_runtime_failure();
    Ok(EvalControl::None)
}

/// Emits the failing EvalIR statement only when opt-in runtime tracing is enabled.
fn trace_failed_statement(
    stmt: &EvalStmt,
    status: EvalStatus,
    context: &ElephcEvalContext,
) {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_none() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let call_site = context.call_site();
        eprintln!(
            "[elephc-eval-trace] phase=statement_error status={status:?} file={:?} line={} stmt={stmt:?}",
            call_site.0,
            call_site.2,
        );
    }));
}

/// Executes one statement and returns `Some` only for eval `return`.
pub(in crate::interpreter) fn execute_stmt(
    stmt: &EvalStmt,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    match stmt {
        EvalStmt::ArrayAppendVar { name, value } => {
            eval_array_append_var_stmt(name, value, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::ArrayAppend { target, value } => {
            eval_array_append(target, value, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::ArrayAppendReferenceBind { target, source } => {
            eval_array_append_reference_bind(target, source, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::ArraySetVar { name, index, value } => {
            eval_array_set_var_stmt(name, index, value, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::ArrayDestructure { targets, value } => {
            eval_array_destructure_stmt(targets, value, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::ArrayReferenceBind { target, source } => {
            eval_array_reference_bind(target, source, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::Break(level) => Ok(EvalControl::Break(*level)),
        EvalStmt::Continue(level) => Ok(EvalControl::Continue(*level)),
        EvalStmt::DeclareStrictTypes(enabled) => {
            context.set_strict_types(*enabled);
            Ok(EvalControl::None)
        }
        EvalStmt::DeclareTicks { every, body } => {
            execute_declare_ticks_stmt(*every, body.as_deref(), context, scope, values)
        }
        EvalStmt::DeclareDirective { name, body } => {
            execute_declare_directive_stmt(name, body.as_deref(), context, scope, values)
        }
        EvalStmt::SourceLine(line) => {
            context.set_call_line(*line);
            Ok(EvalControl::None)
        }
        EvalStmt::Goto(label) => Ok(EvalControl::Goto(label.clone())),
        EvalStmt::DoWhile { body, condition } => {
            execute_do_while_stmt(body, condition, context, scope, values)
        }
        EvalStmt::Echo(expr) => {
            let value = eval_expr(expr, context, scope, values)?;
            let value = eval_string_context_value(value, context, values)?;
            values.echo(value)?;
            Ok(EvalControl::None)
        }
        EvalStmt::For {
            init,
            condition,
            update,
            body,
        } => execute_for_stmt(
            init,
            condition.as_ref(),
            update,
            body,
            context,
            scope,
            values,
        ),
        EvalStmt::ClassDecl(class) => {
            // Named here rather than inside `execute_class_decl_stmt`: its stage helper cannot
            // hold `context`, because every call already borrows it mutably for the validation it
            // wraps. The stage itself stays available through ELEPHC_EVAL_TRACE; what the user
            // gets is the class, which is what makes 55 anonymous failures groupable.
            let result = execute_class_decl_stmt(class, context, scope, values);
            if matches!(result, Err(EvalStatus::RuntimeFatal)) {
                note_eval_runtime_failure(
                    format!("class {} could not be declared", class.name()),
                    context,
                );
            }
            result?;
            Ok(EvalControl::None)
        }
        EvalStmt::EnumDecl(enum_decl) => {
            execute_enum_decl_stmt(enum_decl, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::InterfaceDecl(interface) => {
            execute_interface_decl_stmt(interface, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::Label(_) => Ok(EvalControl::None),
        EvalStmt::TraitDecl(trait_decl) => {
            execute_trait_decl_stmt(trait_decl, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::Foreach {
            array,
            key_name,
            value_name,
            value_by_ref,
            body,
        } => execute_foreach_stmt(
            array,
            key_name.as_deref(),
            value_name,
            *value_by_ref,
            body,
            context,
            scope,
            values,
        ),
        EvalStmt::FunctionDecl {
            name,
            source_location,
            attributes,
            params,
            parameter_attributes,
            parameter_types,
            parameter_defaults,
            parameter_is_by_ref,
            parameter_is_variadic,
            return_type,
            returns_by_ref,
            body,
        } => {
            let key = name.to_ascii_lowercase();
            let mut function = EvalFunction::new(name.clone(), params.clone(), body.clone())
                .with_attributes(attributes.clone())
                .with_parameter_attributes(parameter_attributes.clone())
                .with_parameter_types(parameter_types.clone())
                .with_parameter_defaults(parameter_defaults.clone())
                .with_parameter_by_ref_flags(parameter_is_by_ref.clone())
                .with_parameter_variadic_flags(parameter_is_variadic.clone())
                .with_return_type(return_type.clone())
                .with_returns_by_ref(*returns_by_ref)
                // php requires `declare(strict_types=1)` to be a file's FIRST statement, and
                // declarations run in file order, so the mode in force here is the declaring
                // file's -- exactly the mode this function's own `return` must obey later.
                .with_strict_types(context.strict_types());
            if let Some(source_location) = source_location {
                function = function.with_source_location(*source_location);
            }
            context
                .define_function(key, function)
                .map_err(|_| EvalStatus::RuntimeFatal)?;
            Ok(EvalControl::None)
        }
        EvalStmt::Global { vars } => {
            execute_global_stmt(vars, context, scope)?;
            Ok(EvalControl::None)
        }
        EvalStmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if crate::interpreter::expressions::eval_truthy_expr(condition, context, scope, values)? {
                execute_statements(then_branch, context, scope, values)
            } else {
                execute_statements(else_branch, context, scope, values)
            }
        }
        EvalStmt::Return(Some(expr)) => {
            if context.returns_by_ref() {
                return eval_by_ref_return(expr, context, scope, values);
            }
            let value = eval_expr(expr, context, scope, values)?;
            // A return transfers ownership by `release_activation_scope` SKIPPING the returned
            // cell -- which only gives the caller a reference when the activation scope actually
            // owned one. Two named values in scope own nothing to transfer:
            //
            // - a `static`, whose cell belongs to the slot and has no scope entry to skip. A
            //   discarded `f();` then destroyed the value the next call was supposed to read.
            // - a by-value PARAMETER, which `bind_method_scope_args` binds `Borrowed` on
            //   purpose. `return $param;` handed back a borrowed cell where every caller treats
            //   a call result as owned (`eval_expr_result_aliases_storage` deliberately does not
            //   list a call), so a discarded `f($x);` released the CALLER's `$x`. Symfony's
            //   `HttpKernel::filterResponse` lost its `$event` to exactly that: it is
            //   `$this->dispatcher->dispatch($event, ...)` as a statement, and `dispatch()`
            //   returns its `$event` parameter.
            if let EvalExpr::LoadVar(name) = expr {
                // `$this` is excluded: the receiver already has its own compensation on the
                // CALL side (a method result that aliases the receiver is retained there), so
                // retaining here too would hand back two references and a discarded
                // `$obj->fluent();` would never run the object's destructor.
                let activation_owns = name == "this"
                    || scope.entry(name).is_some_and(|entry| {
                        entry.flags().ownership == ScopeCellOwnership::Owned
                    });
                if !activation_owns || scope.static_alias_slot(name).is_some() {
                    return Ok(EvalControl::Return(values.retain(value)?));
                }
            }
            Ok(EvalControl::Return(value))
        }
        EvalStmt::Return(None) => Ok(EvalControl::ReturnVoid),
        EvalStmt::ReferenceAssign { target, source } => {
            for replaced in set_reference_alias(context, scope, target, source, values)? {
                values.release(replaced)?;
            }
            Ok(EvalControl::None)
        }
        EvalStmt::VarReferenceBind { target, source } => {
            eval_var_reference_bind(target, source, context, scope, values)?;
            Ok(EvalControl::None)
        }
        stmt @ (
            EvalStmt::PropertyReferenceBind { .. }
            | EvalStmt::DynamicPropertyReferenceBind { .. }
            | EvalStmt::DynamicPropertySet { .. }
            | EvalStmt::DynamicPropertyArrayAppend { .. }
            | EvalStmt::DynamicPropertyArraySet { .. }
            | EvalStmt::DynamicPropertyCompoundAssign { .. }
            | EvalStmt::DynamicPropertyIncDec { .. }
            | EvalStmt::PropertySet { .. }
            | EvalStmt::PropertyArrayAppend { .. }
            | EvalStmt::PropertyArraySet { .. }
            | EvalStmt::PropertyCompoundAssign { .. }
            | EvalStmt::PropertyIncDec { .. }
            | EvalStmt::StaticPropertySet { .. }
            | EvalStmt::StaticPropertyReferenceBind { .. }
            | EvalStmt::StaticPropertyArrayAppend { .. }
            | EvalStmt::StaticPropertyArraySet { .. }
            | EvalStmt::StaticPropertyIncDec { .. }
            | EvalStmt::DynamicStaticPropertySet { .. }
            | EvalStmt::DynamicStaticPropertyReferenceBind { .. }
            | EvalStmt::DynamicStaticPropertyArrayAppend { .. }
            | EvalStmt::DynamicStaticPropertyArraySet { .. }
            | EvalStmt::DynamicStaticPropertyIncDec { .. }
            | EvalStmt::DynamicStaticPropertyNameSet { .. }
            | EvalStmt::DynamicStaticPropertyNameReferenceBind { .. }
            | EvalStmt::DynamicStaticPropertyNameArrayAppend { .. }
            | EvalStmt::DynamicStaticPropertyNameArraySet { .. }
            | EvalStmt::DynamicStaticPropertyNameIncDec { .. }
            | EvalStmt::UnsetProperty { .. }
            | EvalStmt::UnsetDynamicProperty { .. }
            | EvalStmt::UnsetStaticProperty { .. }
            | EvalStmt::UnsetDynamicStaticProperty { .. }
            | EvalStmt::UnsetDynamicStaticPropertyName { .. }
        ) => execute_property_stmt(stmt, context, scope, values),
        EvalStmt::StaticVar { name, init } => {
            execute_static_var_stmt(name, init, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::StoreVar { name, value } => {
            let trace = std::env::var_os("ELEPHC_EVAL_TRACE").is_some();
            // Mirrors `eval_assign`'s copy rule for the expression form of the same assignment:
            // any RHS that aliases persistent storage -- a variable read, but also a class
            // constant or static property fetch -- needs an independent copy before the scope
            // takes ownership of it, or a later reassignment releases storage this statement
            // never owned. This used to check `LoadVar` alone, which is what let `$state =
            // self::STATE_B;` bind straight to the class constant's shared cell.
            let copies_borrowed_variable = eval_expr_result_aliases_storage(value);
            let value = eval_expr(value, context, scope, values).map_err(|status| {
                if trace {
                    eprintln!("[elephc-eval-trace] phase=store_var_error stage=value name={name:?} status={status:?}");
                }
                status
            })?;
            let value = if copies_borrowed_variable {
                values.copy_value(value)?
            } else {
                value
            };
            let reference_target = scope.reference_target(name).cloned();
            if let Some(target) = reference_target {
                write_back_method_ref_target(&target, value, context, values).map_err(|status| {
                    if trace {
                        eprintln!("[elephc-eval-trace] phase=store_var_error stage=writeback name={name:?} status={status:?}");
                    }
                    status
                })?;
            }
            let replaced = set_scope_cell(
                context,
                scope,
                name.clone(),
                value,
                if scope.reference_target(name).is_some() {
                    ScopeCellOwnership::Borrowed
                } else {
                    ScopeCellOwnership::Owned
                },
            ).map_err(|status| {
                if trace {
                    eprintln!("[elephc-eval-trace] phase=store_var_error stage=scope name={name:?} status={status:?}");
                }
                status
            })?;
            for replaced in replaced {
                eval_release_value(context, values, replaced).map_err(|status| {
                    if trace {
                        eprintln!("[elephc-eval-trace] phase=store_var_error stage=release name={name:?} status={status:?}");
                    }
                    status
                })?;
            }
            Ok(EvalControl::None)
        }
        EvalStmt::Switch { expr, cases } => {
            execute_switch_stmt(expr, cases, context, scope, values)
        }
        EvalStmt::Throw(expr) => {
            let thrown = eval_expr(expr, context, scope, values)?;
            if values.type_tag(thrown)? != EVAL_TAG_OBJECT {
                return Err(EvalStatus::RuntimeFatal);
            }
            Ok(EvalControl::Throw(thrown))
        }
        EvalStmt::Try {
            body,
            catches,
            finally_body,
        } => execute_try_stmt(body, catches, finally_body, context, scope, values),
        EvalStmt::UnsetArrayElement { array, index } => {
            eval_array_unset_element_stmt(array, index, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::UnsetVar { name } => {
            if let Some(replaced) = unset_scope_cell(scope, name.clone()) {
                eval_release_value(context, values, replaced)?;
            }
            Ok(EvalControl::None)
        }
        EvalStmt::While { condition, body } => {
            while crate::interpreter::expressions::eval_truthy_expr(condition, context, scope, values)? {
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
        EvalStmt::Expr(expr) => {
            let result = eval_expr(expr, context, scope, values)?;
            // An expression used as a statement discards its value, but an ASSIGNMENT hands back
            // the very cell it just stored and a variable read hands back the scope's own. The
            // storage owns those; releasing them here gave back a reference the statement never
            // took, which destroyed the object the variable still pointed at.
            if !eval_expr_result_aliases_storage(expr) {
                eval_release_value(context, values, result)?;
            }
            Ok(EvalControl::None)
        }
    }
}

/// Runs a `declare(ticks=N)` body, or turns ticking on for the rest of the enclosing scope.
///
/// The interval is saved and restored around a BLOCK body, because php scopes a block-form
/// directive to that block: `declare(ticks=1) { … }` ticks inside the braces and nowhere after.
/// The statement form has no body and simply runs to the end of the scope it was written in.
fn execute_declare_ticks_stmt(
    every: i64,
    body: Option<&[EvalStmt]>,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let Some(body) = body else {
        context.set_tick_interval(every);
        return Ok(EvalControl::None);
    };
    let previous = context.set_tick_interval(every);
    let result = execute_statements(body, context, scope, values);
    context.set_tick_interval(previous);
    result
}

/// Runs any other `declare(name=…)`, warning the way php warns and continuing either way.
///
/// Measured with `php -n` 8.5.6: `declare(encoding='UTF-8');` says
/// `declare(encoding=...) ignored because Zend multibyte feature is turned off by settings`, and
/// any unknown name says `Unsupported declare 'NAME'`. Both are warnings; the script runs on.
fn execute_declare_directive_stmt(
    name: &str,
    body: Option<&[EvalStmt]>,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    if name.eq_ignore_ascii_case("encoding") {
        values.warning(
            "declare(encoding=...) ignored because Zend multibyte feature is turned off by \
             settings",
        )?;
    } else {
        values.warning(&format!("Unsupported declare '{name}'"))?;
    }
    match body {
        None => Ok(EvalControl::None),
        Some(body) => execute_statements(body, context, scope, values),
    }
}

/// Runs the registered tick handlers when `declare(ticks=N)` says one is due.
///
/// Measured with `php -n` 8.5.6: with `declare(ticks=1)` and one handler registered, the handler
/// runs after EVERY statement of the declared scope, including the `register_tick_function()`
/// call that registered it and the statements of a function declared in the same file -- but not
/// after the `unregister_tick_function()` that removed it.
fn eval_run_due_tick(
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if !context.tick_statement_is_due() {
        return Ok(());
    }
    let handlers = context.tick_functions();
    let previous = context.set_tick_running(true);
    let mut result = Ok(());
    for handler in handlers {
        if result.is_err() {
            break;
        }
        result = eval_call_user_func_with_values(vec![handler], context, values).map(|value| {
            let _ = values.release(value);
        });
    }
    context.set_tick_running(previous);
    result
}

/// Names one statement variant for a diagnostic, the way `eval_expr_kind` names an expression.
fn eval_stmt_kind(stmt: &EvalStmt) -> String {
    let rendered = format!("{stmt:?}");
    rendered
        .split(|ch: char| ch == '(' || ch == '{' || ch == ' ')
        .next()
        .unwrap_or("")
        .to_string()
}
