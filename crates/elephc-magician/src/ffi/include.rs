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

#[cfg(test)]
mod native_state_registration_tests {
    use super::*;
    thread_local! { static CELL: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }

    unsafe extern "C" fn lookup(path: *const u8, length: u64) -> *mut u64 {
        if unsafe { std::slice::from_raw_parts(path, length as usize) } == b"/native.php" {
            CELL.with(std::cell::Cell::as_ptr)
        } else { std::ptr::null_mut() }
    }
    unsafe extern "C" fn other_lookup(_: *const u8, _: u64) -> *mut u64 { std::ptr::null_mut() }
    unsafe extern "C" fn reset() { CELL.with(|cell| cell.set(0)); }

    #[test]
    fn native_include_registration_uses_the_context_request_state() {
        let mut context = ElephcEvalContext::new();
        unsafe {
            reset();
            assert_eq!(__elephc_eval_register_native_include_state(&mut context, Some(lookup), Some(reset)), 0);
        }
        context.mark_included_file("/native.php");
        CELL.with(|cell| assert_eq!(cell.get(), 1));
        unsafe {
            assert_eq!(__elephc_eval_register_native_include_state(&mut context, Some(lookup), Some(reset)), 0);
            assert_eq!(__elephc_eval_register_native_include_state(&mut context, Some(other_lookup), Some(reset)), EvalStatus::RuntimeFatal.code());
            reset();
        }
        assert!(!context.has_included_file("/native.php"));
        context.mark_included_file("/native.php");
        CELL.with(|cell| assert_eq!(cell.get(), 1));
        unsafe { reset(); }
    }

    #[test]
    fn native_include_registration_rejects_missing_callbacks_and_bad_abi() {
        let mut context = ElephcEvalContext::new();
        let mut bad = ElephcEvalContext::for_abi_version(ABI_VERSION + 1);
        unsafe {
            assert_eq!(__elephc_eval_register_native_include_state(&mut context, None, Some(reset)), EvalStatus::RuntimeFatal.code());
            assert_eq!(__elephc_eval_register_native_include_state(&mut bad, Some(lookup), Some(reset)), EvalStatus::AbiMismatch.code());
        }
    }
}

/// Connects inclusion state to the generated program's actual native guard cells.
/// This registers state access only, not a compiled source provider or declaration.
///
/// # Safety
/// Context must be null or live. Callbacks must remain valid for the program's
/// lifetime, must not execute PHP/reenter, and lookup must return null or an aligned
/// writable u64 cell. All native/bridge cell access must be serialized by the caller.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_register_native_include_state(
    ctx: *mut ElephcEvalContext,
    lookup: Option<unsafe extern "C" fn(*const u8, u64) -> *mut u64>,
    reset: Option<unsafe extern "C" fn()>,
) -> i32 {
    std::panic::catch_unwind(|| {
        let context = unsafe { ctx.as_ref() };
        if context.is_some_and(|context| context.abi_version() != ABI_VERSION) {
            return EvalStatus::AbiMismatch.code();
        }
        let (Some(lookup), Some(reset)) = (lookup, reset) else {
            return EvalStatus::RuntimeFatal.code();
        };
        if unsafe { crate::context::install_native_include_hooks(context, crate::context::NativeIncludeHooks { lookup, reset }) } {
            EvalStatus::Ok.code()
        } else {
            eprintln!("Fatal error: conflicting native inclusion-state owner");
            EvalStatus::RuntimeFatal.code()
        }
    }).unwrap_or(EvalStatus::RuntimeFatal.code())
}

/// Clears dynamic include bookkeeping before a generated web request starts.
///
/// CLI binaries never call this entry point, so their include registry naturally
/// lasts for the single PHP request represented by the process lifetime.
#[no_mangle]
pub extern "C" fn __elephc_eval_include_request_reset() {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_include_request_reset");
    let _ = std::panic::catch_unwind(|| {
        crate::context::reset_global_eval_function_contexts();
        crate::context::reset_global_eval_autoload_contexts();
        #[cfg(not(test))]
        crate::context::reset_global_eval_classes();
        // Context cleanup may execute PHP destructors, which must still observe
        // the current request's inclusion state until cleanup has completed.
        crate::context::reset_global_eval_included_files();
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_include");
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
