//! Purpose:
//! Emits `__rt_interface_exists`, the runtime helper backing a non-literal
//! `interface_exists($name)` call by searching the closed-world interface registry.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` (gated by the
//!   `class_introspection` feature) and the EIR `interface_exists` lowering.
//!
//! Key details:
//! - Input:  x0/rdi=name_ptr, x1/rsi=name_len. The `$autoload` argument is
//!   irrelevant in a closed-world build and is dropped before this helper.
//! - Output: x0/rax = 1 when a matching interface is declared, 0 otherwise. No throw.
//! - PHP interface names are case-insensitive, so the query is lowercased with
//!   `__rt_strtolower` before searching the 16-byte `_interface_table` entries
//!   (whose entries are stored pre-lowercased, see `const_registry.rs`) via
//!   `__rt_sorted_name_search`, which also strips a single leading `\`.

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Emits the target-specific `__rt_interface_exists` interface-existence helper.
pub fn emit_rt_interface_exists(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Emits the AArch64 implementation of `__rt_interface_exists`.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: interface_exists ---");
    emitter.label_global("__rt_interface_exists");

    emitter.instruction("stp x29, x30, [sp, #-16]!");                           // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish the new frame pointer

    // -- lowercase the query name so the lookup matches PHP's case-insensitive interface names --
    emitter.instruction("mov x2, x1");                                          // stash the query length before strtolower's args overwrite x1
    emitter.instruction("mov x1, x0");                                          // move the query pointer into strtolower's pointer argument
    emitter.instruction("bl __rt_strtolower");                                  // x1=lowercased query copy, x2=length unchanged

    // -- search the interface registry --
    emitter.instruction("mov x0, x1");                                          // lowercased query pointer -> search arg0
    emitter.instruction("mov x1, x2");                                          // query length -> search arg1
    abi::emit_symbol_address(emitter, "x2", "_interface_table");
    abi::emit_load_symbol_to_reg(emitter, "x3", "_interface_table_count", 0);
    emitter.instruction("mov x4, #16");                                         // interface entries are 16 bytes wide
    emitter.instruction("bl __rt_sorted_name_search");                          // search for the interface name
    emitter.instruction("cmp x0, #0");                                          // did the search return a matching entry?
    emitter.instruction("cset x0, ne");                                         // map a non-null entry pointer to true
    emitter.instruction("ldp x29, x30, [sp], #16");                             // restore frame pointer and return address
    emitter.instruction("ret");                                                 // return the boolean in x0
}

/// Emits the x86_64 implementation of `__rt_interface_exists`.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: interface_exists ---");
    emitter.label_global("__rt_interface_exists");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // keep the nested call 16-byte aligned

    // -- lowercase the query name so the lookup matches PHP's case-insensitive interface names --
    emitter.instruction("mov rdx, rsi");                                        // stash the query length before strtolower's args overwrite it
    emitter.instruction("mov rax, rdi");                                        // move the query pointer into strtolower's pointer argument
    emitter.instruction("call __rt_strtolower");                                // rax=lowercased query copy, rdx=length unchanged

    // -- search the interface registry --
    emitter.instruction("mov rdi, rax");                                        // lowercased query pointer -> search arg0
    emitter.instruction("mov rsi, rdx");                                        // query length -> search arg1
    abi::emit_symbol_address(emitter, "rdx", "_interface_table");
    abi::emit_load_symbol_to_reg(emitter, "rcx", "_interface_table_count", 0);
    emitter.instruction("mov r8, 16");                                          // interface entries are 16 bytes wide
    emitter.instruction("call __rt_sorted_name_search");                        // search for the interface name
    emitter.instruction("test rax, rax");                                       // did the search return a matching entry?
    emitter.instruction("setne al");                                            // map a non-null entry pointer to true
    emitter.instruction("movzx eax, al");                                       // widen the boolean byte into the result register
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the boolean in rax
}
