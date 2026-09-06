//! Purpose:
//! Exports runtime include/require execution through the optional bridge.
//! Accepts an already boxed PHP path value and executes the selected file in
//! the caller's persistent eval scope.
//!
//! Called from:
//! - Generated EIR backend assembly through `__elephc_eval_include`.
//! - The generated web reset through `__elephc_eval_include_request_reset`.
//!
//! Key details:
//! - Include values, warnings, request-scoped once tracking, and Throwable propagation stay in Magician.
//! - The path cell is borrowed for the duration of the call.

use super::util::clear_result;
#[cfg(not(test))]
use super::util::write_outcome;
use crate::abi::{ElephcEvalContext, ElephcEvalResult, ElephcEvalScope, ABI_VERSION};
#[cfg(not(test))]
use crate::context::EvalCallFrame;
use crate::errors::EvalStatus;
#[cfg(not(test))]
use crate::interpreter;
#[cfg(not(test))]
use crate::runtime_hooks::ElephcRuntimeOps;
use crate::value::RuntimeCellHandle;
use std::ffi::c_void;

/// Clears dynamic include bookkeeping before a generated web request starts.
///
/// CLI binaries never call this entry point, so their include registry naturally
/// lasts for the single PHP request represented by the process lifetime.
#[no_mangle]
pub extern "C" fn __elephc_eval_include_request_reset() {
    let _ = std::panic::catch_unwind(|| {
        crate::context::reset_global_eval_included_files();
        crate::context::reset_global_eval_function_contexts();
        crate::context::reset_global_eval_autoload_contexts();
    });
}

/// Executes one runtime include/require against a materialized caller scope.
///
/// # Safety
/// Context, scope, path, and result pointers must be null or valid for their ABI roles;
/// a non-null path must identify a live boxed PHP runtime cell for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_include(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
    path: *mut c_void,
    required: u64,
    once: u64,
    out: *mut ElephcEvalResult,
) -> i32 {
    std::panic::catch_unwind(|| unsafe {
        execute_include_inner(ctx, scope, path, required != 0, once != 0, out)
    })
    .unwrap_or_else(|_| {
        if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
            eprintln!("[elephc-eval-trace] phase=include_panic status=RuntimeFatal");
        }
        EvalStatus::RuntimeFatal.code()
    })
}

/// Runs the include ABI body after the exported wrapper installs a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_include`; every non-null pointer must remain valid for this call.
unsafe fn execute_include_inner(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
    path: *mut c_void,
    required: bool,
    once: bool,
    out: *mut ElephcEvalResult,
) -> i32 {
    if !ctx.is_null() && (*ctx).abi_version() != ABI_VERSION {
        return EvalStatus::AbiMismatch.code();
    }
    if path.is_null() {
        return EvalStatus::RuntimeFatal.code();
    }
    clear_result(out);
    execute_materialized_include(ctx, scope, RuntimeCellHandle::from_raw(path), required, once, out)
}

/// Returns the PHP name of the include form, which is the frame's `function`.
#[cfg(not(test))]
fn eval_include_frame_name(required: bool, once: bool) -> &'static str {
    match (required, once) {
        (true, true) => "require_once",
        (true, false) => "require",
        (false, true) => "include_once",
        (false, false) => "include",
    }
}

/// Executes the include in production builds through elephc runtime value hooks.
///
/// # Safety
/// Scope and result pointers must be null or point to bridge-owned ABI storage.
#[cfg(not(test))]
unsafe fn execute_materialized_include(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
    path: RuntimeCellHandle,
    required: bool,
    once: bool,
    out: *mut ElephcEvalResult,
) -> i32 {
    let mut fallback_context;
    let context = if let Some(ctx) = ctx.as_mut() {
        ctx
    } else {
        fallback_context = ElephcEvalContext::new();
        crate::context::sync_global_eval_aot_metadata(&mut fallback_context);
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
    // php describes an include as a frame of its own, named for the form that was written and
    // carrying the path as its single argument, so code in the included file sees the include
    // above itself. The path cell belongs to the caller for the whole call and a frame does not
    // retain its arguments, so it can be named here without a copy.
    let frame = EvalCallFrame::function(
        eval_include_frame_name(required, once),
        Some(vec![path]),
        context,
    );
    context.push_call_frame(frame);
    let outcome = interpreter::execute_include_outcome_with_context(
        context,
        scope,
        path,
        required,
        once,
        &mut values,
    );
    context.pop_call_frame();
    match outcome {
        Ok(outcome) => {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                let call_site = context.call_site();
                eprintln!(
                    "[elephc-eval-trace] phase=include_ok file={:?} line={}",
                    call_site.0, call_site.2,
                );
            }
            write_outcome(outcome, out).code()
        }
        Err(status) => {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                let call_site = context.call_site();
                eprintln!(
                    "[elephc-eval-trace] phase=include_error status={status:?} file={:?} line={}",
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
/// Result storage must be null or valid for the duration of the call.
#[cfg(test)]
unsafe fn execute_materialized_include(
    _ctx: *mut ElephcEvalContext,
    _scope: *mut ElephcEvalScope,
    _path: RuntimeCellHandle,
    _required: bool,
    _once: bool,
    _out: *mut ElephcEvalResult,
) -> i32 {
    EvalStatus::UnsupportedConstruct.code()
}
