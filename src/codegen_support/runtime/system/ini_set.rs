//! Purpose:
//! Emits `__rt_ini_set`, the persistent-ini-table writer backing PHP `ini_set`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - Input: directive name in the string result registers (x1/x2, or rax/rdx on x86_64) and
//!   a BORROWED (not yet persisted) coerced-to-string value in x3/x4 (or rcx/r8 on x86_64).
//!   The EIR lowering coerces the raw PHP value to a string but leaves ownership to this
//!   routine, which persists it only after confirming the directive is registered.
//! - Registration: PHP rejects UNREGISTERED directives. The seeded table doubles as the
//!   registered set: if `__rt_hash_get` misses (directive was never seeded) the routine
//!   returns PHP false and stores NOTHING — and, because the new value is persisted only on
//!   the found path, the unregistered path allocates nothing to leak.
//! - Ownership: the previous value must be `__rt_str_persist`-copied BEFORE `__rt_hash_set`
//!   overwrites it, because hash_set releases the table's old value on update — returning the
//!   borrowed pointer would dangle (UAF).

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Emits `__rt_ini_set` for every supported target.
///
/// Ensures the table is seeded, looks the directive up, and — only if it is registered
/// (seeded) — snapshots the previous value, persists the caller's borrowed new value,
/// overwrites the entry, and returns the previous string. An unregistered directive
/// returns PHP false and stores nothing (persisting no new value, so it leaks nothing).
pub fn emit_ini_set(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ini_set_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ini_set ---");
    emitter.label_global("__rt_ini_set");

    // Frame layout (64 bytes): [sp,#0]=name ptr, [sp,#8]=name len, [sp,#16]=borrowed new value
    // ptr, [sp,#24]=borrowed new value len, [sp,#32]=owned prev ptr, [sp,#40]=owned prev len,
    // [sp,#48/56]=x29/x30.
    emitter.instruction("sub sp, sp, #64");                                     // allocate the ini_set frame
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish the new frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the directive name pointer
    emitter.instruction("str x2, [sp, #8]");                                    // save the directive name length
    emitter.instruction("str x3, [sp, #16]");                                   // save the borrowed coerced new value pointer
    emitter.instruction("str x4, [sp, #24]");                                   // save the borrowed coerced new value length

    abi::emit_call_label(emitter, "__rt_ini_table_ensure");
    abi::emit_load_symbol_to_reg(emitter, "x0", "_rt_ini_table", 0);
    emitter.instruction("ldr x1, [sp, #0]");                                    // key_lo = directive name pointer
    emitter.instruction("ldr x2, [sp, #8]");                                    // key_hi = directive name length
    abi::emit_call_label(emitter, "__rt_hash_get");
    emitter.instruction("cbz x0, __rt_ini_set_unregistered");                   // unregistered directive: return false, store nothing

    // -- registered: snapshot the previous value as an owned copy before it is overwritten --
    abi::emit_call_label(emitter, "__rt_str_persist");
    emitter.instruction("str x1, [sp, #32]");                                   // snapshot the previous value before overwrite
    emitter.instruction("str x2, [sp, #40]");                                   // snapshot the previous value length

    // -- persist the new value ONLY now (registered): nothing was allocated on the miss path --
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload the borrowed coerced new value pointer
    emitter.instruction("ldr x2, [sp, #24]");                                   // reload the borrowed coerced new value length
    abi::emit_call_label(emitter, "__rt_str_persist");
    emitter.instruction("mov x3, x1");                                          // value_lo = owned new value pointer
    emitter.instruction("mov x4, x2");                                          // value_hi = owned new value length
    abi::emit_load_symbol_to_reg(emitter, "x0", "_rt_ini_table", 0);
    emitter.instruction("ldr x1, [sp, #0]");                                    // key_lo = directive name pointer
    emitter.instruction("ldr x2, [sp, #8]");                                    // key_hi = directive name length
    emitter.instruction("mov x5, #1");                                          // value tag = Str
    abi::emit_call_label(emitter, "__rt_hash_set");
    abi::emit_store_reg_to_symbol(emitter, "x0", "_rt_ini_table", 0);

    emitter.instruction("ldr x1, [sp, #32]");                                   // return the owned previous value pointer
    emitter.instruction("ldr x2, [sp, #40]");                                   // return the previous value length
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate the ini_set frame
    emitter.instruction("ret");                                                 // return the previous string result

    emitter.label("__rt_ini_set_unregistered");
    emitter.instruction("mov x1, #0");                                          // null string pointer signals PHP false
    emitter.instruction("mov x2, #0");                                          // zero length for the PHP false result
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate the ini_set frame
    emitter.instruction("ret");                                                 // return PHP false for the unregistered directive
}

/// Emits `__rt_ini_set` for x86_64 Linux targets.
///
/// System V variant: the directive name arrives in rax/rdx and the BORROWED coerced new
/// value in rcx/r8. An unregistered directive returns PHP false and stores nothing; a
/// registered directive snapshots the previous value, persists the new value, overwrites
/// the entry, and returns the previous value. Both persists use the rdi-input `__rt_str_persist`.
fn emit_ini_set_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ini_set ---");
    emitter.label_global("__rt_ini_set");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 48");                                         // reserve slots for name, new value, and previous value
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the directive name pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the directive name length
    emitter.instruction("mov QWORD PTR [rbp - 24], rcx");                       // save the borrowed coerced new value pointer
    emitter.instruction("mov QWORD PTR [rbp - 32], r8");                        // save the borrowed coerced new value length

    abi::emit_call_label(emitter, "__rt_ini_table_ensure");
    abi::emit_load_symbol_to_reg(emitter, "rdi", "_rt_ini_table", 0);
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // key_lo = directive name pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // key_hi = directive name length
    abi::emit_call_label(emitter, "__rt_hash_get");
    emitter.instruction("test rax, rax");                                       // is the directive registered (already seeded)?
    emitter.instruction("jz __rt_ini_set_unregistered");                        // unregistered directive: return false, store nothing

    // -- registered: snapshot the previous value as an owned copy before it is overwritten --
    emitter.instruction("mov rdx, rsi");                                        // move the borrowed previous length into the persist input
    abi::emit_call_label(emitter, "__rt_str_persist");
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // snapshot the previous value before overwrite
    emitter.instruction("mov QWORD PTR [rbp - 48], rdx");                       // snapshot the previous value length

    // -- persist the new value ONLY now (registered): nothing was allocated on the miss path --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // reload the borrowed coerced new value pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // reload the borrowed coerced new value length
    abi::emit_call_label(emitter, "__rt_str_persist");
    emitter.instruction("mov rcx, rax");                                        // value_lo = owned new value pointer
    emitter.instruction("mov r8, rdx");                                         // value_hi = owned new value length
    abi::emit_load_symbol_to_reg(emitter, "rdi", "_rt_ini_table", 0);
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // key_lo = directive name pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // key_hi = directive name length
    emitter.instruction("mov r9, 1");                                           // value tag = Str
    abi::emit_call_label(emitter, "__rt_hash_set");
    abi::emit_store_reg_to_symbol(emitter, "rax", "_rt_ini_table", 0);

    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // return the owned previous value pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // return the previous value length
    emitter.instruction("add rsp, 48");                                         // deallocate the ini_set frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the previous string result

    emitter.label("__rt_ini_set_unregistered");
    emitter.instruction("xor eax, eax");                                        // null string pointer signals PHP false
    emitter.instruction("xor edx, edx");                                        // zero length for the PHP false result
    emitter.instruction("add rsp, 48");                                         // deallocate the ini_set frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return PHP false for the unregistered directive
}
