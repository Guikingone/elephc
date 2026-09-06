//! Purpose:
//! Defines the opaque eval context storage layout and initializes all registries.
//!
//! Called from:
//! - Public context construction and every context method family.
//!
//! Key details:
//! - Generated code only passes this value opaquely; Rust owns every internal collection.

use super::*;
use std::sync::atomic::{AtomicBool, AtomicUsize};

/// Process-level eval context passed opaquely across the C ABI.
///
/// Generated code never inspects this layout directly; it only passes pointers
/// back to the eval bridge. Keeping a concrete Rust type here lets the bridge
/// grow dynamic registries without exposing them to generated assembly.
pub struct ElephcEvalContext {
    pub(super) abi_version: u32,
    pub(super) classes: HashMap<String, EvalClass>,
    pub(super) class_source_files: HashMap<String, String>,
    pub(super) class_aliases: HashMap<String, EvalClassAlias>,
    pub(super) declared_class_names: Arc<Vec<String>>,
    pub(super) interfaces: HashMap<String, EvalInterface>,
    pub(super) declared_interface_names: Arc<Vec<String>>,
    pub(super) traits: HashMap<String, EvalTrait>,
    pub(super) declared_trait_names: Arc<Vec<String>>,
    pub(super) enums: HashMap<String, EvalEnum>,
    pub(super) declared_enum_names: Vec<String>,
    pub(super) enum_cases: HashMap<(String, String), RuntimeCellHandle>,
    pub(super) enum_case_values: HashMap<(String, String), RuntimeCellHandle>,
    pub(super) constants: HashMap<String, RuntimeCellHandle>,
    pub(super) functions: HashMap<String, EvalFunction>,
    pub(super) closures: HashMap<String, EvalClosure>,
    pub(super) closure_objects: HashMap<u64, EvalClosureObjectTarget>,
    pub(super) live_closure_objects: AtomicUsize,
    pub(super) live_dynamic_objects: AtomicUsize,
    pub(super) live_global_functions: AtomicUsize,
    pub(super) live_autoload_contexts: AtomicUsize,
    pub(super) retained_context_free_requested: AtomicBool,
    pub(super) next_closure_id: usize,
    pub(super) native_functions: Arc<HashMap<String, NativeFunction>>,
    pub(super) native_methods: Arc<HashMap<(String, String), NativeCallableSignature>>,
    pub(super) native_static_methods: Arc<HashMap<(String, String), NativeCallableSignature>>,
    pub(super) native_constructors: Arc<HashMap<String, NativeCallableSignature>>,
    pub(super) native_class_parents: Arc<HashMap<String, String>>,
    pub(super) native_class_attributes: Arc<HashMap<String, Vec<EvalAttribute>>>,
    pub(super) native_method_attributes: Arc<HashMap<(String, String), Vec<EvalAttribute>>>,
    pub(super) native_constant_attributes: Arc<HashMap<(String, String), Vec<EvalAttribute>>>,
    pub(super) native_interface_properties:
        Arc<HashMap<String, Vec<(String, EvalInterfaceProperty)>>>,
    pub(super) native_abstract_properties:
        Arc<HashMap<String, Vec<(String, EvalInterfaceProperty)>>>,
    pub(super) native_property_types: Arc<HashMap<(String, String), EvalParameterType>>,
    pub(super) native_property_defaults: Arc<HashMap<(String, String), NativeCallableDefault>>,
    pub(super) native_property_attributes: Arc<HashMap<(String, String), Vec<EvalAttribute>>>,
    /// One scope holding every `static` slot, php's storage for `static $x`.
    ///
    /// A scope rather than a map because a WRITE to a static arrives through `set_scope_cell`,
    /// which only ever holds `&ElephcEvalContext` -- the same shape the global scope already
    /// solves, and solved the same way: the context owns the scope and hands out a raw pointer.
    /// `RefCell` was tried first and is wrong here, because it is not `RefUnwindSafe` and the FFI
    /// crosses a hundred `catch_unwind` boundaries holding this context.
    pub(super) static_scope: Box<ElephcEvalScope>,
    /// Overrides the key `static` slots hang from, for callables the function name cannot name.
    ///
    /// Only closures need it: `__FUNCTION__` and a backtrace must keep saying `{closure}`, which
    /// is the FUNCTION name, while the statics belong to this closure OBJECT.
    pub(super) static_slot_keys: Vec<String>,
    pub(super) static_properties: HashMap<(String, String), RuntimeCellHandle>,
    pub(super) static_property_aliases: HashMap<(String, String), EvalReferenceTarget>,
    pub(super) class_constants: HashMap<(String, String), RuntimeCellHandle>,
    pub(super) included_files: HashSet<String>,
    pub(super) claimed_aot_include_classlikes: HashSet<String>,
    pub(super) include_execution_stack: Vec<bool>,
    pub(super) dynamic_objects: HashMap<u64, String>,
    pub(super) dynamic_destructing_objects: HashSet<u64>,
    pub(super) dynamic_destructed_objects: HashSet<u64>,
    pub(super) dynamic_property_values: HashMap<(u64, String), RuntimeCellHandle>,
    /// Overlay property names per object in the order they were first written.
    ///
    /// PHP reports an object's dynamic properties in creation order, and the overlay map
    /// above cannot answer that. Everything that ENUMERATES an object reads its order from
    /// here, so the answer is PHP's rather than a hash order or an alphabetical stand-in.
    pub(super) dynamic_property_order: HashMap<u64, Vec<String>>,
    pub(super) dynamic_property_aliases: HashMap<(u64, String), EvalReferenceTarget>,
    pub(super) array_element_aliases: HashMap<(u64, EvalArrayReferenceKey), EvalReferenceTarget>,
    pub(super) array_cursors: HashMap<usize, EvalArrayCursor>,
    pub(super) dynamic_initialized_properties: HashSet<(u64, String)>,
    /// Live interpreter call frames, innermost last, for `debug_backtrace()`.
    /// Execution state of every live generator, keyed by its object identity.
    ///
    /// A generator outlives the call that created it, so its frame — including its own
    /// scope — cannot live on that call's stack.
    pub(super) eval_generators: HashMap<u64, EvalGeneratorFrame>,
    pub(super) eval_reflection_attributes: HashMap<u64, EvalReflectionAttributeMetadata>,
    pub(super) eval_reflection_classes: HashMap<u64, String>,
    pub(super) eval_reflection_functions: HashMap<u64, String>,
    pub(super) eval_reflection_function_closure_targets: HashMap<u64, EvalClosureObjectTarget>,
    pub(super) eval_reflection_methods: HashMap<u64, (String, String)>,
    pub(super) eval_reflection_properties: HashMap<u64, (String, String)>,
    pub(super) eval_dynamic_reflection_properties: HashSet<u64>,
    pub(super) eval_reflection_class_constants: HashMap<u64, (String, String, u64)>,
    pub(super) eval_static_callables: HashMap<usize, EvalStaticCallableMetadata>,
    pub(super) eval_object_callables: HashMap<usize, EvalObjectCallableMetadata>,
    pub(super) global_scope: Option<*mut ElephcEvalScope>,
    pub(super) owns_global_scope: bool,
    pub(super) autoload_callbacks: Vec<(i64, RuntimeCellHandle)>,
    pub(super) autoloading_classes: HashSet<String>,
    pub(super) function_stack: Vec<String>,
    pub(super) returns_by_ref: bool,
    pub(super) strict_types: bool,
    /// `declare(ticks=N)`'s N, or 0 when no tick directive is in force.
    pub(super) tick_interval: i64,
    /// Statements executed since the last tick fired.
    pub(super) tick_counter: i64,
    /// Callables `register_tick_function()` has registered, in registration order.
    pub(super) tick_functions: Vec<RuntimeCellHandle>,
    /// Whether a tick handler is running, so its own statements do not tick again.
    pub(super) tick_running: bool,
    pub(super) array_iterators: HashMap<u64, EvalArrayIteratorState>,
    pub(super) pending_return_reference: Option<(EvalReferenceTarget, RuntimeCellHandle)>,
    pub(super) class_stack: Vec<String>,
    pub(super) called_class_stack: Vec<String>,
    pub(super) magic_stack: Vec<EvalMagicScope>,
    pub(super) pending_throw: Option<RuntimeCellHandle>,
    pub(super) spl_autoload_extensions: String,
    pub(super) streams: EvalStreamResources,
    pub(super) json_last_error: i64,
    pub(super) json_last_error_msg: String,
    pub(super) default_timezone: String,
    pub(super) http_response_code: i64,
    pub(super) call_file: String,
    pub(super) call_dir: String,
    pub(super) call_line: i64,
    pub(super) file_magic_override: Option<String>,
    pub(super) error_suppression_depth: usize,
}

impl ElephcEvalContext {
    /// Creates a context using the current eval bridge ABI version.
    pub fn new() -> Self {
        Self {
            abi_version: ABI_VERSION,
            classes: HashMap::new(),
            class_source_files: HashMap::new(),
            class_aliases: HashMap::new(),
            declared_class_names: Arc::default(),
            interfaces: HashMap::new(),
            declared_interface_names: Arc::default(),
            traits: HashMap::new(),
            declared_trait_names: Arc::default(),
            enums: HashMap::new(),
            declared_enum_names: Vec::new(),
            enum_cases: HashMap::new(),
            enum_case_values: HashMap::new(),
            constants: HashMap::new(),
            functions: HashMap::new(),
            closures: HashMap::new(),
            closure_objects: HashMap::new(),
            live_closure_objects: AtomicUsize::new(0),
            live_dynamic_objects: AtomicUsize::new(0),
            live_global_functions: AtomicUsize::new(0),
            live_autoload_contexts: AtomicUsize::new(0),
            retained_context_free_requested: AtomicBool::new(false),
            next_closure_id: 0,
            native_functions: Arc::default(),
            native_methods: Arc::default(),
            native_static_methods: Arc::default(),
            native_constructors: Arc::default(),
            native_class_parents: Arc::default(),
            native_class_attributes: Arc::default(),
            native_method_attributes: Arc::default(),
            native_constant_attributes: Arc::default(),
            native_interface_properties: Arc::default(),
            native_abstract_properties: Arc::default(),
            native_property_types: Arc::default(),
            native_property_defaults: Arc::default(),
            native_property_attributes: Arc::default(),
            static_scope: Box::new(ElephcEvalScope::new()),
            static_slot_keys: Vec::new(),
            static_properties: HashMap::new(),
            static_property_aliases: HashMap::new(),
            class_constants: HashMap::new(),
            included_files: HashSet::new(),
            claimed_aot_include_classlikes: HashSet::new(),
            include_execution_stack: Vec::new(),
            dynamic_objects: HashMap::new(),
            dynamic_destructing_objects: HashSet::new(),
            dynamic_destructed_objects: HashSet::new(),
            dynamic_property_values: HashMap::new(),
            dynamic_property_order: HashMap::new(),
            dynamic_property_aliases: HashMap::new(),
            array_element_aliases: HashMap::new(),
            array_cursors: HashMap::new(),
            dynamic_initialized_properties: HashSet::new(),
            eval_generators: HashMap::new(),
            eval_reflection_attributes: HashMap::new(),
            eval_reflection_classes: HashMap::new(),
            eval_reflection_functions: HashMap::new(),
            eval_reflection_function_closure_targets: HashMap::new(),
            eval_reflection_methods: HashMap::new(),
            eval_reflection_properties: HashMap::new(),
            eval_dynamic_reflection_properties: HashSet::new(),
            eval_reflection_class_constants: HashMap::new(),
            eval_static_callables: HashMap::new(),
            eval_object_callables: HashMap::new(),
            global_scope: None,
            owns_global_scope: false,
            autoload_callbacks: Vec::new(),
            autoloading_classes: HashSet::new(),
            function_stack: Vec::new(),
            returns_by_ref: false,
            strict_types: false,
            tick_interval: 0,
            tick_counter: 0,
            tick_functions: Vec::new(),
            tick_running: false,
            array_iterators: HashMap::new(),
            pending_return_reference: None,
            class_stack: Vec::new(),
            called_class_stack: Vec::new(),
            magic_stack: Vec::new(),
            pending_throw: None,
            spl_autoload_extensions: String::from(".inc,.php"),
            streams: EvalStreamResources::default(),
            json_last_error: 0,
            json_last_error_msg: String::from("No error"),
            default_timezone: String::from("UTC"),
            http_response_code: 200,
            call_file: String::new(),
            call_dir: String::new(),
            call_line: 0,
            file_magic_override: None,
            error_suppression_depth: 0,
        }
    }

    /// Creates a context with an explicit ABI version for compatibility tests.
    #[cfg(test)]
    pub fn for_abi_version(abi_version: u32) -> Self {
        Self {
            abi_version,
            classes: HashMap::new(),
            class_source_files: HashMap::new(),
            class_aliases: HashMap::new(),
            declared_class_names: Arc::default(),
            interfaces: HashMap::new(),
            declared_interface_names: Arc::default(),
            traits: HashMap::new(),
            declared_trait_names: Arc::default(),
            enums: HashMap::new(),
            declared_enum_names: Vec::new(),
            enum_cases: HashMap::new(),
            enum_case_values: HashMap::new(),
            constants: HashMap::new(),
            functions: HashMap::new(),
            closures: HashMap::new(),
            closure_objects: HashMap::new(),
            live_closure_objects: AtomicUsize::new(0),
            live_dynamic_objects: AtomicUsize::new(0),
            live_global_functions: AtomicUsize::new(0),
            live_autoload_contexts: AtomicUsize::new(0),
            retained_context_free_requested: AtomicBool::new(false),
            next_closure_id: 0,
            native_functions: Arc::default(),
            native_methods: Arc::default(),
            native_static_methods: Arc::default(),
            native_constructors: Arc::default(),
            native_class_parents: Arc::default(),
            native_class_attributes: Arc::default(),
            native_method_attributes: Arc::default(),
            native_constant_attributes: Arc::default(),
            native_interface_properties: Arc::default(),
            native_abstract_properties: Arc::default(),
            native_property_types: Arc::default(),
            native_property_defaults: Arc::default(),
            native_property_attributes: Arc::default(),
            static_scope: Box::new(ElephcEvalScope::new()),
            static_slot_keys: Vec::new(),
            static_properties: HashMap::new(),
            static_property_aliases: HashMap::new(),
            class_constants: HashMap::new(),
            included_files: HashSet::new(),
            claimed_aot_include_classlikes: HashSet::new(),
            include_execution_stack: Vec::new(),
            dynamic_objects: HashMap::new(),
            dynamic_destructing_objects: HashSet::new(),
            dynamic_destructed_objects: HashSet::new(),
            dynamic_property_values: HashMap::new(),
            dynamic_property_order: HashMap::new(),
            dynamic_property_aliases: HashMap::new(),
            array_element_aliases: HashMap::new(),
            array_cursors: HashMap::new(),
            dynamic_initialized_properties: HashSet::new(),
            eval_generators: HashMap::new(),
            eval_reflection_attributes: HashMap::new(),
            eval_reflection_classes: HashMap::new(),
            eval_reflection_functions: HashMap::new(),
            eval_reflection_function_closure_targets: HashMap::new(),
            eval_reflection_methods: HashMap::new(),
            eval_reflection_properties: HashMap::new(),
            eval_dynamic_reflection_properties: HashSet::new(),
            eval_reflection_class_constants: HashMap::new(),
            eval_static_callables: HashMap::new(),
            eval_object_callables: HashMap::new(),
            global_scope: None,
            owns_global_scope: false,
            autoload_callbacks: Vec::new(),
            autoloading_classes: HashSet::new(),
            function_stack: Vec::new(),
            returns_by_ref: false,
            strict_types: false,
            tick_interval: 0,
            tick_counter: 0,
            tick_functions: Vec::new(),
            tick_running: false,
            array_iterators: HashMap::new(),
            pending_return_reference: None,
            class_stack: Vec::new(),
            called_class_stack: Vec::new(),
            magic_stack: Vec::new(),
            pending_throw: None,
            spl_autoload_extensions: String::from(".inc,.php"),
            streams: EvalStreamResources::default(),
            json_last_error: 0,
            json_last_error_msg: String::from("No error"),
            default_timezone: String::from("UTC"),
            http_response_code: 200,
            call_file: String::new(),
            call_dir: String::new(),
            call_line: 0,
            file_magic_override: None,
            error_suppression_depth: 0,
        }
    }

    /// Returns the ABI version this context was created for.
    pub const fn abi_version(&self) -> u32 {
        self.abi_version
    }

    /// Enters one nested PHP error-suppression (`@`) expression scope.
    pub fn push_error_suppression(&mut self) {
        self.error_suppression_depth += 1;
    }

    /// Leaves one nested PHP error-suppression (`@`) expression scope.
    pub fn pop_error_suppression(&mut self) {
        self.error_suppression_depth = self.error_suppression_depth.saturating_sub(1);
    }

    /// Reports whether the current eval expression suppresses non-fatal diagnostics.
    pub const fn errors_suppressed(&self) -> bool {
        self.error_suppression_depth != 0
    }

    /// Enters one `isset`/`empty`/`??`/`??=` operand scope, where an uninitialized typed
    /// property ANSWERS instead of raising.
    ///
    /// This is not error suppression. `@` silences a diagnostic that is still produced, and it
    /// propagates into calls; PHP's quiet fetch is a different FETCH MODE that answers "absent"
    /// without ever performing the read, and it stops at a call boundary. Measured against
    /// `php -n` 8.5.6: `$n->leaf->t ?? 'D'` answers `'D'` with `Node::$leaf` itself
    /// uninitialized, so the mode reaches down a whole property/dim chain, while
    /// `readsUninit($n->leaf) ?? 'D'` and `$n->leaf->getT() ?? 'D'` both THROW, so it must not
    /// reach into a call. `enter_call_barrier` is what implements that second half.
    pub fn push_quiet_property_fetch(&mut self) {
        QUIET_PROPERTY_FETCH_DEPTH.with(|depth| depth.set(depth.get() + 1));
    }

    /// Leaves one quiet-fetch operand scope.
    pub fn pop_quiet_property_fetch(&mut self) {
        QUIET_PROPERTY_FETCH_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }

    /// Reports whether an uninitialized typed property should answer rather than raise.
    pub fn quiet_property_fetch(&self) -> bool {
        QUIET_PROPERTY_FETCH_DEPTH.with(|depth| depth.get() != 0)
    }

    /// Suspends the quiet-fetch mode for the duration of a call, returning the depth to restore.
    ///
    /// PHP's quiet fetch does not cross into a function or method body: a callee that reads an
    /// uninitialized typed property raises even when the CALL sits in the operand of `??`.
    #[must_use]
    pub fn enter_call_barrier(&mut self) -> usize {
        QUIET_PROPERTY_FETCH_DEPTH.with(|depth| depth.replace(0))
    }

    /// Restores the quiet-fetch depth saved by `enter_call_barrier`.
    pub fn leave_call_barrier(&mut self, saved: usize) {
        QUIET_PROPERTY_FETCH_DEPTH.with(|depth| depth.set(saved));
    }
}

thread_local! {
    /// Depth of the enclosing quiet-fetch operand scopes for THIS request.
    ///
    /// Deliberately not a field on the context. A single PHP expression routinely crosses
    /// several eval contexts -- `isset($this->p[$k])` inside an included file reaches an
    /// AOT-declared class through the bridge -- and a per-context counter would go quiet on the
    /// context that entered `isset()` while the context that performs the read still raised.
    /// A request runs on one thread and a forked web worker gets its own copy, so the execution
    /// stack the mode belongs to is exactly thread-local, the same reasoning the call-frame
    /// stack already follows.
    static QUIET_PROPERTY_FETCH_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
