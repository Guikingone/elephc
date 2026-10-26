//! Purpose:
//! Defines the normalized call-argument plan shared by checker and codegen.
//! Records source values, regular slots, variadic entries, spread bounds checks, and planner errors.
//!
//! Called from:
//! - `crate::types::call_args::planner`
//!
//! Key details:
//! - Plan data must retain enough span and source-index information to produce PHP-compatible diagnostics and evaluation order.

use crate::names::Name;
use crate::parser::ast::{BinOp, Expr, ExprKind};
use crate::span::Span;

/// Normalized call-argument plan used by both the type checker and codegen.
///
/// Retains source-order args, resolved regular/variadic slots, spread bounds checks,
/// and any planner errors. Enough span and source-index data is kept to produce
/// PHP-compatible diagnostics while preserving evaluation order.
pub(crate) struct CallArgPlan {
    pub(crate) source_args: Vec<Expr>,
    pub(crate) regular_args: Vec<PlannedRegularArg>,
    pub(crate) variadic_args: Vec<PlannedVariadicArg>,
    pub(crate) source_values: Vec<PlannedSourceValue>,
    pub(crate) spread_bounds_checks: Vec<SpreadBoundsCheck>,
    pub(crate) first_named_pos: Option<usize>,
    pub(crate) prefix_has_dynamic_named_spread: bool,
    pub(super) passthrough_args: Option<Vec<Expr>>,
    pub(super) regular_param_count: usize,
}

/// A resolved regular (non-variadic) parameter slot in the plan.
#[derive(Clone)]
pub(crate) enum PlannedRegularArg {
    Source {
        source_index: usize,
        expr: Expr,
    },
    SpreadElement {
        spread_expr: Expr,
        spread_span: Span,
        element_idx: usize,
        prefix_element_idx: usize,
        param_name: Option<String>,
        prefer_named_key: bool,
        default: Option<Expr>,
        guaranteed_present: bool,
    },
    Default(Expr),
}

/// A resolved variadic argument entry collected from positional or named call-site values.
#[derive(Clone)]
pub(crate) struct PlannedVariadicArg {
    pub(crate) source_index: usize,
    pub(crate) key: Option<String>,
    pub(crate) expr: Expr,
}

/// A call-site argument projected into the parameter space, tracking which
/// source argument it came from and which parameter slot it fills.
#[derive(Clone)]
pub(crate) enum PlannedSourceValue {
    Regular {
        source_index: usize,
        param_idx: usize,
        expr: Expr,
    },
    Variadic {
        source_index: usize,
        key: Option<String>,
        expr: Expr,
    },
}

/// Records the minimum and optional maximum number of elements a spread argument
/// must contain to satisfy the parameter slots it feeds. When the maximum is
/// imposed by a later named argument, keeps that parameter name for runtime
/// overwrite diagnostics.
#[derive(Clone)]
pub(crate) struct SpreadBoundsCheck {
    pub(crate) min_len: usize,
    pub(crate) max_len: Option<usize>,
    pub(crate) max_len_param_name: Option<String>,
}

/// Errors from `plan_call_args` when call-site arguments cannot be mapped to the signature.
#[derive(Debug)]
pub(crate) enum CallArgPlanError {
    UnknownNamed {
        span: Span,
        name: String,
    },
    Duplicate {
        span: Span,
        param_idx: usize,
        name: String,
    },
    PositionalAfterNamed {
        span: Span,
    },
    PositionalAfterSpread {
        span: Span,
    },
    SpreadAfterNamed {
        span: Span,
    },
    MissingRequired {
        span: Span,
        param_idx: usize,
    },
}

impl CallArgPlan {
    /// Marks normalized regular slots supplied by a declaration default, not by caller storage.
    pub(crate) fn default_argument_mask(&self) -> Vec<bool> {
        self.regular_args.iter()
            .map(|arg| matches!(arg, PlannedRegularArg::Default(_)))
            .collect()
    }

    /// Marks normalized values projected from a dynamic spread into regular parameter slots.
    ///
    /// The checker uses this provenance to permit runtime descriptor unboxing only for values
    /// that lowering will actually extract from a spread container. A direct `Mixed` argument
    /// must not gain the same exception merely because its normalized expression has that type.
    pub(crate) fn descriptor_projection_mask(&self) -> Vec<bool> {
        if let Some(args) = &self.passthrough_args {
            let first_spread = args
                .iter()
                .position(|arg| matches!(arg.kind, ExprKind::Spread(_)));
            return (0..self.regular_param_count)
                .map(|param_idx| first_spread.is_some_and(|spread_idx| param_idx >= spread_idx))
                .collect();
        }

        self.regular_args
            .iter()
            .map(|arg| matches!(arg, PlannedRegularArg::SpreadElement { .. }))
            .chain(self.variadic_args.iter().map(|_| false))
            .collect()
    }

    /// Returns `true` if any source argument used the spread (`...`) operator.
    pub(crate) fn has_spread_args(&self) -> bool {
        self.source_args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::Spread(_)))
    }

    /// Returns a flat list of expressions in parameter order suitable for codegen
    /// materialization. Preserves spread expressions for static-array spreads that
    /// were resolved in place; applies default guards for optional slots when the
    /// spread may not have provided those elements.
    pub(crate) fn normalized_args(&self) -> Vec<Expr> {
        if let Some(args) = &self.passthrough_args {
            return args.clone();
        }

        let mut args = Vec::new();
        for arg in &self.regular_args {
            args.push(match arg {
                PlannedRegularArg::Source { expr, .. } => expr.clone(),
                PlannedRegularArg::SpreadElement {
                    spread_expr,
                    spread_span,
                    element_idx,
                    param_name,
                    prefer_named_key,
                    default,
                    guaranteed_present,
                    ..
                } => {
                    if let Some(default) = default {
                        if *guaranteed_present {
                            spread_element_expr(
                                spread_expr,
                                *element_idx,
                                param_name.as_deref(),
                                *prefer_named_key,
                                *spread_span,
                            )
                        } else {
                            spread_element_or_default_expr(
                                spread_expr,
                                *element_idx,
                                param_name.as_deref(),
                                *prefer_named_key,
                                default.clone(),
                                *spread_span,
                            )
                        }
                    } else {
                        spread_element_expr(
                            spread_expr,
                            *element_idx,
                            param_name.as_deref(),
                            *prefer_named_key,
                            *spread_span,
                        )
                    }
                }
                PlannedRegularArg::Default(default) => default.clone(),
            });
        }

        for arg in &self.variadic_args {
            if let Some(key) = &arg.key {
                args.push(Expr::new(
                    ExprKind::NamedArg {
                        name: key.clone(),
                        value: Box::new(arg.expr.clone()),
                    },
                    arg.expr.span,
                ));
            } else {
                args.push(arg.expr.clone());
            }
        }
        args
    }

    /// Returns the leading positional-only prefix as either a bare spread expression
    /// (when there is exactly one prefix element that is a spread) or an array literal
    /// of all prefix elements. Used to reconstruct the leading positional arguments
    /// when forwarding calls with named parameters.
    pub(crate) fn positional_prefix_expr(&self, call_span: Span) -> Option<Expr> {
        let first_named_pos = self.first_named_pos?;
        let prefix_args = self.source_args[..first_named_pos].to_vec();
        let prefix_span = prefix_args.first().map(|arg| arg.span).unwrap_or(call_span);
        if let [arg] = prefix_args.as_slice() {
            if let ExprKind::Spread(inner) = &arg.kind {
                return Some((**inner).clone());
            }
        }
        Some(Expr::new(ExprKind::ArrayLiteral(prefix_args), prefix_span))
    }
}

impl PlannedSourceValue {
    /// The position of this value's source argument in the original call-site argument list.
    pub(crate) fn source_index(&self) -> usize {
        match self {
            PlannedSourceValue::Regular { source_index, .. }
            | PlannedSourceValue::Variadic { source_index, .. } => *source_index,
        }
    }

    /// The parameter index this value fills, or `None` for variadic entries.
    pub(crate) fn param_idx(&self) -> Option<usize> {
        match self {
            PlannedSourceValue::Regular { param_idx, .. } => Some(*param_idx),
            PlannedSourceValue::Variadic { .. } => None,
        }
    }

    /// The string key for variadic named entries, or `None` for positional variadic entries.
    pub(crate) fn key(&self) -> Option<&str> {
        match self {
            PlannedSourceValue::Regular { .. } => None,
            PlannedSourceValue::Variadic { key, .. } => key.as_deref(),
        }
    }

    /// The expression from the source argument for this planned value.
    pub(crate) fn expr(&self) -> &Expr {
        match self {
            PlannedSourceValue::Regular { expr, .. }
            | PlannedSourceValue::Variadic { expr, .. } => expr,
        }
    }
}

/// Emits an `ArrayAccess` expression to extract `spread_expr[element_idx]` using
/// a numeric index. When `prefer_named_key` is `true` and `param_name` is present,
/// uses the parameter name as the string key instead of the numeric index.
/// Used to materialize a single element from a spread array.
fn spread_element_expr(
    spread_expr: &Expr,
    element_idx: usize,
    param_name: Option<&str>,
    prefer_named_key: bool,
    span: Span,
) -> Expr {
    let positional = Expr::new(
        ExprKind::ArrayAccess {
            array: Box::new(spread_expr.clone()),
            index: Box::new(Expr::new(ExprKind::IntLiteral(element_idx as i64), span)),
        },
        span,
    );
    let Some(param_name) = param_name.filter(|_| prefer_named_key) else {
        return positional;
    };
    let key = Expr::new(ExprKind::StringLiteral(param_name.to_string()), span);
    if !spread_source_is_rereadable(spread_expr) {
        // The source is read exactly once: evaluating a call or a property chain four times
        // would repeat its side effects. Only a local is safe to probe by name AND position.
        return Expr::new(
            ExprKind::ArrayAccess {
                array: Box::new(spread_expr.clone()),
                index: Box::new(key),
            },
            span,
        );
    }
    // PHP binds an unpacked array per KEY: a string key names a parameter, an integer key fills
    // the next position. The name is asked first, then the position (`f(...[2], factor: 4)`
    // binds `$value` from key 0; asking only for `'value'` bound the default).
    Expr::new(
        ExprKind::Ternary {
            condition: Box::new(array_key_exists_expr(key.clone(), spread_expr, span)),
            then_expr: Box::new(Expr::new(
                ExprKind::ArrayAccess {
                    array: Box::new(spread_expr.clone()),
                    index: Box::new(key),
                },
                span,
            )),
            else_expr: Box::new(positional),
        },
        span,
    )
}

/// Whether the spread source can be read several times without repeating side effects.
fn spread_source_is_rereadable(spread_expr: &Expr) -> bool {
    matches!(spread_expr.kind, ExprKind::Variable(_))
}

/// Emits `array_key_exists($key, $spread_expr)`.
fn array_key_exists_expr(key: Expr, spread_expr: &Expr, span: Span) -> Expr {
    Expr::new(
        ExprKind::FunctionCall {
            name: Name::unqualified("array_key_exists"),
            args: vec![key, spread_expr.clone()],
        },
        span,
    )
}

/// Emits a ternary that selects `spread_expr[element_idx]` if the spread is
/// long enough, otherwise falls back to `default_expr`. The bounds check
/// uses `array_key_exists` when a named key is available, or `count() > element_idx`
/// for numeric keys. This handles optional spread elements that may or may not
/// be present at a given position.
fn spread_element_or_default_expr(
    spread_expr: &Expr,
    element_idx: usize,
    param_name: Option<&str>,
    prefer_named_key: bool,
    default_expr: Expr,
    span: Span,
) -> Expr {
    let Some(param_name) = param_name.filter(|_| prefer_named_key) else {
        return Expr::new(
            ExprKind::Ternary {
                condition: Box::new(spread_len_gt_expr(spread_expr, element_idx, span)),
                then_expr: Box::new(spread_element_expr(
                    spread_expr,
                    element_idx,
                    None,
                    false,
                    span,
                )),
                else_expr: Box::new(default_expr),
            },
            span,
        );
    };
    if !spread_source_is_rereadable(spread_expr) {
        // Name-only probe, so a source with side effects is evaluated as few times as before.
        let key = Expr::new(ExprKind::StringLiteral(param_name.to_string()), span);
        return Expr::new(
            ExprKind::Ternary {
                condition: Box::new(array_key_exists_expr(key, spread_expr, span)),
                then_expr: Box::new(spread_element_expr(
                    spread_expr,
                    element_idx,
                    Some(param_name),
                    true,
                    span,
                )),
                else_expr: Box::new(default_expr),
            },
            span,
        );
    }
    // The name first, then the position, then the default (see `spread_element_expr`). A source
    // that may carry names may also be sparse, so the position is asked by key, not by length.
    let index = Expr::new(ExprKind::IntLiteral(element_idx as i64), span);
    let positional = Expr::new(
        ExprKind::Ternary {
            condition: Box::new(array_key_exists_expr(index, spread_expr, span)),
            then_expr: Box::new(spread_element_expr(spread_expr, element_idx, None, false, span)),
            else_expr: Box::new(default_expr),
        },
        span,
    );
    let key = Expr::new(ExprKind::StringLiteral(param_name.to_string()), span);
    Expr::new(
        ExprKind::Ternary {
            condition: Box::new(array_key_exists_expr(key.clone(), spread_expr, span)),
            then_expr: Box::new(Expr::new(
                ExprKind::ArrayAccess {
                    array: Box::new(spread_expr.clone()),
                    index: Box::new(key),
                },
                span,
            )),
            else_expr: Box::new(positional),
        },
        span,
    )
}

/// Emits `count($spread_expr) > element_idx` as a `BinaryOp` expression.
fn spread_len_gt_expr(spread_expr: &Expr, element_idx: usize, span: Span) -> Expr {
    Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(spread_len_expr(spread_expr, span)),
            op: BinOp::Gt,
            right: Box::new(Expr::new(ExprKind::IntLiteral(element_idx as i64), span)),
        },
        span,
    )
}

/// Emits `count($spread_expr)` as a function call expression.
fn spread_len_expr(spread_expr: &Expr, span: Span) -> Expr {
    Expr::new(
        ExprKind::FunctionCall {
            name: Name::unqualified("count"),
            args: vec![spread_expr.clone()],
        },
        span,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FunctionSig, PhpType};

    /// Implements the `sig_with_defaults` operation for this module.
    fn sig_with_defaults(defaults: Vec<Option<Expr>>) -> FunctionSig {
        let params = ["a", "b", "c"]
            .iter()
            .map(|name| ((*name).to_string(), PhpType::Int))
            .collect();
        FunctionSig {
            params,
            param_type_exprs: vec![None; 3],
            param_attributes: Vec::new(),
            defaults,
            return_type: PhpType::Int,
            declared_return: true,
            by_ref_return: false,
            ref_params: vec![false; 3],
            declared_params: vec![true; 3],
            variadic: None,
            deprecation: None,
            is_generator: false,
        }
    }

    /// Implements the `spread_then_named_c` operation for this module.
    fn spread_then_named_c() -> Vec<Expr> {
        vec![
            Expr::new(ExprKind::Spread(Box::new(Expr::var("args"))), Span::dummy()),
            Expr::new(
                ExprKind::NamedArg {
                    name: "c".to_string(),
                    value: Box::new(Expr::int_lit(30)),
                },
                Span::dummy(),
            ),
        ]
    }

    /// Default provenance is positional metadata, even when caller literals equal declaration defaults.
    #[test]
    fn default_argument_mask_distinguishes_supplied_literals() {
        let sig = sig_with_defaults(vec![
            Some(Expr::int_lit(1)), Some(Expr::int_lit(2)), Some(Expr::int_lit(3)),
        ]);
        let named = |name: &str, value| Expr::new(
            ExprKind::NamedArg { name: name.to_string(), value: Box::new(Expr::int_lit(value)) },
            Span::dummy(),
        );
        let plan = super::super::planner::plan_call_args(
            &sig, &[named("b", 2)], Span::dummy(), false, true,
        ).expect("omitted defaults should be planned");
        assert_eq!(plan.default_argument_mask(), vec![true, false, true]);
        let explicit = super::super::planner::plan_call_args(
            &sig, &[named("b", 2), named("a", 1), named("c", 3)], Span::dummy(), false, true,
        ).expect("explicit equal literals should remain source arguments");
        assert_eq!(explicit.default_argument_mask(), vec![false, false, false]);
    }

    /// Implements the `normalized_args_skip_default_guard_when_spread_check_guarantees_slot`
    /// operation for this module.
    #[test]
    fn normalized_args_skip_default_guard_when_spread_check_guarantees_slot() {
        let sig = sig_with_defaults(vec![Some(Expr::int_lit(1)), None, None]);
        let plan = super::super::planner::plan_call_args(
            &sig,
            &spread_then_named_c(),
            Span::dummy(),
            false,
            true,
        )
        .expect("planner should accept spread before named argument");

        assert_eq!(plan.spread_bounds_checks[0].min_len, 2);
        let normalized = plan.normalized_args();
        // The slot is read by name, else by position, and never falls back to the default.
        match &normalized[0].kind {
            ExprKind::ArrayAccess { .. } => {}
            ExprKind::Ternary { then_expr, else_expr, .. } => {
                assert!(matches!(then_expr.kind, ExprKind::ArrayAccess { .. }));
                assert!(matches!(else_expr.kind, ExprKind::ArrayAccess { .. }));
            }
            other => panic!("unexpected guaranteed spread slot shape: {other:?}"),
        }
    }

    /// Implements the `normalized_args_keep_default_guard_when_spread_may_skip_optional_slot`
    /// operation for this module.
    #[test]
    fn normalized_args_keep_default_guard_when_spread_may_skip_optional_slot() {
        let sig = sig_with_defaults(vec![None, Some(Expr::int_lit(2)), None]);
        let plan = super::super::planner::plan_call_args(
            &sig,
            &spread_then_named_c(),
            Span::dummy(),
            false,
            true,
        )
        .expect("planner should accept spread before named argument");

        assert_eq!(plan.spread_bounds_checks[0].min_len, 1);
        let normalized = plan.normalized_args();
        assert!(matches!(&normalized[1].kind, ExprKind::Ternary { .. }));
    }

    /// Positional passthrough keeps descriptor provenance in parameter space.
    #[test]
    fn descriptor_projection_mask_marks_only_slots_fed_by_positional_spread() {
        let sig = sig_with_defaults(vec![None, None, None]);
        let spread = Expr::new(
            ExprKind::Spread(Box::new(Expr::var("args"))),
            Span::dummy(),
        );
        let plan = super::super::planner::plan_call_args(
            &sig,
            &[Expr::int_lit(1), spread],
            Span::dummy(),
            false,
            true,
        )
        .expect("positional spread should remain a passthrough source");

        assert_eq!(plan.descriptor_projection_mask(), vec![false, true, true]);
    }

    /// A direct unknown value never inherits descriptor-spread provenance.
    #[test]
    fn descriptor_projection_mask_does_not_mark_direct_values() {
        let sig = sig_with_defaults(vec![None, None, None]);
        let plan = super::super::planner::plan_call_args(
            &sig,
            &[Expr::var("first"), Expr::var("second")],
            Span::dummy(),
            false,
            true,
        )
        .expect("direct positional values should remain passthrough sources");

        assert_eq!(plan.descriptor_projection_mask(), vec![false, false, false]);
    }
}
