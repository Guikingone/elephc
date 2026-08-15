//! Purpose:
//! ReflectionProperty value access and reflected-target extraction.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers `ReflectionProperty::getValue($object)` when the reflected property is known.
pub(super) fn lower_reflection_property_value_call(
    ctx: &mut LoweringContext<'_, '_>,
    object_expr: Option<&Expr>,
    reflection_property: LoweredValue,
    method: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let object_expr = object_expr?;
    let reflection_property_ty = ctx.builder.value_php_type(reflection_property.value);
    let reflection_property_class = singular_object_class(&reflection_property_ty)
        .map(|(class_name, _)| class_name.to_string())
        .filter(|class_name| !class_name.trim_start_matches('\\').is_empty())
        .or_else(|| expression_object_class(ctx, object_expr))?;
    if php_symbol_key(reflection_property_class.trim_start_matches('\\'))
        != php_symbol_key("ReflectionProperty")
    {
        return None;
    }
    match php_symbol_key(method).as_str() {
        "getvalue" => {
            if let Some((declaring_class, property, property_ty)) =
                reflection_property_static_target(ctx, object_expr)
            {
                return lower_reflection_property_get_static_value(
                    ctx,
                    &declaring_class,
                    &property,
                    property_ty,
                    args,
                    expr,
                );
            }
            let (_, property, _) = reflection_property_instance_target(ctx, object_expr)?;
            lower_reflection_property_get_value(ctx, &property, args, expr)
        }
        "getrawvalue" => {
            if let Some((declaring_class, property, property_ty)) =
                reflection_property_static_target(ctx, object_expr)
            {
                return lower_reflection_property_get_static_value(
                    ctx,
                    &declaring_class,
                    &property,
                    property_ty,
                    args,
                    expr,
                );
            }
            if let Some((_, property, _)) =
                reflection_property_any_instance_target(ctx, object_expr)
            {
                return lower_reflection_property_get_raw_value(ctx, &property, args, expr);
            }
            lower_reflection_property_get_runtime_raw_value(
                ctx,
                reflection_property,
                args,
                expr,
            )
        }
        "setvalue" => {
            if let Some((declaring_class, property, _)) =
                reflection_property_static_target(ctx, object_expr)
            {
                return lower_reflection_property_set_static_value(
                    ctx,
                    &declaring_class,
                    &property,
                    args,
                    expr,
                );
            }
            let (_, property, _) = reflection_property_instance_target(ctx, object_expr)?;
            lower_reflection_property_set_value(ctx, &property, args, expr)
        }
        "setrawvalue" | "setrawvaluewithoutlazyinitialization" => {
            if let Some((declaring_class, property, _)) =
                reflection_property_static_target(ctx, object_expr)
            {
                return lower_reflection_property_set_static_value(
                    ctx,
                    &declaring_class,
                    &property,
                    args,
                    expr,
                );
            }
            if let Some((_, property, _)) =
                reflection_property_any_instance_target(ctx, object_expr)
            {
                return lower_reflection_property_set_raw_value(ctx, &property, args, expr);
            }
            lower_reflection_property_set_runtime_raw_value(
                ctx,
                reflection_property,
                args,
                expr,
            )
        }
        "isinitialized" => {
            if let Some((declaring_class, property, _)) =
                reflection_property_static_target(ctx, object_expr)
            {
                return lower_reflection_property_static_is_initialized(
                    ctx,
                    &declaring_class,
                    &property,
                    args,
                    expr,
                );
            }
            let (_, property, _) = reflection_property_any_instance_target(ctx, object_expr)?;
            lower_reflection_property_is_initialized(ctx, &property, args, expr)
        }
        _ => None,
    }
}

/// Returns the concrete object class retained in the lowering environment for an expression.
fn expression_object_class(ctx: &LoweringContext<'_, '_>, expr: &Expr) -> Option<String> {
    let environment_ty = match &expr.kind {
        ExprKind::Variable(name) => ctx.local_type(name),
        _ => fallback_expr_type(expr),
    };
    singular_object_class(&environment_ty)
        .map(|(class_name, _)| class_name.trim_start_matches('\\').to_string())
        .filter(|class_name| !class_name.is_empty())
}

/// Loads the reflected runtime property name from a ReflectionProperty value.
fn lower_reflection_property_runtime_name(
    ctx: &mut LoweringContext<'_, '_>,
    reflection_property: LoweredValue,
    expr: &Expr,
) -> LoweredValue {
    let data = ctx.intern_string("__name");
    ctx.emit_value(
        Op::PropGet,
        vec![reflection_property.value],
        Some(Immediate::Data(data)),
        PhpType::Str,
        Op::PropGet.default_effects(),
        Some(expr.span),
    )
}

/// Restores a statically known object shape after loading an object from gradual storage.
fn lower_reflection_property_object_argument(
    ctx: &mut LoweringContext<'_, '_>,
    object_arg: &Expr,
    expr: &Expr,
) -> LoweredValue {
    let object = lower_expr(ctx, object_arg);
    if ctx.builder.value_php_type(object.value).codegen_repr() != PhpType::Mixed {
        return object;
    }
    let Some(class_name) = expression_object_class(ctx, object_arg) else {
        return object;
    };
    ctx.emit_value(
        Op::MixedUnbox,
        vec![object.value],
        None,
        PhpType::Object(class_name),
        Op::MixedUnbox.default_effects(),
        Some(expr.span),
    )
}

/// Lowers a raw reflected-property read whose property name is known only at runtime.
fn lower_reflection_property_get_runtime_raw_value(
    ctx: &mut LoweringContext<'_, '_>,
    reflection_property: LoweredValue,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let object_arg = reflection_property_get_value_arg(args)?;
    let object = lower_reflection_property_object_argument(ctx, &object_arg, expr);
    let object = box_generic_object_for_dynamic_property(ctx, object, expr.span);
    let property = lower_reflection_property_runtime_name(ctx, reflection_property, expr);
    let result = ctx.emit_value(
        Op::DynamicPropGet,
        vec![object.value, property.value],
        None,
        PhpType::Mixed,
        Op::DynamicPropGet.default_effects(),
        Some(expr.span),
    );
    if ctx.value_is_owning_temporary(property) {
        crate::ir_lower::ownership::release_if_owned(ctx, property, Some(expr.span));
    }
    Some(stabilize_borrowed_result_and_release_receiver(
        ctx, object, result, expr.span,
    ))
}

/// Lowers a raw reflected-property write whose property name is known only at runtime.
fn lower_reflection_property_set_runtime_raw_value(
    ctx: &mut LoweringContext<'_, '_>,
    reflection_property: LoweredValue,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let (object_arg, value_arg) = reflection_property_set_value_args(args)?;
    let object = lower_reflection_property_object_argument(ctx, &object_arg, expr);
    let object = box_generic_object_for_dynamic_property(ctx, object, expr.span);
    let value = lower_expr(ctx, &value_arg);
    let property = lower_reflection_property_runtime_name(ctx, reflection_property, expr);
    ctx.emit_void(
        Op::DynamicPropSet,
        vec![object.value, property.value, value.value],
        None,
        Op::DynamicPropSet.default_effects(),
        Some(expr.span),
    );
    if ctx.value_is_owning_temporary(property) {
        crate::ir_lower::ownership::release_if_owned(ctx, property, Some(expr.span));
    }
    if ctx.value_is_owning_temporary(object) {
        crate::ir_lower::ownership::release_if_owned(ctx, object, Some(expr.span));
    }
    Some(lower_null(ctx, expr))
}

/// Lowers `ReflectionProperty::getValue($object)` to a direct property read.
pub(super) fn lower_reflection_property_get_value(
    ctx: &mut LoweringContext<'_, '_>,
    property: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let object_arg = reflection_property_get_value_arg(args)?;
    let object = lower_expr(ctx, &object_arg);
    Some(lower_property_get_from_value(
        ctx,
        object,
        property,
        Op::PropGet,
        expr,
    ))
}

/// Lowers a raw ReflectionProperty read to the reflected backing slot without hooks.
pub(super) fn lower_reflection_property_get_raw_value(
    ctx: &mut LoweringContext<'_, '_>,
    property: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let object_arg = reflection_property_get_value_arg(args)?;
    let object = lower_expr(ctx, &object_arg);
    Some(lower_raw_property_get_from_value(
        ctx, object, property, expr,
    ))
}

/// Lowers `ReflectionProperty::setValue($object, $value)` to a direct property write.
pub(super) fn lower_reflection_property_set_value(
    ctx: &mut LoweringContext<'_, '_>,
    property: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let (object_arg, value_arg) = reflection_property_set_value_args(args)?;
    let target = Expr::new(
        ExprKind::PropertyAccess {
            object: Box::new(object_arg),
            property: property.to_string(),
        },
        expr.span,
    );
    lower_non_local_assignment_write(ctx, &target, &value_arg, expr.span);
    Some(lower_null(ctx, expr))
}

/// Lowers a raw ReflectionProperty write to the reflected backing slot without hooks.
pub(super) fn lower_reflection_property_set_raw_value(
    ctx: &mut LoweringContext<'_, '_>,
    property: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let (object_arg, value_arg) = reflection_property_set_value_args(args)?;
    crate::ir_lower::stmt::lower_raw_property_assign(
        ctx,
        &object_arg,
        property,
        &value_arg,
        expr.span,
    );
    Some(lower_null(ctx, expr))
}

/// Lowers `ReflectionProperty::isInitialized($object)` to a direct slot probe.
pub(super) fn lower_reflection_property_is_initialized(
    ctx: &mut LoweringContext<'_, '_>,
    property: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let object_arg = reflection_property_get_value_arg(args)?;
    let object = lower_expr(ctx, &object_arg);
    let data = ctx.intern_string(property);
    Some(ctx.emit_value(
        Op::PropInitialized,
        vec![object.value],
        Some(Immediate::Data(data)),
        PhpType::Bool,
        Op::PropInitialized.default_effects(),
        Some(expr.span),
    ))
}

/// Lowers static `ReflectionProperty::getValue()` to a reflection static-property read.
pub(super) fn lower_reflection_property_get_static_value(
    ctx: &mut LoweringContext<'_, '_>,
    declaring_class: &str,
    property: &str,
    property_ty: PhpType,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if let Some(ignored_object) = reflection_property_static_get_value_ignored_arg(args)? {
        lower_ignored_reflection_argument(ctx, &ignored_object);
    }
    Some(lower_reflection_static_property_get_by_class_name(
        ctx,
        declaring_class,
        property,
        property_ty,
        expr,
    ))
}

/// Lowers static `ReflectionProperty::isInitialized()` to a direct static-slot probe.
pub(super) fn lower_reflection_property_static_is_initialized(
    ctx: &mut LoweringContext<'_, '_>,
    declaring_class: &str,
    property: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if let Some(ignored_object) = reflection_property_static_get_value_ignored_arg(args)? {
        lower_ignored_reflection_argument(ctx, &ignored_object);
    }
    Some(lower_reflection_static_property_initialized_by_class_name(
        ctx,
        declaring_class,
        property,
        expr,
    ))
}

/// Lowers static `ReflectionProperty::setValue(null, $value)` to a reflection static-property write.
pub(super) fn lower_reflection_property_set_static_value(
    ctx: &mut LoweringContext<'_, '_>,
    declaring_class: &str,
    property: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let (ignored_object, value_arg) = reflection_property_static_set_value_args(args)?;
    lower_ignored_reflection_argument(ctx, &ignored_object);
    let value = lower_expr(ctx, &value_arg);
    store_reflection_static_property_by_class_name(
        ctx,
        declaring_class,
        property,
        value.value,
        expr.span,
    );
    Some(lower_null(ctx, expr))
}

/// Evaluates an ignored Reflection argument and releases temporary objects.
pub(super) fn lower_ignored_reflection_argument(ctx: &mut LoweringContext<'_, '_>, arg: &Expr) {
    let value = lower_expr(ctx, arg);
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(arg.span));
    }
}

/// Returns the explicit object argument passed to `ReflectionProperty::getValue()`.
pub(super) fn reflection_property_get_value_arg(args: &[Expr]) -> Option<Expr> {
    let args = reflection_class_new_instance_args(args);
    if args.iter().any(is_spread_arg) {
        return None;
    }
    let object = if !crate::types::call_args::has_named_args(&args) {
        match args.as_slice() {
            [object] => object.clone(),
            _ => return None,
        }
    } else {
        reflection_property_named_object_arg(&args)?
    };
    (!matches!(&object.kind, ExprKind::Null)).then_some(object)
}

/// Returns the explicit object and value arguments passed to `ReflectionProperty::setValue()`.
pub(super) fn reflection_property_set_value_args(args: &[Expr]) -> Option<(Expr, Expr)> {
    let args = reflection_class_new_instance_args(args);
    if args.iter().any(is_spread_arg) {
        return None;
    }
    let (object, value) =
        reflection_class_static_property_regular_args(&args, "object", Some("value"))?;
    let object = object?;
    if matches!(&object.kind, ExprKind::Null) {
        return None;
    }
    Some((object, value?))
}

/// Returns the optional ignored object argument for static `ReflectionProperty::getValue()`.
pub(super) fn reflection_property_static_get_value_ignored_arg(args: &[Expr]) -> Option<Option<Expr>> {
    let args = reflection_class_new_instance_args(args);
    if args.iter().any(is_spread_arg) {
        return None;
    }
    if !crate::types::call_args::has_named_args(&args) {
        return match args.as_slice() {
            [] => Some(None),
            [object] => Some(Some(object.clone())),
            _ => None,
        };
    }
    reflection_property_named_optional_object_arg(&args)
}

/// Returns the ignored object and value arguments for static `ReflectionProperty::setValue()`.
pub(super) fn reflection_property_static_set_value_args(args: &[Expr]) -> Option<(Expr, Expr)> {
    let args = reflection_class_new_instance_args(args);
    if args.iter().any(is_spread_arg) {
        return None;
    }
    let (object, value) =
        reflection_class_static_property_regular_args(&args, "object", Some("value"))?;
    Some((object?, value?))
}

/// Returns a required named `object` argument for ReflectionProperty value access.
pub(super) fn reflection_property_named_object_arg(args: &[Expr]) -> Option<Expr> {
    reflection_property_named_optional_object_arg(args)?
}

/// Returns an optional named `object` argument for ReflectionProperty value access.
pub(super) fn reflection_property_named_optional_object_arg(args: &[Expr]) -> Option<Option<Expr>> {
    let mut object = None;
    for arg in args {
        match &arg.kind {
            ExprKind::NamedArg { name, value } if php_symbol_key(name) == "object" => {
                object = Some((**value).clone());
            }
            _ => return None,
        }
    }
    Some(object)
}

/// Resolves an inline `new ReflectionProperty(Known::class, "prop")` instance property target.
pub(super) fn reflection_property_instance_target(
    ctx: &LoweringContext<'_, '_>,
    object_expr: &Expr,
) -> Option<(String, String, PhpType)> {
    let (class_name, property) = reflection_property_reflected_target(ctx, object_expr)?;
    let class_info = ctx.classes.get(class_name.trim_start_matches('\\'))?;
    if class_info
        .static_properties
        .iter()
        .any(|(name, _)| name == &property)
    {
        return None;
    }
    if class_info.property_visibilities.get(&property) != Some(&Visibility::Public) {
        return None;
    }
    let (_, (_, property_ty)) = class_info.visible_property(&property)?;
    Some((
        class_name,
        property,
        normalize_value_php_type(property_ty.codegen_repr()),
    ))
}

/// Resolves a known non-static ReflectionProperty target without enforcing visibility.
pub(super) fn reflection_property_any_instance_target(
    ctx: &LoweringContext<'_, '_>,
    object_expr: &Expr,
) -> Option<(String, String, PhpType)> {
    let (class_name, property) = reflection_property_reflected_target(ctx, object_expr)?;
    let class_info = ctx.classes.get(class_name.trim_start_matches('\\'))?;
    if class_info
        .static_properties
        .iter()
        .any(|(name, _)| name == &property)
    {
        return None;
    }
    let (_, (_, property_ty)) = class_info.visible_property(&property)?;
    Some((
        class_name,
        property,
        normalize_value_php_type(property_ty.codegen_repr()),
    ))
}

/// Resolves an inline `ReflectionProperty` target for a static property.
pub(super) fn reflection_property_static_target(
    ctx: &LoweringContext<'_, '_>,
    object_expr: &Expr,
) -> Option<(String, String, PhpType)> {
    let (class_name, property) = reflection_property_reflected_target(ctx, object_expr)?;
    let (declaring_class, property_ty) =
        reflection_class_static_property_target(ctx, &class_name, &property)?;
    Some((declaring_class, property, property_ty))
}

/// Extracts the known class and property name from a supported ReflectionProperty source.
pub(super) fn reflection_property_reflected_target(
    ctx: &LoweringContext<'_, '_>,
    object_expr: &Expr,
) -> Option<(String, String)> {
    reflection_property_constructor_target(ctx, object_expr)
        .or_else(|| reflection_property_class_get_property_target(ctx, object_expr))
        .or_else(|| reflection_property_class_get_properties_index_target(ctx, object_expr))
        .or_else(|| {
            let ExprKind::Variable(name) = &object_expr.kind else {
                return None;
            };
            ctx.reflection_property_local(name)
        })
}

/// Extracts the known class and method name from a supported ReflectionMethod source.
pub(super) fn reflection_method_reflected_target(
    ctx: &LoweringContext<'_, '_>,
    object_expr: &Expr,
) -> Option<(String, String)> {
    reflection_method_constructor_target(ctx, object_expr)
        .or_else(|| reflection_method_class_get_constructor_target(ctx, object_expr))
        .or_else(|| reflection_method_class_get_method_target(ctx, object_expr))
        .or_else(|| reflection_method_class_get_methods_index_target(ctx, object_expr))
        .or_else(|| {
            let ExprKind::Variable(name) = &object_expr.kind else {
                return None;
            };
            ctx.reflection_method_local(name)
        })
}

/// Extracts the known function name from a supported ReflectionFunction source.
pub(super) fn reflection_function_reflected_target(
    ctx: &LoweringContext<'_, '_>,
    object_expr: &Expr,
) -> Option<String> {
    reflection_function_constructor_target(ctx, object_expr).or_else(|| {
        let ExprKind::Variable(name) = &object_expr.kind else {
            return None;
        };
        ctx.reflection_function_local(name)
    })
}
