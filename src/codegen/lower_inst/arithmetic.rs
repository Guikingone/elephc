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
    // An add whose overflow a fused compare tests leaves its flags for that compare (x86_64
    // `add` always sets OF; AArch64 needs the flag-setting `adds`).
    let flags_for = (inst.op == crate::ir::Op::IAdd)
        .then_some(inst.result)
        .flatten()
        .filter(|result| ctx.overflow_sums.contains(result));
    let aarch64_mnemonic = if flags_for.is_some() && aarch64_mnemonic == "add" {
        "adds"
    } else {
        aarch64_mnemonic
    };
    let result = lower_int_binop_inner(ctx, inst, lhs, rhs, lhs_op, rhs_op, dst_reg, aarch64_mnemonic, x86_64_mnemonic);
    if flags_for.is_some() && result.is_ok() {
        ctx.add_flags_for = flags_for;
    }
    result
}

/// Emits a two-operand integer op once its operands are classified (see `lower_int_binop`).
#[allow(clippy::too_many_arguments)]
fn lower_int_binop_inner(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    lhs: ValueId,
    rhs: ValueId,
    lhs_op: IntOperand,
    rhs_op: IntOperand,
    dst_reg: Option<&'static str>,
    aarch64_mnemonic: &str,
    x86_64_mnemonic: &str,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let lhs_reg = aarch64_int_reg(ctx, lhs_op, "x0");
            let rhs_text = match rhs_op {
                IntOperand::Imm(value)
                    if matches!(aarch64_mnemonic, "add" | "adds" | "sub") && (0..=4095).contains(&value) =>
                {
                    format!("#{}", value)
                }
                IntOperand::Imm(value)
                    if matches!(aarch64_mnemonic, "and" | "orr" | "eor")
                        && is_aarch64_logical_immediate(value as u64) =>
                {
                    format!("#0x{:x}", value as u64)
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
            // A commutative op whose destination already holds the right operand folds the left
            // operand into it directly (`add r8, rdi` rather than going through rax).
            let commutative = matches!(x86_64_mnemonic, "add" | "and" | "or" | "xor" | "imul");
            if let (Some(reg), true, true) = (dst_reg, commutative, lhs != rhs) {
                let lhs_fits = match lhs_op {
                    IntOperand::Reg(_) => true,
                    IntOperand::Imm(value) => i32::try_from(value).is_ok(),
                    _ => false,
                };
                if rhs_op == IntOperand::Reg(reg) && lhs_fits {
                    let lhs_text = x86_int_operand_text(lhs_op);
                    if x86_64_mnemonic == "imul" && matches!(lhs_op, IntOperand::Imm(_)) {
                        ctx.emitter.instruction(&format!("imul {}, {}, {}", reg, reg, lhs_text)); // multiply the destination by the left immediate
                    } else {
                        ctx.emitter.instruction(&format!("{} {}, {}", x86_64_mnemonic, reg, lhs_text)); // fold the left operand into the destination holding the right one
                    }
                    return Ok(());
                }
            }
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
            if x86_64_mnemonic == "imul" && rhs_is_imm32 && !matches!(lhs_op, IntOperand::Imm(_)) {
                // The three-operand form reads the left operand in place (register or frame slot).
                ctx.emitter.instruction(&format!("imul {}, {}, {}", acc, x86_int_operand_text(lhs_op), rhs_text)); // multiply the left operand by the immediate straight into the accumulator
            } else {
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

/// Returns true when `value` is encodable as an AArch64 64-bit logical (bitmask) immediate:
/// a rotated run of ones replicated across an element size of 2..=64 bits. All-zeros and
/// all-ones are not encodable.
pub(super) fn is_aarch64_logical_immediate(value: u64) -> bool {
    if value == 0 || value == u64::MAX {
        return false;
    }
    let mut size = 64u32;
    // Shrink to the smallest element size whose replication reproduces the value.
    while size > 2 {
        let half = size / 2;
        let mask = (1u64 << half) - 1;
        if (value & mask) != ((value >> half) & mask) {
            break;
        }
        size = half;
    }
    let mask = if size == 64 { u64::MAX } else { (1u64 << size) - 1 };
    let element = value & mask;
    // The element must be a rotation of a contiguous run of ones: rotate so the run starts
    // at bit 0, then it must look like 0b0..01..1.
    let rotate_right = |v: u64, r: u32| -> u64 {
        if r == 0 {
            v
        } else {
            ((v >> r) | (v << (size - r))) & mask
        }
    };
    (0..size).any(|r| {
        let rotated = rotate_right(element, r);
        rotated != 0 && rotated != mask && (rotated & (rotated + 1)) == 0
    })
}

/// Lowers `imul_overflow`: 1 when the signed 64-bit product overflows, else 0. Touches only
/// the result and scratch registers (it is volatile-safe).
pub(super) fn lower_int_mul_overflow(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let lhs_op = int_operand(ctx, lhs, inst)?;
    let rhs_op = int_operand(ctx, rhs, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let rhs_reg = aarch64_int_reg(ctx, rhs_op, "x10");
            let lhs_reg = aarch64_int_reg(ctx, lhs_op, "x9");
            ctx.emitter.instruction(&format!("mul x11, {}, {}", lhs_reg, rhs_reg)); // low 64 bits of the product
            ctx.emitter.instruction(&format!("smulh x9, {}, {}", lhs_reg, rhs_reg)); // high 64 bits of the product
            ctx.emitter.instruction("cmp x9, x11, asr #63");                    // the product fits when the high word is the low word's sign
            ctx.emitter.instruction("cset x0, ne");                             // 1 when the product overflowed
        }
        Arch::X86_64 => {
            match lhs_op {
                IntOperand::Imm(value) => abi::emit_load_int_immediate(ctx.emitter, "rax", value),
                other => {
                    ctx.emitter.instruction(&format!("mov rax, {}", x86_int_operand_text(other))); // left factor
                }
            }
            let rhs_text = match rhs_op {
                IntOperand::Imm(value) => {
                    abi::emit_load_int_immediate(ctx.emitter, "r10", value);
                    "r10".to_string()
                }
                other => x86_int_operand_text(other),
            };
            ctx.emitter.instruction(&format!("imul rax, {}", rhs_text));        // signed product, OF = overflow
            ctx.emitter.instruction("seto al");                                 // 1 when the product overflowed
            ctx.emitter.instruction("movzx eax, al");                           // widen the flag
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

/// Lowers `ICheckedDiv`: throws `DivisionByZeroError` for a zero divisor, then calls
/// `__rt_int_div_checked`, which boxes an exact quotient as int and any other as float.
pub(super) fn lower_int_checked_div(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let lhs_reg = abi::int_result_reg(ctx.emitter);
    let rhs_reg = abi::secondary_scratch_reg(ctx.emitter);
    load_integer_operand(ctx, lhs, lhs_reg, inst)?;
    load_integer_operand(ctx, rhs, rhs_reg, inst)?;
    if !value_is_nonzero_constant(ctx, rhs) {
        emit_int_zero_divisor_throw(ctx, rhs_reg, "Division by zero");
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov x1, {}", rhs_reg));           // divisor in the second helper argument register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov rdi, {}", lhs_reg));          // dividend in the first SysV argument register
            ctx.emitter.instruction(&format!("mov rsi, {}", rhs_reg));          // divisor in the second SysV argument register
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_int_div_checked");
    store_if_result(ctx, inst)
}

/// Lowers a checked integer add/sub/mul whose boxed `int|float` result only feeds a cast back
/// to `int` (`($a + $b) & 0xFFFF`), writing the cast's result directly.
///
/// The native operation runs first; only when it overflows does the slow path box the result
/// through the checked helper, convert it with `__rt_mixed_cast_int` and release the box —
/// exactly what the unfused instructions do — so the common case allocates nothing.
pub(super) fn lower_fused_checked_to_int(
    ctx: &mut FunctionContext<'_>,
    checked: &Instruction,
    cast: &Instruction,
    helper: &str,
) -> Result<()> {
    let lhs = expect_operand(checked, 0)?;
    let rhs = expect_operand(checked, 1)?;
    let slow = ctx.next_label("checked_cast_slow");
    let done = ctx.next_label("checked_cast_done");
    let result_reg = abi::int_result_reg(ctx.emitter);
    let rhs_reg = abi::secondary_scratch_reg(ctx.emitter);
    load_integer_operand(ctx, lhs, result_reg, checked)?;
    load_integer_operand(ctx, rhs, rhs_reg, checked)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            match checked.op {
                crate::ir::Op::ICheckedAdd => {
                    ctx.emitter.instruction("adds x9, x0, x10");                // add and set the overflow flag
                    ctx.emitter.instruction(&format!("b.vs {}", slow));         // signed overflow: take the boxed path
                }
                crate::ir::Op::ICheckedSub => {
                    ctx.emitter.instruction("subs x9, x0, x10");                // subtract and set the overflow flag
                    ctx.emitter.instruction(&format!("b.vs {}", slow));         // signed overflow: take the boxed path
                }
                _ => {
                    ctx.emitter.instruction("mul x9, x0, x10");                 // low 64 bits of the product
                    ctx.emitter.instruction("smulh x11, x0, x10");              // high 64 bits of the product
                    ctx.emitter.instruction("cmp x11, x9, asr #63");            // does the high half only extend the sign?
                    ctx.emitter.instruction(&format!("b.ne {}", slow));         // no — the product overflowed
                }
            }
            ctx.emitter.instruction("mov x0, x9");                              // the native result is the int value
            ctx.emitter.instruction(&format!("b {}", done));                    // skip the boxed path
            ctx.emitter.label(&slow);
            ctx.emitter.instruction("mov x1, x10");                             // second checked-helper argument
            abi::emit_call_label(ctx.emitter, helper);
            ctx.emitter.instruction("str x0, [sp, #-16]!");                     // keep the boxed result for its release
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int");
            ctx.emitter.instruction("str x0, [sp, #8]");                        // keep the converted int
            ctx.emitter.instruction("ldr x0, [sp]");                            // reload the box
            abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            ctx.emitter.instruction("ldr x0, [sp, #8]");                        // restore the converted int
            ctx.emitter.instruction("add sp, sp, #16");                         // release the temporary slots
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rcx, rax");                            // keep the left operand for the slow path
            match checked.op {
                crate::ir::Op::ICheckedAdd => ctx.emitter.instruction(&format!("add rax, {}", rhs_reg)), // native add
                crate::ir::Op::ICheckedSub => ctx.emitter.instruction(&format!("sub rax, {}", rhs_reg)), // native subtract
                _ => ctx.emitter.instruction(&format!("imul rax, {}", rhs_reg)), // native multiply
            }
            ctx.emitter.instruction(&format!("jo {}", slow));                   // signed overflow: take the boxed path
            ctx.emitter.instruction(&format!("jmp {}", done));                  // the native result is the int value
            ctx.emitter.label(&slow);
            ctx.emitter.instruction("mov rdi, rcx");                            // first checked-helper argument
            ctx.emitter.instruction(&format!("mov rsi, {}", rhs_reg));          // second checked-helper argument
            abi::emit_call_label(ctx.emitter, helper);
            ctx.emitter.instruction("sub rsp, 16");                             // keep the box and the converted int
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");                // save the boxed result for its release
            ctx.emitter.instruction("mov rdi, rax");                            // convert the box to int
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int");
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");            // keep the converted int
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp]");                // reload the box
            abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 8]");            // restore the converted int
            ctx.emitter.instruction("add rsp, 16");                             // release the temporary slots
        }
    }
    ctx.emitter.label(&done);
    store_if_result(ctx, cast)
}

/// Lowers a signed integer modulo operation with the established zero-divisor guard.
pub(super) fn lower_int_mod(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let result_reg = abi::int_result_reg(ctx.emitter);
    let rhs_reg = abi::secondary_scratch_reg(ctx.emitter);
    load_integer_operand(ctx, lhs, result_reg, inst)?;
    load_integer_operand(ctx, rhs, rhs_reg, inst)?;
    if !value_is_nonzero_constant(ctx, rhs) {
        emit_int_zero_divisor_throw(ctx, rhs_reg, "Modulo by zero");
    }
    let minus_one_label = ctx.next_label("mod_minus_one");
    let done_label = ctx.next_label("mod_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let quotient_reg = abi::tertiary_scratch_reg(ctx.emitter);
            ctx.emitter.instruction(&format!("cmn {}, #1", rhs_reg));           // is the divisor -1?
            ctx.emitter.instruction(&format!("b.eq {}", minus_one_label));      // x % -1 is 0 (and PHP_INT_MIN % -1 must not trap)
            ctx.emitter.instruction(&format!("sdiv {}, {}, {}", quotient_reg, result_reg, rhs_reg)); // compute signed quotient for the modulo operation
            ctx.emitter.instruction(&format!("msub {}, {}, {}, {}", result_reg, quotient_reg, rhs_reg, result_reg)); // compute left - quotient * right as the remainder
            ctx.emitter.instruction(&format!("b {}", done_label));              // skip the -1 shortcut
            ctx.emitter.label(&minus_one_label);
            ctx.emitter.instruction(&format!("mov {}, #0", result_reg));        // any integer modulo -1 is zero
            ctx.emitter.label(&done_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp {}, -1", rhs_reg));           // is the divisor -1?
            ctx.emitter.instruction(&format!("je {}", minus_one_label));        // x % -1 is 0 (and PHP_INT_MIN % -1 must not fault in idiv)
            ctx.emitter.instruction("cqo");                                     // sign-extend the dividend before signed division
            ctx.emitter.instruction(&format!("idiv {}", rhs_reg));              // divide signed integers with quotient in rax and remainder in rdx
            ctx.emitter.instruction(&format!("mov {}, rdx", result_reg));       // move the signed remainder into the integer result register
            ctx.emitter.instruction(&format!("jmp {}", done_label));            // skip the -1 shortcut
            ctx.emitter.label(&minus_one_label);
            ctx.emitter.instruction(&format!("mov {}, 0", result_reg));         // any integer modulo -1 is zero
            ctx.emitter.label(&done_label);
        }
    }
    store_if_result(ctx, inst)
}

/// Returns whether `value` is a constant that is known not to be zero (no division-by-zero
/// check needed).
pub(super) fn value_is_nonzero_constant(ctx: &FunctionContext<'_>, value: ValueId) -> bool {
    let Some(crate::ir::ValueDef::Instruction { inst, .. }) = ctx.function.value(value).map(|v| v.def)
    else {
        return false;
    };
    let Some(inst) = ctx.function.instruction(inst) else {
        return false;
    };
    match (inst.op, inst.immediate.as_ref()) {
        (crate::ir::Op::ConstI64, Some(Immediate::I64(n))) => *n != 0,
        (crate::ir::Op::ConstF64, Some(Immediate::F64(f))) => *f != 0.0,
        _ => false,
    }
}

/// Emits `if (divisor == 0) throw new DivisionByZeroError(message)` for an integer divisor
/// already loaded in `divisor_reg`; the throw never returns, so its helper calls may clobber
/// any register.
fn emit_int_zero_divisor_throw(ctx: &mut FunctionContext<'_>, divisor_reg: &str, message: &str) {
    let ok = ctx.next_label("div_nonzero");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbnz {}, {}", divisor_reg, ok));  // a non-zero divisor divides normally
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("test {}, {}", divisor_reg, divisor_reg)); // is the divisor zero?
            ctx.emitter.instruction(&format!("jnz {}", ok));                    // a non-zero divisor divides normally
        }
    }
    super::exceptions::emit_division_by_zero_error(ctx, message);
    ctx.emitter.label(&ok);
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
    if !value_is_nonzero_constant(ctx, rhs) {
        emit_int_zero_divisor_throw(ctx, rhs_reg, "Division by zero");
    }
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
    if !mixed_numeric_int_fast_path_applies(ctx, inst)? {
        emit_mixed_numeric_generic(ctx, inst)?;
        return store_if_result(ctx, inst);
    }
    let slow = ctx.next_label("mixed_num_slow");
    let done = ctx.next_label("mixed_num_done");
    emit_mixed_numeric_int_fast_path(ctx, inst, &slow)?;
    // Box the native int result.
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x9");                              // int payload for the boxing helper
            ctx.emitter.instruction("mov x0, #0");                              // runtime tag 0 = int
            abi::emit_call_label(ctx.emitter, "__rt_mixed_box_scalar");
            ctx.emitter.instruction(&format!("b {}", done));                    // boxed result ready
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // int payload for the boxing helper
            ctx.emitter.instruction("mov rax, 0");                              // runtime tag 0 = int
            abi::emit_call_label(ctx.emitter, "__rt_mixed_box_scalar");
            ctx.emitter.instruction(&format!("jmp {}", done));                  // boxed result ready
        }
    }
    ctx.emitter.label(&slow);
    emit_mixed_numeric_generic(ctx, inst)?;
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)
}

/// Lowers a mixed `+`/`-`/`*` whose boxed result is only converted back to `int`: when both
/// operands hold ints the native result is the answer (no box at all); otherwise the generic
/// helper's box is converted with PHP `(int)` semantics and released.
pub(super) fn lower_fused_mixed_numeric_to_int(
    ctx: &mut FunctionContext<'_>,
    binop: &Instruction,
    cast: &Instruction,
) -> Result<()> {
    let slow = ctx.next_label("mixed_num_cast_slow");
    let done = ctx.next_label("mixed_num_cast_done");
    emit_mixed_numeric_int_fast_path(ctx, binop, &slow)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x9");                              // the native int result
            ctx.emitter.instruction(&format!("b {}", done));                    // skip the generic path
        }
        Arch::X86_64 => ctx.emitter.instruction(&format!("jmp {}", done)),       // the native int result is in rax
    }
    ctx.emitter.label(&slow);
    emit_mixed_numeric_generic(ctx, binop)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp, #-16]!");                     // keep the boxed result for its release
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int");
            ctx.emitter.instruction("str x0, [sp, #8]");                        // keep the converted int
            ctx.emitter.instruction("ldr x0, [sp]");                            // reload the box
            abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            ctx.emitter.instruction("ldr x0, [sp, #8]");                        // restore the converted int
            ctx.emitter.instruction("add sp, sp, #16");                         // release the temporary slots
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("sub rsp, 16");                             // keep the box and the converted int
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");                // save the boxed result for its release
            ctx.emitter.instruction("mov rdi, rax");                            // convert the box to int
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int");
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");            // keep the converted int
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp]");                // reload the box
            abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 8]");            // restore the converted int
            ctx.emitter.instruction("add rsp, 16");                             // release the temporary slots
        }
    }
    ctx.emitter.label(&done);
    store_if_result(ctx, cast)
}

/// Lowers a checked int add/sub/mul whose boxed `int|float` result only feeds the mixed
/// `+`/`-`/`*` `binop` (`$row[$j] += $a * $b`). The native operation runs first and, when it
/// does not overflow, its raw int is kept in the checked value's home and the mixed operation
/// is lowered with that operand typed `int`; on overflow the checked helper boxes the float
/// result, the generic mixed operation consumes it and the box is released — what the
/// unfused instructions do.
pub(super) fn lower_fused_checked_into_mixed(
    ctx: &mut FunctionContext<'_>,
    checked: &Instruction,
    binop: &Instruction,
    helper: &str,
) -> Result<()> {
    let lhs = expect_operand(checked, 0)?;
    let rhs = expect_operand(checked, 1)?;
    let checked_value = checked
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("checked arithmetic without a result"))?;
    let overflow = ctx.next_label("checked_mixed_overflow");
    let done = ctx.next_label("checked_mixed_done");
    let result_reg = abi::int_result_reg(ctx.emitter);
    let rhs_reg = abi::secondary_scratch_reg(ctx.emitter);
    load_integer_operand(ctx, lhs, result_reg, checked)?;
    load_integer_operand(ctx, rhs, rhs_reg, checked)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            match checked.op {
                crate::ir::Op::ICheckedAdd => {
                    ctx.emitter.instruction("adds x9, x0, x10");                // add and set the overflow flag
                    ctx.emitter.instruction(&format!("b.vs {}", overflow));     // signed overflow: take the boxed path
                }
                crate::ir::Op::ICheckedSub => {
                    ctx.emitter.instruction("subs x9, x0, x10");                // subtract and set the overflow flag
                    ctx.emitter.instruction(&format!("b.vs {}", overflow));     // signed overflow: take the boxed path
                }
                _ => {
                    ctx.emitter.instruction("mul x9, x0, x10");                 // low 64 bits of the product
                    ctx.emitter.instruction("smulh x11, x0, x10");              // high 64 bits of the product
                    ctx.emitter.instruction("cmp x11, x9, asr #63");            // does the high half only extend the sign?
                    ctx.emitter.instruction(&format!("b.ne {}", overflow));     // no — the product overflowed
                }
            }
            ctx.emitter.instruction("mov x0, x9");                              // the native int result
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rcx, rax");                            // keep the left operand for the overflow path
            match checked.op {
                crate::ir::Op::ICheckedAdd => ctx.emitter.instruction(&format!("add rax, {}", rhs_reg)), // native add
                crate::ir::Op::ICheckedSub => ctx.emitter.instruction(&format!("sub rax, {}", rhs_reg)), // native subtract
                _ => ctx.emitter.instruction(&format!("imul rax, {}", rhs_reg)), // native multiply
            }
            ctx.emitter.instruction(&format!("jo {}", overflow));               // signed overflow: take the boxed path
        }
    }
    ctx.store_int_result_value(checked_value)?;
    ctx.value_type_overrides.insert(checked_value, PhpType::Int);
    let fast = lower_mixed_numeric_binop(ctx, binop);
    ctx.value_type_overrides.remove(&checked_value);
    fast?;
    abi::emit_jump(ctx.emitter, &done);

    ctx.emitter.label(&overflow);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x10");                             // second checked-helper argument (x0 still holds the first)
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rcx");                            // first checked-helper argument
            ctx.emitter.instruction(&format!("mov rsi, {}", rhs_reg));          // second checked-helper argument
        }
    }
    abi::emit_call_label(ctx.emitter, helper);
    ctx.store_int_result_value(checked_value)?;
    emit_mixed_numeric_generic(ctx, binop)?;
    abi::emit_push_reg(ctx.emitter, result_reg);
    ctx.load_value_to_reg(checked_value, result_reg)?;
    abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");                      // release the overflow box the unfused release would free
    abi::emit_pop_reg(ctx.emitter, result_reg);
    store_if_result(ctx, binop)?;
    ctx.emitter.label(&done);
    Ok(())
}

/// Returns whether a mixed `+`/`-`/`*` has only `int`, `int|null` and `mixed` operands (at
/// least one of them not a plain int), so both can be tested for an int payload and combined natively.
pub(in crate::codegen) fn mixed_numeric_int_fast_path_applies(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<bool> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    if expect_mixed_numeric_op(inst)? == MixedNumericOp::Div {
        return Ok(false);
    }
    let lhs_ty = ctx.value_php_type(lhs)?.codegen_repr();
    let rhs_ty = ctx.value_php_type(rhs)?.codegen_repr();
    Ok(int_probe_operand_type(&lhs_ty)
        && int_probe_operand_type(&rhs_ty)
        && (lhs_ty != PhpType::Int || rhs_ty != PhpType::Int))
}

/// Emits the int fast path of a mixed `+`/`-`/`*`: branches to `slow` unless every boxed
/// operand holds an int payload and the native operation does not overflow. On fall-through
/// the int result is in x9 (AArch64) / rax (x86_64).
fn emit_mixed_numeric_int_fast_path(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    slow: &str,
) -> Result<()> {
    let lhs = expect_operand(inst, 0)?;
    let rhs = expect_operand(inst, 1)?;
    let op = expect_mixed_numeric_op(inst)?;
    emit_int_operand_probe(ctx, lhs, rhs, slow)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => match op {
            MixedNumericOp::Add => {
                ctx.emitter.instruction("adds x9, x0, x10");                    // add and set the overflow flag
                ctx.emitter.instruction(&format!("b.vs {}", slow));             // overflow: generic path (float result)
            }
            MixedNumericOp::Sub => {
                ctx.emitter.instruction("subs x9, x0, x10");                    // subtract and set the overflow flag
                ctx.emitter.instruction(&format!("b.vs {}", slow));             // overflow: generic path (float result)
            }
            MixedNumericOp::Mul => {
                ctx.emitter.instruction("mul x9, x0, x10");                     // low 64 bits of the product
                ctx.emitter.instruction("smulh x11, x0, x10");                  // high 64 bits of the product
                ctx.emitter.instruction("cmp x11, x9, asr #63");                // does the high half only extend the sign?
                ctx.emitter.instruction(&format!("b.ne {}", slow));             // overflow: generic path (float result)
            }
            MixedNumericOp::Div => {
                ctx.emitter.instruction(&format!("b {}", slow));                // division always takes the runtime helper
            }
        },
        Arch::X86_64 => {
            match op {
                MixedNumericOp::Add => ctx.emitter.instruction("add rax, rcx"), // native add
                MixedNumericOp::Sub => ctx.emitter.instruction("sub rax, rcx"), // native subtract
                MixedNumericOp::Mul => ctx.emitter.instruction("imul rax, rcx"), // native multiply
                MixedNumericOp::Div => ctx.emitter.instruction(&format!("jmp {}", slow)), // division always takes the runtime helper
            }
            ctx.emitter.instruction(&format!("jo {}", slow));                   // overflow: generic path (float result)
        }
    }
    Ok(())
}

/// Returns whether a value's codegen type can be probed for an int payload at run time.
pub(in crate::codegen) fn int_probe_operand_type(ty: &PhpType) -> bool {
    matches!(ty, PhpType::Int | PhpType::Mixed | PhpType::TaggedScalar)
}

/// Loads two `int`/`mixed`/`int|null` operands as native ints, branching to `slow` unless every
/// boxed or tagged operand currently holds an int. On fall-through the left int is in x0/rax
/// and the right int in x10/rcx. Only result and scratch registers are touched.
pub(in crate::codegen) fn emit_int_operand_probe(
    ctx: &mut FunctionContext<'_>,
    lhs: ValueId,
    rhs: ValueId,
    slow: &str,
) -> Result<()> {
    let result_reg = abi::int_result_reg(ctx.emitter);
    let mut kinds = [PhpType::Int, PhpType::Int];
    for (index, value) in [lhs, rhs].into_iter().enumerate() {
        let ty = ctx.value_php_type(value)?.codegen_repr();
        match ty {
            PhpType::Mixed => {
                ctx.load_value_to_reg(value, result_reg)?;
                match ctx.emitter.target.arch {
                    Arch::AArch64 => {
                        ctx.emitter.instruction(&format!("cbz x0, {}", slow));  // no box: generic path
                        ctx.emitter.instruction("ldr x9, [x0]");                // runtime tag of the box
                        ctx.emitter.instruction(&format!("cbnz x9, {}", slow)); // not an int payload: generic path
                    }
                    Arch::X86_64 => {
                        ctx.emitter.instruction("test rax, rax");               // no box?
                        ctx.emitter.instruction(&format!("jz {}", slow));       // generic path
                        ctx.emitter.instruction("cmp QWORD PTR [rax], 0");      // runtime tag of the box
                        ctx.emitter.instruction(&format!("jne {}", slow));      // not an int payload: generic path
                    }
                }
            }
            PhpType::TaggedScalar => {
                ctx.load_value_to_result(value)?;
                match ctx.emitter.target.arch {
                    Arch::AArch64 => {
                        ctx.emitter.instruction(&format!("cbnz x1, {}", slow)); // null tag: generic path
                    }
                    Arch::X86_64 => {
                        ctx.emitter.instruction("test rdx, rdx");               // int tag?
                        ctx.emitter.instruction(&format!("jnz {}", slow));      // null tag: generic path
                    }
                }
            }
            _ => {}
        }
        kinds[index] = ty;
    }
    for (index, value) in [lhs, rhs].into_iter().enumerate() {
        match kinds[index] {
            PhpType::TaggedScalar => {
                ctx.load_value_to_result(value)?;
            }
            PhpType::Mixed => {
                ctx.load_value_to_reg(value, result_reg)?;
                match ctx.emitter.target.arch {
                    Arch::AArch64 => ctx.emitter.instruction("ldr x0, [x0, #8]"), // the int payload
                    Arch::X86_64 => ctx.emitter.instruction("mov rax, QWORD PTR [rax + 8]"), // the int payload
                }
            }
            _ => {
                ctx.load_value_to_reg(value, result_reg)?;
            }
        }
        if index == 0 {
            abi::emit_push_reg(ctx.emitter, result_reg);
        }
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x10, x0");                             // right operand
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rcx, rax");                            // right operand
            abi::emit_pop_reg(ctx.emitter, "rax");
        }
    }
    Ok(())
}

/// Emits the generic mixed `+`/`-`/`*`: boxes concrete operands, calls the runtime helper and
/// releases the temporary boxes, leaving the boxed result in the integer result register.
fn emit_mixed_numeric_generic(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
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
    if op == MixedNumericOp::Div {
        // The division helper returns no box for a zero divisor.
        let result_reg = abi::int_result_reg(ctx.emitter);
        emit_int_zero_divisor_throw(ctx, result_reg, "Division by zero");
    }
    Ok(())
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
        MixedNumericOp::Div => "__rt_mixed_numeric_div",
    }
}

#[cfg(test)]
mod logical_immediate_tests {
    use super::is_aarch64_logical_immediate;

    /// Checks the AArch64 bitmask-immediate encoder against known encodable and rejected values.
    #[test]
    fn test_aarch64_logical_immediate_classification() {
        for value in [7u64, 0x3fff_ffff, 0xff, 0xff00, 0x5555_5555_5555_5555, 1 << 63, 0xffff_fffe, 0x0f0f_0f0f_0f0f_0f0f, u64::MAX - 1] {
            assert!(is_aarch64_logical_immediate(value), "{value:#x} should be encodable");
        }
        for value in [0u64, u64::MAX, 5, 0x1234, 0xff00_ff01, 13] {
            assert!(!is_aarch64_logical_immediate(value), "{value:#x} should be rejected");
        }
    }
}
