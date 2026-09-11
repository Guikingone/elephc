//! Purpose:
//! Declarative eval registry entry and implementation for `unserialize`.
//!
//! Called from:
//! - `crate::interpreter::builtins::core`.
//!
//! Key details:
//! - Parses php's own storable-representation grammar: `N;`, `b:0|1;`, `i:<int>;`, `d:<float>;`,
//!   `s:<byte-len>:"<bytes>";`, `a:<count>:{<key><value>...}`,
//!   `O:<name-len>:"<name>":<count>:{<key><value>...}`.
//! - Object hydration for a class-name-allowed payload writes each DECLARED property through the
//!   SAME storage-name + overlay + slot-mirror machinery `eval_dynamic_class_allocate_object()`
//!   uses when it seeds default property values (`crate::interpreter::statements::
//!   class_resolution`), so a private property from an UNMANGLED payload key
//!   (`s:1:"a";s:2:"AA";`) lands in the DECLARED private slot -- `"\0ClassName\0a"` -- rather than
//!   creating a dynamic public property beside a still-default private one. This is deliberately
//!   the FIRST thing this file's tests assert (see `builtins_unserialize.rs`): PLAN.md notes this
//!   exact shape is the literal content of a Symfony cache-freshness payload, so a wrong binding
//!   here is a wrong cache decision with no crash, not a test failure anyone would notice by
//!   running the app.
//! - NOT implemented (measured absent from the real 404 payloads; each is a separate, larger
//!   ticket): `r:`/`R:` reference back-edges, `E:` enum payloads, `C:` `Serializable` payloads,
//!   `unserialize_callback_func` (the ini key still round-trips through `ini_get`/`ini_set`
//!   independently of this file), and `max_depth`/`unserialize_max_depth` (no depth guard at
//!   all -- a malicious/corrupt deeply-nested payload can blow the Rust stack, matching php's own
//!   guard being the thing missing, not present-but-wrong). Byte-exact warning OFFSETS are
//!   best-effort (the offset is wherever this scanner's cursor sat when it gave up), not a
//!   guaranteed match to php's own scanner position for every malformed shape.

use super::super::super::*;

eval_builtin! {
    contract: "unserialize",
    area: Core,
    direct: Core,
    values: Core,
}

/// Dispatches a direct eval `unserialize()` call, preserving PHP source argument order.
pub(in crate::interpreter) fn eval_builtin_unserialize(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [] => eval_throw_argument_count_error(
            "unserialize() expects at least 1 argument, 0 given",
            context,
            values,
        ),
        [data] => {
            let data = eval_expr(data, context, scope, values)?;
            eval_unserialize_call(data, None, context, values)
        }
        [data, options] => {
            let data = eval_expr(data, context, scope, values)?;
            let options = eval_expr(options, context, scope, values)?;
            eval_unserialize_call(data, Some(options), context, values)
        }
        _ => eval_throw_argument_count_error(
            &format!("unserialize() expects at most 2 arguments, {} given", args.len()),
            context,
            values,
        ),
    }
}

/// Dispatches an evaluated-argument `unserialize()` call (named args, spread, callable dispatch).
pub(in crate::interpreter) fn eval_unserialize_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match evaluated_args {
        [] => eval_throw_argument_count_error(
            "unserialize() expects at least 1 argument, 0 given",
            context,
            values,
        ),
        [data] => eval_unserialize_call(*data, None, context, values),
        [data, options] => eval_unserialize_call(*data, Some(*options), context, values),
        _ => eval_throw_argument_count_error(
            &format!(
                "unserialize() expects at most 2 arguments, {} given",
                evaluated_args.len()
            ),
            context,
            values,
        ),
    }
}

/// Which class names `unserialize()` may hydrate as real objects.
enum EvalUnserializeAllowedClasses {
    All,
    None,
    List(Vec<String>),
}

impl EvalUnserializeAllowedClasses {
    fn allows(&self, class_name: &str) -> bool {
        match self {
            Self::All => true,
            Self::None => false,
            Self::List(names) => names.iter().any(|name| name.eq_ignore_ascii_case(class_name)),
        }
    }
}

/// Validates arguments, reads the `allowed_classes` option, and drives the parse.
fn eval_unserialize_call(
    data: RuntimeCellHandle,
    options: Option<RuntimeCellHandle>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if values.type_tag(data)? != EVAL_TAG_STRING {
        let given = eval_given_type_spelling(data, values)?;
        return eval_throw_type_error(
            &format!("unserialize(): Argument #1 ($data) must be of type string, {given} given"),
            context,
            values,
        );
    }
    let allowed_classes = match options {
        None => EvalUnserializeAllowedClasses::All,
        Some(options) => {
            if !matches!(values.type_tag(options)?, EVAL_TAG_ARRAY | EVAL_TAG_ASSOC) {
                let given = eval_given_type_spelling(options, values)?;
                return eval_throw_type_error(
                    &format!(
                        "unserialize(): Argument #2 ($options) must be of type array, {given} given"
                    ),
                    context,
                    values,
                );
            }
            eval_unserialize_allowed_classes_option(options, context, values)?
        }
    };

    let bytes = values.string_bytes(data)?;
    if bytes.is_empty() {
        // Measured: `unserialize('')` is `false` with NO warning -- Symfony reads possibly-empty
        // cache files, and this silence is load-bearing.
        return values.bool_value(false);
    }

    let mut cursor = EvalUnserializeCursor { data: &bytes, pos: 0 };
    match eval_unserialize_parse_value(&mut cursor, &allowed_classes, context, values) {
        Ok(value) => {
            if cursor.pos < bytes.len() {
                values.warning(&format!(
                    "unserialize(): Extra data starting at offset {} of {} bytes",
                    cursor.pos,
                    bytes.len()
                ))?;
            }
            Ok(value)
        }
        Err(EvalUnserializeFailure::AlreadyWarned) => values.bool_value(false),
        Err(EvalUnserializeFailure::Generic) => {
            values.warning(&format!(
                "unserialize(): Error at offset {} of {} bytes",
                cursor.pos,
                bytes.len()
            ))?;
            values.bool_value(false)
        }
    }
}

/// Reads `$options['allowed_classes']`, defaulting to `All` when the key is absent.
fn eval_unserialize_allowed_classes_option(
    options: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalUnserializeAllowedClasses, EvalStatus> {
    let key = values.string("allowed_classes")?;
    let exists = values.array_key_exists(key, options)?;
    let present = values.truthy(exists)?;
    if !present {
        return Ok(EvalUnserializeAllowedClasses::All);
    }
    let key = values.string("allowed_classes")?;
    let value = values.array_get(options, key)?;
    match values.type_tag(value)? {
        EVAL_TAG_BOOL => Ok(if values.truthy(value)? {
            EvalUnserializeAllowedClasses::All
        } else {
            EvalUnserializeAllowedClasses::None
        }),
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => {
            let len = values.array_len(value)?;
            let mut names = Vec::with_capacity(len);
            for position in 0..len {
                let key = values.array_iter_key(value, position)?;
                let element = values.array_get(value, key)?;
                let bytes = values.string_bytes(element)?;
                names.push(String::from_utf8_lossy(&bytes).into_owned());
            }
            Ok(EvalUnserializeAllowedClasses::List(names))
        }
        _ => {
            let given = eval_given_type_spelling(value, values)?;
            eval_throw_type_error(
                &format!(
                    "unserialize(): Option \"allowed_classes\" must be of type array|bool, {given} given"
                ),
                context,
                values,
            )
        }
    }
}

/// A parse failure: either a specific warning was already emitted (a warning shape more precise
/// than the generic wrapper), or the caller must emit php's generic `Error at offset` wrapper.
enum EvalUnserializeFailure {
    AlreadyWarned,
    Generic,
}

/// A byte-position cursor over the serialized payload.
struct EvalUnserializeCursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> EvalUnserializeCursor<'a> {
    fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    fn take_byte(&mut self, byte: u8) -> Result<(), EvalUnserializeFailure> {
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(EvalUnserializeFailure::Generic)
        }
    }

    /// Reads bytes up to (not including) the next occurrence of `delim`, advancing past it.
    fn take_until(&mut self, delim: u8) -> Result<&'a [u8], EvalUnserializeFailure> {
        let start = self.pos;
        while self.peek() != Some(delim) {
            if self.peek().is_none() {
                return Err(EvalUnserializeFailure::Generic);
            }
            self.pos += 1;
        }
        let slice = &self.data[start..self.pos];
        self.pos += 1;
        Ok(slice)
    }

    /// Reads exactly `len` bytes without any delimiter.
    fn take_exact(&mut self, len: usize) -> Result<&'a [u8], EvalUnserializeFailure> {
        if self.pos + len > self.data.len() {
            return Err(EvalUnserializeFailure::Generic);
        }
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }
}

/// Parses one serialized value at the cursor's current position.
fn eval_unserialize_parse_value(
    cursor: &mut EvalUnserializeCursor<'_>,
    allowed_classes: &EvalUnserializeAllowedClasses,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalUnserializeFailure> {
    match cursor.peek() {
        Some(b'N') => {
            cursor.pos += 1;
            cursor.take_byte(b';')?;
            values.null().map_err(|_| EvalUnserializeFailure::Generic)
        }
        Some(b'b') => {
            cursor.pos += 1;
            cursor.take_byte(b':')?;
            let digit = cursor.take_until(b';')?;
            let flag = match digit {
                b"1" => true,
                b"0" => false,
                _ => return Err(EvalUnserializeFailure::Generic),
            };
            values.bool_value(flag).map_err(|_| EvalUnserializeFailure::Generic)
        }
        Some(b'i') => {
            cursor.pos += 1;
            cursor.take_byte(b':')?;
            let digits = cursor.take_until(b';')?;
            let text = std::str::from_utf8(digits).map_err(|_| EvalUnserializeFailure::Generic)?;
            match text.parse::<i64>() {
                Ok(n) => values.int(n).map_err(|_| EvalUnserializeFailure::Generic),
                Err(_) => {
                    if text.is_empty() || !text.trim_start_matches('-').bytes().all(|b| b.is_ascii_digit())
                    {
                        return Err(EvalUnserializeFailure::Generic);
                    }
                    // Digit string is well-formed but out of i64 range: php clamps to the
                    // PHP_INT_MAX/PHP_INT_MIN boundary with a recoverable warning, not a failure.
                    values
                        .warning("unserialize(): Numerical result out of range")
                        .map_err(|_| EvalUnserializeFailure::Generic)?;
                    let clamped = if text.starts_with('-') { i64::MIN } else { i64::MAX };
                    values.int(clamped).map_err(|_| EvalUnserializeFailure::Generic)
                }
            }
        }
        Some(b'd') => {
            cursor.pos += 1;
            cursor.take_byte(b':')?;
            let digits = cursor.take_until(b';')?;
            let text = std::str::from_utf8(digits).map_err(|_| EvalUnserializeFailure::Generic)?;
            let parsed = match text {
                "NAN" => f64::NAN,
                "INF" => f64::INFINITY,
                "-INF" => f64::NEG_INFINITY,
                other => other.parse::<f64>().map_err(|_| EvalUnserializeFailure::Generic)?,
            };
            values.float(parsed).map_err(|_| EvalUnserializeFailure::Generic)
        }
        Some(b's') => {
            let bytes = eval_unserialize_parse_raw_string(cursor)?;
            values
                .string_bytes_value(&bytes)
                .map_err(|_| EvalUnserializeFailure::Generic)
        }
        Some(b'a') => eval_unserialize_parse_array(cursor, allowed_classes, context, values),
        Some(b'O') => eval_unserialize_parse_object(cursor, allowed_classes, context, values),
        _ => Err(EvalUnserializeFailure::Generic),
    }
}

/// Parses the raw byte payload of one `s:<len>:"<bytes>";` string, WITHOUT constructing a cell --
/// shared by value parsing and by array/object key parsing, which both need the same grammar.
fn eval_unserialize_parse_raw_string(
    cursor: &mut EvalUnserializeCursor<'_>,
) -> Result<Vec<u8>, EvalUnserializeFailure> {
    cursor.take_byte(b's')?;
    cursor.take_byte(b':')?;
    let len_digits = cursor.take_until(b':')?;
    let len: usize = std::str::from_utf8(len_digits)
        .ok()
        .and_then(|text| text.parse().ok())
        .ok_or(EvalUnserializeFailure::Generic)?;
    cursor.take_byte(b'"')?;
    let bytes = cursor.take_exact(len)?.to_vec();
    cursor.take_byte(b'"')?;
    cursor.take_byte(b';')?;
    Ok(bytes)
}

/// Parses `a:<count>:{<key><value>...}` into a fresh eval array.
fn eval_unserialize_parse_array(
    cursor: &mut EvalUnserializeCursor<'_>,
    allowed_classes: &EvalUnserializeAllowedClasses,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalUnserializeFailure> {
    cursor.take_byte(b'a')?;
    cursor.take_byte(b':')?;
    let count_digits = cursor.take_until(b':')?;
    let count: usize = std::str::from_utf8(count_digits)
        .ok()
        .and_then(|text| text.parse().ok())
        .ok_or(EvalUnserializeFailure::Generic)?;
    cursor.take_byte(b'{')?;
    let mut result = values.assoc_new(count).map_err(|_| EvalUnserializeFailure::Generic)?;
    for _ in 0..count {
        if cursor.peek().is_none() {
            values
                .warning("unserialize(): Unexpected end of serialized data")
                .map_err(|_| EvalUnserializeFailure::Generic)?;
            return Err(EvalUnserializeFailure::AlreadyWarned);
        }
        let key = eval_unserialize_parse_value(cursor, allowed_classes, context, values)?;
        if cursor.peek().is_none() {
            values
                .warning("unserialize(): Unexpected end of serialized data")
                .map_err(|_| EvalUnserializeFailure::Generic)?;
            return Err(EvalUnserializeFailure::AlreadyWarned);
        }
        let element = eval_unserialize_parse_value(cursor, allowed_classes, context, values)?;
        result = values
            .array_set(result, key, element)
            .map_err(|_| EvalUnserializeFailure::Generic)?;
    }
    cursor.take_byte(b'}')?;
    eval_unserialize_retag_as_list_if_packed(result, values)
}

/// Re-tags a fold result as a packed list when its key sequence is exactly `0..len`, matching
/// `array_replace`'s own rule: the tag must follow the keys, not the builder's internal choice.
fn eval_unserialize_retag_as_list_if_packed(
    assoc: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalUnserializeFailure> {
    let len = values.array_len(assoc).map_err(|_| EvalUnserializeFailure::Generic)?;
    for position in 0..len {
        let key = values
            .array_iter_key(assoc, position)
            .map_err(|_| EvalUnserializeFailure::Generic)?;
        let is_sequential_int = values.type_tag(key).map_err(|_| EvalUnserializeFailure::Generic)?
            == EVAL_TAG_INT
            && values.raw_value_word(key).map_err(|_| EvalUnserializeFailure::Generic)? as i64
                == position as i64;
        if !is_sequential_int {
            return Ok(assoc);
        }
    }
    let mut list = values.array_new(len).map_err(|_| EvalUnserializeFailure::Generic)?;
    for position in 0..len {
        let key = values
            .array_iter_key(assoc, position)
            .map_err(|_| EvalUnserializeFailure::Generic)?;
        let value = values
            .array_get(assoc, key)
            .map_err(|_| EvalUnserializeFailure::Generic)?;
        list = values
            .array_set(list, key, value)
            .map_err(|_| EvalUnserializeFailure::Generic)?;
    }
    values.release(assoc).map_err(|_| EvalUnserializeFailure::Generic)?;
    Ok(list)
}

/// Parses `O:<name-len>:"<name>":<count>:{<key><value>...}` and hydrates an object.
fn eval_unserialize_parse_object(
    cursor: &mut EvalUnserializeCursor<'_>,
    allowed_classes: &EvalUnserializeAllowedClasses,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalUnserializeFailure> {
    cursor.take_byte(b'O')?;
    cursor.take_byte(b':')?;
    let name_len_digits = cursor.take_until(b':')?;
    let name_len: usize = std::str::from_utf8(name_len_digits)
        .ok()
        .and_then(|text| text.parse().ok())
        .ok_or(EvalUnserializeFailure::Generic)?;
    cursor.take_byte(b'"')?;
    let name_bytes = cursor.take_exact(name_len)?.to_vec();
    cursor.take_byte(b'"')?;
    cursor.take_byte(b':')?;
    let class_name =
        String::from_utf8(name_bytes).map_err(|_| EvalUnserializeFailure::Generic)?;
    let count_digits = cursor.take_until(b':')?;
    let count: usize = std::str::from_utf8(count_digits)
        .ok()
        .and_then(|text| text.parse().ok())
        .ok_or(EvalUnserializeFailure::Generic)?;
    cursor.take_byte(b'{')?;

    let mut payload: Vec<(String, RuntimeCellHandle)> = Vec::with_capacity(count);
    for _ in 0..count {
        if cursor.peek().is_none() {
            values
                .warning("unserialize(): Unexpected end of serialized data")
                .map_err(|_| EvalUnserializeFailure::Generic)?;
            return Err(EvalUnserializeFailure::AlreadyWarned);
        }
        let key_bytes = eval_unserialize_parse_raw_string(cursor)?;
        let key = String::from_utf8_lossy(&key_bytes).into_owned();
        if cursor.peek().is_none() {
            values
                .warning("unserialize(): Unexpected end of serialized data")
                .map_err(|_| EvalUnserializeFailure::Generic)?;
            return Err(EvalUnserializeFailure::AlreadyWarned);
        }
        let value = eval_unserialize_parse_value(cursor, allowed_classes, context, values)?;
        payload.push((key, value));
    }
    cursor.take_byte(b'}')?;

    eval_unserialize_hydrate_object(&class_name, payload, allowed_classes, context, values)
        .map_err(|_| EvalUnserializeFailure::Generic)
}

/// Builds the actual object once a class name and its property payload have been parsed --
/// separated from the byte-cursor parser above so its error type is the ordinary `EvalStatus`
/// every other object-construction helper in this crate already uses.
fn eval_unserialize_hydrate_object(
    class_name: &str,
    payload: Vec<(String, RuntimeCellHandle)>,
    allowed_classes: &EvalUnserializeAllowedClasses,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let class_is_known = context.class(class_name).is_some() || values.class_exists(class_name)?;
    if !class_is_known || !allowed_classes.allows(class_name) {
        return eval_unserialize_incomplete_class(class_name, payload, values);
    }
    if let Some(class) = context.class(class_name).cloned() {
        let mut scope = ElephcEvalScope::new();
        let object = eval_dynamic_class_allocate_object(&class, context, &mut scope, values)?;
        let identity = values.object_identity(object)?;
        // Borrows `payload` (not moved yet): a `__unserialize()` hook below needs the RAW,
        // unmangled payload it was given, not the object's post-write declared-property table
        // (which would also carry every default `eval_dynamic_class_allocate_object` just
        // seeded for a property the payload never mentioned).
        for &(ref name, value) in &payload {
            if let Some((declaring_class, property)) = context.class_property(class.name(), name) {
                let storage_name = eval_instance_property_storage_name(&declaring_class, &property);
                if property.visibility() == EvalVisibility::Public {
                    let _ = values.property_set(object, &storage_name, value);
                }
                eval_store_dynamic_property_value(identity, &storage_name, value, context, values)?;
                context.mark_dynamic_property_initialized(identity, &storage_name);
            } else {
                let _ = values.property_set(object, name, value);
                eval_store_dynamic_property_value(identity, name, value, context, values)?;
                context.mark_dynamic_property_initialized(identity, name);
            }
        }
        if let Some((declaring_class, method)) = context.class_method(class.name(), "__unserialize") {
            let mut payload_array = values.assoc_new(payload.len())?;
            for (name, value) in payload {
                let key = values.string(&name)?;
                payload_array = values.array_set(payload_array, key, value)?;
            }
            let result = eval_dynamic_method_with_values(
                &declaring_class,
                class.name(),
                &method,
                object,
                vec![EvaluatedCallArg {
                    name: None,
                    value: payload_array,
                    ref_target: None,
                    owned: true,
                }],
                context,
                values,
            )?;
            values.release(result)?;
        }
        return Ok(object);
    }
    // A native/AOT class with no eval-declared metadata (e.g. a genuine built-in): there is no
    // declared-property table to consult, so every payload property lands as a plain dynamic
    // one. Known limitation: a native class with real private state would mis-bind here exactly
    // as an undeclared eval property would -- out of scope, no such class is on the 404 path.
    let object = values.new_object(class_name)?;
    for (name, value) in payload {
        let _ = values.property_set(object, &name, value);
    }
    Ok(object)
}

/// Builds a `__PHP_Incomplete_Class` object for a disallowed or unresolvable class name, with
/// `__PHP_Incomplete_Class_Name` set FIRST and every payload property carried through, unmangled,
/// in payload order -- no declared defaults, matching php's own (measured) shape exactly.
fn eval_unserialize_incomplete_class(
    class_name: &str,
    payload: Vec<(String, RuntimeCellHandle)>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let object = values.new_object("__PHP_Incomplete_Class")?;
    let marker_name = values.string(class_name)?;
    values.property_set(object, "__PHP_Incomplete_Class_Name", marker_name)?;
    for (name, value) in payload {
        values.property_set(object, &name, value)?;
    }
    Ok(object)
}
