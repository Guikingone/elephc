//! Purpose:
//! Emits PHP `scandir` path-oriented builtin calls.
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

/// Emits the `scandir` builtin call (1-argument legacy form only — this frozen
/// backend is not extended for `$sorting_order`/`$context`, see the EIR
/// lowering in `crate::codegen_ir::lower_inst::builtins::io::lower_scandir_with_sort`
/// for the real optional-arg semantics).
///
/// First argument (path) is evaluated and emitted as an expression. Then calls the
/// `__rt_scandir` runtime helper which enumerates directory entries and returns them
/// as a string array. On failure (e.g., invalid path, not a directory), runtime returns
/// `false` rather than an array — callers must handle false-on-failure semantics.
///
/// # Arguments
/// * `_name` - Unused; present for dispatcher uniformity with other builtin emitters.
/// * `args` - Must contain at least a path expression as the first element.
/// * `emitter` - Target-aware assembly emitter.
/// * `ctx` - Codegen context carrying variable layout and metadata.
/// * `data` - Data section for relocations and static storage.
///
/// # Returns
/// `Some(PhpType::Array(Box::new(PhpType::Str)))` — callers should treat `false`
/// from runtime as the actual failure indicator.
pub fn emit(
    _name: &str,
    args: &[Expr],
    emitter: &mut Emitter,
    ctx: &mut Context,
    data: &mut DataSection,
) -> Option<PhpType> {
    emitter.comment("scandir()");
    emit_expr(&args[0], emitter, ctx, data);
    // `__rt_scandir` gained a `$sorting_order` argument register (H5); this frozen
    // legacy caller only supports the 1-arg form, so pass the SCANDIR_SORT_ASCENDING
    // default explicitly instead of leaving the register undefined.
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction("mov x3, #0"),                     // default sorting_order = SCANDIR_SORT_ASCENDING
        Arch::X86_64 => emitter.instruction("mov edi, 0"),                      // default sorting_order = SCANDIR_SORT_ASCENDING
    }
    abi::emit_call_label(emitter, "__rt_scandir");                              // call the target-aware runtime helper that lists directory entries into a string array
    Some(PhpType::Array(Box::new(PhpType::Str)))
}
