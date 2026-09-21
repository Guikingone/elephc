//! Purpose:
//! Eval registry entry and implementation for `strncmp` and `strncasecmp`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Both take a THIRD argument, the byte length to compare, which is why they cannot reuse the
//!   two-argument `StringCompare` hook that `strcmp`/`strcasecmp` share.
//! - php-src compares `min($length, strlen($a))` against `min($length, strlen($b))` — it does NOT
//!   pad — so `strncmp('ab', 'abc', 5)` is negative, not zero.
//! - A negative `$length` is a `ValueError` in PHP 8 (`must be greater than or equal to 0`).
//! - Twig's `FilesystemLoader::findTemplate` calls `strncmp($name, '@', 1)` on every template
//!   lookup, so a compiled Symfony app renders no template at all without this.

use super::super::super::*;

eval_builtin! {
    contract: "strncmp",
    area: String,
    direct: StringCompareN,
    values: StringCompareN,
}

/// Evaluates PHP `strncmp(...)` from unevaluated call-site expressions.
pub(in crate::interpreter) fn eval_builtin_strncmp(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_string_compare_n_named("strncmp", args, context, scope, values)
}

/// Evaluates PHP `strncasecmp(...)` from unevaluated call-site expressions.
pub(in crate::interpreter) fn eval_builtin_strncasecmp(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_string_compare_n_named("strncasecmp", args, context, scope, values)
}

/// Evaluates one named length-limited string comparison from call-site expressions.
pub(in crate::interpreter) fn eval_builtin_string_compare_n_named(
    name: &str,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [left, right, length] = args else {
        return eval_throw_argument_count_error(
            &format!("{name}() expects exactly 3 arguments, {} given", args.len()),
            context,
            values,
        );
    };
    let left = eval_expr(left, context, scope, values)?;
    let right = eval_expr(right, context, scope, values)?;
    let length = eval_expr(length, context, scope, values)?;
    eval_string_compare_n_named_result(name, &[left, right, length], context, values)
}

/// Applies one named length-limited string comparison to already-evaluated arguments.
pub(in crate::interpreter) fn eval_string_compare_n_named_result(
    name: &str,
    args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [left, right, length] = args else {
        return eval_throw_argument_count_error(
            &format!("{name}() expects exactly 3 arguments, {} given", args.len()),
            context,
            values,
        );
    };
    let mut left = eval_require_string_arg(*left, name, 1, "string1", context, values)?;
    let mut right = eval_require_string_arg(*right, name, 2, "string2", context, values)?;
    let length = eval_require_int_arg(*length, name, 3, "length", context, values)?;
    if length < 0 {
        return eval_throw_builtin_value_error(
            &format!("{name}(): Argument #3 ($length) must be greater than or equal to 0"),
            context,
            values,
        );
    }
    match name {
        "strncmp" => {}
        "strncasecmp" => {
            left.make_ascii_lowercase();
            right.make_ascii_lowercase();
        }
        _ => return Err(EvalStatus::UnsupportedConstruct),
    }
    // php-src truncates each side to `$length` and compares what is left; it does not pad the
    // shorter one, so a prefix still sorts before the longer string it is a prefix of.
    let limit = usize::try_from(length).unwrap_or(usize::MAX);
    let left = &left[..left.len().min(limit)];
    let right = &right[..right.len().min(limit)];
    // THE MAGNITUDE IS OBSERVABLE. `zend_binary_strncmp` returns `memcmp`'s value and falls back
    // to the length difference, so php prints the BYTE DIFFERENCE, not a normalized -1/0/1:
    // `strncmp('ns/x', '@', 1)` is 46 (`'n'` 110 minus `'@'` 64), and `strcmp('Abc', 'abc')` is
    // -32. Measured with `php -n` 8.5; a sign-only result is a different function.
    let result = left
        .iter()
        .zip(right.iter())
        .find(|(a, b)| a != b)
        .map(|(a, b)| i64::from(*a) - i64::from(*b))
        .unwrap_or_else(|| left.len() as i64 - right.len() as i64);
    values.int(result)
}
