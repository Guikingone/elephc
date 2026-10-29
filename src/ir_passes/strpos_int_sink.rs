//! Purpose:
//! Specializes a boxed `strpos()`-family search whose only observation is an `(int)` cast
//! into the raw integer search result.
//!
//! Called from:
//! - The EIR fixed-point pass driver after the length and math sinks.
//!
//! Key details:
//! - Applies to `strpos`, `stripos`, `strrpos` and `strripos`; the search still runs with
//!   its original arguments, coercions, offset validation and warnings. Only the result
//!   representation changes from `int|false` to `int`.
//! - A not-found sentinel becomes zero, exactly like PHP's `(int)false`.
//! - Any observer beyond one `Cast` to `I64` plus release scaffolding keeps the boxed call.

use crate::ir::{DataPool, Function, Immediate, InstId, IrType, Op, Ownership, RuntimeCallTarget, RuntimeFnId};
use crate::types::PhpType;

use super::array_sum_int_sink::analyze_uses;
use super::driver::IrPass;
use super::rewrite::{neutralize_to_nop, replace_all_uses};

/// Rewrites provably int-observed `strpos()`-family calls to the raw integer search.
pub struct StrposIntSink;

impl IrPass for StrposIntSink {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "strpos_int_sink"
    }

    /// Skips functions with no boxed `strpos()`-family runtime call.
    fn is_applicable(&self, function: &Function) -> bool {
        function.instructions.iter().any(is_boxed_position)
    }

    /// Specializes every independently proven call.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        let candidates: Vec<InstId> = function
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, inst)| is_boxed_position(inst))
            .map(|(raw, _)| InstId::from_raw(raw as u32))
            .collect();
        let mut changed = false;
        for candidate in candidates {
            if specialize(function, candidate) {
                changed = true;
            }
        }
        changed
    }
}

/// Returns whether `inst` is a two/three-operand boxed position search.
fn is_boxed_position(inst: &crate::ir::Instruction) -> bool {
    inst.op == Op::RuntimeCall
        && matches!(
            inst.immediate,
            Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                RuntimeFnId::Stripos | RuntimeFnId::Strpos | RuntimeFnId::Strripos | RuntimeFnId::Strrpos
            )))
        )
        && matches!(inst.operands.len(), 2 | 3)
        && inst.result.is_some()
        && inst.result_php_type.codegen_repr() == PhpType::Mixed
}

/// Returns the raw integer runtime function for a boxed position search.
fn raw_variant(target: RuntimeFnId) -> Option<RuntimeFnId> {
    match target {
        RuntimeFnId::Stripos => Some(RuntimeFnId::StriposInt),
        RuntimeFnId::Strpos => Some(RuntimeFnId::StrposInt),
        RuntimeFnId::Strripos => Some(RuntimeFnId::StrriposInt),
        RuntimeFnId::Strrpos => Some(RuntimeFnId::StrrposInt),
        _ => None,
    }
}

/// Specializes one candidate after proving its complete use graph.
fn specialize(function: &mut Function, candidate: InstId) -> bool {
    let Some(call) = function.instruction(candidate) else {
        return false;
    };
    let Some(result) = call.result else {
        return false;
    };
    let Some((cast, cast_result, releases)) = analyze_uses(function, result) else {
        return false;
    };

    let Some(call) = function.instruction_mut(candidate) else {
        return false;
    };
    let Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(current))) = call.immediate else {
        return false;
    };
    let Some(raw) = raw_variant(current) else {
        return false;
    };
    call.immediate = Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(raw)));
    call.result_type = IrType::I64;
    call.result_php_type = PhpType::Int;
    call.result_ownership = Ownership::NonHeap;
    call.effects = raw.intrinsic_effects();
    if let Some(value) = function.value_mut(result) {
        value.ir_type = IrType::I64;
        value.php_type = PhpType::Int;
        value.ownership = Ownership::NonHeap;
    }
    replace_all_uses(
        function,
        &std::collections::HashMap::from([(cast_result, result)]),
    );
    if let Some(inst) = function.instruction_mut(cast) {
        neutralize_to_nop(inst);
    }
    for release in releases {
        if let Some(inst) = function.instruction_mut(release) {
            neutralize_to_nop(inst);
        }
    }
    true
}
