//! Purpose:
//! Declarative eval registry entry for `file_put_contents`.
//!
//! Called from:
//! - `crate::interpreter::builtins::filesystem`.
//!
//! Key details:
//! - Runtime dispatch is declared here and delegated through the one-shot file write helper.
//! - `$flags` honours `FILE_APPEND` and `LOCK_EX` on local files (Symfony's `SodiumVault`
//!   writes its keys with `LOCK_EX`); `$context` is accepted and ignored.
//! - Array `$data` is written as its values concatenated, as php does.

eval_builtin! {
    contract: "file_put_contents",
    area: Filesystem,
    direct: Filesystem,
    values: Filesystem,
}

use super::super::super::*;
use super::*;
use crate::stream_wrappers;
use std::io::Write;

/// PHP's `FILE_APPEND` flag.
const FILE_APPEND: i64 = 8;
/// PHP's `LOCK_EX` flag.
const LOCK_EX: i64 = 2;

/// Dispatches direct eval calls for the `file_put_contents` filesystem builtin through the area dispatcher.
pub(in crate::interpreter) fn eval_file_put_contents_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_file_put_contents(args, context, scope, values)
}

/// Dispatches evaluated-argument calls for the `file_put_contents` filesystem builtin through the area dispatcher.
pub(in crate::interpreter) fn eval_file_put_contents_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match evaluated_args {
        [filename, data] => eval_file_put_contents_result(*filename, *data, 0, context, values),
        [filename, data, flags] | [filename, data, flags, _] => {
            let flags = eval_int_value(*flags, values)?;
            eval_file_put_contents_result(*filename, *data, flags, context, values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Evaluates PHP `file_put_contents($filename, $data, $flags = 0, $context = null)`.
pub(in crate::interpreter) fn eval_builtin_file_put_contents(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !(2..=4).contains(&args.len()) {
        return Err(EvalStatus::RuntimeFatal);
    }
    let mut evaluated = Vec::with_capacity(args.len());
    for arg in args {
        evaluated.push(eval_expr(arg, context, scope, values)?);
    }
    eval_file_put_contents_declared_values_result(&evaluated, context, values)
}

/// Writes a PHP string to a local file or supported wrapper and returns a byte count.
pub(in crate::interpreter) fn eval_file_put_contents_result(
    filename: RuntimeCellHandle,
    data: RuntimeCellHandle,
    flags: i64,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let path = eval_path_string(filename, values)?;
    let data = if values.is_array_like(data)? {
        let separator = values.string("")?;
        let joined = eval_implode_result(separator, data, values)?;
        values.string_bytes(joined)?
    } else {
        values.string_bytes(data)?
    };
    if stream_wrappers::is_phar_stream(&path) {
        return match elephc_phar::put_url_bytes(path.as_bytes(), &data) {
            Some(len) => values.int(i64::try_from(len).map_err(|_| EvalStatus::RuntimeFatal)?),
            None => values.bool_value(false),
        };
    }
    if let Some(result) =
        eval_user_wrapper_file_put_contents_result(&path, &data, context, values)?
    {
        return Ok(result);
    }
    let Some(path) = stream_wrappers::local_filesystem_path(&path) else {
        return values.bool_value(false);
    };
    match write_local_file(path.as_ref(), &data, flags) {
        Ok(()) => values.int(i64::try_from(data.len()).map_err(|_| EvalStatus::RuntimeFatal)?),
        Err(_) => values.bool_value(false),
    }
}

/// Writes `data` to a local path with php's `FILE_APPEND` / `LOCK_EX` semantics: the file is
/// opened without truncation, locked when asked, and only then truncated (unless appending), so
/// a concurrent reader never sees the file emptied before the lock is held.
fn write_local_file(path: &std::path::Path, data: &[u8], flags: i64) -> std::io::Result<()> {
    let append = flags & FILE_APPEND != 0;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(append)
        .open(path)?;
    if flags & LOCK_EX != 0 {
        use std::os::fd::AsRawFd;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    if !append {
        file.set_len(0)?;
    }
    file.write_all(data)
}
