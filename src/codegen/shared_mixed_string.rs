//! Purpose:
//! Emits module-wide helpers for boxed Mixed string contexts.
//! Replaces repeated per-class `__toString` dispatch ladders with one body per context mode.
//!
//! Called from:
//! - `crate::codegen::block_emit::emit_module()` before ordinary function bodies.
//! - `crate::codegen::lower_inst::conversions` at boxed Mixed string sites.
//! - `crate::codegen::frame` when deciding whether a function reserves the nested-call register.
//!
//! Key details:
//! - Sharing starts at two sites and only when the module has a usable `__toString` candidate.
//! - Result and stdout modes have independent helpers and memoized decisions.
//! - The helper body reuses the same lowering routine as the former inline ladder.

use crate::codegen::context::FunctionContext;
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::shared_state::SharedCodegenState;
use crate::ir::{Function, IrType, Module};
use crate::types::PhpType;

use super::lower_inst::{emit_mixed_string_dispatch_from_result, MixedStringContextMode};
use super::shared_helper::{emit_shared_helper, helper_value};
use super::Result;

/// Assembly label for the helper that returns a PHP string.
pub(super) const MIXED_TO_STRING_LABEL: &str = "_eir_shared_mixed_to_string";
/// Assembly label for the helper that writes the converted value to stdout.
pub(super) const MIXED_ECHO_LABEL: &str = "_eir_shared_mixed_echo";

/// Returns whether the module should share one Mixed string-context mode.
pub(super) fn module_shares_mixed_string_ladder(
    module: &Module,
    mode: &MixedStringContextMode,
    shared: &mut SharedCodegenState,
) -> bool {
    let mode_index = match mode {
        MixedStringContextMode::Result => 0,
        MixedStringContextMode::Stdout => 1,
    };
    if let Some(cached) = shared.mixed_string_sharing(mode_index) {
        return cached;
    }
    let shares = module_has_tostring_candidate(module) && mixed_string_site_count(module, mode) >= 2;
    shared.set_mixed_string_sharing(mode_index, shares);
    shares
}

/// Returns whether any emitted class publishes a zero-argument `__toString` method.
fn module_has_tostring_candidate(module: &Module) -> bool {
    let method_key = crate::names::php_symbol_key("__toString");
    module.class_infos.values().any(|class_info| {
        class_info
            .methods
            .get(&method_key)
            .is_some_and(|signature| signature.params.is_empty())
    })
}

/// Counts string-context sites of one mode across all emitted function bodies.
fn mixed_string_site_count(module: &Module, mode: &MixedStringContextMode) -> usize {
    module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
        .map(|function| {
            function
                .instructions
                .iter()
                .filter(|inst| instruction_is_mixed_string_site(function, inst, mode))
                .count()
        })
        .sum()
}

/// Returns whether an instruction uses either object-aware Mixed string ladder.
pub(super) fn instruction_uses_mixed_string_ladder(
    function: &Function,
    inst: &crate::ir::Instruction,
) -> bool {
    instruction_is_mixed_string_site(function, inst, &MixedStringContextMode::Result)
        || instruction_is_mixed_string_site(function, inst, &MixedStringContextMode::Stdout)
}

/// Returns whether an instruction is a boxed Mixed string site for the requested mode.
fn instruction_is_mixed_string_site(
    function: &Function,
    inst: &crate::ir::Instruction,
    mode: &MixedStringContextMode,
) -> bool {
    let matches_op = match mode {
        MixedStringContextMode::Result => {
            inst.op == crate::ir::Op::Cast
                && inst.immediate == Some(crate::ir::Immediate::CastTarget(IrType::Str))
        }
        MixedStringContextMode::Stdout => inst.op == crate::ir::Op::EchoValue,
    };
    if !matches_op {
        return false;
    }
    inst.operands.first().is_some_and(|operand| {
        function.value(*operand).is_some_and(|value| {
            matches!(
                value.php_type.codegen_repr(),
                PhpType::Mixed | PhpType::Union(_)
            )
        })
    })
}

/// Returns the shared helper label for a call site, or `None` when the ladder stays inline.
pub(super) fn shared_ladder_label(
    ctx: &mut FunctionContext<'_>,
    mode: &MixedStringContextMode,
) -> Option<&'static str> {
    let label = match mode {
        MixedStringContextMode::Result => MIXED_TO_STRING_LABEL,
        MixedStringContextMode::Stdout => MIXED_ECHO_LABEL,
    };
    if ctx.function.name == label {
        return None;
    }
    let module = ctx.module;
    module_shares_mixed_string_ladder(module, mode, ctx.shared).then_some(label)
}

/// Emits the result and stdout helpers that the module's site counts require.
pub(super) fn emit_shared_mixed_string_helpers(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
    regalloc_linear: bool,
) -> Result<()> {
    if module_shares_mixed_string_ladder(module, &MixedStringContextMode::Result, shared) {
        emit_one_helper(
            module,
            emitter,
            data,
            shared,
            regalloc_linear,
            MIXED_TO_STRING_LABEL,
            MixedStringContextMode::Result,
            PhpType::Str,
        )?;
    }
    if module_shares_mixed_string_ladder(module, &MixedStringContextMode::Stdout, shared) {
        emit_one_helper(
            module,
            emitter,
            data,
            shared,
            regalloc_linear,
            MIXED_ECHO_LABEL,
            MixedStringContextMode::Stdout,
            PhpType::Void,
        )?;
    }
    Ok(())
}

/// Emits one helper around the existing object-aware Mixed string dispatch routine.
#[allow(clippy::too_many_arguments)]
fn emit_one_helper(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
    regalloc_linear: bool,
    label: &str,
    mode: MixedStringContextMode,
    return_php_type: PhpType,
) -> Result<()> {
    emit_shared_helper(
        module,
        emitter,
        data,
        shared,
        regalloc_linear,
        label,
        return_php_type,
        &format!("--- shared Mixed string context: {} ---", label),
        |ctx: &mut FunctionContext<'_>| {
            emit_mixed_string_dispatch_from_result(ctx, helper_value(), mode)
        },
    )
}
