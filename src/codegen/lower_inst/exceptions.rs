//! Purpose:
//! Emits catchable built-in `Error` and `TypeError` objects for codegen guards.
//!
//! Called from:
//! - EIR instruction lowerers that detect PHP runtime type/null errors.
//!
//! Key details:
//! - Active handlers receive a normal throwable through `__rt_throw_current`.
//! - Unhandled errors keep a specific PHP-style fatal diagnostic instead of the
//!   runtime unwinder's generic uncaught-exception fallback.
//! - That diagnostic is written HERE, before the throwable is allocated, so it never reaches
//!   `__rt_report_uncaught_exception` and shares none of its logic. The exit status is therefore
//!   imported rather than spelled out: an uncaught `DivisionByZeroError` and an uncaught
//!   `throw new RuntimeException(...)` must not leave a script with different `$?` values.
//! - These messages carry no ` in <file>:<line>` suffix, unlike the unwinder's. The error is
//!   synthesized by a codegen guard rather than by a user `new`, and the message string is baked
//!   at emit time from a caller that passes no span — so there is no origin to print. Reference
//!   PHP does report one here (the operation's own line), which stays a known gap.
//! - `emit_value_error_unless()` is the shared builtin argument-range guard: it keeps
//!   out-of-range arguments (empty separators, non-positive lengths, negative counts,
//!   oversized array lengths) from ever reaching a runtime helper that would read
//!   uninitialized memory, allocate an unrepresentable size, or loop forever, and raises
//!   reference PHP's catchable `ValueError` instead.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen_support::runtime::UNCAUGHT_EXIT_STATUS;
use crate::ir::ValueId;

use super::super::context::FunctionContext;
use super::super::Result;

/// Throws a catchable PHP `Error` carrying a static message.
pub(super) fn emit_error(ctx: &mut FunctionContext<'_>, message: &str) {
    emit_static_exception(ctx, "Error", "_spl_error_class_id", message);
}

/// Throws a catchable PHP `TypeError` carrying a static message.
pub(super) fn emit_type_error(ctx: &mut FunctionContext<'_>, message: &str) {
    emit_static_exception(ctx, "TypeError", "_spl_type_error_class_id", message);
}

/// Throws a catchable PHP `ValueError` carrying a static message.
///
/// Reference PHP raises this `Error` subclass — not a fatal — when a builtin argument has
/// the right type but a value the function cannot honor (`str_pad()` with an empty pad
/// string, `str_split()` with a non-positive chunk length, `str_repeat()` with a negative
/// count, `explode()` with an empty separator, `array_fill()` with a negative count,
/// `random_int()` with `$min > $max`). `catch (ValueError $e)`, `catch (Error $e)`, and
/// `catch (Throwable $e)` all match; callers pass php-src's own verbatim wording.
pub(super) fn emit_value_error(ctx: &mut FunctionContext<'_>, message: &str) {
    emit_static_exception(ctx, "ValueError", "_spl_value_error_class_id", message);
}

/// The register condition a materialized builtin argument must satisfy to skip its
/// `ValueError`.
///
/// The register is inspected right before the runtime helper call, while the argument
/// still sits in its target ABI register, so the same guard works for every supported
/// target without re-materializing the operand.
pub(super) enum ValueGuard<'a> {
    /// The 64-bit register, read as a signed integer, must be `>= minimum`
    /// (`str_split()` chunk length, `str_repeat()`/`array_fill()` counts).
    SignedAtLeast(&'a str, i64),
    /// The 64-bit register, read as a signed integer, must satisfy
    /// `-maximum <= value <= maximum` (`array_pad()`'s `$length` magnitude).
    ///
    /// The bound is checked on the signed argument itself rather than on `abs(value)`
    /// so `PHP_INT_MIN`, whose magnitude is not representable, fails the guard instead
    /// of wrapping back to a negative "absolute" length.
    SignedMagnitudeAtMost(&'a str, i64),
    /// The 64-bit register, read as a signed integer, must satisfy
    /// `minimum <= value <= maximum` (`round()`'s `$mode` enumeration).
    ///
    /// Both ends are inclusive; the guard is used for builtin arguments whose accepted
    /// values are a small contiguous set of PHP constants rather than a magnitude limit.
    SignedInRange(&'a str, i64, i64),
}

/// Throws a catchable PHP `ValueError` unless the guarded register satisfies `guard`.
///
/// Emits the compare/branch pair for the active target, falls through to the throw
/// sequence when the guard fails, and leaves the caller's continuation label in place so
/// the runtime helper call that follows only ever runs with an in-range argument.
pub(super) fn emit_value_error_unless(
    ctx: &mut FunctionContext<'_>,
    guard: ValueGuard<'_>,
    message: &str,
) {
    let ok_label = ctx.next_label("value_guard_ok");
    match (ctx.emitter.target.arch, &guard) {
        (Arch::AArch64, ValueGuard::SignedAtLeast(reg, minimum)) => {
            ctx.emitter.instruction(&format!("cmp {}, #{}", reg, minimum));     // compare the materialized argument against its PHP minimum
            ctx.emitter.instruction(&format!("b.ge {}", ok_label));             // an argument at or above the minimum is in range
        }
        (Arch::X86_64, ValueGuard::SignedAtLeast(reg, minimum)) => {
            ctx.emitter.instruction(&format!("cmp {}, {}", reg, minimum));      // compare the materialized argument against its PHP minimum
            ctx.emitter.instruction(&format!("jge {}", ok_label));              // an argument at or above the minimum is in range
        }
        (Arch::AArch64, ValueGuard::SignedMagnitudeAtMost(reg, maximum)) => {
            let fail_label = ctx.next_label("value_guard_fail");
            ctx.emitter.instruction(&format!("mov x9, #{}", maximum));          // materialize the largest magnitude PHP accepts for this argument
            ctx.emitter.instruction(&format!("cmp {}, x9", reg));               // compare the materialized argument against the positive bound
            ctx.emitter.instruction(&format!("b.gt {}", fail_label));           // a value above the bound is out of range
            ctx.emitter.instruction(&format!("cmn {}, x9", reg));               // compare the materialized argument against the negated bound
            ctx.emitter.instruction(&format!("b.ge {}", ok_label));             // a value at or above the negated bound is in range
            ctx.emitter.label(&fail_label);
        }
        (Arch::X86_64, ValueGuard::SignedMagnitudeAtMost(reg, maximum)) => {
            let fail_label = ctx.next_label("value_guard_fail");
            ctx.emitter.instruction(&format!("cmp {}, {}", reg, maximum));      // compare the materialized argument against the positive bound
            ctx.emitter.instruction(&format!("jg {}", fail_label));             // a value above the bound is out of range
            ctx.emitter.instruction(&format!("cmp {}, -{}", reg, maximum));     // compare the materialized argument against the negated bound
            ctx.emitter.instruction(&format!("jge {}", ok_label));              // a value at or above the negated bound is in range
            ctx.emitter.label(&fail_label);
        }
        (Arch::AArch64, ValueGuard::SignedInRange(reg, minimum, maximum)) => {
            let fail_label = ctx.next_label("value_guard_fail");
            ctx.emitter.instruction(&format!("cmp {}, #{}", reg, minimum));     // compare the materialized argument against the inclusive lower bound
            ctx.emitter.instruction(&format!("b.lt {}", fail_label));           // a value below the range is rejected
            ctx.emitter.instruction(&format!("cmp {}, #{}", reg, maximum));     // compare the materialized argument against the inclusive upper bound
            ctx.emitter.instruction(&format!("b.le {}", ok_label));             // a value at or below the upper bound is in range
            ctx.emitter.label(&fail_label);
        }
        (Arch::X86_64, ValueGuard::SignedInRange(reg, minimum, maximum)) => {
            let fail_label = ctx.next_label("value_guard_fail");
            ctx.emitter.instruction(&format!("cmp {}, {}", reg, minimum));      // compare the materialized argument against the inclusive lower bound
            ctx.emitter.instruction(&format!("jl {}", fail_label));             // a value below the range is rejected
            ctx.emitter.instruction(&format!("cmp {}, {}", reg, maximum));      // compare the materialized argument against the inclusive upper bound
            ctx.emitter.instruction(&format!("jle {}", ok_label));              // a value at or below the upper bound is in range
            ctx.emitter.label(&fail_label);
        }
    }
    emit_value_error(ctx, message);
    ctx.emitter.label(&ok_label);
}

/// Throws a catchable PHP `DivisionByZeroError` carrying a static message.
///
/// Reference PHP raises this `ArithmeticError` subclass — not a bare fatal — for a
/// zero divisor, so `catch (DivisionByZeroError $e)`, `catch (ArithmeticError $e)`,
/// `catch (Error $e)`, and `catch (Throwable $e)` all match. Callers pass php-src's
/// own wording (`"Division by zero"` / `"Modulo by zero"`).
pub(super) fn emit_division_by_zero_error(ctx: &mut FunctionContext<'_>, message: &str) {
    emit_static_exception(
        ctx,
        "DivisionByZeroError",
        "_spl_division_by_zero_error_class_id",
        message,
    );
}

/// Throws a catchable PHP `ArithmeticError` carrying a static message.
///
/// Reference PHP raises this for arithmetic that has no representable result but is not a
/// division by zero — currently `<<`/`>>` with a negative shift count
/// (`ArithmeticError: Bit shift by negative number`). `catch (ArithmeticError $e)`,
/// `catch (Error $e)`, and `catch (Throwable $e)` all match; `DivisionByZeroError` does not.
pub(super) fn emit_arithmetic_error(ctx: &mut FunctionContext<'_>, message: &str) {
    emit_static_exception(
        ctx,
        "ArithmeticError",
        "_spl_arithmetic_error_class_id",
        message,
    );
}

/// Throws a catchable PHP `Error` whose message is a runtime string value.
pub(super) fn emit_error_value(ctx: &mut FunctionContext<'_>, message: ValueId) -> Result<()> {
    let (message_ptr_reg, message_len_reg) = abi::string_result_regs(ctx.emitter);
    ctx.load_string_value_to_regs(message, message_ptr_reg, message_len_reg)?;
    abi::emit_push_reg_pair(ctx.emitter, message_ptr_reg, message_len_reg);
    emit_uncaught_dynamic_error_fatal_if_no_handler(ctx);
    emit_dynamic_error_object(ctx);
    Ok(())
}

/// Allocates one built-in throwable and transfers control to the standard unwinder.
fn emit_static_exception(
    ctx: &mut FunctionContext<'_>,
    class_name: &str,
    class_id_symbol: &str,
    message: &str,
) {
    let fatal_message = format!("Fatal error: Uncaught {}: {}\n", class_name, message);
    let (fatal_label, fatal_len) = ctx.data.add_string(fatal_message.as_bytes());
    emit_uncaught_exception_fatal_if_no_handler(ctx, &fatal_label, fatal_len);

    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_int_immediate(ctx.emitter, "x0", 56); // compact Throwable: message/code/previous
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = throwable object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            ctx.emitter.instruction("bl __rt_object_handle_acquire");           // bind the new object to its PHP object handle
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", class_id_symbol, 0);
            ctx.emitter.instruction("str x9, [x0]");                            // store the built-in throwable class id
            abi::emit_symbol_address(ctx.emitter, "x9", &message_label);
            ctx.emitter.instruction("str x9, [x0, #8]");                        // store the static exception message pointer
            abi::emit_load_int_immediate(ctx.emitter, "x9", message_len as i64);
            ctx.emitter.instruction("str x9, [x0, #16]");                       // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
            crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(ctx.emitter, "x0");
            ctx.emitter.instruction("str xzr, [x0, #40]");                      // previous defaults to null
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0);
            abi::emit_jump(ctx.emitter, "__rt_throw_current");
        }
        Arch::X86_64 => {
            abi::emit_load_int_immediate(ctx.emitter, "rax", 56); // compact Throwable: message/code/previous
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
            ctx.emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(6))); // stamp the canonical x86_64 heap-kind word (magic + kind 6 throwable)
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            ctx.emitter.instruction("call __rt_object_handle_acquire");         // bind the new object to its PHP object handle
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", class_id_symbol, 0);
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the built-in throwable class id
            abi::emit_symbol_address(ctx.emitter, "r10", &message_label);
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10");            // store the static exception message pointer
            abi::emit_load_int_immediate(ctx.emitter, "r10", message_len as i64);
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r10");           // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
            crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(ctx.emitter, "rax");
            ctx.emitter.instruction("mov QWORD PTR [rax + 40], 0");             // previous defaults to null
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0);
            abi::emit_jump(ctx.emitter, "__rt_throw_current");
        }
    }
}

/// Writes the specific uncaught diagnostic and exits when no catch handler is active.
fn emit_uncaught_exception_fatal_if_no_handler(
    ctx: &mut FunctionContext<'_>,
    fatal_label: &str,
    fatal_len: usize,
) {
    let throw_label = ctx.next_label("static_exception_throw");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_exc_handler_top", 0);
            ctx.emitter.instruction(&format!("cbnz x9, {}", throw_label));      // use the standard unwinder when a catch handler is active
            abi::emit_symbol_address(ctx.emitter, "x1", fatal_label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", fatal_len as i64);
            ctx.emitter.instruction("mov x0, #2");                              // write the uncaught PHP diagnostic to stderr
            ctx.emitter.syscall(4);
            abi::emit_exit(ctx.emitter, UNCAUGHT_EXIT_STATUS);
        }
        Arch::X86_64 => {
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_exc_handler_top", 0);
            ctx.emitter.instruction("test r10, r10");                           // check whether a catch handler is active
            ctx.emitter.instruction(&format!("jnz {}", throw_label));           // use the standard unwinder when a handler can receive the error
            abi::emit_symbol_address(ctx.emitter, "rsi", fatal_label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", fatal_len as i64);
            ctx.emitter.instruction("mov edi, 2");                              // write the uncaught PHP diagnostic to stderr
            ctx.emitter.instruction("mov eax, 1");                              // Linux x86_64 syscall 1 = write
            ctx.emitter.instruction("syscall");                                 // emit the specific fatal message
            abi::emit_exit(ctx.emitter, UNCAUGHT_EXIT_STATUS);
        }
    }
    ctx.emitter.label(&throw_label);
}

/// Writes an uncaught dynamic `Error` diagnostic, or continues when a handler exists.
fn emit_uncaught_dynamic_error_fatal_if_no_handler(ctx: &mut FunctionContext<'_>) {
    let throw_label = ctx.next_label("dynamic_error_throw");
    let (prefix_label, prefix_len) = ctx.data.add_string(b"Fatal error: Uncaught Error: ");
    let (suffix_label, suffix_len) = ctx.data.add_string(b"\n");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_exc_handler_top", 0);
            ctx.emitter.instruction(&format!("cbnz x9, {}", throw_label));      // use the standard unwinder when a catch handler is active
            ctx.emitter.instruction("mov x0, #2");                              // write the uncaught dynamic-error prefix to stderr
            abi::emit_symbol_address(ctx.emitter, "x1", &prefix_label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", prefix_len as i64);
            ctx.emitter.syscall(4);
            ctx.emitter.instruction("mov x0, #2");                              // write the runtime error message to stderr
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 0);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x2", 8);
            ctx.emitter.syscall(4);
            ctx.emitter.instruction("mov x0, #2");                              // terminate the uncaught diagnostic with a newline
            abi::emit_symbol_address(ctx.emitter, "x1", &suffix_label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", suffix_len as i64);
            ctx.emitter.syscall(4);
            abi::emit_exit(ctx.emitter, UNCAUGHT_EXIT_STATUS);
        }
        Arch::X86_64 => {
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_exc_handler_top", 0);
            ctx.emitter.instruction("test r10, r10");                           // check whether a catch handler is active
            ctx.emitter.instruction(&format!("jnz {}", throw_label));           // use the standard unwinder when a handler can receive the error
            abi::emit_symbol_address(ctx.emitter, "rsi", &prefix_label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", prefix_len as i64);
            ctx.emitter.instruction("mov edi, 2");                              // write the uncaught dynamic-error prefix to stderr
            ctx.emitter.instruction("mov eax, 1");                              // Linux x86_64 syscall 1 = write
            ctx.emitter.instruction("syscall");                                 // emit the dynamic-error prefix
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", 0);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdx", 8);
            ctx.emitter.instruction("mov edi, 2");                              // write the runtime error message to stderr
            ctx.emitter.instruction("mov eax, 1");                              // Linux x86_64 syscall 1 = write
            ctx.emitter.instruction("syscall");                                 // emit the runtime error message
            abi::emit_symbol_address(ctx.emitter, "rsi", &suffix_label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", suffix_len as i64);
            ctx.emitter.instruction("mov edi, 2");                              // terminate the uncaught diagnostic with a newline
            ctx.emitter.instruction("mov eax, 1");                              // Linux x86_64 syscall 1 = write
            ctx.emitter.instruction("syscall");                                 // emit the dynamic-error suffix
            abi::emit_exit(ctx.emitter, UNCAUGHT_EXIT_STATUS);
        }
    }
    ctx.emitter.label(&throw_label);
}

/// Allocates a built-in `Error` that owns the runtime message stored on the stack.
fn emit_dynamic_error_object(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_int_immediate(ctx.emitter, "x0", 56); // compact Throwable: message/code/previous
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = throwable object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            ctx.emitter.instruction("bl __rt_object_handle_acquire");           // bind the new object to its PHP object handle
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_spl_error_class_id", 0);
            ctx.emitter.instruction("str x9, [x0]");                            // store the built-in Error class id
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x9", 0);
            ctx.emitter.instruction("str x9, [x0, #8]");                        // store the runtime exception message pointer
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x9", 8);
            ctx.emitter.instruction("str x9, [x0, #16]");                       // store the runtime exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
            crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(ctx.emitter, "x0");
            ctx.emitter.instruction("str xzr, [x0, #40]");                      // previous defaults to null
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0);
            abi::emit_jump(ctx.emitter, "__rt_throw_current");
        }
        Arch::X86_64 => {
            abi::emit_load_int_immediate(ctx.emitter, "rax", 56); // compact Throwable: message/code/previous
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
            ctx.emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(6))); // stamp the canonical x86_64 heap-kind word (magic + kind 6 throwable)
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            ctx.emitter.instruction("call __rt_object_handle_acquire");         // bind the new object to its PHP object handle
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_spl_error_class_id", 0);
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the built-in Error class id
            abi::emit_load_temporary_stack_slot(ctx.emitter, "r10", 0);
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10");            // store the runtime exception message pointer
            abi::emit_load_temporary_stack_slot(ctx.emitter, "r10", 8);
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r10");           // store the runtime exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
            crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(ctx.emitter, "rax");
            ctx.emitter.instruction("mov QWORD PTR [rax + 40], 0");             // previous defaults to null
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0);
            abi::emit_jump(ctx.emitter, "__rt_throw_current");
        }
    }
}
