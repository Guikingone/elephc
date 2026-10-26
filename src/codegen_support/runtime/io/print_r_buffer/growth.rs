//! Purpose:
//! Grows the owned print_r capture buffer without truncating rendered PHP values.
//! Preserves every byte and transfers storage through the ordinary string ABI.
//!
//! Called from:
//! - `super::emit_pr_append()` through the runtime platform emitter.
//!
//! Key details:
//! - Active capture globals own one string allocation; growth transfers that owner.
//! - Finish adopts the buffer, so its capacity includes a trailing zero byte.
//! - Byte copying uses only caller-saved registers on every supported target.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

const FRAME: usize = 80;
const SOURCE: usize = 8;
const LENGTH: usize = 16;
const OLD: usize = 24;
const OFFSET: usize = 32;
const CAPACITY: usize = 40;
const NEW: usize = 48;

/// Emits an append operation whose growable storage owns all captured bytes.
pub fn emit_pr_append(emitter: &mut Emitter) {
    emitter.blank();
    emitter.label_global("__rt_pr_append");
    abi::emit_frame_prologue(emitter, FRAME);
    abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 0), SOURCE);
    abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 1), LENGTH);
    let result = abi::int_result_reg(emitter);
    for (symbol, slot) in [("_print_r_buf", OLD), ("_print_r_off", OFFSET), ("_print_r_cap", CAPACITY)] {
        abi::emit_load_symbol_to_reg(emitter, result, symbol, 0);
        abi::store_at_offset(emitter, result, slot);
    }
    match emitter.target.arch {
        Arch::AArch64 => {
            abi::load_at_offset(emitter, "x9", OFFSET);
            abi::load_at_offset(emitter, "x10", LENGTH);
            abi::load_at_offset(emitter, "x11", CAPACITY);
            emitter.instruction("add x9, x9, x10");                             // compute the new byte count
            emitter.instruction("add x9, x9, #1");                              // include the terminator in the required capacity
            emitter.instruction("cmp x11, x9");                                 // does the active allocation have enough space?
            emitter.instruction("b.hs __rt_pr_append_copy");                    // reuse its existing owner when no growth is needed
            emitter.instruction("lsl x10, x11, #1");                            // prefer amortized geometric growth
            emitter.instruction("cmp x10, x9");                                 // a large write may exceed the doubled capacity
            emitter.instruction("csel x10, x10, x9, hs");                       // include every requested byte without truncation
            emitter.instruction("mov x12, #256");                               // start small captures with a useful minimum capacity
            emitter.instruction("cmp x10, x12");                                // compare the request with the initial capacity
            emitter.instruction("csel x0, x10, x12, hs");                       // pass the larger allocation size to the heap
        }
        Arch::X86_64 => {
            abi::load_at_offset(emitter, "rax", OFFSET);
            abi::load_at_offset(emitter, "r10", LENGTH);
            abi::load_at_offset(emitter, "r11", CAPACITY);
            emitter.instruction("add rax, r10");                                // compute the new byte count
            emitter.instruction("add rax, 1");                                  // include the terminator in the required capacity
            emitter.instruction("cmp r11, rax");                                // does the active allocation have enough space?
            emitter.instruction("jae __rt_pr_append_copy");                     // reuse its existing owner when no growth is needed
            emitter.instruction("lea r10, [r11 + r11]");                        // prefer amortized geometric growth
            emitter.instruction("cmp r10, rax");                                // a large write may exceed the doubled capacity
            emitter.instruction("cmovb r10, rax");                              // include every requested byte without truncation
            emitter.instruction("mov rax, 256");                                // start small captures with a useful minimum capacity
            emitter.instruction("cmp r10, rax");                                // compare the request with the initial capacity
            emitter.instruction("cmovae rax, r10");                             // pass the larger allocation size to the heap
        }
    }
    abi::store_at_offset(emitter, result, CAPACITY);
    abi::emit_call_label(emitter, "__rt_heap_alloc");
    stamp_owned_buffer(emitter);
    abi::store_at_offset(emitter, result, NEW);
    load_copy_arguments(emitter, NEW, OLD, OFFSET);
    abi::emit_call_label(emitter, "__rt_pr_copy_bytes");
    abi::load_at_offset(emitter, result, OLD);
    abi::emit_call_label(emitter, "__rt_decref_any");
    abi::load_at_offset(emitter, result, NEW);
    abi::emit_store_reg_to_symbol(emitter, result, "_print_r_buf", 0);
    abi::load_at_offset(emitter, result, CAPACITY);
    abi::emit_store_reg_to_symbol(emitter, result, "_print_r_cap", 0);
    emitter.label("__rt_pr_append_copy");
    let destination = abi::int_arg_reg_name(emitter.target, 0);
    abi::emit_load_symbol_to_reg(emitter, destination, "_print_r_buf", 0);
    match emitter.target.arch {
        Arch::AArch64 => {
            abi::load_at_offset(emitter, "x9", OFFSET);
            emitter.instruction("add x0, x0, x9");                              // append after the bytes already captured
        }
        Arch::X86_64 => {
            abi::load_at_offset(emitter, "r11", OFFSET);
            emitter.instruction("add rdi, r11");                                // append after the bytes already captured
        }
    }
    abi::load_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 1), SOURCE);
    abi::load_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 2), LENGTH);
    abi::emit_call_label(emitter, "__rt_pr_copy_bytes");
    match emitter.target.arch {
        Arch::AArch64 => {
            abi::load_at_offset(emitter, "x0", OFFSET);
            abi::load_at_offset(emitter, "x10", LENGTH);
            emitter.instruction("add x0, x0, x10");                             // commit the complete captured byte count
        }
        Arch::X86_64 => {
            abi::load_at_offset(emitter, "rax", OFFSET);
            abi::load_at_offset(emitter, "r10", LENGTH);
            emitter.instruction("add rax, r10");                                // commit the complete captured byte count
        }
    }
    abi::emit_store_reg_to_symbol(emitter, result, "_print_r_off", 0);
    abi::emit_load_symbol_to_reg(emitter, abi::secondary_scratch_reg(emitter), "_print_r_buf", 0);
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction("strb wzr, [x10, x0]"),            // terminate the captured string without counting that byte
        Arch::X86_64 => emitter.instruction("mov BYTE PTR [r10 + rax], 0"),     // terminate the captured string without counting that byte
    }
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
    emit_copy_bytes(emitter);
}

/// Marks a freshly allocated byte buffer as an independently owned string leaf.
fn stamp_owned_buffer(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("mov x9, #1");                                  // use the ordinary string heap kind for captured bytes
            emitter.instruction("str x9, [x0, #-8]");                           // publish a leaf allocation that cleanup may release
        }
        Arch::X86_64 => {
            let kind = crate::codegen_support::sentinels::x86_64_heap_kind_word(1);
            emitter.instruction(&format!("mov r10, 0x{kind:x}"));               // preserve the x86 heap-kind marker and string tag
            emitter.instruction("mov QWORD PTR [rax - 8], r10");                // publish a leaf allocation that cleanup may release
        }
    }
}

/// Loads the three saved operands without exposing architecture-specific argument registers.
fn load_copy_arguments(emitter: &mut Emitter, destination: usize, source: usize, length: usize) {
    for (index, slot) in [(0, destination), (1, source), (2, length)] {
        abi::load_at_offset(emitter, abi::int_arg_reg_name(emitter.target, index), slot);
    }
}

/// Copies arbitrary bytes without libc ABI adapters or touching callee-saved registers.
fn emit_copy_bytes(emitter: &mut Emitter) {
    emitter.blank();
    emitter.label_global("__rt_pr_copy_bytes");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("cbz x2, __rt_pr_copy_bytes_done");             // an empty copy permits a null source pointer
            emitter.label("__rt_pr_copy_bytes_loop");
            emitter.instruction("ldrb w9, [x1], #1");                           // load the next captured byte and advance the source
            emitter.instruction("strb w9, [x0], #1");                           // store the byte and advance the destination
            emitter.instruction("subs x2, x2, #1");                             // count the remaining bytes
            emitter.instruction("b.ne __rt_pr_copy_bytes_loop");                // continue until all requested bytes were copied
        }
        Arch::X86_64 => {
            emitter.instruction("test rdx, rdx");                               // an empty copy permits a null source pointer
            emitter.instruction("jz __rt_pr_copy_bytes_done");                  // avoid dereferencing either empty operand
            emitter.label("__rt_pr_copy_bytes_loop");
            emitter.instruction("mov r11b, BYTE PTR [rsi]");                    // load the next captured byte
            emitter.instruction("mov BYTE PTR [rdi], r11b");                    // store the byte at the destination
            emitter.instruction("inc rsi");                                     // advance the source cursor
            emitter.instruction("inc rdi");                                     // advance the destination cursor
            emitter.instruction("dec rdx");                                     // count the remaining bytes
            emitter.instruction("jnz __rt_pr_copy_bytes_loop");                 // continue until all requested bytes were copied
        }
    }
    emitter.label("__rt_pr_copy_bytes_done");
    abi::emit_return(emitter);
}
