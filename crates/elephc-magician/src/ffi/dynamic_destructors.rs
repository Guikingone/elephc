//! Purpose:
//! Owns the process-local registry that lets native object release call back
//! into eval-declared `__destruct()` methods for dynamic classes.
//!
//! Called from:
//! - `crate::context::ElephcEvalContext` when dynamic objects are registered or freed.
//! - The generated runtime through the installed destructor hook function pointer.
//!
//! Key details:
//! - The runtime owns object storage and calls this hook while the object is still
//!   intact but already in the final-release path.
//! - Registry values are stored as integer addresses so the global mutex remains
//!   `Sync`; every use revalidates null pointers and ABI version.

#[cfg(not(test))]
use crate::abi::ABI_VERSION;
use crate::abi::ElephcEvalContext;
#[cfg(not(test))]
use crate::errors::EvalStatus;
#[cfg(not(test))]
use crate::interpreter::eval_dynamic_destructor_for_object_cell;
#[cfg(not(test))]
use crate::interpreter::eval_json_encode_serialized_object;
#[cfg(not(test))]
use crate::interpreter::RuntimeValueOps;
#[cfg(not(test))]
use crate::runtime_hooks::{self, ElephcRuntimeOps};
#[cfg(not(test))]
use crate::value::RuntimeCell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

/// Hashes an object address for the owner registry.
///
/// The runtime asks this registry about EVERY object it frees once the bridge is linked, and
/// SipHash on those lookups was 4% of a Symfony request. Addresses are not adversarial input,
/// only aligned: the low four bits carry nothing, and the bucket is taken from the low bits, so
/// they are dropped before a multiplicative mix whose high half is folded back down.
#[derive(Default)]
struct ObjectAddressHasher(u64);

impl Hasher for ObjectAddressHasher {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0.rotate_left(8) ^ u64::from(*byte)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
    }

    fn write_u64(&mut self, address: u64) {
        let mixed = (address >> 4).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.0 = mixed ^ (mixed >> 32);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

type ObjectOwnerMap = HashMap<u64, usize, BuildHasherDefault<ObjectAddressHasher>>;

static DYNAMIC_DESTRUCTOR_CONTEXTS: OnceLock<Mutex<ObjectOwnerMap>> = OnceLock::new();

/// Buckets in the owner filter. Must match `EVAL_OWNER_FILTER_MASK` in the generated runtime,
/// which picks a bucket as `(identity >> 4) & (OWNER_FILTER_BUCKETS - 1)`.
pub(crate) const OWNER_FILTER_BUCKETS: usize = 4096;

/// How many registered identities fall in each bucket, read by generated code without a call.
///
/// WHY. Once the bridge is linked, every non-final method call and declared-property read on a
/// typed receiver asks this registry whether the receiver is eval-owned, and every object free
/// asks it for an eval destructor: some 900 calls and a mutex per Symfony request, for a map
/// that holds a handful of objects. A zero bucket answers "no owner" for all of them. The counts
/// change only under the registry mutex and always equal the map's membership per bucket, so a
/// zero is exact; a non-zero bucket only means "ask".
static OWNER_FILTER: [AtomicU32; OWNER_FILTER_BUCKETS] =
    [const { AtomicU32::new(0) }; OWNER_FILTER_BUCKETS];

/// Returns the filter bucket of one object identity, as the generated probe computes it.
fn owner_filter_bucket(identity: u64) -> &'static AtomicU32 {
    &OWNER_FILTER[(identity >> 4) as usize & (OWNER_FILTER_BUCKETS - 1)]
}

/// Returns the process-local dynamic object to eval context registry.
fn dynamic_destructor_contexts() -> &'static Mutex<ObjectOwnerMap> {
    DYNAMIC_DESTRUCTOR_CONTEXTS.get_or_init(|| Mutex::new(ObjectOwnerMap::default()))
}

/// Installs the eval dynamic object destructor callback into the generated runtime.
#[cfg(not(test))]
pub(crate) fn install_dynamic_object_destructor_hook() {
    unsafe {
        runtime_hooks::install_dynamic_object_destructor_hook(
            __elephc_eval_dynamic_object_destruct as *const () as usize,
            OWNER_FILTER.as_ptr() as usize,
        );
    }
}

/// Records which eval context owns one dynamic object's eval class metadata.
pub(crate) fn register_dynamic_object_context(identity: u64, context: *mut ElephcEvalContext) {
    if identity == 0 || context.is_null() {
        return;
    }
    if let Ok(mut contexts) = dynamic_destructor_contexts().lock() {
        if contexts.insert(identity, context as usize).is_none() {
            owner_filter_bucket(identity).fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Removes one dynamic object from the process-local destructor registry.
pub(crate) fn unregister_dynamic_object(identity: u64) {
    if identity == 0 {
        return;
    }
    if let Ok(mut contexts) = dynamic_destructor_contexts().lock() {
        if contexts.remove(&identity).is_some() {
            owner_filter_bucket(identity).fetch_sub(1, Ordering::Relaxed);
        }
    }
}

/// Removes every dynamic object currently associated with a soon-to-be-freed context.
pub(crate) fn unregister_dynamic_objects_for_context(context: *mut ElephcEvalContext) {
    if context.is_null() {
        return;
    }
    let context = context as usize;
    if let Ok(mut contexts) = dynamic_destructor_contexts().lock() {
        contexts.retain(|identity, owner| {
            let keep = *owner != context;
            if !keep {
                owner_filter_bucket(*identity).fetch_sub(1, Ordering::Relaxed);
            }
            keep
        });
    }
}

/// Forgets every dynamic object at a generated web request boundary.
///
/// The key is the object's IDENTITY, which is its address in the PHP arena, and `__rt_web_reset`
/// hands that whole arena back to pure-bump allocation. Keeping the map means the next request's
/// object lands on a previous request's address and inherits its entry: `__rt_heap_free` then
/// asks this registry for an owner, gets one, and runs a `__destruct()` belonging to a class the
/// new object never had, through a context whose own tables were emptied at the same boundary.
/// Nothing in the map can still be valid after the wipe, so the whole map goes.
#[cfg(not(test))]
pub(crate) fn reset_dynamic_object_contexts() {
    if let Ok(mut contexts) = dynamic_destructor_contexts().lock() {
        contexts.clear();
        for bucket in &OWNER_FILTER {
            bucket.store(0, Ordering::Relaxed);
        }
    }
}

/// Unit tests run a single interpreter context with no registry, so no object has a foreign
/// owner; the cross-context closure path (`eval_foreign_closure_owner`) then stays inert.
#[cfg(test)]
pub(crate) fn dynamic_object_owner_context(_identity: u64) -> Option<*mut ElephcEvalContext> {
    None
}

/// Looks up the eval context that owns one dynamic object identity.
#[cfg(not(test))]
pub(crate) fn dynamic_object_owner_context(identity: u64) -> Option<*mut ElephcEvalContext> {
    let contexts = dynamic_destructor_contexts().lock().ok()?;
    let context = *contexts.get(&identity)?;
    Some(context as *mut ElephcEvalContext)
}

/// Checks an object identity without allocating argument cells or dereferencing the object.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn __elephc_eval_object_has_dynamic_owner(identity: u64) -> u64 {
    u64::from(dynamic_object_owner_context(identity).is_some())
}

/// Answers what a compiled `json_encode()` should encode for an object the interpreter owns.
///
/// Returns an owned boxed Mixed cell -- the object's `jsonSerialize()` result, or an array of its
/// public properties -- or null when the interpreter does not own the object or cannot answer, in
/// which case the compiled encoder emits `null`.
///
/// # Safety
/// `object` must be null or a live elephc runtime object pointer.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_json_serialize(object: *mut RuntimeCell) -> *mut RuntimeCell {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_json_serialize");
    std::panic::catch_unwind(|| unsafe { json_serialize_inner(object) })
        .unwrap_or(std::ptr::null_mut())
}

/// Executes the serializer body after the exported ABI shim has installed a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_json_serialize`.
#[cfg(not(test))]
unsafe fn json_serialize_inner(object: *mut RuntimeCell) -> *mut RuntimeCell {
    if object.is_null() {
        return std::ptr::null_mut();
    }
    let Some(context) = dynamic_object_owner_context(object as u64) else {
        return std::ptr::null_mut();
    };
    let Some(context) = (unsafe { context.as_mut() }) else {
        return std::ptr::null_mut();
    };
    if context.abi_version() != ABI_VERSION {
        return std::ptr::null_mut();
    }
    let mut values = ElephcRuntimeOps::with_context(context as *const ElephcEvalContext);
    let Ok(object_cell) = ElephcRuntimeOps::object_from_raw(object) else {
        return std::ptr::null_mut();
    };
    let result = eval_json_encode_serialized_object(object_cell, context, &mut values);
    let _ = values.release(object_cell);
    match result {
        Ok(handle) => handle.as_ptr(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Runs an eval dynamic object destructor from the native object free path.
///
/// # Safety
/// `object` must be null or a live elephc runtime object pointer. The runtime
/// calls this only while its object destruction guard bit is set, so boxing the
/// borrowed object for `$this` cannot recursively free the same storage.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_dynamic_object_destruct(
    object: *mut RuntimeCell,
) -> u64 {
    crate::ffi::util::trace_eval_ffi_entry("__elephc_eval_dynamic_object_destruct");
    std::panic::catch_unwind(|| unsafe { dynamic_object_destruct_inner(object) }).unwrap_or(0)
}

/// Executes the callback body after the exported ABI shim has installed a panic boundary.
///
/// # Safety
/// Mirrors `__elephc_eval_dynamic_object_destruct`; callers must pass a live raw
/// object pointer whose refcount guard already marks destruction as active.
#[cfg(not(test))]
unsafe fn dynamic_object_destruct_inner(object: *mut RuntimeCell) -> u64 {
    if object.is_null() {
        return 0;
    }
    let identity = object as u64;
    if crate::eval_trace::enabled() {
        let owner_class = dynamic_object_owner_context(identity)
            .and_then(|owner| unsafe { owner.as_ref() })
            .and_then(|context| context.dynamic_object_class(identity))
            .map(|class| class.name().to_string());
        eprintln!(
            "[elephc-eval-trace] phase=dynamic_object_destruct_begin object={object:p} identity={identity} class={owner_class:?}"
        );
    }
    let Some(context) = dynamic_object_owner_context(identity) else {
        return 0;
    };
    let Some(context) = (unsafe { context.as_mut() }) else {
        unregister_dynamic_object(identity);
        return 0;
    };
    if context.abi_version() != ABI_VERSION {
        unregister_dynamic_object(identity);
        return 0;
    }
    if context.closure_object_target(identity).is_some() {
        let context_ptr = context as *mut ElephcEvalContext;
        let receiver = context.closure_object_target(identity).and_then(|target| target.receiver());
        // Keep this closure registered while releasing its receiver: a nested
        // object destructor must not finalize the owning context underneath us.
        if let Some(receiver) = receiver {
            let mut values = ElephcRuntimeOps::with_context(context_ptr);
            let _ = values.release(receiver);
        }
        let should_finalize = context.forget_closure_object(identity);
        if should_finalize {
            unsafe { crate::ffi::context::finalize_eval_context_free(context_ptr) };
        }
        return 0;
    }
    // A generator the interpreter created is an instance of the compiled `Generator` class, not
    // of an eval class, but its frame lives in this context and has to go with the object:
    // `eval_dynamic_destructor_for_object_cell` runs its pending `finally` blocks and drops it.
    // Returning here left the frame registered under the freed ADDRESS, and the next compiled
    // generator allocated there was answered for by the dead one -- "finished" before it ran,
    // so an interpreted `yield from compiled_gen()` produced nothing the second time (every
    // Twig block after the first rendered empty).
    if context.dynamic_object_class(identity).is_none() && !context.has_eval_generator(identity) {
        unregister_dynamic_object(identity);
        return 0;
    }

    let mut values = ElephcRuntimeOps::with_context(context as *const ElephcEvalContext);
    let object_cell = match ElephcRuntimeOps::object_from_raw(object) {
        Ok(object_cell) => object_cell,
        Err(_) => {
            let (property_values, should_finalize) = context.forget_dynamic_object(identity);
            for value in property_values {
                let _ = values.release(value);
            }
            drop(values);
            if should_finalize {
                unsafe { crate::ffi::context::finalize_eval_context_free(context as *mut _) };
            }
            return 1;
        }
    };
    let destruct_result =
        eval_dynamic_destructor_for_object_cell(identity, object_cell, context, &mut values);
    let (property_values, should_finalize) = context.forget_dynamic_object(identity);
    let overlay_release_result = property_values
        .into_iter()
        .try_for_each(|value| values.release(value));
    let release_result = values.release(object_cell);
    let result = match (destruct_result, overlay_release_result, release_result) {
        (Ok(true), Ok(()), Ok(())) => 1,
        (Ok(false), Ok(()), Ok(())) => 0,
        (Err(EvalStatus::UnsupportedConstruct), _, _) => 1,
        (Err(_), _, _) | (_, Err(_), _) | (_, _, Err(_)) => 1,
    };
    drop(values);
    if should_finalize {
        unsafe { crate::ffi::context::finalize_eval_context_free(context as *mut _) };
    }
    result
}
