//! Purpose:
//! Home of the PHP `proc_close` builtin: declaration and lowering (no check hook —
//! `process` is a `Mixed` resource|false accepted as-is).
//!
//! Called from:
//! - The builtin registry (declaration) and the EIR backend (lower hook) via
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - C1a wires the surface only; `__rt_proc_close` is a stub returning -1.
//! - `process` is the `resource|false` value returned by `proc_open`.

use crate::codegen::context::FunctionContext;
use crate::codegen::CodegenIrError;
use crate::ir::Instruction;

builtin! {
    name: "proc_close",
    area: Io,
    params: [process: Mixed],
    returns: Int,
    lower: lower,
    summary: "Close a process opened by proc_open and return the exit status.",
    php_manual: "function.proc-close",
}

/// Lowers a `proc_close` call by dispatching to the shared io emitter.
fn lower(ctx: &mut FunctionContext, inst: &Instruction) -> Result<(), CodegenIrError> {
    crate::codegen::lower_inst::builtins::io::lower_proc_close(ctx, inst)
}