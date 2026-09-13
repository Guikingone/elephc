//! Purpose:
//! Manages mutable eval runtime state, execution scopes, resources, and call-site metadata.
//!
//! Called from:
//! - Interpreter execution and builtins requiring process-level eval state.
//!
//! Key details:
//! - Static cells, include state, scope stacks, errors, timezone, HTTP status, and magic paths live here.
//! - Internal array pointers live here too, because runtime array cells carry no
//!   PHP-visible cursor of their own.

use super::*;

/// Hands out the request-wide registration order of SPL autoload callbacks.
///
/// Appends count up from zero and prepends count down, so ordering the two together puts every
/// prepend ahead of every append and keeps each group in the order PHP would run it.
static EVAL_AUTOLOAD_APPEND_SEQUENCE: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(0);
static EVAL_AUTOLOAD_PREPEND_SEQUENCE: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(0);

/// Returns the next registration number for one callback.
fn next_autoload_registration_sequence(prepend: bool) -> i64 {
    use std::sync::atomic::Ordering;
    if prepend {
        EVAL_AUTOLOAD_PREPEND_SEQUENCE.fetch_sub(1, Ordering::Relaxed) - 1
    } else {
        EVAL_AUTOLOAD_APPEND_SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1
    }
}

impl ElephcEvalContext {
    /// Returns true when the context has a dynamic or native function with this lowercase PHP name.
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name) || self.native_functions.contains_key(name)
    }

    /// Returns true when the context has a closure registered under this synthetic name.
    pub fn has_closure(&self, name: &str) -> bool {
        self.closures.contains_key(name)
    }

    /// Returns a stored static local cell for an eval-declared function.
    ///
    /// Reads the one scope every `static` slot lives in, so a caller that asks between calls --
    /// `ReflectionFunction::getStaticVariables()` does -- sees what the last activation left, and
    /// an activation still on the stack sees what a deeper one wrote.
    pub fn static_local(&self, function_name: &str, name: &str) -> Option<RuntimeCellHandle> {
        self.static_scope
            .visible_cell(&Self::static_slot_name(function_name, name))
    }

    /// Returns the scope holding every `static` slot, for alias reads and writes.
    ///
    /// Handed out as a raw pointer for the same reason `global_scope_ptr` is: a write to a static
    /// reaches this through a shared context, and the alternative -- interior mutability -- is not
    /// `RefUnwindSafe` and would poison every `catch_unwind` in the FFI.
    pub fn static_scope_ptr(&self) -> *mut ElephcEvalScope {
        std::ptr::from_ref::<ElephcEvalScope>(&*self.static_scope).cast_mut()
    }

    /// Pushes the key this activation's `static` slots hang from.
    pub fn push_static_slot_key(&mut self, key: impl Into<String>) {
        self.static_slot_keys.push(key.into());
    }

    /// Pops the innermost static slot key override.
    pub fn pop_static_slot_key(&mut self) {
        self.static_slot_keys.pop();
    }

    /// Returns the key the running activation's `static` slots hang from.
    ///
    /// The override when one is in force, otherwise the function name -- which is right for a
    /// plain function, and right for a method, where php shares one slot across instances and
    /// with inheriting classes.
    pub fn current_static_slot_key(&self) -> Option<String> {
        self.static_slot_keys
            .last()
            .cloned()
            .or_else(|| self.current_function().map(str::to_string))
    }

    /// Names one `static` slot uniquely across every function that declares one.
    ///
    /// The separator is a NUL so it cannot collide with a slot key built from a PHP function name,
    /// which never contains one.
    pub fn static_slot_name(function_name: &str, name: &str) -> String {
        format!("{function_name}\0{name}")
    }

    /// Stores one static local cell and returns any replaced distinct cell.
    ///
    pub fn set_static_local(
        &mut self,
        function_name: impl Into<String>,
        name: impl Into<String>,
        cell: RuntimeCellHandle,
    ) -> Option<RuntimeCellHandle> {
        let slot = Self::static_slot_name(&function_name.into(), &name.into());
        let previous = self
            .static_scope
            .set(slot, cell, crate::scope::ScopeCellOwnership::Owned);
        previous.filter(|previous| *previous != cell)
    }

    /// Returns a stored static property cell for an eval-declared class.
    pub fn static_property(&self, class_name: &str, name: &str) -> Option<RuntimeCellHandle> {
        self.static_properties
            .get(&(normalize_class_name(class_name), name.to_string()))
            .copied()
    }

    /// Stores one eval static property cell and returns any replaced distinct cell.
    pub fn set_static_property(
        &mut self,
        class_name: &str,
        name: impl Into<String>,
        cell: RuntimeCellHandle,
    ) -> Option<RuntimeCellHandle> {
        let previous = self
            .static_properties
            .insert((normalize_class_name(class_name), name.into()), cell);
        previous.filter(|previous| *previous != cell)
    }

    /// Binds one eval static property slot to a persistent PHP reference target.
    pub fn bind_static_property_alias(
        &mut self,
        class_name: &str,
        name: &str,
        target: EvalReferenceTarget,
    ) -> Option<EvalReferenceTarget> {
        self.static_property_aliases
            .insert((normalize_class_name(class_name), name.to_string()), target)
    }

    /// Returns the persistent reference target bound to one eval static property slot.
    pub fn static_property_alias(
        &self,
        class_name: &str,
        name: &str,
    ) -> Option<&EvalReferenceTarget> {
        self.static_property_aliases
            .get(&(normalize_class_name(class_name), name.to_string()))
    }

    /// Returns a materialized eval class constant cell.
    pub fn class_constant_cell(&self, class_name: &str, name: &str) -> Option<RuntimeCellHandle> {
        self.class_constants
            .get(&(normalize_class_name(class_name), name.to_string()))
            .copied()
    }

    /// Stores one eval class constant cell and returns any replaced distinct cell.
    pub fn set_class_constant_cell(
        &mut self,
        class_name: &str,
        name: impl Into<String>,
        cell: RuntimeCellHandle,
    ) -> Option<RuntimeCellHandle> {
        let previous = self
            .class_constants
            .insert((normalize_class_name(class_name), name.into()), cell);
        previous.filter(|previous| *previous != cell)
    }

    /// Marks one class-like constant as under evaluation, detecting a self-referencing cycle.
    ///
    /// Returns `false` when the constant is already being evaluated on this call chain -- php's
    /// `Cannot declare self-referencing constant` condition -- so the caller can raise a
    /// catchable `Error` instead of recursing into the same uncached cell forever.
    pub fn begin_class_constant_evaluation(&mut self, class_name: &str, name: &str) -> bool {
        self.evaluating_class_constants
            .insert((normalize_class_name(class_name), name.to_string()))
    }

    /// Clears a class-like constant's in-progress marker once its value resolves or fails.
    pub fn end_class_constant_evaluation(&mut self, class_name: &str, name: &str) {
        self.evaluating_class_constants
            .remove(&(normalize_class_name(class_name), name.to_string()));
    }

    /// Returns the PHP internal array pointer tracked for one runtime array cell.
    ///
    /// Cells without a stored cursor answer `Position(0)`, matching PHP, where a
    /// freshly built array points at its first element.
    pub fn array_cursor(&self, array: RuntimeCellHandle) -> EvalArrayCursor {
        self.array_cursors
            .get(&(array.as_ptr() as usize))
            .copied()
            .unwrap_or(EvalArrayCursor::Position(0))
    }

    /// Stores the PHP internal array pointer for one runtime array cell.
    ///
    /// The default cursor is dropped instead of stored so a later array cell that
    /// reuses this address starts from PHP's fresh-array state.
    pub fn set_array_cursor(&mut self, array: RuntimeCellHandle, cursor: EvalArrayCursor) {
        let key = array.as_ptr() as usize;
        if cursor == EvalArrayCursor::Position(0) {
            self.array_cursors.remove(&key);
            return;
        }
        self.array_cursors.insert(key, cursor);
    }

    /// Queries the request's single include-state owner.
    pub fn has_included_file(&self, path: impl AsRef<std::path::Path>) -> bool {
        self.include_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(path.as_ref())
    }

    /// Registers an opened source once, without a context-local shadow copy.
    pub fn mark_included_file(&mut self, path: impl Into<std::path::PathBuf>) {
        self.include_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
            .mark(path.into());
    }

    /// Pushes whether the next interpreter program originates from an actual include file.
    pub fn push_include_execution(&mut self, is_include: bool) {
        self.include_execution_stack.push(is_include);
    }

    /// Restores the previous interpreter program origin after nested include or eval execution.
    pub fn pop_include_execution(&mut self) {
        self.include_execution_stack.pop();
    }

    /// Returns true only for statements parsed directly from the active include file.
    pub fn executing_include(&self) -> bool {
        self.include_execution_stack.last().copied().unwrap_or(false)
    }

    /// Claims one AOT-backed class-like declaration at its first runtime include point.
    pub fn claim_aot_include_classlike(&mut self, name: &str) -> bool {
        self.claimed_aot_include_classlikes
            .insert(normalize_class_name(name))
    }

    /// Stores the non-owned global scope handle used by eval `global` aliases.
    pub fn set_global_scope(&mut self, scope: *mut ElephcEvalScope) -> bool {
        if scope.is_null() {
            self.global_scope = None;
            self.owns_global_scope = false;
            false
        } else {
            self.global_scope = Some(scope);
            self.owns_global_scope = false;
            true
        }
    }

    /// Returns the non-owned global scope handle for eval `global` aliases.
    pub fn global_scope_ptr(&self) -> Option<*mut ElephcEvalScope> {
        self.global_scope
    }

    /// Reports whether this context has no retained SPL autoload callbacks.
    pub(crate) fn has_no_autoload_callbacks(&self) -> bool {
        self.autoload_callbacks.is_empty()
    }

    /// Stores one already-retained SPL autoload callback at PHP's requested position.
    ///
    /// The position is recorded as a request-wide number, not merely as a place in THIS context's
    /// list. PHP has one autoload queue per request and runs it in registration order, while
    /// elephc keeps each callback on the context that registered it, because that is where it is
    /// retained and released. Sorting the contexts' callbacks by this number reproduces the single
    /// queue -- an append takes the next number up, a prepend the next number down, so a prepend
    /// lands ahead of everything registered before it no matter which context owns it.
    pub(crate) fn register_autoload_callback(
        &mut self,
        callback: RuntimeCellHandle,
        prepend: bool,
    ) {
        let sequence = next_autoload_registration_sequence(prepend);
        if prepend {
            self.autoload_callbacks.insert(0, (sequence, callback));
        } else {
            self.autoload_callbacks.push((sequence, callback));
        }
    }

    /// Returns a snapshot of this context's callbacks in their PHP invocation order.
    pub(crate) fn autoload_callbacks(&self) -> Vec<RuntimeCellHandle> {
        self.autoload_callbacks
            .iter()
            .map(|(_, callback)| *callback)
            .collect()
    }

    /// Returns this context's callbacks with the request-wide order they were registered in.
    pub(crate) fn autoload_callbacks_ordered(&self) -> Vec<(i64, RuntimeCellHandle)> {
        self.autoload_callbacks.clone()
    }

    /// Removes the callback stored at one position, which the caller matched BY VALUE.
    ///
    /// PHP matches an autoload callback by value, not by cell identity: `spl_autoload_register`
    /// then `spl_autoload_unregister` with the same `'name'` written twice is two different cells
    /// and one callback. Identity matching made unregistration answer `false` for a callback that
    /// was plainly there, and let the same loader be registered twice.
    pub(crate) fn remove_autoload_callback_at(&mut self, index: usize) -> Option<RuntimeCellHandle> {
        (index < self.autoload_callbacks.len())
            .then(|| self.autoload_callbacks.remove(index))
            .map(|(_, callback)| callback)
    }

    /// Drains every retained callback when the surrounding PHP request ends.
    pub(crate) fn take_autoload_callbacks(&mut self) -> Vec<RuntimeCellHandle> {
        std::mem::take(&mut self.autoload_callbacks)
            .into_iter()
            .map(|(_, callback)| callback)
            .collect()
    }

    /// Starts one class-autoload attempt and rejects recursive attempts for the same class.
    pub(crate) fn begin_autoload_class(&mut self, class_name: &str) -> bool {
        self.autoloading_classes
            .insert(normalize_class_name(class_name))
    }

    /// Ends one class-autoload attempt, including error paths.
    pub(crate) fn end_autoload_class(&mut self, class_name: &str) {
        self.autoloading_classes
            .remove(&normalize_class_name(class_name));
    }

    /// Transfers the active eval global scope to a context retained beyond its AOT frame.
    pub(crate) fn retain_global_scope_for_request(
        &mut self,
        scope: *mut ElephcEvalScope,
    ) -> bool {
        if scope.is_null()
            || self.global_scope != Some(scope)
            || !self.has_retained_request_lifetime()
        {
            return false;
        }
        self.owns_global_scope = true;
        true
    }

    /// Returns the frame scope whose ownership was transferred to this context.
    pub(crate) fn take_owned_global_scope(&mut self) -> Option<*mut ElephcEvalScope> {
        if !self.owns_global_scope {
            return None;
        }
        self.owns_global_scope = false;
        self.global_scope.take()
    }

    /// Pushes an eval-executed function name for magic-constant resolution.
    pub fn push_function(&mut self, name: impl Into<String>) {
        self.function_stack.push(name.into());
    }

    /// Turns PHP's `declare(strict_types=1)` on or off, returning the previous setting.
    ///
    /// php scopes this to the file containing the code doing the coercing. An argument is coerced
    /// at the CALL, so the caller's file decides; a returned value is coerced at the `return`, so
    /// the callee's file decides. The flag therefore holds the mode of the file whose code is
    /// running right now: an include frame saves and restores it, and each body-execution site
    /// sets it to the callee's declaration stamp and puts the caller's back afterwards.
    pub fn set_strict_types(&mut self, strict_types: bool) -> bool {
        std::mem::replace(&mut self.strict_types, strict_types)
    }

    /// Returns whether scalar arguments and returns are checked strictly.
    pub const fn strict_types(&self) -> bool {
        self.strict_types
    }

    /// Sets `declare(ticks=N)`'s interval, returning the previous one.
    ///
    /// Zero means no tick directive is in force, which is the state every program starts in and
    /// the one `register_tick_function()` alone does not leave: php needs BOTH the directive and
    /// a registered handler before anything runs.
    pub fn set_tick_interval(&mut self, interval: i64) -> i64 {
        self.tick_counter = 0;
        std::mem::replace(&mut self.tick_interval, interval)
    }

    /// Returns the tick interval in force, or zero when ticking is off.
    pub const fn tick_interval(&self) -> i64 {
        self.tick_interval
    }

    /// Counts one executed statement and reports whether a tick is due.
    ///
    /// Measured with `php -n` 8.5.6: `declare(ticks=3)` over six statements fires twice, so the
    /// counter resets on each fire rather than testing a running total.
    pub fn tick_statement_is_due(&mut self) -> bool {
        if self.tick_interval <= 0 || self.tick_running || self.tick_functions.is_empty() {
            return false;
        }
        self.tick_counter += 1;
        if self.tick_counter < self.tick_interval {
            return false;
        }
        self.tick_counter = 0;
        true
    }

    /// Returns the registered tick handlers, in registration order.
    pub fn tick_functions(&self) -> Vec<RuntimeCellHandle> {
        self.tick_functions.clone()
    }

    /// Registers one tick handler, taking ownership of the retained callable.
    pub fn push_tick_function(&mut self, handler: RuntimeCellHandle) {
        self.tick_functions.push(handler);
    }

    /// Removes the tick handler at one index and hands its owned reference back.
    pub fn take_tick_function(&mut self, index: usize) -> Option<RuntimeCellHandle> {
        if index >= self.tick_functions.len() {
            return None;
        }
        Some(self.tick_functions.remove(index))
    }

    /// Marks a tick handler as running, returning the previous state.
    ///
    /// php does not tick inside a tick handler; without this the first handler statement would
    /// schedule another tick and the interpreter would recurse until it ran out of stack.
    pub fn set_tick_running(&mut self, running: bool) -> bool {
        std::mem::replace(&mut self.tick_running, running)
    }

    /// Marks whether the body about to run returns BY REFERENCE, returning the previous value.
    ///
    /// Saved and restored by the caller rather than kept on a stack, because the three places
    /// that run a body -- function, closure, method -- already own that save/restore shape and a
    /// parallel stack could desync from `function_stack` on an early return.
    pub fn set_returns_by_ref(&mut self, returns_by_ref: bool) -> bool {
        std::mem::replace(&mut self.returns_by_ref, returns_by_ref)
    }

    /// Returns whether the currently executing body was declared to return by reference.
    pub const fn returns_by_ref(&self) -> bool {
        self.returns_by_ref
    }

    /// Records the reference a by-reference `return` produced, with the value already RETAINED.
    ///
    /// Whatever outlives the call owns its reference: the callee's activation scope is drained
    /// the moment it returns, so the cell handed to the caller has to carry a reference of its
    /// own. Any previous unclaimed reference is given back, which is what a `&f();` statement --
    /// a by-reference call whose result nobody binds -- leaves behind.
    pub fn set_pending_return_reference(
        &mut self,
        target: EvalReferenceTarget,
        value: RuntimeCellHandle,
    ) -> Option<RuntimeCellHandle> {
        self.pending_return_reference
            .replace((target, value))
            .map(|(_, value)| value)
    }

    /// Takes the reference a by-reference `return` produced, handing its ownership to the caller.
    pub fn take_pending_return_reference(
        &mut self,
    ) -> Option<(EvalReferenceTarget, RuntimeCellHandle)> {
        self.pending_return_reference.take()
    }

    /// Pops the current eval-executed function name after its body completes.
    pub fn pop_function(&mut self) {
        self.function_stack.pop();
    }

    /// Returns the current eval-executed function name, if execution is inside one.
    pub fn current_function(&self) -> Option<&str> {
        self.function_stack.last().map(String::as_str)
    }

    /// Pushes the eval class whose method is currently executing.
    pub fn push_class_scope(&mut self, name: impl Into<String>) {
        self.class_stack.push(name.into());
    }

    /// Pops the current eval class method scope.
    pub fn pop_class_scope(&mut self) {
        self.class_stack.pop();
    }

    /// Returns the current eval class scope, if execution is inside a method.
    pub fn current_class_scope(&self) -> Option<&str> {
        self.class_stack.last().map(String::as_str)
    }

    /// Renders the whole class-scope stack, innermost last, for a diagnostic.
    ///
    /// A visibility refusal only ever prints the TOP of this stack, which says what the check
    /// compared but not how execution got there. Kept off every hot path on purpose: this is
    /// called from failure paths only, so it carries no `ELEPHC_EVAL_TRACE` lookup of its own.
    pub fn debug_class_scope_stack(&self) -> String {
        self.class_stack.join(" > ")
    }

    /// Enters a class-like-member-default evaluation (constant initializer, property default,
    /// enum case value), where `static::` has no live call frame to bind late.
    pub fn push_compile_time_constant_context(&mut self) {
        self.compile_time_constant_depth += 1;
    }

    /// Leaves a class-like-member-default evaluation started by
    /// `push_compile_time_constant_context`.
    pub fn pop_compile_time_constant_context(&mut self) {
        self.compile_time_constant_depth = self.compile_time_constant_depth.saturating_sub(1);
    }

    /// Returns whether evaluation is currently inside a class-like-member default, where php
    /// refuses `static::` as an uncatchable compile-time error rather than binding it late.
    pub fn in_compile_time_constant_context(&self) -> bool {
        self.compile_time_constant_depth > 0
    }

    /// Pushes the class name used to dispatch the current eval method call.
    pub fn push_called_class_scope(&mut self, name: impl Into<String>) {
        self.called_class_stack.push(name.into());
    }

    /// Pops the current late-static-bound eval class scope.
    pub fn pop_called_class_scope(&mut self) {
        self.called_class_stack.pop();
    }

    /// Returns the current late-static-bound eval class scope, if execution is inside a method.
    pub fn current_called_class_scope(&self) -> Option<&str> {
        self.called_class_stack.last().map(String::as_str)
    }

    /// Returns a dynamic called-class override for a generated/AOT frame entering eval.
    pub fn native_frame_called_class_override(
        &self,
        class_name: &str,
        called_class_name: &str,
    ) -> Option<String> {
        let class_name = class_name.trim_start_matches('\\');
        let called_class_name = called_class_name.trim_start_matches('\\');
        if class_name.is_empty() || !called_class_name.eq_ignore_ascii_case(class_name) {
            return None;
        }
        if let Some(called_class) =
            native_frame_called_class_override(class_name, called_class_name)
        {
            return Some(called_class);
        }
        let active = self.current_called_class_scope()?.trim_start_matches('\\');
        if active.is_empty() || active.eq_ignore_ascii_case(class_name) {
            return None;
        }
        let active = self
            .resolve_class_name(active)
            .unwrap_or_else(|| active.to_string());
        self.class_parent_names(&active)
            .iter()
            .any(|parent| parent.eq_ignore_ascii_case(class_name))
            .then_some(active)
    }

    /// Pushes PHP-visible method magic constants for the current eval method frame.
    pub fn push_method_magic_scope(&mut self, class_name: &str, method: &EvalClassMethod) {
        self.magic_stack.push(EvalMagicScope {
            function_name: method.magic_function_name().to_string(),
            method_name: method.magic_method_name(class_name),
            class_name: class_name.trim_start_matches('\\').to_string(),
            trait_name: method
                .trait_origin()
                .map(|trait_name| trait_name.trim_start_matches('\\').to_string())
                .unwrap_or_default(),
        });
    }

    /// Pushes PHP-visible class-like member magic constants for default expressions.
    pub fn push_class_like_member_magic_scope(
        &mut self,
        class_name: &str,
        trait_name: Option<&str>,
    ) {
        self.magic_stack.push(EvalMagicScope {
            function_name: String::new(),
            method_name: String::new(),
            class_name: class_name.trim_start_matches('\\').to_string(),
            trait_name: trait_name
                .map(|trait_name| trait_name.trim_start_matches('\\').to_string())
                .unwrap_or_default(),
        });
    }

    /// Pushes PHP-visible callable magic constants for reflected parameter defaults.
    pub fn push_callable_magic_scope(
        &mut self,
        function_name: &str,
        method_name: &str,
        class_name: Option<&str>,
        trait_name: Option<&str>,
    ) {
        self.magic_stack.push(EvalMagicScope {
            function_name: function_name.to_string(),
            method_name: method_name.to_string(),
            class_name: class_name
                .map(|class_name| class_name.trim_start_matches('\\').to_string())
                .unwrap_or_default(),
            trait_name: trait_name
                .map(|trait_name| trait_name.trim_start_matches('\\').to_string())
                .unwrap_or_default(),
        });
    }

    /// Pops the current PHP-visible eval magic-constant scope.
    pub fn pop_magic_scope(&mut self) {
        self.magic_stack.pop();
    }

    /// Returns the PHP `__FUNCTION__` value for the current eval frame.
    pub fn current_magic_function(&self) -> Option<&str> {
        self.magic_stack
            .last()
            .map(|scope| scope.function_name.as_str())
    }

    /// Returns the PHP `__METHOD__` value for the current eval method frame.
    pub fn current_magic_method(&self) -> Option<&str> {
        self.magic_stack
            .last()
            .map(|scope| scope.method_name.as_str())
    }

    /// Returns the PHP `__CLASS__` value for the current eval method frame.
    pub fn current_magic_class(&self) -> Option<&str> {
        self.magic_stack
            .last()
            .map(|scope| scope.class_name.as_str())
    }

    /// Returns the PHP `__TRAIT__` value for the current eval method frame.
    pub fn current_magic_trait(&self) -> Option<&str> {
        self.magic_stack
            .last()
            .map(|scope| scope.trait_name.as_str())
    }

    /// Captures the current eval execution stacks for later caller-context-sensitive work.
    pub fn execution_scope(&self) -> ElephcEvalExecutionScope {
        ElephcEvalExecutionScope {
            function_stack: self.function_stack.clone(),
            class_stack: self.class_stack.clone(),
            called_class_stack: self.called_class_stack.clone(),
        }
    }

    /// Replaces eval execution stacks and returns the previous stacks for restoration.
    pub fn replace_execution_scope(
        &mut self,
        scope: ElephcEvalExecutionScope,
    ) -> ElephcEvalExecutionScope {
        ElephcEvalExecutionScope {
            function_stack: std::mem::replace(&mut self.function_stack, scope.function_stack),
            class_stack: std::mem::replace(&mut self.class_stack, scope.class_stack),
            called_class_stack: std::mem::replace(
                &mut self.called_class_stack,
                scope.called_class_stack,
            ),
        }
    }

    /// Records a Throwable cell that escaped from an eval-executed function call.
    pub fn set_pending_throw(&mut self, value: RuntimeCellHandle) {
        self.pending_throw = Some(value);
    }

    /// Returns and clears the Throwable cell currently escaping through eval.
    pub fn take_pending_throw(&mut self) -> Option<RuntimeCellHandle> {
        self.pending_throw.take()
    }

    /// Returns the eval-local SPL autoload extension list.
    pub fn spl_autoload_extensions(&self) -> &str {
        &self.spl_autoload_extensions
    }

    /// Replaces the eval-local SPL autoload extension list.
    pub fn set_spl_autoload_extensions(&mut self, extensions: impl Into<String>) {
        self.spl_autoload_extensions = extensions.into();
    }

    /// Returns the eval-local stream resource table.
    pub(crate) fn stream_resources(&self) -> &EvalStreamResources {
        &self.streams
    }

    /// Returns mutable access to the eval-local stream resource table.
    pub(crate) fn stream_resources_mut(&mut self) -> &mut EvalStreamResources {
        &mut self.streams
    }

    /// Clears the eval-local JSON error state after a successful JSON operation.
    pub fn clear_json_error(&mut self) {
        self.json_last_error = 0;
        self.json_last_error_msg.clear();
        self.json_last_error_msg.push_str("No error");
    }

    /// Records the eval-local JSON error state for `json_last_error*()` calls.
    pub fn set_json_error(&mut self, code: i64, message: impl Into<String>) {
        self.json_last_error = code;
        self.json_last_error_msg = message.into();
    }

    /// Returns the PHP `JSON_ERROR_*` code for the last eval JSON operation.
    pub const fn json_last_error(&self) -> i64 {
        self.json_last_error
    }

    /// Returns the PHP message for the last eval JSON operation.
    pub fn json_last_error_msg(&self) -> &str {
        &self.json_last_error_msg
    }

    /// Returns the eval-local PHP default timezone identifier.
    pub fn default_timezone(&self) -> &str {
        &self.default_timezone
    }

    /// Replaces the eval-local PHP default timezone identifier.
    pub fn set_default_timezone(&mut self, timezone: impl Into<String>) {
        self.default_timezone = timezone.into();
    }

    /// Returns the eval-local HTTP response code used by web-facing builtins.
    pub const fn http_response_code(&self) -> i64 {
        self.http_response_code
    }

    /// Applies a new eval-local HTTP response code and returns the previous one.
    pub fn replace_http_response_code(&mut self, response_code: i64) -> i64 {
        let previous = self.http_response_code;
        if response_code > 0 {
            self.http_response_code = response_code;
        }
        previous
    }

    /// Updates the source file, directory, and line for the current eval call site.
    pub fn set_call_site(&mut self, file: impl Into<String>, dir: impl Into<String>, line: i64) {
        self.call_file = file.into();
        self.call_dir = dir.into();
        self.call_line = line;
        self.file_magic_override = None;
    }

    /// Moves the current line without disturbing the file or directory it belongs to.
    ///
    /// `set_call_site` replaces all three and clears the `__FILE__` override, which is right when
    /// ENTERING a file and wrong for every statement inside it. This is what an
    /// `EvalStmt::SourceLine` marker calls, and what makes a `debug_backtrace()` frame, a
    /// diagnostic's `on line N` and an `eval()`'d-code spelling name the statement that is
    /// actually running rather than the line the file was entered on.
    pub fn set_call_line(&mut self, line: i64) -> i64 {
        std::mem::replace(&mut self.call_line, line)
    }

    /// Returns the line currently being executed.
    pub const fn call_line(&self) -> i64 {
        self.call_line
    }

    /// Returns a copy of the current call-site metadata for temporary overrides.
    pub fn call_site(&self) -> (String, String, i64, Option<String>) {
        (
            self.call_file.clone(),
            self.call_dir.clone(),
            self.call_line,
            self.file_magic_override.clone(),
        )
    }

    /// Overrides `__FILE__` while executing an actual file through eval include.
    pub fn set_file_magic_override(&mut self, file: Option<String>) {
        self.file_magic_override = file;
    }

    /// Returns the source directory associated with the current eval call site.
    pub fn call_dir(&self) -> &str {
        &self.call_dir
    }

    /// Returns PHP's `__FILE__` string for code currently running inside eval.
    pub fn eval_file_magic(&self) -> String {
        if let Some(file) = &self.file_magic_override {
            return file.clone();
        }
        if self.call_file.is_empty() {
            return String::new();
        }
        format!("{}({}) : eval()'d code", self.call_file, self.call_line)
    }
}
