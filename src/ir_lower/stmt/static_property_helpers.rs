//! Purpose:
//! Static-property reads, writes, and property type lookup.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Loads a static property value through a high-level EIR read.
pub(super) fn load_static_property(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    span: Span,
) -> LoweredValue {
    load_static_property_as(ctx, receiver, property, PhpType::Mixed, span)
}

/// Loads a static property value using known PHP metadata.
pub(super) fn load_static_property_as(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    php_type: PhpType,
    span: Span,
) -> LoweredValue {
    let name = format!("{}::{}", receiver_name(receiver), property);
    let data = ctx.intern_string(&name);
    ctx.emit_value(
        Op::LoadStaticProperty,
        Vec::new(),
        Some(Immediate::Data(data)),
        php_type,
        Op::LoadStaticProperty.default_effects(),
        Some(span),
    )
}

/// Stores a static property value through a high-level EIR write.
pub(super) fn store_static_property(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: crate::ir::ValueId,
    span: Span,
) {
    let name = format!("{}::{}", receiver_name(receiver), property);
    let data = ctx.intern_string(&name);
    ctx.emit_void(
        Op::StoreStaticProperty,
        vec![value],
        Some(Immediate::Data(data)),
        Op::StoreStaticProperty.default_effects(),
        Some(span),
    );
}

/// Formats a static receiver for metadata immediates.
pub(super) fn receiver_name(receiver: &StaticReceiver) -> String {
    match receiver {
        StaticReceiver::Named(name) => name.as_str().to_string(),
        StaticReceiver::Self_ => "self".to_string(),
        StaticReceiver::Static => "static".to_string(),
        StaticReceiver::Parent => "parent".to_string(),
    }
}

/// Resolves the declared PHP type of a static property for statement lowering.
pub(super) fn static_property_type(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
) -> Option<PhpType> {
    let class_name = static_receiver_class_name(ctx, receiver)?;
    ctx.classes
        .get(class_name.as_str())?
        .static_properties
        .iter()
        .find(|(name, _)| name == property)
        .map(|(_, property_ty)| normalize_value_php_type(property_ty.codegen_repr()))
}

/// Resolves a static receiver to a concrete class name when lexical metadata is available.
pub(super) fn static_receiver_class_name(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> Option<String> {
    match receiver {
        StaticReceiver::Named(name) => Some(name.as_str().trim_start_matches('\\').to_string()),
        StaticReceiver::Self_ | StaticReceiver::Static => ctx.current_class.clone(),
        StaticReceiver::Parent => {
            let current = ctx.current_class.as_deref()?;
            ctx.classes
                .get(current)
                .and_then(|class_info| class_info.parent.clone())
        }
    }
}

/// Resolves the declared PHP type of an object property for statement lowering.
pub(in crate::ir_lower) fn object_property_type(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> Option<PhpType> {
    let object_ty = ctx.builder.value_php_type(object);
    let PhpType::Object(class_name) = object_ty else {
        return None;
    };
    ctx.classes
        .get(class_name.trim_start_matches('\\'))?
        .visible_property(property)
        .map(|(_, (_, property_ty))| normalize_value_php_type(property_ty.clone()))
}

/// Resolves an array-like property type for a receiver declared as bare PHP `object`.
pub(in crate::ir_lower) fn generic_object_array_property_type(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> Option<PhpType> {
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object) else {
        return None;
    };
    if !class_name.trim_start_matches('\\').is_empty() {
        return object_property_type(ctx, object, property);
    }
    let candidates = ctx
        .classes
        .values()
        .filter_map(|class_info| {
            class_info
                .visible_property(property)
                .map(|(_, (_, property_ty))| normalize_value_php_type(property_ty.clone()))
        })
        .filter(|property_ty| {
            matches!(
                property_ty.codegen_repr(),
                PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed
            ) || matches!(
                property_ty.codegen_repr(),
                PhpType::Object(class_name) if class_name.trim_start_matches('\\').is_empty()
            ) || type_satisfies_array_access_for_ir(ctx, property_ty)
        })
        .collect::<Vec<_>>();
    let first = candidates.first()?.clone();
    if candidates
        .iter()
        .all(|candidate| {
            matches!(
                candidate.codegen_repr(),
                PhpType::Object(class_name) if class_name.trim_start_matches('\\').is_empty()
            ) || type_satisfies_array_access_for_ir(ctx, candidate)
        })
    {
        return Some(PhpType::Object("ArrayAccess".to_string()));
    }
    if candidates.iter().all(|candidate| candidate == &first) {
        return Some(first);
    }
    if candidates
        .iter()
        .all(|candidate| matches!(candidate.codegen_repr(), PhpType::Array(_)))
    {
        return Some(PhpType::Array(Box::new(PhpType::Mixed)));
    }
    if candidates
        .iter()
        .all(|candidate| matches!(candidate.codegen_repr(), PhpType::AssocArray { .. }))
    {
        return Some(PhpType::AssocArray {
            key: Box::new(PhpType::Mixed),
            value: Box::new(PhpType::Mixed),
        });
    }
    Some(PhpType::Mixed)
}

/// Returns true when a property type uses concrete indexed-array storage.
pub(super) fn is_indexed_array_type(php_type: &PhpType) -> bool {
    matches!(php_type.codegen_repr(), PhpType::Array(_))
}

/// Returns true when a property type uses concrete associative-array storage.
pub(super) fn is_assoc_array_type(php_type: &PhpType) -> bool {
    matches!(php_type.codegen_repr(), PhpType::AssocArray { .. })
}

/// Normalizes non-materializable statement metadata to the EIR null sentinel type.
pub(super) fn normalize_value_php_type(php_type: PhpType) -> PhpType {
    if matches!(php_type, PhpType::Never) {
        PhpType::Void
    } else {
        php_type
    }
}
