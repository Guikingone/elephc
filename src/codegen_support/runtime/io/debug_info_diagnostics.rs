//! Purpose:
//! Emits diagnostics required by the native magic-debug view protocol.
//! Null-return notices share ordinary PHP handler dispatch and error reporting.
//!
//! Called from:
//! - The var_dump object-view emitter after its getter returns.
//!
//! Key details:
//! - Warning fragments retain one complete diagnostic and its PHP deprecation level.
//! - The caller guards warning delivery so an exception can follow the closing brace.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

const PREFIX: &str = "Deprecated: Returning null from ";
const SUFFIX: &str = "::__debugInfo() is deprecated, return an empty array instead\n";
const INVALID: &str = "Fatal error: __debuginfo() must return an array\n";

/// Emits the null-return warning and the uncatchable invalid-result boundary.
pub fn emit_debug_info_diagnostics(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let base = abi::symbol_scratch_reg(emitter);
    let index = abi::secondary_scratch_reg(emitter);
    let (ptr, len) = diagnostic_registers(emitter);
    emitter.label_global("__rt_debug_info_warn_null");
    abi::emit_frame_prologue(emitter, 32);
    abi::store_at_offset(emitter, result, 8);
    fragment(emitter, "_vd_debug_null_prefix", PREFIX.len(), false);
    abi::emit_symbol_address(emitter, base, "_class_name_entries");
    abi::load_at_offset(emitter, index, 8);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("add {base}, {base}, {index}, lsl #4"));// select the captured class-name pair without reading a dead receiver
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("shl {index}, 4"));                    // scale the class id to its pointer/length metadata pair
            emitter.instruction(&format!("add {base}, {index}"));               // select the captured class-name pair
        }
    }
    abi::emit_load_from_address(emitter, ptr, base, 0);
    abi::emit_load_from_address(emitter, len, base, 8);
    abi::emit_call_label(emitter, "__rt_diag_warning_fragment");
    fragment(emitter, "_vd_debug_null_suffix", SUFFIX.len(), true);
    abi::emit_frame_restore(emitter, 32);
    abi::emit_return(emitter);

    emitter.label_global("__rt_debug_info_invalid_result");
    abi::emit_load_symbol_to_reg(emitter, result, "_php_error_reporting", 0);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("tst x0, #1");                                  // respect the PHP E_ERROR reporting mask before publishing text
            emitter.instruction("b.eq __rt_debug_info_invalid_exit");           // suppressed fatals still terminate with the PHP status
        }
        Arch::X86_64 => {
            emitter.instruction("test rax, 1");                                 // respect the PHP E_ERROR reporting mask before publishing text
            emitter.instruction("je __rt_debug_info_invalid_exit");             // suppressed fatals still terminate with the PHP status
        }
    }

    abi::emit_store_zero_to_symbol(emitter, "_rt_diag_suppression", 0);
    abi::emit_symbol_address(emitter, ptr, "_vd_debug_invalid_result");
    abi::emit_load_int_immediate(emitter, len, INVALID.len() as i64);
    abi::emit_call_label(emitter, "__rt_diag_write");
    emitter.label("__rt_debug_info_invalid_exit");
    abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(emitter.target, 0), 255);
    abi::emit_call_label(emitter, &emitter.target.extern_symbol("exit"));
    abi::emit_return(emitter);

    emitter.raw(".data");
    for (symbol, text) in [
        ("_vd_debug_null_prefix", PREFIX), ("_vd_debug_null_suffix", SUFFIX),
        ("_vd_debug_invalid_result", INVALID),
    ] {
        emitter.label(symbol);
        emitter.raw(&format!("    .ascii {text:?}"));
    }
    emitter.raw(".text");
}

/// Returns the historical diagnostic writer's pointer and byte-length registers.
fn diagnostic_registers(emitter: &Emitter) -> (&'static str, &'static str) {
    match emitter.target.arch {
        Arch::AArch64 => ("x1", "x2"),
        Arch::X86_64 => ("rdi", "rsi"),
    }
}

/// Adds one static fragment, optionally finishing the joined warning for handler delivery.
fn fragment(emitter: &mut Emitter, symbol: &str, length: usize, complete: bool) {
    let (ptr, len) = diagnostic_registers(emitter);
    abi::emit_symbol_address(emitter, ptr, symbol);
    abi::emit_load_int_immediate(emitter, len, length as i64);
    abi::emit_call_label(emitter, if complete { "__rt_diag_warning" } else { "__rt_diag_warning_fragment" });
}
