//! Purpose:
//! Spills and reloads descriptor entry arguments across target ABIs.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;

/// Converts a descriptor overflow offset into a caller-stack frame offset.
pub(super) fn descriptor_entry_caller_stack_offset(
    emitter: &crate::codegen::emit::Emitter,
    stack_offset: usize,
) -> usize {
    let cursor = abi::IncomingArgCursor::for_target(emitter.target, 0);
    cursor.caller_stack_offset + stack_offset
}

/// Returns integer scratch registers that cannot overlap live descriptor argument registers.
pub(super) fn descriptor_entry_int_spill_pair(
    emitter: &crate::codegen::emit::Emitter,
) -> (&'static str, &'static str) {
    let lo_reg = abi::secondary_scratch_reg(emitter);
    let hi_reg = match emitter.target.arch {
        Arch::AArch64 => abi::tertiary_scratch_reg(emitter),
        Arch::X86_64 => "r11",
    };
    (lo_reg, hi_reg)
}

/// Stores one incoming descriptor entry argument into its spill slot.
pub(super) fn store_descriptor_entry_incoming_arg(
    emitter: &mut crate::codegen::emit::Emitter,
    ty: &PhpType,
    assignment: &abi::OutgoingArgAssignment,
    offset: usize,
    stack_offset: Option<usize>,
) {
    match ty.codegen_repr() {
        PhpType::Float => {
            let reg = if assignment.in_register() {
                abi::float_arg_reg_name(emitter.target, assignment.start_reg)
            } else {
                let caller_offset = descriptor_entry_caller_stack_offset(
                    emitter,
                    stack_offset.expect("stack offset"),
                );
                let spill_reg = abi::float_spill_scratch_reg(emitter.target);
                abi::load_from_caller_stack(emitter, spill_reg, caller_offset);
                spill_reg
            };
            abi::store_at_offset(emitter, reg, offset);
        }
        PhpType::Str => {
            let (ptr_reg, len_reg) = if assignment.in_register() {
                (
                    abi::int_arg_reg_name(emitter.target, assignment.start_reg),
                    abi::int_arg_reg_name(emitter.target, assignment.start_reg + 1),
                )
            } else {
                let caller_offset = descriptor_entry_caller_stack_offset(
                    emitter,
                    stack_offset.expect("stack offset"),
                );
                let (ptr_spill_reg, len_spill_reg) = descriptor_entry_int_spill_pair(emitter);
                abi::load_from_caller_stack(emitter, ptr_spill_reg, caller_offset);
                abi::load_from_caller_stack(emitter, len_spill_reg, caller_offset + 8);
                (ptr_spill_reg, len_spill_reg)
            };
            abi::store_at_offset(emitter, ptr_reg, offset);
            abi::store_at_offset(emitter, len_reg, offset - 8);
        }
        PhpType::TaggedScalar => {
            let (payload_reg, tag_reg) = if assignment.in_register() {
                (
                    abi::int_arg_reg_name(emitter.target, assignment.start_reg),
                    abi::int_arg_reg_name(emitter.target, assignment.start_reg + 1),
                )
            } else {
                let caller_offset = descriptor_entry_caller_stack_offset(
                    emitter,
                    stack_offset.expect("stack offset"),
                );
                let (payload_spill_reg, tag_spill_reg) = descriptor_entry_int_spill_pair(emitter);
                abi::load_from_caller_stack(emitter, payload_spill_reg, caller_offset);
                abi::load_from_caller_stack(emitter, tag_spill_reg, caller_offset + 8);
                (payload_spill_reg, tag_spill_reg)
            };
            abi::store_at_offset(emitter, payload_reg, offset);
            abi::store_at_offset(emitter, tag_reg, offset - 8);
        }
        PhpType::Void | PhpType::Never => {}
        _ => {
            let reg = if assignment.in_register() {
                abi::int_arg_reg_name(emitter.target, assignment.start_reg)
            } else {
                let caller_offset = descriptor_entry_caller_stack_offset(
                    emitter,
                    stack_offset.expect("stack offset"),
                );
                let spill_reg = abi::secondary_scratch_reg(emitter);
                abi::load_from_caller_stack(emitter, spill_reg, caller_offset);
                spill_reg
            };
            abi::store_at_offset(emitter, reg, offset);
        }
    }
}

/// Loads one spilled descriptor entry argument into its real method ABI assignment.
pub(super) fn load_descriptor_entry_actual_arg(
    emitter: &mut crate::codegen::emit::Emitter,
    ty: &PhpType,
    assignment: &abi::OutgoingArgAssignment,
    offset: usize,
    stack_offset: Option<usize>,
) {
    match ty.codegen_repr() {
        PhpType::Float => {
            let reg = if assignment.in_register() {
                abi::float_arg_reg_name(emitter.target, assignment.start_reg)
            } else {
                abi::float_spill_scratch_reg(emitter.target)
            };
            abi::load_at_offset(emitter, reg, offset);
            if let Some(out_offset) = stack_offset {
                abi::emit_store_to_sp(emitter, reg, out_offset);
            }
        }
        PhpType::Str => {
            let (ptr_reg, len_reg) = if assignment.in_register() {
                (
                    abi::int_arg_reg_name(emitter.target, assignment.start_reg),
                    abi::int_arg_reg_name(emitter.target, assignment.start_reg + 1),
                )
            } else {
                descriptor_entry_int_spill_pair(emitter)
            };
            abi::load_at_offset(emitter, ptr_reg, offset);
            abi::load_at_offset(emitter, len_reg, offset - 8);
            if let Some(out_offset) = stack_offset {
                abi::emit_store_to_sp(emitter, ptr_reg, out_offset);
                abi::emit_store_to_sp(emitter, len_reg, out_offset + 8);
            }
        }
        PhpType::TaggedScalar => {
            let (payload_reg, tag_reg) = if assignment.in_register() {
                (
                    abi::int_arg_reg_name(emitter.target, assignment.start_reg),
                    abi::int_arg_reg_name(emitter.target, assignment.start_reg + 1),
                )
            } else {
                descriptor_entry_int_spill_pair(emitter)
            };
            abi::load_at_offset(emitter, payload_reg, offset);
            abi::load_at_offset(emitter, tag_reg, offset - 8);
            if let Some(out_offset) = stack_offset {
                abi::emit_store_to_sp(emitter, payload_reg, out_offset);
                abi::emit_store_to_sp(emitter, tag_reg, out_offset + 8);
            }
        }
        PhpType::Void | PhpType::Never => {}
        _ => {
            let reg = if assignment.in_register() {
                abi::int_arg_reg_name(emitter.target, assignment.start_reg)
            } else {
                abi::secondary_scratch_reg(emitter)
            };
            abi::load_at_offset(emitter, reg, offset);
            if let Some(out_offset) = stack_offset {
                abi::emit_store_to_sp(emitter, reg, out_offset);
            }
        }
    }
}

/// Rounds `value` up to a 16-byte multiple.
pub(super) fn align16(value: usize) -> usize {
    (value + 15) & !15
}

