//! Purpose:
//! Emits the module's one copy of the eval bridge's non-zero status switch, so the parse-error,
//! uncaught-Throwable, unsupported, pcntl and fatal exits exist once per program instead of once
//! per eval-bridge call site.
//!
//! Called from:
//! - `crate::codegen::block_emit::emit_module()` before the module's own functions.
//! - `crate::codegen::lower_inst::builtins::eval::status::emit_eval_status_check` at every site.
//!
//! Key details:
//! - A site tests the status for zero inline and otherwise calls the helper with the status in
//!   the integer result register and its eval error cell in the second integer argument
//!   register. The helper never returns: it throws or exits.
//! - A Throwable unwinds through the caller's activation record exactly as the inlined throw did:
//!   shared helpers own no cleanup-tracked locals.
//! - Emitted exactly when the module shares its static throws and uses the eval bridge or eval
//!   scope, so a site and the emitter cannot disagree about whether the label exists.

use crate::codegen::context::FunctionContext;
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::shared_state::SharedCodegenState;
use crate::ir::Module;
use crate::types::PhpType;

use super::lower_inst::{emit_shared_eval_status_fail_body, emit_shared_eval_status_fail_cells_body};
use super::shared_helper::emit_shared_helper;
use super::shared_static_throw::module_shares_static_throw;
use super::Result;

/// Label of the helper that handles a non-zero eval bridge status.
const EVAL_STATUS_FAIL_LABEL: &str = "_eir_shared_eval_status_fail";

/// Label of the helper that first releases the site's boxed cells, then fails like the one above.
const EVAL_STATUS_FAIL_CELLS_LABEL: &str = "_eir_shared_eval_status_fail_cells";

/// Whether this module carries the helper.
fn module_wants_helper(module: &Module, shared: &mut SharedCodegenState) -> bool {
    let features = &module.required_runtime_features;
    (features.eval_bridge || features.eval_scope) && module_shares_static_throw(module, shared)
}

/// Returns the helper label a status check in `ctx` should call, if any.
pub(in crate::codegen) fn shared_label(ctx: &mut FunctionContext<'_>) -> Option<&'static str> {
    if ctx.function.name == EVAL_STATUS_FAIL_LABEL {
        return None;
    }
    let module = ctx.module;
    module_wants_helper(module, ctx.shared).then_some(EVAL_STATUS_FAIL_LABEL)
}

/// Returns the label of the releasing variant a status check that boxed operands should call.
pub(in crate::codegen) fn shared_cells_label(ctx: &mut FunctionContext<'_>) -> Option<&'static str> {
    if ctx.function.name == EVAL_STATUS_FAIL_LABEL || ctx.function.name == EVAL_STATUS_FAIL_CELLS_LABEL {
        return None;
    }
    let module = ctx.module;
    module_wants_helper(module, ctx.shared).then_some(EVAL_STATUS_FAIL_CELLS_LABEL)
}

/// Emits the shared helper when the module uses it.
pub(super) fn emit_shared_eval_status(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
    regalloc_linear: bool,
) -> Result<()> {
    if !module_wants_helper(module, shared) {
        return Ok(());
    }
    emit_shared_helper(
        module,
        emitter,
        data,
        shared,
        regalloc_linear,
        EVAL_STATUS_FAIL_LABEL,
        PhpType::Void,
        &format!(
            "--- shared eval status failure: {} (status and error cell in registers) ---",
            EVAL_STATUS_FAIL_LABEL
        ),
        emit_shared_eval_status_fail_body,
    )?;
    emit_shared_helper(
        module,
        emitter,
        data,
        shared,
        regalloc_linear,
        EVAL_STATUS_FAIL_CELLS_LABEL,
        PhpType::Void,
        &format!(
            "--- shared eval status failure releasing boxed cells: {} (status, error cell, cell base, cell mask) ---",
            EVAL_STATUS_FAIL_CELLS_LABEL
        ),
        |ctx| emit_shared_eval_status_fail_cells_body(ctx, EVAL_STATUS_FAIL_LABEL),
    )
}
