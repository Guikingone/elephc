//! Purpose:
//! Builds EvalIR statements and attribute values from parsed property-like syntax.
//!
//! Called from:
//! - Assignment, member, attribute, and property-hook parsing.
//!
//! Key details:
//! - Reference, inc/dec, array mutation, literals, and class-name attribute args are normalized here.

use super::*;
use crate::parser::expressions::precedence::is_assignment_target;

/// Builds a property by-reference binding statement from a parsed property target.
pub(super) fn property_reference_bind_stmt(
    target: EvalExpr,
    source: String,
) -> Result<EvalStmt, EvalParseError> {
    match target {
        EvalExpr::PropertyGet { object, property } => Ok(EvalStmt::PropertyReferenceBind {
            object: *object,
            property,
            source,
        }),
        EvalExpr::DynamicPropertyGet { object, property } => {
            Ok(EvalStmt::DynamicPropertyReferenceBind {
                object: *object,
                property: *property,
                source,
            })
        }
        EvalExpr::DynamicStaticPropertyGet {
            class_name,
            property,
        } => Ok(EvalStmt::DynamicStaticPropertyReferenceBind {
            class_name: *class_name,
            property,
            source,
        }),
        EvalExpr::DynamicStaticPropertyNameGet {
            class_name,
            property,
        } => Ok(EvalStmt::DynamicStaticPropertyNameReferenceBind {
            class_name: *class_name,
            property: *property,
            source,
        }),
        // An array element of any writable chain, `$this->data[$key] = &$array;` above all
        // (`symfony/http-foundation/Session/SessionBagProxy.php:72`). The statement that writes
        // through a whole element path already exists and takes any target expression; only this
        // builder refused to reach it, because it enumerated property shapes alone.
        target if is_assignment_target(&target) => Ok(EvalStmt::ArrayReferenceBind {
            target,
            source: EvalExpr::LoadVar(source),
        }),
        _ => Err(EvalParseError::UnexpectedToken),
    }
}

/// Builds an object-property increment/decrement statement from a parsed property target.
pub(super) fn property_inc_dec_stmt(target: EvalExpr, increment: bool) -> Result<EvalStmt, EvalParseError> {
    match target {
        EvalExpr::PropertyGet { object, property } => Ok(EvalStmt::PropertyIncDec {
            object: *object,
            property,
            increment,
        }),
        EvalExpr::DynamicPropertyGet { object, property } => Ok(EvalStmt::DynamicPropertyIncDec {
            object: *object,
            property: *property,
            increment,
        }),
        EvalExpr::DynamicStaticPropertyGet {
            class_name,
            property,
        } => Ok(EvalStmt::DynamicStaticPropertyIncDec {
            class_name: *class_name,
            property,
            increment,
        }),
        EvalExpr::DynamicStaticPropertyNameGet {
            class_name,
            property,
        } => Ok(EvalStmt::DynamicStaticPropertyNameIncDec {
            class_name: *class_name,
            property: *property,
            increment,
        }),
        // Any other writable chain — `++$sanitizedLogs[$errorId]['errorCount']` above all. As a
        // STATEMENT the prefix and postfix forms have the same effect, and `PostfixIncDec` already
        // routes through the general lvalue machinery, so the deeper path needs no statement of
        // its own.
        target if is_assignment_target(&target) => Ok(EvalStmt::Expr(EvalExpr::PostfixIncDec {
            target: Box::new(target),
            increment,
        })),
        _ => Err(EvalParseError::UnexpectedToken),
    }
}

/// Builds an object-property array append statement from a parsed property target.
pub(super) fn property_array_append_stmt(target: EvalExpr, value: EvalExpr) -> Result<EvalStmt, EvalParseError> {
    match target {
        EvalExpr::PropertyGet { object, property } => Ok(EvalStmt::PropertyArrayAppend {
            object: *object,
            property,
            value,
        }),
        EvalExpr::DynamicPropertyGet { object, property } => {
            Ok(EvalStmt::DynamicPropertyArrayAppend {
                object: *object,
                property: *property,
                value,
            })
        }
        EvalExpr::DynamicStaticPropertyGet {
            class_name,
            property,
        } => Ok(EvalStmt::DynamicStaticPropertyArrayAppend {
            class_name: *class_name,
            property,
            value,
        }),
        EvalExpr::DynamicStaticPropertyNameGet {
            class_name,
            property,
        } => Ok(EvalStmt::DynamicStaticPropertyNameArrayAppend {
            class_name: *class_name,
            property: *property,
            value,
        }),
        // Any other writable chain — `$this->listeners[$name][$priority][] = $listener;` above
        // all. `interpreter::eval_array_append` already appends through ANY writable lvalue; only
        // this builder refused to hand it one, because it enumerated property shapes alone. That
        // refusal cost twenty-one Symfony files, and it surfaced at the NEXT statement's first
        // token, which is why they were reported as `unexpected "}"` and `unexpected "return"`.
        target if is_assignment_target(&target) => Ok(EvalStmt::ArrayAppend { target, value }),
        _ => Err(EvalParseError::UnexpectedToken),
    }
}

/// Builds an object-property array write statement from a parsed property target.
pub(super) fn property_array_set_stmt(
    target: EvalExpr,
    index: EvalExpr,
    op: Option<EvalBinOp>,
    value: EvalExpr,
) -> Result<EvalStmt, EvalParseError> {
    match target {
        EvalExpr::PropertyGet { object, property } => Ok(EvalStmt::PropertyArraySet {
            object: *object,
            property,
            index,
            op,
            value,
        }),
        EvalExpr::DynamicPropertyGet { object, property } => Ok(EvalStmt::DynamicPropertyArraySet {
            object: *object,
            property: *property,
            index,
            op,
            value,
        }),
        EvalExpr::DynamicStaticPropertyGet {
            class_name,
            property,
        } => Ok(EvalStmt::DynamicStaticPropertyArraySet {
            class_name: *class_name,
            property,
            index,
            op,
            value,
        }),
        EvalExpr::DynamicStaticPropertyNameGet {
            class_name,
            property,
        } => Ok(EvalStmt::DynamicStaticPropertyNameArraySet {
            class_name: *class_name,
            property: *property,
            index,
            op,
            value,
        }),
        // Any other writable chain — `$statistics[$name]['time'] += …`. The nested element is an
        // ordinary assignment target once it is spelled as one, and `EvalExpr::Assign` and
        // `CompoundAssign` both write through the general location machinery.
        target if is_assignment_target(&target) => {
            let target = EvalExpr::ArrayGet {
                array: Box::new(target),
                index: Box::new(index),
            };
            Ok(EvalStmt::Expr(match op {
                Some(op) => EvalExpr::CompoundAssign {
                    target: Box::new(target),
                    op,
                    value: Box::new(value),
                },
                None => EvalExpr::Assign {
                    target: Box::new(target),
                    value: Box::new(value),
                },
            }))
        }
        _ => Err(EvalParseError::UnexpectedToken),
    }
}

/// Converts a parsed attribute argument expression into retained literal metadata.
pub(super) fn eval_attribute_arg_from_expr(expr: &EvalExpr) -> Option<EvalAttributeArg> {
    match expr {
        EvalExpr::Const(EvalConst::String(value)) => Some(EvalAttributeArg::String(value.clone())),
        EvalExpr::Const(EvalConst::Int(value)) => Some(EvalAttributeArg::Int(*value)),
        EvalExpr::Const(EvalConst::Float(value)) => Some(EvalAttributeArg::Float(value.to_bits())),
        EvalExpr::Const(EvalConst::Bool(value)) => Some(EvalAttributeArg::Bool(*value)),
        EvalExpr::Const(EvalConst::Null) => Some(EvalAttributeArg::Null),
        EvalExpr::Unary {
            op: EvalUnaryOp::Negate,
            expr,
        } => match expr.as_ref() {
            EvalExpr::Const(EvalConst::Int(value)) => {
                Some(EvalAttributeArg::Int(value.wrapping_neg()))
            }
            EvalExpr::Const(EvalConst::Float(value)) => {
                Some(EvalAttributeArg::Float((-*value).to_bits()))
            }
            _ => None,
        },
        EvalExpr::ClassNameFetch { class_name } => {
            eval_attribute_class_name_arg(class_name).map(EvalAttributeArg::String)
        }
        EvalExpr::Array(elements) => eval_attribute_array_arg_from_elements(elements),
        _ => None,
    }
}

/// Converts an eval array literal into retained attribute metadata.
pub(super) fn eval_attribute_array_arg_from_elements(
    elements: &[EvalArrayElement],
) -> Option<EvalAttributeArg> {
    elements
        .iter()
        .map(|element| match element {
            EvalArrayElement::Value(value) => eval_attribute_arg_from_expr(value),
            EvalArrayElement::Reference(_) => None,
            EvalArrayElement::KeyValue { key, value } => {
                let value = eval_attribute_arg_from_expr(value)?;
                eval_attribute_array_keyed_arg(key, value)
            }
            // An attribute argument is retained metadata, and a spread is not a literal.
            EvalArrayElement::KeyReference { .. } | EvalArrayElement::Spread(_) => None,
        })
        .collect::<Option<Vec<_>>>()
        .map(EvalAttributeArg::Array)
}

/// Wraps an attribute array value with the PHP-normalized literal key metadata.
pub(super) fn eval_attribute_array_keyed_arg(
    key: &EvalExpr,
    value: EvalAttributeArg,
) -> Option<EvalAttributeArg> {
    match key {
        EvalExpr::Const(EvalConst::String(name)) => Some(EvalAttributeArg::Named {
            name: name.clone(),
            value: Box::new(value),
        }),
        EvalExpr::Const(EvalConst::Int(key)) => Some(EvalAttributeArg::IntKeyed {
            key: *key,
            value: Box::new(value),
        }),
        EvalExpr::Const(EvalConst::Bool(key)) => Some(EvalAttributeArg::IntKeyed {
            key: i64::from(*key),
            value: Box::new(value),
        }),
        EvalExpr::Const(EvalConst::Null) => Some(EvalAttributeArg::Named {
            name: String::new(),
            value: Box::new(value),
        }),
        EvalExpr::Const(EvalConst::Float(key)) => Some(EvalAttributeArg::IntKeyed {
            key: *key as i64,
            value: Box::new(value),
        }),
        EvalExpr::Unary {
            op: EvalUnaryOp::Negate,
            expr,
        } => eval_attribute_array_negated_keyed_arg(expr, value),
        EvalExpr::ClassNameFetch { class_name } => {
            eval_attribute_class_name_arg(class_name).map(|name| EvalAttributeArg::Named {
                name,
                value: Box::new(value),
            })
        }
        _ => None,
    }
}

/// Wraps an attribute array value with a normalized negative numeric literal key.
pub(super) fn eval_attribute_array_negated_keyed_arg(
    key: &EvalExpr,
    value: EvalAttributeArg,
) -> Option<EvalAttributeArg> {
    match key {
        EvalExpr::Const(EvalConst::Int(key)) => Some(EvalAttributeArg::IntKeyed {
            key: key.wrapping_neg(),
            value: Box::new(value),
        }),
        EvalExpr::Const(EvalConst::Float(key)) => Some(EvalAttributeArg::IntKeyed {
            key: (-*key) as i64,
            value: Box::new(value),
        }),
        _ => None,
    }
}

/// Returns a compile-time class-name string for named `ClassName::class` attribute args.
pub(super) fn eval_attribute_class_name_arg(class_name: &str) -> Option<String> {
    let class_name = class_name.trim_start_matches('\\');
    if ["self", "parent", "static"]
        .iter()
        .any(|special| class_name.eq_ignore_ascii_case(special))
    {
        return None;
    }
    Some(class_name.to_string())
}
