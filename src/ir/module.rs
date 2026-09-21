//! Purpose:
//! Defines the top-level EIR module, data pool, extern declarations, and
//! metadata tables needed by later lowering/codegen phases.
//!
//! Called from:
//! - Future AST-to-EIR lowering and the EIR-to-ASM backend.
//!
//! Key details:
//! - Runtime helper bodies remain outside EIR; modules reference runtime
//!   features and metadata needed to select/link helpers.

use std::collections::{HashMap, HashSet};

use crate::codegen::platform::Target;
use crate::codegen::RuntimeFeatures;
use crate::ir::function::{Function, FunctionId};
use crate::ir::types::IrType;
use crate::parser::ast::{ExprKind, Visibility};
use crate::types::{
    ClassInfo, EnumInfo, ExternClassInfo, FunctionSig, InterfaceInfo, PackedClassInfo, PhpType,
};

/// Data-pool identifier shared by string, float, and name tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DataId(u32);

impl DataId {
    /// Creates a data identifier from its raw zero-based table index.
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw zero-based table index represented by this identifier.
    pub fn as_raw(self) -> u32 {
        self.0
    }
}

/// Scalar value materialized into the closed-world constant registry.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstScalar {
    /// Integer constant.
    Int(i64),
    /// Floating-point constant.
    Float(f64),
    /// Boolean constant.
    Bool(bool),
    /// String constant.
    Str(String),
    /// Null constant.
    Null,
}

/// Method metadata retained for standalone trait reflection.
#[derive(Debug, Clone)]
pub struct TraitMethodInfo {
    pub signature: FunctionSig,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_final: bool,
    pub is_abstract: bool,
}

/// Complete EIR module for one compile target.
#[derive(Debug, Clone)]
pub struct Module {
    pub target: Target,
    pub source_path: Option<String>,
    /// Immutable inputs for compiled source entries, not runtime inclusion state.
    source_catalog: Option<std::sync::Arc<super::SourceCatalog>>,
    /// Catalog sources the COMPILER already included, so the runtime must not include them again.
    ///
    /// The autoload pass performs, at compile time, the inclusions PHP's autoloader would have
    /// performed at runtime: it opens the class file and splices its declarations into the
    /// program. Those declarations exist from process start, so a later `include_once` of the
    /// same path has nothing left to do -- and everything to break, because re-running the file
    /// redeclares a symbol that is already there. This is the file set that answers "already
    /// included" at runtime; a statically-resolved `include` is deliberately NOT in it, because
    /// its body is emitted inline and its guard is set when that code actually runs.
    pub preincluded_sources: std::collections::BTreeSet<std::path::PathBuf>,
    /// LOWERCASE names of compiled classes that must still answer `class_exists($n, false)` false.
    ///
    /// A closed-world build declares everything it compiled from the first instruction. For a
    /// class the autoload pass pulled in ONLY so an existence probe could be answered, that is
    /// not what php does: php loads a class when something uses it, and `$autoload = false` asks
    /// precisely whether that has happened yet. Each name here gets a request-scoped flag that
    /// starts clear and is raised by the first probe that allows autoloading.
    pub deferred_class_loads: std::collections::BTreeSet<String>,
    /// Public function names whose implementation is selected during execution.
    pub(crate) runtime_bound_functions: std::sync::Arc<std::collections::HashSet<String>>,
    /// `--probe` build key, embedded as `_elephc_probe_key` so the probe endpoint
    /// can prove the binary's identity through the HMAC handshake. `None` unless
    /// `--probe` is set.
    pub probe_key: Option<[u8; 32]>,
    pub functions: Vec<Function>,
    pub class_methods: Vec<Function>,
    pub closures: Vec<Function>,
    pub fiber_wrappers: Vec<Function>,
    pub callback_wrappers: Vec<Function>,
    pub extern_callback_trampolines: Vec<Function>,
    pub runtime_callable_invokers: Vec<Function>,
    pub data: DataPool,
    pub extern_decls: Vec<ExternDecl>,
    pub callable_param_sigs: HashMap<(String, String), FunctionSig>,
    pub class_table: ClassTable,
    pub enum_table: EnumTable,
    pub interface_table: InterfaceTable,
    pub trait_table: TraitTable,
    pub declared_class_names: Vec<String>,
    pub declared_interface_names: Vec<String>,
    pub declared_trait_names: Vec<String>,
    pub declared_trait_source_lines: HashMap<String, u32>,
    pub declared_function_source_lines: HashMap<String, (u32, u32)>,
    pub declared_class_source_files: HashMap<String, String>,
    pub declared_function_source_files: HashMap<String, String>,
    pub declared_trait_uses: HashMap<String, Vec<String>>,
    pub declared_trait_method_names: HashMap<String, Vec<String>>,
    pub declared_trait_methods: HashMap<String, HashMap<String, TraitMethodInfo>>,
    pub declared_trait_property_names: HashMap<String, Vec<String>>,
    pub declared_trait_constant_names: HashMap<String, Vec<String>>,
    pub declared_trait_constants: HashMap<String, HashMap<String, crate::parser::ast::Expr>>,
    pub declared_trait_constant_types:
        HashMap<String, HashMap<String, crate::parser::ast::TypeExpr>>,
    pub declared_trait_constant_visibilities: HashMap<String, HashMap<String, Visibility>>,
    pub declared_trait_final_constants: HashMap<String, HashSet<String>>,
    /// Prescanned global constant values used by EIR lowering and eval metadata registration.
    pub global_constants: HashMap<String, (ExprKind, PhpType)>,
    pub class_infos: crate::fast_hash::FastMap<String, ClassInfo>,
    pub interface_infos: HashMap<String, InterfaceInfo>,
    pub enum_infos: HashMap<String, EnumInfo>,
    pub extern_class_infos: HashMap<String, ExternClassInfo>,
    pub packed_class_infos: HashMap<String, PackedClassInfo>,
    pub packed_layouts: PackedLayoutTable,
    pub extern_globals: HashMap<String, PhpType>,
    pub required_runtime_features: RuntimeFeatures,
    /// True when this module is being lowered for a `--web` compile. Threaded
    /// down from the CLI flag (`CliConfig.web`, mirroring what
    /// `codegen_ir::block_emit::emit_module` receives) so lowering can gate
    /// request-superglobal (`$_SERVER`/`$_SESSION`/…) type seeding: only
    /// `--web` builds pre-initialize the shared `_eir_global_*` storage for
    /// those names, so a non-web read/write must not assume a live Hash
    /// pointer is already there.
    pub web: bool,
}

impl Module {
    /// Creates an empty module for the given target.
    pub fn new(target: Target) -> Self {
        Self {
            source_catalog: None,
            preincluded_sources: Default::default(),
            deferred_class_loads: Default::default(),
            runtime_bound_functions: Default::default(),
            target,
            source_path: None,
            probe_key: None,
            functions: Vec::new(),
            class_methods: Vec::new(),
            closures: Vec::new(),
            fiber_wrappers: Vec::new(),
            callback_wrappers: Vec::new(),
            extern_callback_trampolines: Vec::new(),
            runtime_callable_invokers: Vec::new(),
            data: DataPool::default(),
            extern_decls: Vec::new(),
            callable_param_sigs: HashMap::new(),
            class_table: ClassTable::default(),
            enum_table: EnumTable::default(),
            interface_table: InterfaceTable::default(),
            trait_table: TraitTable::default(),
            declared_class_names: Vec::new(),
            declared_interface_names: Vec::new(),
            declared_trait_names: Vec::new(),
            declared_trait_source_lines: HashMap::new(),
            declared_function_source_lines: HashMap::new(),
            declared_class_source_files: HashMap::new(),
            declared_function_source_files: HashMap::new(),
            declared_trait_uses: HashMap::new(),
            declared_trait_method_names: HashMap::new(),
            declared_trait_methods: HashMap::new(),
            declared_trait_property_names: HashMap::new(),
            declared_trait_constant_names: HashMap::new(),
            declared_trait_constants: HashMap::new(),
            declared_trait_constant_types: HashMap::new(),
            declared_trait_constant_visibilities: HashMap::new(),
            declared_trait_final_constants: HashMap::new(),
            global_constants: HashMap::new(),
            class_infos: crate::fast_hash::FastMap::default(),
            interface_infos: HashMap::new(),
            enum_infos: HashMap::new(),
            extern_class_infos: HashMap::new(),
            packed_class_infos: HashMap::new(),
            packed_layouts: PackedLayoutTable::default(),
            extern_globals: HashMap::new(),
            required_runtime_features: RuntimeFeatures::none(),
            web: false,
        }
    }

    /// Creates a module whose source identities are frozen before any body is lowered.
    pub fn with_source_catalog(target: Target, catalog: super::SourceCatalog) -> Self {
        let mut module = Self::new(target);
        module.source_catalog = Some(std::sync::Arc::new(catalog));
        module
    }

    /// Freezes the module's complete source catalog exactly once.
    pub fn bind_source_units(
        &mut self,
        units: impl IntoIterator<Item = crate::resolver::SourceUnit>,
    ) -> Result<(), crate::errors::CompileError> {
        if self.source_catalog.is_some() {
            return Err(crate::errors::CompileError::new(
                crate::span::Span::dummy(), "Source catalog is already bound",
            ));
        }
        self.source_catalog = Some(std::sync::Arc::new(super::SourceCatalog::from_units(units)?));
        Ok(())
    }

    /// Returns source metadata without exposing mutable ID assignment.
    pub fn source_catalog(&self) -> Option<&super::SourceCatalog> { self.source_catalog.as_deref() }

    /// Shares immutable lookup with a lowering body without copying source maps or text.
    pub(crate) fn shared_source_catalog(&self) -> Option<std::sync::Arc<super::SourceCatalog>> {
        self.source_catalog.clone()
    }

    /// Adds a user function and returns its module-local identifier.
    pub fn add_function(&mut self, mut function: Function) -> FunctionId {
        let id = FunctionId::from_raw(self.functions.len() as u32);
        function.set_id(id);
        self.functions.push(function);
        id
    }

    /// Adds a closure function and returns its closure-table identifier.
    pub fn add_closure(&mut self, mut function: Function) -> FunctionId {
        let id = FunctionId::from_raw(self.closures.len() as u32);
        function.set_id(id);
        self.closures.push(function);
        id
    }
}

/// Deterministic literal/name pool used by IR immediates and printer output.
#[derive(Debug, Clone, Default)]
pub struct DataPool {
    pub strings: Vec<String>,
    pub float_literals: Vec<f64>,
    pub global_names: Vec<String>,
    pub function_names: Vec<String>,
    pub class_names: Vec<String>,
    pub method_names: Vec<String>,
    pub property_names: Vec<String>,
}

impl DataPool {
    /// Interns a string literal and returns its stable data identifier.
    pub fn intern_string(&mut self, value: &str) -> DataId {
        intern_string_vec(&mut self.strings, value)
    }

    /// Interns a floating-point literal by exact bit pattern.
    pub fn intern_float(&mut self, value: f64) -> DataId {
        if let Some(idx) = self
            .float_literals
            .iter()
            .position(|existing| existing.to_bits() == value.to_bits())
        {
            return DataId::from_raw(idx as u32);
        }
        let id = DataId::from_raw(self.float_literals.len() as u32);
        self.float_literals.push(value);
        id
    }

    /// Interns a global symbol name and returns its stable data identifier.
    pub fn intern_global_name(&mut self, value: &str) -> DataId {
        intern_string_vec(&mut self.global_names, value)
    }

    /// Interns a function name and returns its stable data identifier.
    pub fn intern_function_name(&mut self, value: &str) -> DataId {
        intern_string_vec(&mut self.function_names, value)
    }

    /// Interns a class name and returns its stable data identifier.
    pub fn intern_class_name(&mut self, value: &str) -> DataId {
        intern_string_vec(&mut self.class_names, value)
    }
}

/// Interns `value` into a string vector and returns its zero-based index.
fn intern_string_vec(values: &mut Vec<String>, value: &str) -> DataId {
    if let Some(idx) = values.iter().position(|existing| existing == value) {
        return DataId::from_raw(idx as u32);
    }
    let id = DataId::from_raw(values.len() as u32);
    values.push(value.to_string());
    id
}

/// C-facing extern function declaration referenced by EIR.
#[derive(Debug, Clone)]
pub struct ExternDecl {
    pub name: String,
    pub params: Vec<ExternParamDecl>,
    pub return_type: IrType,
    pub return_php_type: PhpType,
    pub link_libs: Vec<String>,
}

/// One extern function parameter.
#[derive(Debug, Clone)]
pub struct ExternParamDecl {
    pub name: String,
    pub ir_type: IrType,
    pub php_type: PhpType,
}

/// Minimal class metadata table placeholder for Phase 02.
#[derive(Debug, Clone, Default)]
pub struct ClassTable {
    pub names: Vec<String>,
}

/// Minimal enum metadata table placeholder for Phase 02.
#[derive(Debug, Clone, Default)]
pub struct EnumTable {
    pub names: Vec<String>,
}

/// Minimal interface metadata table placeholder for Phase 02.
#[derive(Debug, Clone, Default)]
pub struct InterfaceTable {
    pub names: Vec<String>,
}

/// Minimal trait metadata table placeholder for Phase 04 introspection.
#[derive(Debug, Clone, Default)]
pub struct TraitTable {
    pub names: Vec<String>,
}

/// Minimal packed-layout metadata table placeholder for Phase 02.
#[derive(Debug, Clone, Default)]
pub struct PackedLayoutTable {
    pub names: Vec<String>,
}
