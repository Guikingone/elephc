//! Purpose:
//! Null coalesce and ternary-like lazy branch lowering.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers null coalesce so the default expression is evaluated only for null values.
pub(super) fn lower_null_coalesce(
    ctx: &mut LoweringContext<'_, '_>,
    value: &Expr,
    default: &Expr,
    expr: &Expr,
) -> LoweredValue {
    // Only the LEFT operand is fetched quietly. The default is ordinary code and must keep
    // raising: `$o->t ?? $o->u` answers for `$o->t` and still refuses `$o->u`.
    let value = lower_operand_in_quiet_property_fetch(ctx, value, |ctx| {
        lower_null_coalesce_value(ctx, value)
    });
    let is_null = ctx.emit_value(
        Op::IsNull,
        vec![value.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        Some(expr.span),
    );
    let result_type = null_coalesce_result_type(ctx, value.value, default);
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let split_initialized = ctx.initialized_slots_snapshot();
    let default_block = ctx
        .builder
        .create_named_block("coalesce.default", Vec::new());
    let value_block = ctx.builder.create_named_block("coalesce.value", Vec::new());
    let merge = ctx.builder.create_named_block("coalesce.merge", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: default_block,
        then_args: Vec::new(),
        else_target: value_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(default_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    store_branch_expr_into_temp(ctx, &temp_name, result_type.clone(), default, expr.span);
    release_discarded_branch_value(ctx, value, expr.span);
    let default_reachable = !ctx.builder.insertion_block_is_terminated();
    let default_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge);

    ctx.builder.position_at_end(value_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    store_value_into_temp(ctx, &temp_name, result_type, value, expr.span);
    let value_reachable = !ctx.builder.insertion_block_is_terminated();
    let value_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    ctx.restore_initialized_slots(merge_initialized_slots_for_expr(
        &split_initialized,
        default_initialized,
        default_reachable,
        value_initialized,
        value_reachable,
    ));
    take_owned_temp(ctx, &temp_name, expr.span)
}

/// Lowers the value side of `??`, suppressing undefined-offset warnings from
/// native array reads while preserving nullsafe-chain lazy evaluation.
pub(super) fn lower_null_coalesce_value(ctx: &mut LoweringContext<'_, '_>, value: &Expr) -> LoweredValue {
    if let Some(value) = nullsafe_chain::lower_with_missing_warning(ctx, value, false) {
        return value;
    }
    if let ExprKind::ArrayAccess { array, index } = &value.kind {
        if let Some(probed) = lower_array_access_offset_exists_coalesce(ctx, array, index, value) {
            return probed;
        }
        return lower_array_access_with_missing_warning(ctx, array, index, value, false);
    }
    if let Some(value) = lower_known_missing_property_probe(ctx, value) {
        return value;
    }
    if let Some(value) = lower_initialized_property_null_coalesce_probe(ctx, value) {
        return value;
    }
    // A typed property with no default starts UNINITIALIZED, and an ordinary read of one is
    // fatal in PHP. `??` is precisely the construct that must not raise there, so a property
    // that can be in that state is read the way `isset()` reads it. Every other property keeps
    // the ordinary path and its exact slot type.
    if let ExprKind::PropertyAccess { object, property } = &value.kind {
        // The left operand of `??` is one of php's silent probes, exactly like `isset()`: a name
        // this scope may not reach answers the DEFAULT rather than raising, and a name that was
        // never created answers it without an `Undefined property` warning. A receiver whose
        // class is only known at run time needs the runtime-name form to carry that mode.
        let needs_probe = property_probe_needs_runtime_name_form(ctx, object);
        let object = lower_expr(ctx, object);
        if property_can_be_uninitialized(ctx, object.value, property) {
            return lower_initialized_property_value(ctx, object, property, value);
        }
        if needs_probe {
            // `??` is the one silent probe that php still routes through `__get`.
            return super::property_access::lower_property_fetch_mode_from_value(
                ctx, object, property, PropertyFetchMode::Coalesce, value,
            );
        }
        return lower_property_get_from_value(ctx, object, property, Op::PropGet, value);
    }
    if let ExprKind::DynamicPropertyAccess { object, property } = &value.kind {
        // `$o->{$k} ?? $d` asks `__isset` and then `__get`, like the static-name form above.
        return lower_dynamic_property_fetch(ctx, object, property, PropertyFetchMode::Coalesce, value);
    }
    // A typed STATIC property starts uninitialized the same way, and its guard lives in the
    // backend rather than in an operation the lowering could branch on — so `??` needs its own
    // probe here too. `S::$s ?? "d"` raised where PHP answers `d`.
    if let ExprKind::StaticPropertyAccess { receiver, property } = &value.kind {
        if static_property_can_be_uninitialized(ctx, receiver, property) {
            return lower_initialized_static_property_value(ctx, receiver, property, value);
        }
    }
    lower_expr(ctx, value)
}

/// Probes a declared property before reading it so `??` treats an uninitialized slot as null.
pub(super) fn lower_initialized_property_null_coalesce_probe(
    ctx: &mut LoweringContext<'_, '_>,
    value: &Expr,
) -> Option<LoweredValue> {
    let ExprKind::PropertyAccess { object, property } = &value.kind else {
        return None;
    };
    if !property_probe_receiver_uses_native_object_storage(ctx, object) {
        return None;
    }
    if !matches!(
        super::native_isset::property_isset_action(ctx, object, property),
        Some(super::native_isset::IssetPropertyAction::Initialized)
    ) {
        return None;
    }

    let result_type = property_access_expr_type_for_ir(ctx, object, property)
        .filter(|ty| singular_object_class(ty).is_some())
        .map(nullable_result_type)
        .unwrap_or(PhpType::Mixed);
    let object = lower_subscript_receiver_silently(ctx, object);
    let object_is_owned = ctx.value_is_owning_temporary(object);
    let property_data = ctx.intern_string(property);
    let initialized = ctx.emit_value(
        Op::PropInitialized,
        vec![object.value],
        Some(Immediate::Data(property_data)),
        PhpType::Bool,
        Op::PropInitialized.default_effects(),
        Some(value.span),
    );
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let split_initialized = ctx.initialized_slots_snapshot();
    let missing_block = ctx
        .builder
        .create_named_block("coalesce.property.uninitialized", Vec::new());
    let read_block = ctx
        .builder
        .create_named_block("coalesce.property.read", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("coalesce.property.merge", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: initialized.value,
        then_target: read_block,
        then_args: Vec::new(),
        else_target: missing_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(missing_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    let null = lower_boxed_null(ctx, value);
    store_value_into_temp(ctx, &temp_name, result_type.clone(), null, value.span);
    if object_is_owned {
        crate::ir_lower::ownership::release_if_owned(ctx, object, Some(value.span));
    }
    let missing_reachable = !ctx.builder.insertion_block_is_terminated();
    let missing_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge);

    ctx.builder.position_at_end(read_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    let property_value =
        lower_property_get_from_value(ctx, object, property, Op::PropGet, value);
    store_value_into_temp(
        ctx,
        &temp_name,
        result_type,
        property_value,
        value.span,
    );
    let read_reachable = !ctx.builder.insertion_block_is_terminated();
    let read_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    ctx.restore_initialized_slots(merge_initialized_slots_for_expr(
        &split_initialized,
        missing_initialized,
        missing_reachable,
        read_initialized,
        read_reachable,
    ));
    Some(take_owned_temp(ctx, &temp_name, value.span))
}

/// Fetches an overloaded prefix quietly, retaining its receiver across the magic callbacks.
pub(super) fn lower_magic_property_null_coalesce_probe(
    ctx: &mut LoweringContext<'_, '_>,
    value: &Expr,
) -> Option<LoweredValue> {
    let ExprKind::PropertyAccess { object, property } = &value.kind else {
        return None;
    };
    let class_name = super::native_isset::isset_object_expr_class(ctx, object)?.0;
    let class_info = ctx.classes.get(&class_name)?;
    if property_is_accessible_for_ir(ctx, &class_name, class_info, property) {
        return None;
    }
    let getter = class_method_signature(ctx, &class_name, &php_symbol_key("__get"))?;
    let declares_isset = class_method_signature(ctx, &class_name, &php_symbol_key("__isset")).is_some();
    let has_isset = declares_isset || ctx.classes.iter().any(|(name, info)| {
        info.methods.contains_key("__isset") && class_extends_class(ctx, name, &class_name)
    });
    let result_type = nullable_result_type(getter.return_type.clone());
    let receiver = lower_subscript_receiver_silently(ctx, object);
    let receiver = if ctx.value_is_owning_temporary(receiver) {
        receiver
    } else {
        // A hook may remove the original variable's last reference. Pin the receiver
        // for the whole fetch, not merely for the first callback.
        crate::ir_lower::ownership::acquire_lifetime_pin_if_refcounted(
            ctx, receiver, Some(value.span),
        )
    };
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let split_initialized = ctx.initialized_slots_snapshot();
    let false_block = ctx.builder.create_named_block("isset.magic_prefix.false", Vec::new());
    let get_block = ctx.builder.create_named_block("isset.magic_prefix.get", Vec::new());
    let merge = ctx.builder.create_named_block("isset.magic_prefix.merge", Vec::new());
    let probe_block = has_isset.then(|| {
        ctx.builder.create_named_block("isset.magic_prefix.probe", Vec::new())
    });
    let lookup_block = (has_isset && !declares_isset).then(|| {
        ctx.builder.create_named_block("isset.magic_prefix.lookup", Vec::new())
    });
    let is_null = ctx.emit_value(
        Op::IsNull, vec![receiver.value], None, PhpType::Bool,
        Op::IsNull.default_effects(), Some(value.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: false_block,
        then_args: Vec::new(),
        else_target: lookup_block.or(probe_block).unwrap_or(get_block),
        else_args: Vec::new(),
    });
    if let Some(lookup_block) = lookup_block {
        ctx.builder.position_at_end(lookup_block);
        ctx.restore_initialized_slots(split_initialized.clone());
        let method = lower_expr(ctx, &Expr::new(
            ExprKind::StringLiteral("__isset".to_string()), value.span,
        ));
        let target = crate::ir::RuntimeFnId::MethodExists;
        let available = ctx.emit_value(
            Op::RuntimeCall,
            vec![receiver.value, method.value],
            Some(Immediate::RuntimeCall(crate::ir::RuntimeCallTarget::Function(target))),
            PhpType::Bool, target.effects(), Some(value.span),
        );
        ctx.builder.terminate(Terminator::CondBr {
            cond: available.value,
            then_target: probe_block.expect("lookup requires a magic probe block"),
            then_args: Vec::new(),
            else_target: get_block,
            else_args: Vec::new(),
        });
    }
    let mut get_initialized = split_initialized.clone();
    let mut false_initialized = split_initialized.clone();
    if let Some(probe_block) = probe_block {
        ctx.builder.position_at_end(probe_block);
        ctx.restore_initialized_slots(split_initialized.clone());
        // Borrow from the held owner: method-call cleanup must not release the
        // reference that is still needed by __get or the missing-value arm.
        let borrowed = ctx.emit_value(
            Op::Borrow, vec![receiver.value], None,
            ctx.builder.value_php_type(receiver.value),
            Op::Borrow.default_effects(), Some(value.span),
        );
        let exists = lower_method_call_with_receiver(
            ctx, borrowed, "__isset",
            &[Expr::new(ExprKind::StringLiteral(property.clone()), value.span)],
            Op::MethodCall, value,
        );
        let exists = ctx.truthy_consuming(exists, Some(value.span));
        get_initialized = ctx.initialized_slots_snapshot();
        false_initialized = merge_initialized_slots_for_expr(
            &split_initialized, get_initialized.clone(), true,
            split_initialized.clone(), true,
        );
        ctx.builder.terminate(Terminator::CondBr {
            cond: exists.value,
            then_target: get_block,
            then_args: Vec::new(),
            else_target: false_block,
            else_args: Vec::new(),
        });
    }

    ctx.builder.position_at_end(false_block);
    ctx.restore_initialized_slots(false_initialized);
    let null = lower_boxed_null(ctx, value);
    store_value_into_temp(ctx, &temp_name, result_type.clone(), null, value.span);
    crate::ir_lower::ownership::release_if_owned(ctx, receiver, Some(value.span));
    let false_reachable = !ctx.builder.insertion_block_is_terminated();
    let false_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge);

    ctx.builder.position_at_end(get_block);
    ctx.restore_initialized_slots(get_initialized);
    let getter_value = lower_method_call_with_receiver(
        ctx, receiver, "__get",
        &[Expr::new(ExprKind::StringLiteral(property.clone()), value.span)],
        Op::MethodCall, value,
    );
    store_value_into_temp(ctx, &temp_name, result_type, getter_value, value.span);
    let get_reachable = !ctx.builder.insertion_block_is_terminated();
    let get_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    ctx.restore_initialized_slots(merge_initialized_slots_for_expr(
        &split_initialized, false_initialized, false_reachable,
        get_initialized, get_reachable,
    ));
    Some(take_owned_temp(ctx, &temp_name, value.span))
}

/// Returns whether a property receiver has a known native class, including nullable storage.
fn property_probe_receiver_uses_native_object_storage(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
) -> bool {
    instance_callable_object_class_and_nullability(ctx, object).is_some()
}

/// Evaluates the receiver of a statically missing concrete-class property and
/// materializes null, matching PHP's warning-free `$object->missing ?? $default` probe.
fn lower_known_missing_property_probe(
    ctx: &mut LoweringContext<'_, '_>,
    value: &Expr,
) -> Option<LoweredValue> {
    let ExprKind::PropertyAccess { object, property } = &value.kind else {
        return None;
    };
    let class_name = instance_callable_object_class(ctx, object)?;
    let class_info = ctx.classes.get(class_name.trim_start_matches('\\'))?;
    if class_info
        .properties
        .iter()
        .any(|(name, _)| name == property)
        || class_info.methods.contains_key("__get")
        || class_info.allow_dynamic_properties
    {
        return None;
    }
    let receiver = lower_expr(ctx, object);
    if ctx.value_is_owning_temporary(receiver) {
        crate::ir_lower::ownership::release_if_owned(ctx, receiver, Some(value.span));
    }
    Some(ctx.emit_value(
        Op::ConstNull,
        Vec::new(),
        None,
        PhpType::Void,
        Op::ConstNull.default_effects(),
        Some(value.span),
    ))
}

/// Returns the materialized result type for a null-coalesce merge.
pub(super) fn null_coalesce_result_type(
    ctx: &LoweringContext<'_, '_>,
    value: ValueId,
    default: &Expr,
) -> PhpType {
    let value_php_type = strip_void_from_union(ctx.builder.value_php_type(value));
    let default_php_type = materialized_expr_type_for_merge(ctx, default);
    let value_ty = value_php_type.codegen_repr();
    let default_ty = default_php_type.codegen_repr();
    // `PhpType::Callable` hides two runtime representations: AOT callables are descriptor
    // pointers, while eval-created Closure values can still be object cells. A `?Closure ??
    // static fn (...) => ...` merge must preserve the selected runtime tag instead of storing
    // both shapes in one untagged callable slot.
    if matches!(&value_ty, PhpType::Callable) && matches!(&default_ty, PhpType::Callable) {
        return PhpType::Mixed;
    }
    // `$x ?? null` keeps the null. `wider_type_for_merge` lets a `Void` side vanish into the
    // other side's type, which is right for storage that can SAY null and wrong for storage
    // that cannot: a string slot stored the null as `""`, and an object slot as a null pointer
    // that `var_dump` printed as nothing and `=== null` answered false for. MEASURED against
    // php 8.5.10, over a missing offset:
    //     ['a'][5] ?? null            php NULL     elephc string(0) ""
    //     [new U()][5] ?? null        php NULL     elephc (nothing), then the program stopped
    // An int slot already came out right, so only the representations that cannot hold null
    // are widened to the boxed cell that can.
    if matches!(default_ty, PhpType::Void)
        && matches!(
            value_ty,
            PhpType::Str
                | PhpType::Object(_)
                | PhpType::Callable
                | PhpType::Array(_)
                | PhpType::AssocArray { .. }
        )
    {
        return PhpType::Mixed;
    }
    wider_type_for_merge(&value_ty, &default_ty)
}

/// Chooses the wider materialized type for branch-local merge storage.
pub(super) fn wider_type_for_merge(left: &PhpType, right: &PhpType) -> PhpType {
    let left = left.codegen_repr();
    let right = right.codegen_repr();
    if left == right {
        return left;
    }
    if matches!(left, PhpType::Void | PhpType::Never) {
        return right;
    }
    if matches!(right, PhpType::Void | PhpType::Never) {
        return left;
    }
    match (&left, &right) {
        // Mismatched element types must widen elementwise (issue #549): letting
        // one side win wholesale relabels the other side's runtime slots, so
        // typed reads through the merged type misinterpret the payload bytes.
        (PhpType::Array(left_elem), PhpType::Array(right_elem)) => {
            PhpType::Array(Box::new(merge_ir_indexed_element_type(
                (**left_elem).clone(),
                (**right_elem).clone(),
            )))
        }
        (
            PhpType::AssocArray { key: left_key, value: left_value },
            PhpType::AssocArray { key: right_key, value: right_value },
        ) => PhpType::AssocArray {
            key: Box::new(merge_ir_assoc_value_type(
                (**left_key).clone(),
                (**right_key).clone(),
            )),
            value: Box::new(merge_ir_assoc_value_type(
                (**left_value).clone(),
                (**right_value).clone(),
            )),
        },
        (
            PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never,
            PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never,
        ) => right.clone(),
        _ => PhpType::Mixed,
    }
}

/// Removes the null sentinel type from nullable unions after a successful `??` value branch.
pub(super) fn strip_void_from_union(php_type: PhpType) -> PhpType {
    let PhpType::Union(members) = php_type else {
        return php_type;
    };
    let mut non_void = members
        .into_iter()
        .filter(|member| !matches!(member, PhpType::Void))
        .collect::<Vec<_>>();
    if non_void.is_empty() {
        PhpType::Void
    } else if non_void.len() == 1 {
        non_void.remove(0)
    } else {
        PhpType::Union(non_void)
    }
}

/// Lowers `expr ?: default`, preserving single evaluation of the first expression.
pub(super) fn lower_short_ternary(
    ctx: &mut LoweringContext<'_, '_>,
    value: &Expr,
    default: &Expr,
    expr: &Expr,
) -> LoweredValue {
    let condition_span = value.span;
    let result_type = short_ternary_merge_result_type(ctx, value, default);
    let value = lower_expr(ctx, value);
    let cond = ctx.truthy(value, Some(condition_span));
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let split_initialized = ctx.initialized_slots_snapshot();
    let value_block = ctx
        .builder
        .create_named_block("short_ternary.value", Vec::new());
    let default_block = ctx
        .builder
        .create_named_block("short_ternary.default", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("short_ternary.merge", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond.value,
        then_target: value_block,
        then_args: Vec::new(),
        else_target: default_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(value_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    store_value_into_temp(ctx, &temp_name, result_type.clone(), value, expr.span);
    let value_reachable = !ctx.builder.insertion_block_is_terminated();
    let value_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge);

    ctx.builder.position_at_end(default_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    store_branch_expr_into_temp(ctx, &temp_name, result_type, default, expr.span);
    release_discarded_branch_value(ctx, value, expr.span);
    let default_reachable = !ctx.builder.insertion_block_is_terminated();
    let default_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    ctx.restore_initialized_slots(merge_initialized_slots_for_expr(
        &split_initialized,
        value_initialized,
        value_reachable,
        default_initialized,
        default_reachable,
    ));
    take_owned_temp(ctx, &temp_name, expr.span)
}

/// Releases a lowered value that a lazy branch tested but did not forward.
pub(super) fn release_discarded_branch_value(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) {
    if ctx.value_needs_release_after_retaining_store(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Lowers `$object[$k] ?? ...` over an `ArrayAccess` receiver the way php does: `offsetExists($k)`
/// first, and `offsetGet($k)` only when that answered true. The plain quiet read called
/// `offsetGet` alone, so a container whose `offsetGet` RESOLVES the key (Laravel's
/// `$this->app['session'] ?? null`) answered for keys `offsetExists` denies, and the read of a
/// `mixed` receiver came back as `int(0)` in a closure.
///
/// Only a receiver that can be read twice without effect takes this form (a variable or a
/// property chain over one, with a side-effect-free offset): the receiver is evaluated once and
/// both calls go through it. Any other shape keeps the quiet read.
fn lower_array_access_offset_exists_coalesce(
    ctx: &mut LoweringContext<'_, '_>,
    array: &Expr,
    index: &Expr,
    expr: &Expr,
) -> Option<LoweredValue> {
    fn repeatable(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Variable(_) | ExprKind::This | ExprKind::StaticPropertyAccess { .. } => true,
            ExprKind::PropertyAccess { object, .. } => repeatable(object),
            _ => false,
        }
    }
    if !repeatable(array) || crate::optimize::expr_has_side_effects(index) {
        return None;
    }
    let receiver = lower_subscript_receiver_silently(ctx, array);
    let receiver_ty = ctx.builder.value_php_type(receiver.value);
    let statically_array_access = type_satisfies_array_access_for_ir(ctx, &receiver_ty);
    let maybe_array_access = matches!(receiver_ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
        && !receiver_ty_is_nullable_only(&receiver_ty)
        && super::native_isset::program_may_hold_array_access_objects(ctx);
    if !(statically_array_access || maybe_array_access) {
        return Some(lower_array_access_from_receiver(ctx, receiver, index, expr, false));
    }
    // The receiver is read by `offsetExists` and then by `offsetGet`, and each call consumes the
    // reference it is handed: a second reference, pinned before the first call, serves the second.
    let pin = crate::ir_lower::ownership::acquire_lifetime_pin_if_refcounted(
        ctx,
        receiver,
        Some(expr.span),
    );
    let temp_name = ctx.declare_owned_hidden_temp(PhpType::Mixed);
    let data = ctx.intern_class_name("ArrayAccess");
    let is_array_access = ctx.emit_value(
        Op::InstanceOf,
        vec![receiver.value],
        Some(Immediate::Data(data)),
        PhpType::Bool,
        Op::InstanceOf.default_effects(),
        Some(expr.span),
    );
    let split_initialized = ctx.initialized_slots_snapshot();
    let access_block = ctx.builder.create_named_block("coalesce.array_access", Vec::new());
    let plain_block = ctx.builder.create_named_block("coalesce.array_plain", Vec::new());
    let get_block = ctx.builder.create_named_block("coalesce.array_get", Vec::new());
    let missing_block = ctx.builder.create_named_block("coalesce.array_missing", Vec::new());
    let merge = ctx.builder.create_named_block("coalesce.array_merge", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_array_access.value,
        then_target: access_block,
        then_args: Vec::new(),
        else_target: plain_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(access_block);
    let exists = lower_method_call_with_receiver(
        ctx,
        receiver,
        "offsetExists",
        &[index.clone()],
        Op::MethodCall,
        expr,
    );
    let exists = ctx.truthy_consuming(exists, Some(expr.span));
    ctx.builder.terminate(Terminator::CondBr {
        cond: exists.value,
        then_target: get_block,
        then_args: Vec::new(),
        else_target: missing_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(get_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    let got = lower_method_call_with_receiver(
        ctx,
        pin,
        "offsetGet",
        &[index.clone()],
        Op::MethodCall,
        expr,
    );
    store_value_into_temp(ctx, &temp_name, PhpType::Mixed, got, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(missing_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    crate::ir_lower::ownership::release_if_owned(ctx, pin, Some(expr.span));
    let null = lower_boxed_null(ctx, expr);
    store_value_into_temp(ctx, &temp_name, PhpType::Mixed, null, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(plain_block);
    ctx.restore_initialized_slots(split_initialized);
    crate::ir_lower::ownership::release_if_owned(ctx, pin, Some(expr.span));
    let read = lower_array_access_from_receiver(ctx, receiver, index, expr, false);
    store_value_into_temp(ctx, &temp_name, PhpType::Mixed, read, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    Some(take_owned_temp(ctx, &temp_name, expr.span))
}

/// Whether a type holds nothing but `null`-like members.
fn receiver_ty_is_nullable_only(ty: &PhpType) -> bool {
    match ty {
        PhpType::Void | PhpType::Never => true,
        PhpType::Union(members) => members.iter().all(|m| matches!(m, PhpType::Void | PhpType::Never)),
        _ => false,
    }
}

/// `store_expr_into_temp` for an expression that runs on ONE path of a conditional expression
/// (`?:`/`??` default, a ternary branch, a `match` arm).
///
/// A local the branch stores (`$c[0] ?: $c = 'z'`, `$f ? ($c = 'z') : 1`) leaves the merge typed by
/// its slot's storage, as an `&&`/`||` operand does (`join_short_circuit_operand_types`). The
/// branch's own view used to survive the merge, so the code after it read a slot the untaken path
/// had left holding an array as a string ("Array to string conversion", `is_array()` false).
pub(super) fn store_branch_expr_into_temp(
    ctx: &mut LoweringContext<'_, '_>,
    temp_name: &str,
    result_type: PhpType,
    branch: &Expr,
    span: crate::span::Span,
) {
    let saved_types = ctx.local_types_snapshot();
    store_expr_into_temp(ctx, temp_name, result_type, branch, span);
    super::unary_logic::join_short_circuit_operand_types(ctx, saved_types);
}
