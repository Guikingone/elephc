//! Purpose:
//! Exports native-to-eval callable dispatch and probes for callback values that
//! may reference eval-declared functions, methods, or objects. Generated code
//! uses this ABI when native descriptor metadata cannot answer dynamically.
//!
//! Called from:
//! - Generated EIR backend assembly through `__elephc_eval_callable_call_array`.
//! - Generated EIR backend assembly through `__elephc_eval_is_callable`.
//!
//! Key details:
//! - Callback and argument containers are boxed Mixed cells owned by generated
//!   code. Dispatch results and uncaught throwables are returned through
//!   `ElephcEvalResult`; probe failures fail closed as `false`.

use super::util::{clear_result, write_outcome};
use crate::abi::{ElephcEvalContext, ElephcEvalResult, ABI_VERSION};
use crate::errors::EvalStatus;
use crate::interpreter::{self, RuntimeValueOps};
use crate::runtime_hooks::ElephcRuntimeOps;
use crate::value::{RuntimeCell, RuntimeCellHandle};

/// Checks whether a callback value is callable in the eval context.
///
/// # Safety
/// `ctx` must be a valid eval context handle and `callback` must point at a
/// boxed runtime cell.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_is_callable(
    ctx: *mut ElephcEvalContext,
    callback: *mut RuntimeCell,
) -> i32 {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_is_callable");
    std::panic::catch_unwind(|| unsafe { eval_is_callable_inner(ctx, callback) }).unwrap_or(0)
}

/// Returns the live eval context that owns a PHP Closure object, or null for every other value.
///
/// # Safety
/// `callback` must be null or a live boxed runtime cell. The lookup borrows only the object
/// identity; a context remains alive after its AOT frame returns while this Closure is live.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_callable_owner_context(
    callback: *mut RuntimeCell,
) -> *mut ElephcEvalContext {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_callable_owner_context");
    std::panic::catch_unwind(|| unsafe { eval_callable_owner_context_inner(callback) })
        .unwrap_or(std::ptr::null_mut())
}

/// Returns the live eval context that declared a dynamic PHP function, or null when absent.
///
/// # Safety
/// `name_ptr` must be null or readable for `name_len` bytes. The context remains live for the
/// current request and is removed from this registry before a web worker recycles its heap.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_function_owner_context(
    name_ptr: *const u8,
    name_len: u64,
) -> *mut ElephcEvalContext {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_function_owner_context");
    std::panic::catch_unwind(|| unsafe { eval_function_owner_context_inner(name_ptr, name_len) })
        .unwrap_or(std::ptr::null_mut())
}

/// Registers an AOT callback in the persistent SPL autoload table of its eval context.
///
/// # Safety
/// `ctx` must be a valid bridge context and `callback` must be a live boxed runtime cell.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_register_spl_autoload(
    ctx: *mut ElephcEvalContext,
    callback: *mut RuntimeCell,
    prepend: i32,
) -> i32 {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_register_spl_autoload");
    std::panic::catch_unwind(|| unsafe {
        let (context, created_context) = if let Some(context) = ctx.as_mut() {
            (context, false)
        } else {
            let context = crate::ffi::context::__elephc_eval_context_new();
            let Some(context) = context.as_mut() else {
                return 0;
            };
            (context, true)
        };
        if context.abi_version() != ABI_VERSION || callback.is_null() {
            return 0;
        }
        if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
            eprintln!("[elephc-eval-trace] phase=aot_autoload_register stage=entered callback={callback:p} prepend={prepend}");
        }
        let mut values = ElephcRuntimeOps::with_context(context as *const ElephcEvalContext);
        let result = crate::interpreter::register_runtime_spl_autoload_callback(
            RuntimeCellHandle::from_raw(callback),
            prepend != 0,
            context,
            &mut values,
        );
        if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
            eprintln!("[elephc-eval-trace] phase=aot_autoload_register stage=registered success={}", result.is_ok());
        }
        if created_context {
            let context_ptr = context as *mut ElephcEvalContext;
            if context.request_retained_context_free() {
                crate::ffi::context::finalize_eval_context_free(context_ptr);
            }
        }
        result.map(|_| 1).unwrap_or(0)
    })
    .unwrap_or(0)
}

/// Dispatches a callback value with a PHP argument array through the eval context.
///
/// # Safety
/// `ctx` must be a valid eval context handle. `callback` and `arg_array` must
/// point at boxed runtime cells, and `out` may be null.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_callable_call_array(
    ctx: *mut ElephcEvalContext,
    callback: *mut RuntimeCell,
    arg_array: *mut RuntimeCell,
    out: *mut ElephcEvalResult,
) -> i32 {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_callable_call_array");
    std::panic::catch_unwind(|| unsafe {
        eval_callable_call_array_inner(ctx, callback, arg_array, out)
    })
    .unwrap_or_else(|_| EvalStatus::RuntimeFatal.code())
}

/// Runs the eval callable-probe ABI body after installing a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_is_callable`; invalid handles fail closed as false.
#[cfg(not(test))]
unsafe fn eval_is_callable_inner(
    ctx: *mut ElephcEvalContext,
    callback: *mut RuntimeCell,
) -> i32 {
    let Some(context) = ctx.as_mut() else {
        return 0;
    };
    if context.abi_version() != ABI_VERSION || callback.is_null() {
        return 0;
    }
    let mut values = ElephcRuntimeOps::with_context(context as *const ElephcEvalContext);
    match interpreter::execute_context_is_callable(
        context,
        RuntimeCellHandle::from_raw(callback),
        &mut values,
    ) {
        Ok(callable) => i32::from(callable),
        Err(_) => 0,
    }
}

/// Looks up a closure's owning context from its native object identity.
///
/// # Safety
/// Mirrors `__elephc_eval_callable_owner_context`; the caller owns the boxed callback cell for
/// the full duration of this lookup.
#[cfg(not(test))]
unsafe fn eval_callable_owner_context_inner(
    callback: *mut RuntimeCell,
) -> *mut ElephcEvalContext {
    if callback.is_null() {
        return std::ptr::null_mut();
    }
    let callback = RuntimeCellHandle::from_raw(callback);
    let mut values = ElephcRuntimeOps::with_context(std::ptr::null());
    let Ok(identity) = values.object_identity(callback) else {
        return std::ptr::null_mut();
    };
    let Some(context) = crate::ffi::dynamic_destructors::dynamic_object_owner_context(identity)
    else {
        return std::ptr::null_mut();
    };
    let Some(context_ref) = (unsafe { context.as_ref() }) else {
        return std::ptr::null_mut();
    };
    if context_ref.abi_version() != ABI_VERSION
        || context_ref.closure_object_target(identity).is_none()
    {
        return std::ptr::null_mut();
    }
    context
}

/// Resolves one PHP function name through the request-local dynamic declaration registry.
///
/// # Safety
/// Mirrors `__elephc_eval_function_owner_context`; the name slice remains borrowed only during
/// this lookup.
#[cfg(not(test))]
unsafe fn eval_function_owner_context_inner(
    name_ptr: *const u8,
    name_len: u64,
) -> *mut ElephcEvalContext {
    if name_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(name) = std::str::from_utf8(unsafe {
        std::slice::from_raw_parts(name_ptr, name_len as usize)
    }) else {
        return std::ptr::null_mut();
    };
    let Some(context) = crate::context::global_eval_function_owner_context(name) else {
        return std::ptr::null_mut();
    };
    let Some(context_ref) = (unsafe { context.as_ref() }) else {
        return std::ptr::null_mut();
    };
    (context_ref.abi_version() == ABI_VERSION).then_some(context).unwrap_or(std::ptr::null_mut())
}

/// Runs the eval callable-array ABI body after installing a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_callable_call_array`; callers must provide boxed callback and
/// argument-array cells. A null context requests a temporary request-global fallback.
#[cfg(not(test))]
unsafe fn eval_callable_call_array_inner(
    ctx: *mut ElephcEvalContext,
    callback: *mut RuntimeCell,
    arg_array: *mut RuntimeCell,
    out: *mut ElephcEvalResult,
) -> i32 {
    if callback.is_null() || arg_array.is_null() {
        return EvalStatus::RuntimeFatal.code();
    }
    let callback = RuntimeCellHandle::from_raw(callback);
    let caller_context = (!ctx.is_null()).then_some(ctx);
    let owner_context = eval_callable_owner_context(callback);
    let (context_ptr, created_context) = if let Some(context) = owner_context.or(caller_context) {
        (context, false)
    } else {
        let context = crate::ffi::context::__elephc_eval_context_new();
        if context.is_null() {
            return EvalStatus::RuntimeFatal.code();
        }
        (context, true)
    };
    let Some(context) = context_ptr.as_mut() else {
        return EvalStatus::RuntimeFatal.code();
    };
    if context.abi_version() != ABI_VERSION {
        if created_context {
            crate::ffi::context::finalize_eval_context_free(context_ptr);
        }
        return EvalStatus::AbiMismatch.code();
    }
    if created_context {
        crate::context::sync_global_eval_aot_metadata(context);
        context.sync_global_eval_classes();
    }
    let forwarded_scopes = caller_context
        .filter(|caller| !std::ptr::eq(*caller, context))
        .and_then(|caller| caller.as_ref())
        .map(|caller| {
            (
                caller.current_class_scope().map(str::to_string),
                caller.current_called_class_scope().map(str::to_string),
            )
        });
    if let Some((class_scope, called_class_scope)) = &forwarded_scopes {
        if let Some(class_scope) = class_scope {
            context.push_class_scope(class_scope.clone());
        }
        if let Some(called_class_scope) = called_class_scope {
            context.push_called_class_scope(called_class_scope.clone());
        }
    }
    // Every caller of this ABI is generated native code. Refresh the captured
    // scope before interpreting the callback, then publish its changes before
    // returning either a value or an error to the native caller.
    let global_sync = context.native_global_sync.zip(context.global_scope_ptr());
    if let Some((hooks, scope)) = global_sync {
        (hooks.native_to_eval)(scope);
    }
    clear_result(out);
    let mut values = ElephcRuntimeOps::with_context(context as *const ElephcEvalContext);
    let status = match interpreter::execute_context_callable_call_array_outcome(
        context,
        callback,
        RuntimeCellHandle::from_raw(arg_array),
        &mut values,
    ) {
        Ok(outcome) => write_outcome(outcome, out).code(),
        Err(status) => status.code(),
    };
    if let Some((class_scope, called_class_scope)) = forwarded_scopes {
        if called_class_scope.is_some() {
            context.pop_called_class_scope();
        }
        if class_scope.is_some() {
            context.pop_class_scope();
        }
    }
    if let Some((hooks, scope)) = global_sync {
        (hooks.eval_to_native)(scope);
    }
    if created_context && context.request_retained_context_free() {
        crate::ffi::context::finalize_eval_context_free(context_ptr);
    }
    status
}

/// Resolves the owner context of an eval-owned Closure or object-method callback array.
#[cfg(not(test))]
fn eval_callable_owner_context(
    callback: RuntimeCellHandle,
) -> Option<*mut ElephcEvalContext> {
    let mut values = ElephcRuntimeOps::with_context(std::ptr::null());
    if let Ok(identity) = values.object_identity(callback) {
        return crate::ffi::dynamic_destructors::dynamic_object_owner_context(identity);
    }
    let key = values.array_iter_key(callback, 0).ok()?;
    let receiver = values.array_get(callback, key).ok()?;
    let identity = values.object_identity(receiver).ok()?;
    crate::ffi::dynamic_destructors::dynamic_object_owner_context(identity)
}
