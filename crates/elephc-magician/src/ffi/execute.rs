//! Purpose:
//! Exports eval fragment execution through the optional bridge.
//! This layer validates ABI pointers, parses fragment bytes, and dispatches
//! parsed EvalIR to the interpreter with runtime hooks in production builds.
//!
//! Called from:
//! - Generated EIR backend assembly through `__elephc_eval_execute`.
//!
//! Key details:
//! - Tests keep a controlled unsupported stub because generated runtime wrappers
//!   are not linked into the crate unit-test binary.

use super::util::clear_result;
#[cfg(not(test))]
use super::util::write_outcome;
use crate::abi::{ElephcEvalContext, ElephcEvalResult, ElephcEvalScope, ABI_VERSION};
use crate::errors::{EvalParseError, EvalStatus};
use crate::eval_ir;
#[cfg(not(test))]
use crate::interpreter;
use crate::parse_cache;
#[cfg(not(test))]
use crate::runtime_hooks::ElephcRuntimeOps;
use std::slice;

const EVAL_TRACE_ENV: &str = "ELEPHC_EVAL_TRACE";

/// Executes an eval fragment against a materialized caller scope.
///
/// The FFI shape is final for the initial bridge: context/scope are opaque
/// runtime handles, `code_ptr`/`code_len` identify the PHP fragment bytes, and
/// `out` receives the eval return cell when provided. Non-test builds execute
/// the current EvalIR subset; test builds return `UnsupportedConstruct` because
/// they do not link elephc's generated runtime value wrappers.
///
/// # Safety
/// Callers must pass valid pointers for any non-null handle and ensure
/// `code_ptr` is readable for `code_len` bytes when `code_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_execute(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
    code_ptr: *const u8,
    code_len: u64,
    out: *mut ElephcEvalResult,
) -> i32 {
    std::panic::catch_unwind(|| unsafe { execute_eval_inner(ctx, scope, code_ptr, code_len, out) })
        .unwrap_or_else(|_| EvalStatus::RuntimeFatal.code())
}

/// Runs the eval ABI body after the exported wrapper has installed a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_execute`; callers must provide valid handles and code
/// storage for every non-null pointer argument.
unsafe fn execute_eval_inner(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
    code_ptr: *const u8,
    code_len: u64,
    out: *mut ElephcEvalResult,
) -> i32 {
    if !ctx.is_null() && (*ctx).abi_version() != ABI_VERSION {
        return EvalStatus::AbiMismatch.code();
    }
    if code_len > 0 && code_ptr.is_null() {
        return EvalStatus::RuntimeFatal.code();
    }
    let Ok(code_len) = usize::try_from(code_len) else {
        return EvalStatus::RuntimeFatal.code();
    };
    let code = if code_len == 0 {
        &[]
    } else {
        slice::from_raw_parts(code_ptr, code_len)
    };
    trace_eval_input(ctx, code);
    let program = match parse_cache::parse_fragment_cached(code) {
        Ok(program) => program,
        Err(err) => {
            let status = err.clone().status().code();
            trace_eval_parse_error(ctx, code, &err, status);
            return status;
        }
    };
    clear_result(out);
    execute_parsed_eval(ctx, scope, program.as_ref(), out)
}

/// Returns whether opt-in eval bridge tracing is enabled for this process.
fn eval_trace_enabled() -> bool {
    std::env::var_os(EVAL_TRACE_ENV).is_some()
}

/// Emits the exact eval input and propagated call-site metadata before cache lookup.
///
/// # Safety
/// `ctx` must be null or a valid eval context handle supplied to the FFI entry point.
unsafe fn trace_eval_input(ctx: *const ElephcEvalContext, code: &[u8]) {
    if !eval_trace_enabled() {
        return;
    }
    let _ = std::panic::catch_unwind(|| {
        let (file, dir, line, file_override) = unsafe { ctx.as_ref() }
            .map(ElephcEvalContext::call_site)
            .unwrap_or_else(|| (String::new(), String::new(), 0, None));
        eprintln!(
            "[elephc-eval-trace] phase=input ctx={ctx:p} code_ptr={:p} len={} file={file:?} dir={dir:?} line={line} file_override={file_override:?}",
            code.as_ptr(),
            code.len(),
        );
    });
}

/// Emits the precise parser failure while preserving the original ABI status.
///
/// # Safety
/// `ctx` must be null or a valid eval context handle supplied to the FFI entry point.
unsafe fn trace_eval_parse_error(
    ctx: *const ElephcEvalContext,
    code: &[u8],
    error: &EvalParseError,
    status: i32,
) {
    if !eval_trace_enabled() {
        return;
    }
    let _ = std::panic::catch_unwind(|| {
        let (file, dir, line, file_override) = unsafe { ctx.as_ref() }
            .map(ElephcEvalContext::call_site)
            .unwrap_or_else(|| (String::new(), String::new(), 0, None));
        eprintln!(
            "[elephc-eval-trace] phase=parse_error ctx={ctx:p} code_ptr={:p} len={} status={status} error={error:?} file={file:?} dir={dir:?} line={line} file_override={file_override:?}",
            code.as_ptr(),
            code.len(),
        );
    });
}

/// Executes a parsed eval program in production builds using elephc runtime hooks.
///
/// # Safety
/// `scope` and `out` must be null or valid pointers supplied by generated code.
#[cfg(not(test))]
unsafe fn execute_parsed_eval(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
    program: &eval_ir::EvalProgram,
    out: *mut ElephcEvalResult,
) -> i32 {
    let mut fallback_context;
    let context = if let Some(ctx) = ctx.as_mut() {
        ctx
    } else {
        fallback_context = ElephcEvalContext::new();
        &mut fallback_context
    };
    let mut fallback_scope;
    let scope = if let Some(scope) = scope.as_mut() {
        scope
    } else {
        fallback_scope = ElephcEvalScope::new();
        &mut fallback_scope
    };
    context.sync_global_eval_classes();
    let mut values = ElephcRuntimeOps::with_context(context as *const ElephcEvalContext);
    match interpreter::execute_program_outcome_with_context(context, program, scope, &mut values) {
        Ok(outcome) => write_outcome(outcome, out).code(),
        Err(status) => {
            if eval_trace_enabled() {
                let call_site = context.call_site();
                eprintln!(
                    "[elephc-eval-trace] phase=runtime_error status={status:?} file={:?} line={}",
                    call_site.0,
                    call_site.2,
                );
            }
            status.code()
        }
    }
}

/// Keeps crate unit tests independent from generated runtime assembly wrappers.
///
/// # Safety
/// `out` must be null or valid result storage supplied by the test caller.
#[cfg(test)]
unsafe fn execute_parsed_eval(
    _ctx: *mut ElephcEvalContext,
    _scope: *mut ElephcEvalScope,
    _program: &eval_ir::EvalProgram,
    _out: *mut ElephcEvalResult,
) -> i32 {
    EvalStatus::UnsupportedConstruct.code()
}
