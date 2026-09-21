//! Purpose:
//! Stores module-wide codegen artifacts that can be shared across function contexts.
//! Deduplicates runtime callable descriptors, wrappers, and invokers by semantic shape.
//!
//! Called from:
//! - `crate::codegen::block_emit::emit_module()` creates one state per generated module.
//! - `crate::codegen::lower_inst::callables` reuses emitted callable artifacts through it.
//!
//! Key details:
//! - Cached labels are global assembly entries emitted at their first call site.
//! - Receiver-bearing descriptors cache only immutable templates; each call still captures its object.
//! - Owns the module-wide assembly label counter. It must not be per function: the readable part
//!   of a label is a lossy fragment of the PHP function/block name, so only a module-unique
//!   trailing id keeps two functions with similar names from emitting the same label.
//! - Indexes emitted instance and static method bodies once from immutable EIR module metadata.

// The emitted-method sets are probed at every method dispatch lowering, and a sample of the
// codegen phase caught SipHash at 7% of it. `fast_hash` exists for exactly these keys.
use crate::fast_hash::FastSet as HashSet;

use crate::codegen::callable_dispatch::{RuntimeCallableCase, RuntimeStaticMethodCallableCase};
use crate::ir::Module;
use crate::names::php_symbol_key;
use crate::types::{FunctionSig, PhpType};

use super::shared_reflection::SharedReflectionState;

/// Module-wide artifacts emitted once and reused by every function lowering context.
pub(crate) struct SharedCodegenState {
    pub(super) reflection: SharedReflectionState,
    emitted_instance_methods: HashSet<(String, String)>,
    emitted_static_methods: HashSet<(String, String)>,
    runtime_string_descriptor_cases:
        Vec<(Option<PhpType>, Option<Vec<String>>, bool, Vec<RuntimeCallableCase>)>,
    runtime_static_method_descriptor_cases:
        Vec<(Option<Vec<String>>, Vec<RuntimeStaticMethodCallableCase>)>,
    runtime_static_method_descriptor_case_entries: Vec<RuntimeStaticMethodCallableCase>,
    /// Receiver-captured descriptor templates, bucketed by a hash of theirthree--part name key.
    ///
    /// This was a flat `Vec` scanned linearly, comparing three strings AND a whole `FunctionSig`
    /// per entry. The Symfony `--web` module fills it with thousands of templates and asks for
    /// one at every dynamic call site, so the scan is quadratic: a 40-second sample of the
    /// codegen phase put this one lookup at 4 404 of ~28 000 non-idle samples, 15.7%.
    ///
    /// The bucket key deliberately excludes the signature, which is the expensive part to
    /// compare: a bucket holds the handful of entries sharing a name, and the signature is
    /// compared only within it.
    runtime_instance_method_descriptors:
        crate::fast_hash::FastMap<u64, Vec<RuntimeInstanceMethodDescriptorCacheEntry>>,
    runtime_callable_invokers: Vec<RuntimeCallableInvokerCacheEntry>,
    eval_registration_helper: Option<String>,
    runtime_builtin_wrappers: Vec<RuntimeCallWrapperCacheEntry>,
    runtime_extern_wrappers: Vec<RuntimeCallWrapperCacheEntry>,
    /// Memoized sharing decision for open Mixed callable dispatch by strictness profile.
    mixed_callable_sharing: [Option<bool>; 2],
    label_counter: usize,
    /// Bumped by every shared-cache access, READ or write. `emit_module` samples it per body
    /// under `ELEPHC_CODEGEN_SHARED_TOUCHES` to learn how many bodies could be emitted without
    /// the shared state at all, which is what decides whether a parallel emission pass is worth
    /// it. A `Cell` because the lookups take `&self`, and a read that MISSES leads to emitting a
    /// helper, so a read counts exactly as much as a write.
    cache_touches: std::cell::Cell<usize>,
    /// Accesses to caches whose helper symbols are NOT yet derived from the cache key. A body
    /// that reaches one of those still has to be emitted serially: two workers would give the
    /// same helper two names and the merge could not collapse them.
    unkeyed_cache_touches: std::cell::Cell<usize>,
    /// Memoized "does this module share the Mixed string-context ladder", indexed by mode.
    ///
    /// The predicate counts sites across every body in the module and is consulted at EVERY
    /// string context, so computing it per site is quadratic in module size. That is not
    /// theoretical here: an `eval()` program emits close to a million lines of assembly, and
    /// its sites would each rescan the whole instruction stream.
    mixed_string_sharing: [Option<bool>; 2],
    /// Memoized "does this module share the `count()` countable guard", for the same reason.
    count_guard_sharing: Option<bool>,
    /// Memoized "does this module share the codegen-raised throwable", for the same reason.
    static_throw_sharing: Option<bool>,
    /// Class and interface names by their PHP case-insensitive key, built on first use.
    ///
    /// `class_info_by_name` scanned every class in the module and called `php_symbol_key` — which
    /// allocates a lowercased `String` — on EVERY entry, so one lookup cost thousands of
    /// allocations, and property lowering calls it per access and again for each parent hop. A
    /// sample of the codegen phase put the lookup at 11% and the allocator it feeds at 57%.
    ///
    /// `RefCell` because the lookup is reached through a shared `&FunctionContext`; the index is
    /// a pure function of the module, so filling it through a shared reference changes nothing an
    /// observer could see.
    class_key_index: std::cell::RefCell<Option<crate::fast_hash::FastMap<String, String>>>,
    /// Interface names by the same key, for the same reason.
    interface_key_index: std::cell::RefCell<Option<crate::fast_hash::FastMap<String, String>>>,
    /// Trait POSITIONS in `trait_table.names` by the same key; the table is a `Vec`, so the index
    /// stores where the name lives rather than a second copy of it.
    trait_key_index: std::cell::RefCell<Option<crate::fast_hash::FastMap<String, usize>>>,
    /// Enum names by the same key, for the same reason.
    enum_key_index: std::cell::RefCell<Option<crate::fast_hash::FastMap<String, String>>>,
    /// Every global needing shared reference-cell storage, collected in one walk of the module.
    ///
    /// `superglobals::uses_shared_ref_cell` answers for ONE name by walking every instruction of
    /// every body, and codegen asks it per scope entry at each `eval` site: a sample of the
    /// codegen phase found that chain holding its heaviest iterator.
    shared_ref_cell_globals: Option<crate::fast_hash::FastSet<String>>,
    /// Memoized "does this module call `pcntl_async_signals()`" and "… `pcntl_signal()`".
    ///
    /// These are the same trap as the two above, and they were paying it: `FunctionContext::new`
    /// asked BOTH on every body, and each answer walks every instruction of every function,
    /// method, closure, fiber wrapper, callback wrapper, extern trampoline and runtime invoker
    /// in the module. On the Symfony `--web` module that is 745 237 instructions revisited by
    /// each of 8 786 bodies, twice — and a 50-second sample of the codegen phase put
    /// `Chain::try_fold` at 22 916 of ~35 000 non-idle samples, about two thirds of it.
    pcntl_async_signals: Option<bool>,
    /// See `pcntl_async_signals`.
    pcntl_signal_handlers: Option<bool>,
    /// Memoized "did this module keep the `__elephc_shutdown_run` drain entry".
    ///
    /// Same trap, smaller walk — the module's FUNCTION LIST, not its instructions — but it is
    /// asked once per body and the Symfony module has 8 786 of them, so it is memoized with the
    /// rest rather than left as a linear scan repeated per body.
    php_shutdown_drain: Option<bool>,
    /// Armed by a codegen worker around one body: the `unkeyed_cache_touches` value the body
    /// started from. While it is set, the block walker stops the body at the first instruction
    /// that moved the count, instead of finishing a body the worker is going to discard.
    deferral_watermark: Option<usize>,
    /// `--counters`: every non-synthetic PHP function prologue increments a BSS slot.
    pub(super) counters: bool,
    /// `--instrument`: every non-synthetic PHP function calls
    /// `elephc_instr_enter(id)`/`_exit(id)` around its body for exact timing.
    pub(super) instrument: crate::codegen::Instrumentation,
    /// `--probe`: the embedded symbol table `(data label, entry count)` main's
    /// prologue hands to `elephc_probe_init`. `None` unless the probe is enabled.
    pub(super) probe_table: Option<(String, usize)>,
    /// Counted functions as `(display name, counter symbol)`, in emission order. Main's
    /// epilogue renders the exit dump from this list — main is emitted last, so the
    /// registry is complete by then.
    counter_registry: Vec<(String, String)>,
    /// Instrumented function names, in id order (id = index). Main emits the
    /// name table `elephc_instr_init` reads; main is emitted last, so it is
    /// complete by then.
    instr_registry: Vec<String>,
}

/// Reusable static descriptor template for one public instance method.
#[derive(Clone)]
pub(super) struct RuntimeInstanceMethodDescriptorTemplate {
    pub(super) descriptor_label: String,
}

/// Cache key and emitted template for one receiver-class/method/signature shape.
struct RuntimeInstanceMethodDescriptorCacheEntry {
    class_name: String,
    method_key: String,
    impl_class: String,
    signature: FunctionSig,
    template: RuntimeInstanceMethodDescriptorTemplate,
}

/// Cache key and label for one signature-compatible descriptor invoker body.
struct RuntimeCallableInvokerCacheEntry {
    signature: FunctionSig,
    captures: Vec<(String, PhpType, bool)>,
    label: String,
}

/// Cache key and label for one synthetic builtin or extern entry wrapper.
struct RuntimeCallWrapperCacheEntry {
    name: String,
    signature: FunctionSig,
    strict_php: bool,
    label: String,
}

/// Buckets a receiver-captured descriptor template by its name key, signature excluded.
fn instance_descriptor_bucket(class_name: &str, method_key: &str, impl_class: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    class_name.hash(&mut hasher);
    method_key.hash(&mut hasher);
    impl_class.hash(&mut hasher);
    hasher.finish()
}

impl SharedCodegenState {
    /// Builds module-wide caches and immutable method-membership indexes before emission starts.
    pub(super) fn for_module(module: &Module) -> Self {
        let mut state = Self::empty();
        state.reflection = SharedReflectionState::for_module(module);
        for function in &module.class_methods {
            let Some((class_name, method_name)) = function.name.rsplit_once("::") else {
                continue;
            };
            let key = (class_name.to_string(), php_symbol_key(method_name));
            if function.flags.is_static {
                state.emitted_static_methods.insert(key);
            } else {
                state.emitted_instance_methods.insert(key);
            }
        }
        state
    }

    /// Creates private empty storage used only while constructing a module-populated state.
    fn empty() -> Self {
        Self {
            reflection: SharedReflectionState::empty(),
            emitted_instance_methods: HashSet::default(),
            emitted_static_methods: HashSet::default(),
            runtime_string_descriptor_cases: Vec::new(),
            runtime_static_method_descriptor_cases: Vec::new(),
            runtime_static_method_descriptor_case_entries: Vec::new(),
            runtime_instance_method_descriptors: crate::fast_hash::FastMap::default(),
            runtime_callable_invokers: Vec::new(),
            eval_registration_helper: None,
            runtime_builtin_wrappers: Vec::new(),
            runtime_extern_wrappers: Vec::new(),
            mixed_string_sharing: [None; 2],
            mixed_callable_sharing: [None; 2],
            label_counter: 0,
            cache_touches: std::cell::Cell::new(0),
            unkeyed_cache_touches: std::cell::Cell::new(0),
            count_guard_sharing: None,
            static_throw_sharing: None,
            class_key_index: std::cell::RefCell::new(None),
            interface_key_index: std::cell::RefCell::new(None),
            trait_key_index: std::cell::RefCell::new(None),
            enum_key_index: std::cell::RefCell::new(None),
            shared_ref_cell_globals: None,
            pcntl_async_signals: None,
            pcntl_signal_handlers: None,
            php_shutdown_drain: None,
            deferral_watermark: None,
            counters: false,
            instrument: crate::codegen::Instrumentation::default(),
            probe_table: None,
            counter_registry: Vec::new(),
            instr_registry: Vec::new(),
        }
    }

    /// Returns whether the immutable module inventory contains the requested method body.
    pub(super) fn emitted_method_contains(
        &self,
        class_name: &str,
        canonical_method_key: &str,
        is_static: bool,
    ) -> bool {
        let methods = if is_static {
            &self.emitted_static_methods
        } else {
            &self.emitted_instance_methods
        };
        methods.contains(&(
            class_name.to_string(),
            canonical_method_key.to_string(),
        ))
    }

    /// Borrows the immutable instance-method inventory for contains-only interface validation.
    pub(super) fn emitted_instance_method_keys(&self) -> &HashSet<(String, String)> {
        &self.emitted_instance_methods
    }

    /// Returns the memoized Mixed callable sharing decision for one strictness profile.
    pub(super) fn mixed_callable_sharing(&self, profile_index: usize) -> Option<bool> {
        self.mixed_callable_sharing[profile_index]
    }

    /// Stores the Mixed callable sharing decision for one strictness profile.
    pub(super) fn set_mixed_callable_sharing(&mut self, profile_index: usize, shares: bool) {
        self.mixed_callable_sharing[profile_index] = Some(shares);
    }

    /// Reserves the next module-unique assembly label id.
    ///
    /// Every generated local label ends in `_<id>` taken from this counter. Because the id is a
    /// decimal run terminated by the preceding `_`, it is recoverable from the finished label,
    /// which makes the whole label unique no matter how ambiguous its readable prefix is.
    /// Returns how many shared-cache accesses have been made so far.
    ///
    /// Only meaningful as a DIFFERENCE across one body: a body whose count does not move could
    /// have been emitted against a read-only view of this state, on any thread.
    pub(super) fn cache_touches(&self) -> usize {
        // The reflection materializers are a cache of the same kind, living on their own struct:
        // a body that reaches one cannot be emitted against a private copy of this state either.
        self.cache_touches.get() + self.reflection.cache_touches()
    }

    // `checkpoint()`/`rollback_to()` used to live here, to undo a deferred body's cache
    // entries. They are gone on purpose: one instruction's lowering can emit thousands of
    // descriptors, so the worker's abort cannot land inside it, and the rollback then makes
    // every following body re-emit its helpers -- measured on the Symfony module, 2 101 439 143
    // discarded bytes became 29 397 554 561 and the worker pass 40s became 121s. What makes
    // their absence safe is that a body which TOUCHES a cache is always deferred, so a kept
    // body never references a cached helper.

    /// Records one access to a cache a worker may NOT reach, so its body is emitted serially.
    ///
    /// Two reasons a family stays here. Its helper's symbol may still be minted from the
    /// emitting body, so two workers would give one helper two names. Or — the one that is not
    /// obvious — the cache may be module-wide STATE rather than a lookup: the static-method
    /// descriptor cases accumulate a dispatch table, and splitting that table across per-worker
    /// states left the 404 page rendering a raw exception dump instead of the error page.
    /// Duplication across workers is safe; a partitioned table is not, and keying its labels
    /// would not have helped.
    fn touch_cache(&self) {
        self.cache_touches.set(self.cache_touches.get() + 1);
        self.unkeyed_cache_touches
            .set(self.unkeyed_cache_touches.get() + 1);
    }

    /// Records one access to a cache whose helper symbols are derived from the cache key.
    ///
    /// Such a helper can be emitted by any worker: every copy spells the same symbols, so the
    /// merge keeps one and drops the rest, and the body does not have to go serial.
    ///
    /// No family uses this yet. Two are ready for it — the callable invoker and the
    /// builtin/extern wrappers now mint their symbols from their cache keys — but opting them in
    /// while the OTHER families still defer is what made deferral quadratic, because a deferred
    /// body has to undo its cache entries and the next body then re-emits everything. It pays
    /// once every family can be reached in parallel.
    #[allow(dead_code)]
    fn touch_cache_keyed(&self) {
        self.cache_touches.set(self.cache_touches.get() + 1);
    }

    /// Returns how many accesses so far were to caches that still force a serial body.
    pub(super) fn unkeyed_cache_touches(&self) -> usize {
        self.unkeyed_cache_touches.get()
    }

    pub(super) fn next_label_id(&mut self) -> usize {
        let id = self.label_counter;
        self.label_counter += 1;
        id
    }

    /// Returns the memoized string-context sharing decision for one mode, if already computed.
    pub(super) fn mixed_string_sharing(&self, mode_index: usize) -> Option<bool> {
        self.mixed_string_sharing[mode_index]
    }

    /// Records the string-context sharing decision so later sites reuse it.
    pub(super) fn set_mixed_string_sharing(&mut self, mode_index: usize, shares: bool) {
        self.mixed_string_sharing[mode_index] = Some(shares);
    }

    /// Returns the memoized `count()` guard sharing decision, if already computed.
    pub(super) fn count_guard_sharing(&self) -> Option<bool> {
        self.count_guard_sharing
    }

    /// Records the `count()` guard sharing decision so later sites reuse it.
    pub(super) fn set_count_guard_sharing(&mut self, shares: bool) {
        self.count_guard_sharing = Some(shares);
    }

    /// Returns the memoized codegen-raised throwable sharing decision, if already computed.
    pub(super) fn static_throw_sharing(&self) -> Option<bool> {
        self.static_throw_sharing
    }

    /// Records the codegen-raised throwable sharing decision so later raises reuse it.
    pub(super) fn set_static_throw_sharing(&mut self, shares: bool) {
        self.static_throw_sharing = Some(shares);
    }

    /// Returns the module's spelling of the class whose PHP key is `key`, indexing once.
    pub(crate) fn class_name_for_key(&self, module: &Module, key: &str) -> Option<String> {
        let mut index = self.class_key_index.borrow_mut();
        index
            .get_or_insert_with(|| {
                module
                    .class_infos
                    .keys()
                    .map(|name| {
                        (
                            php_symbol_key(name.trim_start_matches('\\')),
                            name.clone(),
                        )
                    })
                    .collect()
            })
            .get(key)
            .cloned()
    }

    /// Returns the module's spelling of the interface whose PHP key is `key`, indexing once.
    pub(crate) fn interface_name_for_key(&self, module: &Module, key: &str) -> Option<String> {
        let mut index = self.interface_key_index.borrow_mut();
        index
            .get_or_insert_with(|| {
                module
                    .interface_infos
                    .keys()
                    .map(|name| {
                        (
                            php_symbol_key(name.trim_start_matches('\\')),
                            name.clone(),
                        )
                    })
                    .collect()
            })
            .get(key)
            .cloned()
    }

    /// Returns where in `trait_table.names` the trait whose PHP key is `key` lives, indexing once.
    pub(crate) fn trait_position_for_key(&self, module: &Module, key: &str) -> Option<usize> {
        let mut index = self.trait_key_index.borrow_mut();
        index
            .get_or_insert_with(|| {
                module
                    .trait_table
                    .names
                    .iter()
                    .enumerate()
                    .map(|(position, name)| (php_symbol_key(name.trim_start_matches('\\')), position))
                    .collect()
            })
            .get(key)
            .copied()
    }

    /// Returns the module's spelling of the enum whose PHP key is `key`, indexing once.
    pub(crate) fn enum_name_for_key(&self, module: &Module, key: &str) -> Option<String> {
        let mut index = self.enum_key_index.borrow_mut();
        index
            .get_or_insert_with(|| {
                module
                    .enum_infos
                    .keys()
                    .map(|name| {
                        (
                            php_symbol_key(name.trim_start_matches('\\')),
                            name.clone(),
                        )
                    })
                    .collect()
            })
            .get(key)
            .cloned()
    }

    /// Returns whether `name` needs shared reference-cell storage, walking the module once.
    pub(crate) fn uses_shared_ref_cell(&mut self, module: &Module, name: &str) -> bool {
        self.shared_ref_cell_globals
            .get_or_insert_with(|| crate::superglobals::shared_ref_cell_globals(module))
            .contains(name)
    }

    /// Returns whether this module calls `pcntl_async_signals()`, computing it at most once.
    pub(super) fn pcntl_async_signals(&mut self, compute: impl FnOnce() -> bool) -> bool {
        *self.pcntl_async_signals.get_or_insert_with(compute)
    }

    /// Returns whether this module registers `pcntl_signal()` handlers, computed at most once.
    pub(super) fn pcntl_signal_handlers(&mut self, compute: impl FnOnce() -> bool) -> bool {
        *self.pcntl_signal_handlers.get_or_insert_with(compute)
    }

    /// Returns whether this module still declares the shutdown drain, computed at most once.
    pub(super) fn php_shutdown_drain(&mut self, compute: impl FnOnce() -> bool) -> bool {
        *self.php_shutdown_drain.get_or_insert_with(compute)
    }

    /// Starts watching this body for a cache a worker may not reach.
    pub(super) fn arm_deferral_watch(&mut self) {
        self.deferral_watermark = Some(self.unkeyed_cache_touches.get());
    }

    /// Stops watching, for the serial pass, which may reach anything.
    pub(super) fn disarm_deferral_watch(&mut self) {
        self.deferral_watermark = None;
    }

    /// Returns whether the body being emitted has reached a cache a worker may not reach.
    ///
    /// False whenever the watch is disarmed, which is every path but a worker's.
    pub(super) fn must_defer(&self) -> bool {
        self.deferral_watermark
            .is_some_and(|mark| self.unkeyed_cache_touches.get() != mark)
    }

    /// Records one counted function for the exit dump.
    pub(super) fn register_counter(&mut self, display_name: String, symbol: String) {
        self.counter_registry.push((display_name, symbol));
    }

    /// Returns the counted functions in emission order.
    pub(super) fn counter_registry(&self) -> &[(String, String)] {
        &self.counter_registry
    }

    /// Registers one instrumented function and returns its stable id (its index
    /// in the name table `elephc_instr_init` receives).
    pub(super) fn register_instr(&mut self, display_name: String) -> usize {
        let id = self.instr_registry.len();
        self.instr_registry.push(display_name);
        id
    }

    /// Returns the instrumented function names in id order.
    pub(super) fn instr_registry(&self) -> &[String] {
        &self.instr_registry
    }

    /// Returns cached runtime string-callable cases for the requested specialization.
    pub(super) fn runtime_string_descriptor_cases(
        &self,
        source_arg_ty: Option<&PhpType>,
        candidate_names: Option<&[String]>,
        strict_php: bool,
    ) -> Option<Vec<RuntimeCallableCase>> {
        self.touch_cache();
        self.runtime_string_descriptor_cases
            .iter()
            .find(|(cached_ty, cached_names, cached_strict_php, _)| {
                cached_ty.as_ref() == source_arg_ty
                    && cached_names.as_deref() == candidate_names
                    && *cached_strict_php == strict_php
            })
            .map(|(_, _, _, cases)| cases.clone())
    }

    /// Stores runtime string-callable cases after their global wrappers are emitted.
    pub(super) fn cache_runtime_string_descriptor_cases(
        &mut self,
        source_arg_ty: Option<&PhpType>,
        candidate_names: Option<&[String]>,
        strict_php: bool,
        cases: &[RuntimeCallableCase],
    ) {
        self.touch_cache();
        self.runtime_string_descriptor_cases.push((
            source_arg_ty.cloned(),
            candidate_names.map(|names| names.to_vec()),
            strict_php,
            cases.to_vec(),
        ));
    }

    /// Returns the module-wide public static-method descriptor cases, if emitted.
    pub(super) fn runtime_static_method_descriptor_cases(
        &self,
        candidate_names: Option<&[String]>,
    ) -> Option<Vec<RuntimeStaticMethodCallableCase>> {
        self.touch_cache();
        self.runtime_static_method_descriptor_cases
            .iter()
            .find(|(cached_names, _)| cached_names.as_deref() == candidate_names)
            .map(|(_, cases)| cases.clone())
    }

    /// Stores public static-method descriptors for reuse by later call sites.
    pub(super) fn cache_runtime_static_method_descriptor_cases(
        &mut self,
        candidate_names: Option<&[String]>,
        cases: &[RuntimeStaticMethodCallableCase],
    ) {
        self.touch_cache();
        self.runtime_static_method_descriptor_cases.push((
            candidate_names.map(|names| names.to_vec()),
            cases.to_vec(),
        ));
    }

    /// Returns one static-method descriptor case already emitted for another target set.
    pub(super) fn runtime_static_method_descriptor_case(
        &self,
        php_name: &str,
    ) -> Option<RuntimeStaticMethodCallableCase> {
        self.touch_cache();
        self.runtime_static_method_descriptor_case_entries
            .iter()
            .find(|case| case.case.php_name.as_deref() == Some(php_name))
            .cloned()
    }

    /// Records one static-method descriptor case for reuse across candidate sets.
    pub(super) fn cache_runtime_static_method_descriptor_case(
        &mut self,
        case: &RuntimeStaticMethodCallableCase,
    ) {
        self.touch_cache();
        self.runtime_static_method_descriptor_case_entries
            .push(case.clone());
    }

    /// Returns an emitted receiver-captured descriptor template for one method shape.
    pub(super) fn runtime_instance_method_descriptor(
        &self,
        class_name: &str,
        method_key: &str,
        impl_class: &str,
        signature: &FunctionSig,
    ) -> Option<RuntimeInstanceMethodDescriptorTemplate> {
        self.touch_cache();
        self.runtime_instance_method_descriptors
            .get(&instance_descriptor_bucket(class_name, method_key, impl_class))?
            .iter()
            .find(|entry| {
                entry.class_name == class_name
                    && entry.method_key == method_key
                    && entry.impl_class == impl_class
                    && entry.signature == *signature
            })
            .map(|entry| entry.template.clone())
    }

    /// Stores a receiver-captured descriptor template after first emission.
    pub(super) fn cache_runtime_instance_method_descriptor(
        &mut self,
        class_name: &str,
        method_key: &str,
        impl_class: &str,
        signature: &FunctionSig,
        template: RuntimeInstanceMethodDescriptorTemplate,
    ) {
        self.touch_cache();
        self.runtime_instance_method_descriptors
            .entry(instance_descriptor_bucket(class_name, method_key, impl_class))
            .or_default()
            .push(RuntimeInstanceMethodDescriptorCacheEntry {
                class_name: class_name.to_string(),
                method_key: method_key.to_string(),
                impl_class: impl_class.to_string(),
                signature: signature.clone(),
                template,
            });
    }

    /// Returns an already-emitted descriptor invoker with the same ABI shape.
    pub(super) fn runtime_callable_invoker(
        &self,
        signature: &FunctionSig,
        captures: &[(String, PhpType, bool)],
    ) -> Option<String> {
        self.touch_cache();
        self.runtime_callable_invokers
            .iter()
            .find(|entry| entry.signature == *signature && entry.captures == captures)
            .map(|entry| entry.label.clone())
    }

    /// Records a descriptor invoker body for module-wide signature reuse.
    pub(super) fn cache_runtime_callable_invoker(
        &mut self,
        signature: &FunctionSig,
        captures: &[(String, PhpType, bool)],
        label: &str,
    ) {
        self.touch_cache();
        self.runtime_callable_invokers
            .push(RuntimeCallableInvokerCacheEntry {
                signature: signature.clone(),
                captures: captures.to_vec(),
                label: label.to_string(),
            });
    }

    /// Returns the module-wide eval metadata registration helper label, if emitted.
    pub(super) fn eval_registration_helper(&self) -> Option<String> {
        self.touch_cache();
        self.eval_registration_helper.clone()
    }

    /// Publishes the module-wide eval metadata registration helper label.
    pub(super) fn cache_eval_registration_helper(&mut self, label: String) {
        self.touch_cache();
        debug_assert!(self.eval_registration_helper.is_none());
        self.eval_registration_helper = Some(label);
    }

    /// Returns a previously emitted synthetic builtin wrapper for the same signature.
    pub(super) fn runtime_builtin_wrapper(
        &self,
        name: &str,
        signature: &FunctionSig,
        strict_php: bool,
    ) -> Option<String> {
        self.touch_cache();
        cached_runtime_call_wrapper(
            &self.runtime_builtin_wrappers,
            name,
            signature,
            strict_php,
        )
    }

    /// Records a synthetic builtin wrapper for module-wide reuse.
    pub(super) fn cache_runtime_builtin_wrapper(
        &mut self,
        name: &str,
        signature: &FunctionSig,
        strict_php: bool,
        label: &str,
    ) {
        self.touch_cache();
        cache_runtime_call_wrapper(
            &mut self.runtime_builtin_wrappers,
            name,
            signature,
            strict_php,
            label,
        );
    }

    /// Returns a previously emitted synthetic extern wrapper for the same signature.
    pub(super) fn runtime_extern_wrapper(
        &self,
        name: &str,
        signature: &FunctionSig,
    ) -> Option<String> {
        self.touch_cache();
        cached_runtime_call_wrapper(&self.runtime_extern_wrappers, name, signature, false)
    }

    /// Records a synthetic extern wrapper for module-wide reuse.
    pub(super) fn cache_runtime_extern_wrapper(
        &mut self,
        name: &str,
        signature: &FunctionSig,
        label: &str,
    ) {
        self.touch_cache();
        cache_runtime_call_wrapper(
            &mut self.runtime_extern_wrappers,
            name,
            signature,
            false,
            label,
        );
    }
}

/// Looks up a cached synthetic call wrapper by PHP name and ABI signature.
fn cached_runtime_call_wrapper(
    entries: &[RuntimeCallWrapperCacheEntry],
    name: &str,
    signature: &FunctionSig,
    strict_php: bool,
) -> Option<String> {
    entries
        .iter()
        .find(|entry| {
            entry.name == name
                && entry.signature == *signature
                && entry.strict_php == strict_php
        })
        .map(|entry| entry.label.clone())
}

/// Adds one synthetic call wrapper to its module-wide cache.
fn cache_runtime_call_wrapper(
    entries: &mut Vec<RuntimeCallWrapperCacheEntry>,
    name: &str,
    signature: &FunctionSig,
    strict_php: bool,
    label: &str,
) {
    entries.push(RuntimeCallWrapperCacheEntry {
        name: name.to_string(),
        signature: signature.clone(),
        strict_php,
        label: label.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::Target;
    use crate::ir::{Function, IrType};

    /// Creates a minimal EIR method body with the requested raw name and static flag.
    fn method(name: &str, is_static: bool) -> Function {
        let mut function = Function::new(name.to_string(), IrType::Void, PhpType::Void);
        function.flags.is_static = is_static;
        function
    }

    /// Reproduces the pre-index linear membership predicate for differential assertions.
    fn old_scan_contains(
        module: &Module,
        class_name: &str,
        method_key: &str,
        is_static: bool,
    ) -> bool {
        module.class_methods.iter().any(|function| {
            function.flags.is_static == is_static
                && function
                    .name
                    .rsplit_once("::")
                    .is_some_and(|(candidate_class, candidate_method)| {
                        candidate_class == class_name
                            && php_symbol_key(candidate_method) == method_key
                    })
        })
    }

    /// Proves the immutable index preserves raw classes and method-only canonicalization.
    #[test]
    fn emitted_method_index_matches_previous_scan_semantics() {
        let mut module = Module::new(Target::detect_host());
        module.class_methods = vec![
            method("Ns\\Thing::DoWork", false),
            method("Ns\\Thing::StaticWork", true),
            method("Ns\\Thing::Dual", false),
            method("Ns\\Thing::Dual", true),
            method("Outer::Inner::MiXeD", false),
            method("\\Ns\\Thing::Leading", false),
            method("missing_delimiter", false),
        ];
        let state = SharedCodegenState::for_module(&module);
        let queries = [
            ("Ns\\Thing", "dowork", false),
            ("Ns\\Thing", "DOWORK", false),
            ("Ns\\Thing", "staticwork", true),
            ("Ns\\Thing", "staticwork", false),
            ("Ns\\Thing", "dual", false),
            ("Ns\\Thing", "dual", true),
            ("Outer::Inner", "mixed", false),
            ("Outer", "inner::mixed", false),
            ("\\Ns\\Thing", "leading", false),
            ("Ns\\Thing", "leading", false),
            ("", "missing_delimiter", false),
        ];

        for (class_name, method_key, is_static) in queries {
            assert_eq!(
                state.emitted_method_contains(class_name, method_key, is_static),
                old_scan_contains(&module, class_name, method_key, is_static),
                "membership mismatch for {class_name}::{method_key}, static={is_static}"
            );
        }
    }
}
