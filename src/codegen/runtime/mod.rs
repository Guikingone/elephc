//! Purpose:
//! Defines the runtime module boundary and re-exports the runtime emission entry points.
//! This is the narrow public surface used by codegen to attach helper assembly and data sections.
//!
//! Called from:
//! - `crate::codegen::driver_support::generate_runtime()` while building the cached runtime object.
//! - `crate::codegen::main_emission::finish_user_asm()` when appending user-specific runtime data.
//!
//! Key details:
//! - Keep this surface small: runtime codegen imports these re-exports instead of reaching into leaf emitters directly.

mod arrays;
mod buffers;
mod callables;
mod data;
mod diagnostics;
mod emitters;
mod exceptions;
mod fibers;
/// `ext/filter` (`filter_var()`) dedicated runtime parsers (int/float/bool).
mod filter;
/// Runtime helpers for generator state management (yield, resume, stack frames).
pub(crate) mod generators;
mod io;
mod objects;
mod pointers;
mod strings;
/// Standard PHP library constants, functions, and classes.
pub(crate) mod spl;
mod system;

pub(crate) use data::emit_class_registry_data;
pub(crate) use data::emit_class_relation_registry_data;
pub(crate) use data::emit_const_registry_data;
pub(crate) use data::emit_runtime_data_fixed;
/// Emit the closed-world flat method/property registry tables backing dynamic
/// `ReflectionMethod`/`ReflectionProperty` construction and `getMethod`/`getProperty`.
pub(crate) use data::emit_reflect_member_registry_data;
pub(crate) use data::{
    CLASS_ID_ROW_CLASS_ID_OFFSET, CLASS_ID_ROW_SIZE, INDEX_ROW_SIZE, METHOD_ROW_MODIFIERS_OFFSET,
    METHOD_ROW_REAL_NAME_LEN_OFFSET, METHOD_ROW_REAL_NAME_PTR_OFFSET, METHOD_ROW_SIZE,
    PROPERTY_ROW_MODIFIERS_OFFSET, PROPERTY_ROW_SIZE,
};
pub(crate) use data::{method_decl_order_and_names, property_decl_order_and_names};
/// Emit fixed runtime data section (symbols, constants, type metadata).
pub(crate) use data::emit_runtime_data_user;
/// Emit user-program-specific runtime data section.
pub(crate) use emitters::emit_runtime;
/// Fatal-message constants for `printf()`/`vprintf()` inside an active output
/// buffer, and the shared entry guard every raw-syscall output path that
/// bypasses `__rt_stdout_write` (`ob_start()`'s choke point) opens with.
/// Re-exported here (rather than reaching into `runtime::data`/`runtime::io`
/// directly) so `codegen_ir::lower_inst::builtins::strings`'s inlined
/// per-call-site `printf`/`vprintf` write path can use them — those two are
/// NOT shared runtime routines emitted once (unlike the var_dump/print_r
/// walkers, which stay within the `runtime` module tree and reference
/// `runtime::data`/`runtime::io` directly).
pub(crate) use data::{OB_PRINTF_UNSUPPORTED_MSG, OB_VPRINTF_UNSUPPORTED_MSG};
pub(crate) use io::emit_ob_incompat_check;
/// Emit full runtime helpers (orchestrates all runtime sections).
pub(crate) use fibers::{
    FIBER_CALLABLE_OFFSET, FIBER_PENDING_THROW_OFFSET, FIBER_STACK_BASE_OFFSET,
    FIBER_STACK_SIZE_OFFSET, FIBER_START_ARG_COUNT_OFFSET, FIBER_START_ARGS_MAX,
    FIBER_START_ARGS_OFFSET, FIBER_STATE_NOT_STARTED, FIBER_STATE_RUNNING,
    FIBER_STATE_SUSPENDED, FIBER_STATE_TERMINATED, FIBER_TRANSFER_VALUE_OFFSET,
    FIBER_USER_ARG_MAX_OFFSET,
};
