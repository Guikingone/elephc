//! Purpose:
//! Implements tokenizer builtins in runtime eval using the shared PHP lexer/parser.
//!
//! Called from:
//! - Declarative direct and evaluated-argument hooks.
//!
//! Key details:
//! - Runtime arrays own their text and nested token tuples through EvalArrayBuilder.
//! - Parse errors leave through the ordinary catchable Throwable channel.

use super::super::*;
use super::collection_builder::EvalArrayBuilder;
mod token_get_all;
mod token_name;

/// Evaluates source arguments in order before dispatching one tokenizer builtin.
pub(in crate::interpreter) fn eval_builtin_tokenizer(
    name: &str, args: &[EvalExpr], context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope, values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let operands = args.iter().collect::<Vec<_>>();
    with_eval_operands(&operands, context, scope, values, |args, context, _scope, values| {
        eval_tokenizer_values(name, args, context, values)
    })
}

/// Materializes PHP's array-or-character stream from the shared engine's token offsets.
pub(in crate::interpreter) fn eval_tokenizer_values(
    name: &str, args: &[RuntimeCellHandle], context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if name == "token_name" {
        let [value] = args else { return Err(EvalStatus::RuntimeFatal); };
        let id = tokenizer_int(*value, name, 1, "id", context, values)?;
        return values.string(elephc_tokenizer_engine::token_name(id).unwrap_or("UNKNOWN"));
    }
    let Some(code) = args.first().copied() else { return Err(EvalStatus::RuntimeFatal); };
    let tag = values.type_tag(code)?;
    if matches!(tag, EVAL_TAG_ARRAY | EVAL_TAG_ASSOC | EVAL_TAG_RESOURCE | EVAL_TAG_CALLABLE)
        || tag == EVAL_TAG_OBJECT && !eval_object_has_to_string(code, context, values)? {
        let given = eval_given_type_name(code, context, values)?;
        return eval_throw_type_error(&format!("token_get_all(): Argument #1 ($code) must be of type string, {given} given"), context, values);
    }
    let code = if tag == EVAL_TAG_OBJECT {
        let text = eval_string_context_value(code, context, values)?;
        let result = values.string_bytes(text);
        let released = values.release(text);
        let bytes = result?;
        released?;
        bytes
    } else { values.string_bytes(code)? };
    let flags = match args.get(1) {
        Some(value) => tokenizer_int(*value, name, 2, "flags", context, values)?, None => 0,
    };
    let tokens = match elephc_tokenizer_engine::tokenize(&code, flags) {
        Ok(tokens) => tokens,
        Err(error) => return eval_throw_builtin_exception("ParseError", &error.message, context, values),
    };
    let mut out = EvalArrayBuilder::indexed(values, tokens.len())?;
    for (index, token) in tokens.iter().enumerate() {
        let text = &code[token.pos..token.pos + token.len];
        out.index(index, |values| {
            if token.id < 256 { return values.string_bytes_value(text); }
            let mut row = EvalArrayBuilder::indexed(values, 3)?;
            row.index(0, |values| values.int(token.id))?;
            row.index(1, |values| values.string_bytes_value(text))?;
            row.index(2, |values| values.int(token.line as i64))?;
            Ok(row.finish())
        })?;
    }
    Ok(out.finish())
}

/// Converts an internal integer parameter, rejecting containers and nonnumeric strings.
fn tokenizer_int(
    value: RuntimeCellHandle, name: &str, number: usize, parameter: &str,
    context: &mut ElephcEvalContext, values: &mut impl RuntimeValueOps,
) -> Result<i64, EvalStatus> {
    let tag = values.type_tag(value)?;
    let invalid = matches!(tag, EVAL_TAG_ARRAY | EVAL_TAG_ASSOC | EVAL_TAG_OBJECT | EVAL_TAG_RESOURCE | EVAL_TAG_CALLABLE)
        || tag == EVAL_TAG_STRING && {
            let bytes = values.string_bytes(value)?;
            std::str::from_utf8(&bytes).ok().and_then(|s| s.trim().parse::<f64>().ok()).is_none_or(|n| !n.is_finite())
        };
    if invalid {
        let given = eval_given_type_name(value, context, values)?;
        return eval_throw_type_error(&format!("{name}(): Argument #{number} (${parameter}) must be of type int, {given} given"), context, values);
    }
    eval_int_value(value, values)
}

/// Enforces PhpToken::tokenize's construction precondition for eval-declared subclasses.
pub(in crate::interpreter) fn eval_tokenizer_static_class_precondition(
    owner: &str, method: &str, called_class: &str,
    context: &mut ElephcEvalContext, values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if owner.trim_start_matches('\\').eq_ignore_ascii_case("PhpToken")
        && method.eq_ignore_ascii_case("tokenize")
        && context.class(called_class).is_some_and(|class| class.is_abstract()) {
        return eval_throw_error(&format!("Cannot instantiate abstract class {called_class}"), context, values);
    }
    Ok(())
}
