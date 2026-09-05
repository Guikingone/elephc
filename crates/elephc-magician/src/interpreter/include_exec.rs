//! Purpose:
//! Executes nested `eval(...)`, include, include_once, require, and require_once expressions.
//! This keeps source-file loading and PHP open/close-tag handling outside the core interpreter loop.
//!
//! Called from:
//! - `crate::interpreter::eval_positional_expr_call()` for `eval(...)`.
//! - `crate::interpreter::eval_expr()` for include/require expression nodes.
//!
//! Key details:
//! - Included code runs against the current eval context and materialized scope.
//! - Missing include emits a warning and returns false; missing require is fatal.

use super::*;
use crate::parse_cache::parse_fragment_cached;

const EVAL_TRACE_ENV: &str = "ELEPHC_EVAL_TRACE";

/// Evaluates nested `eval(...)` calls against the current materialized scope.
pub(super) fn eval_nested_eval(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [code] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let code = eval_expr(code, context, scope, values)?;
    let code = values.string_bytes(code)?;
    let program = parse_fragment_cached(&code).map_err(|diagnostic| {
        let (file, _, line, _) = context.call_site();
        report_fatal_diagnostic(&diagnostic.eval_message(&file, line));
        diagnostic.status()
    })?;
    context.push_include_execution(false);
    let result = execute_program_with_context(context, program.as_ref(), scope, values);
    context.pop_include_execution();
    result
}

/// Evaluates an eval-fragment include or require expression.
pub(super) fn eval_include_expr(
    path: &EvalExpr,
    required: bool,
    once: bool,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let path = eval_expr(path, context, scope, values)?;
    eval_include_value(path, required, once, context, scope, values)
}

/// Evaluates an already materialized include path cell against the current caller scope.
pub(super) fn eval_include_value(
    path: RuntimeCellHandle,
    required: bool,
    once: bool,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let path = eval_path_string(path, values)?;
    let resolved_path = eval_resolve_include_path(&path, context);
    let include_key = eval_include_key(&resolved_path);
    if once && context.has_included_file(&include_key) {
        return values.bool_value(true);
    }
    let bytes = match std::fs::read(&resolved_path) {
        Ok(bytes) => bytes,
        Err(_) => return eval_include_missing_file(&path, required, values),
    };
    context.mark_included_file(include_key);
    eval_execute_include_bytes(&bytes, &resolved_path, context, scope, values)
}

/// Returns the include/require result for a file that cannot be opened.
fn eval_include_missing_file(
    path: &str,
    required: bool,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let construct = if required { "require" } else { "include" };
    values.warning(&format!(
        "Warning: {construct}({path}): Failed to open stream: No such file or directory\n"
    ))?;
    values.warning(&format!(
        "Warning: {construct}(): Failed opening '{path}' for inclusion\n"
    ))?;
    if required {
        Err(EvalStatus::RuntimeFatal)
    } else {
        values.bool_value(false)
    }
}

/// Resolves eval include paths using PHP's cwd-first and caller-directory fallback.
fn eval_resolve_include_path(path: &str, context: &ElephcEvalContext) -> std::path::PathBuf {
    let raw_path = std::path::Path::new(path);
    if raw_path.is_absolute() || raw_path.exists() {
        return raw_path.to_path_buf();
    }
    if context.call_dir().is_empty() {
        return raw_path.to_path_buf();
    }
    let caller_path = std::path::Path::new(context.call_dir()).join(raw_path);
    if caller_path.exists() {
        caller_path
    } else {
        raw_path.to_path_buf()
    }
}

/// Builds the stable include_once key for a resolved path.
fn eval_include_key(path: &std::path::Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

/// Executes a local include file, alternating raw output and PHP code blocks.
fn eval_execute_include_bytes(
    bytes: &[u8],
    path: &std::path::Path,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut cursor = 0;
    while let Some((tag_start, code_start)) = eval_find_php_open_tag(bytes, cursor) {
        eval_echo_include_bytes(&bytes[cursor..tag_start], values)?;
        let close = crate::lexer::find_php_close_tag(bytes, code_start);
        let code_end = close.unwrap_or(bytes.len());
        let line_offset = eval_include_line_offset(bytes, code_start);
        match eval_execute_include_code(
            &bytes[code_start..code_end],
            path,
            line_offset,
            context,
            scope,
            values,
        )? {
            EvalControl::None => {}
            EvalControl::ReturnVoid => return values.null(),
            EvalControl::Return(value) => return Ok(value),
            EvalControl::Throw(value) => {
                context.set_pending_throw(value);
                return Err(EvalStatus::UncaughtThrowable);
            }
            EvalControl::Break(_) | EvalControl::Continue(_) | EvalControl::Goto(_) => {
                return Err(EvalStatus::UnsupportedConstruct);
            }
        }
        let Some(close) = close else {
            return values.int(1);
        };
        cursor = close + 2;
    }
    eval_echo_include_bytes(&bytes[cursor..], values)?;
    values.int(1)
}

/// Returns how many lines of an included file precede one PHP code block.
///
/// Parsing sees only the bytes after `<?php`, so a fragment line equals the file line only for
/// the block that opens the file. Every later block, and any block that follows inline HTML,
/// needs the newlines before it added back before a diagnostic can name PHP's file line.
fn eval_include_line_offset(bytes: &[u8], code_start: usize) -> i64 {
    i64::try_from(
        bytes[..code_start.min(bytes.len())]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count(),
    )
    .unwrap_or(0)
}

/// Parses and executes one PHP code block from an included file.
fn eval_execute_include_code(
    code: &[u8],
    path: &std::path::Path,
    line_offset: i64,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    trace_include_fragment("input", code, path, context, None);
    let program = parse_fragment_cached(code).map_err(|diagnostic| {
        trace_include_fragment("parse_error", code, path, context, Some(&diagnostic));
        report_fatal_diagnostic(
            &diagnostic
                .with_line_offset(line_offset)
                .include_message(&path.to_string_lossy()),
        );
        diagnostic.status()
    })?;
    let previous = context.call_site();
    let file = path.to_string_lossy().into_owned();
    let dir = path
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();
    context.set_call_site(file.clone(), dir, 1);
    context.set_file_magic_override(Some(file));
    context.push_include_execution(true);
    let result = execute_statements(program.statements(), context, scope, values);
    if let Err(ref status) = result {
        if std::env::var_os(EVAL_TRACE_ENV).is_some() {
            eprintln!(
                "[elephc-eval-trace] kind=include phase=execute_error path={path:?} status={status:?}"
            );
        }
    }
    context.pop_include_execution();
    context.set_call_site(previous.0, previous.1, previous.2);
    context.set_file_magic_override(previous.3);
    result
}

/// Emits an opt-in, panic-safe trace for the exact PHP block parsed from an included file.
fn trace_include_fragment(
    phase: &str,
    code: &[u8],
    path: &std::path::Path,
    context: &ElephcEvalContext,
    error: Option<&EvalParseDiagnostic>,
) {
    if std::env::var_os(EVAL_TRACE_ENV).is_none() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let caller = context.call_site();
        if error.is_some() {
            let escaped = code.escape_ascii().map(char::from).collect::<String>();
            eprintln!(
                "[elephc-eval-trace] kind=include phase={phase} path={:?} len={} error={error:?} caller_file={:?} caller_dir={:?} caller_line={} caller_override={:?} bytes={escaped}",
                path,
                code.len(),
                caller.0,
                caller.1,
                caller.2,
                caller.3,
            );
        } else {
            eprintln!(
                "[elephc-eval-trace] kind=include phase={phase} path={:?} len={} caller_file={:?} caller_line={}",
                path,
                code.len(),
                caller.0,
                caller.2,
            );
        }
    }));
}

/// Echoes raw non-PHP include bytes through the eval value hooks.
fn eval_echo_include_bytes(
    bytes: &[u8],
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if bytes.is_empty() {
        return Ok(());
    }
    let output = values.string_bytes_value(bytes)?;
    values.echo(output)
}

/// Finds the next `<?php` opening tag and returns tag and code byte offsets.
fn eval_find_php_open_tag(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    bytes
        .get(start..)?
        .windows(5)
        .position(eval_is_php_open_tag)
        .map(|offset| {
            let tag_start = start + offset;
            (tag_start, tag_start + 5)
        })
}

/// Returns true when a five-byte window is a case-insensitive `<?php` tag.
fn eval_is_php_open_tag(window: &[u8]) -> bool {
    window.len() == 5
        && window[0] == b'<'
        && window[1] == b'?'
        && window[2].eq_ignore_ascii_case(&b'p')
        && window[3].eq_ignore_ascii_case(&b'h')
        && window[4].eq_ignore_ascii_case(&b'p')
}
