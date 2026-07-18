//! Purpose:
//! Emits PHP `glob` path-oriented builtin calls.
//! Marshals path strings into runtime helpers that normalize, split, or enumerate filesystem paths.
//!
//! Called from:
//! - `crate::codegen::builtins::io::emit()`.
//!
//! Key details:
//! - Returned strings and arrays must use runtime allocation/layout compatible with PHP false-on-failure behavior.

use crate::codegen::context::Context;
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::expr::emit_expr;
use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::parser::ast::Expr;
use crate::types::PhpType;

/// Emits codegen for the PHP `glob()` builtin (1-argument legacy form only —
/// this frozen backend is not extended for `$flags`, see the EIR lowering in
/// `crate::codegen_ir::lower_inst::builtins::io::lower_glob` for the real
/// `GLOB_NOSORT`/`GLOB_ONLYDIR`/`GLOB_MARK`/`GLOB_BRACE` semantics).
///
/// Evaluates the pattern argument, then calls `__rt_glob` to expand the glob pattern
/// into an array of matching file paths. Returns `Array<Str>` on success, or `false`
/// on failure (handled by the runtime helper's false-on-failure return convention).
pub fn emit(
    _name: &str,
    args: &[Expr],
    emitter: &mut Emitter,
    ctx: &mut Context,
    data: &mut DataSection,
) -> Option<PhpType> {
    emitter.comment("glob()");
    emit_expr(&args[0], emitter, ctx, data);
    // `__rt_glob` gained `libc_flags`/`onlydir` argument registers (H5); this
    // frozen legacy caller only supports the 1-arg form, so pass flags=0
    // explicitly instead of leaving the registers undefined.
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("mov x3, #0");                                  // libc_flags = 0
            emitter.instruction("mov x4, #0");                                  // onlydir = false
        }
        Arch::X86_64 => {
            emitter.instruction("mov edi, 0");                                  // libc_flags = 0
            emitter.instruction("mov esi, 0");                                  // onlydir = false
        }
    }
    abi::emit_call_label(emitter, "__rt_glob");                                 // call the target-aware runtime helper that expands the glob pattern into a string array
    Some(PhpType::Array(Box::new(PhpType::Str)))
}
