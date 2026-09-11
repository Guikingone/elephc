//! Purpose:
//! Declarative eval registry entry and implementation for `serialize`.
//!
//! Called from:
//! - `crate::interpreter::builtins::core`.
//!
//! Key details:
//! - The wire format is php's own `var_export`-adjacent serialization grammar
//!   (`N;`, `b:0|1;`, `i:<int>;`, `d:<float>;`, `s:<byte-len>:"<bytes>";`,
//!   `a:<count>:{<key><value>...}`, `O:<name-len>:"<name>":<count>:{<key><value>...}`).
//! - Object property enumeration and visibility reuse the SAME machinery `print_r()`/
//!   `var_dump()` use (`eval_debug_object_properties`, `crate::interpreter::builtins::core::
//!   print_r`), so a private/protected property gets the correct NUL-mangled key
//!   (`"\0ClassName\0prop"` / `"\0*\0prop"`) instead of a plain (public) one -- the exact bug
//!   PLAN.md warns about: a wrong mangling here corrupts the app's own cache-freshness metadata
//!   with no crash, just a wrong decision.
//! - NOT implemented (measured absent from the real 404 payloads and out of this wave's scope):
//!   `__serialize()`/`Serializable` magic methods, and php's `r:`/`R:` reference back-edges for
//!   a value or object that appears more than once in the same structure -- a self-referencing
//!   structure is refused with a `RuntimeFatal` (via a simple object-identity cycle guard)
//!   instead of looping forever, which is not php's behavior but is safe.

use super::super::super::*;

eval_builtin! {
    contract: "serialize",
    area: Core,
    direct: Core,
    values: Core,
}

/// Dispatches a direct eval `serialize()` call, preserving PHP source argument order.
pub(in crate::interpreter) fn eval_builtin_serialize(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [value] = args else {
        return eval_throw_argument_count_error(
            &format!(
                "serialize() expects exactly 1 argument, {} given",
                args.len()
            ),
            context,
            values,
        );
    };
    let value = eval_expr(value, context, scope, values)?;
    eval_serialize_result(value, context, values)
}

/// Dispatches an evaluated-argument `serialize()` call (named args, spread, callable dispatch).
pub(in crate::interpreter) fn eval_serialize_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [value] = evaluated_args else {
        return eval_throw_argument_count_error(
            &format!(
                "serialize() expects exactly 1 argument, {} given",
                evaluated_args.len()
            ),
            context,
            values,
        );
    };
    eval_serialize_result(*value, context, values)
}

/// Serializes one eval value into php's storable-representation byte string.
fn eval_serialize_result(
    value: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut buf = Vec::new();
    let mut objects_seen = Vec::new();
    eval_serialize_append_value(value, context, values, &mut objects_seen, &mut buf)?;
    values.string_bytes_value(&buf)
}

/// Appends one value's serialized byte representation, recursing into arrays and objects.
fn eval_serialize_append_value(
    value: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
    objects_seen: &mut Vec<usize>,
    buf: &mut Vec<u8>,
) -> Result<(), EvalStatus> {
    match values.type_tag(value)? {
        EVAL_TAG_NULL => buf.extend_from_slice(b"N;"),
        EVAL_TAG_BOOL => {
            buf.extend_from_slice(if values.truthy(value)? { b"b:1;" } else { b"b:0;" });
        }
        EVAL_TAG_INT => {
            let n = eval_int_value(value, values)?;
            buf.extend_from_slice(format!("i:{n};").as_bytes());
        }
        EVAL_TAG_FLOAT => {
            // Reuses the SAME shared float-to-string conversion `json_encode()` reuses
            // (`values.string_bytes`), then corrects only the NAN/INF spelling to php's
            // serialize wording -- php's own text ("NAN"/"INF"/"-INF") differs from what a
            // Rust `f64::to_string()`-shaped conversion would say ("NaN"/"inf"/"-inf").
            let bytes = values.string_bytes(value)?;
            let text = String::from_utf8(bytes).map_err(|_| EvalStatus::RuntimeFatal)?;
            let wire = match text.as_str() {
                "NaN" => "NAN".to_string(),
                "inf" => "INF".to_string(),
                "-inf" => "-INF".to_string(),
                other => other.to_string(),
            };
            buf.extend_from_slice(format!("d:{wire};").as_bytes());
        }
        EVAL_TAG_STRING => {
            let bytes = values.string_bytes(value)?;
            buf.extend_from_slice(format!("s:{}:\"", bytes.len()).as_bytes());
            buf.extend_from_slice(&bytes);
            buf.extend_from_slice(b"\";");
        }
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => {
            let len = values.array_len(value)?;
            buf.extend_from_slice(format!("a:{len}:{{").as_bytes());
            for position in 0..len {
                let key = values.array_iter_key(value, position)?;
                let element = values.array_get(value, key)?;
                eval_serialize_append_value(key, context, values, objects_seen, buf)?;
                eval_serialize_append_value(element, context, values, objects_seen, buf)?;
            }
            buf.push(b'}');
        }
        EVAL_TAG_OBJECT => {
            let identity = eval_debug_object_identity(value, values);
            let object_key = identity.unwrap_or(value.as_ptr() as usize as u64) as usize;
            if objects_seen.contains(&object_key) {
                // php would emit an `r:`/`R:` back-reference here; not implemented (see the
                // module doc). Refusing beats looping forever on a self-referencing structure.
                return Err(EvalStatus::RuntimeFatal);
            }
            objects_seen.push(object_key);
            let class_name = eval_debug_object_class_name(value, identity, context, values)?;
            let properties =
                eval_debug_object_properties(value, identity, &class_name, context, values)?;
            buf.extend_from_slice(
                format!("O:{}:\"{}\":{}:{{", class_name.len(), class_name, properties.len())
                    .as_bytes(),
            );
            for property in &properties {
                let key = eval_serialize_mangled_property_key(property);
                buf.extend_from_slice(format!("s:{}:\"", key.len()).as_bytes());
                buf.extend_from_slice(key.as_bytes());
                buf.extend_from_slice(b"\";");
                eval_serialize_append_value(property.value, context, values, objects_seen, buf)?;
            }
            buf.push(b'}');
            objects_seen.pop();
        }
        // A resource carries no storable state; php serializes it as `i:0;` with no warning
        // (measured: `serialize(fopen(...))` -> `i:0;`, open or closed).
        _ => buf.extend_from_slice(b"i:0;"),
    }
    Ok(())
}

/// Computes one object property's serialized key, NUL-mangling it for protected/private the
/// same way php's own wire format does: `"\0*\0name"` for protected, `"\0Class\0name"` for
/// private, and the plain name for public.
fn eval_serialize_mangled_property_key(property: &EvalDebugObjectProperty) -> String {
    match &property.visibility.kind {
        EvalDebugPropertyVisibilityKind::Public => property.name.clone(),
        EvalDebugPropertyVisibilityKind::Protected => format!("\0*\0{}", property.name),
        EvalDebugPropertyVisibilityKind::Private(class_name) => {
            format!("\0{class_name}\0{}", property.name)
        }
    }
}
