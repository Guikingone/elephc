//! Purpose:
//! Bridges EvalIR value operations to elephc runtime values.
//! The module wires the stateless adapter type while focused submodules own
//! generated C-ABI symbols, trait operations, and opcode mappings.
//!
//! Called from:
//! - `crate::ffi::execute::__elephc_eval_execute()` in non-test builds.
//!
//! Key details:
//! - The wrapper symbols adapt to elephc's target-specific internal helper ABI.
//! - Unit tests do not link the generated runtime object, so real hooks compile
//!   only outside `cfg(test)`.

#[cfg(not(test))]
mod externs;
#[cfg(not(test))]
mod ops;
#[cfg(not(test))]
mod tags;

#[cfg(not(test))]
use crate::errors::EvalStatus;
#[cfg(not(test))]
use crate::abi::ElephcEvalContext;
#[cfg(not(test))]
use crate::value::{RuntimeCell, RuntimeCellHandle};
#[cfg(not(test))]
use externs::{
    __elephc_eval_install_dynamic_object_destructor_hook,
    __elephc_eval_install_class_autoload_hook,
    __elephc_eval_install_generator_protocol_hook,
    __elephc_eval_install_object_relation_hook, __elephc_eval_install_serialize_object_hook,
    __elephc_eval_install_unserialize_object_hook,
    __elephc_eval_value_array_new,
    __elephc_eval_value_array_set, __elephc_eval_value_int, __elephc_eval_value_object_from_raw,
    __elephc_eval_value_release,
};

/// Runtime hook adapter that produces and consumes boxed elephc Mixed cells.
#[cfg(not(test))]
pub struct ElephcRuntimeOps {
    context: *const ElephcEvalContext,
}

#[cfg(not(test))]
impl ElephcRuntimeOps {
    /// Creates a runtime hook adapter without caller-sensitive eval context.
    pub const fn new() -> Self {
        Self {
            context: std::ptr::null(),
        }
    }

    /// Creates a runtime hook adapter that can expose the active class scope to generated helpers.
    pub const fn with_context(context: *const ElephcEvalContext) -> Self {
        Self { context }
    }

    /// Converts a runtime wrapper result into an interpreter handle.
    pub(crate) fn handle(ptr: *mut RuntimeCell) -> Result<RuntimeCellHandle, EvalStatus> {
        if ptr.is_null() {
            Err(EvalStatus::RuntimeFatal)
        } else {
            Ok(RuntimeCellHandle::from_raw(ptr))
        }
    }

    /// Boxes one borrowed raw object payload for eval `$this` dispatch.
    pub(crate) fn object_from_raw(
        object: *mut RuntimeCell,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_object_from_raw(object) })
    }

    /// Packs source-order argument cells into the boxed eval array ABI.
    fn arg_array(args: Vec<RuntimeCellHandle>) -> Result<RuntimeCellHandle, EvalStatus> {
        let arg_array = unsafe { __elephc_eval_value_array_new(args.len() as u64) };
        let mut arg_array = Self::handle(arg_array)?;
        for (index, value) in args.into_iter().enumerate() {
            let index = match Self::handle(unsafe { __elephc_eval_value_int(index as i64) }) {
                Ok(index) => index,
                Err(status) => {
                    unsafe { __elephc_eval_value_release(arg_array.as_ptr()); }
                    return Err(status);
                }
            };
            let updated = Self::handle(unsafe {
                __elephc_eval_value_array_set(arg_array.as_ptr(), index.as_ptr(), value.as_ptr())
            });
            // The setter borrows its key and retains its value. Only the fresh
            // index belongs to this adapter; the source argument stays borrowed.
            unsafe { __elephc_eval_value_release(index.as_ptr()); }
            match updated {
                Ok(array) => arg_array = array,
                Err(status) => {
                    unsafe { __elephc_eval_value_release(arg_array.as_ptr()); }
                    return Err(status);
                }
            }
        }
        Ok(arg_array)
    }

    /// Returns the active eval class-scope bytes in the generated helper ABI shape.
    fn current_class_scope_abi(&self) -> (*const u8, u64) {
        let Some(context) = (unsafe { self.context.as_ref() }) else {
            return (std::ptr::null(), 0);
        };
        let Some(class_scope) = context.current_class_scope() else {
            return (std::ptr::null(), 0);
        };
        (class_scope.as_ptr(), class_scope.len() as u64)
    }
}

/// Installs the eval dynamic object destructor callback into runtime data.
#[cfg(not(test))]
pub(crate) unsafe fn install_dynamic_object_destructor_hook(callback: usize) {
    unsafe {
        __elephc_eval_install_dynamic_object_destructor_hook(callback);
    }
}

/// Installs the eval `Generator` protocol callback into runtime data.
///
/// # Safety
/// `callback` must be the address of a
/// `extern "C" fn(*mut RuntimeCell, u64, *mut RuntimeCell, *mut u64) -> u64` following the
/// generator-protocol ABI; the `__rt_gen_*` helpers call through it before touching the
/// receiver's fiber fields.
#[cfg(not(test))]
pub(crate) unsafe fn install_generator_protocol_hook(callback: usize) {
    unsafe {
        __elephc_eval_install_generator_protocol_hook(callback);
    }
}

/// Installs the eval class-autoload callback into runtime data.
///
/// # Safety
/// `callback` must be the address of an `extern "C" fn(*const u8, u64) -> u64` following the
/// class-autoload ABI; the unserialize decoder calls through it for an unknown class name.
#[cfg(not(test))]
pub(crate) unsafe fn install_class_autoload_hook(callback: usize) {
    unsafe {
        __elephc_eval_install_class_autoload_hook(callback);
    }
}

/// Installs the eval unserialize-object callback into runtime data.
///
/// # Safety
/// `callback` must be the address of an
/// `extern "C" fn(*const u8, u64, u64) -> *mut RuntimeCell` following the unserialize-object
/// ABI; the decoder calls through it for a class with no compiled layout.
#[cfg(not(test))]
pub(crate) unsafe fn install_unserialize_object_hook(callback: usize) {
    unsafe {
        __elephc_eval_install_unserialize_object_hook(callback);
    }
}

/// Installs the eval object class-relation callback into the generated runtime.
///
/// # Safety
/// `callback` must be the address of an
/// `extern "C" fn(*mut RuntimeCell, *const u8, u64, u64) -> u64` following the object-relation
/// ABI; `__elephc_eval_value_is_a` calls through it when its AOT metadata answers false.
#[cfg(not(test))]
pub(crate) unsafe fn install_object_relation_hook(callback: usize) {
    unsafe {
        __elephc_eval_install_object_relation_hook(callback);
    }
}

/// Installs the eval serialize-object callback into the generated runtime.
///
/// # Safety
/// `callback` must be the address of an `extern "C" fn(*mut c_void) -> *mut RuntimeCell`
/// following the serialize-object ABI; `__rt_serialize_object` calls through it before reading a
/// class id the interpreter's own objects do not carry.
#[cfg(not(test))]
pub(crate) unsafe fn install_serialize_object_hook(callback: usize) {
    unsafe {
        __elephc_eval_install_serialize_object_hook(callback);
    }
}

/// Installs the eval output-buffering handler callback into the generated runtime.
///
/// # Safety
/// `callback` must be the address of a `fn(i64, *const u8, i64, i64) -> *mut RuntimeCell`
/// with the eval ob-handler ABI; the runtime calls through it on buffer flushes.
#[cfg(not(test))]
pub(crate) unsafe fn install_ob_handler_hook(callback: usize) {
    unsafe {
        externs::install_ob_handler_hook_raw(callback);
    }
}

/// Reports whether a compiled class has been LOADED, in php's `class_exists($n, false)` sense.
///
/// A closed-world build declares everything it compiled from the first instruction, but a class
/// the compiler pulled in ONLY so an existence probe could be answered is one php would never
/// have loaded. Those classes carry a request-scoped flag; every other compiled class -- and
/// every name the generated table does not list -- is loaded outright.
#[cfg(not(test))]
pub(crate) fn compiled_class_is_loaded(name: &str) -> bool {
    match deferred_class_flag(name) {
        Some(cell) => unsafe { cell.read() != 0 },
        None => true,
    }
}

/// Records that a probe which ALLOWS autoloading has now loaded this class.
#[cfg(not(test))]
pub(crate) fn mark_compiled_class_loaded(name: &str) {
    if let Some(cell) = deferred_class_flag(name) {
        unsafe { cell.write(1) };
    }
}

/// Returns the generated load flag for one class name, folded to php's case-insensitive form.
#[cfg(not(test))]
fn deferred_class_flag(name: &str) -> Option<*mut u64> {
    let folded = name.trim_start_matches('\\').to_ascii_lowercase();
    let cell = unsafe {
        externs::__elephc_eval_class_deferred_lookup(folded.as_ptr(), folded.len() as u64)
    };
    (!cell.is_null()).then_some(cell)
}

/// Test builds link no generated table, so every compiled class counts as loaded.
#[cfg(test)]
pub(crate) fn compiled_class_is_loaded(_name: &str) -> bool {
    true
}

#[cfg(test)]
pub(crate) fn mark_compiled_class_loaded(_name: &str) {}

/// Drives one COMPILED generator through the generated runtime's own helpers.
///
/// The interpreter cannot reach a compiled `Generator` through ordinary method dispatch: its
/// methods are runtime helpers, not registered native methods, so `valid()` on one fails. These
/// wrappers are the supported route, and they are what `yield from` uses when its delegate is a
/// generator the interpreter does not own.
///
/// Under `cfg(test)` there is no generated runtime to call, so each one answers the shape an
/// exhausted generator has. A unit test that reaches these has no compiled generator to drive.
#[cfg(not(test))]
pub(crate) fn native_generator_valid(generator: u64) -> bool {
    unsafe { externs::__elephc_eval_gen_valid(generator as *mut RuntimeCell) != 0 }
}

/// Returns an owned copy of a compiled generator's current value.
#[cfg(not(test))]
pub(crate) fn native_generator_current(generator: u64) -> Option<RuntimeCellHandle> {
    let value = unsafe { externs::__elephc_eval_gen_current(generator as *mut RuntimeCell) };
    (!value.is_null()).then(|| RuntimeCellHandle::from_raw(value))
}

/// Returns an owned copy of a compiled generator's current key.
#[cfg(not(test))]
pub(crate) fn native_generator_key(generator: u64) -> Option<RuntimeCellHandle> {
    let value = unsafe { externs::__elephc_eval_gen_key(generator as *mut RuntimeCell) };
    (!value.is_null()).then(|| RuntimeCellHandle::from_raw(value))
}

/// Advances a compiled generator to its next yield.
#[cfg(not(test))]
pub(crate) fn native_generator_next(generator: u64) {
    unsafe { externs::__elephc_eval_gen_next(generator as *mut RuntimeCell) };
}

/// Returns an owned copy of a compiled generator's `getReturn()` value.
#[cfg(not(test))]
pub(crate) fn native_generator_return(generator: u64) -> Option<RuntimeCellHandle> {
    let value = unsafe { externs::__elephc_eval_gen_get_return(generator as *mut RuntimeCell) };
    (!value.is_null()).then(|| RuntimeCellHandle::from_raw(value))
}

#[cfg(test)]
use crate::value::RuntimeCellHandle;

#[cfg(test)]
pub(crate) fn native_generator_valid(_generator: u64) -> bool {
    false
}

#[cfg(test)]
pub(crate) fn native_generator_current(_generator: u64) -> Option<RuntimeCellHandle> {
    None
}

#[cfg(test)]
pub(crate) fn native_generator_key(_generator: u64) -> Option<RuntimeCellHandle> {
    None
}

#[cfg(test)]
pub(crate) fn native_generator_next(_generator: u64) {}

#[cfg(test)]
pub(crate) fn native_generator_return(_generator: u64) -> Option<RuntimeCellHandle> {
    None
}
