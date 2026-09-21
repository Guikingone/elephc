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
        // The fold below scans the whole class table, and its answer depends only on the
        // base class, the method name and metadata `LoweringContext` borrows as `&'m`.
        let fold_key = crate::types::DispatchFoldKey {
            base_class: Some(base_class.to_string()),
            method_key: method_key.clone(),
        };
        if let Some(cached) = ctx.return_alias_summaries.cached_dispatch_fold(&fold_key) {
            return cached;
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
        let folded = summary.unwrap_or(ReturnArgAlias::Unknown);
        ctx.return_alias_summaries
            .store_dispatch_fold(fold_key, folded.clone());
        return folded;
    }
    if dynamic_method_receiver_needs_mixed_fallback(&object_ty) {
        if ctx.has_eval_barrier() {
            return ReturnArgAlias::Unknown;
        }
        // The `mixed` fallback has no receiver to narrow by, so it folds over EVERY class —
        // and the answer therefore depends on nothing but the method name.
        let fold_key = crate::types::DispatchFoldKey {
            base_class: None,
            method_key: method_key.clone(),
        };
        if let Some(cached) = ctx.return_alias_summaries.cached_dispatch_fold(&fold_key) {
            return cached;
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
        let folded = summary.unwrap_or(ReturnArgAlias::Unknown);
        ctx.return_alias_summaries
            .store_dispatch_fold(fold_key, folded.clone());
        return folded;
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
        .filter(|signature| {
            // For a GRADUAL receiver the common signature is one class's, chosen only because it
            // shares the method's name. It may normalize arguments, but it decides the result only
            // if it could actually be CALLED this way — the same arity test the checker applies in
            // `mixed_receiver_method_return_type`. Symfony's `PdoAdapter::doFetch` is why:
            // `$stmt->execute()` matched `Console\Command\Command::execute($input, $output): int`,
            // two required parameters and an unrelated class, so `$result` became `int` and
            // `$result->iterateNumeric()` reached the backend as `method call receiver for PHP
            // type Int`.
            !dynamic_method_receiver_needs_mixed_fallback(&object_ty)
                || method_call_argument_count(expr)
                    .is_none_or(|count| signature_accepts_argument_count(signature, count))
        })
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
    // A common same-named signature can normalize arguments for a gradual receiver, but it does
    // not prove the receiver belongs to that class, so it decides no result AT ALL -- not a
    // nominal object and not a scalar either. The scalar half is not theoretical: `twig/twig`'s
    // `AbstractTokenParser::$parser` is declared with a docblock and no type, and the only
    // `getEnvironment()` this world compiles is `AppVariable`'s `: string`, so
    // `$this->parser->getEnvironment()->getExpressionParsers()` reached the backend as
    // `method call receiver for PHP type Str`. Codegen boxes a Mixed and dispatches it at
    // runtime, which is what PHP does with a receiver whose class is only known then.
    let return_ty = if dynamic_method_receiver_needs_mixed_fallback(&object_ty) {
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
    // Comparing borrowed signatures and cloning only the winner: the previous form cloned a
    // whole `FunctionSig` for every class in the module on every call, to throw all but one away.
    let mut common: Option<&FunctionSig> = None;
    for class_name in ctx.classes.keys() {
        let Some(signature) = class_method_signature(ctx, class_name, method_key) else {
            continue;
        };
        match common {
            Some(existing) if existing != signature => return None,
            Some(_) => {}
            None => common = Some(signature),
        }
    }
    common.cloned()
}

/// Returns the callee signature for a call that passes `arg_count` arguments.
///
/// Identical to [`method_signature`] for a receiver with a single compile-time class. For a
/// GRADUAL one the answer is a common same-named signature chosen across every class that declares
/// the method, and one that could not be CALLED this way is not the callee — so it is dropped
/// rather than used to materialize arguments. Symfony's `HtmlErrorRenderer` reached
/// `$request->headers->get('X-Php-Ob-Level', -1)` through a gradual `headers`, matched a `get`
/// with a by-reference parameter beyond the two supplied, and the backend refused with
/// `receiver-register method call with missing non-value parameter`.
pub(super) fn method_signature_for_call(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
    arg_count: usize,
) -> Option<FunctionSig> {
    let signature = method_signature(ctx, object, method)?;
    if !dynamic_method_receiver_needs_mixed_fallback(&ctx.builder.value_php_type(object)) {
        return Some(signature);
    }
    if !signature_accepts_argument_count(&signature, arg_count) {
        return None;
    }
    // An omitted by-reference parameter is NOT a reason to drop the signature: `pad_omitted_by_ref_args`
    // gives it a real place, and dropping it here instead left the call lowering its arguments
    // without a signature while codegen still resolved the callee nominally — two operands for a
    // three-parameter ABI.
    Some(signature)
}

/// Returns the number of arguments a method-call expression passes, when it is one.
fn method_call_argument_count(expr: &Expr) -> Option<usize> {
    match &expr.kind {
        ExprKind::MethodCall { args, .. } | ExprKind::NullsafeMethodCall { args, .. } => {
            Some(args.len())
        }
        _ => None,
    }
}

/// Returns whether a signature can be CALLED with `arg_count` arguments.
///
/// Mirrors `Checker::signature_accepts_argument_count`: `defaults` gives the first optional
/// position, and a variadic tail takes any number beyond the fixed parameters.
fn signature_accepts_argument_count(signature: &FunctionSig, arg_count: usize) -> bool {
    let required = signature
        .defaults
        .iter()
        .position(|default| default.is_some())
        .unwrap_or(signature.params.len());
    if arg_count < required {
        return false;
    }
    signature.variadic.is_some() || arg_count <= signature.params.len()
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
