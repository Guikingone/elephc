//! Purpose:
//! Provides the public type-system entry point used by the compile pipeline.
//! Re-exports type models, signatures, schemas, call-argument planning, warnings, and checker results.
//!
//! Called from:
//! - `crate::pipeline::compile()`
//!
//! Key details:
//! - `check()` runs after name resolution and before optimization/codegen so later passes receive canonical type metadata.

/// Type checking module.
pub mod checker;
/// Trait flattening and resolution.
pub mod traits;
/// Array key type inference, normalization, and PHP integer/string coercion rules.
mod array_keys;
/// PHP array extension integer constants.
pub(crate) mod array_constants;
/// PHP error-level and debug-backtrace integer constants (E_ERROR, E_ALL, DEBUG_BACKTRACE_*).
pub(crate) mod error_constants;
/// PHP runtime and version integer constants (PHP_MAJOR_VERSION, PHP_INT_SIZE, etc.).
pub(crate) mod php_runtime_constants;
/// Call argument planning: named, positional, and spread semantics.
pub(crate) mod call_args;
/// Fiber/stack introspection for async and coroutine analysis.
pub(crate) mod fibers;
/// `ext/date` integer constants (e.g. `SUNFUNCS_RET_*`).
pub(crate) mod date_constants;
/// POSIX locale category integer constants (LC_ALL, LC_NUMERIC, etc.).
pub(crate) mod locale_constants;
/// String-function integer constants (STR_PAD_*, ENT_* html-entity flags).
pub(crate) mod string_constants;
/// Sort comparison-mode integer constants (SORT_STRING, SORT_NATURAL).
pub(crate) mod sort_constants;
/// mbstring case-mode integer constants (MB_CASE_FOLD_SIMPLE).
pub(crate) mod mbstring_constants;
/// `ext/filter` validation-filter integer constants (FILTER_VALIDATE_BOOL[EAN]).
pub(crate) mod filter_constants;
/// `ext/pcntl` signal integer constants (SIGINT, SIGTERM, SIG_DFL, SIG_IGN, etc.).
pub(crate) mod pcntl_constants;
/// PHP file-upload error integer constants (UPLOAD_ERR_*).
pub(crate) mod upload_constants;
/// `parse_url()`/`http_build_query()` integer constants (PHP_URL_*, PHP_QUERY_*).
pub(crate) mod url_constants;
/// `ext/tokenizer` token-kind integer constants (T_START_HEREDOC, T_END_HEREDOC).
pub(crate) mod tokenizer_constants;
/// `ext/dom` node-kind integer constants (XML_DOCUMENT_TYPE_NODE).
pub(crate) mod xml_constants;
/// `ENT_*` HTML-escaping flag constants shared by checker and codegen.
pub(crate) mod ent_constants;
/// C FFI type mapping utilities.
mod ffi;
/// JSON literal constant type inference.
pub(crate) mod json_constants;
/// PHP type model and type environment for tracking variable types.
mod model;
/// Preg/PCRE flag constants shared by checker and codegen.
pub(crate) mod preg_constants;
/// Return-to-argument storage alias summaries used by ownership lowering.
mod return_alias;
/// Type checker result types and the `check` entry point.
mod result;
/// Class, interface, enum, and FFI schema definitions.
mod schema;
/// `ext/session` integer constants (`PHP_SESSION_*`).
pub(crate) mod session_constants;
/// Function signature representation and builtin signature helpers.
mod signatures;
pub(crate) mod stream_constants;
/// Type checker diagnostics and warnings.
mod warnings;

pub(crate) use array_keys::{
    array_key_type_from_value_type, is_php_integer_array_key, merge_array_key_types,
    normalized_array_key_type, parse_php_string_offset_literal,
    static_array_key_forces_hash_storage,
};
pub use ffi::{ctype_stack_size, ctype_to_php_type, packed_type_size};
pub use model::{PhpType, TypeEnv};
pub(crate) use return_alias::{
    collect_return_alias_summaries, ReturnAliasSummaries, ReturnArgAlias,
};
pub use result::{check_with_target, CheckResult, ThrowAccessInfo, ThrowAccessKind};
pub use schema::{
    AttrArgEntry, AttrArgValue, AttrKey, ClassInfo, EnumCaseInfo, EnumCaseValue, EnumInfo,
    ExternClassInfo, ExternFieldInfo, ExternFunctionSig, InterfaceInfo, PackedClassInfo,
    PackedFieldInfo, PropertyHookContract,
};
pub(crate) use schema::{collect_attribute_args, collect_attribute_names};
pub(crate) use signatures::{
    builtin_call_sig, callable_wrapper_sig, first_class_callable_builtin_sig,
};
pub use signatures::FunctionSig;

/// Type checks the program after name resolution. Returns `CheckResult` with type
/// metadata, function/class/interface/enum info, warnings, required libraries, and the
/// internal `Mixed` type for heterogeneous assoc-array values. Runs before optimization/codegen.
#[allow(dead_code)]
pub fn check(
    program: &crate::parser::ast::Program,
) -> Result<CheckResult, crate::errors::CompileError> {
    result::check(program)
}
