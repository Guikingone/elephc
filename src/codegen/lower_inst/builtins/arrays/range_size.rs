//! Purpose:
//! Raises reference PHP's `"The supplied range exceeds the maximum array size"` `ValueError`
//! for a `range()` whose element count is past PHP's maximum, before `__rt_range` is asked for
//! the allocation.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::arrays::emit_range_guards()`.
//!
//! Key details:
//! - php-src computes `(zend_ulong) high - low) / step` on the NORMALIZED endpoints (`low` is the
//!   smaller argument, `step` its magnitude) and refuses the range as soon as that quotient
//!   reaches `HT_MAX_SIZE - 1`, i.e. as soon as the array would need more than `2^30 - 1`
//!   elements. The subtraction and the division are both unsigned, so `range(PHP_INT_MIN,
//!   PHP_INT_MAX)` reports a `2^64 - 1` span instead of wrapping to a small signed one.
//! - The message interpolates the normalized `low`, `high` and `|step|`, so it is built at
//!   runtime through `__rt_itoa`/`__rt_concat` and persisted before the throwable takes it over.
//! - The guard reads `$start`/`$end`/`$step` while they still sit in their ABI argument
//!   registers, so one sequence covers every supported target; the x86_64 path stages `$step`
//!   in a scratch register because the unsigned `div` overwrites `rdx`, which `__rt_range`
//!   still needs when the guard passes.

use crate::codegen::abi;
use crate::codegen::context::FunctionContext;
use crate::codegen::platform::Arch;

/// `HT_MAX_SIZE - 1`: the element-count-minus-one php-src refuses to build a `range()` for.
///
/// php-src rejects when `(high - low) / |step| >= HT_MAX_SIZE - 1`, so the largest range it
/// accepts holds `1073741823` elements: `range(1, 1073741823)` is built (and then fails on
/// memory), `range(0, 1073741823)` is a `ValueError`.
const RANGE_MAX_SPAN_STEPS: i64 = 1_073_741_823;

/// The fixed head of php-src's oversized-range `ValueError`, up to the normalized low endpoint.
const RANGE_SIZE_MESSAGE_PREFIX: &str =
    "The supplied range exceeds the maximum array size: start=";

/// The message fragment php-src writes between the normalized low and high endpoints.
const RANGE_SIZE_MESSAGE_END_SEPARATOR: &str = " end=";

/// The message fragment php-src writes between the normalized high endpoint and the step.
const RANGE_SIZE_MESSAGE_STEP_SEPARATOR: &str = " step=";

/// Raises PHP's oversized-range `ValueError` unless the requested element count fits an array.
///
/// Expects `$start`, `$end` and `$step` in `__rt_range`'s first three ABI argument registers and
/// leaves them untouched on the accepted path. A rejected range never returns: the failure path
/// builds php-src's exact message from the normalized endpoints and hands it to the unwinder.
pub(super) fn emit_range_size_guard(ctx: &mut FunctionContext<'_>) {
    let ok_label = ctx.next_label("range_size_ok");
    let normalized = match ctx.emitter.target.arch {
        Arch::AArch64 => emit_span_check_aarch64(ctx, &ok_label),
        Arch::X86_64 => emit_span_check_x86_64(ctx, &ok_label),
    };
    emit_range_size_value_error(ctx, normalized);
    ctx.emitter.label(&ok_label);
}

/// The registers holding the normalized interval on the guard's failure path.
///
/// `low`/`high` are the ordered endpoints and `step` their stride magnitude — exactly the three
/// values php-src interpolates into the message, in the order it prints them.
struct NormalizedRange {
    /// Register holding the smaller of `$start` and `$end`.
    low: &'static str,
    /// Register holding the larger of `$start` and `$end`.
    high: &'static str,
    /// Register holding `abs($step)`.
    step: &'static str,
}

/// Emits the AArch64 span check and branches to `ok_label` for a range that fits an array.
///
/// Normalizes the endpoints with `csel`, the step with `cneg`, then divides the unsigned span by
/// the stride. `udiv` by zero answers zero on AArch64, so even a step the caller failed to reject
/// cannot fault here.
fn emit_span_check_aarch64(ctx: &mut FunctionContext<'_>, ok_label: &str) -> NormalizedRange {
    ctx.emitter.instruction("cmp x0, x1");                                      // is the requested interval increasing?
    ctx.emitter.instruction("csel x9, x0, x1, le");                             // x9 = low, the smaller of start and end
    ctx.emitter.instruction("csel x10, x1, x0, le");                            // x10 = high, the larger of start and end
    ctx.emitter.instruction("cmp x2, #0");                                      // is the requested step negative?
    ctx.emitter.instruction("cneg x11, x2, mi");                                // x11 = |step|, the stride PHP counts with
    ctx.emitter.instruction("sub x12, x10, x9");                                // x12 = high - low, the spanned interval as an unsigned width
    ctx.emitter.instruction("udiv x12, x12, x11");                              // x12 = span / |step|, PHP's element count minus one
    abi::emit_load_int_immediate(ctx.emitter, "x13", RANGE_MAX_SPAN_STEPS);
    ctx.emitter.instruction("cmp x12, x13");                                    // compare the requested element count against PHP's maximum array size
    ctx.emitter.instruction(&format!("b.lo {}", ok_label));                     // a range below the maximum array size is built normally
    NormalizedRange {
        low: "x9",
        high: "x10",
        step: "x11",
    }
}

/// Emits the x86_64 span check and branches to `ok_label` for a range that fits an array.
///
/// `div` reads its dividend from `rdx:rax` and writes the remainder back to `rdx`, which still
/// carries `$step` for the call that follows, so the step is staged in `r9` and restored right
/// after the divide.
fn emit_span_check_x86_64(ctx: &mut FunctionContext<'_>, ok_label: &str) -> NormalizedRange {
    ctx.emitter.instruction("mov r10, rdi");                                    // stage start as the interval low endpoint
    ctx.emitter.instruction("mov r11, rsi");                                    // stage end as the interval high endpoint
    ctx.emitter.instruction("cmp rdi, rsi");                                    // is the requested interval decreasing?
    ctx.emitter.instruction("cmovg r10, rsi");                                  // r10 = low, the smaller of start and end
    ctx.emitter.instruction("cmovg r11, rdi");                                  // r11 = high, the larger of start and end
    ctx.emitter.instruction("mov r9, rdx");                                     // preserve the step argument across the unsigned divide
    ctx.emitter.instruction("mov rcx, rdx");                                    // stage the step before normalizing its magnitude
    ctx.emitter.instruction("neg rcx");                                         // negate the step so a negative one yields its magnitude
    ctx.emitter.instruction("test rdx, rdx");                                   // is the requested step negative?
    ctx.emitter.instruction("cmovns rcx, rdx");                                 // rcx = |step|, the stride PHP counts with
    ctx.emitter.instruction("mov rax, r11");                                    // stage the interval high endpoint before subtracting the low one
    ctx.emitter.instruction("sub rax, r10");                                    // rax = high - low, the spanned interval as an unsigned width
    ctx.emitter.instruction("xor edx, edx");                                    // clear the dividend high word for the unsigned divide
    ctx.emitter.instruction("div rcx");                                         // rax = span / |step|, PHP's element count minus one
    ctx.emitter.instruction("mov rdx, r9");                                     // restore the step argument for the range helper
    abi::emit_load_int_immediate(ctx.emitter, "r8", RANGE_MAX_SPAN_STEPS);
    ctx.emitter.instruction("cmp rax, r8");                                     // compare the requested element count against PHP's maximum array size
    ctx.emitter.instruction(&format!("jb {}", ok_label));                       // a range below the maximum array size is built normally
    NormalizedRange {
        low: "r10",
        high: "r11",
        step: "rcx",
    }
}

/// Builds php-src's oversized-range message from the normalized interval and throws it.
///
/// The three integers are parked in one 32-byte temporary first, because `__rt_itoa` and
/// `__rt_concat` both clobber every caller-saved register. The partially built message is parked
/// the same way across each following `__rt_itoa`, so only one concat result is ever live in
/// registers at a time. Control never returns from here.
fn emit_range_size_value_error(ctx: &mut FunctionContext<'_>, normalized: NormalizedRange) {
    // Temporary layout after both pushes: [0]=|step|, [16]=low, [24]=high.
    abi::emit_push_reg_pair(ctx.emitter, normalized.low, normalized.high);
    abi::emit_push_reg(ctx.emitter, normalized.step);
    emit_itoa_from_temporary_slot(ctx, 16);
    emit_concat_static_prefix(ctx, RANGE_SIZE_MESSAGE_PREFIX);
    emit_concat_static_suffix(ctx, RANGE_SIZE_MESSAGE_END_SEPARATOR);
    emit_concat_temporary_slot_integer(ctx, 40);
    emit_concat_static_suffix(ctx, RANGE_SIZE_MESSAGE_STEP_SEPARATOR);
    emit_concat_temporary_slot_integer(ctx, 16);
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    abi::emit_call_label(ctx.emitter, "__rt_str_persist");
    crate::codegen::lower_inst::exceptions::emit_value_error_from_string_result(ctx);
}

/// Converts the integer parked at `offset` in the temporary stack to its decimal digits.
///
/// Leaves the digits in the target's string-result registers, which is where `__rt_concat`
/// expects its left operand.
fn emit_itoa_from_temporary_slot(ctx: &mut FunctionContext<'_>, offset: usize) {
    let integer_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, integer_reg, offset);
    abi::emit_call_label(ctx.emitter, "__rt_itoa");
}

/// Prepends a static fragment to the string currently held in the string-result registers.
fn emit_concat_static_prefix(ctx: &mut FunctionContext<'_>, prefix: &str) {
    let (text_ptr, text_len) = abi::string_result_regs(ctx.emitter);
    let (right_ptr, right_len) = concat_right_operand_regs(ctx);
    let (prefix_label, prefix_len) = ctx.data.add_string(prefix.as_bytes());
    ctx.emitter.instruction(&format!("mov {}, {}", right_ptr, text_ptr));       // move the built text into the concat right operand
    ctx.emitter.instruction(&format!("mov {}, {}", right_len, text_len));       // move its length into the concat right operand
    abi::emit_symbol_address(ctx.emitter, text_ptr, &prefix_label);
    abi::emit_load_int_immediate(ctx.emitter, text_len, prefix_len as i64);
    abi::emit_call_label(ctx.emitter, "__rt_concat");
}

/// Appends a static fragment to the string currently held in the string-result registers.
fn emit_concat_static_suffix(ctx: &mut FunctionContext<'_>, suffix: &str) {
    let (right_ptr, right_len) = concat_right_operand_regs(ctx);
    let (suffix_label, suffix_len) = ctx.data.add_string(suffix.as_bytes());
    abi::emit_symbol_address(ctx.emitter, right_ptr, &suffix_label);
    abi::emit_load_int_immediate(ctx.emitter, right_len, suffix_len as i64);
    abi::emit_call_label(ctx.emitter, "__rt_concat");
}

/// Appends the decimal digits of the integer parked at `offset` to the built message.
///
/// The message itself is parked in a fresh 16-byte temporary across `__rt_itoa`, so `offset` must
/// already account for that push — the caller passes the deeper offset.
fn emit_concat_temporary_slot_integer(ctx: &mut FunctionContext<'_>, offset: usize) {
    let (text_ptr, text_len) = abi::string_result_regs(ctx.emitter);
    let (right_ptr, right_len) = concat_right_operand_regs(ctx);
    abi::emit_push_reg_pair(ctx.emitter, text_ptr, text_len);
    emit_itoa_from_temporary_slot(ctx, offset);
    ctx.emitter.instruction(&format!("mov {}, {}", right_ptr, text_ptr));       // move the fresh digits into the concat right operand
    ctx.emitter.instruction(&format!("mov {}, {}", right_len, text_len));       // move their length into the concat right operand
    abi::emit_load_temporary_stack_slot(ctx.emitter, text_ptr, 0);
    abi::emit_load_temporary_stack_slot(ctx.emitter, text_len, 8);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    abi::emit_call_label(ctx.emitter, "__rt_concat");
}

/// Returns the registers `__rt_concat` reads its right operand pointer/length from.
fn concat_right_operand_regs(ctx: &FunctionContext<'_>) -> (&'static str, &'static str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ("x3", "x4"),
        Arch::X86_64 => ("rdi", "rsi"),
    }
}
