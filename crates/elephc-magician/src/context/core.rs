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
    pub(super) declared_class_names: Vec<String>,
    pub(super) interfaces: HashMap<String, EvalInterface>,
    pub(super) declared_interface_names: Vec<String>,
    pub(super) traits: HashMap<String, EvalTrait>,
    pub(super) declared_trait_names: Vec<String>,
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
    pub(super) native_functions: HashMap<String, NativeFunction>,
    pub(super) native_methods: HashMap<(String, String), NativeCallableSignature>,
    pub(super) native_static_methods: HashMap<(String, String), NativeCallableSignature>,
    pub(super) native_constructors: HashMap<String, NativeCallableSignature>,
    pub(super) native_class_parents: HashMap<String, String>,
    pub(super) native_class_attributes: HashMap<String, Vec<EvalAttribute>>,
    pub(super) native_method_attributes: HashMap<(String, String), Vec<EvalAttribute>>,
    pub(super) native_constant_attributes: HashMap<(String, String), Vec<EvalAttribute>>,
    pub(super) native_interface_properties: HashMap<String, Vec<(String, EvalInterfaceProperty)>>,
    pub(super) native_abstract_properties: HashMap<String, Vec<(String, EvalInterfaceProperty)>>,
    pub(super) native_property_types: HashMap<(String, String), EvalParameterType>,
    pub(super) native_property_defaults: HashMap<(String, String), NativeCallableDefault>,
    pub(super) native_property_attributes: HashMap<(String, String), Vec<EvalAttribute>>,
    pub(super) static_locals: HashMap<(String, String), RuntimeCellHandle>,
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
    pub(super) autoload_callbacks: Vec<RuntimeCellHandle>,
    pub(super) autoloading_classes: HashSet<String>,
    pub(super) function_stack: Vec<String>,
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
            declared_class_names: Vec::new(),
            interfaces: HashMap::new(),
            declared_interface_names: Vec::new(),
            traits: HashMap::new(),
            declared_trait_names: Vec::new(),
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
            native_functions: HashMap::new(),
            native_methods: HashMap::new(),
            native_static_methods: HashMap::new(),
            native_constructors: HashMap::new(),
            native_class_parents: HashMap::new(),
            native_class_attributes: HashMap::new(),
            native_method_attributes: HashMap::new(),
            native_constant_attributes: HashMap::new(),
            native_interface_properties: HashMap::new(),
            native_abstract_properties: HashMap::new(),
            native_property_types: HashMap::new(),
            native_property_defaults: HashMap::new(),
            native_property_attributes: HashMap::new(),
            static_locals: HashMap::new(),
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
            declared_class_names: Vec::new(),
            interfaces: HashMap::new(),
            declared_interface_names: Vec::new(),
            traits: HashMap::new(),
            declared_trait_names: Vec::new(),
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
            native_functions: HashMap::new(),
            native_methods: HashMap::new(),
            native_static_methods: HashMap::new(),
            native_constructors: HashMap::new(),
            native_class_parents: HashMap::new(),
            native_class_attributes: HashMap::new(),
            native_method_attributes: HashMap::new(),
            native_constant_attributes: HashMap::new(),
            native_interface_properties: HashMap::new(),
            native_abstract_properties: HashMap::new(),
            native_property_types: HashMap::new(),
            native_property_defaults: HashMap::new(),
            native_property_attributes: HashMap::new(),
            static_locals: HashMap::new(),
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
}
