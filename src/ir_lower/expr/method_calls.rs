//! Purpose:
//! Core instance-method dispatch and Closure rebinding methods.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Whether a method-call receiver is a reflector whose metadata the eval bridge may be the only
/// holder of: the same four classes codegen routes to the bridge
/// (`method_dispatch::reflection_class_runtime_metadata_method`), alone or as a union's arms.
fn receiver_is_runtime_metadata_reflector(ctx: &LoweringContext<'_, '_>, value: ValueId) -> bool {
    // A closed-world build has no bridge to hold a reflector's metadata: every reflector is built
    // from the module reflection table and answers its own methods.
    if crate::closed_world::is_enabled() {
        return false;
    }
    let is_reflector = |name: &str| {
        matches!(
            php_symbol_key(name.trim_start_matches('\\')).as_str(),
            "reflectionattribute" | "reflectionclass" | "reflectionobject" | "reflectionenum"
        )
    };
    match ctx.builder.value_php_type(value) {
        PhpType::Object(name) => is_reflector(&name),
        PhpType::Union(members) => {
            let mut any = false;
            for member in members {
                match member {
                    PhpType::Object(name) if is_reflector(&name) => any = true,
                    PhpType::Void | PhpType::False => {}
                    _ => return false,
                }
            }
            any
        }
        _ => false,
    }
}

/// The private helpers the table-reading reflector constructors call on `$this`
/// (`__fill_from_row`, `__decode`, `__fill_generic<slot>`): they run on the object being built, so
/// no bridge can hold its metadata and the call is a plain method call.
fn is_table_reflector_internal(method: &str) -> bool {
    method == "__fill_from_row" || method == "__decode" || method.starts_with("__fill_generic")
}

/// Lowers a method call on a receiver that is statically never an object as the `Error` php
/// raises for it, before evaluating an argument (`?->` short-circuits null only); the backend has
/// no dispatch for a scalar. Returns `None` for any other receiver.
pub(super) fn lower_method_call_on_scalar(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    method: &str,
    op: Op,
    expr: &Expr,
) -> Option<LoweredValue> {
    if !matches!(op, Op::MethodCall | Op::NullsafeMethodCall) {
        return None;
    }
    let given = match ctx.builder.value_php_type(object.value).codegen_repr() {
        PhpType::Str => "string",
        PhpType::Int => "int",
        PhpType::Float => "float",
        PhpType::Array(_) | PhpType::AssocArray { .. } => "array",
        _ => return None,
    };
    release_owning_receiver_temporary(ctx, object, expr.span);
    Some(crate::ir_lower::stmt::lower_throw_access_error_expr(
        ctx,
        &format!("Call to a member function {method}() on {given}"),
        expr.span,
    ))
}

/// Lowers an object method call.
pub(super) fn lower_method_call(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    method: &str,
    args: &[Expr],
    op: Op,
    expr: &Expr,
) -> LoweredValue {
    // A statically-decided private/protected method access from an inaccessible
    // scope raises a catchable `Error` in PHP rather than a compile-time error,
    // but the receiver expression must still be evaluated first.
    let throw_access_message = if op == Op::MethodCall {
        ctx.throw_access_sites
            .get(&(ctx.loop_storage_scope.clone(), expr.span))
            .and_then(|info| {
            if let ThrowAccessKind::PrivateMethod {
                visibility,
                class_name,
                method: m,
            } = &info.kind
            {
                Some(format!(
                    "Call to {} method {}::{}() from global scope",
                    visibility, class_name, m
                ))
            } else {
                None
            }
        })
    } else {
        None
    };
    let object_expr = object;
    let object = lower_expr(ctx, object_expr);
    // A reflector's members can live only in the eval bridge: one built from a RUNTIME class name
    // is materialized there with no member arrays of its own. Codegen routes such calls to the
    // bridge only in a function that owns an eval context, so a reflector handed to another
    // function -- symfony/routing's `AttributeClassLoader` passes its `ReflectionClass` around --
    // listed no methods at all there. The function that CALLS the reflector needs the context as
    // much as the one that built it.
    if op == Op::MethodCall && !is_table_reflector_internal(method) && receiver_is_runtime_metadata_reflector(ctx, object.value) {
        ctx.declare_eval_context_local();
    }
    if let Some(message) = throw_access_message {
        release_owning_receiver_temporary(ctx, object, expr.span);
        return crate::ir_lower::stmt::lower_throw_access_error_expr(ctx, &message, expr.span);
    }
    // Before the null test, which reads an int's zero payload as the null sentinel.
    if let Some(error) = lower_method_call_on_scalar(ctx, object, method, op, expr) {
        return error;
    }
    if op == Op::MethodCall && value_is_definitely_null(ctx, object.value) {
        let null_value = lower_null(ctx, expr);
        terminate_method_call_on_null(ctx, method);
        return null_value;
    }
    // An unknown receiver whose class has `__call()` but not this method is not on the dispatch
    // ladder; see `magic_call_fallback`.
    if op == Op::MethodCall {
        if let Some(call) = super::magic_call_fallback::lower_magic_call_fallback(
            ctx, object_expr, object, method, args, expr,
        ) {
            return call;
        }
    }
    if op == Op::MethodCall {
        if let Some(value) =
            lower_reflection_function_invoke_call(ctx, Some(object_expr), method, args, expr)
        {
            release_owning_receiver_temporary(ctx, object, expr.span);
            return value;
        }
        if let Some(value) =
            lower_reflection_method_invoke_call(ctx, Some(object_expr), method, args, expr)
        {
            release_owning_receiver_temporary(ctx, object, expr.span);
            return value;
        }
    }
    if op == Op::MethodCall
        && (value_is_nullable(ctx, object.value)
            || value_may_carry_container_miss(ctx, object.value))
    {
        return lower_nullable_regular_method_call(ctx, object, method, args, expr);
    }
    if op == Op::MethodCall
        && is_reflection_class_new_instance_call(ctx, object.value, method)
        && reflection_class_new_instance_forwards_statically(ctx, Some(object_expr), args)
    {
        return lower_reflection_class_new_instance(ctx, Some(object_expr), object, args, expr);
    }
    if op == Op::MethodCall
        && is_reflection_class_new_instance_args_call(ctx, object.value, method)
        && reflection_class_new_instance_args_forward_statically(ctx, Some(object_expr), args)
    {
        return lower_reflection_class_new_instance_args(
            ctx,
            Some(object_expr),
            object,
            args,
            expr,
        );
    }
    if op == Op::MethodCall
        && is_reflection_class_new_instance_without_constructor_call(ctx, object.value, method)
    {
        return lower_reflection_class_new_instance_without_constructor(ctx, object, args, expr);
    }
    // Both answers come from compile-time metadata, so the receiver is never read; a temporary
    // one (`(new ReflectionClass(X::class))->getMethods()`) is still released, or its whole
    // member graph outlives the call. A static-property access to an undeclared name ends the
    // block with php's throw, and nothing may follow a terminator.
    if op == Op::MethodCall {
        if let Some(value) = lower_reflection_class_static_property_value_call(
            ctx,
            Some(object_expr),
            method,
            args,
            expr,
        ) {
            if !ctx.builder.insertion_block_is_terminated() {
                release_owning_receiver_temporary(ctx, object, expr.span);
            }
            return value;
        }
    }
    if op == Op::MethodCall {
        if let Some(value) =
            lower_reflection_class_member_list_call(ctx, Some(object_expr), method, args, expr)
        {
            if !ctx.builder.insertion_block_is_terminated() {
                release_owning_receiver_temporary(ctx, object, expr.span);
            }
            return value;
        }
    }
    if op == Op::MethodCall {
        if let Some(value) = lower_reflection_property_value_call(
            ctx,
            Some(object_expr),
            object,
            method,
            args,
            expr,
        )
        {
            release_owning_receiver_temporary(ctx, object, expr.span);
            return value;
        }
    }
    if matches!(
        ctx.builder.value_php_type(object.value).codegen_repr(),
        PhpType::Callable
    ) {
        // `$f->__invoke($x)` is `$f($x)`. There is no instance dispatch for a callable receiver,
        // so it goes through the call path the short spelling uses.
        if php_symbol_key(method) == "__invoke" {
            return lower_expr_call_from_value(ctx, object, args, expr);
        }
        if let Some(result) =
            lower_closure_bind_method(ctx, object_expr, &object, method, args, expr)
        {
            return result;
        }
    }
    if op == Op::MethodCall {
        if let Some(receiver) = object_static_method_receiver(ctx, object.value, method) {
            let call = lower_static_method_call(ctx, &receiver, method, args, expr);
            release_owning_receiver_temporary(ctx, object, expr.span);
            return call;
        }
    }
    let magic_args;
    let (dispatch_method, args) = if let Some(args) =
        magic_call_dispatch_args(ctx, object.value, method, args, object_expr.span)
    {
        magic_args = args;
        ("__call", magic_args.as_slice())
    } else {
        (method, args)
    };
    let result_type = method_call_result_type(ctx, object.value, dispatch_method, op, expr);
    let sig = super::method_metadata::boxed_receiver_source_signature(
        ctx,
        object.value,
        method_call_argument_signature(ctx, object_expr, object.value, dispatch_method),
    );
    // An override wider than the signature this site is typed against needs its defaults padded;
    // see `override_arity`.
    if op == Op::MethodCall && dispatch_method == method {
        if let Some(call) =
            lower_wider_override_dispatch(ctx, object, method, args, sig.as_ref(), expr, false)
        {
            return call;
        }
    }
    // A receiver typed only `object` may be an instance of ANY class with this method, including
    // one the interpreter declared, so the signature found for it is one candidate's and cannot
    // place named arguments: `function f(object $o) { $o->m(b: 1); }` bound `b` by whichever
    // compiled class happened to declare `m`. The boxed-receiver path hands the names to the
    // runtime as `call_user_func_array` keys, which is php's own rule.
    if op == Op::MethodCall
        && dispatch_method == method
        && crate::types::call_args::has_named_args(args)
    {
        if let Some(call) = lower_boxed_receiver_variadic_method_call(ctx, object, method, args, expr)
        {
            return call;
        }
    }
    if let Some(call) = lower_runtime_spread_method_call(
        ctx,
        object,
        dispatch_method,
        args,
        sig.as_ref(),
        result_type.clone(),
        expr,
    ) {
        return call;
    }
    let mut operands = vec![object.value];
    // An omitted BY-REFERENCE parameter still needs a cell to be written through; see
    // `pad_omitted_by_ref_args`. Done before the ref-place rewrite so the padded place goes
    // through the same machinery a supplied one does.
    let padded_args;
    // A by-reference shape borrowed for a `mixed` receiver describes SOME candidate, not the
    // callee; inventing cells for the parameters it omits would hand them to the wrong class.
    let borrowed_shape =
        sig.is_some() && method_signature(ctx, object.value, dispatch_method).is_none();
    let args = match sig
        .as_ref()
        .filter(|_| !borrowed_shape)
        .and_then(|signature| pad_omitted_by_ref_args(ctx, dispatch_method, signature, args))
    {
        Some(padded) => {
            padded_args = padded;
            padded_args.as_slice()
        }
        None => args,
    };
    promote_eval_bridge_method_argument_locals(ctx, object.value, sig.as_ref(), args);
    promote_pdo_binding_ref_argument(ctx, object.value, dispatch_method, args);
    // A by-reference-returning method transfers a lease that must outlive this caller's
    // argument and receiver cleanup, so its staging is published before the arguments.
    let reference_staging = begin_reference_return_call(ctx, sig.as_ref(), expr.span);
    // An ordinary owned result needs the same protection: retiring the argument roots, the
    // evaluation intermediates and an owning receiver all run PHP destructors that can throw,
    // and the result is only an SSA temporary until this staging holds it. Both the result type
    // and the alias summary are resolved from the receiver and the method alone, so they are
    // available here, before any argument expression runs.
    let return_alias = method_return_arg_alias(ctx, object.value, dispatch_method);
    let result_staging = prepublish_user_call_result(
        ctx, sig.as_ref(), &return_alias, &result_type, expr.span,
    );
    // A receiver that is an owning temporary (`(new X())->m()`, `$o->a()->b()`) is pure SSA: its
    // release follows the call, so a throwing argument expression or a callee that lets an
    // exception propagate would strand it. Pin it over the whole call window; the normal path
    // still releases it below.
    let receiver_pins = pin_in_flight_owners(ctx, &[object.value], expr.span);
    let prepared = sig
        .as_ref()
        .and_then(|signature| ref_place_args::prepare_ref_place_args(ctx, signature, args));
    let call_args = prepared
        .as_ref()
        .map(|(call_args, _)| call_args.as_slice())
        .unwrap_or(args);
    begin_call_argument_evaluation(ctx);
    let arg_values = lower_user_args_with_signature(ctx, sig.as_ref(), call_args);
    let mut arg_values = arg_values;
    let evaluation_intermediates = finish_call_argument_evaluation(ctx, &mut arg_values);
    let roots = root_user_call_operands(
        ctx,
        &mut arg_values,
        sig.as_ref(),
        &return_alias,
        &result_type,
        expr.span,
    );
    operands.extend(arg_values.iter().copied());
    let data = ctx.intern_string(dispatch_method);
    let call = ctx.emit_value(
        op,
        operands,
        Some(Immediate::Data(data)),
        result_type,
        op.default_effects(),
        Some(expr.span),
    );
    ctx.invalidate_callable_ref_argument_locals(sig.as_ref(), &arg_values);
    let call = finish_reference_return_call(
        ctx, call, sig.as_ref(), reference_staging.as_ref(), expr.span,
    );
    stage_call_result(ctx, result_staging.as_ref(), call, expr.span);
    release_owned_call_arg_temporaries_with_roots(
        ctx,
        &arg_values,
        Some(call.value),
        &return_alias,
        sig.as_ref(),
        &roots,
        expr.span,
    );
    retire_call_argument_intermediates(ctx, &evaluation_intermediates);
    if let Some((_, plans)) = prepared {
        ref_place_args::write_back_ref_place_args(ctx, plans);
    }
    unpin_in_flight_owners(ctx, receiver_pins, expr.span);
    release_owning_receiver_temporary(ctx, object, expr.span);
    let call = take_prepublished_call_result(ctx, result_staging, call, expr.span);
    finish_reference_return_value(ctx, call, reference_staging, expr.span)
}

/// Promotes local arguments that an eval-backed mixed receiver may later mutate by reference.
pub(super) fn promote_eval_bridge_method_argument_locals(
    ctx: &mut LoweringContext<'_, '_>,
    object: ValueId,
    signature: Option<&FunctionSig>,
    args: &[Expr],
) {
    if !dynamic_method_receiver_needs_mixed_fallback(&ctx.builder.value_php_type(object)) {
        return;
    }
    // A no-argument call still needs the per-request eval context: its receiver can be an
    // eval-owned object or an AOT class outside the statically emitted candidate set. Without
    // it, codegen has no generic fallback and turns a live object into a misleading null-method
    // fatal. Web handlers retain one context for the request, so this remains generic and does
    // not depend on any framework or package layout.
    // Only a program that can load PHP at run time (an `eval()`, an unexpanded include, or class
    // files handed to the runtime autoloader) can meet such a receiver; in a closed program every
    // class is in the candidate ladder, so a call that matches none is php's own "undefined
    // method" error and no interpreter has to be linked for it.
    if ctx.web && ctx.program_loads_code_at_run_time() {
        ctx.declare_eval_context_local();
    }
    if !plain_positional_call_args(args) {
        return;
    }
    let bridge_args = match signature {
        Some(signature) => signature
            .ref_params
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, is_ref)| is_ref.then(|| args.get(index)).flatten())
            .collect::<Vec<_>>(),
        None => args.iter().collect(),
    };
    if bridge_args.is_empty() {
        return;
    }
    // The context belongs to the interpreter's by-reference binding; a closed-world build links
    // none, so the arguments are promoted (below) without one.
    if !crate::closed_world::is_enabled() {
        ctx.declare_eval_context_local();
    }
    for argument in bridge_args {
        if let ExprKind::Variable(name) = &argument.kind {
            // A reference-bound name's representation is its CELL's, which this body does not
            // own: a by-reference parameter's cell is the caller's slot. Retyping `int &$cursor`
            // to `mixed` read the caller's raw int as a box pointer from here on, and the next
            // `++$cursor` stored a heap address into it -- Symfony's YAML lexer failed on every
            // quoted string through `isset($this->currentLine[$cursor])`, whose `ArrayAccess`
            // arm is such a call.
            if ctx.is_ref_bound_local(name)
                && ctx.ref_cell_capture_storage_type(name).codegen_repr() != PhpType::Mixed
            {
                continue;
            }
            ctx.set_local_type(name, PhpType::Mixed);
        }
    }
}

/// Routes a method call with a spread that cannot use a fixed ABI through the
/// callable descriptor ABI.
///
/// A known signature with one trailing indexed spread can materialize its visible
/// parameters directly. Every other spread, including a concrete method that is
/// absent from an interface's static signature, must preserve its runtime keys so
/// the shared invoker can apply PHP positional/named argument rules before
/// selecting the method wrapper.
pub(super) fn lower_runtime_spread_method_call(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    method: &str,
    args: &[Expr],
    signature: Option<&FunctionSig>,
    result_type: PhpType,
    expr: &Expr,
) -> Option<LoweredValue> {
    if !args.iter().any(is_spread_arg) {
        return None;
    }
    if signature.is_some() && has_statically_indexed_trailing_method_spread(ctx, args) {
        return None;
    }
    // `Fiber::start()` has no emitted method body to build a descriptor over: its callback-shaped
    // signature (see `ir_lower::fibers`) places named and spread arguments statically instead.
    if signature.is_some() && method_is_fiber_start(ctx, object.value, method) {
        return None;
    }
    let data = ctx.intern_string(&format!("object::{}", method));
    let descriptor = ctx.emit_value(
        Op::FirstClassCallableNew,
        vec![object.value],
        Some(Immediate::ProfiledData {
            data,
            strict_php: crate::strict_php::is_enabled(),
        }),
        PhpType::Callable,
        Op::FirstClassCallableNew.default_effects(),
        Some(expr.span),
    );
    // The descriptor is rooted before the arguments are evaluated, so a throwing argument
    // cannot strand it.
    let descriptor = super::descriptor_invoke::root_descriptor_callback(
        ctx,
        descriptor,
        result_type,
        expr.span,
    );
    let arg_container = lower_untyped_descriptor_invoker_arg_container(ctx, args, expr.span);
    Some(emit_callable_descriptor_invoke(ctx, descriptor, arg_container, expr.span))
}

/// Returns whether a fixed method signature can materialize one trailing
/// spread directly without discarding runtime named-argument keys.
fn has_statically_indexed_trailing_method_spread(
    ctx: &LoweringContext<'_, '_>,
    args: &[Expr],
) -> bool {
    let Some(spread_idx) = single_trailing_indexed_spread_arg(ctx, args) else {
        return false;
    };
    let ExprKind::Spread(inner) = &args[spread_idx].kind else {
        return false;
    };
    matches!(
        indexed_spread_source_type(ctx, inner),
        Some(PhpType::Array(_))
    )
}

/// Resolves a static-only method invoked through an object receiver.
///
/// PHP permits `$object->staticMethod()`. The object expression is still evaluated by the caller,
/// but dispatch ignores its value and uses the receiver's class exactly like `Class::method()`.
/// An ordinary instance method with the same name always wins.
pub(super) fn object_static_method_receiver(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
) -> Option<StaticReceiver> {
    let object_ty = ctx.builder.value_php_type(object);
    let (class_name, _) = singular_object_class(&object_ty)?;
    static_only_object_method_receiver(ctx, class_name, method)
}

/// Resolves static-syntax object calls whose receiver type names one concrete class.
///
/// The parser represents `$object::method()` as an internal callable-array invocation, so this
/// helper recovers the known static target before that dynamic path discards static methods.
pub(super) fn object_static_method_receiver_for_expr(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    method: &str,
) -> Option<StaticReceiver> {
    let class_name = instance_callable_object_class(ctx, object)?;
    static_only_object_method_receiver(ctx, &class_name, method)
}

/// Resolves a static-only method without shadowing a same-named instance method.
fn static_only_object_method_receiver(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
    method: &str,
) -> Option<StaticReceiver> {
    let normalized = class_name.trim_start_matches('\\');
    let method_key = php_symbol_key(method);
    let class_info = ctx.classes.get(normalized)?;
    if class_info.methods.contains_key(&method_key) {
        return None;
    }
    let receiver = StaticReceiver::Named(Name::from(normalized.to_string()));
    static_method_implementation_signature(ctx, &receiver, method)?;
    Some(receiver)
}

/// Lowers the `Closure` rebinding methods on a closure (`Callable`) receiver:
/// `$closure->bindTo($newThis [, $scope])` and `$closure->call($newThis, ...$args)`.
/// Returns `None` for any other method so normal dispatch (and its diagnostics)
/// still apply. A statically known explicit `$scope` receives its own closure
/// function specialization, preserving PHP member visibility without changing
/// the original closure's lexical scope.
pub(super) fn lower_closure_bind_method(
    ctx: &mut LoweringContext<'_, '_>,
    closure_expr: &Expr,
    closure: &LoweredValue,
    method: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let method = php_symbol_key(method);
    let result_type = match method.as_str() {
        "bindto" => PhpType::Callable,
        "call" => PhpType::Mixed,
        _ => return None,
    };
    if method == "bindto" {
        if let (Some(new_this), Some(scope)) = (args.first(), args.get(1)) {
            if let Some(plan) = plan_scoped_closure_bind(ctx, closure_expr, scope) {
                return Some(lower_planned_scoped_closure_bind(
                    ctx, plan, *closure, new_this, scope, expr,
                ));
            }
        }
    }
    // `$closure->call($newThis, ...)` binds `$this` AND the class scope of `$newThis`, so the
    // body may read its private members (php: "temporarily binds the closure to newThis").
    // Binding only `$this` left the body unscoped and `$this->privateProp` raised
    // "Cannot access private property". The object's static class names the scope.
    if method == "call" {
        if let Some(new_this) = args.first() {
            if let Some(class_name) = instance_callable_object_class(ctx, new_this) {
                let scope = Expr::new(ExprKind::StringLiteral(class_name), new_this.span);
                if let Some(plan) = plan_scoped_closure_bind(ctx, closure_expr, &scope) {
                    let bound = lower_planned_scoped_closure_bind(
                        ctx, plan, *closure, new_this, &scope, expr,
                    );
                    // The bound descriptor is consumed right here, not by a later call site.
                    let _ = ctx.take_pending_static_callable_result();
                    let result_staging = prepublish_call_result(ctx, &PhpType::Mixed, expr.span);
                    let bound = root_descriptor_callback(ctx, bound, PhpType::Mixed, expr.span);
                    let arg_container =
                        lower_untyped_descriptor_invoker_arg_container(ctx, &args[1..], expr.span);
                    let result = emit_callable_descriptor_invoke(ctx, bound, arg_container, expr.span);
                    stage_call_result(ctx, result_staging.as_ref(), result, expr.span);
                    return Some(take_prepublished_call_result(
                        ctx,
                        result_staging,
                        result,
                        expr.span,
                    ));
                }
            }
        }
    }
    // An escaped closure, such as a global replaced by a destructor or an error handler, is
    // stored in a boxed Mixed cell once its compile-time callable identity is invalidated. The
    // runtime binder consumes the raw callable descriptor, not the address of that Mixed box.
    // Extract an owning descriptor view at the same boundary used for declared callable
    // parameters before publishing the bind operands for unwind cleanup.
    let closure = if matches!(
        ctx.builder.value_php_type(closure.value).codegen_repr(),
        PhpType::Mixed | PhpType::Union(_)
    ) {
        unbox_callable_param_storage(ctx, *closure, Some(closure_expr.span))
    } else {
        *closure
    };
    // Binding retains its own receiver and environment. Original temporary inputs still
    // need their own retirement, including a throw while evaluating the new receiver or
    // invocation arguments. The result must outlive both original-input cleanup records.
    let result_staging = prepublish_call_result(ctx, &result_type, expr.span);
    let (closure, closure_owner) = root_owned_call_operand(ctx, closure, expr.span);
    let new_this = match args.first() {
        Some(arg) => lower_expr(ctx, arg),
        None => lower_null(ctx, expr),
    };
    let (new_this, receiver_owner) = root_owned_call_operand(ctx, new_this, expr.span);
    // The scope is stamped at run time when no specialization above could name it:
    // `bindTo($newThis, $scope)` names it, `call($newThis)` takes the receiver's own class.
    let scope_owner_value;
    let scope = if method == "call" {
        let receiver_type = ctx.builder.value_php_type(new_this.value).codegen_repr();
        matches!(
            receiver_type,
            PhpType::Object(_) | PhpType::Mixed | PhpType::Union(_)
        )
        .then(|| {
            let class_id = ctx.emit_value(
                Op::ClassNameToId,
                vec![new_this.value],
                None,
                PhpType::Int,
                Op::ClassNameToId.default_effects(),
                args.first().map(|arg| arg.span),
            );
            (class_id.value, new_this.value)
        })
    } else {
        None
    };
    let bindto_scope = if method == "bindto" {
        super::static_method_calls::lower_runtime_bind_scope(ctx, args.get(1))
    } else {
        None
    };
    scope_owner_value = bindto_scope.as_ref().map(|scope| (scope.value, scope.span));
    let scope = scope.or_else(|| {
        bindto_scope
            .as_ref()
            .map(|scope| (scope.class_id, scope.value.value))
    });
    let bound = super::static_method_calls::emit_closure_bind_with_scope(
        ctx,
        closure.value,
        new_this.value,
        scope,
        expr,
    );
    if let Some((value, span)) = scope_owner_value {
        if ctx.value_is_owning_temporary(value) {
            crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
        }
    }
    let result = if method == "call" {
        let call_args = &args[args.len().min(1)..];
        let bound = root_descriptor_callback(ctx, bound, PhpType::Mixed, expr.span);
        let arg_container =
            lower_untyped_descriptor_invoker_arg_container(ctx, call_args, expr.span);
        emit_callable_descriptor_invoke(ctx, bound, arg_container, expr.span)
    } else {
        bound
    };
    stage_call_result(ctx, result_staging.as_ref(), result, expr.span);
    if let Some(slot) = receiver_owner {
        retire_owned_call_operand(ctx, slot, expr.span);
    }
    if let Some(slot) = closure_owner {
        retire_owned_call_operand(ctx, slot, expr.span);
    }
    Some(take_prepublished_call_result(ctx, result_staging, result, expr.span))
}

/// A statically resolved closure function cloned for one explicit PHP visibility scope.
pub(super) struct ScopedClosureBindPlan {
    name: String,
    signature: FunctionSig,
    return_alias: ReturnArgAlias,
}

/// Plans a scope-specialized binding without emitting EIR or evaluating source expressions.
///
/// The generic runtime binder copies a descriptor, so it cannot alter the already compiled
/// function's lexical member-access scope. When both the closure and scope class are statically
/// known, clone that EIR function and assign the requested lexical class to the clone. The source
/// function remains unchanged for unbound calls and bindings using another scope.
pub(super) fn plan_scoped_closure_bind(
    ctx: &mut LoweringContext<'_, '_>,
    closure_expr: &Expr,
    scope_expr: &Expr,
) -> Option<ScopedClosureBindPlan> {
    let binding = static_callable_binding_for_expr(ctx, closure_expr)
        .or_else(|| ctx.take_pending_static_callable_result())?;
    let StaticCallableBinding::Closure {
        name,
        signature,
        captures,
        return_alias,
    } = binding
    else {
        return None;
    };
    let requested_scope = static_callable_class_name(ctx, scope_expr)?;
    let scope = lookup_folded_name(
        ctx.classes.keys(),
        requested_scope.trim_start_matches('\\'),
    )?;
    let mut function = ctx.closure_function(&name)?.clone();
    let capture_params = function
        .params
        .get(function.params.len().checked_sub(captures.len())?..)?;
    let [this_param] = capture_params else {
        return None;
    };
    if this_param.name != "this"
        || this_param.php_type.codegen_repr() != PhpType::Mixed
        || this_param.by_ref
    {
        return None;
    }
    let scoped_name = ctx.next_closure_name();
    function.name = scoped_name.clone();
    function.lexical_class = Some(scope);
    ctx.extend_closures([function]);
    Some(ScopedClosureBindPlan {
        name: scoped_name,
        signature,
        return_alias,
    })
}

/// Materializes a descriptor whose compiled function uses the planned PHP visibility scope.
pub(super) fn lower_planned_scoped_closure_bind(
    ctx: &mut LoweringContext<'_, '_>,
    plan: ScopedClosureBindPlan,
    closure: LoweredValue,
    new_this_expr: &Expr,
    scope_expr: &Expr,
    expr: &Expr,
) -> LoweredValue {
    let result_staging = prepublish_call_result(ctx, &PhpType::Callable, expr.span);
    let (_, closure_owner) = root_owned_call_operand(ctx, closure, expr.span);
    let new_this = lower_expr(ctx, new_this_expr);
    let (new_this, receiver_owner) = root_owned_call_operand(ctx, new_this, new_this_expr.span);
    let scope = lower_expr(ctx, scope_expr);
    let (_, scope_owner) = root_owned_call_operand(ctx, scope, scope_expr.span);

    let receiver_capture =
        crate::ir_lower::ownership::acquire_if_refcounted(ctx, new_this, Some(expr.span));
    let boxed_this = if ctx
        .builder
        .value_php_type(receiver_capture.value)
        .codegen_repr()
        == PhpType::Mixed
    {
        receiver_capture
    } else {
        ctx.box_value_as_mixed(receiver_capture, PhpType::Mixed, Some(expr.span))
    };
    let captures = vec![ClosureCapture {
        value: boxed_this.value,
        by_ref_local: None,
    }];
    let data = ctx.intern_string(&plan.name);
    let bound = ctx.emit_value(
        Op::ClosureNew,
        vec![boxed_this.value],
        Some(Immediate::Data(data)),
        PhpType::Callable,
        Op::ClosureNew.default_effects(),
        Some(expr.span),
    );
    stage_call_result(ctx, result_staging.as_ref(), bound, expr.span);
    if let Some(slot) = scope_owner {
        retire_owned_call_operand(ctx, slot, scope_expr.span);
    }
    if let Some(slot) = receiver_owner {
        retire_owned_call_operand(ctx, slot, new_this_expr.span);
    }
    if let Some(slot) = closure_owner {
        retire_owned_call_operand(ctx, slot, expr.span);
    }
    let bound = take_prepublished_call_result(ctx, result_staging, bound, expr.span);
    ctx.set_pending_static_callable_result(StaticCallableBinding::Closure {
        name: plan.name,
        signature: plan.signature,
        captures,
        return_alias: plan.return_alias,
    });
    bound
}

/// Builds synthetic `__call` arguments when a class lacks the requested method.
pub(super) fn magic_call_dispatch_args(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
    args: &[Expr],
    span: Span,
) -> Option<Vec<Expr>> {
    if method_signature(ctx, object, method).is_some() {
        return None;
    }
    let object_ty = ctx.builder.value_php_type(object);
    let Some((class_name, _)) = singular_object_class(&object_ty) else {
        return None;
    };
    let normalized = class_name.trim_start_matches('\\');
    class_method_signature(ctx, normalized, &php_symbol_key("__call"))?;
    Some(vec![
        Expr::new(ExprKind::StringLiteral(method.to_string()), span),
        Expr::new(ExprKind::ArrayLiteral(args.to_vec()), span),
    ])
}

/// Returns the signature to use for method-call argument normalization.
pub(super) fn method_call_argument_signature(
    ctx: &LoweringContext<'_, '_>,
    object_expr: &Expr,
    object: crate::ir::ValueId,
    method: &str,
) -> Option<FunctionSig> {
    if method_is_fiber_start(ctx, object, method) {
        return crate::ir_lower::fibers::start_sig_for_expr(ctx, object_expr);
    }
    method_signature(ctx, object, method).or_else(|| {
        matches!(
            ctx.builder.value_php_type(object).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
        .then(|| mixed_receiver_by_ref_signature(ctx, method))
        .flatten()
    })
}

/// The by-reference shape every class declaring `method` agrees on, for a `mixed` receiver.
///
/// The checker already binds such arguments by reference (`mixed_receiver_by_ref_signature`), but
/// the call itself passed them BY VALUE: Symfony's `NativeSessionStorage::loadSession()` runs
/// `$bag->initialize($session[$key])` over a mixed list of bags, each `initialize(array &$a)`
/// binding a property to the element, and the session data written later never reached
/// `$_SESSION`. Candidates must agree, as in the checker; otherwise the call keeps by-value args.
///
/// Only the by-reference SHAPE is borrowed. The receiver may well be a class whose same-named
/// method takes no reference at all, so the candidate's parameter types are not this call's:
/// Symfony's `Translator::trans()` calls `$catalogue->get($id, $domain)` on a mixed catalogue,
/// the cache contracts' `get(string $key, callable $callback, ..., ?array &$metadata)` was the
/// candidate, and `$domain` was normalized as a callable ("Call to undefined function").
pub(super) fn mixed_receiver_by_ref_signature(
    ctx: &LoweringContext<'_, '_>,
    method: &str,
) -> Option<FunctionSig> {
    let key = php_symbol_key(method);
    let mut found: Option<&FunctionSig> = None;
    for class_info in ctx.classes.values() {
        let Some(sig) = class_info.methods.get(&key) else {
            continue;
        };
        if !sig.ref_params.iter().any(|by_ref| *by_ref) {
            continue;
        }
        match found {
            Some(previous) if previous.ref_params != sig.ref_params => return None,
            Some(_) => {}
            None => found = Some(sig),
        }
    }
    let mut shape = found?.clone();
    // The candidate's DEFAULTS are not this call's: the receiver may be any class declaring
    // `method`, and the lowering pads omitted trailing parameters from `defaults`. Twig's
    // `Environment::parse()` calls `$this->parser->parse($stream)` through an untyped property,
    // the polyfill `NumberFormatter::parse(string, int $type = 3, &$offset = null)` was the one
    // candidate with a reference, and `Twig\Parser::parse($stream, $test = null)` received `3`.
    // Omitted parameters are left to whichever callee the receiver turns out to be.
    shape.defaults.iter_mut().for_each(|default| *default = None);
    for (_, ty) in &mut shape.params {
        *ty = PhpType::Mixed;
    }
    shape.param_type_exprs.iter_mut().for_each(|expr| *expr = None);
    shape.declared_params.iter_mut().for_each(|declared| *declared = false);
    shape.return_type = PhpType::Mixed;
    shape.declared_return = false;
    Some(shape)
}

/// Returns true when a method call targets PHP's built-in `Fiber::start()`.
pub(super) fn method_is_fiber_start(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
) -> bool {
    if php_symbol_key(method) != "start" {
        return false;
    }
    let object_ty = ctx.builder.value_php_type(object);
    let Some((class_name, _)) = singular_object_class(&object_ty) else {
        return false;
    };
    php_symbol_key(class_name.trim_start_matches('\\')) == "fiber"
}

/// Unboxes a boxed `?C` receiver to `C` once the null branch has been taken away.
///
/// The call then lowers exactly as on a `C` receiver. Left boxed, it took the backend's nullable
/// path, which knows ordinary method bodies but none of the built-in classes' intrinsics:
/// `function g(): ?LogicException` then `g()->getMessage()` jumped to an empty vtable slot,
/// `?SplStack->top()` printed the stack, and `?Generator->current()` dispatched over every
/// `Iterator` and pulled in pcre2. The unbox takes its own reference, which the call releases
/// as it does any owning receiver; the caller still releases the box it was given.
fn narrow_non_null_receiver(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    span: Span,
) -> Option<LoweredValue> {
    let object_ty = ctx.builder.value_php_type(object.value);
    if !matches!(object_ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_)) {
        return None;
    }
    let (class_name, true) = singular_object_class(&object_ty)? else {
        return None;
    };
    let target = PhpType::Object(class_name.to_string());
    Some(ctx.emit_value(
        Op::MixedUnbox,
        vec![object.value],
        None,
        target,
        Op::MixedUnbox.default_effects(),
        Some(span),
    ))
}

/// Lowers `?Object->method()` calls so null receivers fatal before argument evaluation.
pub(super) fn lower_nullable_regular_method_call(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    method: &str,
    args: &[Expr],
    expr: &Expr,
) -> LoweredValue {
    let result_type = method_call_result_type(ctx, object.value, method, Op::MethodCall, expr);
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let fatal_block = ctx
        .builder
        .create_named_block("method.null.fatal", Vec::new());
    let call_block = ctx
        .builder
        .create_named_block("method.non_null.call", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("method.nullable.merge", Vec::new());
    let is_null = ctx.emit_value(
        Op::IsNull,
        vec![object.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: fatal_block,
        then_args: Vec::new(),
        else_target: call_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(fatal_block);
    terminate_method_call_on_null(ctx, method);

    ctx.builder.position_at_end(call_block);
    let call = if let Some(object_ty) =
        super::override_arity::nullable_receiver_ladder_type(ctx, object, method, args)
    {
        lower_call_on_unboxed_receiver(ctx, object, object_ty, method, args, expr)
    } else {
        match narrow_non_null_receiver(ctx, object, expr.span) {
            Some(narrowed) => {
                // The boxed receiver stays pure SSA until its release below; keep it reachable
                // from the unwinder while the call (and its arguments) can throw.
                let receiver_pins = pin_in_flight_owners(ctx, &[object.value], expr.span);
                let call = lower_method_call_with_receiver(
                    ctx,
                    narrowed,
                    method,
                    args,
                    Op::MethodCall,
                    expr,
                );
                unpin_in_flight_owners(ctx, receiver_pins, expr.span);
                release_owning_receiver_temporary(ctx, object, expr.span);
                call
            }
            None => {
                lower_method_call_with_receiver(ctx, object, method, args, Op::MethodCall, expr)
            }
        }
    };
    store_value_into_temp(ctx, &temp_name, result_type.clone(), call, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    take_owned_temp(ctx, &temp_name, expr.span)
}

/// Calls `method` on a non-null boxed receiver unboxed to `object_ty`.
///
/// The unboxed object is parked in a synthetic local and the call borrows it from there, so the
/// call lowers exactly as it would on a typed local (the override ladder refuses an owning
/// temporary receiver). The local's reference and the boxed receiver are released after it.
pub(super) fn lower_call_on_unboxed_receiver(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    object_ty: PhpType,
    method: &str,
    args: &[Expr],
    expr: &Expr,
) -> LoweredValue {
    // The boxed receiver is released after the call and is pure SSA until then, so pin it over
    // the call window: a throwing argument or callee must not strand it.
    let receiver_pins = pin_in_flight_owners(ctx, &[object.value], expr.span);
    let unboxed = ctx.emit_value(
        Op::MixedUnbox,
        vec![object.value],
        None,
        object_ty.clone(),
        Op::MixedUnbox.default_effects(),
        Some(expr.span),
    );
    let receiver_temp = ctx.declare_synthetic_php_local(object_ty.clone());
    // A PHP-local store retains and releases the owned unboxed reference itself.
    ctx.store_local(&receiver_temp, unboxed, object_ty.clone(), Some(expr.span));
    let receiver = ctx.load_local(&receiver_temp, Some(expr.span));
    let sig = method_signature(ctx, receiver.value, method);
    let call = lower_wider_override_dispatch(ctx, receiver, method, args, sig.as_ref(), expr, true)
        .unwrap_or_else(|| {
            lower_method_call_with_receiver(ctx, receiver, method, args, Op::MethodCall, expr)
        });
    let slot = ctx.declare_local(&receiver_temp, object_ty);
    ctx.release_stored_local_value(&receiver_temp, slot, Some(expr.span));
    ctx.emit_void(
        Op::UnsetLocal,
        Vec::new(),
        Some(Immediate::LocalSlot(slot)),
        Op::UnsetLocal.default_effects(),
        Some(expr.span),
    );
    unpin_in_flight_owners(ctx, receiver_pins, expr.span);
    release_owning_receiver_temporary(ctx, object, expr.span);
    call
}
