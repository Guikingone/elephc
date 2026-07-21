//! Purpose:
//! Emits `__rt_get_cfg_var`, the immutable compiled-master-value reader backing
//! PHP `get_cfg_var`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//! - `crate::codegen_support::runtime::data::fixed` reads `INI_MASTER` to emit the master literals.
//!
//! Key details:
//! - `get_cfg_var` reads the immutable MASTER map, NOT the mutable ini table, so it never
//!   reflects `ini_set` changes and needs no seeding. It is a pure `__rt_str_eq` compare-chain
//!   over the master directives; a match returns an OWNED copy of the master literal and a miss
//!   returns a null string pointer (boxed as PHP `false` by the EIR lowering).

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// The immutable compiled MASTER directive values (measured with `php -n`).
///
/// Separate from `INI_SEED`: `get_cfg_var` returns these regardless of `ini_set`,
/// and every directive absent here (including `date.timezone`) yields `false`.
pub(crate) const INI_MASTER: &[(&str, &str)] = &[
    ("memory_limit", "128M"),
    ("max_execution_time", "0"),
    ("precision", "14"),
    ("serialize_precision", "-1"),
    ("default_charset", "UTF-8"),
    ("display_errors", "1"),
    ("output_buffering", "0"),
];

/// Returns the runtime data symbol name for the master directive key at `index`.
///
/// Shared with the `.ascii` data emitter in `crate::codegen_support::runtime::data::fixed`.
pub(crate) fn ini_master_key_symbol(index: usize) -> String {
    format!("_rt_ini_master_key_{}", index)
}

/// Returns the runtime data symbol name for the master directive value at `index`.
pub(crate) fn ini_master_val_symbol(index: usize) -> String {
    format!("_rt_ini_master_val_{}", index)
}

/// Emits `__rt_get_cfg_var` for every supported target.
///
/// Compares the requested directive name against the immutable MASTER map and returns
/// an owned copy of the matching master literal, or a null string pointer on no match.
pub fn emit_get_cfg_var(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_get_cfg_var_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: get_cfg_var (immutable master map) ---");
    emitter.label_global("__rt_get_cfg_var");

    // Frame layout (32 bytes): [sp,#0]=name ptr, [sp,#8]=name len, [sp,#16]=x29, [sp,#24]=x30.
    emitter.instruction("sub sp, sp, #32");                                     // allocate the get_cfg_var frame
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the new frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the directive name pointer
    emitter.instruction("str x2, [sp, #8]");                                    // save the directive name length

    for (index, (key, _value)) in INI_MASTER.iter().enumerate() {
        emitter.comment(&format!("-- compare master directive: {} --", key));
        abi::emit_symbol_address(emitter, "x3", &ini_master_key_symbol(index));
        emitter.instruction(&format!("mov x4, #{}", key.len()));                // master key literal length
        emitter.instruction("ldr x1, [sp, #0]");                                // ptr_a = requested directive name pointer
        emitter.instruction("ldr x2, [sp, #8]");                                // len_a = requested directive name length
        abi::emit_call_label(emitter, "__rt_str_eq");
        emitter.instruction(&format!("cbnz x0, __rt_get_cfg_var_match_{}", index)); // matched: return this master value
    }

    // -- no master directive matched: return PHP false --
    emitter.instruction("mov x1, #0");                                          // null string pointer signals PHP false
    emitter.instruction("b __rt_get_cfg_var_done");                             // skip the per-directive match blocks

    for (index, (_key, value)) in INI_MASTER.iter().enumerate() {
        emitter.label(&format!("__rt_get_cfg_var_match_{}", index));
        abi::emit_symbol_address(emitter, "x1", &ini_master_val_symbol(index));
        emitter.instruction(&format!("mov x2, #{}", value.len()));              // master value literal length
        abi::emit_call_label(emitter, "__rt_str_persist");
        emitter.instruction("b __rt_get_cfg_var_done");                         // return the owned master value copy
    }

    emitter.label("__rt_get_cfg_var_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate the get_cfg_var frame
    emitter.instruction("ret");                                                 // return the string-or-false result
}

/// Emits `__rt_get_cfg_var` for x86_64 Linux targets.
///
/// System V variant: the directive name arrives in rax/rdx; `__rt_str_eq` takes the
/// pair in rdi/rsi (name) and rdx/rcx (master key), and matches persist the master value.
fn emit_get_cfg_var_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: get_cfg_var (immutable master map) ---");
    emitter.label_global("__rt_get_cfg_var");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 16");                                         // reserve slots for the directive name ptr/len
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the directive name pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the directive name length

    for (index, (key, _value)) in INI_MASTER.iter().enumerate() {
        emitter.comment(&format!("-- compare master directive: {} --", key));
        abi::emit_symbol_address(emitter, "rdx", &ini_master_key_symbol(index));
        emitter.instruction(&format!("mov rcx, {}", key.len()));                // master key literal length
        emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                    // ptr_a = requested directive name pointer
        emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                   // len_a = requested directive name length
        abi::emit_call_label(emitter, "__rt_str_eq");
        emitter.instruction("test rax, rax");                                   // did the requested name equal this master key?
        emitter.instruction(&format!("jnz __rt_get_cfg_var_match_{}", index));  // matched: return this master value
    }

    // -- no master directive matched: return PHP false --
    emitter.instruction("xor eax, eax");                                        // null string pointer signals PHP false
    emitter.instruction("jmp __rt_get_cfg_var_done");                           // skip the per-directive match blocks

    for (index, (_key, value)) in INI_MASTER.iter().enumerate() {
        emitter.label(&format!("__rt_get_cfg_var_match_{}", index));
        abi::emit_symbol_address(emitter, "rdi", &ini_master_val_symbol(index));
        emitter.instruction(&format!("mov rdx, {}", value.len()));              // master value literal length
        abi::emit_call_label(emitter, "__rt_str_persist");
        emitter.instruction("jmp __rt_get_cfg_var_done");                       // return the owned master value copy
    }

    emitter.label("__rt_get_cfg_var_done");
    emitter.instruction("add rsp, 16");                                         // deallocate the get_cfg_var frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the string-or-false result
}
