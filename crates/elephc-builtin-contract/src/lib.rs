//! Purpose:
//! Dependency-neutral PHP builtin identities and surface contracts shared by
//! the elephc compiler and the Magician runtime interpreter.
//!
//! Called from:
//! - `elephc::builtins` when joining compiler-specific checker/EIR semantics.
//! - `elephc_magician::interpreter::builtins` when joining eval dispatch hooks.
//!
//! Key details:
//! - This crate must remain independent of compiler AST, EIR, codegen, EvalIR,
//!   target ABI, and runtime-cell implementations.
//! - Builtin identities are derived from canonical lowercase PHP names and are
//!   validated for uniqueness by catalog consumers.

mod catalog_data;
mod catalog_surfaces;
mod eval_profile;
mod id;
mod registry;
mod requirements;
mod runtime_id;
mod spec;
mod support;

pub use id::BuiltinId;
pub use eval_profile::{
    eval_signature, eval_signature_profile, EvalSignatureOverrideReason, EvalSignatureProfile,
};
pub use registry::{contracts, lookup, lookup_id};
pub use runtime_id::{runtime_builtin_id, RuntimeBuiltinId, RuntimeBuiltinStatus};
pub use spec::{
    Area, BuiltinContract, BuiltinKind, BuiltinRequirement, BuiltinSignature, DefaultSpec,
    ParamSpec, PassingMode, TypeSpec,
};
pub use support::{
    aot_support, backend_support, eval_execution, eval_support, BackendImplementation,
    BackendSupport, BuiltinBackend, EvalAdapterReason, EvalExecution, UnsupportedReason,
};
