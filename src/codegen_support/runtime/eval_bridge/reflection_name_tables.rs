//! Purpose:
//! Emits class-like name membership scanners for both targets.
//!
//! Called from:
//! - The eval bridge runtime facade and sibling bridge emitters.
//!
//! Key details:
//! - Table row shape and case-insensitive comparisons remain unchanged.

use super::*;

/// Emits a class-like name membership scanner for ARM64 eval bridge hooks.
pub(super) fn emit_aarch64_eval_name_table_exists(
    emitter: &mut Emitter,
    exported_label: &str,
    count_symbol: &str,
    table_symbol: &str,
    local_stem: &str,
) {
    label_c_global(emitter, exported_label);
    emitter.instruction("sub sp, sp, #64");                                     // reserve lookup state while comparing metadata names
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address across string compares
    emitter.instruction("add x29, sp, #48");                                    // establish a stable name-table lookup frame
    emitter.instruction("str x0, [sp, #0]");                                    // save the requested name pointer
    emitter.instruction("str x1, [sp, #8]");                                    // save the requested name length
    abi::emit_symbol_address(emitter, "x9", count_symbol);
    emitter.instruction("ldr x9, [x9]");                                        // load the metadata-name table count
    emitter.instruction(&format!("cbz x9, {local_stem}_miss"));                 // an empty table cannot contain the requested name
    emitter.instruction("str x9, [sp, #16]");                                   // save the table count across string compares
    abi::emit_symbol_address(emitter, "x10", table_symbol);
    emitter.instruction("str x10, [sp, #24]");                                  // save the current metadata-name table cursor
    emitter.instruction("mov x11, #0");                                         // start scanning at table index zero
    emitter.label(&format!("{local_stem}_loop"));
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload the metadata-name table count
    emitter.instruction("cmp x11, x9");                                         // have all metadata-name entries been scanned?
    emitter.instruction(&format!("b.ge {local_stem}_miss"));                    // no metadata name matched before the end
    emitter.instruction("ldr x10, [sp, #24]");                                  // reload the current metadata-name table entry
    emitter.instruction("ldr x12, [x10, #8]");                                  // load the stored metadata-name length
    emitter.instruction("ldr x2, [sp, #8]");                                    // reload the requested name length
    emitter.instruction("cmp x12, x2");                                         // compare stored and requested name lengths
    emitter.instruction(&format!("b.ne {local_stem}_skip"));                    // length mismatch means this entry cannot match
    emitter.instruction("str x11, [sp, #32]");                                  // save the table index across the string compare
    emitter.instruction("ldr x1, [sp, #0]");                                    // pass the requested name pointer
    emitter.instruction("ldr x2, [sp, #8]");                                    // pass the requested name length
    emitter.instruction("ldr x3, [x10]");                                       // pass the stored metadata-name pointer
    emitter.instruction("mov x4, x12");                                         // pass the stored metadata-name length
    emitter.instruction("bl __rt_strcasecmp");                                  // compare names with PHP case-insensitive rules
    emitter.instruction("ldr x11, [sp, #32]");                                  // restore the table index after the string compare
    emitter.instruction("cmp x0, #0");                                          // did the requested name match this entry?
    emitter.instruction(&format!("b.eq {local_stem}_hit"));                     // report true on a metadata-name match
    emitter.label(&format!("{local_stem}_skip"));
    emitter.instruction("ldr x10, [sp, #24]");                                  // reload the current metadata-name table entry
    emitter.instruction("add x10, x10, #16");                                   // advance to the next metadata-name table entry
    emitter.instruction("str x10, [sp, #24]");                                  // persist the advanced table cursor
    emitter.instruction("add x11, x11, #1");                                    // advance the table index
    emitter.instruction(&format!("b {local_stem}_loop"));                       // continue scanning the metadata-name table
    emitter.label(&format!("{local_stem}_hit"));
    emitter.instruction("mov x0, #1");                                          // return true for a matched metadata name
    emitter.instruction(&format!("b {local_stem}_done"));                       // skip the false result after a match
    emitter.label(&format!("{local_stem}_miss"));
    emitter.instruction("mov x0, #0");                                          // return false when no metadata name matched
    emitter.label(&format!("{local_stem}_done"));
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the name-table lookup frame
    emitter.instruction("ret");                                                 // return the metadata-name existence flag to Rust
}

/// Emits an x86_64 eval wrapper that scans one `(name_ptr, name_len)` metadata table.
pub(super) fn emit_x86_64_eval_name_table_exists(
    emitter: &mut Emitter,
    exported_label: &str,
    count_symbol: &str,
    table_symbol: &str,
    local_stem: &str,
) {
    label_c_global(emitter, exported_label);
    emitter.instruction("push rbp");                                            // preserve the Rust caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable name-table lookup frame
    emitter.instruction("sub rsp, 48");                                         // reserve slots for name, count, cursor, and index
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the requested name pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the requested name length
    abi::emit_symbol_address(emitter, "r10", count_symbol);
    emitter.instruction("mov r10, QWORD PTR [r10]");                            // load the metadata-name table count
    emitter.instruction("test r10, r10");                                       // is the metadata-name table empty?
    emitter.instruction(&format!("jz {local_stem}_miss"));                      // an empty table cannot contain the requested name
    emitter.instruction("mov QWORD PTR [rbp - 24], r10");                       // save the table count across string compares
    abi::emit_symbol_address(emitter, "r11", table_symbol);
    emitter.instruction("mov QWORD PTR [rbp - 32], r11");                       // save the current metadata-name table cursor
    emitter.instruction("xor r11d, r11d");                                      // start scanning at table index zero
    emitter.label(&format!("{local_stem}_loop"));
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the metadata-name table count
    emitter.instruction("cmp r11, r10");                                        // have all metadata-name entries been scanned?
    emitter.instruction(&format!("jae {local_stem}_miss"));                     // no metadata name matched before the end
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the current metadata-name table entry
    emitter.instruction("mov rcx, QWORD PTR [r10 + 8]");                        // load the stored metadata-name length
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 16]");                       // compare stored and requested name lengths
    emitter.instruction(&format!("jne {local_stem}_skip"));                     // length mismatch means this entry cannot match
    emitter.instruction("mov QWORD PTR [rbp - 40], r11");                       // save the table index across the string compare
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the requested name pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // pass the requested name length
    emitter.instruction("mov rdx, QWORD PTR [r10]");                            // pass the stored metadata-name pointer
    emitter.instruction("call __rt_strcasecmp");                                // compare names with PHP case-insensitive rules
    emitter.instruction("mov r11, QWORD PTR [rbp - 40]");                       // restore the table index after the string compare
    emitter.instruction("test rax, rax");                                       // did the requested name match this entry?
    emitter.instruction(&format!("je {local_stem}_hit"));                       // report true on a metadata-name match
    emitter.label(&format!("{local_stem}_skip"));
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the current metadata-name table entry
    emitter.instruction("add r10, 16");                                         // advance to the next metadata-name table entry
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // persist the advanced table cursor
    emitter.instruction("inc r11");                                             // advance the table index
    emitter.instruction(&format!("jmp {local_stem}_loop"));                     // continue scanning the metadata-name table
    emitter.label(&format!("{local_stem}_hit"));
    emitter.instruction("mov eax, 1");                                          // return true for a matched metadata name
    emitter.instruction(&format!("jmp {local_stem}_done"));                     // skip the false result after a match
    emitter.label(&format!("{local_stem}_miss"));
    emitter.instruction("xor eax, eax");                                        // return false when no metadata name matched
    emitter.label(&format!("{local_stem}_done"));
    emitter.instruction("mov rsp, rbp");                                        // discard helper spill slots
    emitter.instruction("pop rbp");                                             // restore the Rust caller frame pointer
    emitter.instruction("ret");                                                 // return the metadata-name existence flag to Rust
}
