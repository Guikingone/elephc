//! Purpose:
//! Home of the PHP `proc_open` builtin: its declaration, type-check hook, and lowering.
//!
//! Called from:
//! - The builtin registry (declaration), the type checker (check hook), and the EIR
//!   backend (lower hook), all via `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Union(stream_resource, Bool)` to reflect PHP's false-on-failure.
//! - C1a wires the surface only; `__rt_proc_open` is a stub returning -1 (boxes false).
//! - Pipe-only: `descriptor_spec` is an array, `command` is a string, `pipes` is by-ref.
//! - `TypeSpec` has no plain `Array` variant, so the two array parameters are declared
//!   as `Mixed` (matching `preg_match`'s by-ref `matches` array). The check hook does
//!   not refine on them.
//! - `cwd`/`env`/`options` are deferred to C2; the C1a signature has exactly 3 params.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::codegen::context::FunctionContext;
use crate::codegen::CodegenIrError;
use crate::errors::CompileError;
use crate::ir::Instruction;
use crate::types::PhpType;

builtin! {
    name: "proc_open",
    area: Io,
    params: [descriptor_spec: Mixed, command: Str, ref pipes: Mixed],
    returns: Mixed,
    check: check,
    lower: lower,
    summary: "Execute a command and open file pointers for I/O.",
    php_manual: "function.proc-open",
}

/// Returns `Union(stream_resource, Bool)` for the proc_open result.
///
/// `descriptor_spec` is an array and `pipes` is a by-ref array written by the runtime;
/// no resource validation is performed here. The common registry path pre-infers the
/// arguments.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(cx.checker.normalize_union_type(vec![
        PhpType::stream_resource(),
        PhpType::Bool,
    ]))
}

/// Lowers a `proc_open` call by dispatching to the shared io emitter.
fn lower(ctx: &mut FunctionContext, inst: &Instruction) -> Result<(), CodegenIrError> {
    crate::codegen::lower_inst::builtins::io::lower_proc_open(ctx, inst)
}