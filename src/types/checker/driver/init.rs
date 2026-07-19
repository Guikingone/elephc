//! Purpose:
//! Implements the checker driver init phase.
//! Owns one ordered step in building checker state and validating the program before optimization/codegen.
//!
//! Called from:
//! - `crate::types::checker::driver::check_types_impl()`
//!
//! Key details:
//! - Phase order controls diagnostics, available declarations, required libraries, and function-local environments.

use std::collections::{HashMap, HashSet};

use crate::codegen::platform::Platform;
use crate::types::array_constants::ARRAY_INT_CONSTANTS;
use crate::types::date_constants::{DATE_INT_CONSTANTS, DATE_STRING_CONSTANTS};
use crate::types::error_constants::ERROR_INT_CONSTANTS;
use crate::types::json_constants::JSON_INT_CONSTANTS;
use crate::types::php_runtime_constants::{
    PHP_RUNTIME_INT_CONSTANTS, PHP_RUNTIME_PLATFORM_CONSTANTS,
};
use crate::types::stream_constants::{GLOB_PLATFORM_CONSTANTS, STREAM_INT_CONSTANTS};
use crate::types::locale_constants::LOCALE_INT_CONSTANTS;
use crate::types::preg_constants::PREG_INT_CONSTANTS;
use crate::types::string_constants::STRING_INT_CONSTANTS;
use crate::types::sort_constants::SORT_INT_CONSTANTS;
use crate::types::mbstring_constants::MBSTRING_INT_CONSTANTS;
use crate::types::filter_constants::FILTER_INT_CONSTANTS;
use crate::types::pcntl_constants::{PCNTL_INT_CONSTANTS, PCNTL_PLATFORM_SIGNALS};
use crate::types::upload_constants::UPLOAD_ERR_INT_CONSTANTS;
use crate::types::url_constants::URL_INT_CONSTANTS;
use crate::types::tokenizer_constants::TOKENIZER_INT_CONSTANTS;
use crate::types::xml_constants::XML_INT_CONSTANTS;
use crate::types::PhpType;

use super::super::Checker;

impl Checker {
    /// Constructs a new `Checker` with pre-populated builtin constants and empty declaration tables.
    ///
    /// Initializes the global constant map with PHP built-in constants (`PHP_OS`, pathinfo
    /// constants, `FNM_*` flags, `STDIN`/`STDOUT`/`STDERR` stream resources, `LOCK_*` constants),
    /// array constants, JSON integer constants, and preg flag constants. All other tables (function declarations,
    /// classes, interfaces, enums, etc.) are initialized empty.
    ///
    /// # Arguments
    /// * `target_platform` - The compilation target platform, stored for use in platform-specific
    ///   type checks and library requirements.
    ///
    /// # Returns
    /// A `Checker` instance ready for the program to be loaded into.
    pub(super) fn new(target_platform: Platform) -> Self {
        let mut constants = HashMap::new();
        constants.insert("PHP_OS".to_string(), PhpType::Str);
        constants.insert("PATHINFO_DIRNAME".to_string(), PhpType::Int);
        constants.insert("PATHINFO_BASENAME".to_string(), PhpType::Int);
        constants.insert("PATHINFO_EXTENSION".to_string(), PhpType::Int);
        constants.insert("PATHINFO_FILENAME".to_string(), PhpType::Int);
        constants.insert("PATHINFO_ALL".to_string(), PhpType::Int);
        constants.insert("FNM_NOESCAPE".to_string(), PhpType::Int);
        constants.insert("FNM_PATHNAME".to_string(), PhpType::Int);
        constants.insert("FNM_PERIOD".to_string(), PhpType::Int);
        constants.insert("FNM_CASEFOLD".to_string(), PhpType::Int);
        constants.insert("STDIN".to_string(), PhpType::stream_resource());
        constants.insert("STDOUT".to_string(), PhpType::stream_resource());
        constants.insert("STDERR".to_string(), PhpType::stream_resource());
        constants.insert("LOCK_SH".to_string(), PhpType::Int);
        constants.insert("LOCK_EX".to_string(), PhpType::Int);
        constants.insert("LOCK_UN".to_string(), PhpType::Int);
        constants.insert("LOCK_NB".to_string(), PhpType::Int);
        for (name, _value) in ARRAY_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in JSON_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in STREAM_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in PREG_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in DATE_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in ERROR_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in PHP_RUNTIME_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in LOCALE_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in STRING_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in SORT_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in MBSTRING_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in FILTER_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in PCNTL_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in UPLOAD_ERR_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in URL_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in TOKENIZER_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        for (name, _value) in XML_INT_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        // Platform-conditional user signals (SIGUSR1/SIGUSR2): only the NAME is
        // needed for type-checking; the target-specific VALUE is materialized by
        // the codegen prescan. Register unconditionally (target-agnostic).
        for (name, _macos_value, _linux_value) in PCNTL_PLATFORM_SIGNALS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        // Platform-conditional runtime constants (PHP_MAXPATHLEN): same pattern —
        // only the NAME is needed here, the target-specific VALUE is materialized
        // by the codegen prescan.
        for (name, _macos_value, _linux_value) in PHP_RUNTIME_PLATFORM_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        // Platform-conditional glob() bit flags (GLOB_MARK/NOSORT/BRACE/...): same
        // pattern — only the NAME is needed here, the target-specific VALUE is
        // materialized by the codegen prescan.
        for (name, _macos_value, _linux_value) in GLOB_PLATFORM_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Int);
        }
        // PHP_SAPI, PHP_VERSION, PHP_OS_FAMILY, and PCRE_VERSION are string constants.
        // Their type is registered here; their values are materialized in prescan.
        constants.insert("PHP_SAPI".to_string(), PhpType::Str);
        constants.insert("PHP_VERSION".to_string(), PhpType::Str);
        constants.insert("PHP_OS_FAMILY".to_string(), PhpType::Str);
        constants.insert("PCRE_VERSION".to_string(), PhpType::Str);
        // DATE_* format-string constants (DATE_ATOM, DATE_RFC3339, ...): registered
        // as Str here; their literal values are materialized in prescan.
        for (name, _value) in DATE_STRING_CONSTANTS {
            constants.insert((*name).to_string(), PhpType::Str);
        }

        Self {
            target_platform,
            fn_decls: HashMap::new(),
            function_variant_groups: HashMap::new(),
            functions: HashMap::new(),
            constants,
            closure_return_types: HashMap::new(),
            callable_sigs: HashMap::new(),
            callable_param_names: HashSet::new(),
            callable_param_sigs: HashMap::new(),
            param_specialization_seen: HashSet::new(),
            callable_return_sigs: HashMap::new(),
            callable_array_return_sigs: HashMap::new(),
            callable_captures: HashMap::new(),
            callable_array_targets: HashMap::new(),
            first_class_callable_targets: HashMap::new(),
            interfaces: HashMap::new(),
            classes: HashMap::new(),
            static_return_methods: HashSet::new(),
            declared_classes: HashSet::new(),
            enums: HashMap::new(),
            declared_interfaces: HashSet::new(),
            current_class: None,
            bound_scope_context: None,
            current_method: None,
            current_method_is_static: false,
            current_by_ref_return: false,
            closure_depth: 0,
            in_callable_body: false,
            extern_functions: HashMap::new(),
            extern_classes: HashMap::new(),
            packed_classes: HashMap::new(),
            extern_globals: HashMap::new(),
            required_libraries: Vec::new(),
            top_level_env: HashMap::new(),
            active_ref_params: HashSet::new(),
            declared_byref_param_locals: HashSet::new(),
            active_globals: HashSet::new(),
            active_statics: HashSet::new(),
            foreach_key_locals: HashSet::new(),
            declared_typed_locals: HashSet::new(),
            break_continue_depth: 0,
            finally_break_continue_bases: Vec::new(),
            warnings: Vec::new(),
            absent_class_warnings: std::cell::RefCell::new(Vec::new()),
            reference_property_promotions: HashSet::new(),
            reference_property_rebind_targets: HashSet::new(),
            func_args_functions: HashSet::new(),
        }
    }
}
