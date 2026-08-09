//! Purpose:
//! Emits `__rt_builtin_wrapper_index`, which maps a wrapper scheme name to its
//! index in the built-in wrapper list, plus the disabled-wrapper bitmask helpers
//! backing `stream_wrapper_unregister()` / `stream_wrapper_restore()`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - `stream_wrapper_unregister` / `stream_wrapper_restore`, `stream_get_wrappers`,
//!   and the `fopen` built-in-scheme guard.
//!
//! Key details:
//! - PHP lets a built-in wrapper be unregistered and later restored. elephc kept
//!   built-ins outside the user table, so unregistering one reported false and
//!   restoring it was a no-op that always claimed success.
//! - Disabled built-ins live in a bitmask rather than a table so the state costs
//!   one word and a restore is a single bit clear.

use crate::codegen_support::data_section::comm_directive;
use crate::codegen_support::{abi, emit::Emitter, platform::Arch, platform::Target};
use crate::types::stream_constants::STREAM_WRAPPERS;

/// Emits the built-in wrapper name table into the runtime data section.
pub(crate) fn emit_builtin_wrapper_table(out: &mut String, target: Target) {
    for (index, name) in STREAM_WRAPPERS.iter().enumerate() {
        out.push_str(&format!(
            ".globl _bw_name_{index}\n_bw_name_{index}:\n    .ascii \"{name}\"\n"
        ));
    }
    out.push_str(".p2align 3\n.globl _bw_table\n_bw_table:\n");
    for (index, name) in STREAM_WRAPPERS.iter().enumerate() {
        out.push_str(&format!(
            "    .quad _bw_name_{index}\n    .quad {}\n    .quad {index}\n",
            name.len()
        ));
    }
    out.push_str("    .quad 0\n    .quad 0\n    .quad 0\n");
    // One bit per built-in wrapper; a set bit means stream_wrapper_unregister()
    // removed it and fopen() must refuse the scheme until it is restored.
    out.push_str(&comm_directive("_disabled_builtin_wrappers", 8, target));
}

/// `__rt_builtin_wrapper_index(ptr, len) -> index`, or -1 when not built in.
pub fn emit_builtin_wrapper_index(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_builtin_wrapper_index_linux_x86_64(emitter);
        return;
    }
    emitter.blank();
    emitter.comment("--- runtime: resolve a built-in wrapper scheme name ---");
    emitter.label_global("__rt_builtin_wrapper_index");
    abi::emit_symbol_address(emitter, "x9", "_bw_table");
    emitter.label("__rt_bwi_entry");
    emitter.instruction("ldr x10, [x9]");                                       // candidate name pointer
    emitter.instruction("cbz x10, __rt_bwi_miss");                              // null terminates the table
    emitter.instruction("ldr x11, [x9, #8]");                                   // candidate name length
    emitter.instruction("cmp x11, x1");                                         // does the length match?
    emitter.instruction("b.ne __rt_bwi_next");
    emitter.instruction("mov x12, #0");                                         // byte compare cursor
    emitter.label("__rt_bwi_bytes");
    emitter.instruction("cmp x12, x1");                                         // compared every byte?
    emitter.instruction("b.ge __rt_bwi_hit");
    emitter.instruction("ldrb w13, [x10, x12]");
    emitter.instruction("ldrb w14, [x0, x12]");
    emitter.instruction("cmp w13, w14");
    emitter.instruction("b.ne __rt_bwi_next");
    emitter.instruction("add x12, x12, #1");
    emitter.instruction("b __rt_bwi_bytes");
    emitter.label("__rt_bwi_next");
    emitter.instruction("add x9, x9, #24");                                     // next 3-word entry
    emitter.instruction("b __rt_bwi_entry");
    emitter.label("__rt_bwi_hit");
    emitter.instruction("ldr x0, [x9, #16]");                                   // return the built-in index
    emitter.instruction("ret");
    emitter.label("__rt_bwi_miss");
    emitter.instruction("mov x0, #-1");                                         // not a built-in wrapper
    emitter.instruction("ret");
}

/// x86_64 variant of [`emit_builtin_wrapper_index`].
fn emit_builtin_wrapper_index_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: resolve a built-in wrapper scheme name ---");
    emitter.label_global("__rt_builtin_wrapper_index");
    abi::emit_symbol_address(emitter, "r9", "_bw_table");
    emitter.label("__rt_bwi_entry_x");
    emitter.instruction("mov r10, QWORD PTR [r9]");
    emitter.instruction("test r10, r10");
    emitter.instruction("jz __rt_bwi_miss_x");
    emitter.instruction("mov r11, QWORD PTR [r9 + 8]");
    emitter.instruction("cmp r11, rsi");
    emitter.instruction("jne __rt_bwi_next_x");
    emitter.instruction("xor rcx, rcx");
    emitter.label("__rt_bwi_bytes_x");
    emitter.instruction("cmp rcx, rsi");
    emitter.instruction("jge __rt_bwi_hit_x");
    emitter.instruction("mov dl, BYTE PTR [r10 + rcx]");
    emitter.instruction("mov r8b, BYTE PTR [rdi + rcx]");
    emitter.instruction("cmp dl, r8b");
    emitter.instruction("jne __rt_bwi_next_x");
    emitter.instruction("add rcx, 1");
    emitter.instruction("jmp __rt_bwi_bytes_x");
    emitter.label("__rt_bwi_next_x");
    emitter.instruction("add r9, 24");
    emitter.instruction("jmp __rt_bwi_entry_x");
    emitter.label("__rt_bwi_hit_x");
    emitter.instruction("mov rax, QWORD PTR [r9 + 16]");
    emitter.instruction("ret");
    emitter.label("__rt_bwi_miss_x");
    emitter.instruction("mov rax, -1");
    emitter.instruction("ret");
}
