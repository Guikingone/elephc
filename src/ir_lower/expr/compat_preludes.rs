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

/// Routes `is_callable($value, $syntax_only[, &$name])` to the prelude, keeping the one-argument
/// form on the backend predicate.
///
/// The backend answers one question -- can this value be called -- and `$syntax_only` asks another
/// one, whether it merely has callable SHAPE. `symfony/http-kernel`'s `ServiceValueResolver` uses
/// the second form to decide whether `[$class, $method]` can be flattened to `Class::method`.
pub(super) fn lower_is_callable_with_options(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "is_callable"
        || !matches!(args.len(), 2 | 3)
        || crate::types::call_args::has_named_args(args)
        || args.iter().any(is_spread_arg)
    {
        return None;
    }
    Some(lower_function_call(
        ctx,
        &crate::names::Name::unqualified(crate::is_callable_prelude::IS_CALLABLE_EXT_NAME),
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
    if crate::types::call_args::has_named_args(args)
        || (builtin != "array_unshift" && args.iter().any(is_spread_arg))
    {
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
    if builtin == "array_combine" && args.len() == 2 {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_COMBINE_GRADUAL_NAME,
            ),
            args,
            expr,
        ));
    }
    // Only the two-argument form: `preserve_keys` changes the result SHAPE and a gradual
    // source's keys are not known statically, so the three-argument form stays on its own path.
    //
    // `array<mixed>` is included deliberately. It is what a declared `array` parameter and an
    // untyped property both get, and it does NOT promise indexed storage -- the same reason
    // `lower_array_union` picks its gradual helper for that pair. Leaving it native ran the dense
    // indexed walk over hash storage and produced a short, malformed result. Only a NARROWER
    // element type guarantees the dense representation the native path needs.
    if builtin == "array_chunk"
        && (2..=3).contains(&args.len())
        && match materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr() {
            // A DENSE source keeps its native path in the three-argument form: both arms are
            // lowerable there, and redirecting it to the prelude handed the call site a result
            // shaped like the helper's rather than its own, which printed empty chunks.
            PhpType::Array(element) => args.len() == 2 && element.codegen_repr() == PhpType::Mixed,
            PhpType::Mixed | PhpType::Union(_) | PhpType::AssocArray { .. } => true,
            _ => false,
        }
    {
        // The three-argument form takes its own helper: its chunks carry the source keys and are
        // therefore hashes, where the two-argument helper's are dense arrays.
        let helper = if args.len() == 3 {
            crate::backend_gap_prelude::ARRAY_CHUNK_GRADUAL_FLAGGED_NAME
        } else {
            crate::backend_gap_prelude::ARRAY_CHUNK_GRADUAL_NAME
        };
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(helper),
            args,
            expr,
        ));
    }
    // A callback the lowering cannot recover a native pointer for -- a boxed callable in a local,
    // a parameter, a property -- has no static binding for the array runtime, so the whole call
    // goes to the PHP helper, which calls it the way php does. `twig/twig`'s
    // `CoreExtension::filter($env, $isSandboxed, $array, $arrow)` forwards an untyped parameter.
    if builtin == "array_filter"
        && (2..=3).contains(&args.len())
        && matches!(
            materialized_expr_type_for_merge(ctx, &args[1]).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
    {
        let mut helper_args = vec![args[0].clone(), args[1].clone()];
        helper_args.push(match args.get(2) {
            Some(mode) => mode.clone(),
            // php's default mode: the value is the callback's only argument.
            None => Expr::new(ExprKind::IntLiteral(0), expr.span),
        });
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::array_filter_prelude::ARRAY_FILTER_CALLBACK_NAME,
            ),
            &helper_args,
            expr,
        ));
    }
    // `preg_match_all()` with `$flags` goes to the prelude helper: the native capture helper
    // builds pattern order and ignores the flags. `src/preg_match_all_prelude.rs` has the
    // measurement. A NAMED argument keeps the native path, whose binder places it.
    if builtin == "preg_match_all"
        && (4..=5).contains(&args.len())
        && !args.iter().any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. }))
    {
        let mut helper_args = args[..4].to_vec();
        helper_args.push(match args.get(4) {
            Some(offset) => offset.clone(),
            None => Expr::new(ExprKind::IntLiteral(0), expr.span),
        });
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::preg_match_all_prelude::PREG_MATCH_ALL_FLAGS_NAME,
            ),
            &helper_args,
            expr,
        ));
    }
    // `array_walk()` and `array_walk_recursive()` go to the PHP helpers WHOLESALE, not just for
    // the shapes the native runtimes refuse. The refusals and the silent divergences come from the
    // same place -- a native callback pointer, a fixed argument list the callback cannot write
    // through, and an element read as a raw integer -- and the helpers are defined in terms the
    // compiler already lowers correctly. `src/array_walk_prelude.rs` carries the measurements.
    if (builtin == "array_walk" || builtin == "array_walk_recursive")
        && (2..=3).contains(&args.len())
    {
        let has_arg = args.len() == 3;
        let mut helper_args = vec![args[0].clone(), args[1].clone()];
        helper_args.push(Expr::new(ExprKind::BoolLiteral(has_arg), expr.span));
        helper_args.push(match args.get(2) {
            Some(arg) => arg.clone(),
            None => Expr::new(ExprKind::Null, expr.span),
        });
        let helper = if builtin == "array_walk" {
            crate::array_walk_prelude::ARRAY_WALK_NAME
        } else {
            crate::array_walk_prelude::ARRAY_WALK_RECURSIVE_NAME
        };
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(helper),
            &helper_args,
            expr,
        ));
    }
    // A gradual or hash `array_rand()`: the native helper only walks a dense indexed array, and a
    // hash's key is as likely to be a string. The prelude helper collects the keys first.
    if builtin == "array_rand"
        && (1..=2).contains(&args.len())
        && !matches!(
            materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr(),
            PhpType::Array(_)
        )
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_RAND_GRADUAL_NAME,
            ),
            &args[..1],
            expr,
        ));
    }
    // The three-argument `array_column()` re-keys its result from the data, which is a hash the
    // backend has no lowering for. The prelude helper answers it in PHP.
    if builtin == "array_column" && args.len() == 3 {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_COLUMN_INDEXED_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "array_reverse" && args.len() == 1 {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::ARRAY_REVERSE_GRADUAL_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "array_search"
        && (2..=3).contains(&args.len())
        && !matches!(
            materialized_expr_type_for_merge(ctx, &args[1]).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
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
    if builtin == "preg_replace"
        && args.len() == 3
        && matches!(
            materialized_expr_type_for_merge(ctx, &args[0]).codegen_repr(),
            PhpType::Array(_) | PhpType::AssocArray { .. }
        )
    {
        return Some(lower_function_call(
            ctx,
            &crate::names::Name::unqualified(
                crate::backend_gap_prelude::PREG_REPLACE_ARRAY_NAME,
            ),
            args,
            expr,
        ));
    }
    if builtin == "array_splice" && !args.iter().any(is_spread_arg) {
        let replacement = args.iter().enumerate().find_map(|(index, arg)| match &arg.kind {
            ExprKind::NamedArg { name, value } if php_symbol_key(name) == "replacement" => {
                Some(value.as_ref())
            }
            ExprKind::NamedArg { .. } => None,
            _ if index == 3 => Some(arg),
            _ => None,
        });
        if replacement.is_some_and(|replacement| {
            matches!(
                materialized_expr_type_for_merge(ctx, replacement).codegen_repr(),
                PhpType::Mixed | PhpType::Union(_)
            )
        }) {
            return Some(lower_function_call(
                ctx,
                &crate::names::Name::unqualified(
                    crate::backend_gap_prelude::ARRAY_SPLICE_MIXED_REPLACEMENT_NAME,
                ),
                args,
                expr,
            ));
        }
    }
    if builtin == "array_unshift" {
        let receiver = args.iter().enumerate().find_map(|(index, arg)| match &arg.kind {
            ExprKind::NamedArg { name, value } if php_symbol_key(name) == "array" => {
                Some(value.as_ref())
            }
            ExprKind::NamedArg { .. } => None,
            _ if index == 0 => Some(arg),
            _ => None,
        });
        if let (Some(receiver), Some(last)) = (receiver, args.last()) {
            if let ExprKind::Spread(spread) = &last.kind {
                if matches!(receiver.kind, ExprKind::Variable(_))
                    && args[..args.len() - 1].iter().all(|arg| !is_spread_arg(arg))
                {
                    let leading = Expr::new(
                        ExprKind::ArrayLiteral(args[1..args.len() - 1].to_vec()),
                        expr.span,
                    );
                    let helper_args = vec![receiver.clone(), leading, spread.as_ref().clone()];
                    return Some(lower_function_call(
                        ctx,
                        &crate::names::Name::unqualified(
                            crate::backend_gap_prelude::ARRAY_UNSHIFT_TRAILING_SPREAD_NAME,
                        ),
                        &helper_args,
                        expr,
                    ));
                }
            }
        }
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
/// Folds a variadic `array_replace()` into the nested two-argument form it is defined as.
///
/// php's `array_replace($a, $b, $c)` is `array_replace(array_replace($a, $b), $c)`: each array
/// overwrites the keys of the ones before it, left to right. Every lowering below takes exactly
/// two hash operands, so the fold reuses them all unchanged and needs no new runtime helper and
/// no new prelude -- it is a pure AST rewrite of the call site.
///
/// Symfony's `UrlGenerator` writes the three-argument form
/// (`array_replace($defaults, $this->context->getParameters(), $parameters)`), and refusing it
/// with `array_replace() takes exactly 2 arguments` kept the whole routing component out of the
/// closed world -- which is what made the runtime autoloader re-interpret `UrlMatcher` and
/// `CompiledUrlMatcher` on every request.
pub(super) fn lower_variadic_array_replace(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let key = php_symbol_key(name.trim_start_matches('\\'));
    if key != "array_replace" && key != "array_replace_recursive" {
        return None;
    }
    if args.len() <= 2 || crate::types::call_args::has_named_args(args) {
        return None;
    }
    if args.iter().any(is_spread_arg) {
        return None;
    }
    let callee = crate::names::Name::unqualified(name.trim_start_matches('\\'));
    let mut folded = args[0].clone();
    for next in &args[1..] {
        folded = Expr::new(
            ExprKind::FunctionCall {
                name: callee.clone(),
                args: vec![folded, next.clone()],
            },
            expr.span,
        );
    }
    Some(lower_expr(ctx, &folded))
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
