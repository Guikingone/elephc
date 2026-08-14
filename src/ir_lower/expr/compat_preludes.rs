//! Purpose:
//! Routes PHP builtin call shapes through narrow compatibility preludes when the native EIR
//! representation cannot preserve their semantics.
//!
//! Called from:
//! - `crate::ir_lower::expr::function_calls::lower_function_call()` before registry lowering.
//!
//! Key details:
//! - Routing remains arity- and representation-sensitive; native packed-array paths stay native.

use super::*;

/// Routes `assert($assertion)` to the prelude while the richer description form remains native.
pub(super) fn lower_single_arg_assert(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "assert"
        || args.len() != 1
        || args.iter().any(is_spread_arg)
    {
        return None;
    }
    Some(lower_function_call(
        ctx,
        &crate::names::Name::unqualified(crate::assert_prelude::ASSERT_ONE_ARG_NAME),
        args,
        expr,
    ))
}

/// Routes PHP's two-argument `array_reduce()` form to the boxed-carry prelude helper.
pub(super) fn lower_default_initial_array_reduce(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "array_reduce"
        || args.len() != 2
        || crate::types::call_args::has_named_args(args)
        || args.iter().any(is_spread_arg)
    {
        return None;
    }
    Some(lower_function_call(
        ctx,
        &crate::names::Name::unqualified(
            crate::array_reduce_prelude::ARRAY_REDUCE_DEFAULT_NAME,
        ),
        args,
        expr,
    ))
}

/// Routes narrowly supported builtin arities to their elephc-PHP compatibility helpers.
pub(super) fn lower_backend_gap_builtin_shape(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let builtin = php_symbol_key(name.trim_start_matches('\\'));
    let single_value_arg = match args {
        [arg] if !is_spread_arg(arg) => Some(match &arg.kind {
            ExprKind::NamedArg { value, .. } => value.as_ref(),
            _ => arg,
        }),
        _ => None,
    };
    if matches!(builtin.as_str(), "sort" | "rsort")
        && single_value_arg.is_some_and(|arg| {
            match materialized_expr_type_for_merge(ctx, arg).codegen_repr() {
                PhpType::Array(element) => element.codegen_repr() == PhpType::Mixed,
                PhpType::AssocArray { value, .. } => {
                    matches!(value.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
                }
                PhpType::Mixed | PhpType::Union(_) => true,
                _ => false,
            }
        })
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(if builtin == "sort" {
                crate::backend_gap_prelude::SORT_MIXED_NAME
            } else {
                crate::backend_gap_prelude::RSORT_MIXED_NAME
            }),
            args,
            expr,
        ));
    }
    if crate::types::call_args::has_named_args(args) || args.iter().any(is_spread_arg) {
        return None;
    }
    if builtin == "array_unique"
        && args.len() == 1
        && match materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr() {
            PhpType::Array(element) => element.codegen_repr() == PhpType::Mixed,
            PhpType::AssocArray { value, .. } => {
                matches!(value.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
            }
            PhpType::Mixed | PhpType::Union(_) => true,
            _ => false,
        }
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_UNIQUE_ASSOC_MIXED_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "array_diff" && args.len() == 2 {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_DIFF_STRING_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "array_intersect" && args.len() == 2 {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(crate::backend_gap_prelude::ARRAY_INTERSECT_NAME),
            args,
            expr,
        ));
    }
    if builtin == "array_reverse"
        && args.len() == 1
        && match materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr() {
            PhpType::Array(element) => element.codegen_repr() == PhpType::Str,
            PhpType::Mixed | PhpType::Union(_) => true,
            _ => false,
        }
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_REVERSE_STRING_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "array_search"
        && (2..=3).contains(&args.len())
        && (args.len() == 3
            || matches!(
                materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr(),
                PhpType::Mixed | PhpType::Union(_)
            )
            || matches!(
                materialized_expr_type_for_merge(ctx, &args[1]).codegen_repr(),
                PhpType::Array(element) if element.codegen_repr() == PhpType::Mixed
            ))
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_SEARCH_MIXED_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "array_slice" {
        let gradual_length = args.get(2).is_some_and(|length| {
            matches!(
                materialized_expr_type_for_merge(ctx, length).codegen_repr(),
                PhpType::TaggedScalar | PhpType::Mixed | PhpType::Union(_)
            )
        });
        if !(2..=4).contains(&args.len())
            || !match materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr() {
                PhpType::Array(element) => {
                    element.codegen_repr() == PhpType::Mixed || gradual_length
                }
                PhpType::AssocArray { .. } => true,
                PhpType::Mixed | PhpType::Union(_) => true,
                _ => false,
            }
        {
            return None;
        }
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(crate::backend_gap_prelude::ARRAY_SLICE_NAME),
            args,
            expr,
        ));
    }
    if builtin == "pack" {
        let Some(Expr {
            kind: ExprKind::StringLiteral(format),
            ..
        }) = args.first()
        else {
            return None;
        };
        if format.is_empty()
            || !format.chars().all(|code| matches!(code, 'V' | 'N' | 'n'))
            || args.len() != format.chars().count() + 1
        {
            return None;
        }
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(crate::backend_gap_prelude::PACK_INTEGER_NAME),
            args,
            expr,
        ));
    }
    if builtin == "http_build_query" && (1..=4).contains(&args.len()) {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::HTTP_BUILD_QUERY_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "file_put_contents"
        && (2..=4).contains(&args.len())
        && matches!(
            materialized_expr_type_for_merge(ctx, &args[1]).codegen_repr(),
            PhpType::Array(_) | PhpType::AssocArray { .. }
        )
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::FILE_PUT_CONTENTS_ARRAY_NAME,
            ),
            args,
            expr,
        ));
    }
    let gradual_sort_operand = || {
        matches!(
            materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_) | PhpType::AssocArray { .. }
        ) || matches!(
            materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr(),
            PhpType::Array(element)
                if matches!(
                    element.codegen_repr(),
                    PhpType::Mixed | PhpType::Union(_) | PhpType::Array(_) | PhpType::Object(_)
                )
        )
    };
    let gradual_sort_helper = match (builtin.as_str(), args.len()) {
        ("uasort", 2) if gradual_sort_operand() => {
            Some(crate::backend_gap_prelude::UASORT_MIXED_NAME)
        }
        ("usort", 2) if gradual_sort_operand() => {
            Some(crate::backend_gap_prelude::USORT_MIXED_NAME)
        }
        ("uksort", 2) if gradual_sort_operand() => {
            Some(crate::backend_gap_prelude::UKSORT_MIXED_NAME)
        }
        ("asort", 1) if gradual_sort_operand() => {
            Some(crate::backend_gap_prelude::ASORT_MIXED_NAME)
        }
        _ => None,
    };
    if let Some(helper) = gradual_sort_helper {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(helper),
            args,
            expr,
        ));
    }
    if builtin == "array_fill_keys"
        && args.len() == 2
        && matches!(
            materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_FILL_KEYS_MIXED_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "array_reduce"
        && args.len() == 3
        && matches!(
            materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::array_reduce_prelude::ARRAY_REDUCE_DEFAULT_NAME,
            ),
            args,
            expr,
        ));
    }
    let helper = match (builtin.as_str(), args.len()) {
        ("levenshtein", 2) => crate::backend_gap_prelude::LEVENSHTEIN_TWO_ARG_NAME,
        ("strip_tags", 1) => crate::backend_gap_prelude::STRIP_TAGS_ONE_ARG_NAME,
        ("is_countable", 1) => crate::backend_gap_prelude::IS_COUNTABLE_NAME,
        ("random_bytes", 1) => crate::backend_gap_prelude::RANDOM_BYTES_NAME,
        ("escapeshellarg", 1) => crate::backend_gap_prelude::ESCAPESHELLARG_NAME,
        ("str_getcsv", 1..=4) => crate::backend_gap_prelude::STR_GETCSV_NAME,
        ("cli_set_process_title" | "setproctitle", 1) => {
            crate::backend_gap_prelude::CLI_SET_PROCESS_TITLE_NAME
        }
        _ => return None,
    };
    Some(lower_function_call(
        ctx,
        &crate::names::Name::unqualified(helper),
        args,
        expr,
    ))
}
/// Routes `array_merge(...$arrays)` to the elephc-PHP prelude for gradual or keyed operands.
///
/// The six `__rt_array_merge*` helpers copy 8- or 16-byte slots out of INDEXED arrays; none
/// walks a hash. A boxed `mixed` operand therefore has no native path, and the gradual trick used
/// for `array_chunk` — rewriting through `array_values` — is not available here: `array_merge`
/// renumbers integer keys but PRESERVES string keys, so `array_values` would silently drop them.
/// `crate::array_merge_prelude` writes PHP's rule in PHP instead, which needs no new runtime helper.
///
/// Gradual values require a runtime shape check, while statically associative arrays require PHP's
/// string-key preservation and integer-key renumbering. Packed arrays keep the native slot-copy
/// lowering. Any other shape keeps its loud refusal rather than silently changing semantics.
pub(super) fn lower_gradual_array_merge(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "array_merge" {
        return None;
    }
    if crate::types::call_args::has_named_args(args) {
        return None;
    }
    let has_spread = args.iter().any(is_spread_arg);
    if !has_spread
        && !args
            .iter()
            .any(|arg| array_merge_operand_requires_prelude(ctx, arg))
    {
        return None;
    }
    if has_spread {
        if args.iter().filter(|arg| is_spread_arg(arg)).count() != 1 {
            return None;
        }
        let ExprKind::Spread(rest) = &args.last()?.kind else {
            return None;
        };
        let (helper, helper_args) = match args.len() {
            1 => (
                crate::array_merge_prelude::GRADUAL_ARRAY_MERGE_ONLY_SPREAD_NAME,
                vec![rest.as_ref().clone()],
            ),
            2 => (
                crate::array_merge_prelude::GRADUAL_ARRAY_MERGE_SPREAD_NAME,
                vec![args[0].clone(), rest.as_ref().clone()],
            ),
            _ => return None,
        };
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(helper),
            &helper_args,
            expr,
        ));
    }
    let helper = match args.len() {
        2 => crate::array_merge_prelude::GRADUAL_ARRAY_MERGE_NAME,
        3 => crate::array_merge_prelude::GRADUAL_ARRAY_MERGE_THREE_NAME,
        5 => crate::array_merge_prelude::GRADUAL_ARRAY_MERGE_FIVE_NAME,
        6 => crate::array_merge_prelude::GRADUAL_ARRAY_MERGE_SIX_NAME,
        _ => return None,
    };
    Some(lower_function_call(
        ctx,
        &crate::names::Name::unqualified(helper),
        args,
        expr,
    ))
}

/// Returns whether an `array_merge` argument needs key-aware PHP-level iteration.
fn array_merge_operand_requires_prelude(ctx: &LoweringContext<'_, '_>, expr: &Expr) -> bool {
    match materialized_expr_type_for_merge(ctx, expr).codegen_repr() {
        PhpType::Mixed | PhpType::Union(_) | PhpType::AssocArray { .. } => true,
        PhpType::Array(element) => element.codegen_repr() == PhpType::Mixed,
        _ => false,
    }
}
