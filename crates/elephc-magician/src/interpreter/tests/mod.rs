//! Purpose:
//! Interpreter test module wiring and shared fake runtime support.
//! The concrete tests live in focused child modules so each file owns one
//! execution surface instead of one large mixed test bucket.
//!
//! Called from:
//! - `cargo test -p elephc-magician` through Rust's test harness.
//!
//! Key details:
//! - `support` exposes the fake runtime cells used by all interpreter tests.
//! - Child modules import the interpreter entry points from their parent module.

mod append_reference_bind;
mod append_slot_reference;
mod array_append_expression;
mod array_iterator;
mod array_spread;
mod array_literals;
mod builtin_interface_covariance;
mod by_ref_declarations;
mod by_ref_return;
mod builtins_arrays_core;
mod builtins_arrays_iterators;
mod builtins_arrays_sets;
mod builtins_bcmath;
mod builtins_class_metadata;
mod builtins_debug_backtrace;
mod builtins_debug_output;
mod builtins_directory_streams;
mod builtins_output_buffering;
mod builtins_file_streams;
mod builtins_filesystem_metadata;
mod builtins_filesystem_ops;
mod builtins_json;
mod builtins_language_constructs;
mod builtins_math_formatting;
mod builtins_process_pipes;
mod builtins_raw_memory;
mod builtins_readline;
mod builtins_reflection_functions;
mod builtins_scalars;
mod builtins_spl_autoload;
mod builtins_stream_contexts;
mod builtins_stream_extensions;
mod builtins_stream_settings;
mod builtins_stream_sockets;
mod builtins_stream_wrapper_cast;
mod builtins_stream_wrapper_directories;
mod builtins_stream_wrapper_file_io;
mod builtins_stream_wrapper_metadata;
mod builtins_stream_wrapper_options;
mod builtins_stream_wrapper_path_ops;
mod builtins_stream_wrappers;
mod builtins_strings_binary;
mod builtins_strings_encoding;
mod builtins_strings_openssl;
mod builtins_strings_text;
mod builtins_symbols;
mod builtins_system_network;
mod cast_precedence;
mod class_constants;
mod classes;
mod closures;
mod control_flow;
mod core;
mod declare_directives;
mod destructuring;
mod dynamic_calls;
mod dynamic_property_store;
mod element_reference_bind;
mod enums;
mod expressions;
mod foreach_object;
mod functions_namespaces;
mod generators;
mod include_reflection;
mod interleaved_html_include;
mod method_arguments;
mod native_scope;
mod nested_increment;
mod new_class_expressions;
mod object_cast;
mod parent_override_compatibility;
mod reference_bind_expression;
mod reflection_property_store;
mod static_members;
mod strict_types;
mod strict_types_scope;
mod support;
mod trait_adaptations;
