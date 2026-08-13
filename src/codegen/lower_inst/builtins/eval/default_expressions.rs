//! Purpose:
//! Converts representable PHP default expressions into eval metadata.
//!
//! Called from:
//! - The eval lowering facade and sibling eval support modules.
//!
//! Key details:
//! - Constant evaluation stays bounded and PHP-equivalent for supported values.

use super::*;

/// Converts a PHP signature default into the compact eval bridge default ABI.
pub(super) fn eval_native_callable_default(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
) -> Option<EvalNativeCallableDefault> {
    eval_native_callable_default_at(expr, default_context, 0)
}

/// Converts a PHP default expression while preserving a recursion limit for constants.
pub(super) fn eval_native_callable_default_at(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<EvalNativeCallableDefault> {
    if depth > MAX_NATIVE_DEFAULT_CONSTANT_DEPTH {
        return None;
    }
    eval_native_literal_default(expr)
        .or_else(|| eval_native_object_default(expr, default_context, depth))
        .or_else(|| eval_native_array_default(expr, default_context, depth))
        .or_else(|| eval_native_constant_expression_default(expr, default_context, depth))
}

/// Converts representable pure constant expressions into native eval defaults.
pub(super) fn eval_native_constant_expression_default(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<EvalNativeCallableDefault> {
    match &expr.kind {
        ExprKind::ConstRef(name) => {
            eval_native_global_constant_default(default_context, name, depth + 1)
        }
        ExprKind::ClassConstant { receiver } => {
            eval_native_static_receiver_name(default_context, receiver)
                .map(EvalNativeCallableDefault::String)
        }
        ExprKind::ScopedConstantAccess { receiver, name } => {
            eval_native_scoped_constant_default(default_context, receiver, name, depth + 1)
        }
        ExprKind::BinaryOp { left, op, right } => {
            eval_native_binary_expression_default(left, op, right, default_context, depth + 1)
        }
        ExprKind::Not(inner) => eval_native_default_truthy(&eval_native_callable_default_at(
            inner,
            default_context,
            depth + 1,
        )?)
        .map(|value| eval_native_bool_default(!value)),
        ExprKind::BitNot(inner) => eval_native_default_int(inner, default_context, depth + 1)
            .map(|value| eval_native_int_default(!value)),
        ExprKind::NullCoalesce { value, default } => {
            let value = eval_native_callable_default_at(value, default_context, depth + 1)?;
            if eval_native_default_is_null(&value) {
                eval_native_callable_default_at(default, default_context, depth + 1)
            } else {
                Some(value)
            }
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            if eval_native_default_truthy(&eval_native_callable_default_at(
                condition,
                default_context,
                depth + 1,
            )?)? {
                eval_native_callable_default_at(then_expr, default_context, depth + 1)
            } else {
                eval_native_callable_default_at(else_expr, default_context, depth + 1)
            }
        }
        ExprKind::ShortTernary { value, default } => {
            let value = eval_native_callable_default_at(value, default_context, depth + 1)?;
            if eval_native_default_truthy(&value)? {
                Some(value)
            } else {
                eval_native_callable_default_at(default, default_context, depth + 1)
            }
        }
        _ => None,
    }
}

/// Converts one supported binary constant expression into a native eval default.
pub(super) fn eval_native_binary_expression_default(
    left: &Expr,
    op: &BinOp,
    right: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<EvalNativeCallableDefault> {
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow => {
            eval_native_numeric_binary_default(left, op, right, default_context, depth + 1)
        }
        BinOp::Mod => {
            let left = eval_native_default_int(left, default_context, depth + 1)?;
            let right = eval_native_default_int(right, default_context, depth + 1)?;
            (right != 0).then(|| eval_native_int_default(left % right))
        }
        BinOp::Concat => {
            let left = eval_native_default_string(left, default_context, depth + 1)?;
            let right = eval_native_default_string(right, default_context, depth + 1)?;
            Some(EvalNativeCallableDefault::String(format!("{left}{right}")))
        }
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
            let left = eval_native_default_int(left, default_context, depth + 1)?;
            let right = eval_native_default_int(right, default_context, depth + 1)?;
            let value = match op {
                BinOp::BitAnd => left & right,
                BinOp::BitOr => left | right,
                BinOp::BitXor => left ^ right,
                _ => unreachable!("bitwise default operator was prefiltered"),
            };
            Some(eval_native_int_default(value))
        }
        BinOp::ShiftLeft | BinOp::ShiftRight => {
            let left = eval_native_default_int(left, default_context, depth + 1)?;
            let right =
                u32::try_from(eval_native_default_int(right, default_context, depth + 1)?).ok()?;
            let value = match op {
                BinOp::ShiftLeft => left.checked_shl(right),
                BinOp::ShiftRight => left.checked_shr(right),
                _ => unreachable!("shift default operator was prefiltered"),
            }?;
            Some(eval_native_int_default(value))
        }
        BinOp::And | BinOp::Or | BinOp::Xor => {
            let left = eval_native_default_truthy(&eval_native_callable_default_at(
                left,
                default_context,
                depth + 1,
            )?)?;
            let right = eval_native_default_truthy(&eval_native_callable_default_at(
                right,
                default_context,
                depth + 1,
            )?)?;
            let value = match op {
                BinOp::And => left && right,
                BinOp::Or => left || right,
                BinOp::Xor => left ^ right,
                _ => unreachable!("logical default operator was prefiltered"),
            };
            Some(eval_native_bool_default(value))
        }
        BinOp::NullCoalesce => {
            let left = eval_native_callable_default_at(left, default_context, depth + 1)?;
            if eval_native_default_is_null(&left) {
                eval_native_callable_default_at(right, default_context, depth + 1)
            } else {
                Some(left)
            }
        }
        _ => None,
    }
}

/// Converts one supported arithmetic expression into a native eval default.
pub(super) fn eval_native_numeric_binary_default(
    left: &Expr,
    op: &BinOp,
    right: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<EvalNativeCallableDefault> {
    if let (Some(left), Some(right)) = (
        eval_native_default_int(left, default_context, depth + 1),
        eval_native_default_int(right, default_context, depth + 1),
    ) {
        return match op {
            BinOp::Add => left.checked_add(right).map(eval_native_int_default),
            BinOp::Sub => left.checked_sub(right).map(eval_native_int_default),
            BinOp::Mul => left.checked_mul(right).map(eval_native_int_default),
            BinOp::Div if right != 0 => Some(eval_native_float_default(left as f64 / right as f64)),
            BinOp::Pow => {
                let value = (left as f64).powf(right as f64);
                value.is_finite().then(|| eval_native_float_default(value))
            }
            _ => None,
        };
    }

    let left = eval_native_default_numeric(left, default_context, depth + 1)?;
    let right = eval_native_default_numeric(right, default_context, depth + 1)?;
    let value = match op {
        BinOp::Add => left + right,
        BinOp::Sub => left - right,
        BinOp::Mul => left * right,
        BinOp::Div if right != 0.0 => left / right,
        BinOp::Pow => left.powf(right),
        _ => return None,
    };
    value.is_finite().then(|| eval_native_float_default(value))
}

/// Builds one bool default metadata value.
pub(super) fn eval_native_bool_default(value: bool) -> EvalNativeCallableDefault {
    EvalNativeCallableDefault::Scalar {
        kind: NATIVE_DEFAULT_BOOL,
        payload: i64::from(value),
    }
}

/// Builds one int default metadata value.
pub(super) fn eval_native_int_default(value: i64) -> EvalNativeCallableDefault {
    EvalNativeCallableDefault::Scalar {
        kind: NATIVE_DEFAULT_INT,
        payload: value,
    }
}

/// Builds one float default metadata value.
pub(super) fn eval_native_float_default(value: f64) -> EvalNativeCallableDefault {
    EvalNativeCallableDefault::Scalar {
        kind: NATIVE_DEFAULT_FLOAT,
        payload: value.to_bits() as i64,
    }
}

/// Returns true when one default metadata value is PHP `null`.
pub(super) fn eval_native_default_is_null(default: &EvalNativeCallableDefault) -> bool {
    matches!(
        default,
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_NULL,
            ..
        }
    )
}

/// Returns PHP truthiness for one representable native eval default.
pub(super) fn eval_native_default_truthy(default: &EvalNativeCallableDefault) -> Option<bool> {
    match default {
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_NULL,
            ..
        } => Some(false),
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_BOOL,
            payload,
        } => Some(*payload != 0),
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_INT,
            payload,
        } => Some(*payload != 0),
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_FLOAT,
            payload,
        } => Some(f64::from_bits(*payload as u64) != 0.0),
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_EMPTY_ARRAY,
            ..
        } => Some(false),
        EvalNativeCallableDefault::String(value) => Some(!value.is_empty() && value != "0"),
        EvalNativeCallableDefault::Array(_) | EvalNativeCallableDefault::Object { .. } => None,
        EvalNativeCallableDefault::Scalar { .. } => None,
    }
}

/// Extracts an int value from one representable default expression.
pub(super) fn eval_native_default_int(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<i64> {
    match eval_native_callable_default_at(expr, default_context, depth)? {
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_INT,
            payload,
        } => Some(payload),
        _ => None,
    }
}

/// Extracts a numeric value from one representable default expression.
pub(super) fn eval_native_default_numeric(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<f64> {
    match eval_native_callable_default_at(expr, default_context, depth)? {
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_INT,
            payload,
        } => Some(payload as f64),
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_FLOAT,
            payload,
        } => Some(f64::from_bits(payload as u64)),
        _ => None,
    }
}

/// Extracts a string value from one representable default expression.
pub(super) fn eval_native_default_string(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<String> {
    match eval_native_callable_default_at(expr, default_context, depth)? {
        EvalNativeCallableDefault::String(value) => Some(value),
        _ => None,
    }
}

/// Converts scalar/string/empty-array defaults into the compact eval bridge default ABI.
pub(super) fn eval_native_literal_default(expr: &Expr) -> Option<EvalNativeCallableDefault> {
    match &expr.kind {
        ExprKind::Null => Some(EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_NULL,
            payload: 0,
        }),
        ExprKind::BoolLiteral(value) => Some(EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_BOOL,
            payload: i64::from(*value),
        }),
        ExprKind::IntLiteral(value) => Some(EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_INT,
            payload: *value,
        }),
        ExprKind::FloatLiteral(value) => Some(EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_FLOAT,
            payload: value.to_bits() as i64,
        }),
        ExprKind::StringLiteral(value) => Some(EvalNativeCallableDefault::String(value.clone())),
        ExprKind::ArrayLiteral(elements) if elements.is_empty() => {
            Some(EvalNativeCallableDefault::Scalar {
                kind: NATIVE_DEFAULT_EMPTY_ARRAY,
                payload: 0,
            })
        }
        ExprKind::Negate(inner) => eval_native_callable_negated_default(inner),
        _ => None,
    }
}

/// Converts supported object-valued defaults into compact eval bridge metadata.
pub(super) fn eval_native_object_default(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<EvalNativeCallableDefault> {
    let ExprKind::NewObject { class_name, args } = &expr.kind else {
        return None;
    };
    if args.len() > MAX_NATIVE_OBJECT_DEFAULT_ARGS {
        return None;
    }
    let mut default_args = Vec::with_capacity(args.len());
    for arg in args {
        default_args.push(eval_native_object_default_arg(
            arg,
            default_context,
            depth + 1,
        )?);
    }
    Some(EvalNativeCallableDefault::Object {
        class_name: class_name.as_canonical(),
        args: default_args,
    })
}

/// Converts one object-valued default constructor argument into bridge metadata.
pub(super) fn eval_native_object_default_arg(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<EvalNativeCallableObjectDefaultArg> {
    match &expr.kind {
        ExprKind::NamedArg { name, value } => Some(EvalNativeCallableObjectDefaultArg {
            name: Some(name.clone()),
            default: eval_native_callable_default_at(value, default_context, depth + 1)?,
        }),
        ExprKind::Spread(_) => None,
        _ => Some(EvalNativeCallableObjectDefaultArg {
            name: None,
            default: eval_native_callable_default_at(expr, default_context, depth + 1)?,
        }),
    }
}

/// Converts supported array-valued defaults into compact eval bridge metadata.
pub(super) fn eval_native_array_default(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<EvalNativeCallableDefault> {
    match &expr.kind {
        ExprKind::ArrayLiteral(elements) => {
            let mut default_elements = Vec::with_capacity(elements.len());
            for element in elements {
                if matches!(element.kind, ExprKind::Spread(_)) {
                    return None;
                }
                default_elements.push(EvalNativeCallableArrayDefaultElement {
                    key: None,
                    default: eval_native_callable_default_at(element, default_context, depth + 1)?,
                });
            }
            Some(EvalNativeCallableDefault::Array(default_elements))
        }
        ExprKind::ArrayLiteralAssoc(elements) => {
            let mut default_elements = Vec::with_capacity(elements.len());
            for (key, value) in elements {
                default_elements.push(EvalNativeCallableArrayDefaultElement {
                    key: Some(eval_native_array_default_key(
                        key,
                        default_context,
                        depth + 1,
                    )?),
                    default: eval_native_callable_default_at(value, default_context, depth + 1)?,
                });
            }
            Some(EvalNativeCallableDefault::Array(default_elements))
        }
        _ => None,
    }
}

/// Converts one supported static array key into bridge metadata.
pub(super) fn eval_native_array_default_key(
    expr: &Expr,
    default_context: &EvalNativeDefaultContext<'_>,
    depth: usize,
) -> Option<EvalNativeCallableArrayDefaultKey> {
    if let Some(key) = eval_native_literal_array_default_key(expr) {
        return Some(key);
    }
    match eval_native_callable_default_at(expr, default_context, depth + 1)? {
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_NULL,
            ..
        } => Some(EvalNativeCallableArrayDefaultKey::String(String::new())),
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_BOOL,
            payload,
        } => Some(EvalNativeCallableArrayDefaultKey::Int(
            (payload != 0) as i64,
        )),
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_INT,
            payload,
        } => Some(EvalNativeCallableArrayDefaultKey::Int(payload)),
        EvalNativeCallableDefault::Scalar {
            kind: NATIVE_DEFAULT_FLOAT,
            payload,
        } => Some(EvalNativeCallableArrayDefaultKey::Int(
            f64::from_bits(payload as u64) as i64,
        )),
        EvalNativeCallableDefault::String(value) => eval_native_string_array_default_key(&value),
        _ => None,
    }
}
