//! Purpose:
//! Folds `count(json_decode($s, true))` into a count-only runtime helper when the decoded
//! Mixed graph has no other observer.
//!
//! Called from:
//! - The EIR fixed-point pass driver after the JSON length sink.
//!
//! Key details:
//! - The decode must have exactly two operands and an associative flag constant `true`.
//! - The decode result may be observed only by one `count()`, one store into a temporary
//!   slot, one release, and that slot's owner/unset scaffolding.
//! - The helper keeps full JSON validation and falls back to the exact decoder for numeric
//!   roots and escaped/numeric object keys, so `count()` TypeErrors stay php-src exact.

use crate::ir::{
    DataPool, Function, Immediate, InstId, IrType, Op, Ownership, RuntimeCallTarget, RuntimeFnId,
    ValueDef, ValueId,
};
use crate::types::PhpType;

use super::driver::IrPass;
use super::rewrite::neutralize_to_nop;

/// Rewrites the exact count-of-decode graph to the count-only helper.
pub struct JsonDecodeCountSink;

impl IrPass for JsonDecodeCountSink {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "json_decode_count_sink"
    }

    /// Skips functions with no one-operand `count()` call.
    fn is_applicable(&self, function: &Function) -> bool {
        function.instructions.iter().any(is_count_call)
    }

    /// Specializes every independently proven call.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        let candidates: Vec<InstId> = function
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, inst)| is_count_call(inst))
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

/// Returns whether the decode source stays alive until the count-only call.
///
/// A temporary source is released between the original decode and count calls; reusing it as
/// the count-only operand would read freed storage. Locals, borrowed cell payloads and
/// persistent literals have no such release and remain valid through the call.
fn source_stays_alive(function: &Function, source: ValueId) -> bool {
    let Some(value) = function.value(source) else {
        return false;
    };
    if !matches!(
        value.ownership,
        Ownership::Borrowed | Ownership::Persistent | Ownership::MaybeOwned
    ) {
        return false;
    }
    !function
        .instructions
        .iter()
        .any(|inst| inst.op == Op::Release && inst.operands.as_slice() == [source])
}

/// Returns whether `inst` is a one-operand `count()` call returning int.
fn is_count_call(inst: &crate::ir::Instruction) -> bool {
    inst.op == Op::RuntimeCall
        && matches!(
            inst.immediate,
            Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                RuntimeFnId::Count
            )))
        )
        && inst.operands.len() == 1
        && inst.result.is_some()
        && inst.result_php_type.codegen_repr() == PhpType::Int
}

/// Returns the defining instruction of a value, when it is instruction-defined.
fn defining_instruction(function: &Function, value: ValueId) -> Option<(InstId, &crate::ir::Instruction)> {
    let ValueDef::Instruction { inst, .. } = function.value(value)?.def else {
        return None;
    };
    Some((inst, function.instruction(inst)?))
}

/// Returns the decode call and its source operand when it is the exact two-argument assoc form.
fn decode_source(function: &Function, value: ValueId) -> Option<(InstId, ValueId)> {
    let (id, decode) = defining_instruction(function, value)?;
    if decode.op != Op::RuntimeCall
        || !matches!(
            decode.immediate,
            Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                RuntimeFnId::JsonDecode
            )))
        )
        || decode.operands.len() != 2
        || decode.result != Some(value)
        || decode.result_php_type.codegen_repr() != PhpType::Mixed
    {
        return None;
    }
    let assoc = *decode.operands.get(1)?;
    let (_, assoc_inst) = defining_instruction(function, assoc)?;
    if assoc_inst.op != Op::ConstBool || assoc_inst.immediate != Some(Immediate::Bool(true)) {
        return None;
    }
    Some((id, *decode.operands.first()?))
}

/// Returns the temporary slot and the removable decode-result scaffolding.
fn analyze_decode_uses(
    function: &Function,
    decode_result: ValueId,
    count_call: InstId,
) -> Option<(u32, Vec<InstId>)> {
    let mut store = None;
    let mut releases = Vec::new();
    for (raw, inst) in function.instructions.iter().enumerate() {
        if !inst.operands.contains(&decode_result) {
            continue;
        }
        let id = InstId::from_raw(raw as u32);
        if id == count_call {
            continue;
        }
        match inst.op {
            Op::StoreLocal
                if inst.operands.as_slice() == [decode_result]
                    && matches!(inst.immediate, Some(Immediate::LocalSlot(_))) =>
            {
                let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
                    return None;
                };
                if store.replace((slot.as_raw(), id)).is_some() {
                    return None;
                }
            }
            Op::Release if inst.operands.as_slice() == [decode_result] => releases.push(id),
            _ => return None,
        }
    }
    for block in &function.blocks {
        if block
            .terminator
            .as_ref()
            .is_some_and(|term| term.used_values().contains(&decode_result))
        {
            return None;
        }
    }
    let (slot, store) = store?;
    if releases.len() != 1 {
        return None;
    }
    releases.push(store);
    Some((slot, releases))
}

/// Returns the removable owner/unset scaffolding of the decode result's temporary slot.
fn analyze_slot_uses(function: &Function, slot_raw: u32) -> Option<Vec<InstId>> {
    let mut removable = Vec::new();
    for (raw, inst) in function.instructions.iter().enumerate() {
        let id = InstId::from_raw(raw as u32);
        let uses_slot = matches!(
            inst.immediate,
            Some(Immediate::LocalSlot(slot)) if slot.as_raw() == slot_raw
        );
        if !uses_slot {
            continue;
        }
        match inst.op {
            Op::StoreLocal => {}
            Op::PushCallOperandOwner | Op::PopCallOperandOwner | Op::UnsetLocal => {
                removable.push(id);
            }
            // The only store into this slot is the removed decode result, so the pre-store
            // retirement would release uninitialized frame bits; remove it with the store.
            Op::ReleaseLocalSlot => {
                removable.push(id);
            }
            _ => return None,
        }
    }
    Some(removable)
}

/// Specializes one count-of-decode call after proving the complete use graph.
fn specialize(function: &mut Function, candidate: InstId) -> bool {
    let Some(count) = function.instruction(candidate) else {
        return false;
    };
    let Some(count_result) = count.result else {
        return false;
    };
    let Some(&decoded) = count.operands.first() else {
        return false;
    };
    let Some((decode_call, source)) = decode_source(function, decoded) else {
        return false;
    };
    if !source_stays_alive(function, source) {
        return false;
    }
    let Some((slot, mut removable)) = analyze_decode_uses(function, decoded, candidate) else {
        return false;
    };
    let Some(mut slot_scaffolding) = analyze_slot_uses(function, slot) else {
        return false;
    };
    removable.append(&mut slot_scaffolding);

    let assoc = *function
        .instruction(decode_call)
        .and_then(|decode| decode.operands.get(1))
        .expect("checked decode operand");
    let Some(count) = function.instruction_mut(candidate) else {
        return false;
    };
    count.operands = vec![source, assoc];
    count.immediate = Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
        RuntimeFnId::JsonDecodeCount,
    )));
    count.result_type = IrType::I64;
    count.result_php_type = PhpType::Int;
    count.result_ownership = Ownership::NonHeap;
    count.effects = RuntimeFnId::JsonDecodeCount.intrinsic_effects();
    if let Some(value) = function.value_mut(count_result) {
        value.ir_type = IrType::I64;
        value.php_type = PhpType::Int;
        value.ownership = Ownership::NonHeap;
    }
    if let Some(inst) = function.instruction_mut(decode_call) {
        neutralize_to_nop(inst);
    }
    for id in removable {
        if let Some(inst) = function.instruction_mut(id) {
            neutralize_to_nop(inst);
        }
    }
    true
}
