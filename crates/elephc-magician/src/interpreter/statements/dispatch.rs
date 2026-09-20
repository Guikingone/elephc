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
    for stmt in statements {
        eval_pcntl_maybe_dispatch(context, values)?;
        match execute_stmt(stmt, context, scope, values)? {
            EvalControl::None => {}
            control => return Ok(control),
        }
    }
    Ok(EvalControl::None)
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
        EvalStmt::ArraySetVar { name, index, value } => {
            eval_array_set_var_stmt(name, index, value, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::Break => Ok(EvalControl::Break),
        EvalStmt::Continue => Ok(EvalControl::Continue),
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
            execute_class_decl_stmt(class, context, scope, values)?;
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
        EvalStmt::TraitDecl(trait_decl) => {
            execute_trait_decl_stmt(trait_decl, context, scope, values)?;
            Ok(EvalControl::None)
        }
        EvalStmt::Foreach {
            array,
            key_name,
            value_name,
            body,
        } => execute_foreach_stmt(
            array,
            key_name.as_deref(),
            value_name,
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
                .with_return_type(return_type.clone());
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
            let condition = eval_expr(condition, context, scope, values)?;
            if values.truthy(condition)? {
                execute_statements(then_branch, context, scope, values)
            } else {
                execute_statements(else_branch, context, scope, values)
            }
        }
        EvalStmt::Return(Some(expr)) => {
            let value = eval_expr(expr, context, scope, values)?;
            Ok(EvalControl::Return(retain_unretained_scope_borrow(
                expr, value, context, scope, values,
            )?))
        }
        EvalStmt::Return(None) => Ok(EvalControl::ReturnVoid),
        EvalStmt::ReferenceAssign { target, source } => {
            for replaced in set_reference_alias(context, scope, target, source, values)? {
                values.release(replaced)?;
            }
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
        EvalStmt::StoreVar { name, value: value_expr } => {
            let value = eval_expr(value_expr, context, scope, values)?;
            // A pass-through read hands back a reference the reader never owned, so the new
            // cell records that rather than claiming `Owned` over it. Both halves of the scope
            // that act on ownership — the release of a replaced cell, and `drain_owned_cells`
            // — then skip it, which is what makes the alias free rather than merely balanced.
            let ownership = if value_is_an_unretained_scope_borrow(value_expr, value, context, scope)
            {
                ScopeCellOwnership::Borrowed
            } else {
                ScopeCellOwnership::Owned
            };
            for replaced in set_scope_cell(context, scope, name.clone(), value, ownership)? {
                eval_release_value(context, values, replaced)?;
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
            // A thrown value is handed to the catch site, which binds it `Owned` or releases
            // it outright for a variable-less `catch`. Either way it leaves this frame owned,
            // so `throw $param;` needs the same reference `return $param;` does.
            Ok(EvalControl::Throw(retain_unretained_scope_borrow(
                expr, thrown, context, scope, values,
            )?))
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
            while {
                let condition = eval_expr(condition, context, scope, values)?;
                values.truthy(condition)?
            } {
                match execute_statements(body, context, scope, values)? {
                    EvalControl::None | EvalControl::Continue => {}
                    EvalControl::Break => break,
                    EvalControl::Throw(result) => return Ok(EvalControl::Throw(result)),
                    EvalControl::ReturnVoid => return Ok(EvalControl::ReturnVoid),
                    EvalControl::Return(result) => return Ok(EvalControl::Return(result)),
                }
            }
            Ok(EvalControl::None)
        }
        EvalStmt::Expr(expr) => {
            let result = eval_expr(expr, context, scope, values)?;
            // Discarding a statement's value IS releasing it, which is correct for every value
            // the expression owns and fatal for the one it borrowed: a bare `$this;` reached
            // this release with a reference the statement never owned.
            if !value_is_an_unretained_scope_borrow(expr, result, context, scope) {
                eval_release_value(context, values, result)?;
            }
            Ok(EvalControl::None)
        }
    }
}

/// Collects the variable names an expression can hand back *unchanged*.
///
/// These are the shapes that return a subexpression's own value rather than materializing a
/// new one, so whatever ownership that subexpression had is the ownership the caller gets.
/// Everything else — a property read, an array read, a call, an arithmetic result — produces
/// an independent owner and must not be retained again.
fn scope_read_passthrough_names<'a>(expr: &'a EvalExpr, names: &mut Vec<&'a str>) {
    match expr {
        EvalExpr::LoadVar(name) => names.push(name),
        EvalExpr::Ternary {
            condition,
            then_branch,
            else_branch,
        } => {
            // `?:` with no middle operand hands back the condition's own value when truthy.
            if then_branch.is_none() {
                scope_read_passthrough_names(condition, names);
            }
            if let Some(then_branch) = then_branch {
                scope_read_passthrough_names(then_branch, names);
            }
            scope_read_passthrough_names(else_branch, names);
        }
        EvalExpr::NullCoalesce { value, default } => {
            scope_read_passthrough_names(value, names);
            scope_read_passthrough_names(default, names);
        }
        EvalExpr::Match { arms, default, .. } => {
            for arm in arms {
                scope_read_passthrough_names(&arm.value, names);
            }
            if let Some(default) = default {
                scope_read_passthrough_names(default, names);
            }
        }
        _ => {}
    }
}

/// True when a value came out of a BORROWED scope cell without a reference being taken for it.
///
/// `$this` and every by-value parameter are bound into the callee's scope as
/// `ScopeCellOwnership::Borrowed` — the caller keeps the only reference, and `drain_owned_cells`
/// deliberately skips them at teardown. `EvalExpr::LoadVar` hands that same handle straight back
/// with no retain, so every site that does something ownership-bearing with the value is acting
/// on a reference nobody gave it (#982). Three statements did, and each answers it differently:
///
/// - `return $this;` transfers the value to the CALLER, so the reference has to become real:
///   that site retains. Discarding the result — an expression statement, which is how the
///   fluent-interface idiom `$o->add("a");` reads — had `eval_release_value` destroy a live
///   object, so `$o instanceof Bag` answered `bool(false)` where PHP answers `true`.
/// - `$a = $this;` stored it as `ScopeCellOwnership::Owned`, and reassigning or unsetting `$a`
///   releases the cell it replaces — which destroyed the receiver with no `return` involved at
///   all. That site records `Borrowed` instead. Retaining there would work too, but only by
///   making a false label true at the cost of a reference nothing gives back: a method scope is
///   dropped without `drain_owned_cells`, so `$a = $this;` alone leaked three blocks per call,
///   measured. The truthful label costs nothing.
/// - `$this;` as a bare expression statement released it on the spot. A statement's value is
///   discarded BY releasing it, so that site skips the release.
///
/// The decision is made on BOTH axes, because either alone is wrong. The syntactic filter keeps
/// out expressions that already materialize an independent owner: `return $this->self;` reads a
/// property that retains, and it can hand back the very same handle `$this` holds, so a check on
/// the handle alone would act twice and leak. The handle check keeps out the pass-through branch
/// that did not run: in `$c ? $this : new Bag()` only one arm produces the value, and acting
/// because the *other* arm names a borrowed cell would leak just as badly.
fn value_is_an_unretained_scope_borrow(
    expr: &EvalExpr,
    value: RuntimeCellHandle,
    context: &ElephcEvalContext,
    scope: &ElephcEvalScope,
) -> bool {
    let mut names = Vec::new();
    scope_read_passthrough_names(expr, &mut names);
    names.into_iter().any(|name| {
        scope_entry(context, scope, name).is_some_and(|entry| {
            entry.flags().is_visible()
                && entry.flags().ownership == ScopeCellOwnership::Borrowed
                && entry.cell() == value
        })
    })
}

/// Takes a reference for a caller that is about to own a value the callee only borrowed.
fn retain_unretained_scope_borrow(
    expr: &EvalExpr,
    value: RuntimeCellHandle,
    context: &ElephcEvalContext,
    scope: &ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if value_is_an_unretained_scope_borrow(expr, value, context, scope) {
        values.retain(value)
    } else {
        Ok(value)
    }
}
