//! Purpose:
//! Lowers integer arithmetic, bitwise, shift, and integer-to-float division EIR
//! opcodes for the Phase 04 stack-slot backend.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()`.
//!
//! Key details:
//! - The lowering preserves PHP scalar semantics and keeps all target
//!   register choices behind ABI helpers where shared helpers exist.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::ir::{Immediate, Instruction, MixedNumericOp, ValueId};
use crate::types::PhpType;

use super::super::context::FunctionContext;
use super::{expect_operand, require_integer_like, store_if_result};
use crate::codegen::{CodegenIrError, Result};

/// Lowers a two-operand integer arithmetic or bitwise instruction.
///
/// Operands are read in place from allocated registers, frame slots (x86_64 memory
/// operands), or as immediates for rematerialized constants; the result is computed
/// directly into the destination register when it has one.
pub(super) fn lower_int_binop(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    aarch64_mnemonic: &str,
    x86_64_mnemonic: &str,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let lhs_op = int_operand(ctx, lhs, inst)?;
    let rhs_op = int_operand(ctx, rhs, inst)?;
    let dst_reg = inst.result.and_then(|result| ctx.allocation.register_of(result));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let lhs_reg = aarch64_int_reg(ctx, lhs_op, "x0");
            let rhs_text = match rhs_op {
                IntOperand::Imm(value)
                    if matches!(aarch64_mnemonic, "add" | "sub") && (0..=4095).contains(&value) =>
                {
                    format!("#{}", value)
                }
                other => aarch64_int_reg(ctx, other, "x10").to_string(),
            };
            let out = dst_reg.unwrap_or("x0");
            ctx.emitter.instruction(&format!("{} {}, {}, {}", aarch64_mnemonic, out, lhs_reg, rhs_text)); // compute the integer arithmetic result from both SSA operands
            if dst_reg.is_some() {
                return Ok(());
            }
        }
        Arch::X86_64 => {
            let acc = match dst_reg {
                Some(reg) if rhs_op != IntOperand::Reg(reg) || lhs == rhs => reg,
                _ => "rax",
            };
            let rhs_text = x86_int_operand_text(rhs_op);
            let rhs_is_imm32 = matches!(rhs_op, IntOperand::Imm(value) if i32::try_from(value).is_ok());
            let rhs_text = match rhs_op {
                IntOperand::Imm(value) if !rhs_is_imm32 => {
                    abi::emit_load_int_immediate(ctx.emitter, "r10", value);
                    "r10".to_string()
                }
                _ => rhs_text,
            };
            match lhs_op {
                IntOperand::Reg(reg) if reg == acc => {}
                IntOperand::Imm(value) => abi::emit_load_int_immediate(ctx.emitter, acc, value),
                other => {
                    ctx.emitter.instruction(&format!("mov {}, {}", acc, x86_int_operand_text(other))); // load the left integer operand into the accumulator
                }
            }
            if x86_64_mnemonic == "imul" && rhs_is_imm32 {
                ctx.emitter.instruction(&format!("imul {}, {}, {}", acc, acc, rhs_text)); // multiply the accumulator by the immediate
            } else {
                ctx.emitter.instruction(&format!("{} {}, {}", x86_64_mnemonic, acc, rhs_text)); // apply the integer arithmetic with the right operand
            }
            match dst_reg {
                Some(reg) if reg == acc => return Ok(()),
                Some(reg) => {
                    ctx.emitter.instruction(&format!("mov {}, {}", reg, acc)); // move the integer result into its allocated register
                    return Ok(());
                }
                None => {
                    if acc != "rax" {
                        ctx.emitter.instruction(&format!("mov rax, {}", acc)); // move the integer result to the result register
                    }
                }
            }
        }
    }
    store_if_result(ctx, inst)
}

/// Where an integer operand can be read from without first copying it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum IntOperand {
    /// An allocated general-purpose register.
    Reg(&'static str),
    /// A frame slot at this offset below the frame pointer.
    Slot(usize),
    /// A constant.
    Imm(i64),
}

/// Classifies an integer-like operand (`Void`/`Never` read as zero).
pub(super) fn int_operand(
    ctx: &FunctionContext<'_>,
    value: ValueId,
    inst: &Instruction,
) -> Result<IntOperand> {
    let ty = ctx.value_php_type(value)?;
    if matches!(ty, PhpType::Void | PhpType::Never) {
        return Ok(IntOperand::Imm(0));
    }
    require_integer_like(ty, inst)?;
    if let Some(imm) = ctx.const_int(value) {
        return Ok(IntOperand::Imm(imm));
    }
    if let Some(reg) = ctx.allocation.register_of(value) {
        return Ok(IntOperand::Reg(reg));
    }
    Ok(IntOperand::Slot(ctx.value_frame_offset(value)?))
}

/// Renders an x86_64 operand (register, memory, or immediate text).
pub(super) fn x86_int_operand_text(op: IntOperand) -> String {
    match op {
        IntOperand::Reg(reg) => reg.to_string(),
        IntOperand::Slot(offset) => format!("QWORD PTR [rbp - {}]", offset),
        IntOperand::Imm(value) => value.to_string(),
    }
}

/// Returns an AArch64 register holding the operand, loading it into `scratch` when needed.
pub(super) fn aarch64_int_reg(
    ctx: &mut FunctionContext<'_>,
    op: IntOperand,
    scratch: &'static str,
) -> &'static str {
    match op {
        IntOperand::Reg(reg) => reg,
        IntOperand::Slot(offset) => {
            abi::load_at_offset(ctx.emitter, scratch, offset);
            scratch
        }
        IntOperand::Imm(value) => {
            abi::emit_load_int_immediate(ctx.emitter, scratch, value);
            scratch
        }
    }
}

/// Lowers a checked integer binary operation that may overflow to float.
///
/// Loads both I64 operands into ABI argument registers, calls the target runtime
/// helper (e.g. `__rt_int_add_checked`), and stores the boxed Mixed result.
/// The helper returns a `Heap(Mixed)` pointer in the integer result register.
pub(super) fn lower_int_checked_binop(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    helper: &str,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let lhs_reg = abi::int_result_reg(ctx.emitter);
    let rhs_reg = abi::secondary_scratch_reg(ctx.emitter);
    load_integer_operand(ctx, lhs, lhs_reg, inst)?;
    load_integer_operand(ctx, rhs, rhs_reg, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            // AArch64 ABI: x0 = first arg, x1 = second arg.
            // lhs is already in x0 (int_result_reg), but rhs is in x10 (secondary_scratch_reg).
            // Move rhs to x1 to match the helper's expected calling convention.
            ctx.emitter.instruction("mov x1, x10");                             // place the right integer operand in the ABI argument register x1
            abi::emit_call_label(ctx.emitter, helper);
        }
        Arch::X86_64 => {
            // x86_64 SysV ABI: rdi = first arg, rsi = second arg.
            // Move lhs to rdi, rhs to rsi before the call.
            ctx.emitter.instruction(&format!("mov rdi, {}", lhs_reg));          // place the left integer operand in the first SysV argument register
            ctx.emitter.instruction(&format!("mov rsi, {}", rhs_reg));          // place the right integer operand in the second SysV argument register
            abi::emit_call_label(ctx.emitter, helper);
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers a signed integer modulo operation with the established zero-divisor guard.
pub(super) fn lower_int_mod(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let result_reg = abi::int_result_reg(ctx.emitter);
    let rhs_reg = abi::secondary_scratch_reg(ctx.emitter);
    load_integer_operand(ctx, lhs, result_reg, inst)?;
    load_integer_operand(ctx, rhs, rhs_reg, inst)?;
    let zero_label = ctx.next_label("mod_zero");
    let done_label = ctx.next_label("mod_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let quotient_reg = abi::tertiary_scratch_reg(ctx.emitter);
            ctx.emitter.instruction(&format!("cbz {}, {}", rhs_reg, zero_label)); // branch to zero-divisor guard when modulo divisor is zero
            ctx.emitter.instruction(&format!("sdiv {}, {}, {}", quotient_reg, result_reg, rhs_reg)); // compute signed quotient for the modulo operation
            ctx.emitter.instruction(&format!("msub {}, {}, {}, {}", result_reg, quotient_reg, rhs_reg, result_reg)); // compute left - quotient * right as the remainder
            ctx.emitter.instruction(&format!("b {}", done_label));              // skip the modulo zero fallback after a normal remainder
            ctx.emitter.label(&zero_label);
            ctx.emitter.instruction(&format!("mov {}, #0", result_reg));        // return zero for modulo by zero
            ctx.emitter.label(&done_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("test {}, {}", rhs_reg, rhs_reg)); // test whether the modulo divisor is zero
            ctx.emitter.instruction(&format!("je {}", zero_label));             // branch to zero-divisor guard when modulo divisor is zero
            ctx.emitter.instruction("cqo");                                     // sign-extend the dividend before signed division
            ctx.emitter.instruction(&format!("idiv {}", rhs_reg));              // divide signed integers with quotient in rax and remainder in rdx
            ctx.emitter.instruction(&format!("mov {}, rdx", result_reg));       // move the signed remainder into the integer result register
            ctx.emitter.instruction(&format!("jmp {}", done_label));            // skip the modulo zero fallback after a normal remainder
            ctx.emitter.label(&zero_label);
            ctx.emitter.instruction(&format!("mov {}, 0", result_reg));         // return zero for modulo by zero
            ctx.emitter.label(&done_label);
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers PHP `/` for integer operands by promoting both sides to floating point.
pub(super) fn lower_int_div_to_float(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let lhs_reg = abi::secondary_scratch_reg(ctx.emitter);
    let rhs_reg = abi::tertiary_scratch_reg(ctx.emitter);
    load_integer_operand(ctx, lhs, lhs_reg, inst)?;
    load_integer_operand(ctx, rhs, rhs_reg, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("scvtf d0, {}", lhs_reg));         // promote the integer dividend into the float result register
            ctx.emitter.instruction(&format!("scvtf d1, {}", rhs_reg));         // promote the integer divisor into a float scratch register
            ctx.emitter.instruction("fdiv d0, d0, d1");                         // divide promoted operands as PHP floating-point division
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cvtsi2sd xmm0, {}", lhs_reg));    // promote the integer dividend into the float result register
            ctx.emitter.instruction(&format!("cvtsi2sd xmm1, {}", rhs_reg));    // promote the integer divisor into a float scratch register
            ctx.emitter.instruction("divsd xmm0, xmm1");                        // divide promoted operands as PHP floating-point division
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers a single-operand integer instruction.
pub(super) fn lower_int_unary(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    aarch64_mnemonic: &str,
    x86_64_mnemonic: &str,
) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let result_reg = abi::int_result_reg(ctx.emitter);
    load_integer_operand(ctx, value, result_reg, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("{} {}, {}", aarch64_mnemonic, result_reg, result_reg)); // apply the integer unary operation to the loaded operand
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("{} {}", x86_64_mnemonic, result_reg)); // apply the integer unary operation to the loaded operand
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers a variable-count signed integer shift operation.
pub(super) fn lower_int_shift(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    aarch64_mnemonic: &str,
    x86_64_mnemonic: &str,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let result_reg = abi::int_result_reg(ctx.emitter);
    let rhs_reg = abi::secondary_scratch_reg(ctx.emitter);
    load_integer_operand(ctx, lhs, result_reg, inst)?;
    load_integer_operand(ctx, rhs, rhs_reg, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("{} {}, {}, {}", aarch64_mnemonic, result_reg, result_reg, rhs_reg)); // shift the integer operand by the EIR count operand
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov rcx, {}", rhs_reg));          // move the variable shift count into x86_64's required cl register
            ctx.emitter.instruction(&format!("{} {}, cl", x86_64_mnemonic, result_reg)); // shift the integer operand by the low count byte
        }
    }
    store_if_result(ctx, inst)
}

/// Loads an integer arithmetic operand, coercing PHP null to integer zero.
fn load_integer_operand(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    reg: &str,
    inst: &Instruction,
) -> Result<()> {
    match ctx.value_php_type(value)? {
        PhpType::Void | PhpType::Never => {
            abi::emit_load_int_immediate(ctx.emitter, reg, 0);
            Ok(())
        }
        _ => {
            require_integer_like(ctx.load_value_to_reg(value, reg)?, inst)?;
            Ok(())
        }
    }
}

/// Lowers a dynamic mixed numeric add/sub/mul through the boxed-Mixed runtime helpers.
pub(super) fn lower_mixed_numeric_binop(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let op = expect_mixed_numeric_op(inst)?;
    let lhs_ty = ctx.value_php_type(lhs)?;
    let rhs_ty = ctx.value_php_type(rhs)?;
    let left_box_temp = !is_mixed_like(&lhs_ty);
    let right_box_temp = !is_mixed_like(&rhs_ty);

    materialize_value_as_mixed(ctx, lhs, &lhs_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    materialize_value_as_mixed(ctx, rhs, &rhs_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", 16);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 0);
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rax", 16);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", 0);
        }
    }
    abi::emit_call_label(ctx.emitter, mixed_numeric_helper(op));
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    if left_box_temp {
        decref_mixed_temp_at(ctx, 32);
    }
    if right_box_temp {
        decref_mixed_temp_at(ctx, 16);
    }
    abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    store_if_result(ctx, inst)
}

/// Lowers a runtime-polymorphic bitwise operator (`&`/`|`/`^`) on operands that
/// cannot statically choose PHP's bytewise-string vs integer semantics.
///
/// Both operands are boxed to Mixed and passed to `__rt_mixed_bitwise` together
/// with the operator mode (0=And, 1=Or, 2=Xor). The helper returns a freshly boxed
/// Mixed result (a string when both operands are strings, otherwise an integer),
/// and fatals on a runtime array/object operand. Temporary operand boxes created
/// here for concrete (non-Mixed) operands are released after the call, mirroring
/// `lower_mixed_numeric_binop`.
pub(super) fn lower_mixed_bitwise_op(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let mode = expect_str_bit_op(inst)?.as_mode();
    let lhs_ty = ctx.value_php_type(lhs)?;
    let rhs_ty = ctx.value_php_type(rhs)?;
    let left_box_temp = !is_mixed_like(&lhs_ty);
    let right_box_temp = !is_mixed_like(&rhs_ty);

    materialize_value_as_mixed(ctx, lhs, &lhs_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    materialize_value_as_mixed(ctx, rhs, &rhs_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", 16);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 0);
            abi::emit_load_int_immediate(ctx.emitter, "x2", mode);
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rax", 16);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", 0);
            abi::emit_load_int_immediate(ctx.emitter, "rsi", mode);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_bitwise");
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    if left_box_temp {
        decref_mixed_temp_at(ctx, 32);
    }
    if right_box_temp {
        decref_mixed_temp_at(ctx, 16);
    }
    abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    store_if_result(ctx, inst)
}

/// Lowers PHP's runtime-polymorphic unary bitwise NOT (`~$x`) on an operand that cannot
/// statically choose PHP's bytewise-string vs integer semantics.
///
/// The single operand is boxed to Mixed and passed to `__rt_mixed_bitwise_not`, which returns a
/// freshly boxed Mixed result (a bytewise-NOT string when the payload is a string, otherwise an
/// integer `~i`), and fatals on a runtime array/object operand. A temporary operand box created
/// here for a concrete (non-Mixed) operand is released after the call, mirroring
/// `lower_mixed_bitwise_op`.
pub(super) fn lower_mixed_bitwise_not_op(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let operand = expect_operand(inst, 0)?;
    let operand_ty = ctx.value_php_type(operand)?;
    let box_temp = !is_mixed_like(&operand_ty);

    materialize_value_as_mixed(ctx, operand, &operand_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", 0);
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rax", 0);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_bitwise_not");
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    if box_temp {
        decref_mixed_temp_at(ctx, 16);
    }
    abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    store_if_result(ctx, inst)
}

/// Extracts the `StrBitKind` mode carried by a `MixedBitwise` instruction's immediate.
fn expect_str_bit_op(inst: &Instruction) -> Result<crate::ir::StrBitKind> {
    match inst.immediate {
        Some(Immediate::StrBitOp(kind)) => Ok(kind),
        _ => Err(CodegenIrError::invalid_module(format!(
            "{} missing string bitwise op immediate",
            inst.op.name()
        ))),
    }
}

/// Returns true when a PHP type is already represented as a boxed Mixed pointer.
fn is_mixed_like(ty: &PhpType) -> bool {
    matches!(ty.codegen_repr(), PhpType::Mixed)
}

/// Loads an SSA value as a boxed Mixed pointer in the integer result register.
fn materialize_value_as_mixed(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    ty: &PhpType,
) -> Result<()> {
    let ty = ty.codegen_repr();
    if is_mixed_like(&ty) {
        ctx.load_value_to_result(value)?;
        return Ok(());
    }
    match ty {
        PhpType::Void | PhpType::Never => {
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
        }
        _ => {
            ctx.load_value_to_result(value)?;
        }
    }
    crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &ty);
    Ok(())
}

/// Releases a temporary Mixed box saved on the temporary stack.
fn decref_mixed_temp_at(ctx: &mut FunctionContext<'_>, offset: usize) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", offset);
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rax", offset);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
}

/// Returns the mixed numeric operation immediate attached to the EIR instruction.
fn expect_mixed_numeric_op(inst: &Instruction) -> Result<MixedNumericOp> {
    match inst.immediate {
        Some(Immediate::MixedNumericOp(op)) => Ok(op),
        _ => Err(CodegenIrError::invalid_module(format!(
            "{} missing mixed numeric op immediate",
            inst.op.name()
        ))),
    }
}

/// Maps a mixed numeric operation to the target-aware runtime helper label.
fn mixed_numeric_helper(op: MixedNumericOp) -> &'static str {
    match op {
        MixedNumericOp::Add => "__rt_mixed_numeric_add",
        MixedNumericOp::Sub => "__rt_mixed_numeric_sub",
        MixedNumericOp::Mul => "__rt_mixed_numeric_mul",
    }
}
