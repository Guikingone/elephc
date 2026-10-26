//! Purpose:
//! Static property writes and static array mutations.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Returns the value type stored by an array/hash container used for reference binding.
fn reference_element_type(container: &PhpType) -> PhpType {
    match container.codegen_repr() {
        PhpType::AssocArray { value, .. } => (*value).clone(),
        PhpType::Array(element) => (*element).clone(),
        _ => PhpType::Mixed,
    }
}

/// Lowers a static property write.
///
/// A static property outlives the enclosing scope, so it must hold its own
/// reference to a refcounted value. There are two storage disciplines, matched
/// to what the codegen store actually does:
///
/// - **Independent-value store** (the store crosses the boxed/unboxed boundary, in
///   either direction): a Mixed/Union slot receiving a non-Mixed value, e.g.
///   `Class::$h = new C()`, is boxed with `__rt_mixed_from_value`, which takes its
///   *own* retained reference to the child; and a narrowing slot receiving a Mixed
///   value, e.g. `Class::$s += 1` on an `int` slot, is cast out of the box, so the
///   slot holds a payload word that owns nothing of it. Either way the slot keeps a
///   reference independent of the source, so an owning temporary must be *released*
///   after the store (its reference is not the one the slot holds), and a borrowed
///   source must be left untouched. Acquiring here would leak the extra reference on
///   top of the slot's, and skipping the release leaks the source's own -- which is
///   what made every compound assignment to a typed static property leak one Mixed
///   cell (issue #1041).
/// - **Moving store** (every other case: a same-representation store, or a Mixed
///   value into a slot codegen leaves untouched): the store consumes (moves) its
///   value operand. An owning temporary is moved in as-is, but a *borrowed* value (a
///   parameter, local, or container read) must be `Acquire`d first. Without this,
///   storing a borrowed `Mixed` (e.g. `Class::$h = $handler` where `$handler` is a
///   `?SessionHandlerInterface` parameter) leaves the property dangling once the
///   borrow's owner releases its reference, so a later read dispatches on freed
///   memory (a fatal "on null").
///
///   A heap load from a local slot only *looks* like an owning temporary. Main
///   classifies concrete array/object local loads that way for provisional
///   unbox-release tracking, but the slot stays the owner and the frame epilogue
///   releases it, so moving from one hands the property a reference the epilogue
///   then takes back. `throw $e` reached the same conclusion first (issue #448):
///   publishing a pointer somewhere that outlives the frame is not a transfer.
///   A static property outlives it by the widest margin there is — the process —
///   so a local-slot load is acquired here like any other borrow.
pub(super) fn lower_static_property_assign(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: &Expr,
    span: Span,
) {
    let source = lower_expr(ctx, value);
    // A DECLARED type is enforced (php's TypeError for a wrong value); an untyped slot's inferred
    // type is only a storage shape, so its coercion stays unguarded.
    let declared = static_receiver_class_name(ctx, receiver).is_some_and(|class_name| {
        ctx.classes
            .get(class_name.as_str())
            .is_some_and(|class_info| class_info.declared_static_properties.contains(property))
    });
    let mut source = static_property_type(ctx, receiver, property)
        .map(|slot_ty| {
            if declared {
                super::typed_foreach::coerce_property_assign_value(ctx, source, &slot_ty, span)
            } else {
                coerce_typed_assign_value(ctx, source, &slot_ty, span)
            }
        })
        .unwrap_or(source);
    // The checker can see a narrowed array where lowering still holds the boxed value -- a
    // parameter typed `array|string|null`, narrowed by `is_array()` and fed through `array_map()`,
    // stays a Mixed local here. A list slot with concrete elements cannot take the box (its
    // elements are raw); polyfill-mbstring's `mb_detect_order()` writing `self::$encodingList` is
    // this shape. PHP copies the array on assignment anyway, so the store takes a fresh list of
    // the slot's element type, each element converted with the same guarded coercion a spread
    // uses.
    if matches!(
        ctx.builder.value_php_type(source.value).codegen_repr(),
        PhpType::Mixed | PhpType::Union(_)
    ) {
        if let Some(PhpType::Array(element)) = static_property_type(ctx, receiver, property) {
            if element.codegen_repr() != PhpType::Mixed {
                let list = ctx.emit_value(
                    Op::ArrayNew,
                    Vec::new(),
                    Some(Immediate::Capacity(0)),
                    PhpType::Array(element.clone()),
                    Op::ArrayNew.default_effects(),
                    Some(span),
                );
                crate::ir_lower::expr::lower_indexed_array_spread_into_array(
                    ctx,
                    list,
                    source,
                    Some(&*element),
                    span,
                );
                source = list;
            }
        }
    }
    if static_property_store_retains_independent_value(ctx, receiver, property, source) {
        store_static_property(ctx, receiver, property, source.value, span);
        if ctx.value_is_owning_temporary(source) {
            crate::ir_lower::ownership::release_if_owned(ctx, source, Some(span));
        }
        return;
    }
    // A `Str` is PERSISTED into the slot whatever its ownership says, and the source is then
    // released. Ownership is the wrong question for a string here: "owning" distinguishes who
    // must release, not WHERE the bytes live, and a great many string producers hand back a
    // pointer into the shared 64 KiB `_concat_buf` scratch. Moving such a pointer into a slot
    // that outlives the statement stores the right LENGTH over bytes the next producer
    // overwrites — `B::$s = strtoupper($x)` read back a later `str_repeat`'s bytes.
    //
    // Fixing this at the producer instead (reclassifying runtime-call `Str` results as
    // non-owning) was tried and is wrong twice over: it only reaches the producers someone
    // thought to list — `$x . "c"`, `"v=$x!"`, `(string)$i`, `strval($i)` all still clobbered —
    // and above 64 KiB `__rt_concat_reserve` returns an OWNED heap block, so suppressing the
    // release leaked one block per call. The consumer is the only place that knows the value is
    // about to outlive the frame, so it is the only place the question can be answered once.
    //
    // The three storage classes all land correctly, which is why this is safe rather than
    // merely conservative:
    // - scratch slice: `Acquire` is `__rt_str_persist`, which duplicates it onto the heap; the
    //   paired `Release` reaches `__rt_heap_free_safe`, which skips a non-heap pointer.
    // - owned heap block (a unary-string result over 64 KiB): duplicated, then the original is
    //   freed — this is the leak the producer-side attempt introduced.
    // - `CONCAT_TEMP_HEAP_KIND` block (a `.` result over 64 KiB): `__rt_str_persist` takes it
    //   over IN PLACE, and the release cannot double-free it because codegen classifies
    //   `Op::StrConcat` as a scratch string and drops its `Release` entirely.
    //
    // `store_local` reached the same answer for `static` LOCALS at `context.rs`'s
    // `static_local_store_needs_string_retain`; a static property has the identical lifetime
    // problem and now has the identical rule.
    let stores_a_string = matches!(
        ctx.builder.value_php_type(source.value).codegen_repr(),
        PhpType::Str
    );
    let source_is_owning_temporary = ctx.value_is_owning_temporary(source);
    let provisional_load = ctx.value_is_owned_unboxed_local_load(source.value);
    let stored = if stores_a_string || !source_is_owning_temporary || provisional_load {
        crate::ir_lower::ownership::acquire_if_refcounted(ctx, source, Some(span))
    } else {
        source
    };
    // A concrete local load can still be borrowed after final frame typing.
    // Retain its published owner, then retire only an actual Mixed unbox owner.
    if provisional_load {
        crate::ir_lower::ownership::release_if_owned(ctx, source, Some(span));
    }
    store_static_property(ctx, receiver, property, stored.value, span);
    // The string rule's own release, SKIPPED when the provisional-load arm already issued
    // one: the two conditions overlap for an owned unboxed local load of a `Str`, and
    // releasing the same source twice is a double free rather than a tidy-up.
    if stores_a_string && source_is_owning_temporary && !provisional_load {
        crate::ir_lower::ownership::release_if_owned(ctx, source, Some(span));
    }
}

/// Returns true when codegen gives the static-property slot an independently retained value.
///
/// The rule is whether the store crosses the boxed/unboxed boundary. A concrete value going
/// into a Mixed/Union slot is boxed, and the box retains the child itself. A boxed value
/// going into a slot codegen narrows to -- `Str`, `Int`, `Bool`, `Float`, `Object`, or the
/// tagged-scalar pair -- is cast out of the box, and what lands in the slot owns nothing of
/// it (`__rt_str_persist` copies, the scalar casts read a payload word, the object arm
/// increfs the unboxed pointer on its own). Both directions leave the source's reference
/// unrelated to the slot's, so borrowed sources need no `Acquire` and owning temporary
/// sources must be released after the store -- without that release the box outlives its
/// last use and leaks, one cell per compound assignment (issue #1041).
///
/// Every other case is a moving store: a Mixed value into a Mixed/Union slot, or into one of
/// the container slots codegen leaves untouched, writes the pointer itself into the slot, so
/// the store consumes the reference. Unknown metadata conservatively keeps that discipline.
pub(super) fn static_property_store_retains_independent_value(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: LoweredValue,
) -> bool {
    let Some(slot_ty) = static_property_type(ctx, receiver, property) else {
        return false;
    };
    let value_ty = ctx.builder.value_php_type(value.value);
    let slot_ty = slot_ty.codegen_repr();
    let value_ty = value_ty.codegen_repr();
    let value_is_boxed = matches!(value_ty, PhpType::Mixed | PhpType::Union(_));
    let boxes_into_mixed = matches!(slot_ty, PhpType::Mixed | PhpType::Union(_)) && !value_is_boxed;
    // The slot types codegen actually casts a Mixed source out of. Deliberately not "any slot
    // that is not Mixed": an array or iterable slot falls through `load_static_property_store_
    // value_to_result`'s catch-all arm untouched, so the Mixed pointer itself is what gets
    // stored and the store still consumes it.
    let narrows_out_of_mixed = value_is_boxed
        && matches!(
            slot_ty,
            PhpType::Str
                | PhpType::Int
                | PhpType::Bool
                | PhpType::Float
                | PhpType::Object(_)
                | PhpType::TaggedScalar
        );
    boxes_into_mixed || narrows_out_of_mixed
}

/// Lowers `Class::$prop[] = value`.
pub(super) fn lower_static_property_array_push(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: &Expr,
    span: Span,
) {
    if let Some(array) = separate_php_array_static_property(ctx, receiver, property, span) {
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::MixedArrayAppend,
            vec![array.value, value.value],
            None,
            Op::MixedArrayAppend.default_effects(),
            Some(span),
        );
        // The boxed append takes a call's string over; only a copied one is still ours.
        if ctx.builder.value_php_type(value.value).codegen_repr() == PhpType::Str {
            super::array_write_core::release_string_copied_into_box(ctx, value, span);
        } else if ctx.value_is_owning_temporary(value) {
            crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
        }
        return;
    }
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_indexed_array_type)
    {
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::ArrayPush,
            vec![property_value.value, value.value],
            None,
            Op::ArrayPush.default_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    // A hash-typed slot (a keyed write came first: `static::$map[$name] = $v`) appends at the
    // next integer key through the hash writer, which publishes a relocated table back into the
    // loaded value, and that value goes back into the slot. The generic placeholder below would
    // read the slot as a box and drop the write.
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_assoc_array_type)
    {
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        release_persisted_string_operand(ctx, value, span);
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    let property_value = load_static_property(ctx, receiver, property, span);
    let value = lower_expr(ctx, value);
    let boxed_slot = static_property_type(ctx, receiver, property)
        .is_some_and(|ty| matches!(ty.codegen_repr(), PhpType::Mixed));
    if boxed_slot || static_property_may_be_eval_dynamic(ctx, receiver) {
        ctx.emit_void(
            Op::MixedArrayAppend,
            vec![property_value.value, value.value],
            None,
            Op::MixedArrayAppend.default_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }
    ctx.emit_void(
        Op::RuntimeCall,
        vec![property_value.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers `Class::$prop[index] = value`.
///
/// A desugared `??=` becomes a probe plus a conditional insert, and a desugared compound update
/// writes with the key its own read already converted (see `ElementUpdate`).
pub(super) fn lower_static_property_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    let update = desugared_element_update(value, span, |read| {
        reads_static_property_array_element(read, receiver, property, index)
    });
    if let Some(ElementUpdate::NullCoalesce { read, default }) = update {
        crate::ir_lower::expr::lower_null_coalesce_update_stmt(ctx, read, default, span);
        return;
    }
    lower_static_property_array_assign_with_diagnosed_key(
        ctx,
        receiver,
        property,
        index,
        value,
        span,
        update.is_some(),
    );
}

/// Returns whether `read` is the element `Class::$property[index]` a statement writes.
fn reads_static_property_array_element(
    read: &Expr,
    receiver: &StaticReceiver,
    property: &str,
    index: &Expr,
) -> bool {
    matches!(
        &read.kind,
        ExprKind::ArrayAccess { array, index: read_index }
            if read_index.as_ref() == index
                && matches!(
                    &array.kind,
                    ExprKind::StaticPropertyAccess { receiver: read_receiver, property: read_property }
                        if read_receiver == receiver && read_property == property
                )
    )
}

/// Lowers `Class::$prop[index] = value`, optionally reusing a float-key diagnosis.
///
/// `key_already_diagnosed` marks the write half of a compound update whose read of the same
/// element already reported the float key's conversion. An `ArrayAccess` receiver gets the key
/// unconverted, so its `offsetSet` call never consults the flag.
pub(crate) fn lower_static_property_array_assign_with_diagnosed_key(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    key_already_diagnosed: bool,
) {
    let key_marker = key_already_diagnosed.then_some(Immediate::Bool(true));
    // php evaluates the key and the assigned value BEFORE it fetches the static property for
    // write (`zend_delayed_compile_dim`; `??=` recompiles the variable in W mode after its
    // default). Carbon's `static::$singletons[$key] ??= new static($locale)` runs a constructor
    // that unsets another key of the same static array: separating the container first wrote
    // the new translator into the stale copy, and the next read said `Undefined array key "en"`.
    let property_is_php_array =
        static_property_type(ctx, receiver, property).is_some_and(|ty| ty.is_php_array());
    if property_is_php_array {
        let (index, value) = array_write_core::lower_write_key_and_value(ctx, index, value);
        let array = separate_php_array_static_property(ctx, receiver, property, span)
            .expect("a php-array static property separates for write");
        ctx.emit_void(
            Op::RuntimeCall,
            vec![array.value, index.value, value.value],
            key_marker,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        release_persisted_string_operand(ctx, index, span);
        if ctx.value_is_owning_temporary(value) {
            crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
        }
        return;
    }
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_indexed_array_type)
    {
        let array_ty = property_ty.clone();
        // PHP reads a plain-variable index at STORE time, after the right-hand side, so
        // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
        // rule; sharing the helper is what keeps the two from answering differently for
        // the same source line.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        let index =
            coerce_array_key_to_int_at_span(ctx, index, Some(span), key_already_diagnosed);
        let value = coerce_indexed_array_set_value(ctx, &array_ty, value, Some(span));
        ctx.emit_void(
            Op::ArraySet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::ArraySet.default_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    // A reference binding can widen a typed `array` static property to associative Mixed
    // storage after earlier writes have already been parsed. Lower those string/mixed-key
    // writes through the real hash mutation op and publish a relocated table back to the
    // static slot; a generic runtime-call placeholder would silently discard the write.
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_assoc_array_type)
    {
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        ctx.emit_void(
            Op::HashSet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::HashSet.default_effects(),
            Some(span),
        );
        release_persisted_string_operand(ctx, index, span);
        release_persisted_string_operand(ctx, value, span);
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    let array_access_ty = static_property_type(ctx, receiver, property)
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty));
    // `offsetSet` receives the original key, so only a boxed array receiver has a key to rebuild.
    let key_marker = if array_access_ty.is_some() { None } else { key_marker };
    // PHP reads a plain-variable index at STORE time, after the right-hand side, so
    // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
    // rule; sharing the helper is what keeps the two from answering differently for
    // the same source line.
    let (index, value) =
        crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
    let property_value = if let Some(property_ty) = array_access_ty {
        load_static_property_as(ctx, receiver, property, property_ty, span)
    } else {
        load_static_property(ctx, receiver, property, span)
    };
    // A boxed slot is written through the mixed-array writer, which may vivify or relocate what
    // the box holds, so the box goes back to the slot like an eval-owned one.
    let boxed_slot = static_property_type(ctx, receiver, property)
        .is_some_and(|ty| matches!(ty.codegen_repr(), PhpType::Mixed));
    if boxed_slot || static_property_may_be_eval_dynamic(ctx, receiver) {
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, index.value, value.value],
            key_marker,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }
    ctx.emit_void(
        Op::RuntimeCall,
        vec![property_value.value, index.value, value.value],
        key_marker,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers `self::$a[$target] = &self::$a[$source]` through one shared managed cell.
pub(super) fn lower_static_property_element_ref_assign(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    target_index: &Expr,
    source: &Expr,
    span: Span,
) {
    let ExprKind::ArrayAccess {
        index: source_index,
        ..
    } = &source.kind
    else {
        lower_expr(ctx, source);
        return;
    };

    let hash_ty = static_property_type(ctx, receiver, property).unwrap_or(PhpType::Mixed);
    let mut hash = load_static_property_as(ctx, receiver, property, hash_ty, span);
    if let IrType::Heap(crate::ir::IrHeapKind::Array) = hash.ir_type {
        let current_ty = ctx.builder.value_php_type(hash.value);
        let element_ty = reference_element_type(&current_ty);
        let assoc_ty = promoted_assoc_array_type(current_ty, element_ty);
        hash = ctx.emit_value(
            Op::ArrayToHash,
            vec![hash.value],
            None,
            assoc_ty,
            Op::ArrayToHash.default_effects(),
            Some(span),
        );
    }

    let element_ty = reference_element_type(&ctx.builder.value_php_type(hash.value));
    let source_key = lower_expr(ctx, source_index);
    let cell = ctx.emit_value(
        Op::HashRefElement,
        vec![hash.value, source_key.value],
        None,
        element_ty,
        Op::HashRefElement.default_effects(),
        Some(span),
    );
    let hash_ty_after = ctx.builder.value_php_type(hash.value);
    let target_key = lower_expr(ctx, target_index);
    let bound_hash = ctx.emit_value(
        Op::HashBindRefElement,
        vec![hash.value, target_key.value, cell.value],
        None,
        hash_ty_after,
        Op::HashBindRefElement.default_effects(),
        Some(span),
    );
    store_static_property(ctx, receiver, property, bound_hash.value, span);
}

/// Lowers a direct, indexed, or append write through a runtime static-property name.
pub(super) fn lower_dynamic_static_property_write(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &Expr,
    index: Option<&Expr>,
    append: bool,
    value: &Expr,
    span: Span,
) {
    let name = lower_expr(ctx, property);
    let name = coerce_to_string_at_span(ctx, name, Some(property.span)).value;
    let common_ty = dynamic_static_property_common_type(ctx, receiver);

    if append {
        if let Some(array_ty) = common_ty.clone().filter(is_indexed_array_type) {
            let array = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
            let value = lower_expr(ctx, value);
            ctx.emit_void(
                Op::ArrayPush,
                vec![array.value, value.value],
                None,
                Op::ArrayPush.default_effects(),
                Some(span),
            );
            store_dynamic_static_property(ctx, receiver, name, array.value, span);
            return;
        }
        let array_ty = common_ty
            .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
            .unwrap_or(PhpType::Mixed);
        let array = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![array.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        return;
    }

    if let Some(index) = index {
        lower_dynamic_static_array_element_set(
            ctx, receiver, name, common_ty, index, value, span,
        );
        return;
    }

    let value = lower_expr(ctx, value);
    store_dynamic_static_property(ctx, receiver, name, value.value, span);
}

/// Lowers an indexed write through a runtime static-property name.
fn lower_dynamic_static_array_element_set(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    common_ty: Option<PhpType>,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    if let Some(array_ty) = common_ty.clone().filter(is_indexed_array_type) {
        let element_array_ty = array_ty.clone();
        let array = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        let value = coerce_indexed_array_set_value(ctx, &element_array_ty, value, Some(span));
        ctx.emit_void(
            Op::ArraySet,
            vec![array.value, index.value, value.value],
            None,
            Op::ArraySet.default_effects(),
            Some(span),
        );
        store_dynamic_static_property(ctx, receiver, name, array.value, span);
        return;
    }

    let array_ty = common_ty
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
        .unwrap_or(PhpType::Mixed);
    let array = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
    let index = lower_expr(ctx, index);
    let value = lower_expr(ctx, value);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![array.value, index.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Loads a runtime-named static property using a precomputed string value.
fn load_dynamic_static_property_as(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    php_type: PhpType,
    span: Span,
) -> LoweredValue {
    let class_name =
        static_receiver_class_name(ctx, receiver).unwrap_or_else(|| receiver_name(receiver));
    let data = ctx.intern_string(&class_name);
    ctx.emit_value(
        Op::LoadDynamicStaticProperty,
        vec![name],
        Some(Immediate::Data(data)),
        php_type,
        Op::LoadDynamicStaticProperty.default_effects(),
        Some(span),
    )
}

/// Stores through a runtime static-property name using a precomputed string value.
fn store_dynamic_static_property(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    value: crate::ir::ValueId,
    span: Span,
) {
    let class_name =
        static_receiver_class_name(ctx, receiver).unwrap_or_else(|| receiver_name(receiver));
    let data = ctx.intern_string(&class_name);
    ctx.emit_void(
        Op::StoreDynamicStaticProperty,
        vec![name, value],
        Some(Immediate::Data(data)),
        Op::StoreDynamicStaticProperty.default_effects(),
        Some(span),
    );
}

/// Returns the common codegen representation of a class's static properties.
fn dynamic_static_property_common_type(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> Option<PhpType> {
    let class_name = static_receiver_class_name(ctx, receiver)?;
    let class_info = ctx.classes.get(class_name.as_str())?;
    let mut types = class_info
        .static_properties
        .iter()
        .map(|(_, ty)| normalize_value_php_type(ty.codegen_repr()));
    let first = types.next()?;
    if types.all(|ty| ty == first) {
        Some(first)
    } else {
        Some(PhpType::Mixed)
    }
}

/// Publishes a detached boxed PHP array before a static-property element mutation.
/// The static slot consumes the new cell and releases its previous owner through ordinary storage.
fn separate_php_array_static_property(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    span: Span,
) -> Option<LoweredValue> {
    let ty = static_property_type(ctx, receiver, property)?;
    if !ty.is_php_array() {
        return None;
    }
    let previous = load_static_property_as(ctx, receiver, property, ty.clone(), span);
    let separated = ctx.emit_value(
        Op::MixedClone,
        vec![previous.value],
        None,
        ty,
        Op::MixedClone.default_effects(),
        Some(span),
    );
    store_static_property(ctx, receiver, property, separated.value, span);
    Some(separated)
}

/// Returns true when a named static-property receiver may resolve through eval metadata.
pub(super) fn static_property_may_be_eval_dynamic(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> bool {
    let StaticReceiver::Named(class_name) = receiver else {
        return false;
    };
    ctx.has_eval_barrier()
        && !ctx
            .classes
            .contains_key(class_name.as_str().trim_start_matches('\\'))
}
