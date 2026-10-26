//! Purpose:
//! Appends native SPL list debug fields to print_r object output.
//!
//! Called from:
//! - The target-specific object walkers in `super::print_r_object`.
//!
//! Key details:
//! - User properties precede these fields; custom __debugInfo methods suppress this view.
//! - Shared print_r writers preserve capture mode, indentation and recursion handling.
//! - Property enumeration and var_export use their existing descriptors.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
use crate::codegen_support::runtime::spl::{SPL_DLL_ITER_MODE_OFFSET, SPL_DLL_STORAGE_OFFSET};

/// Emits a borrowed receiver/entry-indent adapter on every supported target.
#[rustfmt::skip]
pub(super) fn emit(emitter: &mut Emitter) {
    emitter.label_global("__rt_print_r_native_spl_entries");
    let done = "__rt_print_r_native_spl_done";
    if emitter.target.arch == Arch::X86_64 {
        emitter.instruction("mov r9, QWORD PTR [rdi]");                         // load the concrete receiver class id
        abi::emit_symbol_address(emitter, "r10", "_class_gc_desc_count");
        emitter.instruction("cmp r9, QWORD PTR [r10]");                        // reject synthetic or out-of-range class ids
        emitter.instruction(&format!("jae {done}"));                            // ordinary missing metadata contributes no debug fields
        abi::emit_symbol_address(emitter, "r10", "_class_print_r_native_kinds");
        emitter.instruction("cmp QWORD PTR [r10 + r9 * 8], 1");                // kind 1 is the inherited native DLL debug handler
        emitter.instruction(&format!("jne {done}"));                            // custom handlers and ordinary objects keep their own view
        emitter.instruction("push rbp");                                       // preserve the caller frame
        emitter.instruction("mov rbp, rsp");                                   // establish the native debug frame
        emitter.instruction("sub rsp, 16");                                    // reserve borrowed receiver and indentation spills
        emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                   // preserve the receiver across writers
        emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                  // preserve entry indentation across recursive values
        for (key, length, offset, tag) in fields() {
            abi::emit_symbol_address(emitter, "rdi", key);
            abi::emit_load_int_immediate(emitter, "rsi", length as i64);
            emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");              // write this key at the object's existing entry indentation
            abi::emit_call_label(emitter, "__rt_print_r_str_key");
            emitter.instruction("mov r9, QWORD PTR [rbp - 8]");                // reload the receiver after the key writer
            abi::emit_load_from_address(emitter, "rsi", "r9", offset);
            abi::emit_load_int_immediate(emitter, "rdi", tag);
            emitter.instruction("xor edx, edx");                               // native scalar and array values have no high payload word
            emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");              // load the entry indentation
            emitter.instruction("add rcx, 4");                                 // nested containers open four columns after their key
            abi::emit_call_label(emitter, "__rt_print_r_value");
            abi::emit_symbol_address(emitter, "rsi", "_pr_nl");
            abi::emit_load_int_immediate(emitter, "rdx", 1);
            abi::emit_call_label(emitter, "__rt_pr_write");
        }
        emitter.instruction("mov rsp, rbp");                                   // release native debug spills
        emitter.instruction("pop rbp");                                        // restore the caller frame
    } else {
        emitter.instruction("ldr x9, [x0]");                                   // load the concrete receiver class id
        abi::emit_symbol_address(emitter, "x10", "_class_gc_desc_count");
        emitter.instruction("ldr x10, [x10]");                                 // load the dense class-table extent
        emitter.instruction("cmp x9, x10");                                    // reject synthetic or out-of-range class ids
        emitter.instruction(&format!("b.hs {done}"));                           // missing metadata contributes no debug fields
        abi::emit_symbol_address(emitter, "x10", "_class_print_r_native_kinds");
        emitter.instruction("ldr x9, [x10, x9, lsl #3]");                      // inspect the receiver's inherited native debug kind
        emitter.instruction("cmp x9, #1");                                     // kind 1 is the native DLL debug handler
        emitter.instruction(&format!("b.ne {done}"));                           // custom handlers and ordinary objects keep their own view
        emitter.instruction("sub sp, sp, #32");                                // reserve borrowed inputs and the caller frame
        emitter.instruction("stp x29, x30, [sp, #16]");                         // preserve the caller frame and return address
        emitter.instruction("add x29, sp, #16");                               // establish the native debug frame
        emitter.instruction("stp x0, x1, [sp]");                               // preserve receiver and entry indentation
        for (key, length, offset, tag) in fields() {
            abi::emit_symbol_address(emitter, "x0", key);
            abi::emit_load_int_immediate(emitter, "x1", length as i64);
            emitter.instruction("ldr x2, [sp, #8]");                            // write this key at the object's existing entry indentation
            abi::emit_call_label(emitter, "__rt_print_r_str_key");
            emitter.instruction("ldr x9, [sp]");                               // reload the receiver after the key writer
            abi::emit_load_from_address(emitter, "x1", "x9", offset);
            abi::emit_load_int_immediate(emitter, "x0", tag);
            emitter.instruction("mov x2, xzr");                                // native scalar and array values have no high payload word
            emitter.instruction("ldr x3, [sp, #8]");                            // load the entry indentation
            emitter.instruction("add x3, x3, #4");                              // nested containers open four columns after their key
            abi::emit_call_label(emitter, "__rt_print_r_value");
            abi::emit_symbol_address(emitter, "x1", "_pr_nl");
            abi::emit_load_int_immediate(emitter, "x2", 1);
            abi::emit_call_label(emitter, "__rt_pr_write");
        }
        emitter.instruction("ldp x29, x30, [sp, #16]");                         // restore the caller frame and return address
        emitter.instruction("add sp, sp, #32");                                // release native debug spills
    }
    emitter.label(done);
    abi::emit_return(emitter);
}

/// Describes the render-only keys and their native physical slots.
fn fields() -> [(&'static str, usize, usize, i64); 2] {
    [
        ("_pr_native_dll_flags_key", "flags:SplDoublyLinkedList:private".len(), SPL_DLL_ITER_MODE_OFFSET, 0),
        ("_pr_native_dll_items_key", "dllist:SplDoublyLinkedList:private".len(), SPL_DLL_STORAGE_OFFSET, 4),
    ]
}
