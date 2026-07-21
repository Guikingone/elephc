//! Purpose:
//! Home of the PHP `defined` builtin: its declaration, type-check hook, and lowering.
//!
//! Called from:
//! - The builtin registry (declaration), the type checker (check hook), and the EIR
//!   backend (lower hook), all via `crate::builtins::registry`.
//!
//! Key details:
//! - A string-literal name folds to a compile-time boolean during lowering; a
//!   non-literal name is accepted and lowered to the `__rt_defined` closed-world
//!   constant-registry lookup, so dynamic `defined($name)` probes work at runtime.
//! - `lower` delegates to the module-level `lower_defined` in `src/codegen/lower_inst/builtins.rs`.

use crate::codegen::context::FunctionContext;
use crate::codegen::CodegenIrError;
use crate::ir::Instruction;

builtin! {
    name: "defined",
    area: System,
    params: [constant_name: Str],
    returns: Bool,
    lower: lower,
    summary: "Checks whether the given named constant exists.",
}

/// Lowers a `defined` call by delegating to the shared module-level emitter.
fn lower(ctx: &mut FunctionContext, inst: &Instruction) -> Result<(), CodegenIrError> {
    crate::codegen::lower_inst::builtins::lower_defined(ctx, inst)
}
