//! Purpose:
//! Exposes missing user-wrapper stream_stat metadata through narrow C ABI queries.
//!
//! Called from:
//! - The fileinfo runtime emitter and its prelude after filter creation, before fstat.
//!
//! Key details:
//! - Resolves PHP resource IDs through the existing bounded resource-ID registry.
//! - Reads immutable class-name spans without assuming a trailing NUL byte.
//! - Invokes no PHP callbacks and changes no stream or resource-ID state.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
use crate::codegen_support::runtime::RESOURCE_ID_TABLE_SLOTS;

/// Emits the missing-method metadata probe and explicit pointer/length class-name accessors.
pub(super) fn emit_fileinfo_stat(emitter: &mut Emitter) {
    emit_close_mode(emitter);
    let name = emitter.target.extern_symbol("elephc_runtime_stream_missing_stat_class");
    emitter.label_global(&name);
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64_probe(emitter),
        Arch::X86_64 => emit_x86_probe(emitter),
    }
    for (name, offset) in [("elephc_runtime_stream_stat_name", 0), ("elephc_runtime_stream_stat_name_length", 8)] {
        let symbol = emitter.target.extern_symbol(name);
        emitter.label_global(&symbol);
        match emitter.target.arch {
            Arch::AArch64 => emitter.instruction(&format!("ldr x0, [x0, #{offset}]")), // borrow the requested immutable class-name metadata word
            Arch::X86_64 => emitter.instruction(&format!("mov rax, QWORD PTR [rdi + {offset}]")), // borrow the requested immutable class-name metadata word
        }
        emitter.instruction("ret");                                             // return the metadata word through the target C ABI
    }
}

/// Queries or replaces the native-extension close mode, returning its prior state.
fn emit_close_mode(emitter: &mut Emitter) {
    let name = emitter.target.extern_symbol("elephc_runtime_stream_close_mode");
    emitter.label_global(&name);
    match emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_symbol_to_reg(emitter, "x14", "_stream_close_silent", 0);
            emitter.instruction("tbnz x0, #63, __rt_finfo_close_mode_return");  // a negative mode only queries the current state
            abi::emit_store_reg_to_symbol(emitter, "x0", "_stream_close_silent", 0);
            emitter.label("__rt_finfo_close_mode_return");
            emitter.instruction("mov x0, x14");                                 // return the previous mode without the store helper's x9 scratch
        }
        Arch::X86_64 => {
            abi::emit_load_symbol_to_reg(emitter, "rax", "_stream_close_silent", 0);
            emitter.instruction("test rdi, rdi");                               // a negative mode only queries the current state
            emitter.instruction("js __rt_finfo_close_mode_return");             // leave the mode unchanged for a query
            abi::emit_store_reg_to_symbol(emitter, "rdi", "_stream_close_silent", 0);
            emitter.label("__rt_finfo_close_mode_return");
        }
    }
    emitter.instruction("ret");                                                 // return the previous mode through the target C ABI
}

/// Emits an AArch64 leaf probe using only C ABI caller-saved registers.
fn emit_aarch64_probe(emitter: &mut Emitter) {
    // -- resolve the displayed PHP resource ID to its native payload --
    abi::emit_symbol_address(emitter, "x9", "_resource_id_vals");
    abi::emit_symbol_address(emitter, "x10", "_resource_id_keys");
    emitter.instruction("mov x11, #0");                                         // start at the first resource-ID slot
    emitter.label("__rt_finfo_stat_scan");
    emitter.instruction(&format!("cmp x11, #{RESOURCE_ID_TABLE_SLOTS}"));       // bound the resource-ID scan
    emitter.instruction("b.hs __rt_finfo_stat_empty");                          // absent resource IDs have no wrapper metadata
    emitter.instruction("ldr x12, [x9, x11, lsl #3]");                          // read the displayed resource ID in this slot
    emitter.instruction("cmp x12, x0");                                         // match the caller's resource ID
    emitter.instruction("b.eq __rt_finfo_stat_found");                          // resolve its native payload after a match
    emitter.instruction("add x11, x11, #1");                                    // inspect the next resource-ID slot
    emitter.instruction("b __rt_finfo_stat_scan");                              // continue the bounded scan
    emitter.label("__rt_finfo_stat_found");
    emitter.instruction("ldr x12, [x10, x11, lsl #3]");                         // load the native resource payload
    emitter.instruction("mov x13, #0x4000");                                    // materialize the high word of the synthetic wrapper base
    emitter.instruction("lsl x13, x13, #16");                                   // form the wrapper descriptor base 0x40000000
    emitter.instruction("sub x12, x12, x13");                                   // convert a wrapper descriptor into its handle-table index
    emitter.instruction("cmp x12, #256");                                       // reject non-wrapper payloads and out-of-range slots
    emitter.instruction("b.hs __rt_finfo_stat_empty");                          // unsigned comparison also rejects descriptors below the base
    abi::emit_symbol_address(emitter, "x13", "_user_wrapper_handles");
    emitter.instruction("ldr x12, [x13, x12, lsl #3]");                         // borrow the live wrapper object
    emitter.instruction("cbz x12, __rt_finfo_stat_empty");                      // closed wrapper handles expose no metadata
    emitter.instruction("ldr x12, [x12]");                                      // read the object's dense class ID
    abi::emit_load_symbol_to_reg(emitter, "x13", "_class_name_count", 0);
    emitter.instruction("cmp x12, x13");                                        // check the class metadata bounds before any table access
    emitter.instruction("b.hs __rt_finfo_stat_empty");                          // reject unknown runtime class IDs
    abi::emit_symbol_address(emitter, "x13", "_user_wrapper_vtable_ptrs");
    emitter.instruction("ldr x13, [x13, x12, lsl #3]");                         // load the wrapper class's method table
    emitter.instruction("cbz x13, __rt_finfo_stat_empty");                      // tolerate classes without a wrapper vtable
    emitter.instruction("ldr x14, [x13, #64]");                                 // vtable slot eight is stream_stat
    emitter.instruction("cbnz x14, __rt_finfo_stat_empty");                     // implemented methods need no missing-method warning
    abi::emit_symbol_address(emitter, "x13", "_class_name_entries");
    emitter.instruction("lsl x12, x12, #4");                                    // scale the class ID by the 16-byte pointer/length row
    emitter.instruction("add x0, x13, x12");                                    // return the immutable class-name metadata row
    emitter.instruction("ret");                                                 // leave the class-name span available until program exit
    emitter.label("__rt_finfo_stat_empty");
    emitter.instruction("mov x0, #0");                                          // no missing stream_stat method was found
    emitter.instruction("ret");                                                 // return a null metadata pointer
}

/// Emits the x86_64 leaf probe using only SysV caller-saved registers.
fn emit_x86_probe(emitter: &mut Emitter) {
    // -- resolve the displayed PHP resource ID without minting or changing an ID --
    abi::emit_symbol_address(emitter, "r8", "_resource_id_vals");
    abi::emit_symbol_address(emitter, "r9", "_resource_id_keys");
    emitter.instruction("xor ecx, ecx");                                        // start at the first resource-ID slot
    emitter.label("__rt_finfo_stat_scan");
    emitter.instruction(&format!("cmp rcx, {RESOURCE_ID_TABLE_SLOTS}"));        // bound the resource-ID scan
    emitter.instruction("jae __rt_finfo_stat_empty");                           // absent resource IDs have no wrapper metadata
    emitter.instruction("cmp QWORD PTR [r8 + rcx * 8], rdi");                   // match the caller's displayed resource ID
    emitter.instruction("je __rt_finfo_stat_found");                            // resolve the native payload after a match
    emitter.instruction("inc rcx");                                             // inspect the next resource-ID slot
    emitter.instruction("jmp __rt_finfo_stat_scan");                            // continue the bounded scan
    emitter.label("__rt_finfo_stat_found");
    emitter.instruction("mov rax, QWORD PTR [r9 + rcx * 8]");                   // load the native resource payload
    emitter.instruction("sub rax, 0x40000000");                                 // convert a wrapper descriptor into its handle-table index
    emitter.instruction("cmp rax, 256");                                        // reject non-wrapper payloads and out-of-range slots
    emitter.instruction("jae __rt_finfo_stat_empty");                           // unsigned comparison also rejects descriptors below the base
    abi::emit_symbol_address(emitter, "r8", "_user_wrapper_handles");
    emitter.instruction("mov rax, QWORD PTR [r8 + rax * 8]");                   // borrow the live wrapper object
    emitter.instruction("test rax, rax");                                       // detect a cleared wrapper handle
    emitter.instruction("jz __rt_finfo_stat_empty");                            // closed handles expose no metadata
    emitter.instruction("mov rax, QWORD PTR [rax]");                            // read the object's dense class ID
    abi::emit_load_symbol_to_reg(emitter, "r9", "_class_name_count", 0);
    emitter.instruction("cmp rax, r9");                                         // check class metadata bounds before any table access
    emitter.instruction("jae __rt_finfo_stat_empty");                           // reject unknown runtime class IDs
    abi::emit_symbol_address(emitter, "r8", "_user_wrapper_vtable_ptrs");
    emitter.instruction("mov r8, QWORD PTR [r8 + rax * 8]");                    // load the wrapper class's method table
    emitter.instruction("test r8, r8");                                         // detect a class without a wrapper vtable
    emitter.instruction("jz __rt_finfo_stat_empty");                            // tolerate classes without a wrapper method table
    emitter.instruction("cmp QWORD PTR [r8 + 64], 0");                          // vtable slot eight is stream_stat
    emitter.instruction("jne __rt_finfo_stat_empty");                           // implemented methods need no missing-method warning
    abi::emit_symbol_address(emitter, "r8", "_class_name_entries");
    emitter.instruction("shl rax, 4");                                          // scale the class ID by its 16-byte pointer/length row
    emitter.instruction("add rax, r8");                                         // return the immutable class-name metadata row
    emitter.instruction("ret");                                                 // leave the class-name span available until program exit
    emitter.label("__rt_finfo_stat_empty");
    emitter.instruction("xor eax, eax");                                        // no missing stream_stat method was found
    emitter.instruction("ret");                                                 // return a null metadata pointer
}
