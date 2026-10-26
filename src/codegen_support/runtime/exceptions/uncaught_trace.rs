//! Purpose:
//! Emits the stack-trace block of the uncaught exception report: `__rt_uncaught_print_stack_trace`
//! walks the activation chain, `__rt_uncaught_print_stack_header` opens the block, and
//! `__rt_uncaught_print_main` / `__rt_uncaught_print_thrown_in` close it with php's bottom frame
//! and `  thrown in <file> on line <N>` tail.
//!
//! Called from:
//! - `crate::codegen_support::runtime::exceptions::uncaught_report::emit_report_uncaught_exception`,
//!   after the location line and before the exit.
//!
//! Key details:
//! - Each activation record's frame reader prints its own line in the trace format when called
//!   with a non-negative print index (x2/rdx) — the same reader `debug_print_backtrace()` uses —
//!   so the walk is a thin loop over the chain plus three literal writes.
//! - The walk applies the backtrace walker's guards: a record whose self-pointer word (+16) does
//!   not point back at the record is an ownership guard, not an activation, and is skipped; a
//!   record with no reader is skipped.
//! - The bottom entry is always `{main}`: php prints the script frame that way, and elephc's
//!   process entry frame carries no PHP-visible activation.
//! - The `thrown in` tail repeats the location the report already printed: the module's source
//!   path and the throwable's creation line. A zero line still prints `on line 0` — the corpus
//!   forms always carry a real line, and omitting the tail would hide the block php prints.
//! - The walk cursor lives in x19/r14, which AAPCS/SysV callees preserve, and the frame index
//!   lives in the walker's own frame slot, so both survive every reader call.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::sentinels::THROWABLE_CREATION_LINE_OFFSET;

/// Stack-trace state frame size. The frame index slot sits at the lowest scratch offset.
const TRACE_FRAME_BYTES: i64 = 64;
const TRACE_INDEX_SLOT: i64 = 16;

/// Dispatches to the target-specific emitter.
pub fn emit_uncaught_print_stack_trace(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_uncaught_print_stack_trace_x86_64(emitter);
        return;
    }
    emit_uncaught_print_stack_trace_aarch64(emitter);
}

/// Emits the AArch64 stack-trace walker and its three literal writers.
fn emit_uncaught_print_stack_trace_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: uncaught_print_stack_trace ---");
    emitter.label_global("__rt_uncaught_print_stack_trace");

    emitter.instruction("stp x29, x30, [sp, #-64]!");
    emitter.instruction("mov x29, sp");
    emitter.instruction(&format!("str xzr, [sp, #{}]", TRACE_INDEX_SLOT)); // frame index starts at #0

    emitter.instruction("bl __rt_uncaught_print_stack_header"); // write "Stack trace:\n"

    abi::emit_load_symbol_to_reg(emitter, "x20", "_uncaught_trace_snapshot", 0);
    emitter.label("__rt_uncaught_trace_loop");
    emitter.instruction("cbz x20, __rt_uncaught_trace_bottom");
    emitter.instruction("ldr x9, [x20, #16]"); // the activation self-pointer or an owned-value payload
    emitter.instruction("cmp x9, x20"); // distinguish activation records from ownership guards
    emitter.instruction("b.eq __rt_uncaught_trace_skip"); // a guard record self-points; skip it
    emitter.instruction("ldr x10, [x20, #24]"); // the frame's reader callback
    emitter.instruction("cbz x10, __rt_uncaught_trace_skip");
    emitter.instruction("mov x0, x20"); // record
    emitter.instruction("mov x1, #0"); // options: print the arguments
    emitter.instruction(&format!("ldr x2, [sp, #{}]", TRACE_INDEX_SLOT)); // print index
    emitter.instruction("blr x10");
    emitter.instruction(&format!("ldr x19, [sp, #{}]", TRACE_INDEX_SLOT));
    emitter.instruction("add x19, x19, #1");
    emitter.instruction(&format!("str x19, [sp, #{}]", TRACE_INDEX_SLOT));
    emitter.label("__rt_uncaught_trace_skip");
    emitter.instruction("ldr x20, [x20]"); // next record
    emitter.instruction("b __rt_uncaught_trace_loop");

    emitter.label("__rt_uncaught_trace_bottom");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", TRACE_INDEX_SLOT));
    emitter.instruction("bl __rt_uncaught_print_main"); // "#<index> {main}\n"
    emitter.instruction("bl __rt_uncaught_print_thrown_in"); // "  thrown in <file> on line <N>\n"

    emitter.instruction("ldp x29, x30, [sp], #64");
    emitter.instruction("ret");

    // -- "Stack trace:\n" --
    emitter.label("__rt_uncaught_print_stack_header");
    emitter.instruction("stp x29, x30, [sp, #-16]!");
    emitter.instruction("mov x29, sp");
    abi::emit_symbol_address(emitter, "x1", "_uncaught_exc_stack");
    emitter.instruction(&format!("mov x2, #{}", UNCAUGHT_STACK_LEN));
    emitter.instruction("mov x0, #1"); // fd = stdout
    emitter.syscall(4);
    emitter.instruction("ldp x29, x30, [sp], #16");
    emitter.instruction("ret");

    // -- "#<index> {main}\n"; the index arrives in x0 --
    emitter.label("__rt_uncaught_print_main");
    emitter.instruction("stp x29, x30, [sp, #-32]!");
    emitter.instruction("mov x29, sp");
    emitter.instruction("str x0, [sp, #16]"); // the frame index across the itoa call (clear of the saved x30 at +8)
    abi::emit_symbol_address(emitter, "x1", "_uncaught_exc_hash");
    emitter.instruction(&format!("mov x2, #{}", UNCAUGHT_HASH_LEN));
    emitter.instruction("mov x0, #1");
    emitter.syscall(4);
    emitter.instruction("ldr x0, [sp, #16]");
    emitter.instruction("bl __rt_itoa"); // pointer in x1, length in x2
    emitter.instruction("mov x0, #1");
    emitter.syscall(4);
    abi::emit_symbol_address(emitter, "x1", "_uncaught_exc_main");
    emitter.instruction(&format!("mov x2, #{}", UNCAUGHT_MAIN_LEN));
    emitter.instruction("mov x0, #1");
    emitter.syscall(4);
    emitter.instruction("ldp x29, x30, [sp], #32");
    emitter.instruction("ret");

    // -- "  thrown in <file> on line <N>\n" --
    emitter.label("__rt_uncaught_print_thrown_in");
    emitter.instruction("stp x29, x30, [sp, #-48]!");
    emitter.instruction("mov x29, sp");
    emitter.instruction("str xzr, [sp, #16]"); // remember whether the file part was written (clear of the saved x30 at +8)
    abi::emit_symbol_address(emitter, "x1", "_uncaught_exc_thrown");
    emitter.instruction(&format!("mov x2, #{}", UNCAUGHT_THROWN_LEN));
    emitter.instruction("mov x0, #1");
    emitter.syscall(4);
    abi::emit_load_symbol_to_reg(emitter, "x9", "_script_source_file_len", 0);
    emitter.instruction("cbz x9, __rt_uncaught_thrown_line"); // no source path: skip straight to the line
    emitter.instruction("mov w9, #1");
    emitter.instruction("str w9, [sp, #16]");
    abi::emit_symbol_address(emitter, "x1", "_script_source_file");
    abi::emit_load_symbol_to_reg(emitter, "x2", "_script_source_file_len", 0);
    emitter.instruction("mov x0, #1");
    emitter.syscall(4);
    emitter.label("__rt_uncaught_thrown_line");
    abi::emit_symbol_address(emitter, "x1", "_uncaught_exc_online");
    emitter.instruction(&format!("mov x2, #{}", UNCAUGHT_ONLINE_LEN));
    emitter.instruction("mov x0, #1");
    emitter.syscall(4);
    abi::emit_load_symbol_to_reg(emitter, "x9", "_exc_value", 0);
    emitter.instruction(&format!("ldr x0, [x9, #{}]", THROWABLE_CREATION_LINE_OFFSET));
    emitter.instruction("bl __rt_itoa");
    emitter.instruction("mov x0, #1");
    emitter.syscall(4);
    abi::emit_symbol_address(emitter, "x1", "_uncaught_exc_nl");
    emitter.instruction(&format!("mov x2, #{}", UNCAUGHT_NEWLINE_LEN));
    emitter.instruction("mov x0, #1");
    emitter.syscall(4);
    emitter.instruction("ldp x29, x30, [sp], #48");
    emitter.instruction("ret");
}

/// Emits the Linux x86_64 stack-trace walker and its three literal writers (System V AMD64).
fn emit_uncaught_print_stack_trace_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: uncaught_print_stack_trace ---");
    emitter.label_global("__rt_uncaught_print_stack_trace");

    emitter.instruction("push rbp");
    emitter.instruction("mov rbp, rsp");
    emitter.instruction(&format!("sub rsp, {}", TRACE_FRAME_BYTES));
    emitter.instruction(&format!(
        "mov QWORD PTR [rbp - {}], 0",
        TRACE_FRAME_BYTES - TRACE_INDEX_SLOT
    )); // frame index starts at #0

    emitter.instruction("call __rt_uncaught_print_stack_header");

    abi::emit_load_symbol_to_reg(emitter, "r14", "_uncaught_trace_snapshot", 0);
    emitter.label("__rt_uncaught_trace_loop_x86");
    emitter.instruction("test r14, r14");
    emitter.instruction("jz __rt_uncaught_trace_bottom_x86");
    emitter.instruction("cmp QWORD PTR [r14 + 16], r14"); // activation record vs ownership guard
    emitter.instruction("je __rt_uncaught_trace_skip_x86"); // a guard record self-points; skip it
    emitter.instruction("mov r10, QWORD PTR [r14 + 24]"); // the frame's reader callback
    emitter.instruction("test r10, r10");
    emitter.instruction("jz __rt_uncaught_trace_skip_x86");
    emitter.instruction("mov rdi, r14"); // record
    emitter.instruction("mov rsi, 0"); // options: print the arguments
    emitter.instruction(&format!(
        "mov rdx, QWORD PTR [rbp - {}]",
        TRACE_FRAME_BYTES - TRACE_INDEX_SLOT
    )); // print index
    emitter.instruction("call r10");
    emitter.instruction(&format!(
        "mov rax, QWORD PTR [rbp - {}]",
        TRACE_FRAME_BYTES - TRACE_INDEX_SLOT
    ));
    emitter.instruction("add rax, 1");
    emitter.instruction(&format!(
        "mov QWORD PTR [rbp - {}], rax",
        TRACE_FRAME_BYTES - TRACE_INDEX_SLOT
    ));
    emitter.label("__rt_uncaught_trace_skip_x86");
    emitter.instruction("mov r14, QWORD PTR [r14]"); // next record
    emitter.instruction("jmp __rt_uncaught_trace_loop_x86");

    emitter.label("__rt_uncaught_trace_bottom_x86");
    emitter.instruction(&format!(
        "mov rdi, QWORD PTR [rbp - {}]",
        TRACE_FRAME_BYTES - TRACE_INDEX_SLOT
    ));
    emitter.instruction("call __rt_uncaught_print_main");
    emitter.instruction("call __rt_uncaught_print_thrown_in");

    emitter.instruction(&format!("add rsp, {}", TRACE_FRAME_BYTES));
    emitter.instruction("pop rbp");
    emitter.instruction("ret");

    // -- "Stack trace:\n" --
    emitter.label("__rt_uncaught_print_stack_header");
    emitter.instruction("push rbp");
    emitter.instruction("mov rbp, rsp");
    abi::emit_symbol_address(emitter, "rsi", "_uncaught_exc_stack");
    emitter.instruction(&format!("mov edx, {}", UNCAUGHT_STACK_LEN));
    emitter.instruction("mov edi, 1"); // fd = stdout
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");

    // -- "#<index> {main}\n"; the index arrives in rdi --
    emitter.label("__rt_uncaught_print_main");
    emitter.instruction("push rbp");
    emitter.instruction("mov rbp, rsp");
    emitter.instruction("sub rsp, 16"); // an aligned slot for the frame index across the itoa call
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");
    abi::emit_symbol_address(emitter, "rsi", "_uncaught_exc_hash");
    emitter.instruction(&format!("mov edx, {}", UNCAUGHT_HASH_LEN));
    emitter.instruction("mov edi, 1");
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("call __rt_itoa"); // pointer in rsi, length in rdx
    emitter.instruction("mov edi, 1");
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    abi::emit_symbol_address(emitter, "rsi", "_uncaught_exc_main");
    emitter.instruction(&format!("mov edx, {}", UNCAUGHT_MAIN_LEN));
    emitter.instruction("mov edi, 1");
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    emitter.instruction("add rsp, 16");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");

    // -- "  thrown in <file> on line <N>\n" --
    emitter.label("__rt_uncaught_print_thrown_in");
    emitter.instruction("push rbp");
    emitter.instruction("mov rbp, rsp");
    emitter.instruction("sub rsp, 16");
    emitter.instruction("mov QWORD PTR [rbp - 8], 0"); // remember whether the file part was written
    abi::emit_symbol_address(emitter, "rsi", "_uncaught_exc_thrown");
    emitter.instruction(&format!("mov edx, {}", UNCAUGHT_THROWN_LEN));
    emitter.instruction("mov edi, 1");
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    abi::emit_load_symbol_to_reg(emitter, "r9", "_script_source_file_len", 0);
    emitter.instruction("test r9, r9");
    emitter.instruction("jz __rt_uncaught_thrown_line_x86"); // no source path: skip straight to the line
    emitter.instruction("mov QWORD PTR [rbp - 8], 1");
    abi::emit_symbol_address(emitter, "rsi", "_script_source_file");
    abi::emit_load_symbol_to_reg(emitter, "rdx", "_script_source_file_len", 0);
    emitter.instruction("mov edi, 1");
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    emitter.label("__rt_uncaught_thrown_line_x86");
    abi::emit_symbol_address(emitter, "rsi", "_uncaught_exc_online");
    emitter.instruction(&format!("mov edx, {}", UNCAUGHT_ONLINE_LEN));
    emitter.instruction("mov edi, 1");
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    abi::emit_load_symbol_to_reg(emitter, "r9", "_exc_value", 0);
    emitter.instruction(&format!("mov rdi, QWORD PTR [r9 + {}]", THROWABLE_CREATION_LINE_OFFSET));
    emitter.instruction("call __rt_itoa");
    emitter.instruction("mov edi, 1");
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    abi::emit_symbol_address(emitter, "rsi", "_uncaught_exc_nl");
    emitter.instruction(&format!("mov edx, {}", UNCAUGHT_NEWLINE_LEN));
    emitter.instruction("mov edi, 1");
    emitter.instruction("mov eax, 1"); // syscall 1 = write
    emitter.instruction("syscall");
    emitter.instruction("add rsp, 16");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");
}

/// Byte length of "Stack trace:\n".
const UNCAUGHT_STACK_LEN: i64 = 13;
/// Byte length of "#".
const UNCAUGHT_HASH_LEN: i64 = 1;
/// Byte length of " {main}\n".
const UNCAUGHT_MAIN_LEN: i64 = 8;
/// Byte length of "  thrown in ".
const UNCAUGHT_THROWN_LEN: i64 = 12;
/// Byte length of " on line ".
const UNCAUGHT_ONLINE_LEN: i64 = 9;
/// Byte length of "\n", shared with the report's own terminator symbol.
const UNCAUGHT_NEWLINE_LEN: i64 = 1;
