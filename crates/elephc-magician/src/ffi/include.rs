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
    let _ = std::panic::catch_unwind(crate::context::reset_global_eval_included_files);
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
    .unwrap_or_else(|_| EvalStatus::RuntimeFatal.code())
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
    match interpreter::execute_include_outcome_with_context(
        context,
        scope,
        path,
        required,
        once,
        &mut values,
    ) {
        Ok(outcome) => write_outcome(outcome, out).code(),
        Err(status) => status.code(),
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
