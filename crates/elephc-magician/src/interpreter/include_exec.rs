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

use crate::context::EvalCallFrame;
use super::*;
use crate::parse_cache::{parse_fragment_cached, parse_source_file_at_path, parse_source_file_cached};


#[cfg(all(test, unix))]
mod source_identity_tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn include_keys_preserve_distinct_non_utf8_paths() {
        let a = std::path::PathBuf::from(std::ffi::OsString::from_vec(b"/nonexistent-source/\xff.php".to_vec()));
        let b = std::path::PathBuf::from(std::ffi::OsString::from_vec(b"/nonexistent-source/\xfe.php".to_vec()));
        assert_eq!(a.to_string_lossy(), b.to_string_lossy());
        let first = eval_include_key(&a);
        let second = eval_include_key(&b);
        assert_ne!(first, second, "physical identity must not use lossy display text");
        let mut context = ElephcEvalContext::new();
        context.mark_included_file(first.clone());
        assert!(context.has_included_file(&first));
        assert!(!context.has_included_file(&second));
    }
}

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
    // php describes `eval()` as a frame of its own wherever it is written. The bridge entry point
    // pushes one for an eval called from compiled code; an eval written INSIDE interpreted code
    // comes through here instead and pushed none, so it was missing from `debug_backtrace()` and
    // from the declaring-file question a class declared inside it has to answer.
    let frame = EvalCallFrame::function("eval", None, context);
    context.push_call_frame(frame);
    context.push_include_execution(false);
    // Code inside `eval()` is not IN the file that called it, and php spells that:
    // `FILE(LINE) : eval()'d code`. `eval_file_magic()` builds exactly that string, but only
    // when no `__FILE__` override is set -- and an include sets one, to its own path. Left in
    // place it made an `eval()` inside an included file report the include's plain path, so a
    // class declared there had the wrong `getFileName()`. Cleared for the eval and put back
    // after, because the including file's own `__FILE__` is still its path.
    let previous_file_magic = context.call_site().3;
    context.set_file_magic_override(None);
    let result = execute_program_with_context(context, program.as_ref(), scope, values);
    context.set_file_magic_override(previous_file_magic);
    context.pop_include_execution();
    context.pop_call_frame();
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
    // A file already parsed in this process, unchanged on disk, needs no read: the path cache
    // answers from one `stat`. A web worker re-includes the same vendor tree on every request, so
    // this is the difference between reading 76 files per request and stat-ing them. A cached
    // PARSE ERROR deliberately falls through to the read below, so the diagnostic still carries
    // the offending bytes.
    if let Some(Ok(program)) = parse_source_file_at_path(&resolved_path) {
        context.mark_included_file(include_key);
        trace_cached_include(&resolved_path, context);
        return eval_include_control_value(
            eval_execute_include_program(program, &resolved_path, context, scope, values)?,
            context,
            values,
        );
    }
    let bytes = match std::fs::read(&resolved_path) {
        Ok(bytes) => bytes,
        Err(_) => return eval_include_missing_file(&path, required, values),
    };
    context.mark_included_file(include_key);
    eval_execute_include_bytes(&bytes, &resolved_path, context, scope, values)
}

/// Emits the opt-in include trace for a file served from the parse cache.
fn trace_cached_include(path: &std::path::Path, context: &ElephcEvalContext) {
    if !crate::eval_trace::enabled() {
        return;
    }
    let caller = context.call_site();
    eprintln!(
        "[elephc-eval-trace] kind=include phase=input_cached path={path:?} caller_file={:?} caller_line={}",
        caller.0, caller.2,
    );
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
///
/// The key has to be the CANONICAL path: `include_once` dedupes on it and `get_included_files()`
/// reports it, so two spellings that reach one file must produce one key — and on macOS even a
/// lexically normal absolute path can cross a symlinked component, `/tmp` and `/var` being
/// symlinks into `/private`. php-src canonicalizes here too, for every include and not only the
/// `_once` forms, because a plain `include` adds its file to the same set a later `include_once`
/// consults.
///
/// It is also the single most expensive thing an interpreted include does. `realpath()` issues one
/// `getattrlist` per path component on macOS, the generated Symfony container fragments sit ten
/// components deep, and a `--web` worker re-includes the same tree on every request: 31% of the
/// samples inside request handling were this one call. `realpath_cache` is php-src's answer to
/// exactly that, so the memo behind this call is matching PHP rather than diverging from it.
fn eval_include_key(path: &std::path::Path) -> std::path::PathBuf {
    crate::realpath_cache::canonicalize_cached(path)
}

/// Executes a local include file as one program covering its inline HTML and every PHP block.
///
/// PHP compiles an included file in one pass: the text outside the tags is `T_INLINE_HTML` and
/// becomes an echo, and a closing tag is an implicit semicolon. Parsing each `<?php … ?>` block on
/// its own could never accept a `{` that one block opens and a later block closes, which is how
/// every template in `error-handler/Resources/views` is written.
fn eval_execute_include_bytes(
    bytes: &[u8],
    path: &std::path::Path,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let control = eval_execute_include_code(bytes, path, context, scope, values)?;
    eval_include_control_value(control, context, values)
}

/// Converts the control flow an included file ended with into `include`'s PHP-visible value.
fn eval_include_control_value(
    control: EvalControl,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match control {
        EvalControl::None => values.int(1),
        EvalControl::ReturnVoid => values.null(),
        EvalControl::Return(value) => Ok(value),
        EvalControl::Throw(value) => {
            context.set_pending_throw(value);
            Err(EvalStatus::UncaughtThrowable)
        }
        EvalControl::Break(_) | EvalControl::Continue(_) | EvalControl::Goto(_) => {
            Err(EvalStatus::UnsupportedConstruct)
        }
    }
}

/// Parses and executes one whole included PHP source file.
fn eval_execute_include_code(
    code: &[u8],
    path: &std::path::Path,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    trace_include_fragment("input", code, path, context, None);
    let program = parse_source_file_cached(code).map_err(|diagnostic| {
        trace_include_fragment("parse_error", code, path, context, Some(&diagnostic));
        report_fatal_diagnostic(&diagnostic.include_message(&path.to_string_lossy()));
        diagnostic.status()
    })?;
    eval_execute_include_program(program, path, context, scope, values)
}

/// Executes one already-parsed included PHP source file.
fn eval_execute_include_program(
    program: std::sync::Arc<crate::eval_ir::EvalProgram>,
    path: &std::path::Path,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    let previous = context.call_site();
    // An included file's `declare(strict_types=1)` governs that file and stops at its edge. Left
    // unrestored it made one strict vendor file turn the whole rest of the program strict, which
    // is most of Symfony's vendor tree.
    let previous_strict_types = context.strict_types();
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
        if crate::eval_trace::enabled() {
            eprintln!(
                "[elephc-eval-trace] kind=include phase=execute_error path={path:?} status={status:?}"
            );
        }
    }
    context.pop_include_execution();
    context.set_strict_types(previous_strict_types);
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
    if !crate::eval_trace::enabled() {
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
