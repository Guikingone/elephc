//! Purpose:
//! Implements the checker driver init phase.
//! Owns one ordered step in building checker state and validating the program before optimization/codegen.
//!
//! Called from:
//! - `crate::types::checker::driver::check_types_impl()`
//!
//! Key details:
//! - Phase order controls diagnostics, available declarations, required libraries, and function-local environments.

use crate::types::predefined_constants::{php_type_of, registered_constants};
use std::collections::{HashMap, HashSet};

use crate::codegen::platform::Target;
use crate::types::pcntl_constants::pcntl_int_constants;
use crate::types::token_constants::TOKEN_INT_CONSTANTS;
use crate::types::PhpType;

use super::super::Checker;

impl Checker {
    /// Constructs a new `Checker` with pre-populated builtin constants and empty declaration tables.
    ///
    /// Initializes the global constant map from the shared catalog, which owns every
    /// unconditionally predefined name, plus the two families it cannot: the target-aware PCNTL
    /// signal/errno table and the profile-aware `T_*` tokenizer table. All other tables
    /// (function declarations, classes, interfaces, enums, etc.) are initialized empty.
    ///
    /// # Arguments
    /// * `target` - The full compilation target, stored for platform- and Apple-variant-specific
    ///   type checks and library requirements.
    ///
    /// # Returns
    /// A `Checker` instance ready for the program to be loaded into.
    pub(crate) fn new(target: Target) -> Self {
        // Every unconditionally registered constant comes from the shared catalog; only its
        // TYPE is declared here, the value is baked per compilation by
        // `codegen_support::prescan::collect_constants`.
        let mut constants = HashMap::new();
        for constant in registered_constants() {
            constants.insert(constant.name.to_string(), php_type_of(constant.value));
        }
        for (name, _) in pcntl_int_constants(target) {
            constants.insert(name.to_string(), PhpType::Int);
        }
        // `T_*` is profile-dependent, so it cannot live in the shared catalog; `prescan`
        // registers the same table for codegen.
        for (name, _values) in TOKEN_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }

        Self {
            target,
            fn_decls: HashMap::new(),
            function_variant_groups: HashMap::new(),
            functions: HashMap::new(),
            resolving_functions: HashSet::new(),
            constants,
            closure_return_types: HashMap::new(),
            callable_sigs: HashMap::new(),
            static_property_callable_sigs: HashMap::new(),
            callable_param_names: HashSet::new(),
            callable_param_sigs: HashMap::new(),
            strict_types: false,
            param_specialization_seen: HashSet::new(),
            unspecialized_seed_params: HashSet::new(),
            callable_return_sigs: HashMap::new(),
            callable_array_return_sigs: HashMap::new(),
            callable_captures: HashMap::new(),
            callable_array_targets: HashMap::new(),
            first_class_callable_targets: HashMap::new(),
            reflection_class_targets: HashMap::new(),
            interfaces: HashMap::new(),
            classes: crate::fast_hash::FastMap::default(),
            declared_classes: HashSet::new(),
            declared_class_parents: HashMap::new(),
            declared_class_interfaces: HashMap::new(),
            enums: HashMap::new(),
            declared_interfaces: HashSet::new(),
            declared_traits: HashSet::new(),
            unresolved_catch_types: HashSet::new(),
            declared_trait_methods: HashMap::new(),
            declared_trait_constants: HashMap::new(),
            current_class: None,
            current_method: None,
            current_function: None,
            current_method_is_static: false,
            current_by_ref_return: false,
            closure_depth: 0,
            extern_functions: HashMap::new(),
            extern_classes: HashMap::new(),
            packed_classes: HashMap::new(),
            extern_globals: HashMap::new(),
            required_libraries: Vec::new(),
            top_level_env: HashMap::new(),
            active_ref_params: HashSet::new(),
            active_globals: HashSet::new(),
            // Filled by `check_types_impl` from the whole program before the first walk; an empty
            // set here just means "no `global` declaration is known", which is the safe default
            // for the handful of tests that build a `Checker` directly.
            program_global_names: HashSet::new(),
            active_statics: HashSet::new(),
            foreach_key_locals: HashSet::new(),
            eval_barrier_active: false,
            // Filled by `check_types_impl` from the whole program before the first walk, like
            // `program_global_names` above. `false` here means "this program cannot conjure a
            // class at run time", which is the conservative answer for the handful of tests that
            // build a `Checker` directly: it keeps the compile-time diagnostic.
            program_defers_unknown_classes: false,
            flow_typed_returns: HashMap::new(),
            method_exists_guards: std::collections::HashSet::new(),
            flow_typed_property_accesses: HashMap::new(),
            null_probe_scope_is_top_level: false,
            pending_null_probe_roots: Vec::new(),
            null_probe_depth: 0,
            break_continue_depth: 0,
            finally_break_continue_bases: Vec::new(),
            current_loop_storage_scope: "main".to_string(),
            declared_local_types: HashMap::new(),
            warnings: Vec::new(),
            reference_property_promotions: HashSet::new(),
            throw_access_sites: HashMap::new(),
            builtin_call_types: HashMap::new(),
            loop_storage_types: HashMap::new(),
            string_incdec_locals: HashSet::new(),
            by_ref_local_storage_types: HashMap::new(),
            widened_ref_params: HashSet::new(),
            widened_ref_param_decls: Vec::new(),
            string_suffix_locals: HashMap::new(),
            dynamic_ref_local_types: HashMap::new(),
            strict_locals: false,
            local_conditional_depth: 0,
            local_binding_depth: HashMap::new(),
            ref_aliased_locals: HashSet::new(),
            ref_bound_locals: HashSet::new(),
            by_ref_capture_boxed_locals: HashSet::new(),
            static_local_names: HashSet::new(),
            typed_local_names: HashSet::new(),
            local_bind_kill_sites: HashMap::new(),
            local_retype_sites: HashMap::new(),
            statement_position_expr: None,
            body_contains_eval: false,
            mixed_storage_locals: HashSet::new(),
            retyped_boxed_storage_locals: HashSet::new(),
            mixed_storage_store_sites: HashMap::new(),
            binding_decision_warnings: HashMap::new(),
            retired_mixed_storage_store_sites: HashSet::new(),
        }
    }
}
