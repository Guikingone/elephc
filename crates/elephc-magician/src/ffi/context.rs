//! Purpose:
//! Exports eval context handle allocation and context metadata setters.
//! These functions manage process-level eval state and call-site/global/class
//! scope metadata used while executing fragments.
//!
//! Called from:
//! - Generated EIR backend assembly through `__elephc_eval_context_*` symbols.
//!
//! Key details:
//! - Context handles are opaque across the ABI.
//! - Call-site metadata is UTF-8 and is validated before storing.

use super::util::abi_name_to_string;
use crate::abi::{ElephcEvalContext, ElephcEvalScope, ABI_VERSION};
use crate::context::native_frame_called_class_override_bytes;
use crate::errors::EvalStatus;
#[cfg(not(test))]
use crate::ffi::dynamic_destructors::install_dynamic_object_destructor_hook;
#[cfg(not(test))]
use crate::ffi::ob_handlers::install_ob_handler_hook;
use std::ptr;

/// Returns the ABI version expected by generated elephc eval call sites.
#[no_mangle]
pub extern "C" fn __elephc_eval_abi_version() -> u32 {
    ABI_VERSION
}

/// Allocates a process-level eval context handle for generated code.
#[no_mangle]
pub extern "C" fn __elephc_eval_context_new() -> *mut ElephcEvalContext {
    #[cfg(not(test))]
    install_dynamic_object_destructor_hook();
    #[cfg(not(test))]
    install_ob_handler_hook();
    Box::into_raw(Box::new(ElephcEvalContext::new()))
}

/// Publishes one generated context's complete AOT metadata for null-context fallback execution.
///
/// # Safety
/// `ctx` must be null or a live context handle allocated by `__elephc_eval_context_new()`.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_publish_aot_metadata(
    ctx: *const ElephcEvalContext,
) {
    #[cfg(not(test))]
    if let Some(context) = unsafe { ctx.as_ref() } {
        crate::context::publish_global_eval_aot_metadata(context);
    }
    #[cfg(test)]
    let _ = ctx;
}

/// Imports the process-global immutable AOT snapshot into one generated context.
///
/// Returns one when metadata was available and loaded, otherwise zero so the
/// generated registration helper can publish the first snapshot.
///
/// # Safety
/// `ctx` must be null or a live context handle allocated by `__elephc_eval_context_new()`.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_try_sync_aot_metadata(
    ctx: *mut ElephcEvalContext,
) -> i64 {
    #[cfg(not(test))]
    {
        let Some(context) = (unsafe { ctx.as_mut() }) else {
            return 0;
        };
        return i64::from(crate::context::sync_global_eval_aot_metadata(context));
    }
    #[cfg(test)]
    {
        let _ = ctx;
        0
    }
}

/// Reports the generated function and method when AOT dispatch rejects a null receiver.
///
/// # Safety
/// Both byte ranges must be valid UTF-8 when their corresponding lengths are non-zero.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_trace_aot_null_method_receiver(
    function_ptr: *const u8,
    function_len: u64,
    method_ptr: *const u8,
    method_len: u64,
    line: u64,
) {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_none() {
        return;
    }
    let Ok(function) = abi_name_to_string(function_ptr, function_len) else {
        return;
    };
    let Ok(method) = abi_name_to_string(method_ptr, method_len) else {
        return;
    };
    eprintln!(
        "[elephc-eval-trace] phase=aot_method_call_non_object function={function:?} method={method:?} line={line}",
    );
}

/// Reports the unmodified receiver word that reached an AOT null-method fatal.
///
/// # Safety
/// Both byte ranges must be valid UTF-8 when their corresponding lengths are non-zero.
/// `receiver` is logged as an integer only and is never dereferenced.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_trace_aot_raw_null_method_receiver(
    function_ptr: *const u8,
    function_len: u64,
    method_ptr: *const u8,
    method_len: u64,
    line: u64,
    receiver: usize,
) {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_none() {
        return;
    }
    let Ok(function) = abi_name_to_string(function_ptr, function_len) else {
        return;
    };
    let Ok(method) = abi_name_to_string(method_ptr, method_len) else {
        return;
    };
    let representation = match receiver {
        0 => "zero",
        0x7fff_ffff_ffff_fffe => "null-sentinel",
        1 => "true-scalar",
        _ => "other",
    };
    eprintln!(
        "[elephc-eval-trace] phase=aot_method_call_null_receiver_raw function={function:?} method={method:?} line={line} receiver={receiver:#x} representation={representation}",
    );
}

/// Reports an opt-in EIR exception-handler push or pop with its resulting top.
///
/// # Safety
/// `function_ptr` must reference UTF-8 for `function_len` bytes when the length
/// is non-zero. `top` is logged as an integer and is never dereferenced.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_trace_aot_handler_top(
    event: u64,
    token: u64,
    top: usize,
    function_ptr: *const u8,
    function_len: u64,
) {
    if std::env::var_os("ELEPHC_HANDLER_TRACE").is_none() {
        return;
    }
    let function = if function_len == 0 {
        "<native-boundary>".to_string()
    } else {
        let Ok(function) = abi_name_to_string(function_ptr, function_len) else {
            return;
        };
        function
    };
    let event = match event {
        1 => "push",
        2 => "pop",
        3 => "native-pop",
        _ => "unknown",
    };
    eprintln!(
        "[elephc-handler-trace] event={event} function={function:?} token={token} top={top:#x}"
    );
}

/// Reports a boxed AOT property receiver before or after an opt-in property write trace point.
///
/// # Safety
/// `site_ptr` must name a valid UTF-8 byte range and `cell` must be a readable boxed Mixed cell.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_trace_aot_property_cell(
    site_ptr: *const u8,
    site_len: u64,
    cell: *const usize,
) {
    if std::env::var_os("ELEPHC_AOT_PROPERTY_TRACE").is_none() {
        return;
    }
    let Ok(site) = abi_name_to_string(site_ptr, site_len) else {
        return;
    };
    if let Some(filter) = std::env::var_os("ELEPHC_AOT_PROPERTY_TRACE_FILTER") {
        if !site.contains(filter.to_string_lossy().as_ref()) {
            return;
        }
    }
    if cell.is_null() {
        eprintln!("[elephc-aot-property-trace] site={site:?} cell=null");
        return;
    }
    let words = unsafe { [*cell, *cell.add(1), *cell.add(2)] };
    eprintln!("[elephc-aot-property-trace] site={site:?} cell={cell:p} words={words:#x?}");
}

/// Reports an opt-in AOT typed-property receiver without changing its ownership.
///
/// # Safety
/// `site_ptr` must name a valid UTF-8 byte range; `object` is null or points at
/// a readable AOT object header; and `value` is readable as an object only when
/// `value_is_object` is non-zero.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_trace_aot_raw_property_receiver(
    site_ptr: *const u8,
    site_len: u64,
    object: *const usize,
    value: usize,
    value_is_object: u64,
) {
    if std::env::var_os("ELEPHC_AOT_PROPERTY_TRACE").is_none() {
        return;
    }
    let Ok(site) = abi_name_to_string(site_ptr, site_len) else {
        return;
    };
    if let Some(filter) = std::env::var_os("ELEPHC_AOT_PROPERTY_TRACE_FILTER") {
        if !site.contains(filter.to_string_lossy().as_ref()) {
            return;
        }
    }
    let class_id = (!object.is_null()).then(|| unsafe { *object });
    let value_class_id = (value_is_object != 0 && value != 0)
        .then(|| unsafe { *(value as *const usize) });
    eprintln!(
        "[elephc-aot-property-trace] site={site:?} object={object:p} class_id={class_id:?} value={value:#x} value_is_object={} value_class_id={value_class_id:?}",
        value_is_object != 0,
    );
}

/// Marks this program's eval bridge as strict-PHP: extension builtins
/// (`ptr_*`, `buffer_*`, `class_attribute_*`) disappear from eval dispatch and
/// introspection, matching the PHP interpreter where those names do not exist.
///
/// Generated code emits this call while initializing the eval context, only in
/// binaries compiled with `elephc --strict-php`. The flag is thread-local and
/// elephc programs run every eval on the initializing thread, so one call
/// covers the program lifetime.
#[no_mangle]
pub extern "C" fn __elephc_eval_set_strict_php(enabled: u8) {
    crate::strict_php_mode::set_strict_php_mode(enabled != 0);
}

/// Selects the PHP language profile eval reports, so that a binary compiled
/// `--php-version 8.2` answers `8.2.0` from inside `eval()` exactly as it does
/// natively.
///
/// Generated code emits this call before every runtime eval dispatch. Ids outside
/// the profiles elephc supports leave the active one untouched, which keeps every
/// consumer that links this archive without elephc's codegen — the test harnesses
/// in this crate included — on the default profile.
#[no_mangle]
pub extern "C" fn __elephc_eval_set_php_version_id(version_id: u32) {
    crate::eval_php_profile::set_eval_php_version_id(version_id);
}

/// Frees a process-level eval context handle allocated by the eval bridge.
///
/// # Safety
/// `ctx` must be null or a pointer returned by `__elephc_eval_context_new`
/// that has not already been freed.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_free(ctx: *mut ElephcEvalContext) {
    let should_finalize = unsafe { ctx.as_ref() }
        .is_some_and(ElephcEvalContext::request_retained_context_free);
    if should_finalize {
        unsafe { finalize_eval_context_free(ctx) };
    }
}

/// Performs the final one-time destruction of a context with no retained PHP functions or closures.
///
/// # Safety
/// `ctx` must be a unique live handle returned by `__elephc_eval_context_new()`. Callers first
/// gate this through `ElephcEvalContext::request_retained_context_free()` or the matching last
/// retained-owner release, so no PHP callable can retain the pointer after this point.
pub(crate) unsafe fn finalize_eval_context_free(ctx: *mut ElephcEvalContext) {
    if ctx.is_null() {
        return;
    }
    let owned_global_scope = if let Some(context) = unsafe { ctx.as_mut() } {
        context.unregister_dynamic_object_context();
        context.take_owned_global_scope()
    } else {
        None
    };
    crate::context::unregister_global_eval_functions_for_context(ctx);
    crate::ffi::ob_handlers::unregister_ob_handlers_for_context(ctx);
    if let Some(scope) = owned_global_scope {
        unsafe { crate::ffi::scope::__elephc_eval_scope_free(scope) };
    }
    unsafe { drop(Box::from_raw(ctx)) };
}

/// Transfers an eval global scope to a context that outlives its AOT frame.
///
/// # Safety
/// `ctx` and `scope` must be live bridge handles from the same generated frame. A nonzero result
/// means ownership moved to the context and the frame must not free `scope` itself.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_retain_global_scope(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
) -> i32 {
    std::panic::catch_unwind(|| unsafe {
        ctx.as_mut()
            .is_some_and(|context| context.retain_global_scope_for_request(scope))
            .into()
    })
    .unwrap_or(0)
}

/// Records source metadata for the next eval fragment executed in this context.
///
/// # Safety
/// `ctx` must be a valid eval context handle. `file_ptr` and `dir_ptr` must be
/// readable for their matching lengths when the length is greater than zero.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_set_call_site(
    ctx: *mut ElephcEvalContext,
    file_ptr: *const u8,
    file_len: u64,
    dir_ptr: *const u8,
    dir_len: u64,
    line: u64,
) -> i32 {
    std::panic::catch_unwind(|| unsafe {
        eval_context_set_call_site_inner(ctx, file_ptr, file_len, dir_ptr, dir_len, line)
    })
    .unwrap_or_else(|_| EvalStatus::RuntimeFatal.code())
}

/// Records the materialized program-global eval scope for `global` aliases.
///
/// # Safety
/// `ctx` and `scope` must be valid handles allocated by the eval bridge. The
/// context does not own `scope`; generated code must keep the scope alive for
/// as long as the context can execute eval fragments that reference globals.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_set_global_scope(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
) -> i32 {
    std::panic::catch_unwind(|| unsafe { eval_context_set_global_scope_inner(ctx, scope) })
        .unwrap_or_else(|_| EvalStatus::RuntimeFatal.code())
}

/// Enters a generated caller's class scope for the next eval fragment.
///
/// # Safety
/// `ctx` must be a valid eval context handle. Class name pointers must be
/// readable UTF-8 slices for their declared byte lengths.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_push_class_scope(
    ctx: *mut ElephcEvalContext,
    class_ptr: *const u8,
    class_len: u64,
    called_class_ptr: *const u8,
    called_class_len: u64,
) -> i32 {
    std::panic::catch_unwind(|| unsafe {
        eval_context_push_class_scope_inner(
            ctx,
            class_ptr,
            class_len,
            called_class_ptr,
            called_class_len,
        )
    })
    .unwrap_or_else(|_| EvalStatus::RuntimeFatal.code())
}

/// Leaves a generated caller class scope after an eval fragment returns.
///
/// # Safety
/// `ctx` must be a valid eval context handle previously passed to
/// `__elephc_eval_context_push_class_scope`.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_pop_class_scope(ctx: *mut ElephcEvalContext) -> i32 {
    std::panic::catch_unwind(|| unsafe { eval_context_pop_class_scope_inner(ctx) })
        .unwrap_or_else(|_| EvalStatus::RuntimeFatal.code())
}

/// Reads the late-static override currently installed for a generated/AOT frame.
///
/// # Safety
/// `class_ptr` must be a readable UTF-8 slice for `class_len` bytes. `out_ptr`
/// and `out_len` must be valid writable out-parameters. Returned bytes are
/// owned by eval thread-local state and remain valid until the native frame
/// override guard is dropped.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_native_frame_called_class_override(
    class_ptr: *const u8,
    class_len: u64,
    out_ptr: *mut *const u8,
    out_len: *mut u64,
) -> i32 {
    std::panic::catch_unwind(|| unsafe {
        eval_native_frame_called_class_override_inner(class_ptr, class_len, out_ptr, out_len)
    })
    .unwrap_or(0)
}

/// Runs the call-site metadata setter ABI body after installing a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_context_set_call_site`; callers must pass a valid
/// context and readable UTF-8 file/directory byte slices.
unsafe fn eval_context_set_call_site_inner(
    ctx: *mut ElephcEvalContext,
    file_ptr: *const u8,
    file_len: u64,
    dir_ptr: *const u8,
    dir_len: u64,
    line: u64,
) -> i32 {
    let Some(context) = ctx.as_mut() else {
        return EvalStatus::RuntimeFatal.code();
    };
    if context.abi_version() != ABI_VERSION {
        return EvalStatus::AbiMismatch.code();
    }
    let Ok(file) = abi_name_to_string(file_ptr, file_len) else {
        return EvalStatus::RuntimeFatal.code();
    };
    let Ok(dir) = abi_name_to_string(dir_ptr, dir_len) else {
        return EvalStatus::RuntimeFatal.code();
    };
    let Ok(line) = i64::try_from(line) else {
        return EvalStatus::RuntimeFatal.code();
    };
    context.set_call_site(file, dir, line);
    EvalStatus::Ok.code()
}

/// Runs the global-scope setter ABI body after installing a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_context_set_global_scope`; callers must pass valid
/// context and scope handles owned by generated code.
unsafe fn eval_context_set_global_scope_inner(
    ctx: *mut ElephcEvalContext,
    scope: *mut ElephcEvalScope,
) -> i32 {
    let Some(context) = ctx.as_mut() else {
        return EvalStatus::RuntimeFatal.code();
    };
    if context.abi_version() != ABI_VERSION {
        return EvalStatus::AbiMismatch.code();
    }
    if !context.set_global_scope(scope) {
        return EvalStatus::RuntimeFatal.code();
    }
    EvalStatus::Ok.code()
}

/// Runs the class-scope push ABI body after installing a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_context_push_class_scope`; callers must pass a valid
/// context and readable UTF-8 class-name byte slices.
unsafe fn eval_context_push_class_scope_inner(
    ctx: *mut ElephcEvalContext,
    class_ptr: *const u8,
    class_len: u64,
    called_class_ptr: *const u8,
    called_class_len: u64,
) -> i32 {
    let Some(context) = ctx.as_mut() else {
        return EvalStatus::RuntimeFatal.code();
    };
    if context.abi_version() != ABI_VERSION {
        return EvalStatus::AbiMismatch.code();
    }
    let Ok(class_name) = abi_name_to_string(class_ptr, class_len) else {
        return EvalStatus::RuntimeFatal.code();
    };
    let Ok(called_class_name) = abi_name_to_string(called_class_ptr, called_class_len) else {
        return EvalStatus::RuntimeFatal.code();
    };
    let class_name = class_name.trim_start_matches('\\').to_string();
    if class_name.is_empty() {
        return EvalStatus::RuntimeFatal.code();
    }
    let called_class_name = called_class_name.trim_start_matches('\\');
    let called_class_name = if called_class_name.is_empty() {
        class_name.clone()
    } else {
        called_class_name.to_string()
    };
    let called_class_name = context
        .native_frame_called_class_override(&class_name, &called_class_name)
        .unwrap_or(called_class_name);
    context.push_class_scope(class_name);
    context.push_called_class_scope(called_class_name);
    EvalStatus::Ok.code()
}

/// Runs the class-scope pop ABI body after installing a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_context_pop_class_scope`; callers must pass a valid
/// context handle created by the eval bridge.
unsafe fn eval_context_pop_class_scope_inner(ctx: *mut ElephcEvalContext) -> i32 {
    let Some(context) = ctx.as_mut() else {
        return EvalStatus::RuntimeFatal.code();
    };
    if context.abi_version() != ABI_VERSION {
        return EvalStatus::AbiMismatch.code();
    }
    context.pop_called_class_scope();
    context.pop_class_scope();
    EvalStatus::Ok.code()
}

/// Runs the native-frame called-class lookup after installing a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_native_frame_called_class_override`; generated code
/// passes writable stack slots for both out-parameters.
unsafe fn eval_native_frame_called_class_override_inner(
    class_ptr: *const u8,
    class_len: u64,
    out_ptr: *mut *const u8,
    out_len: *mut u64,
) -> i32 {
    if out_ptr.is_null() || out_len.is_null() {
        return 0;
    }
    unsafe {
        *out_ptr = ptr::null();
        *out_len = 0;
    }
    let Ok(class_name) = abi_name_to_string(class_ptr, class_len) else {
        return 0;
    };
    let Some((called_ptr, called_len)) = native_frame_called_class_override_bytes(&class_name)
    else {
        return 0;
    };
    let Ok(called_len) = u64::try_from(called_len) else {
        return 0;
    };
    unsafe {
        *out_ptr = called_ptr;
        *out_len = called_len;
    }
    1
}
