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
    if !crate::eval_trace::enabled() {
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
            eval_echo_expr(expr, context, scope, values)?;
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
            if eval_condition(condition, context, scope, values)? {
                execute_statements(then_branch, context, scope, values)
            } else {
                execute_statements(else_branch, context, scope, values)
            }
        }
        EvalStmt::Return(Some(expr)) => {
            if context.returns_by_ref() {
                return eval_by_ref_return(expr, context, scope, values);
            }
            if let EvalExpr::NullCoalesce { value, default } = expr {
                if !expr_returns_shared_member_cell(expr)
                    && (expr_returns_shared_member_cell(value)
                        || expr_returns_shared_member_cell(default))
                {
                    return eval_mixed_coalesce_return(value, default, context, scope, values)
                        .map(EvalControl::Return);
                }
            }
            // The returned value is an independent owner: a storage read (a variable, a static, a
            // class constant, a by-value parameter) is copied by `eval_owned_expr`, so a discarded
            // `f($x);` can never release the CALLER's `$x`.
            Ok(EvalControl::Return(eval_owned_expr(expr, context, scope, values)?))
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
            let trace = crate::eval_trace::enabled();
            let value = eval_owned_expr(value, context, scope, values).map_err(|status| {
                if trace {
                    eprintln!("[elephc-eval-trace] phase=store_var_error stage=value name={name:?} status={status:?}");
                }
                status
            })?;
            let reference_target = scope.reference_target(name).cloned();
            if let Some(target) = reference_target {
                write_back_method_ref_target(&target, value, context, values).map_err(|status| {
                    if trace {
                        eprintln!("[elephc-eval-trace] phase=store_var_error stage=writeback name={name:?} status={status:?} target={}", target.kind_name());
                    }
                    status
                })?;
            }
            let replaced = if scope.reference_target(name).is_some() {
                // The by-reference target took the value; the local entry only borrows it.
                set_scope_cell(
                    context,
                    scope,
                    name.clone(),
                    value,
                    ScopeCellOwnership::Borrowed,
                    values,
                )
            } else {
                set_owned_scope_cell(context, scope, name.clone(), value, values)
            }
            .map_err(|status| {
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
            let thrown = eval_owned_expr(expr, context, scope, values)?;
            let tag = values.type_tag(thrown);
            if tag != Ok(EVAL_TAG_OBJECT) {
                let _ = eval_release_value(context, values, thrown);
                tag?;
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
            if let Some(replaced) = unset_scope_cell(context, scope, name.clone()) {
                eval_release_value(context, values, replaced)?;
            }
            Ok(EvalControl::None)
        }
        EvalStmt::UnsetDynamicVar { name } => {
            // Blade's anonymous-component prologue runs `unset($$__key)`.
            let name = crate::interpreter::expressions::eval_dynamic_member_name(
                name, context, scope, values,
            )?;
            if let Some(replaced) = unset_scope_cell(context, scope, name) {
                eval_release_value(context, values, replaced)?;
            }
            Ok(EvalControl::None)
        }
        EvalStmt::GcCollect => {
            values.collect_cycles()?;
            Ok(EvalControl::None)
        }
        EvalStmt::While { condition, body } => {
            while eval_condition(condition, context, scope, values)? {
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
            // `require $file;` written as a STATEMENT throws its value away, which is the one
            // shape where a source the compiler already included can be skipped outright. The
            // expression evaluator cannot know that; only the statement does.
            if let EvalExpr::Include { path, required, once } = expr {
                let result = super::super::include_exec::eval_discarded_include_expr(
                    path, *required, *once, context, scope, values,
                )?;
                eval_release_value(context, values, result)?;
                return Ok(EvalControl::None);
            }
            let result = eval_expr(expr, context, scope, values)?;
            release_expr_result(result, context, values)?;
            Ok(EvalControl::None)
        }
    }
}

/// Returns a `??` whose two sides differ in ownership, deciding by the side actually taken.
///
/// `return static::$validators ?? static::$validators = [...];` answers with the cell the static
/// holds on every call after the first, and with the assignment's own fresh value on the first.
/// Only the first is a member cell that needs a reference of the caller's own; judged as a whole
/// the expression is "mixed" and was returned unretained, so the caller's release after a
/// `foreach` freed the array the static still held and the next route match read freed memory.
/// The left side is evaluated exactly as `??` evaluates it: quietly, without warnings.
fn eval_mixed_coalesce_return(
    value: &EvalExpr,
    default: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    context.push_error_suppression();
    context.push_quiet_property_fetch();
    let left = eval_expr(value, context, scope, values);
    context.pop_quiet_property_fetch();
    context.pop_error_suppression();
    let left = left?;
    let (taken, result) = if values.is_null(left)? {
        (default, eval_expr(default, context, scope, values)?)
    } else {
        (value, left)
    };
    if expr_returns_shared_member_cell(taken) {
        values.retain(result)
    } else {
        Ok(result)
    }
}

/// Runs a `declare(ticks=N)` body, or turns ticking on for the rest of the enclosing scope.
///
/// The interval is saved and restored around a BLOCK body, because php scopes a block-form
/// directive to that block: `declare(ticks=1) { … }` ticks inside the braces and nowhere after.
/// The statement form has no body and simply runs to the end of the scope it was written in.
/// Returns whether every value a returned expression can produce is a class-held member cell.
///
/// A mixed `?:`/`??` (one side a member cell, the other a fresh value or a local) is left alone:
/// retaining a fresh value would leak it, and a local is transferred by the scope release.
fn expr_returns_shared_member_cell(expr: &EvalExpr) -> bool {
    match expr {
        EvalExpr::ClassConstantFetch { .. }
        | EvalExpr::DynamicClassConstantFetch { .. }
        | EvalExpr::DynamicClassConstantNameFetch { .. }
        | EvalExpr::StaticPropertyGet { .. }
        | EvalExpr::DynamicStaticPropertyGet { .. }
        | EvalExpr::DynamicStaticPropertyNameGet { .. } => true,
        EvalExpr::Unary { op: EvalUnaryOp::ErrorSuppress, expr } => {
            expr_returns_shared_member_cell(expr)
        }
        // `return self::$instance ??= new self();` answers with the cell the static slot holds:
        // the existing one, or the new value the slot just took over.
        EvalExpr::NullCoalesceAssign { target, .. } => expr_returns_shared_member_cell(target),
        EvalExpr::Ternary {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_returns_shared_member_cell(then_branch.as_deref().unwrap_or(condition))
                && expr_returns_shared_member_cell(else_branch)
        }
        EvalExpr::NullCoalesce { value, default } => {
            expr_returns_shared_member_cell(value) && expr_returns_shared_member_cell(default)
        }
        _ => false,
    }
}

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
