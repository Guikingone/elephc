//! Purpose:
//! Builds EvalIR array literals and computes PHP-compatible next keys for mixed array construction.
//!
//! Called from:
//! - `crate::interpreter::eval_expr()` for indexed and associative array literal nodes.
//!
//! Key details:
//! - Explicit keys are normalized through runtime string conversion to match PHP array-key rules.
//! - Unkeyed elements continue from the next PHP integer key after explicit keys.

use super::*;

/// Evaluates an indexed array literal into a boxed runtime Mixed array.
pub(super) fn eval_indexed_array(
    elements: &[EvalArrayElement],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut array = values.array_new(elements.len())?;
    for (index, element) in elements.iter().enumerate() {
        let index = values.int(index as i64)?;
        let (value, target) = match element {
            EvalArrayElement::Value(element) => (eval_expr(element, context, scope, values)?, None),
            EvalArrayElement::Reference(element) => {
                let (value, target) =
                    eval_reference_array_element_value(element, context, scope, values)?;
                (value, Some(target))
            }
            EvalArrayElement::KeyValue { .. } | EvalArrayElement::KeyReference { .. } => {
                return Err(EvalStatus::UnsupportedConstruct);
            }
        };
        array = values.array_set(array, index, value)?;
        if let Some(target) = target {
            bind_array_element_reference(context, array, index, target, values)?;
        }
    }
    Ok(array)
}

/// Evaluates an associative array literal into a boxed runtime Mixed hash.
pub(super) fn eval_assoc_array(
    elements: &[EvalArrayElement],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut array = values.assoc_new(elements.len())?;
    let mut next_key = None;
    for (element_index, element) in elements.iter().enumerate() {
        let (key, value, target) = match element {
            EvalArrayElement::Value(value) => {
                let key = match next_key {
                    Some(next_key) => next_key,
                    None => values.int(0)?,
                };
                let one = values.int(1)?;
                next_key = Some(values.add(key, one)?);
                let value = eval_expr(value, context, scope, values).map_err(|status| {
                    trace_array_literal_error("value", element_index, status, context)
                })?;
                (key, value, None)
            }
            EvalArrayElement::Reference(value) => {
                let key = match next_key {
                    Some(next_key) => next_key,
                    None => values.int(0)?,
                };
                let one = values.int(1)?;
                next_key = Some(values.add(key, one)?);
                let (value, target) =
                    eval_reference_array_element_value(value, context, scope, values)?;
                (key, value, Some(target))
            }
            EvalArrayElement::KeyValue { key, value } => {
                let key = eval_expr(key, context, scope, values).map_err(|status| {
                    trace_array_literal_error("key", element_index, status, context)
                })?;
                next_key = eval_array_next_key_after_explicit_key(key, next_key, values)?;
                let value = eval_expr(value, context, scope, values).map_err(|status| {
                    trace_array_literal_error("value", element_index, status, context)
                })?;
                (key, value, None)
            }
            EvalArrayElement::KeyReference { key, value } => {
                let key = eval_expr(key, context, scope, values)?;
                next_key = eval_array_next_key_after_explicit_key(key, next_key, values)?;
                let (value, target) =
                    eval_reference_array_element_value(value, context, scope, values)?;
                (key, value, Some(target))
            }
        };
        array = values.array_set(array, key, value).map_err(|status| {
            trace_array_literal_error("store", element_index, status, context)
        })?;
        if let Some(target) = target {
            bind_array_element_reference(context, array, key, target, values)?;
        }
    }
    Ok(array)
}

/// Emits the associative-array element stage that failed under opt-in runtime tracing.
fn trace_array_literal_error(
    stage: &str,
    index: usize,
    status: EvalStatus,
    context: &ElephcEvalContext,
) -> EvalStatus {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        let call_site = context.call_site();
        eprintln!(
            "[elephc-eval-trace] phase=array_literal_error stage={stage} index={index} status={status:?} file={:?} line={}",
            call_site.0,
            call_site.2,
        );
    }
    status
}

/// Evaluates a by-reference array literal element and captures its writable source target.
fn eval_reference_array_element_value(
    value: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(RuntimeCellHandle, EvalReferenceTarget), EvalStatus> {
    let (value, target) = eval_call_arg_value(value, context, scope, values)?;
    target
        .map(|target| (value, target))
        .ok_or(EvalStatus::RuntimeFatal)
}

/// Records one by-reference array element on the eval context side table.
fn bind_array_element_reference(
    context: &mut ElephcEvalContext,
    array: RuntimeCellHandle,
    key: RuntimeCellHandle,
    target: EvalReferenceTarget,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let key = eval_array_reference_key(key, values)?.ok_or(EvalStatus::RuntimeFatal)?;
    let array_identity = values.raw_value_word(array)?;
    context.bind_array_element_alias(array_identity, key, target);
    Ok(())
}

/// Normalizes a PHP array key for eval reference metadata lookups.
pub(in crate::interpreter) fn eval_array_reference_key(
    key: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<EvalArrayReferenceKey>, EvalStatus> {
    Ok(Some(match values.type_tag(key)? {
        EVAL_TAG_INT => EvalArrayReferenceKey::Int(eval_int_value(key, values)?),
        EVAL_TAG_STRING => {
            let bytes = values.string_bytes(key)?;
            if let Some(key) = eval_numeric_string_array_key(&bytes) {
                EvalArrayReferenceKey::Int(key)
            } else {
                EvalArrayReferenceKey::String(bytes)
            }
        }
        EVAL_TAG_NULL => EvalArrayReferenceKey::String(Vec::new()),
        EVAL_TAG_BOOL | EVAL_TAG_FLOAT => EvalArrayReferenceKey::Int(eval_int_value(key, values)?),
        _ => return Ok(None),
    }))
}

/// Materializes a stable eval array-reference key as a temporary runtime cell.
pub(in crate::interpreter) fn eval_array_reference_key_value(
    key: &EvalArrayReferenceKey,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match key {
        EvalArrayReferenceKey::Int(value) => values.int(*value),
        EvalArrayReferenceKey::String(bytes) => values.string_bytes_value(bytes),
    }
}

/// Advances an array literal's automatic key after an integer-normalized explicit key.
fn eval_array_next_key_after_explicit_key(
    key: RuntimeCellHandle,
    current_next_key: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<RuntimeCellHandle>, EvalStatus> {
    let key = match values.type_tag(key)? {
        EVAL_TAG_INT => key,
        EVAL_TAG_STRING => {
            let bytes = values.string_bytes(key)?;
            let Some(key) = eval_numeric_string_array_key(&bytes) else {
                return Ok(current_next_key);
            };
            values.int(key)?
        }
        EVAL_TAG_NULL => return Ok(current_next_key),
        _ => values.cast_int(key)?,
    };
    let one = values.int(1)?;
    let candidate = values.add(key, one)?;
    let replace = if let Some(current_next_key) = current_next_key {
        let is_greater = values.compare(EvalBinOp::Gt, candidate, current_next_key)?;
        values.truthy(is_greater)?
    } else {
        true
    };
    Ok(if replace {
        Some(candidate)
    } else {
        current_next_key
    })
}

/// Parses PHP integer-string array keys that normalize to integer keys.
pub(in crate::interpreter) fn eval_numeric_string_array_key(bytes: &[u8]) -> Option<i64> {
    if bytes.is_empty() {
        return None;
    }

    let (negative, digits) = if bytes[0] == b'-' {
        if bytes.len() == 1 {
            return None;
        }
        (true, &bytes[1..])
    } else {
        (false, bytes)
    };

    if digits[0] == b'0' {
        return if !negative && digits.len() == 1 {
            Some(0)
        } else {
            None
        };
    }
    if digits.iter().any(|byte| !byte.is_ascii_digit()) {
        return None;
    }

    let limit = if negative {
        i64::MAX as u128 + 1
    } else {
        i64::MAX as u128
    };
    let mut value = 0u128;
    for digit in digits {
        value = (value * 10) + u128::from(digit - b'0');
        if value > limit {
            return None;
        }
    }

    if negative {
        if value == i64::MAX as u128 + 1 {
            Some(i64::MIN)
        } else {
            Some(-(value as i64))
        }
    } else {
        Some(value as i64)
    }
}
