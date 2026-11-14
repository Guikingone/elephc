//! Purpose:
//! Lowers floating-point constants, arithmetic, comparisons, and scalar
//! conversions for the Phase 04 EIR backend.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()`.
//!
//! Key details:
//! - Integer `/` lowers elsewhere; this module owns EIR operations whose
//!   storage/result representation is already floating-point.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::ir::{CmpPredicate, Instruction};
use crate::types::PhpType;

use super::super::context::FunctionContext;
use super::{
    expect_f64, expect_operand, expect_cmp_predicate, require_float, secondary_float_reg,
    store_if_result, x86_64_float_condition,
};
use crate::codegen::{CodegenIrError, Result};

/// Lowers a floating-point constant into the canonical float result register and slot.
pub(super) fn lower_const_f64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_f64(inst)?;
    // Load straight into the value's register when it has one (no extra move).
    let target = inst
        .result
        .and_then(|result| ctx.allocation.register_of(result))
        .map(str::to_string);
    let dest = target.clone().unwrap_or_else(|| abi::float_result_reg(ctx.emitter).to_string());
    if value.to_bits() == 0 {
        // +0.0 needs no memory load.
        match ctx.emitter.target.arch {
            Arch::AArch64 => ctx.emitter.instruction(&format!("fmov {}, xzr", dest)), // +0.0
            Arch::X86_64 => ctx.emitter.instruction(&format!("xorpd {}, {}", dest, dest)), // +0.0
        }
    } else {
        let label = ctx.data.add_float(value);
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                let scratch = abi::symbol_scratch_reg(ctx.emitter);
                abi::emit_symbol_address(ctx.emitter, scratch, &label);
                ctx.emitter.instruction(&format!("ldr {}, [{}]", dest, scratch)); // load the 64-bit float literal through the symbol scratch register
            }
            Arch::X86_64 => {
                ctx.emitter.instruction(&format!("movsd {}, QWORD PTR [rip + {}]", dest, label)); // load the 64-bit float literal RIP-relative
            }
        }
    }
    if target.is_some() {
        return Ok(());
    }
    store_if_result(ctx, inst)
}

/// Lowers a floating-point comparison into a boolean result value.
pub(super) fn lower_float_compare(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let predicate = expect_cmp_predicate(inst)?;
    if inst.result.is_some() && ctx.fused_compare_value == inst.result {
        if let Some(()) = try_fused_float_compare(ctx, lhs, rhs, predicate)? {
            return Ok(());
        }
    }
    let lhs_reg = secondary_float_reg(ctx.emitter.target.arch);
    let rhs_reg = abi::float_result_reg(ctx.emitter);
    require_float(ctx.load_value_to_reg(lhs, lhs_reg)?, inst)?;
    require_float(ctx.load_value_to_reg(rhs, rhs_reg)?, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("fcmp d1, d0");                             // compare float operands for the EIR predicate
            ctx.emitter.instruction(&format!("cset x0, {}", aarch64_float_condition(predicate)?)); // materialize the ordered float predicate result as 0 or 1
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("ucomisd xmm1, xmm0");                      // compare float operands for the EIR predicate
            emit_x86_64_float_predicate_result(ctx, predicate)?;
        }
    }
    store_if_result(ctx, inst)
}

/// Emits only the flag-setting compare for an ordering predicate whose result feeds the next
/// `cond_br`, recording a branch condition that is false for unordered (NaN) operands.
///
/// Equality predicates need two flag tests on x86_64 and are left to the materializing path.
fn try_fused_float_compare(
    ctx: &mut FunctionContext<'_>,
    lhs: crate::ir::ValueId,
    rhs: crate::ir::ValueId,
    predicate: CmpPredicate,
) -> Result<Option<()>> {
    let (first, second, x86_cond) = match predicate {
        CmpPredicate::Sgt | CmpPredicate::Ogt => (lhs, rhs, "a"),
        CmpPredicate::Sge | CmpPredicate::Oge => (lhs, rhs, "ae"),
        CmpPredicate::Slt | CmpPredicate::Olt => (rhs, lhs, "a"),
        CmpPredicate::Sle | CmpPredicate::Ole => (rhs, lhs, "ae"),
        CmpPredicate::Eq | CmpPredicate::Ne => return Ok(None),
    };
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let lhs_reg = float_operand_reg(ctx, lhs, "d1")?;
            let rhs_reg = float_operand_reg(ctx, rhs, "d0")?;
            ctx.emitter.instruction(&format!("fcmp {}, {}", lhs_reg, rhs_reg)); // compare float operands; the following cond_br branches on the flags
            ctx.fused_branch_condition = Some(aarch64_float_condition(predicate)?);
        }
        Arch::X86_64 => {
            let first_reg = match ctx.allocation.register_of(first) {
                Some(reg) => reg,
                None => {
                    ctx.load_value_to_reg(first, "xmm1")?;
                    "xmm1"
                }
            };
            let second_operand = x86_float_operand(ctx, second)?;
            ctx.emitter.instruction(&format!("ucomisd {}, {}", first_reg, second_operand)); // compare float operands; the following cond_br branches on the flags
            ctx.fused_branch_condition = Some(x86_cond);
        }
    }
    Ok(Some(()))
}

/// Returns the AArch64 condition for PHP float comparisons, excluding unordered NaN cases.
fn aarch64_float_condition(predicate: CmpPredicate) -> Result<&'static str> {
    match predicate {
        CmpPredicate::Eq => Ok("eq"),
        CmpPredicate::Ne => Ok("ne"),
        CmpPredicate::Slt | CmpPredicate::Olt => Ok("mi"),
        CmpPredicate::Sle | CmpPredicate::Ole => Ok("ls"),
        CmpPredicate::Sgt | CmpPredicate::Ogt => Ok("gt"),
        CmpPredicate::Sge | CmpPredicate::Oge => Ok("ge"),
    }
}

/// Materializes a PHP float comparison from x86_64 flags, treating unordered as false.
fn emit_x86_64_float_predicate_result(
    ctx: &mut FunctionContext<'_>,
    predicate: CmpPredicate,
) -> Result<()> {
    match predicate {
        CmpPredicate::Ne => {
            ctx.emitter.instruction("setne al");                                // materialize ordered float inequality in the low byte
            ctx.emitter.instruction("setp r10b");                               // materialize unordered NaN comparison as true for !=
            ctx.emitter.instruction("or al, r10b");                             // merge ordered inequality with unordered inequality
        }
        predicate => {
            ctx.emitter.instruction(&format!("set{} al", x86_64_float_condition(predicate)?)); // materialize the ordered float predicate in the low byte
            ctx.emitter.instruction("setnp r10b");                              // materialize whether the comparison was ordered
            ctx.emitter.instruction("and al, r10b");                            // clear ordered predicates for unordered NaN comparisons
        }
    }
    ctx.emitter.instruction("movzx rax, al");                                   // widen the predicate byte into the integer result register
    Ok(())
}

/// Lowers a two-operand floating-point arithmetic instruction.
///
/// Operands are read in place from their allocated registers (or, on x86_64, straight from
/// their frame slots), and the result is computed directly into the destination's register
/// when it has one.
pub(super) fn lower_float_binop(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    aarch64_mnemonic: &str,
    x86_64_mnemonic: &str,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    require_float(ctx.value_php_type(lhs)?, inst)?;
    require_float(ctx.value_php_type(rhs)?, inst)?;
    if aarch64_mnemonic == "fdiv" && !super::arithmetic::value_is_nonzero_constant(ctx, rhs) {
        emit_float_zero_divisor_throw(ctx, rhs)?;
    }
    let dst_reg = inst.result.and_then(|result| ctx.allocation.register_of(result));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let lhs_reg = float_operand_reg(ctx, lhs, "d1")?;
            let rhs_reg = float_operand_reg(ctx, rhs, "d0")?;
            let out = dst_reg.unwrap_or("d0");
            ctx.emitter.instruction(&format!("{} {}, {}, {}", aarch64_mnemonic, out, lhs_reg, rhs_reg)); // compute the floating-point arithmetic result
            if dst_reg.is_some() {
                return Ok(());
            }
        }
        Arch::X86_64 => {
            let rhs_home = ctx.allocation.register_of(rhs);
            // Accumulate in the destination register unless it is also the right operand.
            let acc = match dst_reg {
                Some(reg) if rhs_home != Some(reg) || lhs == rhs => reg,
                _ => "xmm1",
            };
            let rhs_operand = x86_float_operand(ctx, rhs)?;
            if ctx.allocation.register_of(lhs) != Some(acc) {
                let lhs_operand = x86_float_operand(ctx, lhs)?;
                if lhs_operand.starts_with("xmm") {
                    ctx.emitter.instruction(&format!("movapd {}, {}", acc, lhs_operand)); // copy the left float operand into the accumulator
                } else {
                    ctx.emitter.instruction(&format!("movsd {}, {}", acc, lhs_operand)); // load the left float operand into the accumulator
                }
            }
            ctx.emitter.instruction(&format!("{} {}, {}", x86_64_mnemonic, acc, rhs_operand)); // apply the float arithmetic with the right operand
            match dst_reg {
                Some(reg) if reg == acc => return Ok(()),
                Some(reg) => {
                    ctx.emitter.instruction(&format!("movapd {}, {}", reg, acc)); // move the float result into its allocated register
                    return Ok(());
                }
                None => {
                    ctx.emitter.instruction(&format!("movapd xmm0, {}", acc)); // move the float result to the result register
                }
            }
        }
    }
    store_if_result(ctx, inst)
}

/// Returns a register holding a float operand on AArch64, loading it into `scratch` when spilled.
fn float_operand_reg(
    ctx: &mut FunctionContext<'_>,
    value: crate::ir::ValueId,
    scratch: &'static str,
) -> Result<&'static str> {
    if let Some(reg) = ctx.allocation.register_of(value) {
        return Ok(reg);
    }
    ctx.load_value_to_reg(value, scratch)?;
    Ok(scratch)
}

/// Returns an x86_64 operand naming a float value: its register or its frame slot.
/// Emits `if ($divisor == 0.0) throw new DivisionByZeroError("Division by zero")` (both signed
/// zeros) without disturbing allocated registers on the normal path.
fn emit_float_zero_divisor_throw(ctx: &mut FunctionContext<'_>, divisor: crate::ir::ValueId) -> Result<()> {
    let ok = ctx.next_label("fdiv_nonzero");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let reg = float_operand_reg(ctx, divisor, "d0")?;
            ctx.emitter.instruction(&format!("fcmp {}, #0.0", reg));            // compare the divisor with zero (-0.0 compares equal)
            ctx.emitter.instruction(&format!("b.ne {}", ok));                   // a non-zero (or NaN) divisor divides normally
        }
        Arch::X86_64 => {
            let operand = x86_float_operand(ctx, divisor)?;
            if operand.starts_with("xmm") {
                ctx.emitter.instruction(&format!("movq r11, {}", operand));     // copy the divisor bits into a scratch register
            } else {
                ctx.emitter.instruction(&format!("mov r11, {}", operand));      // load the divisor bits into a scratch register
            }
            ctx.emitter.instruction("shl r11, 1");                              // drop the sign bit: zero bits mean +0.0 or -0.0
            ctx.emitter.instruction(&format!("jnz {}", ok));                    // a non-zero divisor divides normally
        }
    }
    super::exceptions::emit_division_by_zero_error(ctx, "Division by zero");
    ctx.emitter.label(&ok);
    Ok(())
}

fn x86_float_operand(ctx: &FunctionContext<'_>, value: crate::ir::ValueId) -> Result<String> {
    if let Some(reg) = ctx.allocation.register_of(value) {
        return Ok(reg.to_string());
    }
    let offset = ctx.value_frame_offset(value)?;
    Ok(format!("QWORD PTR [rbp - {}]", offset))
}

/// Lowers floating-point exponentiation through libc `pow`.
pub(super) fn lower_float_pow(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            require_float(ctx.load_value_to_reg(lhs, "d0")?, inst)?;
            require_float(ctx.load_value_to_reg(rhs, "d1")?, inst)?;
            ctx.emitter.bl_c("pow");
        }
        Arch::X86_64 => {
            require_float(ctx.load_value_to_reg(lhs, "xmm0")?, inst)?;
            require_float(ctx.load_value_to_reg(rhs, "xmm1")?, inst)?;
            ctx.emitter.instruction("call pow");                                // compute floating-point exponentiation through libc pow()
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers floating-point negation.
pub(super) fn lower_float_neg(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    require_float(ctx.load_value_to_result(value)?, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("fneg d0, d0");                             // negate the loaded floating-point operand
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("xorpd xmm1, xmm1");                        // materialize a zero float register for subtraction
            ctx.emitter.instruction("subsd xmm1, xmm0");                        // compute 0.0 - operand as floating-point negation
            ctx.emitter.instruction("movapd xmm0, xmm1");                       // move the negated float back to the result register
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers a float-to-int conversion using the target ABI conversion helper.
pub(super) fn lower_float_to_int(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    require_float(ctx.load_value_to_result(value)?, inst)?;
    abi::emit_float_result_to_int_result(ctx.emitter);
    store_if_result(ctx, inst)
}

/// Lowers an integer-like-to-float conversion, treating PHP null as numeric zero.
pub(super) fn lower_int_to_float(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    match ctx.load_value_to_result(value)? {
        PhpType::Int | PhpType::Bool => {}
        PhpType::Void | PhpType::Never => {
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
        }
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "{} for PHP type {:?}",
                inst.op.name(),
                other
            )))
        }
    }
    abi::emit_int_result_to_float_result(ctx.emitter);
    store_if_result(ctx, inst)
}
