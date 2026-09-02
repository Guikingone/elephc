//! Purpose:
//! Emits `__rt_throw_object_not_array`, PHP's catchable `Error` for indexing an object that is
//! not `ArrayAccess`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::objects::mixed_array_get`'s object paths.
//!
//! Key details:
//! - PHP stops the program for `$o["k"]` on any object that does not implement `ArrayAccess`,
//!   `stdClass` included, and it does so in the quiet contexts too — `isset`, `??` and `empty`
//!   all raise, measured against 8.5. So this helper takes no warning flag: reaching it is
//!   already the error.
//! - The class name comes from the dense `_class_name_entries` metadata `get_class()` reads, so
//!   the message carries php-src's wording verbatim.
//! - `__rt_concat` reads its LEFT operand from the string-result pair and its RIGHT one from a
//!   different pair per target; both are spelled out below rather than assumed, because the
//!   two are not the same registers and only one architecture is exercised by CI.
//! - Control never returns. `__rt_throw_current` unwinds to the nearest handler, or reports the
//!   uncaught Throwable and exits like PHP when there is none.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::runtime::data::{OBJECT_NOT_ARRAY_PREFIX, OBJECT_NOT_ARRAY_SUFFIX};
use crate::codegen_support::sentinels::{
    emit_throwable_creation_line_unknown, x86_64_heap_kind_word,
};

/// Dispatches to the target-specific `__rt_throw_object_not_array` emitter.
pub fn emit_throw_object_not_array(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_throw_object_not_array_x86_64(emitter);
        return;
    }
    emit_throw_object_not_array_aarch64(emitter);
}

/// Emits `__rt_throw_object_not_array` for ARM64.
///
/// Input: `x0` = the unboxed object pointer whose class cannot be indexed. Never returns.
fn emit_throw_object_not_array_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: throw object-not-array Error ---");
    emitter.label_global("__rt_throw_object_not_array");

    // Stack (48 bytes): [sp, #0] holds the message pair across the object allocation.
    emitter.instruction("sub sp, sp, #48");                                     // reserve message state and frame linkage
    emitter.instruction("stp x29, x30, [sp, #32]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #32");                                    // establish a stable Error-construction frame

    emitter.instruction("ldr x13, [x0]");                                       // keep the class id outside the symbol helper's x9 scratch
    abi::emit_load_symbol_to_reg(emitter, "x10", "_class_name_count", 0);
    emitter.instruction("cmp x13, x10");                                        // is the class id within the dense name table?
    emitter.instruction("b.hs __rt_object_not_array_name_fallback");            // malformed ids use the generic spelling
    abi::emit_symbol_address(emitter, "x10", "_class_name_entries");
    emitter.instruction("add x10, x10, x13, lsl #4");                           // select the 16-byte class-name row
    emitter.instruction("ldp x11, x12, [x10]");                                 // borrow the class-name pointer and byte length
    emitter.instruction("cbnz x12, __rt_object_not_array_name_ready");          // a non-empty name is what PHP prints
    emitter.label("__rt_object_not_array_name_fallback");
    abi::emit_symbol_address(emitter, "x11", "_unser_type_object");
    emitter.instruction("mov x12, #6");                                         // fallback length for the bare word "object"
    emitter.label("__rt_object_not_array_name_ready");

    abi::emit_symbol_address(emitter, "x1", "_object_not_array_prefix");        // concat left operand pointer
    emitter.instruction(&format!("mov x2, #{}", OBJECT_NOT_ARRAY_PREFIX.len())); // concat left operand length
    emitter.instruction("mov x3, x11");                                         // right operand: the resolved class name
    emitter.instruction("mov x4, x12");                                         // and its byte length
    emitter.instruction("bl __rt_concat");                                      // build `Cannot use object of type <Class>`
    abi::emit_symbol_address(emitter, "x3", "_object_not_array_suffix");        // right operand pointer
    emitter.instruction(&format!("mov x4, #{}", OBJECT_NOT_ARRAY_SUFFIX.len())); // right operand length
    emitter.instruction("bl __rt_concat");                                      // append PHP's ` as array`
    emitter.instruction("bl __rt_str_persist");                                 // give the Error stable message ownership
    emitter.instruction("stp x1, x2, [sp]");                                    // preserve the message pair across the allocation

    emitter.instruction("mov x0, #56");                                         // canonical Throwable payload size
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the Error object payload
    emitter.instruction("mov x9, #6");                                          // heap kind 6 identifies a throwable object
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the allocation as a runtime object
    emitter.instruction("bl __rt_object_handle_acquire");                       // bind the Error to its PHP object handle
    abi::emit_load_symbol_to_reg(emitter, "x9", "_spl_error_class_id", 0);
    emitter.instruction("str x9, [x0]");                                        // stamp the per-program Error class id
    emitter.instruction("ldp x10, x11, [sp]");                                  // recover the persisted message pair
    emitter.instruction("str x10, [x0, #8]");                                   // message pointer
    emitter.instruction("str x11, [x0, #16]");                                  // message byte length
    // __rt_heap_alloc recycles blocks without zeroing, so every remaining slot is written here.
    emitter.instruction("str xzr, [x0, #24]");                                  // code = 0
    emit_throwable_creation_line_unknown(emitter, "x0");
    emitter.instruction("str xzr, [x0, #40]");                                  // previous = null
    abi::emit_store_reg_to_symbol(emitter, "x0", "_exc_value", 0);              // publish the Throwable for the unwinder
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("b __rt_throw_current");                                // unwind, or report it uncaught and exit like PHP
}

/// Emits `__rt_throw_object_not_array` for x86_64.
///
/// Input: `rdi` = the unboxed object pointer whose class cannot be indexed. Never returns.
fn emit_throw_object_not_array_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: throw object-not-array Error ---");
    emitter.label_global("__rt_throw_object_not_array");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish an Error-construction frame
    emitter.instruction("sub rsp, 32");                                         // reserve the message pair, keeping rsp aligned

    emitter.instruction("mov r13, QWORD PTR [rdi]");                            // runtime class id
    emitter.instruction("cmp r13, QWORD PTR [rip + _class_name_count]");        // is the class id within the dense name table?
    emitter.instruction("jae __rt_object_not_array_name_fallback");             // malformed ids use the generic spelling
    emitter.instruction("lea r10, [rip + _class_name_entries]");                // dense class-name metadata table
    emitter.instruction("shl r13, 4");                                          // scale the class id to the 16-byte row
    emitter.instruction("mov r11, QWORD PTR [r10 + r13]");                      // borrow the class-name pointer
    emitter.instruction("mov r12, QWORD PTR [r10 + r13 + 8]");                  // borrow the class-name byte length
    emitter.instruction("test r12, r12");                                       // is the name non-empty?
    emitter.instruction("jnz __rt_object_not_array_name_ready");                // a non-empty name is what PHP prints
    emitter.label("__rt_object_not_array_name_fallback");
    emitter.instruction("lea r11, [rip + _unser_type_object]");                 // fall back to the bare word "object"
    emitter.instruction("mov r12, 6");                                          // fallback name length
    emitter.label("__rt_object_not_array_name_ready");

    emitter.instruction("lea rax, [rip + _object_not_array_prefix]");           // concat left operand pointer
    emitter.instruction(&format!("mov rdx, {}", OBJECT_NOT_ARRAY_PREFIX.len())); // concat left operand length
    emitter.instruction("mov rdi, r11");                                        // right operand: the resolved class name
    emitter.instruction("mov rsi, r12");                                        // and its byte length
    abi::emit_call_label(emitter, "__rt_concat");                               // build `Cannot use object of type <Class>`
    emitter.instruction("lea rdi, [rip + _object_not_array_suffix]");           // right operand pointer
    emitter.instruction(&format!("mov rsi, {}", OBJECT_NOT_ARRAY_SUFFIX.len())); // right operand length
    abi::emit_call_label(emitter, "__rt_concat");                               // append PHP's ` as array`
    abi::emit_call_label(emitter, "__rt_str_persist");                          // give the Error stable message ownership
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve the message pointer across the allocation
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // preserve the message byte length

    emitter.instruction("mov rax, 56");                                         // canonical Throwable payload size
    abi::emit_call_label(emitter, "__rt_heap_alloc");                           // allocate the Error object payload (rax = payload)
    emitter.instruction(&format!("mov r10, 0x{:x}", x86_64_heap_kind_word(6))); // magic + kind 6 identifies a throwable object
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the uniform heap header
    abi::emit_call_label(emitter, "__rt_object_handle_acquire");                // bind the Error to its PHP object handle
    abi::emit_load_symbol_to_reg(emitter, "r10", "_spl_error_class_id", 0);
    emitter.instruction("mov QWORD PTR [rax], r10");                            // stamp the per-program Error class id
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // recover the message pointer
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                       // recover the message byte length
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // message pointer
    emitter.instruction("mov QWORD PTR [rax + 16], r11");                       // message byte length
    // __rt_heap_alloc recycles blocks without zeroing, so every remaining slot is written here.
    emitter.instruction("mov QWORD PTR [rax + 24], 0");                         // code = 0
    emit_throwable_creation_line_unknown(emitter, "rax");
    emitter.instruction("mov QWORD PTR [rax + 40], 0");                         // previous = null
    abi::emit_store_reg_to_symbol(emitter, "rax", "_exc_value", 0);             // publish the Throwable for the unwinder
    emitter.instruction("mov rsp, rbp");                                        // release the local frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("jmp __rt_throw_current");                              // unwind, or report it uncaught and exit like PHP
}
