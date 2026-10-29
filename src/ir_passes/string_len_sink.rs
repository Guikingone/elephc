//! Purpose:
//! Replaces a `strlen()` of a length-only string producer with its exact length
//! when the produced bytes themselves are never observed.
//!
//! Called from:
//! - The EIR fixed-point pass driver after the integer cast sinks.
//!
//! Key details:
//! - The inner call must receive an already-materialized PHP string, so it cannot throw,
//!   coerce, or invoke user code after its operand has been evaluated.
//! - The inner result may be observed only by one `str_len` and release scaffolding.
//! - `md5()` always returns 32 hex characters; `base64_encode()` returns
//!   `4 * ceil(len / 3)` characters; ASCII case conversion preserves byte length; a
//!   constant `str_repeat()` multiplies a literal source length by its count. The
//!   operand evaluation and its side effects remain.

use crate::ir::{
    DataPool, Function, Immediate, InstId, IrType, Op, Ownership, RuntimeCallTarget,
    RuntimeFnId, UnaryStringRuntime, ValueId,
};
use crate::types::PhpType;

use super::driver::IrPass;
use super::rewrite::{defining_instruction, neutralize_to_nop};

/// Largest folded repeat length; larger results keep the real allocation path.
const MAX_FOLDED_REPEAT: usize = 16 * 1024 * 1024;

/// Rewrites length-only hash and base64 calls.
pub struct StringLenSink;

impl IrPass for StringLenSink {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "string_len_sink"
    }

    /// Skips functions with no length-only string producer call.
    fn is_applicable(&self, function: &Function) -> bool {
        function.instructions.iter().any(is_length_observable_call)
    }

    /// Specializes every independently proven call.
    fn run(&self, function: &mut Function, data: &mut DataPool) -> bool {
        let candidates: Vec<InstId> = function
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, inst)| is_length_observable_call(inst))
            .map(|(raw, _)| InstId::from_raw(raw as u32))
            .collect();
        let mut changed = false;
        for candidate in candidates {
            if specialize(function, candidate, data) {
                changed = true;
            }
        }
        changed
    }
}

/// Returns whether `inst` is a length-only string producer call.
fn is_length_observable_call(inst: &crate::ir::Instruction) -> bool {
    if inst.op != Op::RuntimeCall
        || inst.result.is_none()
        || inst.result_php_type.codegen_repr() != PhpType::Str
    {
        return false;
    }
    match inst.immediate {
        Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
            RuntimeFnId::Md5,
        )))
        | Some(Immediate::RuntimeCall(RuntimeCallTarget::UnaryString(
            UnaryStringRuntime::Base64Encode
                | UnaryStringRuntime::StrToLower
                | UnaryStringRuntime::StrToUpper,
        ))) => inst.operands.len() == 1,
        Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
            RuntimeFnId::StrRepeat,
        ))) => inst.operands.len() == 2,
        _ => false,
    }
}

/// Specializes one candidate after proving its operand and complete use graph.
fn specialize(function: &mut Function, candidate: InstId, data: &DataPool) -> bool {
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
    let is_case_conversion = matches!(
        call.immediate,
        Some(Immediate::RuntimeCall(RuntimeCallTarget::UnaryString(
            UnaryStringRuntime::StrToLower | UnaryStringRuntime::StrToUpper
        )))
    );
    let is_repeat = matches!(
        call.immediate,
        Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
            RuntimeFnId::StrRepeat
        )))
    );
    let repeat_length = if is_repeat {
        let Some(&count_operand) = call.operands.get(1) else {
            return false;
        };
        let Some(source) = const_str_literal(function, data, operand) else {
            return false;
        };
        let Some(count) = const_i64(function, count_operand) else {
            return false;
        };
        if count < 0 {
            return false;
        }
        let Some(length) = source.len().checked_mul(count as usize) else {
            return false;
        };
        if length > MAX_FOLDED_REPEAT {
            return false;
        }
        Some(length as i64)
    } else {
        None
    };
    let Some((length, releases)) = analyze_uses(function, result) else {
        return false;
    };

    // Rewrite the length observer in place, then retire the unobserved producer.
    if is_md5 || is_repeat {
        let Some(length) = function.instruction_mut(length) else {
            return false;
        };
        length.op = Op::ConstI64;
        length.operands.clear();
        length.immediate = Some(Immediate::I64(if is_md5 {
            32
        } else {
            repeat_length.expect("repeat length proven")
        }));
        length.result_type = IrType::I64;
        length.result_php_type = PhpType::Int;
        length.result_ownership = Ownership::NonHeap;
        length.effects = Op::ConstI64.default_effects();
    } else if is_case_conversion {
        let Some(length) = function.instruction_mut(length) else {
            return false;
        };
        length.op = Op::StrLen;
        length.operands = vec![operand];
        length.immediate = None;
        length.result_type = IrType::I64;
        length.result_php_type = PhpType::Int;
        length.result_ownership = Ownership::NonHeap;
        length.effects = Op::StrLen.default_effects();
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

/// Returns the literal data behind a `ConstStr` value, when available.
fn const_str_literal<'a>(function: &Function, data: &'a DataPool, value: ValueId) -> Option<&'a str> {
    let inst = defining_instruction(function, value)?;
    if inst.op != Op::ConstStr {
        return None;
    }
    let Some(Immediate::Data(id)) = inst.immediate else {
        return None;
    };
    data.strings.get(id.as_raw() as usize).map(String::as_str)
}

/// Returns the integer literal behind a value, when available.
fn const_i64(function: &Function, value: ValueId) -> Option<i64> {
    let inst = defining_instruction(function, value)?;
    if inst.op != Op::ConstI64 {
        return None;
    }
    match inst.immediate {
        Some(Immediate::I64(value)) => Some(value),
        _ => None,
    }
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
