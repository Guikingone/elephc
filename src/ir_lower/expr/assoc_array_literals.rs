//! Purpose:
//! Associative array literal typing and expression result inference.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers an associative array literal.
pub(super) fn lower_assoc_array_literal(ctx: &mut LoweringContext<'_, '_>, pairs: &[(Expr, Expr)], expr: &Expr) -> LoweredValue {
    lower_assoc_array_literal_with_guard(ctx, pairs, expr, false)
}

/// Protects a callback's partially built argument hash until invocation consumes its ownership.
pub(super) fn lower_assoc_array_literal_with_guard(
    ctx: &mut LoweringContext<'_, '_>, pairs: &[(Expr, Expr)], expr: &Expr, guarded: bool,
) -> LoweredValue {
    let hash = ctx.emit_value(
        Op::HashNew,
        Vec::new(),
        Some(Immediate::Capacity(pairs.len() as u32)),
        assoc_array_literal_type_for_ir(ctx, pairs, expr),
        Op::HashNew.default_effects(),
        Some(expr.span),
    );
    if guarded { guard_descriptor_container(ctx, hash, expr.span); }
    // The literal is the only owner of the entries already inserted, so a key or value
    // expression that throws (`['a' => 1, 'b' => bad()]`) would strand it. Publish it in an
    // unwind-visible slot for the construction when any entry can throw; a guarded callback
    // argument hash already has its own protection.
    let published = if !guarded
        && pairs.iter().any(|(key, value)| {
            crate::optimize::expr_may_throw(key) || crate::optimize::expr_may_throw(value)
        })
    {
        let hash_ty = ctx.builder.value_php_type(hash.value);
        Some((publish_constructed_container(ctx, hash, expr.span), hash_ty))
    } else {
        None
    };
    for (key, value) in pairs {
        let key = lower_expr(ctx, key);
        let value = lower_array_reference_or_value(ctx, value);
        let target = match &published {
            Some((owner, hash_ty)) => {
                load_published_container(ctx, *owner, hash_ty.clone(), expr.span)
            }
            None => hash,
        };
        ctx.emit_void(Op::HashSet, vec![target.value, key.value, value.value], None, Op::HashSet.default_effects(), Some(expr.span));
        release_string_after_hash_insert(ctx, value, expr.span);
        release_key_after_hash_insert(ctx, key, expr.span);
        if guarded { ctx.refresh_argument_array_guard(hash, expr.span); }
    }
    match published {
        Some((owner, hash_ty)) => take_published_container(ctx, owner, hash_ty, expr.span),
        None => hash,
    }
}

/// Releases an owned string a `HashSet` has just stored.
///
/// The hash writer persists a string value -- it takes its own reference -- whatever the
/// container's value type, while every other owned payload is absorbed. So a call's string
/// result inserted by a literal (`['date' => $d->format('Y'), 'n' => 1]`, every
/// `DateTimeImmutable::__serialize()`) leaked one block per evaluation.
pub(super) fn release_string_after_hash_insert(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: crate::span::Span,
) {
    if ctx.builder.value_php_type(value.value).codegen_repr() == PhpType::Str
        && ctx.value_is_owning_temporary(value)
    {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Releases a computed key a `HashSet` has just used.
///
/// The hash copies the key it stores, so an owned key -- a boxed `$alias ?? 'name'`, a call's
/// string -- is still the literal's to release, exactly as `unset($a[$key])` releases its own.
/// Doctrine's DQL parser builds `[$fieldAlias ?? $field => $arg]`; each evaluation leaked the box
/// and its string.
pub(super) fn release_key_after_hash_insert(
    ctx: &mut LoweringContext<'_, '_>,
    key: LoweredValue,
    span: crate::span::Span,
) {
    if ctx.value_is_owning_temporary(key) {
        crate::ir_lower::ownership::release_if_owned(ctx, key, Some(span));
    } else {
        crate::ir_lower::stmt::release_persisted_string_operand(ctx, key, span);
    }
}

/// Lowers an ordinary array value or an owning marker for a referenced variable.
pub(in crate::ir_lower) fn lower_array_reference_or_value(
    ctx: &mut LoweringContext<'_, '_>,
    value: &Expr,
) -> LoweredValue {
    let ExprKind::ArrayReference(inner) = &value.kind else {
        return lower_expr(ctx, value);
    };
    if matches!(inner.kind, ExprKind::ArrayAccess { .. } | ExprKind::PropertyAccess { .. }) {
        return lower_element_reference_marker(ctx, inner, value.span);
    }
    let ExprKind::Variable(name) = &inner.kind else {
        unreachable!("array reference markers require a variable or element source");
    };
    // A by-reference `foreach` value variable that was NOT promoted to a shared cell aliases the
    // element's raw storage: there is no cell to hand out, and wrapping that address as one
    // corrupted the heap. Store the value instead -- the reference is lost, the memory is not.
    if ctx.is_interior_ref_local(name) {
        return lower_expr(ctx, inner);
    }
    if crate::superglobals::is_superglobal(name) {
        let data = ctx.intern_global_storage_name(name);
        return ctx.emit_value(
            Op::InvokerRefArg,
            Vec::new(),
            Some(Immediate::GlobalName(data)),
            PhpType::Mixed,
            Op::InvokerRefArg.default_effects(),
            Some(value.span),
        );
    }
    // A PHP reference stored in an array can subsequently receive any PHP value, irrespective
    // of the source variable's current value. Promote through Mixed before emitting the marker:
    // the marker records its referenced storage tag once, so preserving an initial null tag would
    // make a later closure/callable assignment read back as null.
    ctx.promote_local_mixed_ref_cell(name, Some(value.span));
    local_ref_cell_marker(ctx, name, value.span)
}

/// Emits the owning array marker over the reference cell local `name` is bound to.
fn local_ref_cell_marker(ctx: &mut LoweringContext<'_, '_>, name: &str, span: Span) -> LoweredValue {
    let local_type = ctx.local_type(name);
    let slot = ctx.declare_local(name, local_type);
    ctx.emit_value(
        Op::ArrayLocalRefCell,
        Vec::new(),
        Some(Immediate::LocalSlot(slot)),
        PhpType::Mixed,
        Op::ArrayLocalRefCell.default_effects(),
        Some(span),
    )
}

/// Lowers `&$a[$k]` or `&$o->p` inside an array literal: the place becomes a reference, and the
/// literal stores a marker over that same reference.
///
/// A hidden local is bound to the place exactly as `$t = &$a[$k]` (or `$t = &$o->p`) would bind
/// it -- which makes the place a shared reference and gives the local one share -- the marker
/// takes its own share from the local, and the hidden binding ends, leaving the place and the
/// literal's slot as the reference's holders. A property's storage was widened to a boxed cell by
/// the checker (`widen_property_for_element_reference`).
fn lower_element_reference_marker(
    ctx: &mut LoweringContext<'_, '_>,
    element: &Expr,
    span: Span,
) -> LoweredValue {
    let temp = ctx.declare_synthetic_php_local(PhpType::Mixed);
    if matches!(element.kind, ExprKind::PropertyAccess { .. }) {
        crate::ir_lower::expr::lower_ref_assign_property(ctx, &temp, element, span);
    } else {
        crate::ir_lower::expr::lower_ref_assign_array_elem(ctx, &temp, element, span);
    }
    let marker = lower_array_reference_or_value(
        ctx,
        &Expr::new(
            ExprKind::ArrayReference(Box::new(Expr::new(
                ExprKind::Variable(temp.clone()),
                span,
            ))),
            span,
        ),
    );
    let null = lower_null(ctx, element);
    ctx.unset_local(&temp, null, Some(span));
    marker
}

/// Returns the associative-array type for a literal that contains at least one associative
/// spread. Mirrors the type checker's `assoc_spread_literal_value_type` so EIR storage matches
/// the value types actually lowered into the hash.
pub(super) fn assoc_array_literal_type_from_spreads(
    ctx: &LoweringContext<'_, '_>,
    items: &[Expr],
    expr: &Expr,
) -> PhpType {
    let mut value_ty = PhpType::Never;
    for item in items {
        let next = match &item.kind {
            ExprKind::Spread(inner) => match array_literal_element_type_for_ir(ctx, inner).codegen_repr() {
                PhpType::Array(elem) => elem.codegen_repr(),
                PhpType::AssocArray { value, .. } => value.codegen_repr(),
                _ => PhpType::Mixed,
            },
            _ => array_literal_element_type_for_ir(ctx, item).codegen_repr(),
        };
        value_ty = merge_ir_assoc_value_type(value_ty, next);
    }
    if matches!(value_ty, PhpType::Never) {
        return fallback_expr_type(expr);
    }
    PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(value_ty),
    }
}

/// Returns the associative-array type that the EIR backend can faithfully materialize.
pub(super) fn assoc_array_literal_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    pairs: &[(Expr, Expr)],
    expr: &Expr,
) -> PhpType {
    if pairs.is_empty() {
        return fallback_expr_type(expr);
    }
    // Seeded from the FIRST entry rather than from a neutral element: `merge_array_key_types`
    // answers Mixed for any two different types, so folding a placeholder into it would widen
    // every literal's key type.
    let mut key_ty: Option<PhpType> = None;
    let mut value_ty: Option<PhpType> = None;
    for (key, value) in pairs {
        let next_key = normalized_array_key_type(key, infer_expr_type_syntactic(key));
        let next_value = assoc_array_literal_value_type_for_ir(ctx, value);
        key_ty = Some(match key_ty {
            Some(current) => merge_array_key_types(current, next_key),
            None => next_key,
        });
        value_ty = Some(match value_ty {
            Some(current) => merge_ir_assoc_value_type(current, next_value),
            None => next_value,
        });
    }
    let (Some(key_ty), Some(value_ty)) = (key_ty, value_ty) else {
        return fallback_expr_type(expr);
    };
    PhpType::AssocArray {
        key: Box::new(key_ty),
        value: Box::new(value_ty),
    }
}

/// Returns the best EIR storage value type for one associative-array literal value.
pub(super) fn assoc_array_literal_value_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    value: &Expr,
) -> PhpType {
    if let Some(storage) = nullsafe_chain::result_storage_type(value) {
        return storage;
    }
    match &value.kind {
        ExprKind::Null => PhpType::Mixed,
        ExprKind::ArrayReference(_) => PhpType::Mixed,
        ExprKind::Ternary { .. } => {
            // Use the actual branch-merge representation before choosing hash storage.
            ir_array_storage_type(materialized_expr_type_for_merge(ctx, value))
        }
        ExprKind::ConstRef(name) => ctx
            .constant_value(name.as_str())
            .map(|(_, ty)| ir_array_storage_type(ty))
            .unwrap_or_else(|| ir_array_storage_type(infer_expr_type_syntactic(value))),
        // A class constant or enum case must be typed the way `lower_scoped_constant`
        // resolves it, not by the syntactic `::class`-is-string default, or the hash
        // value-type stamp would diverge from the lowered value and corrupt reads.
        ExprKind::ScopedConstantAccess { receiver, name } => {
            scoped_constant_value_type_for_ir(ctx, receiver, name, value)
        }
        ExprKind::DynamicScopedConstantAccess { .. } => PhpType::Mixed,
        // A nested literal must be typed by the same context-aware function that will lower
        // it, not by the syntactic fallback: `infer_expr_type_syntactic` cannot see a local's
        // type, so it types every `$v` element `Int`. The outer hash then stamped its value
        // type `array<string, int>` over an inner hash really holding `array<string>`, and a
        // read through both levels returned the inner array pointer as an integer (issue
        // #984). These are the two arms `array_literal_element_type_for_ir` already carries
        // for an INDEXED outer literal, which is why `[["k" => $v]]` was always correct and
        // only `["j" => ["k" => $v]]` was wrong.
        ExprKind::ArrayLiteral(items) => {
            array_literal_type_for_ir(ctx, items, value).codegen_repr()
        }
        ExprKind::ArrayLiteralAssoc(inner_pairs) => {
            assoc_array_literal_type_for_ir(ctx, inner_pairs, value)
        }
        ExprKind::Variable(name) => ir_array_storage_type(
            ctx.local_types
                .get(name)
                .cloned()
                .unwrap_or_else(|| infer_expr_type_syntactic(value)),
        ),
        ExprKind::This => ctx
            .current_class
            .as_ref()
            .map(|class_name| PhpType::Object(class_name.clone()))
            .unwrap_or_else(|| ir_array_storage_type(infer_expr_type_syntactic(value))),
        ExprKind::FunctionCall { name, .. } => {
            let canonical = name.as_str();
            if let Some(sig) = ctx.functions.get(canonical) {
                return ir_array_storage_type(eir_user_function_return_type(sig));
            }
            if let Some(sig) = ctx.extern_functions.get(canonical) {
                return ir_array_storage_type(sig.return_type.clone());
            }
            // A builtin is in neither map, and the syntactic fallback below answers `Int` for
            // every name outside its hand-written allowlist -- which stamps the literal
            // `array<int>` and makes lowering cast the element to match (#1096). The checker
            // already asked the registry contract about this very call.
            if let Some(resolved) = builtin_call_result_type_for_ir(ctx, value.span) {
                return resolved;
            }
            ir_array_storage_type(infer_expr_type_syntactic(value))
        }
        ExprKind::MethodCall { object, method, .. } => {
            method_call_expr_type_for_ir(ctx, object, method)
                .and_then(materializable_array_element_type)
                .unwrap_or_else(|| ir_array_storage_type(infer_expr_type_syntactic(value)))
        }
        ExprKind::StaticMethodCall { receiver, method, .. } => {
            static_method_call_expr_type_for_ir(ctx, receiver, method)
                .and_then(materializable_array_element_type)
                .unwrap_or_else(|| ir_array_storage_type(infer_expr_type_syntactic(value)))
        }
        ExprKind::ShortTernary { value: present, default }
        | ExprKind::NullCoalesce { value: present, default } => merge_ir_assoc_value_type(
            assoc_array_literal_value_type_for_ir(ctx, present),
            assoc_array_literal_value_type_for_ir(ctx, default),
        ),
        ExprKind::ArrayAccess { array, .. } => array_access_expr_value_type_for_ir(ctx, array)
            .unwrap_or_else(|| ir_array_storage_type(infer_expr_type_syntactic(value))),
        ExprKind::PropertyAccess { object, property } => property_access_expr_type_for_ir(
            ctx,
            object,
            property,
        )
        .unwrap_or_else(|| ir_array_storage_type(infer_expr_type_syntactic(value))),
        _ => ir_array_storage_type(infer_expr_type_syntactic(value)),
    }
}

/// Returns the EIR storage value type for a scoped-constant array value,
/// resolving a class/interface constant the same way `lower_scoped_constant`
/// lowers it so the hash value-type stamp matches the value actually stored
/// (rather than the syntactic `::class`-is-string default). Falls back to the
/// syntactic guess when the constant cannot be resolved.
pub(super) fn scoped_constant_value_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    member: &str,
    value: &Expr,
) -> PhpType {
    let class_name = scoped_constant_receiver_name(ctx, receiver);
    let normalized = class_name.trim_start_matches('\\');
    // An enum case lowers to the case *object* singleton (see `lower_scoped_constant`),
    // so the hash must box it as a Mixed cell — stamp the value type Mixed to match.
    if ctx
        .enums
        .get(normalized)
        .is_some_and(|enum_info| enum_info.cases.iter().any(|case| case.name == member))
    {
        return PhpType::Mixed;
    }
    if let Some(const_expr) = ctx.scoped_constant_value(&class_name, member) {
        // Keep the existing scalar classification, but resolve containers through the same
        // literal helpers that lower them. Syntactic inference reports Void for null and can
        // mistake nested scoped constants for strings inside an array initializer.
        return match &const_expr.kind {
            ExprKind::Null => PhpType::Mixed,
            ExprKind::ArrayLiteral(items) => array_literal_type_for_ir(ctx, items, &const_expr),
            ExprKind::ArrayLiteralAssoc(pairs) => {
                assoc_array_literal_type_for_ir(ctx, pairs, &const_expr)
            }
            ExprKind::ArrayLiteralMixed(entries) => {
                assoc_array_literal_type_from_entries(ctx, entries, &const_expr)
            }
            _ => ir_array_storage_type(infer_expr_type_syntactic(&const_expr)),
        };
    }
    ir_array_storage_type(infer_expr_type_syntactic(value))
}

/// Returns the element/value type for an array-access expression used inside a literal.
pub(in crate::ir_lower) fn array_access_expr_value_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    array: &Expr,
) -> Option<PhpType> {
    let array_ty = match &array.kind {
        ExprKind::Variable(name) => ctx.local_types.get(name).cloned(),
        ExprKind::PropertyAccess { object, property } => {
            property_access_expr_type_for_ir(ctx, object, property)
        }
        ExprKind::ArrayLiteral(items) => Some(array_literal_type_for_ir(ctx, items, array)),
        ExprKind::ArrayLiteralAssoc(pairs) => Some(assoc_array_literal_type_for_ir(ctx, pairs, array)),
        // A nested read (`$tags[0]['reset']`) takes the element type of the inner read. Without
        // it the literal fell back to the syntactic guess, whose unknown is `int`, and
        // `['method' => $tags[0]['reset']]` was stamped `array<string, int>` over a string --
        // Symfony's CachePoolPass tag attribute then read back as `0`.
        ExprKind::ArrayAccess { array: inner, .. } => {
            array_access_expr_value_type_for_ir(ctx, inner)
        }
        // A static property's DECLARED type, as for an instance property above. Without it a
        // nested read through one (`static::$formats[$format][0]`) fell back to the syntactic
        // guess, whose unknown is `int`: a ternary merging that read with `null` then stored the
        // inner string's POINTER in a `null|int` slot, and Symfony's `Request::getMimeType()`
        // answered `"4396328192"` -- every cached response went out with that as its Content-Type.
        ExprKind::StaticPropertyAccess { receiver, property } => Some(
            super::property_access::static_property_result_type(ctx, receiver, property, array),
        ),
        _ => None,
    }?
    .codegen_repr();
    match array_ty {
        PhpType::Array(elem_ty) => {
            Some(array_access_element_result_type(normalize_value_php_type(*elem_ty).codegen_repr()))
        }
        PhpType::AssocArray { value, .. } => {
            Some(array_access_element_result_type(normalize_value_php_type(*value).codegen_repr()))
        }
        PhpType::Str => Some(PhpType::Str),
        PhpType::Mixed | PhpType::Union(_) => Some(PhpType::Mixed),
        _ => None,
    }
}

/// Returns the declared type for an object property expression used inside a literal.
pub(in crate::ir_lower) fn property_access_expr_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
) -> Option<PhpType> {
    let class_name = instance_callable_object_class(ctx, object)?;
    let normalized = class_name.trim_start_matches('\\');
    if is_builtin_stdclass_name(normalized) {
        return Some(PhpType::Mixed);
    }
    if let Some(property_ty) = runtime_property_type_override(ctx, normalized, property) {
        return Some(normalize_value_php_type(property_ty));
    }
    let class_info = ctx.classes.get(normalized)?;
    if !property_is_accessible_for_ir(ctx, normalized, class_info, property) {
        return magic_get_result_type(ctx, normalized).map(normalize_value_php_type);
    }
    class_info
        .properties
        .iter()
        .find(|(name, _)| name == property)
        // Keep nullable nominal types available to receiver/isset analysis; storage
        // erasure belongs to the consumer that actually chooses an EIR layout.
        .map(|(_, ty)| normalize_value_php_type(ty.clone()))
}

/// Returns the declared property result type plus `null` when a nullsafe receiver may be null.
pub(super) fn nullsafe_property_access_expr_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
) -> Option<PhpType> {
    let property_type = property_access_expr_type_for_ir(ctx, object, property)?;
    let (_, nullable) = instance_callable_object_class_and_nullability(ctx, object)?;
    if nullable {
        Some(nullable_result_type(property_type))
    } else {
        Some(property_type)
    }
}

/// Returns the declared result type for an instance method call before its receiver is lowered.
pub(in crate::ir_lower) fn method_call_expr_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    method: &str,
) -> Option<PhpType> {
    let class_name = instance_callable_object_class(ctx, object)?;
    let method_key = php_symbol_key(method);
    class_method_signature(ctx, &class_name, &method_key)
        .map(|signature| normalize_value_php_type(signature.return_type.clone()))
}

/// Returns the declared method result type plus `null` when a nullsafe receiver may be null.
pub(super) fn nullsafe_method_call_expr_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    method: &str,
) -> Option<PhpType> {
    let return_type = method_call_expr_type_for_ir(ctx, object, method)?;
    let (_, nullable) = instance_callable_object_class_and_nullability(ctx, object)?;
    if nullable {
        Some(nullable_result_type(return_type))
    } else {
        Some(return_type)
    }
}

/// Merges associative-array value types for EIR storage metadata.
pub(crate) fn merge_ir_assoc_value_type(left: PhpType, right: PhpType) -> PhpType {
    ir_array_storage_type(PhpType::widen_array_branch_element(left, right))
}
