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
use crate::ffi::class_autoload::install_class_autoload_hook;
#[cfg(not(test))]
use crate::ffi::object_relation::install_object_relation_hook;
#[cfg(not(test))]
use crate::ffi::serialize_objects::install_serialize_object_hook;
#[cfg(not(test))]
use crate::ffi::unserialize_objects::install_unserialize_object_hook;
#[cfg(not(test))]
use crate::ffi::generator_protocol::install_generator_protocol_hook;
#[cfg(not(test))]
use crate::ffi::ob_handlers::install_ob_handler_hook;
use std::ptr;

/// Returns the process-wide context every NULL-handle bridge call shares.
///
/// A bridge entry reached with a null `ctx` used to MINT a context and drop it when the call
/// returned, so every piece of per-context interpreter state died with the call that created
/// it. `hash_init()` registered its hash context in one throwaway resource table and
/// `hash_final()` looked it up in another, empty one — reported as a bare
/// `Fatal error: eval() runtime failed`, with nothing to say that the two calls had not shared
/// a world. These tables are REQUEST state in PHP, not call state, so one context per process
/// is the faithful shape; `--web` forks a worker per request, which keeps that scope right.
///
/// The context is leaked on purpose: its address is handed to generated code and to the
/// process-global owner registries, which outlive any single call.
/// Address of the leaked null-handle context, once one has been created.
#[cfg(not(test))]
static SHARED_NULL_HANDLE_CONTEXT: std::sync::OnceLock<usize> = std::sync::OnceLock::new();

/// Every context `__elephc_eval_context_new` handed out and nothing has freed yet.
///
/// Addresses, not pointers, so the mutex stays `Sync`. The null-handle context is not in here:
/// it is built directly and leaked for the process, and the request boundary resets it in place.
#[cfg(not(test))]
static LIVE_EVAL_CONTEXTS: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<usize>>> =
    std::sync::OnceLock::new();

/// Returns the process-local set of contexts that have been allocated and not yet freed.
#[cfg(not(test))]
fn live_eval_contexts() -> &'static std::sync::Mutex<std::collections::HashSet<usize>> {
    LIVE_EVAL_CONTEXTS.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

/// Records one freshly allocated context so the request boundary can find it again.
#[cfg(not(test))]
fn register_live_eval_context(context: *mut ElephcEvalContext) {
    if let Ok(mut contexts) = live_eval_contexts().lock() {
        contexts.insert(context as usize);
    }
}

/// Forgets one context that is being freed, so the boundary never frees it twice.
#[cfg(not(test))]
fn unregister_live_eval_context(context: *mut ElephcEvalContext) {
    if let Ok(mut contexts) = live_eval_contexts().lock() {
        contexts.remove(&(context as usize));
    }
}

/// Frees every context still alive at a generated web request boundary.
///
/// `__elephc_eval_context_free` refuses to free a context while it still owns a live closure, a
/// live dynamic object or a declared global function -- correct inside a request, and wrong at
/// the end of one, because those objects live in the arena `__rt_web_reset` is about to wipe and
/// the counters have no way to learn that. The contexts therefore accumulated: a Symfony request
/// left about a dozen behind, each pinning the `EvalClass` bodies it had parsed, and the worker
/// grew 2.6MB per request until the page latency followed it.
///
/// Nothing can still reference them here. The previous request's AOT frames have returned, the
/// autoload and global-function owners were released just above, and the object-identity registry
/// that could reach one through a destructor was cleared with them. A context PCNTL still holds a
/// signal handler for is left alone, exactly as the ABI free leaves it alone.
#[cfg(not(test))]
pub(crate) fn reset_retained_eval_contexts() {
    let contexts = {
        let Ok(mut contexts) = live_eval_contexts().lock() else {
            return;
        };
        std::mem::take(&mut *contexts).into_iter().collect::<Vec<_>>()
    };
    for address in contexts {
        let context = address as *mut ElephcEvalContext;
        if crate::context::pcntl_runtime::defer_context_free(context) {
            register_live_eval_context(context);
            continue;
        }
        if let Some(live) = unsafe { context.as_mut() } {
            live.forget_request_scoped_state();
        }
        unsafe { drop_eval_context_now(context) };
    }
}

/// Returns the null-handle context's address WITHOUT creating one.
#[cfg(not(test))]
fn shared_null_handle_context_address() -> Option<usize> {
    SHARED_NULL_HANDLE_CONTEXT.get().copied()
}

#[cfg(not(test))]
pub(crate) fn shared_null_handle_context() -> &'static mut ElephcEvalContext {
    let address = *SHARED_NULL_HANDLE_CONTEXT.get_or_init(|| {
        let context = Box::new(ElephcEvalContext::new());
        Box::into_raw(context) as usize
    });
    // SAFETY: the box above is leaked for the life of the process and this is the only
    // function that hands out the pointer, so the borrow cannot outlive the allocation.
    let context = unsafe { &mut *(address as *mut ElephcEvalContext) };
    crate::context::sync_global_eval_aot_metadata(context);
    // Class-like metadata is published separately from AOT metadata, and a context that has
    // the second but not the first cannot decide `instanceof` for an autoloaded interface at
    // all -- it refuses the parameter binding as a runtime fatal rather than as a TypeError.
    context.sync_global_eval_classes();
    context
}

/// Drops every request-scoped value held by the process-lifetime null-handle context.
///
/// Called from the generated web request-boundary reset, beside the global-registry resets it
/// has to agree with. Touches the context only if one was ever created, so a program that never
/// reached an AOT eval callback allocates nothing here.
#[cfg(not(test))]
pub(crate) fn forget_shared_null_handle_declarations() {
    if let Some(address) = shared_null_handle_context_address() {
        // SAFETY: the same leaked allocation `shared_null_handle_context` hands out, and the
        // request boundary is single-threaded with respect to eval work.
        let context = unsafe { &mut *(address as *mut ElephcEvalContext) };
        context.forget_request_scoped_state();
    }
}

/// Registers the module's paired global transfer routines before publishing AOT
/// metadata. Missing halves are rejected without replacing an existing pair.
///
/// # Safety
/// `ctx` must be null or a live context handle. Both hooks must remain executable
/// for the context's lifetime and obey the live-scope transfer ABI.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_set_global_sync_hooks(
    ctx: *mut ElephcEvalContext,
    native_to_eval: Option<crate::context::NativeGlobalSyncHook>,
    eval_to_native: Option<crate::context::NativeGlobalSyncHook>,
) -> i64 {
    let Some(context) = (unsafe { ctx.as_mut() }) else {
        return 0;
    };
    if context.abi_version() != ABI_VERSION {
        return 0;
    }
    let (Some(native_to_eval), Some(eval_to_native)) = (native_to_eval, eval_to_native) else {
        return 0;
    };
    context.native_global_sync = Some(crate::context::NativeGlobalSyncHooks {
        native_to_eval,
        eval_to_native,
    });
    1
}

#[cfg(test)]
mod global_sync_tests {
    use super::*;

    unsafe extern "C" fn transfer(_scope: *mut ElephcEvalScope) {}

    #[test]
    fn global_sync_registration_requires_a_complete_pair() {
        let mut context = ElephcEvalContext::new();
        let mut incompatible = ElephcEvalContext::for_abi_version(ABI_VERSION + 1);
        unsafe {
            assert_eq!(__elephc_eval_context_set_global_sync_hooks(
                &mut incompatible, Some(transfer), Some(transfer)), 0);
            assert!(incompatible.native_global_sync.is_none());
            assert_eq!(__elephc_eval_context_set_global_sync_hooks(
                ptr::null_mut(), Some(transfer), Some(transfer)), 0);
            assert_eq!(__elephc_eval_context_set_global_sync_hooks(
                &mut context, Some(transfer), None), 0);
            assert!(context.native_global_sync.is_none());
            assert_eq!(__elephc_eval_context_set_global_sync_hooks(
                &mut context, Some(transfer), Some(transfer)), 1);
            assert_eq!(__elephc_eval_context_set_global_sync_hooks(
                &mut context, None, Some(transfer)), 0);
            assert!(context.native_global_sync.is_some());
        }
    }
}

/// Returns the ABI version expected by generated elephc eval call sites.
#[no_mangle]
pub extern "C" fn __elephc_eval_abi_version() -> u32 {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_abi_version");
    ABI_VERSION
}

/// Allocates a process-level eval context handle for generated code.
#[no_mangle]
pub extern "C" fn __elephc_eval_context_new() -> *mut ElephcEvalContext {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_new");
    #[cfg(not(test))]
    install_dynamic_object_destructor_hook();
    #[cfg(not(test))]
    install_ob_handler_hook();
    #[cfg(not(test))]
    install_generator_protocol_hook();
    #[cfg(not(test))]
    install_class_autoload_hook();
    #[cfg(not(test))]
    install_unserialize_object_hook();
    #[cfg(not(test))]
    install_object_relation_hook();
    #[cfg(not(test))]
    install_serialize_object_hook();
    let context = Box::into_raw(Box::new(ElephcEvalContext::new()));
    #[cfg(not(test))]
    register_live_eval_context(context);
    context
}

/// Publishes one generated context's complete AOT metadata for null-context fallback execution.
///
/// # Safety
/// `ctx` must be null or a live context handle allocated by `__elephc_eval_context_new()`.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_publish_aot_metadata(
    ctx: *const ElephcEvalContext,
) {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_publish_aot_metadata");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_try_sync_aot_metadata");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_trace_aot_null_method_receiver");
    if !crate::eval_trace::enabled() {
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_trace_aot_raw_null_method_receiver");
    if !crate::eval_trace::enabled() {
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_trace_aot_handler_top");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_trace_aot_property_cell");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_trace_aot_raw_property_receiver");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_set_strict_php");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_set_php_version_id");
    crate::eval_php_profile::set_eval_php_version_id(version_id);
}

/// Forwards the compile mode so `PHP_SAPI` agrees across the eval boundary.
///
/// Generated code emits this beside the version-profile call. A `--web` binary reported
/// `cli-server` natively and `cli` from inside `eval()` before this existed, and interpreted
/// library code that branches on `PHP_SAPI` — Symfony's dumped container most of all — then took
/// the console path inside an HTTP request.
#[no_mangle]
pub extern "C" fn __elephc_eval_set_web_sapi(web: u8) {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_set_web_sapi");
    crate::eval_php_profile::set_eval_web_sapi(web != 0);
}

/// Frees a process-level eval context handle allocated by the eval bridge.
///
/// Releases every retained `CURLOPT_PRIVATE` value in `context.stream_resources` ONE STEP
/// before the context (and, transitively, its `EvalStreamResources`) actually drops — see
/// `EvalStreamResources::release_curl_easy_private_values`'s own doc for why that release
/// cannot happen inside `Drop` itself and has to live here instead.
/// `ElephcRuntimeOps::new()` needs no live eval call or
/// context to construct — it is a stateless adapter over generated runtime C-ABI symbols —
/// so it is safe to build one ad hoc for this one release pass even though this function
/// itself receives no `RuntimeValueOps` parameter.
///
/// ORDER IS LOAD-BEARING: the curl-private release runs BEFORE
/// `context.unregister_dynamic_object_context()` below, not after. A retained
/// `CURLOPT_PRIVATE` value can be (or transitively reference, through an array/object
/// graph) an eval-declared object with a `__destruct()` method; releasing it to a
/// zero refcount runs that destructor, and dispatching it correctly depends on this
/// context still being registered as the object's owner
/// (`crate::ffi::dynamic_destructors::dynamic_object_owner_context`). Releasing AFTER
/// unregistering would let that lookup fail for exactly this one teardown-time release,
/// silently skipping (or misrouting) a `__destruct()` call every other release path in
/// this crate correctly triggers.
///
/// # Safety
/// `ctx` must be null or a pointer returned by `__elephc_eval_context_new`
/// that has not already been freed.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_free(ctx: *mut ElephcEvalContext) {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_free");
    // TWO independent deferrals, and the teardown runs only when NEITHER holds the context: a
    // process-global PCNTL signal handler still referencing it, and a PHP function or closure
    // this context declared that outlives the ABI free.
    if ctx.is_null() || crate::context::pcntl_runtime::defer_context_free(ctx) {
        return;
    }
    if unsafe { ctx.as_ref() }.is_some_and(ElephcEvalContext::request_retained_context_free) {
        unsafe { drop_eval_context_now(ctx) };
    }
}

/// Unregisters every process-global callback into one context, then drops it immediately.
///
/// PCNTL calls this only after the last retained signal handler stops referencing
/// a context whose normal ABI teardown was deferred.
///
/// # Safety
/// `ctx` must point to a live context allocated by `__elephc_eval_context_new`
/// and no process-global PCNTL handler may still reference it.
pub(crate) unsafe fn drop_eval_context_now(ctx: *mut ElephcEvalContext) {
    #[cfg(all(feature = "curl", not(test)))]
    if let Some(context) = unsafe { ctx.as_mut() } {
        let mut values = crate::runtime_hooks::ElephcRuntimeOps::new();
        context
            .stream_resources_mut()
            .release_curl_easy_private_values(&mut values);
    }
    unsafe { finalize_eval_context_free(ctx) };
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
    #[cfg(not(test))]
    unregister_live_eval_context(ctx);
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_retain_global_scope");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_set_call_site");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_set_global_scope");
    std::panic::catch_unwind(|| unsafe { eval_context_set_global_scope_inner(ctx, scope) })
        .unwrap_or_else(|_| EvalStatus::RuntimeFatal.code())
}

/// Enters a generated caller's class scope for the next eval fragment.
///
/// # Safety
/// `ctx` must be a valid eval context handle. Class name pointers must be
/// readable UTF-8 slices for their declared byte lengths.
/// Enters one `isset`/`empty`/`??` operand scope for COMPILED code.
///
/// The quiet fetch is what makes an uninitialized typed property answer instead of raising, and
/// the interpreter pushes it around its own operands. Compiled code needs the same door: an AOT
/// method reading an EVAL-OWNED object's property reaches
/// `__elephc_eval_property_get` through the bridge, and nothing on that route had entered the
/// mode, so `empty($this->p[$k])` raised from compiled code while the identical source raised
/// nothing when interpreted. That is the Symfony stop --
/// `CheckCircularReferencesPass::$checkedLazyNodes`.
///
/// The depth is thread-local and shared with the interpreter's, so a chain that crosses between
/// the two -- which is the normal case here, compiled caller and interpreted property owner --
/// sees one mode rather than two. It takes no context for that reason.
///
/// PHP decides the quiet fetch at COMPILE time, propagating it down property and dim fetch nodes
/// and never into a call, so the compiled side only pushes this around operands whose whole
/// chain is a property/dim walk. A call inside the operand means no push at all, which keeps the
/// "does not cross a call" rule without needing a barrier at every compiled call site.
#[no_mangle]
pub extern "C" fn __elephc_eval_quiet_property_fetch_push() {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_quiet_property_fetch_push");
    crate::context::push_thread_quiet_property_fetch();
}

/// Leaves one compiled-side quiet-fetch operand scope.
#[no_mangle]
pub extern "C" fn __elephc_eval_quiet_property_fetch_pop() {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_quiet_property_fetch_pop");
    crate::context::pop_thread_quiet_property_fetch();
}

#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_context_push_class_scope(
    ctx: *mut ElephcEvalContext,
    class_ptr: *const u8,
    class_len: u64,
    called_class_ptr: *const u8,
    called_class_len: u64,
) -> i32 {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_push_class_scope");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_context_pop_class_scope");
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
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_native_frame_called_class_override");
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

/// Records the lexical class of the compiled frame that is about to call into the bridge.
///
/// A compiled body with no eval context of its own hands the bridge a null caller, and the
/// interpreter then reads no calling scope at all -- a protected method reached from such a
/// body was refused "from global scope". This publishes the one missing fact without an eval
/// context and without widening the method-call ABI. Always paired with
/// `__elephc_eval_pop_native_caller_class` around exactly one call.
///
/// # Safety
/// `class_ptr` must be readable for `class_len` bytes when `class_len > 0`.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_push_native_caller_class(
    class_ptr: *const u8,
    class_len: u64,
) {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_push_native_caller_class");
    let _ = std::panic::catch_unwind(|| {
        let Ok(class_name) = (unsafe { abi_name_to_string(class_ptr, class_len) }) else {
            // Push regardless so the pop stays balanced; an unreadable name simply carries none.
            crate::context::push_native_caller_class("");
            return;
        };
        crate::context::push_native_caller_class(&class_name);
    });
}

/// Drops the innermost compiled-frame lexical class.
///
/// # Safety
/// Must match exactly one earlier `__elephc_eval_push_native_caller_class`.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_pop_native_caller_class() {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_pop_native_caller_class");
    let _ = std::panic::catch_unwind(crate::context::pop_native_caller_class);
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
