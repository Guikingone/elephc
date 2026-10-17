//! Purpose:
//! Implements Throwable location access, including ErrorException's optional overrides.
//!
//! Called from:
//! - Native Throwable accessors and the eval method bridge.
//!
//! Key details:
//! - ErrorException's inherited prefix is message, code, previous, then severity.
//! - The filename cell occupies offset 72; the nullable scalar line uses words 88 and 96.
//! - The getter tests runtime ancestry before reading extended object storage.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits location accessors for both ordinary and explicitly located exceptions.
pub(crate) fn emit_location(emitter: &mut Emitter) {
    for file in [true, false] {
        let stem = if file { "__rt_throwable_get_file" } else { "__rt_throwable_get_line" };
        emitter.blank();
        emitter.label_global(stem);
        match emitter.target.arch {
            Arch::AArch64 => emit_aarch64(emitter, stem, file),
            Arch::X86_64 => emit_x86_64(emitter, stem, file),
        }
    }
}

/// Reads Apple/Linux AArch64 exception location with caller-owned string results.
fn emit_aarch64(emitter: &mut Emitter, stem: &str, file: bool) {
    let fallback = format!("{stem}_default");
    let done = format!("{stem}_done");
    let ready = format!("{stem}_ready");
    emitter.instruction("stp x29, x30, [sp, #-32]!");                           // preserve the caller frame across ancestry and string calls
    emitter.instruction("str x0, [sp, #16]");                                   // keep the receiver alive across the ancestry check
    abi::emit_load_symbol_to_reg(emitter, "x1", "_error_exception_class_id", 0);
    emitter.instruction("mov x2, #0");                                          // request class ancestry rather than interface matching
    abi::emit_call_label(emitter, "__rt_exception_matches");
    emitter.instruction(&format!("cbz x0, {fallback}"));                        // ordinary throwables use their creation location
    emitter.instruction("ldr x9, [sp, #16]");                                   // recover the receiver after ancestry testing
    if file {
        emitter.instruction("ldr x9, [x9, #72]");                               // load the nullable filename cell
        emitter.instruction(&format!("cbz x9, {fallback}"));                    // a missing cell selects the creation location
        emitter.instruction("ldr x10, [x9]");                                   // inspect the boxed filename tag
    } else {
        emitter.instruction("ldr x10, [x9, #96]");                              // inspect the nullable scalar line tag
    }
    emitter.instruction("cmp x10, #8");                                         // null means the constructor did not override this location
    emitter.instruction(&format!("b.eq {fallback}"));                           // preserve the original location for null arguments
    if file {
        emitter.instruction("ldr x1, [x9, #8]");                                // return the filename override bytes
        emitter.instruction("ldr x2, [x9, #16]");                               // return the filename override length
    } else {
        emitter.instruction("ldr x0, [x9, #88]");                               // return the explicit integer line
    }
    emitter.instruction(&format!("b {ready}"));                                 // join before result ownership is established
    emitter.label(&fallback);
    if file {
        abi::emit_symbol_address(emitter, "x1", "_script_source_file");
        abi::emit_load_symbol_to_reg(emitter, "x2", "_script_source_file_len", 0);
    } else {
        emitter.instruction("ldr x9, [sp, #16]");                               // recover the original Throwable receiver
        emit_creation_line_load(emitter, "x9", "x0", "x10", stem);
    }
    emitter.label(&ready);
    if file { abi::emit_call_label(emitter, "__rt_str_persist"); }
    emitter.label(&done);
    emitter.instruction("ldp x29, x30, [sp], #32");                             // restore the caller frame after materialization
    emitter.instruction("ret");                                                 // return the location with its declared PHP representation
}

/// Reads Linux x86_64 exception location with aligned native calls.
fn emit_x86_64(emitter: &mut Emitter, stem: &str, file: bool) {
    let fallback = format!("{stem}_default");
    let ready = format!("{stem}_ready");
    emitter.instruction("push rbp");                                            // preserve the caller's frame pointer
    emitter.instruction("mov rbp, rsp");                                        // anchor receiver storage across nested helpers
    emitter.instruction("sub rsp, 16");                                         // reserve aligned receiver scratch
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve the input object pointer
    emitter.instruction("mov rdi, rax");                                        // pass the Throwable receiver to ancestry matching
    abi::emit_load_symbol_to_reg(emitter, "rsi", "_error_exception_class_id", 0);
    emitter.instruction("xor edx, edx");                                        // request class ancestry rather than interface matching
    abi::emit_call_label(emitter, "__rt_exception_matches");
    emitter.instruction("test rax, rax");                                       // test whether the object derives from ErrorException
    emitter.instruction(&format!("jz {fallback}"));                             // ordinary throwables use their creation location
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // restore the receiver after ancestry testing
    if file {
        emitter.instruction("mov r10, QWORD PTR [r10 + 72]");                   // load the nullable filename cell
        emitter.instruction("test r10, r10");                                   // absent cells do not provide an override
        emitter.instruction(&format!("jz {fallback}"));                         // preserve the original location for an absent cell
        emitter.instruction("cmp QWORD PTR [r10], 8");                          // null means the constructor kept the original filename
    } else {
        emitter.instruction("cmp QWORD PTR [r10 + 96], 8");                     // inspect the nullable scalar line tag
    }
    emitter.instruction(&format!("je {fallback}"));                             // preserve the original location for null arguments
    if file {
        emitter.instruction("mov rax, QWORD PTR [r10 + 8]");                    // return the filename override bytes
        emitter.instruction("mov rdx, QWORD PTR [r10 + 16]");                   // return the filename override length
    } else {
        emitter.instruction("mov rax, QWORD PTR [r10 + 88]");                   // return the explicit integer line
    }
    emitter.instruction(&format!("jmp {ready}"));                               // join before establishing result ownership
    emitter.label(&fallback);
    if file {
        abi::emit_symbol_address(emitter, "rax", "_script_source_file");
        abi::emit_load_symbol_to_reg(emitter, "rdx", "_script_source_file_len", 0);
    } else {
        emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                    // recover the original Throwable receiver
        emit_creation_line_load(emitter, "r10", "rax", "r11", stem);
    }
    emitter.label(&ready);
    if file { abi::emit_call_label(emitter, "__rt_str_persist"); }
    emitter.instruction("leave");                                               // release scratch and restore the caller's frame
    emitter.instruction("ret");                                                 // return the declared PHP location representation
}

/// Loads compact or ordinary creation metadata without calls or ownership changes.
/// The receiver, result, and scratch registers must be distinct; `stem` scopes branch labels.
pub(crate) fn emit_creation_line_load(
    emitter: &mut Emitter,
    object: &str,
    result: &str,
    scratch: &str,
    stem: &str,
) {
    debug_assert!(object != result && object != scratch && result != scratch);
    let compact = format!("{stem}_compact_line");
    let done = format!("{stem}_line_loaded");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("ldurb {scratch_w}, [{object}, #-8]", scratch_w = scratch.replacen('x', "w", 1))); // inspect the physical layout without GC metadata bits
            emitter.instruction(&format!("cmp {scratch}, #6"));                 // compact runtime Throwables retain their fixed metadata word
            emitter.instruction(&format!("b.eq {compact}"));                    // select the compact payload layout
            emitter.instruction(&format!("ldr {scratch}, [{object}]"));         // resolve the ordinary class id
            abi::emit_symbol_address(emitter, result, "_class_throwable_line_offsets");
            emitter.instruction(&format!("ldr {result}, [{result}, {scratch}, lsl #3]")); // locate metadata beyond PHP property and hash storage
            emitter.instruction(&format!("cbz {result}, {done}"));              // absent metadata reports an unknown origin
            emitter.instruction(&format!("ldr {result}, [{object}, {result}]")); // read the ordinary object's dedicated creation line
            emitter.instruction(&format!("b {done}"));                          // skip the compact payload read
            emitter.label(&compact);
            emitter.instruction(&format!("ldr {result}, [{object}, #32]"));     // read the compact Throwable creation line
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("movzx {scratch}, BYTE PTR [{object} - 8]")); // inspect the heap kind without GC or magic bits
            emitter.instruction(&format!("cmp {scratch}, 6"));                  // compact runtime Throwables retain their fixed metadata word
            emitter.instruction(&format!("je {compact}"));                      // select the compact payload layout
            emitter.instruction(&format!("mov {scratch}, QWORD PTR [{object}]")); // resolve the ordinary class id
            abi::emit_symbol_address(emitter, result, "_class_throwable_line_offsets");
            emitter.instruction(&format!("mov {result}, QWORD PTR [{result} + {scratch} * 8]")); // locate metadata beyond PHP property and hash storage
            emitter.instruction(&format!("test {result}, {result}"));           // test whether this layout reserves construction metadata
            emitter.instruction(&format!("jz {done}"));                         // absent metadata reports an unknown origin
            emitter.instruction(&format!("mov {result}, QWORD PTR [{object} + {result}]")); // read the ordinary object's dedicated creation line
            emitter.instruction(&format!("jmp {done}"));                        // skip the compact payload read
            emitter.label(&compact);
            emitter.instruction(&format!("mov {result}, QWORD PTR [{object} + 32]")); // read the compact Throwable creation line
        }
    }
    emitter.label(&done);
}
