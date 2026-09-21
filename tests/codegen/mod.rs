//! Purpose:
//! Groups the top-level end-to-end codegen test modules into the integration suite.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Submodules group focused fixtures for exceptions, fibers, buffers, preprocessor, namespaces, and related suites.

mod exceptions;
mod fibers;
mod buffers;
mod preprocessor;
mod namespaces;
mod null_sentinel;
mod case_insensitive_symbols;
mod cli;
mod strict_php;
mod lfc;
mod benchmarks;
mod echo_vars;
mod inline_html;
mod eval;
mod symbol_catalog;
mod eval_builtin_parity;
mod eval_filter_var;
mod eval_interface_returns;
mod eval_native_method_retention;
mod eval_parse_str;
mod eval_protected_callbacks;
mod eval_runtime_symbol_links;
mod eval_callable_ref_errors;
mod eval_callables;
mod eval_closures;
mod eval_dynamic_properties;
mod eval_foreach_object;
mod eval_constructors;
mod eval_heredoc;
mod eval_reflection_invocation;
mod eval_reflection_property_values;
mod operators;
mod control_flow;
mod arg_return_coercions;
mod scalar_strings;
mod array_basics;
mod array_coalesce_assignment;
mod numeric_scalars;
mod process_control;
mod type_builtins;
mod casts_and_constants;
mod include_builtin_classes;
mod include_deferred_classes;
mod include_paths;
mod include_reflection;
mod magic_constants;
mod strings;
mod curl;
pub(crate) mod io;
mod mysqli;
mod mysqli_mysql;
mod pdo;
#[cfg(feature = "pdo-dblib")]
mod pdo_dblib;
#[cfg(feature = "pdo-firebird")]
mod pdo_firebird;
#[cfg(feature = "pdo-odbc")]
mod pdo_odbc;
#[cfg(feature = "pdo-informix")]
mod pdo_informix;
#[cfg(feature = "pdo-ibm")]
mod pdo_ibm;
#[cfg(feature = "pdo-sqlsrv")]
mod pdo_sqlsrv;
mod parse_str;
#[cfg(feature = "pdo-oci")]
mod pdo_oci;
#[cfg(feature = "pdo-cubrid")]
mod pdo_cubrid;
mod pdo_mysql;
mod pdo_pgsql;
mod image;
mod pcntl;
mod xml;
mod arrays;
mod calendar;
mod call_counters;
mod callables;
mod system;
mod json;
mod serialize;
mod regressions;
mod objects;
mod destructors;
mod references;
mod runtime_gc;
mod runtime_reachability;
mod math;
mod misc;
mod pointers;
mod ffi;
mod oop;
mod static_class_features;
mod types;
mod optimizer;
mod iterators;
mod spl;
mod generators;
mod dead_strip;
mod locals_retype;
mod stack_guard;
mod zval;
