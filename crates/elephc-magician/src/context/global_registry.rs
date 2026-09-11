//! Purpose:
//! Manages native-frame called-class overrides plus process-global eval class and include state.
//!
//! Called from:
//! - Native bridge entry points, include execution, and per-context class-like synchronization.
//!
//! Key details:
//! - Overrides are thread-local guards; global declarations and include keys are mutex-protected outside tests.

use super::*;

/// Late-static override installed while eval dispatches into a generated/AOT frame.
#[derive(Clone)]
pub(super) struct NativeFrameCalledClassOverride {
    #[cfg_attr(test, allow(dead_code))]
    context: *mut ElephcEvalContext,
    frame_class: String,
    called_class: String,
}

/// Scoped guard that removes one native-frame called-class override on drop.
pub(crate) struct NativeFrameCalledClassOverrideGuard;

/// Installs a late-static called-class override for a generated/AOT frame call.
pub(crate) fn push_native_frame_called_class_override(
    context: *mut ElephcEvalContext,
    frame_class: &str,
    called_class: &str,
) -> NativeFrameCalledClassOverrideGuard {
    NATIVE_FRAME_CALLED_CLASS_OVERRIDES.with(|overrides| {
        overrides
            .borrow_mut()
            .push(NativeFrameCalledClassOverride {
                context,
                frame_class: frame_class.trim_start_matches('\\').to_string(),
                called_class: called_class.trim_start_matches('\\').to_string(),
            });
    });
    NativeFrameCalledClassOverrideGuard
}

impl Drop for NativeFrameCalledClassOverrideGuard {
    /// Removes the most recent native-frame called-class override.
    fn drop(&mut self) {
        NATIVE_FRAME_CALLED_CLASS_OVERRIDES.with(|overrides| {
            overrides.borrow_mut().pop();
        });
    }
}

/// Returns the active thread-local late-static override for one generated/AOT frame.
pub(super) fn native_frame_called_class_override(
    frame_class: &str,
    called_class: &str,
) -> Option<String> {
    let frame_class = frame_class.trim_start_matches('\\');
    let called_class = called_class.trim_start_matches('\\');
    if frame_class.is_empty() || !called_class.eq_ignore_ascii_case(frame_class) {
        return None;
    }
    native_frame_called_class_override_for_frame(frame_class)
}

/// Returns the active called-class override for one generated/AOT frame class.
pub(crate) fn native_frame_called_class_override_for_frame(frame_class: &str) -> Option<String> {
    let frame_class = frame_class.trim_start_matches('\\');
    if frame_class.is_empty() {
        return None;
    }
    NATIVE_FRAME_CALLED_CLASS_OVERRIDES.with(|overrides| {
        overrides
            .borrow()
            .iter()
            .rev()
            .find(|entry| entry.frame_class.eq_ignore_ascii_case(frame_class))
            .map(|entry| entry.called_class.clone())
    })
}

/// Returns the active called-class override bytes for one generated/AOT frame class.
pub(crate) fn native_frame_called_class_override_bytes(
    frame_class: &str,
) -> Option<(*const u8, usize)> {
    let frame_class = frame_class.trim_start_matches('\\');
    if frame_class.is_empty() {
        return None;
    }
    NATIVE_FRAME_CALLED_CLASS_OVERRIDES.with(|overrides| {
        overrides
            .borrow()
            .iter()
            .rev()
            .find(|entry| entry.frame_class.eq_ignore_ascii_case(frame_class))
            .map(|entry| (entry.called_class.as_ptr(), entry.called_class.len()))
    })
}

/// Returns the active eval context and called class for one generated/AOT frame.
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn native_frame_called_class_override_context(
    frame_class: &str,
) -> Option<(*mut ElephcEvalContext, String)> {
    let frame_class = frame_class.trim_start_matches('\\');
    if frame_class.is_empty() {
        return None;
    }
    NATIVE_FRAME_CALLED_CLASS_OVERRIDES.with(|overrides| {
        overrides
            .borrow()
            .iter()
            .rev()
            .find(|entry| entry.frame_class.eq_ignore_ascii_case(frame_class))
            .map(|entry| (entry.context, entry.called_class.clone()))
    })
}

#[cfg(not(test))]
#[derive(Default)]
pub(super) struct GlobalEvalClassRegistry {
    pub(super) classes: HashMap<String, EvalClass>,
    pub(super) class_source_files: HashMap<String, String>,
    pub(super) declared_class_names: Vec<String>,
    pub(super) interfaces: HashMap<String, EvalInterface>,
    pub(super) declared_interface_names: Vec<String>,
    pub(super) traits: HashMap<String, EvalTrait>,
    pub(super) declared_trait_names: Vec<String>,
    pub(super) enums: HashMap<String, EvalEnum>,
    pub(super) declared_enum_names: Vec<String>,
    pub(super) aliases: HashMap<String, EvalClassAlias>,
}

/// Immutable AOT metadata shared with fallback contexts created without a generated frame.
///
/// Dynamic class-like declarations stay in `GlobalEvalClassRegistry`; this snapshot contains
/// only compiler-provided declarations and signatures that are valid for the whole binary.
///
/// Every table is reference-counted rather than owned: the first generated context builds
/// them, publishes the handles, and each later context adopts the same allocations instead
/// of deep-copying a Symfony-sized registration stream per request. Contexts only ever read
/// a synced table; the registration setters go through `Arc::make_mut`, so a context that
/// registers after adopting the snapshot silently forks its own table and the published one
/// is never observed to change.
#[cfg(not(test))]
pub(super) struct GlobalEvalAotMetadata {
    native_global_sync: Option<NativeGlobalSyncHooks>,
    declared_class_names: Arc<Vec<String>>,
    declared_interface_names: Arc<Vec<String>>,
    declared_trait_names: Arc<Vec<String>>,
    native_functions: Arc<HashMap<String, NativeFunction>>,
    native_methods: Arc<HashMap<(String, String), NativeCallableSignature>>,
    native_static_methods: Arc<HashMap<(String, String), NativeCallableSignature>>,
    native_constructors: Arc<HashMap<String, NativeCallableSignature>>,
    native_class_parents: Arc<HashMap<String, String>>,
    native_class_attributes: Arc<HashMap<String, Vec<EvalAttribute>>>,
    native_method_attributes: Arc<HashMap<(String, String), Vec<EvalAttribute>>>,
    native_constant_attributes: Arc<HashMap<(String, String), Vec<EvalAttribute>>>,
    native_interface_properties: Arc<HashMap<String, Vec<(String, EvalInterfaceProperty)>>>,
    native_abstract_properties: Arc<HashMap<String, Vec<(String, EvalInterfaceProperty)>>>,
    native_property_types: Arc<HashMap<(String, String), EvalParameterType>>,
    native_property_defaults: Arc<HashMap<(String, String), NativeCallableDefault>>,
    native_property_attributes: Arc<HashMap<(String, String), Vec<EvalAttribute>>>,
}

/// Returns the process-local snapshot of generated AOT metadata.
#[cfg(not(test))]
fn global_eval_aot_metadata() -> &'static Mutex<Option<Arc<GlobalEvalAotMetadata>>> {
    GLOBAL_EVAL_AOT_METADATA.get_or_init(|| Mutex::new(None))
}

/// Publishes compiler-provided metadata after one generated context has finished registration.
#[cfg(not(test))]
pub(crate) fn publish_global_eval_aot_metadata(context: &ElephcEvalContext) {
    let Ok(mut metadata) = global_eval_aot_metadata().lock() else {
        return;
    };
    *metadata = Some(Arc::new(GlobalEvalAotMetadata {
            native_global_sync: context.native_global_sync,
            declared_class_names: Arc::clone(&context.declared_class_names),
            declared_interface_names: Arc::clone(&context.declared_interface_names),
            declared_trait_names: Arc::clone(&context.declared_trait_names),
            native_functions: Arc::clone(&context.native_functions),
            native_methods: Arc::clone(&context.native_methods),
            native_static_methods: Arc::clone(&context.native_static_methods),
            native_constructors: Arc::clone(&context.native_constructors),
            native_class_parents: Arc::clone(&context.native_class_parents),
            native_class_attributes: Arc::clone(&context.native_class_attributes),
            native_method_attributes: Arc::clone(&context.native_method_attributes),
            native_constant_attributes: Arc::clone(&context.native_constant_attributes),
            native_interface_properties: Arc::clone(&context.native_interface_properties),
            native_abstract_properties: Arc::clone(&context.native_abstract_properties),
            native_property_types: Arc::clone(&context.native_property_types),
            native_property_defaults: Arc::clone(&context.native_property_defaults),
            native_property_attributes: Arc::clone(&context.native_property_attributes),
    }));
}

/// Imports compiler-provided metadata into a newly created context.
///
/// Returns whether a process-global AOT snapshot was available. Generated
/// contexts use this to skip replaying the complete registration stream after
/// the first context has published immutable module metadata.
#[cfg(not(test))]
pub(crate) fn sync_global_eval_aot_metadata(context: &mut ElephcEvalContext) -> bool {
    let Some(metadata) = global_eval_aot_metadata()
        .lock()
        .ok()
        .and_then(|metadata| metadata.clone())
    else {
        return false;
    };
    context.declared_class_names = Arc::clone(&metadata.declared_class_names);
    context.native_global_sync = metadata.native_global_sync;
    context.declared_interface_names = Arc::clone(&metadata.declared_interface_names);
    context.declared_trait_names = Arc::clone(&metadata.declared_trait_names);
    context.native_functions = Arc::clone(&metadata.native_functions);
    context.native_methods = Arc::clone(&metadata.native_methods);
    context.native_static_methods = Arc::clone(&metadata.native_static_methods);
    context.native_constructors = Arc::clone(&metadata.native_constructors);
    context.native_class_parents = Arc::clone(&metadata.native_class_parents);
    context.native_class_attributes = Arc::clone(&metadata.native_class_attributes);
    context.native_method_attributes = Arc::clone(&metadata.native_method_attributes);
    context.native_constant_attributes = Arc::clone(&metadata.native_constant_attributes);
    context.native_interface_properties = Arc::clone(&metadata.native_interface_properties);
    context.native_abstract_properties = Arc::clone(&metadata.native_abstract_properties);
    context.native_property_types = Arc::clone(&metadata.native_property_types);
    context.native_property_defaults = Arc::clone(&metadata.native_property_defaults);
    context.native_property_attributes = Arc::clone(&metadata.native_property_attributes);
    true
}

/// Imports the module's published AOT metadata into a context that has never received any.
///
/// A context allocated before the generated module published its snapshot keeps an empty callable
/// table for the rest of the request, because the import happens once, at creation. An AOT
/// `spl_autoload_register()` produces exactly that: the ABI always hands it a null handle, so it
/// allocates the owner context on the spot, and the module's registration helper usually has not
/// run yet at that point. The snapshot is a module constant, so a context still holding nothing of
/// its own can take it later without overwriting anything.
///
/// Returns whether the metadata was imported by this call.
#[cfg(not(test))]
pub(crate) fn sync_global_eval_aot_metadata_when_empty(context: &mut ElephcEvalContext) -> bool {
    if !context.native_functions.is_empty()
        || !context.native_methods.is_empty()
        || !context.native_static_methods.is_empty()
        || !context.native_constructors.is_empty()
    {
        return false;
    }
    sync_global_eval_aot_metadata(context)
}

/// Returns the process-local eval class registry for generated-code eval contexts.
#[cfg(not(test))]
pub(super) fn global_eval_classes() -> &'static Mutex<GlobalEvalClassRegistry> {
    GLOBAL_EVAL_CLASSES.get_or_init(|| Mutex::new(GlobalEvalClassRegistry::default()))
}

/// Returns the process-local map from PHP function name to its owning eval context.
#[cfg(not(test))]
fn global_eval_functions() -> &'static Mutex<HashMap<String, usize>> {
    GLOBAL_EVAL_FUNCTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Returns the request-global owners that retain SPL autoload callback tables.
#[cfg(not(test))]
fn global_eval_autoload_contexts() -> &'static Mutex<Vec<usize>> {
    GLOBAL_EVAL_AUTOLOAD_CONTEXTS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Snapshots every live request-global SPL autoload context without holding its registry lock.
#[cfg(not(test))]
pub(crate) fn global_eval_autoload_contexts_snapshot() -> Vec<*mut ElephcEvalContext> {
    global_eval_autoload_contexts()
        .lock()
        .map(|contexts| {
            contexts
                .iter()
                .copied()
                .map(|context| context as *mut ElephcEvalContext)
                .collect()
        })
        .unwrap_or_default()
}

/// Clears request-scoped dynamic class-like declarations without discarding AOT metadata.
#[cfg(not(test))]
pub(crate) fn reset_global_eval_classes() {
    let mut registry = global_eval_classes()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // These are declarations executed by the previous request, not immutable AOT
    // metadata. Retaining them would make an ordinary include in the next request
    // fail with a redeclaration error. The separate AOT snapshot remains reusable.
    *registry = GlobalEvalClassRegistry::default();
}

/// Clears the process-local include registry at a generated web request boundary.
#[cfg(not(test))]
pub(crate) fn reset_global_eval_included_files() {
    super::request_includes::current_include_state()
        .lock().unwrap_or_else(std::sync::PoisonError::into_inner).reset();
}

/// Releases request-scoped dynamic function owners before the web heap is recycled.
#[cfg(not(test))]
pub(crate) fn reset_global_eval_function_contexts() {
    let owners = {
        let mut functions = global_eval_functions()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::take(&mut *functions)
            .into_values()
            .collect::<Vec<_>>()
    };
    for owner in owners {
        let context = owner as *mut ElephcEvalContext;
        let should_finalize = unsafe { context.as_ref() }
            .is_some_and(ElephcEvalContext::forget_global_function);
        if should_finalize {
            unsafe { crate::ffi::context::finalize_eval_context_free(context) };
        }
    }
}

/// Releases every request-global SPL autoload callback table before recycling the web heap.
#[cfg(not(test))]
pub(crate) fn reset_global_eval_autoload_contexts() {
    let contexts = {
        let mut contexts = global_eval_autoload_contexts()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::take(&mut *contexts).into_iter().collect::<Vec<_>>()
    };
    for context in contexts {
        release_global_eval_autoload_context(context as *mut ElephcEvalContext);
    }
}

/// Keeps crate unit tests isolated without introducing process-global include state.
#[cfg(test)]
pub(crate) fn reset_global_eval_included_files() {}

/// Keeps crate unit tests isolated without process-global dynamic function state.
#[cfg(test)]
pub(crate) fn reset_global_eval_function_contexts() {}

/// Keeps crate unit tests isolated without process-global SPL autoload state.
#[cfg(test)]
pub(crate) fn reset_global_eval_autoload_contexts() {}

/// Keeps crate unit tests independent from process-global dynamic function cleanup.
#[cfg(test)]
pub(crate) fn unregister_global_eval_functions_for_context(_context: *mut ElephcEvalContext) {}

/// Lets test-local contexts register callbacks without introducing process-global ownership.
#[cfg(test)]
pub(crate) fn register_global_eval_autoload_context(
    _context: *mut ElephcEvalContext,
    _prepend: bool,
) -> bool {
    true
}

/// Keeps test callback unregistration local to the owning context.
#[cfg(test)]
pub(crate) fn unregister_global_eval_autoload_context(_context: *mut ElephcEvalContext) {}

/// Registers a dynamic PHP function name to the context that declared it.
#[cfg(not(test))]
pub(super) fn register_global_eval_function(
    name: &str,
    context: *mut ElephcEvalContext,
) -> bool {
    if context.is_null() {
        return false;
    }
    let key = normalize_global_function_name(name);
    if key.is_empty() {
        return false;
    }
    let Ok(mut functions) = global_eval_functions().lock() else {
        return false;
    };
    match functions.get(&key).copied() {
        Some(existing) => existing == context as usize,
        None => {
            functions.insert(key, context as usize);
            let Some(context) = (unsafe { context.as_ref() }) else {
                return false;
            };
            context.retain_global_function();
            true
        }
    }
}

/// Records that a context now owns at least one request-global SPL autoload callback.
#[cfg(not(test))]
pub(crate) fn register_global_eval_autoload_context(
    context: *mut ElephcEvalContext,
    prepend: bool,
) -> bool {
    if context.is_null() {
        return false;
    }
    let Ok(mut contexts) = global_eval_autoload_contexts().lock() else {
        return false;
    };
    if contexts.contains(&(context as usize)) {
        return true;
    }
    if prepend {
        contexts.insert(0, context as usize);
    } else {
        contexts.push(context as usize);
    }
    let Some(context) = (unsafe { context.as_ref() }) else {
        return false;
    };
    context.retain_autoload_context();
    true
}

/// Removes a context after its final SPL autoload callback was unregistered.
#[cfg(not(test))]
pub(crate) fn unregister_global_eval_autoload_context(context: *mut ElephcEvalContext) {
    if context.is_null() {
        return;
    }
    let removed = global_eval_autoload_contexts()
        .lock()
        .ok()
        .is_some_and(|mut contexts| {
            contexts
                .iter()
                .position(|owner| *owner == context as usize)
                .map(|index| contexts.remove(index))
                .is_some()
        });
    if removed {
        release_global_eval_autoload_context(context);
    }
}

/// Finds the live context that declared one dynamically included PHP function.
#[cfg(not(test))]
pub(crate) fn global_eval_function_owner_context(
    name: &str,
) -> Option<*mut ElephcEvalContext> {
    let key = normalize_global_function_name(name);
    let functions = global_eval_functions().lock().ok()?;
    functions
        .get(&key)
        // PHP resolves an unqualified call made in a namespace against that namespace first,
        // then retries the global function table. Codegen supplies the resolved namespace form,
        // so reproduce the second probe for dynamic declarations discovered at runtime.
        .or_else(|| key.rsplit_once('\\').and_then(|(_, bare)| functions.get(bare)))
        .copied()
        .map(|context| context as *mut ElephcEvalContext)
}

/// Removes every function-owner entry associated with a context being destroyed.
#[cfg(not(test))]
pub(crate) fn unregister_global_eval_functions_for_context(context: *mut ElephcEvalContext) {
    if context.is_null() {
        return;
    }
    let context = context as usize;
    if let Ok(mut functions) = global_eval_functions().lock() {
        functions.retain(|_, owner| *owner != context);
    }
}

/// Releases retained callback cells and then drops one request-global autoload ownership root.
#[cfg(not(test))]
fn release_global_eval_autoload_context(context: *mut ElephcEvalContext) {
    let Some(context_ref) = (unsafe { context.as_mut() }) else {
        return;
    };
    let callbacks = context_ref.take_autoload_callbacks();
    let mut values = crate::runtime_hooks::ElephcRuntimeOps::with_context(context.cast_const());
    for callback in callbacks {
        let _ = crate::interpreter::RuntimeValueOps::release(&mut values, callback);
    }
    if context_ref.forget_autoload_context() {
        unsafe { crate::ffi::context::finalize_eval_context_free(context) };
    }
}

/// Canonicalizes PHP function names for the process-global case-insensitive registry.
#[cfg(not(test))]
fn normalize_global_function_name(name: &str) -> String {
    name.trim_start_matches('\\').to_ascii_lowercase()
}

/// Records one eval-declared class so later eval contexts can see PHP-global metadata.
#[cfg(not(test))]
pub(super) fn register_global_eval_class(class: &EvalClass) {
    let key = normalize_class_name(class.name());
    if let Ok(mut registry) = global_eval_classes().lock() {
        if !registry.classes.contains_key(&key) {
            registry.declared_class_names.push(class.name().to_string());
        }
        registry.classes.insert(key, class.clone());
    }
}

/// Records the physical declaration file for a process-global eval class.
#[cfg(not(test))]
pub(super) fn register_global_eval_class_source(class_name: &str, file: String) {
    if file.is_empty() {
        return;
    }
    let key = normalize_class_name(class_name);
    if let Ok(mut registry) = global_eval_classes().lock() {
        registry.class_source_files.insert(key, file);
    }
}

/// Records one eval-declared interface so later eval contexts can see PHP-global metadata.
#[cfg(not(test))]
pub(super) fn register_global_eval_interface(interface: &EvalInterface) {
    let key = normalize_class_name(interface.name());
    if let Ok(mut registry) = global_eval_classes().lock() {
        if !registry.interfaces.contains_key(&key) {
            registry
                .declared_interface_names
                .push(interface.name().to_string());
        }
        registry.interfaces.insert(key, interface.clone());
    }
}

/// Records one eval-declared trait so later eval contexts can see PHP-global metadata.
#[cfg(not(test))]
pub(super) fn register_global_eval_trait(trait_decl: &EvalTrait) {
    let key = normalize_class_name(trait_decl.name());
    if let Ok(mut registry) = global_eval_classes().lock() {
        if !registry.traits.contains_key(&key) {
            registry
                .declared_trait_names
                .push(trait_decl.name().to_string());
        }
        registry.traits.insert(key, trait_decl.clone());
    }
}

/// Records one eval-declared enum so later eval contexts can see PHP-global metadata.
#[cfg(not(test))]
pub(super) fn register_global_eval_enum(enum_decl: &EvalEnum) {
    let key = normalize_class_name(enum_decl.name());
    if let Ok(mut registry) = global_eval_classes().lock() {
        if !registry.enums.contains_key(&key) {
            registry
                .declared_enum_names
                .push(enum_decl.name().trim_start_matches('\\').to_string());
        }
        registry.enums.insert(key, enum_decl.clone());
    }
}

/// Records one eval-defined class-like alias for later generated eval contexts.
#[cfg(not(test))]
pub(super) fn register_global_eval_alias(alias_name: &str, alias: &EvalClassAlias) {
    let key = normalize_class_name(alias_name);
    if let Ok(mut registry) = global_eval_classes().lock() {
        registry.aliases.insert(key, alias.clone());
    }
}
