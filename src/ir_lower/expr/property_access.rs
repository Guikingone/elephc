//! Purpose:
//! Reference assignment and instance or static property reads.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers an object property read.
pub(super) fn lower_property_get(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    op: Op,
    expr: &Expr,
) -> LoweredValue {
    let object = lower_expr(ctx, object);
    lower_property_get_from_value(ctx, object, property, op, expr)
}

/// Lowers `$target = &$obj->prop`: binds the local `$target` to the reference cell
/// stored in the object's reference-property slot, so reads/writes of either side go
/// through the same cell (write-through). The property was promoted to a reference
/// property by the checker, so its slot holds a live cell pointer.
pub(crate) fn lower_ref_assign_property(
    ctx: &mut LoweringContext<'_, '_>,
    target: &str,
    source: &Expr,
    span: Span,
) {
    let ExprKind::PropertyAccess { object, property } = &source.kind else {
        return;
    };
    let object = lower_expr(ctx, object);
    let value_type = property_get_result_type(ctx, object.value, property, Op::PropGet, source);
    let data = ctx.intern_string(property);
    let cell_ptr = ctx.emit_value(
        Op::LoadPropRefCell,
        vec![object.value],
        Some(Immediate::Data(data)),
        value_type.clone(),
        Op::LoadPropRefCell.default_effects(),
        Some(span),
    );
    ctx.bind_local_ref_cell_ptr(target, cell_ptr, value_type, Some(span));
}

/// Lowers a local reference alias to a runtime-named declared property cell.
pub(crate) fn lower_ref_assign_dynamic_property(
    ctx: &mut LoweringContext<'_, '_>,
    target: &str,
    source: &Expr,
    span: Span,
) {
    let ExprKind::DynamicPropertyAccess { object, property } = &source.kind else {
        return;
    };
    let object = lower_expr(ctx, object);
    let property = lower_expr(ctx, property);
    let property = coerce_to_string_at_span(ctx, property, Some(span));
    let value_type = ctx
        .dynamic_ref_local_types
        .get(&(ctx.loop_storage_scope.clone(), target.to_string()))
        .or_else(|| ctx.local_types.get(target))
        .cloned()
        .unwrap_or(PhpType::Mixed);
    let cell_ptr = ctx.emit_value(
        Op::DynamicPropRefCell,
        vec![object.value, property.value],
        None,
        value_type.clone(),
        Op::DynamicPropRefCell.default_effects(),
        Some(span),
    );
    if ctx.value_is_owning_temporary(property) {
        crate::ir_lower::ownership::release_if_owned(ctx, property, Some(span));
    }
    ctx.bind_local_ref_cell_ptr(target, cell_ptr, value_type, Some(span));
}

/// Lowers `$target = &call()`: binds `$target` to the reference cell returned by a
/// by-reference-returning callee. The call yields the cell pointer; the target shares it
/// non-owning (the owner is the object property the callee returned a reference to).
pub(crate) fn lower_ref_assign_call(
    ctx: &mut LoweringContext<'_, '_>,
    target: &str,
    source: &Expr,
    span: Span,
) {
    let cell_ptr = lower_expr(ctx, source);
    let value_type = ctx.builder.value_php_type(cell_ptr.value);
    ctx.bind_local_ref_cell_ptr(target, cell_ptr, value_type, Some(span));
}

/// Lowers `$target =& $arr[idx]`: promotes the array element to reference storage and binds
/// `$target` to it non-owning. Hash promotion may relocate the container, so a static-property
/// receiver is republished before the alias escapes. The array must remain live while the alias
/// is in use because the local does not own its storage.
pub(crate) fn lower_ref_assign_array_elem(
    ctx: &mut LoweringContext<'_, '_>,
    target: &str,
    source: &Expr,
    span: Span,
) {
    let ExprKind::ArrayAccess { array, index } = &source.kind else {
        return;
    };
    let array_value = lower_expr(ctx, array);
    let array_value = prepare_local_array_element_reference_container(
        ctx,
        array,
        array_value,
        span,
    );
    let mut index_value = lower_expr(ctx, index);
    // Use the array's declared element type (the inline storage shape), not the
    // null-capable `TaggedScalar` result type that `array_access_result_type` widens
    // Int elements to. The ref-cell aliases the raw element slot, so loads and stores
    // through the alias must match the element's storage width, not the read result.
    let container_type = ctx.builder.value_php_type(array_value.value).codegen_repr();
    let value_type = match &container_type {
        PhpType::Array(elem_ty) => normalize_value_php_type((**elem_ty).clone()),
        PhpType::AssocArray { value, .. } => normalize_value_php_type((**value).clone()),
        _ => array_access_result_type(ctx, array_value.value, Op::ArrayGet, source),
    };
    let op = if matches!(container_type, PhpType::AssocArray { .. }) {
        Op::HashRefElement
    } else {
        index_value = coerce_to_int_at_span(ctx, index_value, Some(index.span));
        Op::LoadArrayElemRefCell
    };
    let cell_ptr = ctx.emit_value(
        op,
        vec![array_value.value, index_value.value],
        None,
        value_type.clone(),
        op.default_effects(),
        Some(span),
    );
    if op == Op::HashRefElement {
        if let ExprKind::StaticPropertyAccess { receiver, property } = &array.kind {
            crate::ir_lower::stmt::store_static_property(
                ctx,
                receiver,
                property,
                array_value.value,
                span,
            );
        }
    }
    ctx.bind_local_ref_cell_ptr(target, cell_ptr, value_type, Some(span));
}

/// Widens a local array's element storage before exposing one element as a writable reference.
///
/// A PHP reference may later receive a value of any type. Concrete indexed/hash element layouts
/// cannot represent such a write safely, so a plain local source is converted to Mixed-valued
/// storage and written back before its element address is taken. Containers already using Mixed
/// slots and non-local receiver shapes are left unchanged for their dedicated place lowerers.
fn prepare_local_array_element_reference_container(
    ctx: &mut LoweringContext<'_, '_>,
    array: &Expr,
    array_value: LoweredValue,
    span: Span,
) -> LoweredValue {
    let ExprKind::Variable(name) = &array.kind else {
        return array_value;
    };
    let container_type = ctx.builder.value_php_type(array_value.value).codegen_repr();
    let (op, target_type) = match container_type {
        PhpType::Array(element) if element.codegen_repr() != PhpType::Mixed => (
            Op::ArrayToMixed,
            PhpType::Array(Box::new(PhpType::Mixed)),
        ),
        PhpType::AssocArray { key, value } if value.codegen_repr() != PhpType::Mixed => (
            Op::HashToMixed,
            PhpType::AssocArray {
                key,
                value: Box::new(PhpType::Mixed),
            },
        ),
        _ => return array_value,
    };
    let converted = ctx.emit_value(
        op,
        vec![array_value.value],
        None,
        target_type.clone(),
        op.default_effects(),
        Some(span),
    );
    ctx.store_mutated_local(name, converted, target_type, Some(span));
    ctx.load_local(name, Some(span))
}

/// Lowers a named property read once the receiver is already evaluated.
pub(super) fn lower_property_get_from_value(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    op: Op,
    expr: &Expr,
) -> LoweredValue {
    if op == Op::NullsafePropGet && value_is_definitely_null(ctx, object.value) {
        return lower_boxed_null(ctx, expr);
    }
    if property_receiver_is_unresolved_nominal_class(ctx, object.value) {
        // PHP declarations may name a class that is absent until a runtime path actually
        // constructs it. Preserve a property read in such a body as a gradual class-id dispatch
        // instead of requiring a static slot layout during AOT lowering.
        let object = ctx.box_value_as_mixed(object, PhpType::Mixed, Some(expr.span));
        let property_expr = Expr::new(ExprKind::StringLiteral(property.to_string()), expr.span);
        return lower_dynamic_property_get_from_value(ctx, object, &property_expr, expr);
    }
    // Route a read of a get-hooked property to its synthetic accessor, except inside that property's
    // own accessor, where `$this->prop` must read the raw backing slot to avoid infinite recursion.
    // A nullsafe read (`$obj?->prop`) routes to a nullsafe call so the null short-circuit is kept.
    if matches!(op, Op::PropGet | Op::NullsafePropGet)
        && class_declares_hook_accessor(ctx, object.value, &property_hook_get_method(property))
        && !ctx.in_own_property_accessor(property)
    {
        let accessor = property_hook_get_method(property);
        let call_op = if op == Op::NullsafePropGet {
            Op::NullsafeMethodCall
        } else {
            Op::MethodCall
        };
        return lower_method_call_with_receiver(ctx, object, &accessor, &[], call_op, expr);
    }
    let data = ctx.intern_string(property);
    let result_type = property_get_result_type(ctx, object.value, property, op, expr);
    let result = ctx.emit_value(
        op,
        vec![object.value],
        Some(Immediate::Data(data)),
        result_type,
        op.default_effects(),
        Some(expr.span),
    );
    stabilize_borrowed_result_and_release_receiver(ctx, object, result, expr.span)
}

/// Returns whether a nominal object type has no declaration metadata in this compilation unit.
fn property_receiver_is_unresolved_nominal_class(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
) -> bool {
    let object_type = ctx.builder.value_php_type(object);
    let Some((class_name, _)) = singular_object_class(&object_type) else {
        return false;
    };
    let class_name = class_name.trim_start_matches('\\');
    !class_name.is_empty()
        && !ctx.classes.contains_key(class_name)
        && !ctx.interfaces.contains_key(class_name)
        && !ctx.packed_classes.contains_key(class_name)
}

/// Lowers a direct backing-slot read without invoking a declared property get hook.
pub(super) fn lower_raw_property_get_from_value(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    expr: &Expr,
) -> LoweredValue {
    let data = ctx.intern_string(property);
    let result_type = property_get_result_type(ctx, object.value, property, Op::PropGet, expr);
    let result = ctx.emit_value(
        Op::PropGet,
        vec![object.value],
        Some(Immediate::Data(data)),
        result_type,
        Op::PropGet.default_effects(),
        Some(expr.span),
    );
    stabilize_borrowed_result_and_release_receiver(ctx, object, result, expr.span)
}

/// Returns true when value metadata proves the runtime value is PHP null.
pub(super) fn value_is_definitely_null(ctx: &LoweringContext<'_, '_>, value: crate::ir::ValueId) -> bool {
    matches!(ctx.builder.value_php_type(value), PhpType::Void | PhpType::Never)
}

/// Returns true when value metadata permits PHP null at runtime.
pub(super) fn value_is_nullable(ctx: &LoweringContext<'_, '_>, value: crate::ir::ValueId) -> bool {
    match ctx.builder.value_php_type(value) {
        PhpType::Void | PhpType::Never | PhpType::Mixed => true,
        PhpType::Union(members) => members
            .iter()
            .any(|member| matches!(member, PhpType::Void | PhpType::Mixed)),
        _ => false,
    }
}

/// Returns precise PHP metadata for a named property read when class metadata is available.
pub(super) fn property_get_result_type(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    op: Op,
    expr: &Expr,
) -> PhpType {
    if let Some(property_ty) = ctx
        .flow_typed_property_accesses
        .get(&(ctx.loop_storage_scope.clone(), expr.span))
    {
        return normalize_value_php_type(property_ty.clone());
    }
    if op == Op::NullsafePropGet {
        return PhpType::Mixed;
    }
    let object_ty = ctx.builder.value_php_type(object);
    let Some((class_name, nullable)) = singular_object_class(&object_ty) else {
        if matches!(object_ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_)) {
            return PhpType::Mixed;
        }
        if let PhpType::Packed(class_name) = object_ty.codegen_repr() {
            let normalized = class_name.trim_start_matches('\\');
            let Some(class_info) = ctx.packed_classes.get(normalized) else {
                return fallback_expr_type(expr);
            };
            let Some(field) = class_info.fields.iter().find(|field| field.name == property) else {
                return fallback_expr_type(expr);
            };
            return normalize_value_php_type(field.php_type.codegen_repr());
        }
        return fallback_expr_type(expr);
    };
    let nullable = nullable || value_may_carry_container_miss(ctx, object);
    let normalized = class_name.trim_start_matches('\\');
    if normalized.is_empty() {
        return if nullable {
            nullable_result_type(PhpType::Mixed)
        } else {
            PhpType::Mixed
        };
    }
    if is_builtin_stdclass_name(normalized) {
        return if nullable {
            nullable_result_type(PhpType::Mixed)
        } else {
            PhpType::Mixed
        };
    }
    if let Some(property_ty) =
        crate::types::checker::reflection_virtual_property_type(normalized, property)
    {
        return if nullable {
            nullable_result_type(property_ty)
        } else {
            property_ty
        };
    }
    if let Some(property_ty) = interface_property_result_type(ctx, normalized, property) {
        return if nullable {
            nullable_result_type(property_ty)
        } else {
            property_ty
        };
    }
    let Some(class_info) = ctx.classes.get(normalized) else {
        return fallback_expr_type(expr);
    };
    let property = crate::types::checker::reflection_virtual_property_backing(normalized, property)
        .unwrap_or(property);
    if let Some(property_ty) = runtime_property_type_override(ctx, normalized, property) {
        let property_ty = normalize_value_php_type(property_ty);
        return if nullable {
            nullable_result_type(property_ty)
        } else {
            property_ty
        };
    }
    if !property_is_accessible_for_ir(ctx, normalized, class_info, property) {
        if let Some(magic_ty) = magic_get_result_type(ctx, normalized) {
            return if nullable {
                nullable_result_type(magic_ty)
            } else {
                magic_ty
            };
        }
    }
    let Some((_, (_, property_ty))) = class_info.visible_property(property) else {
        if let Some(magic_ty) = magic_get_result_type(ctx, normalized) {
            return if nullable {
                nullable_result_type(magic_ty)
            } else {
                magic_ty
            };
        }
        if class_info.allow_dynamic_properties {
            return if nullable {
                nullable_result_type(PhpType::Mixed)
            } else {
                PhpType::Mixed
            };
        }
        return fallback_expr_type(expr);
    };
    let property_ty = normalize_value_php_type(property_ty.clone());
    if nullable {
        nullable_result_type(property_ty)
    } else {
        property_ty
    }
}

/// Returns the readable property type guaranteed by an interface contract or enum interface.
fn interface_property_result_type(
    ctx: &LoweringContext<'_, '_>,
    interface_name: &str,
    property: &str,
) -> Option<PhpType> {
    let interface = ctx.interfaces.get(interface_name)?;
    if let Some(property_ty) = interface
        .properties
        .get(property)
        .and_then(|contract| contract.get_type.clone())
    {
        return Some(property_ty);
    }
    if property == "name"
        && interface_extends_interface_for_ir(ctx, interface_name, "UnitEnum")
    {
        return Some(PhpType::Str);
    }
    if property == "value"
        && interface_extends_interface_for_ir(ctx, interface_name, "BackedEnum")
    {
        return Some(PhpType::Union(vec![PhpType::Int, PhpType::Str]));
    }
    None
}

/// Returns whether a container read can carry PHP null in a statically non-null pointer type.
pub(super) fn value_may_carry_container_miss(
    ctx: &LoweringContext<'_, '_>,
    value: crate::ir::ValueId,
) -> bool {
    let Some(inst) = ctx.builder.value_defining_instruction(value) else {
        return false;
    };
    match inst.op {
        Op::ArrayGet | Op::ArrayGetSilent | Op::HashGet | Op::HashGetSilent => true,
        Op::Acquire => inst
            .operands
            .first()
            .copied()
            .is_some_and(|source| value_may_carry_container_miss(ctx, source)),
        _ => false,
    }
}

/// Returns the normalized return type for a class `__get` magic property hook.
pub(super) fn magic_get_result_type(ctx: &LoweringContext<'_, '_>, class_name: &str) -> Option<PhpType> {
    class_method_signature(ctx, class_name, &php_symbol_key("__get"))
        .map(|signature| normalize_value_php_type(signature.return_type.clone()))
}

/// Adds nullability to a result type without nesting existing union metadata.
pub(super) fn nullable_result_type(php_type: PhpType) -> PhpType {
    match php_type {
        PhpType::Union(mut members) => {
            if !members.iter().any(|member| matches!(member, PhpType::Void)) {
                members.push(PhpType::Void);
            }
            PhpType::Union(members)
        }
        other => PhpType::Union(vec![other, PhpType::Void]),
    }
}

/// Returns true when the runtime class of `object` declares the synthetic property-hook accessor
/// `accessor_method` (`__propget_<p>` / `__propset_<p>`). Drives the decision to route a property
/// read/write to a hook; inherited (flattened) methods count, so subclasses inherit hooks.
pub(super) fn class_declares_hook_accessor(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    accessor_method: &str,
) -> bool {
    let object_ty = ctx.builder.value_php_type(object);
    let Some((class_name, _nullable)) = singular_object_class(&object_ty) else {
        return false;
    };
    let key = php_symbol_key(accessor_method);
    ctx.classes
        .get(class_name)
        .is_some_and(|info| info.methods.contains_key(&key))
}

/// Returns true when reading `property` on `object` can hit PHP's
/// "must not be accessed before initialization" fatal.
///
/// A property is uninitialized only while it is DECLARED WITH A TYPE and has no default:
/// `public ?P $p;` and `public string $s;` both start uninitialized, and PHP fatals on a plain
/// read of either. A default makes the slot live before the constructor body runs, and an
/// untyped property is plain null, so neither can ever be in that state — which is what keeps
/// this gate off the overwhelmingly common shapes.
///
/// The one case it misses is `unset($this->s)`, which returns an already-initialized typed
/// property to the uninitialized state in PHP.
pub(super) fn property_can_be_uninitialized(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> bool {
    let object_ty = ctx.builder.value_php_type(object);
    let Some((class_name, nullable)) = singular_object_class(&object_ty) else {
        // No single receiver class to name a slot in — an untyped parameter, a `mixed` local, a
        // union of two classes. `Op::PropInitialized` dispatches on the runtime class instead and
        // answers "initialized" for every shape it cannot settle, so the ordinary read still
        // happens wherever it used to. Turning this away sent `$untyped->typedProp ?? "d"` to the
        // plain read, which fatals where PHP answers the default.
        return !matches!(object_ty.codegen_repr(), PhpType::Object(_));
    };
    // `Op::PropInitialized` reads a slot, so it needs an object pointer. A concrete `C`
    // receiver already is one; a `?C` one represents as a boxed `Mixed` and the backend
    // unboxes it, answering FALSE for a null receiver — which is the answer `??` wants there
    // anyway, since `null->p ?? "d"` is the default. Every other boxed shape (plain `Mixed`, a
    // union carrying a scalar arm or two classes) has no single slot to probe and is turned
    // away here, keeping the ordinary read it had before.
    if !nullable && !matches!(object_ty.codegen_repr(), PhpType::Object(_)) {
        return false;
    }
    // A get-HOOKED property has no slot to probe: its value comes from the synthetic accessor
    // that `lower_property_get_from_value` routes to, and the backing slot behind it is
    // legitimately uninitialized. Probing it answers "not initialized" and sends `??` to its
    // default, so `$p?->full ?? "(none)"` on a real object answered `(none)` instead of running
    // the hook. Inside the accessor itself `$this->full` IS the raw slot, which is the one place
    // the probe applies — the same exception the read makes.
    if class_declares_hook_accessor(ctx, object, &property_hook_get_method(property))
        && !ctx.in_own_property_accessor(property)
    {
        return false;
    }
    let Some(info) = ctx.classes.get(class_name) else {
        return false;
    };
    let Some(index) = info.properties.iter().position(|(name, _)| name == property) else {
        return false;
    };
    // Whether the slot was DECLARED with a type — asked of the schema, not inferred from the
    // stored `PhpType`. Both questions agree on `?string`, which represents as `Mixed` and is
    // still declared, and on an untyped `public $x;`, which is plain null from the start and
    // must stay on the ordinary path. They disagree on `public mixed $x;`: it IS declared and
    // starts uninitialized, but its type is literally `Mixed`, so a test on the representation
    // read it as untyped and `$o->x ?? "d"` raised where PHP answers the default.
    //
    // A DEFAULT does not exclude the property. It used to: a defaulted slot is live from
    // construction, so it looked as though it could never be uninitialized. `unset($o->x)`
    // returns a typed property to the uninitialized state whatever its default, and the
    // ordinary read then raises where PHP's `??` answers the default. The runtime probe
    // settles both cases, so the gate asks only whether the property is TYPED.
    info.property_slot_is_declared(index, property)
}

/// Reads `property` the way `isset()` does: yields null instead of raising when the slot is
/// still uninitialized.
///
/// `??` must not fatal on `$o->p` — PHP answers the default — but the ordinary read does. The
/// initialized-aware read already exists for `isset()`, which produces a BOOLEAN; this is its
/// value-producing twin, and it is entered only for the properties
/// `property_can_be_uninitialized` admits, so every other read keeps its exact slot type.
pub(super) fn lower_initialized_property_value(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    expr: &Expr,
) -> LoweredValue {
    let temp_name = ctx.declare_hidden_temp(PhpType::Mixed);
    let uninitialized_block = ctx
        .builder
        .create_named_block("coalesce.property.uninitialized", Vec::new());
    let read_block = ctx
        .builder
        .create_named_block("coalesce.property.read", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("coalesce.property.merge", Vec::new());
    let data = ctx.intern_string(property);
    let initialized = ctx.emit_value(
        Op::PropInitialized,
        vec![object.value],
        Some(Immediate::Data(data)),
        PhpType::Bool,
        Op::PropInitialized.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: initialized.value,
        then_target: read_block,
        then_args: Vec::new(),
        else_target: uninitialized_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(uninitialized_block);
    // This path never reads the property, so nothing downstream consumes the receiver — an
    // OWNING one has to be released here. The read path below hands it to
    // `lower_property_get_from_value`, which disposes of it the way an ordinary read does.
    // `mk()->p ?? "none"` leaked one object per call without this.
    //
    // Only an OWNING one: a plain `$c` receiver is BORROWED from its slot, and releasing it
    // hands back a reference this expression never took. `$c->p ??= 42` through a `?C`
    // parameter died with "Attempt to assign property on null" — the release freed the boxed
    // receiver, and the write that followed read the freed cell. `guard_initialized_chain_property`
    // gates its own cleanup block the same way.
    if ctx.value_is_owning_temporary(object) {
        crate::ir_lower::ownership::release_if_owned(ctx, object, Some(expr.span));
    }
    let null_value = lower_boxed_null(ctx, expr);
    store_value_into_temp(ctx, &temp_name, PhpType::Mixed, null_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(read_block);
    let read_value = lower_property_get_from_value(ctx, object, property, Op::PropGet, expr);
    // Both arms store into one Mixed temporary, so a slot that is not already boxed has to be.
    let read_value = if matches!(
        ctx.builder.value_php_type(read_value.value).codegen_repr(),
        PhpType::Mixed | PhpType::Union(_)
    ) {
        read_value
    } else {
        ctx.emit_value(
            Op::MixedBox,
            vec![read_value.value],
            None,
            PhpType::Mixed,
            Op::MixedBox.default_effects(),
            Some(expr.span),
        )
    };
    store_value_into_temp(ctx, &temp_name, PhpType::Mixed, read_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    take_owned_temp(ctx, &temp_name, expr.span)
}

/// Returns true when reading `property` on `receiver` can hit PHP's "must not be accessed
/// before initialization" fatal for a STATIC slot.
///
/// The rule is the instance one: a static property is uninitialized only while it is DECLARED
/// WITH A TYPE. `public static $u;` is plain null from the start, and a receiver whose class
/// is not known statically cannot be probed at all.
///
/// Unlike the instance gate there is no defaulted-slot question to settle here: `unset()` does
/// not apply to a static property, so a default really does make the slot live for good — but
/// asking only "is it typed" costs one sentinel compare on a slot that can never carry the
/// sentinel, and keeps the two gates reading the same way.
pub(super) fn static_property_can_be_uninitialized(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
) -> bool {
    let Some(class_name) = static_receiver_class_name(ctx, receiver) else {
        return false;
    };
    let Some(class_info) = ctx.classes.get(class_name.as_str()) else {
        return false;
    };
    if !class_info
        .static_properties
        .iter()
        .any(|(name, _)| name == property)
    {
        return false;
    }
    // Whether the slot was DECLARED with a type, asked of the schema rather than inferred from
    // the stored `PhpType`. `public static mixed $s;` is declared AND stores `Mixed`, so a test
    // on the representation read it as untyped and `S::$s ?? "d"` raised where PHP answers the
    // default — the same confusion the instance predicate above had.
    class_info.declared_static_properties.contains(property)
}

/// Reads a static `property` the way `isset()` does: yields null instead of raising when the
/// slot is still uninitialized.
///
/// The instance twin (`lower_initialized_property_value`) branches on `Op::PropInitialized`.
/// The static path had no such operation — its guard is emitted straight into the read — so
/// `S::$s ?? "d"` raised where PHP answers the default. `Op::StaticPropInitialized` is that
/// operation; the probe it lowers to already existed for Reflection and only needed a
/// visibility-enforcing entry point.
///
/// There is no receiver to own or release here, which is the whole difference from the
/// instance form.
pub(super) fn lower_initialized_static_property_value(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    expr: &Expr,
) -> LoweredValue {
    let temp_name = ctx.declare_hidden_temp(PhpType::Mixed);
    let uninitialized_block = ctx
        .builder
        .create_named_block("coalesce.static_property.uninitialized", Vec::new());
    let read_block = ctx
        .builder
        .create_named_block("coalesce.static_property.read", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("coalesce.static_property.merge", Vec::new());
    let name = format!("{}::{}", receiver_name(receiver), property);
    let data = ctx.intern_string(&name);
    let initialized = ctx.emit_value(
        Op::StaticPropInitialized,
        Vec::new(),
        Some(Immediate::Data(data)),
        PhpType::Bool,
        Op::StaticPropInitialized.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: initialized.value,
        then_target: read_block,
        then_args: Vec::new(),
        else_target: uninitialized_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(uninitialized_block);
    let null_value = lower_boxed_null(ctx, expr);
    store_value_into_temp(ctx, &temp_name, PhpType::Mixed, null_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(read_block);
    let read_value = lower_static_property_get(ctx, receiver, property, expr);
    // Both arms store into one Mixed temporary, so a slot that is not already boxed has to be.
    let read_value = if matches!(
        ctx.builder.value_php_type(read_value.value).codegen_repr(),
        PhpType::Mixed | PhpType::Union(_)
    ) {
        read_value
    } else {
        ctx.emit_value(
            Op::MixedBox,
            vec![read_value.value],
            None,
            PhpType::Mixed,
            Op::MixedBox.default_effects(),
            Some(expr.span),
        )
    };
    store_value_into_temp(ctx, &temp_name, PhpType::Mixed, read_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    take_owned_temp(ctx, &temp_name, expr.span)
}

/// Returns the class name and nullability if `php_type` is a single object type (optionally
/// nullable). Heterogeneous unions and non-object types return `None`.
pub(super) fn singular_object_class(php_type: &PhpType) -> Option<(&str, bool)> {
    match php_type {
        PhpType::Object(name) => Some((name.as_str(), false)),
        PhpType::Union(members) => {
            let mut found = None;
            let mut nullable = false;
            for member in members {
                match member {
                    PhpType::Void => nullable = true,
                    PhpType::Object(name) => {
                        if found.is_some_and(|existing| existing != name.as_str()) {
                            return None;
                        }
                        found = Some(name.as_str());
                    }
                    _ => return None,
                }
            }
            found.map(|class_name| (class_name, nullable))
        }
        _ => None,
    }
}

/// Returns precise runtime storage types for inherited SPL callback-filter internals.
pub(super) fn runtime_property_type_override(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
    property: &str,
) -> Option<PhpType> {
    if !class_extends_class(ctx, class_name, "CallbackFilterIterator") {
        return None;
    }
    match property {
        "callback" => Some(PhpType::Callable),
        "callbackEnv" => Some(PhpType::Pointer(None)),
        _ => None,
    }
}

/// Returns true when a class is or extends the target class.
pub(super) fn class_extends_class(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
    target_class: &str,
) -> bool {
    let target_key = php_symbol_key(target_class);
    let mut current = Some(class_name.trim_start_matches('\\').to_string());
    while let Some(name) = current {
        if php_symbol_key(&name) == target_key {
            return true;
        }
        current = ctx
            .classes
            .get(name.as_str())
            .and_then(|class_info| class_info.parent.clone());
    }
    false
}

/// Lowers a dynamic property read.
pub(super) fn lower_dynamic_property_get(ctx: &mut LoweringContext<'_, '_>, object: &Expr, property: &Expr, expr: &Expr) -> LoweredValue {
    let object = lower_expr(ctx, object);
    lower_dynamic_property_get_from_value(ctx, object, property, expr)
}

/// Lowers a dynamic property read once the receiver is already evaluated.
pub(super) fn lower_dynamic_property_get_from_value(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &Expr,
    expr: &Expr,
) -> LoweredValue {
    let result_type = dynamic_property_get_result_type(ctx, object.value, property, expr);
    let object = box_generic_object_for_dynamic_property(ctx, object, expr.span);
    let property = lower_expr(ctx, property);
    let property = coerce_to_string_at_span(ctx, property, Some(expr.span));
    let result = ctx.emit_value(
        Op::DynamicPropGet,
        vec![object.value, property.value],
        None,
        result_type,
        Op::DynamicPropGet.default_effects(),
        Some(expr.span),
    );
    if ctx.value_is_owning_temporary(property) {
        crate::ir_lower::ownership::release_if_owned(ctx, property, Some(expr.span));
    }
    stabilize_borrowed_result_and_release_receiver(ctx, object, result, expr.span)
}

/// Boxes a bare `object` receiver so runtime-name property dispatch can inspect its class id.
pub(super) fn box_generic_object_for_dynamic_property(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    span: Span,
) -> LoweredValue {
    if matches!(
        ctx.builder.value_php_type(object.value).codegen_repr(),
        PhpType::Object(class_name) if class_name.trim_start_matches('\\').is_empty()
    ) {
        ctx.box_value_as_mixed(object, PhpType::Mixed, Some(span))
    } else {
        object
    }
}

/// Returns precise metadata for dynamic property reads when class slots are statically known.
pub(super) fn dynamic_property_get_result_type(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &Expr,
    expr: &Expr,
) -> PhpType {
    if let ExprKind::StringLiteral(name) = &property.kind {
        return property_get_result_type(ctx, object, name, Op::DynamicPropGet, expr);
    }
    let object_ty = ctx.builder.value_php_type(object);
    if matches!(object_ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_)) {
        return PhpType::Mixed;
    }
    let Some((class_name, nullable)) = singular_object_class(&object_ty) else {
        return fallback_expr_type(expr);
    };
    let nullable = nullable || value_may_carry_container_miss(ctx, object);
    let normalized = class_name.trim_start_matches('\\');
    if is_builtin_stdclass_name(normalized) {
        return if nullable {
            nullable_result_type(PhpType::Mixed)
        } else {
            PhpType::Mixed
        };
    }
    let Some(class_info) = ctx.classes.get(normalized) else {
        return fallback_expr_type(expr);
    };
    let members = class_info
        .properties
        .iter()
        .map(|(_, property_ty)| {
            let property_ty = normalize_value_php_type(property_ty.clone());
            if nullable {
                nullable_result_type(property_ty)
            } else {
                property_ty
            }
        })
        .collect::<Vec<_>>();
    normalize_union_members(members).unwrap_or_else(|| fallback_expr_type(expr))
}

/// Returns true when the normalized class name refers to PHP's builtin stdClass.
pub(super) fn is_builtin_stdclass_name(class_name: &str) -> bool {
    crate::types::checker::builtin_stdclass::is_stdclass(class_name)
}

/// Flattens and deduplicates union candidates, with `Mixed` absorbing all members.
pub(super) fn normalize_union_members(members: Vec<PhpType>) -> Option<PhpType> {
    let mut deduped = Vec::new();
    for member in members {
        match member {
            PhpType::Union(inner) => {
                for inner_member in inner {
                    if inner_member == PhpType::Mixed {
                        return Some(PhpType::Mixed);
                    }
                    if !deduped.iter().any(|existing| existing == &inner_member) {
                        deduped.push(inner_member);
                    }
                }
            }
            PhpType::Mixed => return Some(PhpType::Mixed),
            other => {
                if !deduped.iter().any(|existing| existing == &other) {
                    deduped.push(other);
                }
            }
        }
    }
    match deduped.len() {
        0 => None,
        1 => deduped.pop(),
        _ => Some(PhpType::Union(deduped)),
    }
}

/// Lowers a static property read.
pub(super) fn lower_static_property_get(ctx: &mut LoweringContext<'_, '_>, receiver: &StaticReceiver, property: &str, expr: &Expr) -> LoweredValue {
    let name = format!("{}::{}", receiver_name(receiver), property);
    let data = ctx.intern_string(&name);
    let result_type = static_property_result_type(ctx, receiver, property, expr);
    ctx.emit_value(
        Op::LoadStaticProperty,
        Vec::new(),
        Some(Immediate::Data(data)),
        result_type,
        Op::LoadStaticProperty.default_effects(),
        Some(expr.span),
    )
}

/// Lowers a static-property read whose property name is computed at runtime.
pub(super) fn lower_dynamic_static_property_get(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &Expr,
    expr: &Expr,
) -> LoweredValue {
    let name = lower_expr(ctx, property);
    let name = coerce_to_string_at_span(ctx, name, Some(property.span));
    let class_name =
        static_receiver_class_name(ctx, receiver).unwrap_or_else(|| receiver_name(receiver));
    let data = ctx.intern_string(&class_name);
    let result_type = dynamic_static_property_result_type(ctx, receiver);
    ctx.emit_value(
        Op::LoadDynamicStaticProperty,
        vec![name.value],
        Some(Immediate::Data(data)),
        result_type,
        Op::LoadDynamicStaticProperty.default_effects(),
        Some(expr.span),
    )
}

/// Returns common static-property metadata for a runtime-name read.
fn dynamic_static_property_result_type(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> PhpType {
    let Some(class_name) = static_receiver_class_name(ctx, receiver) else {
        return PhpType::Mixed;
    };
    let Some(class_info) = ctx.classes.get(class_name.as_str()) else {
        return PhpType::Mixed;
    };
    let mut types = class_info
        .static_properties
        .iter()
        .map(|(_, ty)| normalize_value_php_type(ty.codegen_repr()));
    let Some(first) = types.next() else {
        return PhpType::Mixed;
    };
    if types.all(|ty| ty == first) {
        first
    } else {
        PhpType::Mixed
    }
}

/// Returns precise PHP metadata for a static property read when class metadata is available.
pub(super) fn static_property_result_type(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    _expr: &Expr,
) -> PhpType {
    let Some(class_name) = static_receiver_class_name(ctx, receiver) else {
        return PhpType::Mixed;
    };
    let Some(class_info) = ctx.classes.get(class_name.as_str()) else {
        return PhpType::Mixed;
    };
    let Some((_, property_ty)) = class_info
        .static_properties
        .iter()
        .find(|(name, _)| name == property)
    else {
        return PhpType::Mixed;
    };
    normalize_value_php_type(property_ty.codegen_repr())
}
