//! Purpose:
//! Emits runtime warning helpers for undefined integer and string array keys.
//! Formats missing key values while preserving concat scratch state where needed.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()`.
//!
//! Key details:
//! - The helper is warning-only: callers still materialize their own null fallback.
//! - `__rt_itoa` uses `_concat_buf`, so `_concat_off` is restored before returning.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

const UNDEFINED_ARRAY_KEY_PREFIX_LEN: usize = "Warning: Undefined array key ".len();
const UNDEFINED_ARRAY_KEY_QUOTE_LEN: usize = "\"".len();
const UNDEFINED_ARRAY_KEY_SUFFIX_LEN: usize = "\n".len();
const ARRAY_OFFSET_ON_NULL_LEN: usize = "Warning: Trying to access array offset on null\n".len();

/// Emits `__rt_warn_undefined_array_key_int` for the active target.
pub fn emit_undefined_array_key_warning(emitter: &mut Emitter) {
    emit_register_keeping_wrapper(emitter, "__rt_warn_undefined_array_key_int_keep", "__rt_warn_undefined_array_key_int");
    emit_register_keeping_wrapper(emitter, "__rt_warn_array_offset_on_null_keep", "__rt_warn_array_offset_on_null");
    if emitter.target.arch == Arch::X86_64 {
        emit_undefined_array_key_warning_x86_64(emitter);
        emit_undefined_array_key_string_warning_x86_64(emitter);
        emit_array_offset_on_null_warning_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: undefined_array_key_warning ---");
    emitter.label_global("__rt_warn_undefined_array_key_int");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #48");                                     // reserve saved key, concat cursor, and frame linkage
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish a stable runtime warning frame
    emitter.instruction("str x0, [sp, #0]");                                    // save the missing integer key across warning fragments
    abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [x9]");                                       // snapshot concat scratch state before formatting the key
    emitter.instruction("str x10, [sp, #8]");                                   // preserve the concat cursor across itoa

    // -- emit prefix --
    abi::emit_symbol_address(emitter, "x1", "_diag_undefined_array_key_prefix");
    emitter.instruction(&format!("mov x2, #{}", UNDEFINED_ARRAY_KEY_PREFIX_LEN)); // pass the undefined-key warning prefix length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the undefined-key warning prefix

    // -- emit formatted key --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the missing integer key for decimal formatting
    abi::emit_call_label(emitter, "__rt_itoa");                                 // format the missing key into concat scratch
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the formatted missing-key value
    emitter.instruction("ldr x10, [sp, #8]");                                   // reload the pre-warning concat cursor
    abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("str x10, [x9]");                                       // restore concat scratch state for surrounding expressions

    // -- emit suffix --
    abi::emit_symbol_address(emitter, "x1", "_diag_undefined_array_key_suffix");
    emitter.instruction(&format!("mov x2, #{}", UNDEFINED_ARRAY_KEY_SUFFIX_LEN)); // pass the undefined-key warning suffix length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the undefined-key warning suffix

    // -- restore stack frame --
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the runtime warning frame
    emitter.instruction("ret");                                                 // return to the array-miss caller

    emit_undefined_array_key_string_warning_aarch64(emitter);
    emit_array_offset_on_null_warning_aarch64(emitter);
}

/// Emits the fixed PHP warning used when an array-offset receiver is null on AArch64.
fn emit_array_offset_on_null_warning_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_offset_on_null_warning ---");
    emitter.label_global("__rt_warn_array_offset_on_null");
    emitter.instruction("stp x29, x30, [sp, #-16]!");                           // preserve frame linkage across the diagnostic call
    emitter.instruction("mov x29, sp");                                         // establish a stable warning helper frame
    abi::emit_symbol_address(emitter, "x1", "_diag_array_offset_on_null");
    emitter.instruction(&format!("mov x2, #{}", ARRAY_OFFSET_ON_NULL_LEN));     // pass the complete array-offset-on-null warning length
    abi::emit_call_label(emitter, "__rt_diag_warning");                       // emit or suppress the PHP warning
    emitter.instruction("ldp x29, x30, [sp], #16");                             // restore frame linkage
    emitter.instruction("ret");                                                 // return to the null-receiver fallback
}

/// Emits the fixed PHP warning used when an array-offset receiver is null on x86_64.
fn emit_array_offset_on_null_warning_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_offset_on_null_warning ---");
    emitter.label_global("__rt_warn_array_offset_on_null");
    emitter.instruction("push rbp");                                            // preserve the caller frame and align the diagnostic call
    emitter.instruction("mov rbp, rsp");                                        // establish a stable warning helper frame
    abi::emit_symbol_address(emitter, "rdi", "_diag_array_offset_on_null");
    emitter.instruction(&format!("mov esi, {}", ARRAY_OFFSET_ON_NULL_LEN));     // pass the complete array-offset-on-null warning length
    abi::emit_call_label(emitter, "__rt_diag_warning");                       // emit or suppress the PHP warning
    emitter.instruction("mov rsp, rbp");                                        // release the warning helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the null-receiver fallback
}

/// Emits the x86_64 implementation of `__rt_warn_undefined_array_key_int`.
fn emit_undefined_array_key_warning_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: undefined_array_key_warning ---");
    emitter.label_global("__rt_warn_undefined_array_key_int");

    // -- set up stack frame --
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable runtime warning frame
    emitter.instruction("sub rsp, 32");                                         // reserve saved key and concat cursor while keeping calls aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the missing integer key across warning fragments
    abi::emit_load_symbol_to_reg(emitter, "r10", "_concat_off", 0);             // snapshot concat scratch state before formatting the key
    emitter.instruction("mov QWORD PTR [rbp - 16], r10");                       // preserve the concat cursor across itoa

    // -- emit prefix --
    abi::emit_symbol_address(emitter, "rdi", "_diag_undefined_array_key_prefix");
    emitter.instruction(&format!("mov esi, {}", UNDEFINED_ARRAY_KEY_PREFIX_LEN)); // pass the undefined-key warning prefix length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the undefined-key warning prefix

    // -- emit formatted key --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the missing integer key for decimal formatting
    abi::emit_call_label(emitter, "__rt_itoa");                                 // format the missing key into concat scratch
    emitter.instruction("mov rdi, rax");                                        // pass the formatted missing-key pointer to the warning helper
    emitter.instruction("mov rsi, rdx");                                        // pass the formatted missing-key length to the warning helper
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the formatted missing-key value
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                       // reload the pre-warning concat cursor
    abi::emit_store_reg_to_symbol(emitter, "r10", "_concat_off", 0);            // restore concat scratch state for surrounding expressions

    // -- emit suffix --
    abi::emit_symbol_address(emitter, "rdi", "_diag_undefined_array_key_suffix");
    emitter.instruction(&format!("mov esi, {}", UNDEFINED_ARRAY_KEY_SUFFIX_LEN)); // pass the undefined-key warning suffix length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the undefined-key warning suffix

    // -- restore stack frame --
    emitter.instruction("mov rsp, rbp");                                        // release the runtime warning frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the array-miss caller
}

/// Emits the ARM64 implementation of `__rt_warn_undefined_array_key_str`.
fn emit_undefined_array_key_string_warning_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: undefined_array_key_string_warning ---");
    emitter.label_global("__rt_warn_undefined_array_key_str");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #48");                                     // reserve saved string key and frame linkage
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish a stable runtime warning frame
    emitter.instruction("str x1, [sp, #0]");                                    // save the missing string key pointer across warning fragments
    emitter.instruction("str x2, [sp, #8]");                                    // save the missing string key length across warning fragments

    // -- emit prefix --
    abi::emit_symbol_address(emitter, "x1", "_diag_undefined_array_key_prefix");
    emitter.instruction(&format!("mov x2, #{}", UNDEFINED_ARRAY_KEY_PREFIX_LEN)); // pass the undefined-key warning prefix length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the undefined-key warning prefix

    // -- emit quoted string key --
    abi::emit_symbol_address(emitter, "x1", "_diag_undefined_array_key_quote");
    emitter.instruction(&format!("mov x2, #{}", UNDEFINED_ARRAY_KEY_QUOTE_LEN)); // pass the opening quote length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the opening quote
    emitter.instruction("ldr x1, [sp, #0]");                                    // reload the missing string key pointer
    emitter.instruction("ldr x2, [sp, #8]");                                    // reload the missing string key length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the missing string key bytes
    abi::emit_symbol_address(emitter, "x1", "_diag_undefined_array_key_quote");
    emitter.instruction(&format!("mov x2, #{}", UNDEFINED_ARRAY_KEY_QUOTE_LEN)); // pass the closing quote length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the closing quote

    // -- emit suffix --
    abi::emit_symbol_address(emitter, "x1", "_diag_undefined_array_key_suffix");
    emitter.instruction(&format!("mov x2, #{}", UNDEFINED_ARRAY_KEY_SUFFIX_LEN)); // pass the undefined-key warning suffix length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the undefined-key warning suffix

    // -- restore stack frame --
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the runtime warning frame
    emitter.instruction("ret");                                                 // return to the array-miss caller
}

/// Emits the x86_64 implementation of `__rt_warn_undefined_array_key_str`.
fn emit_undefined_array_key_string_warning_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: undefined_array_key_string_warning ---");
    emitter.label_global("__rt_warn_undefined_array_key_str");

    // -- set up stack frame --
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable runtime warning frame
    emitter.instruction("sub rsp, 32");                                         // reserve saved key pointer and length while keeping calls aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the missing string key pointer across warning fragments
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the missing string key length across warning fragments

    // -- emit prefix --
    abi::emit_symbol_address(emitter, "rdi", "_diag_undefined_array_key_prefix");
    emitter.instruction(&format!("mov esi, {}", UNDEFINED_ARRAY_KEY_PREFIX_LEN)); // pass the undefined-key warning prefix length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the undefined-key warning prefix

    // -- emit quoted string key --
    abi::emit_symbol_address(emitter, "rdi", "_diag_undefined_array_key_quote");
    emitter.instruction(&format!("mov esi, {}", UNDEFINED_ARRAY_KEY_QUOTE_LEN)); // pass the opening quote length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the opening quote
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the missing string key pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // reload the missing string key length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the missing string key bytes
    abi::emit_symbol_address(emitter, "rdi", "_diag_undefined_array_key_quote");
    emitter.instruction(&format!("mov esi, {}", UNDEFINED_ARRAY_KEY_QUOTE_LEN)); // pass the closing quote length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the closing quote

    // -- emit suffix --
    abi::emit_symbol_address(emitter, "rdi", "_diag_undefined_array_key_suffix");
    emitter.instruction(&format!("mov esi, {}", UNDEFINED_ARRAY_KEY_SUFFIX_LEN)); // pass the undefined-key warning suffix length
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the undefined-key warning suffix

    // -- restore stack frame --
    emitter.instruction("mov rsp, rbp");                                        // release the runtime warning frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the array-miss caller
}

/// Emits `name`, which calls the warning helper `target` while preserving every caller-saved
/// register the register allocator hands out (x12–x15 and d16–d30 on AArch64; rsi/rdi/r8/r9
/// and xmm2–xmm14 on x86_64). Scalar indexed-array reads call it on their cold miss path, so
/// their hot path needs no call-crossing spills around the read.
fn emit_register_keeping_wrapper(emitter: &mut Emitter, name: &str, target: &str) {
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", name));
    emitter.label_global(name);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("stp x29, x30, [sp, #-176]!");                  // frame for the allocatable caller-saved registers
            emitter.instruction("mov x29, sp");                                 // anchor the wrapper frame
            emitter.instruction("stp x12, x13, [sp, #16]");                     // keep allocated integer registers
            emitter.instruction("stp x14, x15, [sp, #32]");
            for (index, pair) in [(16, 17), (18, 19), (20, 21), (22, 23), (24, 25), (26, 27), (28, 29)].iter().enumerate() {
                emitter.instruction(&format!("stp d{}, d{}, [sp, #{}]", pair.0, pair.1, 48 + index * 16)); // keep allocated float registers
            }
            emitter.instruction("str d30, [sp, #160]");
            emitter.instruction(&format!("bl {}", target));                     // emit the warning (arguments are untouched)
            emitter.instruction("ldp x12, x13, [sp, #16]");                     // restore allocated integer registers
            emitter.instruction("ldp x14, x15, [sp, #32]");
            for (index, pair) in [(16, 17), (18, 19), (20, 21), (22, 23), (24, 25), (26, 27), (28, 29)].iter().enumerate() {
                emitter.instruction(&format!("ldp d{}, d{}, [sp, #{}]", pair.0, pair.1, 48 + index * 16)); // restore allocated float registers
            }
            emitter.instruction("ldr d30, [sp, #160]");
            emitter.instruction("ldp x29, x30, [sp], #176");                    // drop the wrapper frame
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");                                    // keep the caller frame pointer
            emitter.instruction("mov rbp, rsp");                                // anchor the wrapper frame (16-byte aligned)
            emitter.instruction("sub rsp, 240");                                // slots for rsi/rdi/r8/r9 and xmm2–xmm14
            emitter.instruction("mov QWORD PTR [rsp], rsi");                    // keep allocated integer registers
            emitter.instruction("mov QWORD PTR [rsp + 8], rdi");
            emitter.instruction("mov QWORD PTR [rsp + 16], r8");
            emitter.instruction("mov QWORD PTR [rsp + 24], r9");
            for reg in 2..=14 {
                emitter.instruction(&format!("movdqu XMMWORD PTR [rsp + {}], xmm{}", 32 + (reg - 2) * 16, reg)); // keep allocated float registers
            }
            emitter.instruction(&format!("call {}", target));                   // emit the warning (arguments are untouched)
            emitter.instruction("mov rsi, QWORD PTR [rsp]");                    // restore allocated integer registers
            emitter.instruction("mov rdi, QWORD PTR [rsp + 8]");
            emitter.instruction("mov r8, QWORD PTR [rsp + 16]");
            emitter.instruction("mov r9, QWORD PTR [rsp + 24]");
            for reg in 2..=14 {
                emitter.instruction(&format!("movdqu xmm{}, XMMWORD PTR [rsp + {}]", reg, 32 + (reg - 2) * 16)); // restore allocated float registers
            }
            emitter.instruction("leave");                                       // drop the wrapper frame
            emitter.instruction("ret");
        }
    }
}
