//! Purpose:
//! Emits `__rt_ini_get`, the persistent-ini-table reader backing PHP `ini_get`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - Input: directive name in the string result registers (x1/x2, or rax/rdx on x86_64).
//!   Output: an OWNED PHP string (ptr/len) for a seeded directive, or a null string pointer
//!   for an unset directive (boxed as PHP `false` by the EIR lowering).
//! - The hash returns a BORROWED value, so `ini_get` MUST `__rt_str_persist` an independent
//!   copy: a later `ini_set` overwrite releases the table's copy, so returning the borrowed
//!   pointer would dangle (UAF). This is the critical ownership rule for the reader.

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Emits `__rt_ini_get` for every supported target.
///
/// Ensures the table is seeded, looks the directive up by name, and returns an
/// independent owned copy of a found value or a null string pointer on a miss.
pub fn emit_ini_get(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ini_get_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ini_get ---");
    emitter.label_global("__rt_ini_get");

    // Frame layout (32 bytes): [sp,#0]=name ptr, [sp,#8]=name len, [sp,#16]=x29, [sp,#24]=x30.
    emitter.instruction("sub sp, sp, #32");                                     // allocate the ini_get frame
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the new frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the directive name pointer
    emitter.instruction("str x2, [sp, #8]");                                    // save the directive name length

    abi::emit_call_label(emitter, "__rt_ini_table_ensure");
    abi::emit_load_symbol_to_reg(emitter, "x0", "_rt_ini_table", 0);
    emitter.instruction("ldr x1, [sp, #0]");                                    // key_lo = directive name pointer
    emitter.instruction("ldr x2, [sp, #8]");                                    // key_hi = directive name length
    abi::emit_call_label(emitter, "__rt_hash_get");
    emitter.instruction("cbz x0, __rt_ini_get_miss");                           // a lookup miss returns PHP false
    abi::emit_call_label(emitter, "__rt_str_persist");
    emitter.instruction("b __rt_ini_get_done");                                 // return the independent owned copy

    emitter.label("__rt_ini_get_miss");
    emitter.instruction("mov x1, #0");                                          // null string pointer signals PHP false

    emitter.label("__rt_ini_get_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate the ini_get frame
    emitter.instruction("ret");                                                 // return the string-or-false result
}

/// Emits `__rt_ini_get` for x86_64 Linux targets.
///
/// System V variant: the directive name arrives in rax/rdx, the hash value comes
/// back in rdi/rsi, and `__rt_str_persist` (rdi-input) produces the owned copy in rax/rdx.
fn emit_ini_get_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ini_get ---");
    emitter.label_global("__rt_ini_get");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 16");                                         // reserve slots for the directive name ptr/len
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the directive name pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the directive name length

    abi::emit_call_label(emitter, "__rt_ini_table_ensure");
    abi::emit_load_symbol_to_reg(emitter, "rdi", "_rt_ini_table", 0);
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // key_lo = directive name pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // key_hi = directive name length
    abi::emit_call_label(emitter, "__rt_hash_get");
    emitter.instruction("test rax, rax");                                       // did the directive lookup find a value?
    emitter.instruction("jz __rt_ini_get_miss");                                // a lookup miss returns PHP false
    emitter.instruction("mov rdx, rsi");                                        // move the borrowed value length into the persist input
    abi::emit_call_label(emitter, "__rt_str_persist");
    emitter.instruction("jmp __rt_ini_get_done");                               // return the independent owned copy

    emitter.label("__rt_ini_get_miss");
    emitter.instruction("xor eax, eax");                                        // null string pointer signals PHP false

    emitter.label("__rt_ini_get_done");
    emitter.instruction("add rsp, 16");                                         // deallocate the ini_get frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the string-or-false result
}
