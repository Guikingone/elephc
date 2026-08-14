//! Purpose:
//! Shared PHP `TypeError` validation for the array-taking builtin argument family.
//!
//! Called from:
//! - `crate::interpreter::builtins::array` leaf builtins and the area dispatchers.
//!
//! Key details:
//! - Mirrors the compiled backend's `ARRAY_OR_FALSE_ARG_SITES` table, including PHP's
//!   variadic naming rules, so eval and codegen word the same throw identically.
//! - Type names are PHP's DEBUG spellings, not `gettype()`'s legacy ones.

use super::super::super::*;

/// The array-taking builtin arguments eval validates, with PHP's own parameter naming.
///
/// Each entry is `(builtin, zero-based argument index, PHP's parameter name)`. A `None`
/// name is PHP's VARIADIC spelling, which carries no `($name)` segment: `array_merge` is
/// fully variadic, so even its FIRST argument is unnamed, where `array_diff` and
/// `array_intersect` do name theirs and leave only the tail unnamed. Every entry was
/// measured against `php -n` 8.5.6, not inferred from the signature.
const EVAL_ARRAY_ARG_SITES: &[(&str, usize, Option<&str>)] = &[
    ("array_column", 0, Some("array")),
    ("array_count_values", 0, Some("array")),
    ("array_diff", 0, Some("array")),
    ("array_diff", 1, None),
    ("array_diff_key", 0, Some("array")),
    ("array_diff_key", 1, None),
    ("array_filter", 0, Some("array")),
    ("array_flip", 0, Some("array")),
    ("array_intersect", 0, Some("array")),
    ("array_intersect", 1, None),
    ("array_intersect_key", 0, Some("array")),
    ("array_intersect_key", 1, None),
    ("array_map", 1, Some("array")),
    ("array_merge", 0, None),
    ("array_merge", 1, None),
    ("array_pad", 0, Some("array")),
    ("array_product", 0, Some("array")),
    ("array_rand", 0, Some("array")),
    ("array_reverse", 0, Some("array")),
    ("array_search", 1, Some("haystack")),
    ("array_slice", 0, Some("array")),
    ("array_sum", 0, Some("array")),
    ("array_unique", 0, Some("array")),
    ("array_values", 0, Some("array")),
    ("in_array", 1, Some("haystack")),
];

/// Returns PHP's `TypeError` spelling for one runtime value's type.
///
/// These are PHP's DEBUG type names, not `gettype()`'s legacy ones: a bool prints as its
/// own literal (`false` / `true`) rather than `bool`, an object prints its CLASS name
/// rather than `object`, and null prints lowercase — all measured against `php -n` 8.5.6.
///
/// A class DECLARED INSIDE the same eval has no runtime class cell to read, so its name
/// comes from the eval context first; reading the runtime cell alone reports the backing
/// `stdClass` and would name the wrong class.
pub(in crate::interpreter) fn eval_argument_type_name(
    value: RuntimeCellHandle,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    let tag = values.type_tag(value)?;
    if tag == EVAL_TAG_OBJECT {
        let identity = values.object_identity(value)?;
        if let Some(class) = context.dynamic_object_class(identity) {
            return Ok(class.name().to_string());
        }
        let class_name = values.object_class_name(value)?;
        let bytes = values.string_bytes(class_name);
        values.release(class_name)?;
        let class_name = String::from_utf8(bytes?).map_err(|_| EvalStatus::RuntimeFatal)?;
        return Ok(class_name.trim_start_matches('\\').to_string());
    }
    let name = match tag {
        EVAL_TAG_INT => "int",
        EVAL_TAG_FLOAT => "float",
        EVAL_TAG_STRING => "string",
        // php prints the bool's own literal, so the value has to be read, not just tagged.
        EVAL_TAG_BOOL => {
            if values.truthy(value)? {
                "true"
            } else {
                "false"
            }
        }
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => "array",
        EVAL_TAG_RESOURCE => "resource",
        _ => "null",
    };
    Ok(name.to_string())
}

/// Composes PHP's "must be of type" `TypeError` wording for one argument slot.
///
/// `argument` is ONE-based, as PHP numbers it in the message.
fn eval_array_arg_type_error_message(
    function: &str,
    argument: usize,
    param: Option<&str>,
    expected: &str,
    actual: &str,
) -> String {
    match param {
        Some(param) => format!(
            "{function}(): Argument #{argument} (${param}) must be of type {expected}, {actual} given"
        ),
        None => {
            format!("{function}(): Argument #{argument} must be of type {expected}, {actual} given")
        }
    }
}

/// Throws PHP's `TypeError` unless one argument holds an array.
pub(in crate::interpreter) fn eval_expect_array_arg(
    value: RuntimeCellHandle,
    function: &str,
    argument: usize,
    param: Option<&str>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if matches!(values.type_tag(value)?, EVAL_TAG_ARRAY | EVAL_TAG_ASSOC) {
        return Ok(());
    }
    let actual = eval_argument_type_name(value, context, values)?;
    eval_throw_type_error(
        &eval_array_arg_type_error_message(function, argument, param, "array", &actual),
        context,
        values,
    )
}

/// Validates every table-declared array argument of one builtin against evaluated cells.
///
/// Argument slots the caller did not supply are skipped: PHP reports a missing REQUIRED
/// argument as an `ArgumentCountError`, which is not this family's concern.
pub(in crate::interpreter) fn eval_check_array_args(
    function: &str,
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for &(name, index, param) in EVAL_ARRAY_ARG_SITES {
        if name != function {
            continue;
        }
        let Some(&value) = evaluated_args.get(index) else {
            continue;
        };
        eval_expect_array_arg(value, function, index + 1, param, context, values)?;
    }
    Ok(())
}

/// Throws PHP's `TypeError` unless `count()`'s argument is an array.
///
/// `count()` is the family's odd member twice over: it expects `Countable|array`, and a
/// `Countable` OBJECT is accepted. The caller dispatches `Countable::count()` first, so
/// reaching here means the value is neither — and a plain object still names its class.
pub(in crate::interpreter) fn eval_expect_countable_arg(
    value: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if matches!(values.type_tag(value)?, EVAL_TAG_ARRAY | EVAL_TAG_ASSOC) {
        return Ok(());
    }
    let actual = eval_argument_type_name(value, context, values)?;
    eval_throw_type_error(
        &eval_array_arg_type_error_message("count", 1, Some("value"), "Countable|array", &actual),
        context,
        values,
    )
}

/// Throws PHP's `TypeError` unless a by-reference ordering builtin received an array.
///
/// The whole ordering family shares ONE wording — `sort`, `rsort`, `asort`, `arsort`,
/// `ksort`, `krsort`, `natsort`, `natcasesort`, `shuffle`, `usort`, `uasort` and `uksort`
/// all name their first parameter `$array`, measured against `php -n` 8.5.6.
pub(in crate::interpreter) fn eval_expect_sort_array_arg(
    value: RuntimeCellHandle,
    function: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    eval_expect_array_arg(value, function, 1, Some("array"), context, values)
}
