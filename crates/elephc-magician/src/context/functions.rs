//! Purpose:
//! Registers dynamic constants, functions, closures, and native function metadata.
//!
//! Called from:
//! - Declaration execution, closure creation, and dynamic function dispatch.
//!
//! Key details:
//! - Synthetic closure identities and native parameter metadata remain context-local.

use super::*;

impl ElephcEvalContext {
    /// Defines an eval dynamic constant value, failing if the name is invalid or already present.
    pub fn define_constant(&mut self, name: &str, value: RuntimeCellHandle) -> bool {
        let key = normalize_constant_name(name);
        if key.is_empty() || self.constants.contains_key(&key) {
            return false;
        }
        self.constants.insert(key, value);
        true
    }

    /// Returns true when this eval context has a dynamic constant with the requested name.
    pub fn has_constant(&self, name: &str) -> bool {
        self.constants.contains_key(&normalize_constant_name(name))
    }

    /// Returns an eval dynamic constant value by case-sensitive PHP constant name.
    pub fn constant(&self, name: &str) -> Option<RuntimeCellHandle> {
        self.constants.get(&normalize_constant_name(name)).copied()
    }

    /// Defines a dynamic user function, failing if the name already exists.
    pub fn define_function(
        &mut self,
        name: impl Into<String>,
        function: EvalFunction,
    ) -> Result<(), EvalFunction> {
        let name = name.into();
        if self.functions.contains_key(&name) || self.native_functions.contains_key(&name) {
            return Err(function);
        }
        #[cfg(not(test))]
        if !crate::context::register_global_eval_function(&name, self as *mut Self) {
            return Err(function);
        }
        self.functions.insert(name, function);
        Ok(())
    }

    /// Stores one eval closure instance under a context-local synthetic callable name.
    pub fn define_closure(&mut self, closure: EvalClosure) -> String {
        let name = format!("{{closure:eval:{}}}", self.next_closure_id);
        self.next_closure_id += 1;
        self.closures.insert(name.clone(), closure);
        name
    }

    /// Associates a PHP `Closure` object identity with an eval closure callable name.
    pub fn register_closure_object(&mut self, identity: u64, closure_name: &str) {
        self.register_closure_object_target(
            identity,
            EvalClosureObjectTarget::Named(closure_name.to_string()),
        );
    }

    /// Associates a PHP `Closure` object identity with any eval callable target.
    pub fn register_closure_object_target(
        &mut self,
        identity: u64,
        target: EvalClosureObjectTarget,
    ) {
        if self.closure_objects.insert(identity, target).is_none() {
            self.live_closure_objects
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        crate::ffi::dynamic_destructors::register_dynamic_object_context(
            identity,
            self as *mut Self,
        );
    }

    /// Returns the callable target bound to a PHP `Closure` object.
    pub fn closure_object_target(&self, identity: u64) -> Option<&EvalClosureObjectTarget> {
        self.closure_objects.get(&identity)
    }

    /// Returns the eval closure callable name bound to a literal PHP `Closure` object.
    pub fn closure_object_name(&self, identity: u64) -> Option<&str> {
        self.closure_objects
            .get(&identity)
            .and_then(|target| match target {
                EvalClosureObjectTarget::Named(name)
                | EvalClosureObjectTarget::BoundNamed { name, .. } => Some(name.as_str()),
                _ => None,
            })
    }

    /// Marks disposal as pending and reports whether no request-global value still owns this context.
    ///
    /// A closure or a function declared by a dynamic include can outlive the AOT frame that
    /// executed that include. Their executable metadata remains in this context until the
    /// associated object is released or the web request reaches its reset boundary.
    pub fn request_retained_context_free(&self) -> bool {
        self.retained_context_free_requested
            .store(true, std::sync::atomic::Ordering::Release);
        !self.has_retained_request_lifetime()
    }

    /// Removes a released Closure object and reports whether it completes a deferred context free.
    pub fn forget_closure_object(&mut self, identity: u64) -> bool {
        if self.closure_objects.remove(&identity).is_none() {
            return false;
        }
        crate::ffi::dynamic_destructors::unregister_dynamic_object(identity);
        let previous = self
            .live_closure_objects
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        debug_assert!(previous > 0, "closure object lifetime count must not underflow");
        previous == 1 && self.should_finalize_retained_context()
    }

    /// Releases one eval-class object lifetime and reports whether deferred context disposal can finish.
    pub(crate) fn forget_dynamic_object_owner(&self) -> bool {
        let previous = self
            .live_dynamic_objects
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        debug_assert!(previous > 0, "dynamic object lifetime count must not underflow");
        previous == 1 && self.should_finalize_retained_context()
    }

    /// Adds one request-global dynamic-function owner to this context.
    pub(crate) fn retain_global_function(&self) {
        self.live_global_functions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Removes one request-global dynamic-function owner and reports deferred-finalization state.
    pub(crate) fn forget_global_function(&self) -> bool {
        let previous = self
            .live_global_functions
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        debug_assert!(previous > 0, "global function lifetime count must not underflow");
        previous == 1 && self.should_finalize_retained_context()
    }

    /// Retains this context while its request-global SPL autoload table is non-empty.
    pub(crate) fn retain_autoload_context(&self) {
        self.live_autoload_contexts
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Releases the request-global SPL autoload ownership and reports finalization state.
    pub(crate) fn forget_autoload_context(&self) -> bool {
        let previous = self
            .live_autoload_contexts
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        debug_assert!(previous > 0, "autoload context lifetime count must not underflow");
        previous == 1 && self.should_finalize_retained_context()
    }

    /// Reports whether the context must outlive its originating AOT frame.
    pub(crate) fn has_retained_request_lifetime(&self) -> bool {
        self.live_closure_objects
            .load(std::sync::atomic::Ordering::Acquire)
            != 0
            || self
                .live_dynamic_objects
                .load(std::sync::atomic::Ordering::Acquire)
                != 0
            || self
                .live_global_functions
                .load(std::sync::atomic::Ordering::Acquire)
                != 0
            || self
                .live_autoload_contexts
                .load(std::sync::atomic::Ordering::Acquire)
                != 0
    }

    /// Reports whether a prior frame cleanup may now finalize this retained context.
    fn should_finalize_retained_context(&self) -> bool {
        self.retained_context_free_requested
            .load(std::sync::atomic::Ordering::Acquire)
            && !self.has_retained_request_lifetime()
    }

    /// Defines a generated native function callback, failing if the name already exists.
    pub fn define_native_function(
        &mut self,
        name: impl Into<String>,
        function: NativeFunction,
    ) -> Result<(), NativeFunction> {
        let name = name.into();
        if self.functions.contains_key(&name) || self.native_functions.contains_key(&name) {
            return Err(function);
        }
        Arc::make_mut(&mut self.native_functions).insert(name, function);
        Ok(())
    }

    /// Returns a dynamic user function by its lowercase PHP function name.
    pub fn function(&self, name: &str) -> Option<&EvalFunction> {
        self.functions.get(name)
    }

    /// Returns a dynamic eval closure by its synthetic callable name.
    pub fn closure(&self, name: &str) -> Option<&EvalClosure> {
        self.closures.get(name)
    }

    /// Returns a native AOT function callback by its lowercase PHP function name.
    pub fn native_function(&self, name: &str) -> Option<NativeFunction> {
        self.native_functions.get(name).cloned()
    }

    /// Records one parameter name for an already registered native AOT callback.
    pub fn define_native_function_param(
        &mut self,
        function_name: &str,
        index: usize,
        param_name: impl Into<String>,
    ) -> bool {
        Arc::make_mut(&mut self.native_functions)
            .get_mut(function_name)
            .is_some_and(|function| function.set_param_name(index, param_name))
    }

    /// Records one parameter type for an already registered native AOT callback.
    pub fn define_native_function_param_type(
        &mut self,
        function_name: &str,
        index: usize,
        param_type: EvalParameterType,
    ) -> bool {
        Arc::make_mut(&mut self.native_functions)
            .get_mut(function_name)
            .is_some_and(|function| function.set_param_type(index, param_type))
    }

    /// Records one parameter default for an already registered native AOT callback.
    pub fn define_native_function_param_default(
        &mut self,
        function_name: &str,
        index: usize,
        default: NativeCallableDefault,
    ) -> bool {
        Arc::make_mut(&mut self.native_functions)
            .get_mut(function_name)
            .is_some_and(|function| function.set_param_default(index, default))
    }

    /// Records whether one native AOT callback parameter is by-reference.
    pub fn define_native_function_param_by_ref(
        &mut self,
        function_name: &str,
        index: usize,
        by_ref: bool,
    ) -> bool {
        Arc::make_mut(&mut self.native_functions)
            .get_mut(function_name)
            .is_some_and(|function| function.set_param_by_ref(index, by_ref))
    }

    /// Records which native AOT callback parameter is variadic.
    pub fn define_native_function_variadic_param(
        &mut self,
        function_name: &str,
        index: usize,
    ) -> bool {
        Arc::make_mut(&mut self.native_functions)
            .get_mut(function_name)
            .is_some_and(|function| function.set_variadic_index(index))
    }

    /// Records one native AOT callback return type.
    pub fn define_native_function_return_type(
        &mut self,
        function_name: &str,
        return_type: EvalParameterType,
    ) -> bool {
        Arc::make_mut(&mut self.native_functions)
            .get_mut(function_name)
            .is_some_and(|function| {
                function.set_return_type(return_type);
                true
            })
    }

    /// Records whether eval may dispatch a native AOT callback through its bridge.
    pub fn define_native_function_bridge_supported(
        &mut self,
        function_name: &str,
        supported: bool,
    ) -> bool {
        Arc::make_mut(&mut self.native_functions)
            .get_mut(function_name)
            .is_some_and(|function| {
                function.set_bridge_supported(supported);
                true
            })
    }
}
