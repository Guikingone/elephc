//! Purpose:
//! Method signatures, aliases, and late-static result metadata.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Returns the checked signature for an instance method call when metadata is available.
pub(super) fn method_signature(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
) -> Option<FunctionSig> {
    let object_ty = ctx.builder.value_php_type(object);
    let key = php_symbol_key(method);
    if let Some((class_name, _)) = singular_object_class(&object_ty) {
        let normalized = class_name.trim_start_matches('\\');
        if ctx.interfaces.contains_key(normalized) {
            if let Some(signature) = narrowed_runtime_method_signature(ctx, normalized, &key) {
                return Some(signature);
            }
        }
        return class_method_signature(ctx, normalized, &key)
            .cloned()
            .or_else(|| narrowed_runtime_method_signature(ctx, normalized, &key));
    }
    if dynamic_method_receiver_needs_mixed_fallback(&object_ty) {
        if ctx.has_eval_barrier() {
            return None;
        }
        return common_dynamic_method_signature(ctx, &key);
    }
    None
}

/// Returns a shared concrete-subtype signature when a nominal base lacks the requested method.
///
/// PHP permits a runtime instance of a descendant to expose a method absent from the static base
/// type. The checker accepts that path through the same closed-world subtype set; preserving one
/// common signature here lets EIR materialize omitted default arguments before codegen performs
/// the class-id dispatch.
fn narrowed_runtime_method_signature(
    ctx: &LoweringContext<'_, '_>,
    receiver_name: &str,
    method_key: &str,
) -> Option<FunctionSig> {
    let receiver_is_interface = ctx.interfaces.contains_key(receiver_name);
    let mut signature = None;
    for (class_name, class_info) in ctx.classes {
        let compatible = if receiver_is_interface {
            class_implements_interface_for_ir(ctx, class_name, receiver_name)
        } else {
            class_extends_class(ctx, class_name, receiver_name)
        };
        if !compatible {
            continue;
        }
        let Some(candidate) = class_info.methods.get(method_key).cloned() else {
            continue;
        };
        match &signature {
            Some(current) if current != &candidate => return None,
            Some(_) => {}
            None => signature = Some(candidate),
        }
    }
    signature
}

/// Promotes the writable destination used by PDOStatement binding methods to a durable Mixed cell.
pub(super) fn promote_pdo_binding_ref_argument(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
    args: &[Expr],
) {
    if !type_may_be_pdo_statement(ctx, &ctx.builder.value_php_type(object)) {
        return;
    }
    let parameter_name = match php_symbol_key(method).as_str() {
        "bindparam" => "variable",
        "bindcolumn" => "var",
        _ => return,
    };
    let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
    let argument = expanded_args
        .iter()
        .enumerate()
        .find_map(|(index, arg)| match &arg.kind {
            ExprKind::NamedArg { name, value } if name == parameter_name => Some(value.as_ref()),
            ExprKind::NamedArg { .. } => None,
            _ if index == 1 => Some(arg),
            _ => None,
        });
    let Some(Expr {
        kind: ExprKind::Variable(name),
        span,
    }) = argument
    else {
        return;
    };
    ctx.promote_local_mixed_ref_cell(name, Some(*span));
}

/// Returns whether a receiver type can dispatch to PDOStatement binding methods.
fn type_may_be_pdo_statement(ctx: &LoweringContext<'_, '_>, ty: &PhpType) -> bool {
    match ty {
        PhpType::Object(class) => class_extends_class(ctx, class, "PDOStatement"),
        PhpType::Union(members) => members
            .iter()
            .any(|member| type_may_be_pdo_statement(ctx, member)),
        _ => false,
    }
}

/// Returns the conservative return-to-argument alias summary for a method dispatch.
///
/// A non-final receiver type includes every closed-world descendant implementation,
/// because runtime dispatch can select an override. Missing or synthetic summaries
/// therefore fall back to `Unknown` rather than enabling unsafe cleanup.
pub(super) fn method_return_arg_alias(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
) -> ReturnArgAlias {
    let object_ty = ctx.builder.value_php_type(object);
    let method_key = php_symbol_key(method);
    let mut summary: Option<ReturnArgAlias> = None;
    if let Some((class_name, _)) = singular_object_class(&object_ty) {
        let base_class = class_name.trim_start_matches('\\');
        let Some(base_info) = ctx.classes.get(base_class) else {
            return ReturnArgAlias::Unknown;
        };
        if base_info.is_final || base_info.final_methods.contains(&method_key) {
            return class_method_return_arg_alias(ctx, base_class, &method_key)
                .unwrap_or(ReturnArgAlias::Unknown);
        }
        for candidate in ctx.classes.keys() {
            if !is_same_or_descendant_class(ctx, candidate, base_class) {
                continue;
            }
            let Some(alias) = class_method_return_arg_alias(ctx, candidate, &method_key) else {
                continue;
            };
            summary = Some(match summary {
                Some(current) => current.merge(&alias),
                None => alias,
            });
        }
        return summary.unwrap_or(ReturnArgAlias::Unknown);
    }
    if dynamic_method_receiver_needs_mixed_fallback(&object_ty) {
        if ctx.has_eval_barrier() {
            return ReturnArgAlias::Unknown;
        }
        for candidate in ctx.classes.keys() {
            let Some(alias) = class_method_return_arg_alias(ctx, candidate, &method_key) else {
                continue;
            };
            summary = Some(match summary {
                Some(current) => current.merge(&alias),
                None => alias,
            });
        }
    }
    summary.unwrap_or(ReturnArgAlias::Unknown)
}

/// Resolves one concrete class's dispatched implementation and its source summary.
pub(super) fn class_method_return_arg_alias(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
    method_key: &str,
) -> Option<ReturnArgAlias> {
    class_method_signature(ctx, class_name, method_key)?;
    let class_info = ctx.classes.get(class_name)?;
    let impl_class = class_info
        .method_impl_classes
        .get(method_key)
        .map(String::as_str)
        .unwrap_or(class_name);
    Some(
        ctx.return_alias_summaries
            .method(impl_class, method_key)
            .cloned()
            .unwrap_or(ReturnArgAlias::Unknown),
    )
}

/// Returns a class/interface method signature, preferring the implementing class metadata.
pub(super) fn class_method_signature<'a>(
    ctx: &'a LoweringContext<'_, '_>,
    class_name: &str,
    method_key: &str,
) -> Option<&'a FunctionSig> {
    let normalized = class_name.trim_start_matches('\\');
    if let Some(class_info) = ctx.classes.get(normalized) {
        let impl_class = class_info
            .method_impl_classes
            .get(method_key)
            .map(String::as_str)
            .unwrap_or(normalized);
        return ctx
            .classes
            .get(impl_class)
            .and_then(|impl_info| impl_info.methods.get(method_key))
            .or_else(|| class_info.methods.get(method_key));
    }
    ctx.interfaces
        .get(normalized)
        .and_then(|interface_info| interface_info.methods.get(method_key))
}

/// Returns the checked return type for an instance method call when metadata is available.
pub(super) fn method_call_result_type(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
    op: Op,
    expr: &Expr,
) -> PhpType {
    let object_ty = ctx.builder.value_php_type(object);
    let nullable = singular_object_class(&object_ty)
        .map(|(_, nullable)| nullable)
        .unwrap_or(false);
    let Some(return_ty) = method_signature(ctx, object, method)
        .map(|signature| normalize_value_php_type(signature.return_type))
    else {
        if let Some(return_ty) = narrowed_runtime_method_return_type(ctx, &object_ty, method) {
            return if op == Op::NullsafeMethodCall && nullable {
                nullable_result_type(return_ty)
            } else {
                return_ty
            };
        }
        if dynamic_method_receiver_needs_mixed_fallback(&object_ty) {
            return PhpType::Mixed;
        }
        return fallback_expr_type(expr);
    };
    // A common same-named signature can normalize arguments for a gradual receiver, but a
    // nominal object return does not prove the receiver belongs to that unrelated class. Match
    // the checker's Mixed result so chained calls retain runtime dispatch.
    let return_ty = if dynamic_method_receiver_needs_mixed_fallback(&object_ty)
        && type_mentions_nominal_object(&return_ty)
    {
        PhpType::Mixed
    } else {
        return_ty
    };
    let return_ty = if let Some((receiver_name, _)) = singular_object_class(&object_ty) {
        instance_method_late_static_return_for_ir(ctx, receiver_name, &php_symbol_key(method))
            .map(|return_type| late_static_return_type_for_ir(ctx, &return_type, receiver_name))
            .unwrap_or(return_ty)
    } else {
        return_ty
    };
    if op == Op::NullsafeMethodCall && nullable {
        nullable_result_type(return_ty)
    } else {
        return_ty
    }
}

/// Infers the result shared by concrete runtime classes compatible with a narrowed receiver.
///
/// Flow-sensitive `instanceof` validation can accept a method that is absent from the receiver's
/// original nominal class or interface. EIR retains that original receiver type, so recover the
/// concrete return contract from the same closed-world class set used by backend class-id dispatch.
fn narrowed_runtime_method_return_type(
    ctx: &LoweringContext<'_, '_>,
    receiver_type: &PhpType,
    method: &str,
) -> Option<PhpType> {
    let (receiver_name, _) = singular_object_class(receiver_type)?;
    let receiver_name = receiver_name.trim_start_matches('\\');
    let method_key = php_symbol_key(method);
    let receiver_is_interface = ctx.interfaces.contains_key(receiver_name);
    let candidates = ctx
        .classes
        .iter()
        .filter(|(class_name, _)| {
            receiver_name.is_empty()
                || if receiver_is_interface {
                    class_implements_interface_for_ir(ctx, class_name, receiver_name)
                } else {
                    class_extends_class(ctx, class_name, receiver_name)
                }
        })
        .filter_map(|(_, class_info)| class_info.methods.get(&method_key))
        .map(|signature| normalize_value_php_type(signature.return_type.codegen_repr()))
        .collect::<Vec<_>>();
    normalize_union_members(candidates)
}

/// Returns whether a type contains a nominal object member whose runtime class is not proven.
fn type_mentions_nominal_object(php_type: &PhpType) -> bool {
    match php_type {
        PhpType::Object(_) => true,
        PhpType::Union(members) => members.iter().any(type_mentions_nominal_object),
        _ => false,
    }
}

/// Returns preserved late-static return syntax for EIR instance dispatch.
pub(super) fn instance_method_late_static_return_for_ir(
    ctx: &LoweringContext<'_, '_>,
    receiver_type: &str,
    method_key: &str,
) -> Option<TypeExpr> {
    let normalized = receiver_type.trim_start_matches('\\');
    if let Some(class_info) = ctx.classes.get(normalized) {
        if let Some(return_type) = class_info.late_static_method_returns.get(method_key) {
            return Some(return_type.clone());
        }
    }
    ctx.interfaces
        .get(normalized)
        .and_then(|interface_info| interface_info.late_static_method_returns.get(method_key))
        .cloned()
}

/// Binds preserved late-static return syntax to an EIR call-site receiver type.
pub(super) fn late_static_return_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    return_type: &TypeExpr,
    receiver_type: &str,
) -> PhpType {
    let bound = return_type.substitute_relative_class_types(receiver_type, None);
    normalize_value_php_type(ctx.type_expr_to_php_type_for_value(&bound))
}

/// Returns a common method signature for dynamic receivers when every candidate agrees.
pub(super) fn common_dynamic_method_signature(
    ctx: &LoweringContext<'_, '_>,
    method_key: &str,
) -> Option<FunctionSig> {
    let mut common = None;
    for class_name in ctx.classes.keys() {
        let Some(signature) = class_method_signature(ctx, class_name, method_key).cloned() else {
            continue;
        };
        match common.as_ref() {
            Some(existing) if existing != &signature => return None,
            Some(_) => {}
            None => common = Some(signature),
        }
    }
    common
}

/// Returns true when an instance-method receiver has no single compile-time class.
pub(super) fn dynamic_method_receiver_needs_mixed_fallback(php_type: &PhpType) -> bool {
    match php_type {
        PhpType::Mixed => true,
        PhpType::Union(members) => members
            .iter()
            .any(|member| matches!(member, PhpType::Mixed | PhpType::Object(_))),
        _ => false,
    }
}
