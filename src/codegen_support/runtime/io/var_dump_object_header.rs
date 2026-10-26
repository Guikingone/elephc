//! Purpose:
//! Emits an object header from the stable identity captured before a debug hook.
//!
//! Called from:
//! - The runtime object-view renderer after invoking the object's debug hook.
//!
//! Key details:
//! - The receiver may already be destroyed; this helper never reads its allocation.
//! - Indentation precedes the hook and is deliberately absent from this header.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits `object(CLASS)#HANDLE (COUNT)` from three C ABI scalar arguments.
pub fn emit_var_dump_open_object_info(emitter: &mut Emitter) {
    const FRAME: usize = 48;
    let result = abi::int_result_reg(emitter);
    let pointer = abi::symbol_scratch_reg(emitter);
    emitter.blank();
    emitter.comment("--- runtime: stable var_dump object header ---");
    emitter.label_global("__rt_var_dump_open_object_info");
    abi::emit_frame_prologue(emitter, FRAME);
    for (index, slot) in [(0, 8), (1, 16), (2, 24)] {
        abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, index), slot);
    }
    write_literal(emitter, "_vd_object_prefix", 7);
    abi::load_at_offset(emitter, result, 8);
    abi::emit_cmp_reg_to_symbol(emitter, result, "_class_name_count");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("b.hs __rt_vd_open_info_anon");                 // unknown class ids have an empty display name
        }
        Arch::X86_64 => {
            emitter.instruction("jae __rt_vd_open_info_anon");                  // unknown class ids have an empty display name
        }
    }
    abi::emit_symbol_address(emitter, pointer, "_class_name_entries");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("add x9, x9, x0, lsl #4");                      // each immutable name entry contains two words
            emitter.instruction("ldr x1, [x9]");                                // pass the stable class-name bytes to the output sink
            emitter.instruction("ldr x2, [x9, #8]");                            // pass the stable class-name length
        }
        Arch::X86_64 => {
            emitter.instruction("shl rax, 4");                                  // each immutable name entry contains two words
            emitter.instruction("add r11, rax");                                // select the captured class id's immutable name entry
            emitter.instruction("mov rsi, QWORD PTR [r11]");                    // pass the stable class-name bytes to the output sink
            emitter.instruction("mov rdx, QWORD PTR [r11 + 8]");                // pass the stable class-name length
        }
    }
    abi::emit_call_label(emitter, "__rt_vd_write");
    emitter.label("__rt_vd_open_info_anon");
    write_literal(emitter, "_vd_object_mid", 2);
    write_number(emitter, 16);
    write_literal(emitter, "_vd_object_count_open", 2);
    write_number(emitter, 24);
    write_literal(emitter, "_vd_brace_open", 4);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
}

/// Writes one immutable header fragment through the ordinary var_dump output sink.
fn write_literal(emitter: &mut Emitter, symbol: &str, length: i64) {
    let pointer = match emitter.target.arch { Arch::AArch64 => "x1", Arch::X86_64 => "rsi" };
    let count = match emitter.target.arch { Arch::AArch64 => "x2", Arch::X86_64 => "rdx" };
    abi::emit_symbol_address(emitter, pointer, symbol);
    abi::emit_load_int_immediate(emitter, count, length);
    abi::emit_call_label(emitter, "__rt_vd_write");
}

/// Formats a captured scalar without consulting the original object's allocation.
fn write_number(emitter: &mut Emitter, slot: usize) {
    abi::load_at_offset(emitter, abi::int_result_reg(emitter), slot);
    abi::emit_call_label(emitter, "__rt_itoa");
    if emitter.target.arch == Arch::X86_64 {
        abi::emit_reg_move(emitter, "rsi", "rax");
    }
    abi::emit_call_label(emitter, "__rt_vd_write");
}
