//! Purpose:
//! Replaces a `strlen(md5($s))` or `strlen(base64_encode($s))` call with its exact
//! length when the encoded bytes themselves are never observed.
//!
//! Called from:
//! - The EIR fixed-point pass driver after the integer cast sinks.
//!
//! Key details:
//! - The inner call must receive an already-materialized PHP string, so it cannot throw,
//!   coerce, or invoke user code after its operand has been evaluated.
//! - The inner result may be observed only by one `str_len` and release scaffolding.
//! - `md5()` always returns 32 hex characters; `base64_encode()` returns
//!   `4 * ceil(len / 3)` characters. The operand evaluation and its side effects remain.

use crate::ir::{
    DataPool, Function, Immediate, InstId, IrType, Op, Ownership, RuntimeCallTarget,
    RuntimeFnId, UnaryStringRuntime, ValueId,
};
use crate::types::PhpType;

use super::driver::IrPass;
use super::rewrite::neutralize_to_nop;

/// Rewrites length-only hash and base64 calls.
pub struct StringLenSink;

impl IrPass for StringLenSink {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "string_len_sink"
    }

    /// Skips functions with no md5/base64 runtime call.
    fn is_applicable(&self, function: &Function) -> bool {
        function.instructions.iter().any(is_length_observable_call)
    }

    /// Specializes every independently proven call.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        let candidates: Vec<InstId> = function
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, inst)| is_length_observable_call(inst))
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

/// Returns whether `inst` is a one-operand md5/base64 call returning a string.
fn is_length_observable_call(inst: &crate::ir::Instruction) -> bool {
    inst.op == Op::RuntimeCall
        && inst.operands.len() == 1
        && inst.result.is_some()
        && inst.result_php_type.codegen_repr() == PhpType::Str
        && matches!(
            inst.immediate,
            Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                RuntimeFnId::Md5
            ))) | Some(Immediate::RuntimeCall(RuntimeCallTarget::UnaryString(
                UnaryStringRuntime::Base64Encode
            )))
        )
}

/// Specializes one candidate after proving its operand and complete use graph.
fn specialize(function: &mut Function, candidate: InstId) -> bool {
    let Some(call) = function.instruction(candidate) else {
        return false;
    };
    let Some(result) = call.result else {
        return false;
    };
    let Some(&operand) = call.operands.first() else {
        return false;
    };
    if function
        .value(operand)
        .is_none_or(|value| value.php_type.codegen_repr() != PhpType::Str)
    {
        return false;
    }
    let is_md5 = matches!(
        call.immediate,
        Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
            RuntimeFnId::Md5
        )))
    );
    let Some((length, releases)) = analyze_uses(function, result) else {
        return false;
    };

    // Rewrite the length observer in place, then retire the unobserved producer.
    if is_md5 {
        let Some(length) = function.instruction_mut(length) else {
            return false;
        };
        length.op = Op::ConstI64;
        length.operands.clear();
        length.immediate = Some(Immediate::I64(32));
        length.result_type = IrType::I64;
        length.result_php_type = PhpType::Int;
        length.result_ownership = Ownership::NonHeap;
        length.effects = Op::ConstI64.default_effects();
    } else {
        let Some(length) = function.instruction_mut(length) else {
            return false;
        };
        length.op = Op::RuntimeCall;
        length.operands = vec![operand];
        length.immediate = Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
            RuntimeFnId::Base64EncodedLen,
        )));
        length.result_type = IrType::I64;
        length.result_php_type = PhpType::Int;
        length.result_ownership = Ownership::NonHeap;
        length.effects = RuntimeFnId::Base64EncodedLen.intrinsic_effects();
    }
    if let Some(inst) = function.instruction_mut(candidate) {
        neutralize_to_nop(inst);
    }
    for release in releases {
        if let Some(inst) = function.instruction_mut(release) {
            neutralize_to_nop(inst);
        }
    }
    true
}

/// Returns the single `str_len` observer and removable releases of `result`.
fn analyze_uses(function: &Function, result: ValueId) -> Option<(InstId, Vec<InstId>)> {
    let mut length = None;
    let mut releases = Vec::new();
    for (raw, inst) in function.instructions.iter().enumerate() {
        if !inst.operands.contains(&result) {
            continue;
        }
        let id = InstId::from_raw(raw as u32);
        match inst.op {
            Op::StrLen if inst.operands.as_slice() == [result] && inst.result.is_some() => {
                if length.replace(id).is_some() {
                    return None;
                }
            }
            Op::Release if inst.operands.as_slice() == [result] => releases.push(id),
            _ => return None,
        }
    }
    for block in &function.blocks {
        if block
            .terminator
            .as_ref()
            .is_some_and(|term| term.used_values().contains(&result))
        {
            return None;
        }
    }
    Some((length?, releases))
}
