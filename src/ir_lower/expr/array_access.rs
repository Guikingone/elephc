//! Purpose:
//! Array, hash, string, and ArrayAccess read lowering.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers array, hash, string, or ArrayAccess indexing.
pub(super) fn lower_array_access(
    ctx: &mut LoweringContext<'_, '_>,
    array: &Expr,
    index: &Expr,
    expr: &Expr,
) -> LoweredValue {
    lower_array_access_with_missing_warning(ctx, array, index, expr, true)
}

/// Lowers array, hash, string, or ArrayAccess indexing with configurable
/// undefined-offset warning behavior for native indexed-array reads. Suppressed
/// warnings propagate through the whole subscript chain: PHP's `isset()` and `??`
/// are silent for every level of `$a[1][2][3]`, not just the outermost read.
pub(super) fn lower_array_access_with_missing_warning(
    ctx: &mut LoweringContext<'_, '_>,
    array: &Expr,
    index: &Expr,
    expr: &Expr,
    warn_on_missing: bool,
) -> LoweredValue {
    let array_value = if warn_on_missing {
        lower_expr(ctx, array)
    } else {
        lower_subscript_receiver_silently(ctx, array)
    };
    if value_is_nullable(ctx, array_value.value) {
        return lower_nullable_array_access(ctx, array_value, index, expr, warn_on_missing);
    }
    lower_array_access_from_value(ctx, array_value, index, expr, warn_on_missing)
}

/// Lowers a subscript-chain receiver with undefined-offset warnings suppressed on
/// nested array reads, so `isset()`/`??` stay silent across chained subscripts.
pub(super) fn lower_subscript_receiver_silently(
    ctx: &mut LoweringContext<'_, '_>,
    array: &Expr,
) -> LoweredValue {
    if let ExprKind::ArrayAccess { array: inner_array, index: inner_index } = &array.kind {
        return lower_array_access_with_missing_warning(ctx, inner_array, inner_index, array, false);
    }
    lower_expr(ctx, array)
}

/// Lowers array access once the receiver is already evaluated.
pub(super) fn lower_array_access_from_value(
    ctx: &mut LoweringContext<'_, '_>,
    array_value: LoweredValue,
    index: &Expr,
    expr: &Expr,
    warn_on_missing: bool,
) -> LoweredValue {
    let mut index_value = lower_expr(ctx, index);
    let op = match array_value.ir_type {
        IrType::Heap(IrHeapKind::Array) => {
            let index_ty = index_expr_key_type(ctx, index);
            if index_ty == PhpType::Int {
                index_value = coerce_to_int_at_span(ctx, index_value, Some(index.span));
                if warn_on_missing {
                    Op::ArrayGet
                } else {
                    Op::ArrayGetSilent
                }
            } else {
                // String or Mixed key on indexed storage: use the mixed-key
                // runtime read path (mirrors Op::ArraySetMixedKey for writes).
                if warn_on_missing {
                    Op::ArrayGetMixedKey
                } else {
                    Op::ArrayGetMixedKeySilent
                }
            }
        }
        IrType::Heap(IrHeapKind::Hash) => {
            if warn_on_missing {
                Op::HashGet
            } else {
                Op::HashGetSilent
            }
        }
        IrType::Heap(IrHeapKind::Buffer) => {
            index_value = coerce_to_int_at_span(ctx, index_value, Some(index.span));
            Op::BufferGet
        }
        IrType::Str => {
            index_value = coerce_to_int_at_span(ctx, index_value, Some(index.span));
            Op::StrCharAt
        }
        _ => Op::RuntimeCall,
    };
    let result_type = array_access_result_type(ctx, array_value.value, op, expr);
    let mut operands = vec![array_value.value, index_value.value];
    if matches!(op, Op::RuntimeCall) {
        let warning_flag = emit_bool_literal(ctx, warn_on_missing, Some(expr.span));
        operands.push(warning_flag.value);
    }
    let result = ctx.emit_value(
        op,
        operands,
        None,
        result_type,
        op.default_effects(),
        Some(expr.span),
    );
    // An owning boxed index temporary (e.g. `$B[$i + 1]` on the mixed-key read
    // path) is consumed by the read without any runtime refcount operation on
    // the key, and the result is freshly allocated storage that never aliases
    // it — release it here or it leaks per read (issue #500). Int-coerced
    // index paths rebound `index_value` to a non-owning raw cast, so the
    // owning-temporary gate makes this a no-op for them.
    release_coerced_source_if_owned(ctx, index_value, Some(index.span));
    // Array access consumes an owning receiver produced by an earlier read,
    // call, or one-shot temp. Preserve borrowed string/callable payloads before
    // dropping that receiver; boxed and retained container reads are already
    // independent and must not be acquired twice.
    stabilize_borrowed_result_and_release_receiver(ctx, array_value, result, expr.span)
}

/// Lowers nullable receiver indexing without evaluating the index on a null receiver.
pub(super) fn lower_nullable_array_access(
    ctx: &mut LoweringContext<'_, '_>,
    array_value: LoweredValue,
    index: &Expr,
    expr: &Expr,
    warn_on_missing: bool,
) -> LoweredValue {
    let is_null = ctx.emit_value(
        Op::IsNull,
        vec![array_value.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        Some(expr.span),
    );
    let result_type = PhpType::Mixed;
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let null_block = ctx
        .builder
        .create_named_block("nullable.index.null", Vec::new());
    let read_block = ctx
        .builder
        .create_named_block("nullable.index.read", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("nullable.index.merge", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: null_block,
        then_args: Vec::new(),
        else_target: read_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(null_block);
    let null_value = lower_boxed_null(ctx, expr);
    store_value_into_temp(ctx, &temp_name, result_type.clone(), null_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(read_block);
    let read_value = lower_array_access_from_value(ctx, array_value, index, expr, warn_on_missing);
    store_value_into_temp(ctx, &temp_name, result_type, read_value, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    take_owned_temp(ctx, &temp_name, expr.span)
}

/// Lowers a subscript read whose receiver has already been evaluated,
/// including the nullable-receiver guard. Used by the nested-assign parent
/// lowering when a receiver produced by a for-write chain turns out not to be
/// a boxed Mixed cell (e.g. ArrayAccess object intermediates, issue #555).
pub(crate) fn lower_array_access_from_lowered_receiver(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: LoweredValue,
    index: &Expr,
    expr: &Expr,
) -> LoweredValue {
    if value_is_nullable(ctx, receiver.value) {
        return lower_nullable_array_access(ctx, receiver, index, expr, true);
    }
    lower_array_access_from_value(ctx, receiver, index, expr, true)
}

/// Returns the statically-known key type for an array index expression.
/// Used to decide between Op::ArrayGet (int key) and Op::ArrayGetMixedKey.
pub(crate) fn index_expr_key_type(_ctx: &LoweringContext<'_, '_>, index: &Expr) -> PhpType {
    let ty = infer_expr_type_syntactic(index);
    normalized_array_key_type(index, ty)
}

/// Returns the best PHP result type for a lowered array/string/hash access.
pub(super) fn array_access_result_type(
    ctx: &LoweringContext<'_, '_>,
    array: crate::ir::ValueId,
    op: Op,
    expr: &Expr,
) -> PhpType {
    match op {
        Op::StrCharAt => PhpType::Str,
        Op::ArrayGet | Op::ArrayGetSilent => match ctx.builder.value_php_type(array).codegen_repr() {
            PhpType::Array(elem_ty) => {
                array_access_element_result_type(normalize_value_php_type(*elem_ty))
            }
            _ => fallback_expr_type(expr),
        },
        Op::HashGet | Op::HashGetSilent => match ctx.builder.value_php_type(array).codegen_repr() {
            PhpType::AssocArray { value, .. } => {
                array_access_element_result_type(normalize_value_php_type(*value))
            }
            _ => fallback_expr_type(expr),
        },
        Op::BufferGet => match ctx.builder.value_php_type(array).codegen_repr() {
            PhpType::Buffer(elem_ty) => normalize_value_php_type(*elem_ty),
            _ => fallback_expr_type(expr),
        },
        Op::RuntimeCall => array_access_runtime_call_result_type(ctx, array, expr),
        Op::ArrayGetMixedKey | Op::ArrayGetMixedKeySilent => PhpType::Mixed,
        _ => match ctx.builder.value_php_type(array).codegen_repr() {
            PhpType::Mixed | PhpType::Union(_) => PhpType::Mixed,
            _ => fallback_expr_type(expr),
        },
    }
}

/// Returns the materialized result type for a PHP array read, including miss-capable int reads.
pub(crate) fn array_access_element_result_type(element_ty: PhpType) -> PhpType {
    if crate::codegen::sentinels::null_repr_is_tagged() && matches!(element_ty, PhpType::Int) {
        PhpType::TaggedScalar
    } else {
        element_ty
    }
}

/// Returns the EIR result type for object indexing routed through `ArrayAccess::offsetGet`.
pub(super) fn array_access_runtime_call_result_type(
    ctx: &LoweringContext<'_, '_>,
    array: crate::ir::ValueId,
    expr: &Expr,
) -> PhpType {
    match ctx.builder.value_php_type(array).codegen_repr() {
        PhpType::Object(class_name) => array_access_offset_get_return_type(ctx, &class_name)
            .unwrap_or_else(|| fallback_expr_type(expr)),
        PhpType::Mixed => PhpType::Mixed,
        _ => fallback_expr_type(expr),
    }
}

/// Looks up the effective `offsetGet` return type for an ArrayAccess class.
pub(super) fn array_access_offset_get_return_type(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
) -> Option<PhpType> {
    if !object_name_satisfies_interface_for_ir(ctx, class_name, "ArrayAccess") {
        return None;
    }
    let method_key = php_symbol_key("offsetGet");
    class_method_return_type_for_ir(ctx, class_name, &method_key)
        .or_else(|| interface_method_return_type_for_ir(ctx, "ArrayAccess", &method_key))
        .map(normalize_value_php_type)
}

/// Returns true when a syntactic array receiver is statically known as `ArrayAccess`.
pub(super) fn array_access_expr_satisfies_array_access(
    ctx: &LoweringContext<'_, '_>,
    array: &Expr,
) -> bool {
    let ty = match &array.kind {
        ExprKind::Variable(name) => ctx
            .local_types
            .get(name)
            .cloned()
            .unwrap_or_else(|| infer_expr_type_syntactic(array)),
        _ => infer_expr_type_syntactic(array),
    };
    type_satisfies_array_access_for_ir(ctx, &ty)
}

