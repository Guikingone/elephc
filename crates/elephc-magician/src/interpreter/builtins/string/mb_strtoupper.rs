//! Purpose:
//! Declarative eval registry entry for PHP's `mb_strtoupper()`, plus the glue the
//! mbstring case pair shares.
//!
//! Called from:
//! - `crate::interpreter::builtins::string` and the declarative direct/values hooks, which
//!   route the pair through the `Iconv` hook group because the engine lives in `elephc-iconv`.
//!
//! Key details:
//! - The conversion is `elephc_iconv::convert_case`, the same code the AOT bridge runs, so
//!   both backends produce identical bytes and identical `ValueError` messages.
//! - An omitted or `null` `$encoding` stays absent and selects UTF-8.
//! - A refused `$encoding` raises a catchable `ValueError` through eval's pending-throw state.

eval_builtin! {
    contract: "mb_strtoupper",
    area: String,
    direct: Iconv,
    values: Iconv,
}

use elephc_iconv::CaseMode;

use super::super::super::*;
use super::iconv::eval_iconv_charset;

/// Applies `mb_strtoupper()` or `mb_strtolower()` to already evaluated arguments.
pub(in crate::interpreter) fn eval_mb_convert_case_result(
    mode: CaseMode,
    subject: RuntimeCellHandle,
    encoding: Option<RuntimeCellHandle>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let subject = values.string_bytes(subject)?;
    let encoding = eval_iconv_charset(encoding, values)?;
    match elephc_iconv::convert_case(mode, &subject, encoding.as_deref()) {
        Ok(bytes) => values.string_bytes_value(&bytes),
        Err(error) => {
            let exception = values.new_object("ValueError")?;
            let message = values.string_bytes_value(&error.value_error_message(mode.function_name()))?;
            let code = values.int(0)?;
            values.construct_object(exception, vec![message, code])?;
            context.set_pending_throw(exception);
            Err(EvalStatus::UncaughtThrowable)
        }
    }
}
