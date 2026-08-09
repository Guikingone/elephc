//! Purpose:
//! Emits the `__rt_unserialize_mixed` runtime helper (and its internal cursor-based
//! recursive parser `__rt_unser_at` / key parser `__rt_unser_key`) that parse a PHP
//! `serialize()` wire string into a freshly boxed Mixed value.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//! - The EIR `unserialize()` lowering in `crate::codegen_support::lower_inst::builtins::system`.
//!
//! Key details:
//! - Recognizes scalars `N;`, `b:0;`/`b:1;`, `i:<int>;`, `d:<float>;` (incl.
//!   `INF`/`-INF`/`NAN`), `s:<bytelen>:"<raw>";`, arrays, objects, and references.
//!   Objects resolve declared classes and invoke supported hydration hooks; references
//!   reuse entries from the per-call registry.
//! - Arrays build a hash (`__rt_hash_new` value_type 7) whose values are boxed Mixed
//!   cells stored with per-entry tag 7 — the canonical heterogeneous representation
//!   (see `__rt_array_to_mixed`). Ownership: scalar/value boxes come from
//!   `__rt_mixed_from_value` (persists strings) and are transferred into the hash by
//!   `__rt_hash_set` (which does not incref values); string keys are borrowed and
//!   persisted by `__rt_hash_set`; the finished hash is boxed without an extra incref.
//! - Manually boxed arrays and objects on x86_64 must preserve the runtime heap marker
//!   in the upper half of their allocation-kind word.
//! - Blocked objects retain their persisted original class name and a Mixed property
//!   hash, so `__PHP_Incomplete_Class` reserializes with correct reference rebasing.
//! - Begin/end helpers isolate reentrant calls by snapshotting the active policy,
//!   parser depth, and used reference-registry prefix. Option values are normalized
//!   into a fresh direct-string array owned by the call, so associative keys are
//!   irrelevant and hydration hooks never observe borrowed policy storage.
//! - An allocation-free recursive preflight validates every cursor, decimal overflow,
//!   delimiter, child, key, and closing brace before the mutating parser can allocate
//!   or invoke hooks. Floats reuse libc `strtod` only after a bounded semicolon scan,
//!   then require its end pointer to identify that exact delimiter.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::runtime::data::{
    UNSER_ALLOWED_CLASSES_ENTRY_PREFIX, UNSER_ALLOWED_CLASSES_POLICY_PREFIX,
    UNSER_OBJECT_STRING_ERROR_PREFIX, UNSER_OBJECT_STRING_ERROR_SUFFIX,
    UNSER_OPTIONS_TYPE_PREFIX, UNSER_TYPE_GIVEN_SUFFIX,
};
use crate::codegen_support::try_handlers::{
    TRY_HANDLER_DIAG_DEPTH_OFFSET, TRY_HANDLER_JMP_BUF_OFFSET,
    TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET, TRY_HANDLER_SLOT_SIZE,
};


/// Emits `__rt_unserialize_mixed` plus its internal parser helpers.
///
/// `__rt_unserialize_mixed` input: AArch64 `x1`=ptr, `x2`=len; x86_64 `rax`=ptr,
/// `rdx`=len. Output: AArch64 `x0` / x86_64 `rax` = boxed Mixed pointer, or 0 on a
/// parse error or unsupported wire form (the caller boxes that as PHP `false`).
pub(crate) fn emit_unserialize(emitter: &mut Emitter) {
    emit_unserialize_allowed_classes(emitter);
    emit_unserialize_type_error_helper(emitter);
    emit_unserialize_object_string_error_helper(emitter);
    emit_unserialize_object_to_string_helper(emitter);
    if emitter.target.arch == Arch::X86_64 {
        emit_unserialize_x86_64(emitter);
        return;
    }
    emit_unserialize_aarch64(emitter);
}

/// Emits the non-returning dynamic unserialize TypeError helper.
///
/// Callers provide a runtime tag/payload plus a message prefix. The helper closes
/// the active public unserialize context exactly once, resolves PHP's actual type
/// name (including declared object class names), persists the composed message,
/// and propagates a standard catchable `TypeError`.
fn emit_unserialize_type_error_helper(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.blank();
            emitter.comment("--- runtime: catchable unserialize TypeError ---");
            emitter.label_global("__rt_unser_throw_type_error");
            emitter.instruction("sub sp, sp, #80");                             // reserve tag/payload/prefix/type/message state
            emitter.instruction("stp x29, x30, [sp, #64]");                     // preserve the caller frame and return address
            emitter.instruction("add x29, sp, #64");                            // establish a stable error-construction frame
            emitter.instruction("stp x0, x1, [sp]");                            // save runtime tag and payload
            emitter.instruction("stp x2, x3, [sp, #16]");                       // save diagnostic prefix pointer and length
            emitter.instruction("mov x0, #0");                                  // end cleanup ignores the placeholder parse result
            emitter.instruction("bl __rt_unserialize_end");                     // close this opened context exactly once
            emitter.instruction("ldr x9, [sp]");                                // reload rejected runtime tag
            emitter.instruction("cmp x9, #0");
            emitter.instruction("b.eq __rt_unser_type_int");
            emitter.instruction("cmp x9, #1");
            emitter.instruction("b.eq __rt_unser_type_string");
            emitter.instruction("cmp x9, #2");
            emitter.instruction("b.eq __rt_unser_type_float");
            emitter.instruction("cmp x9, #3");
            emitter.instruction("b.eq __rt_unser_type_bool");
            emitter.instruction("cmp x9, #4");
            emitter.instruction("b.eq __rt_unser_type_array");
            emitter.instruction("cmp x9, #5");
            emitter.instruction("b.eq __rt_unser_type_array");
            emitter.instruction("cmp x9, #6");
            emitter.instruction("b.eq __rt_unser_type_object");
            emitter.instruction("cmp x9, #8");
            emitter.instruction("b.eq __rt_unser_type_null");
            emitter.instruction("cmp x9, #9");
            emitter.instruction("b.eq __rt_unser_type_resource");
            crate::codegen_support::abi::emit_symbol_address(emitter, "x3", "_unser_type_unknown");
            emitter.instruction("mov x4, #7");                                  // byte length of unknown
            emitter.instruction("b __rt_unser_type_ready");
            for (label, symbol, len) in [
                ("__rt_unser_type_int", "_unser_type_int", 3),
                ("__rt_unser_type_string", "_unser_type_string", 6),
                ("__rt_unser_type_float", "_unser_type_float", 5),
                ("__rt_unser_type_bool", "_unser_type_bool", 4),
                ("__rt_unser_type_array", "_unser_type_array", 5),
                ("__rt_unser_type_null", "_unser_type_null", 4),
                ("__rt_unser_type_resource", "_unser_type_resource", 8),
            ] {
                emitter.label(label);
                crate::codegen_support::abi::emit_symbol_address(emitter, "x3", symbol);
                emitter.instruction(&format!("mov x4, #{}", len));              // materialize the selected PHP type-name length
                emitter.instruction("b __rt_unser_type_ready");                 // join dynamic message construction
            }
            emitter.label("__rt_unser_type_object");
            emitter.instruction("ldr x9, [sp, #8]");                            // rejected object payload
            emitter.instruction("cbz x9, __rt_unser_type_object_generic");      // null payload has no class metadata
            emitter.instruction("ldr x10, [x9]");                               // object class id
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x11", "_class_name_count", 0);
            emitter.instruction("cmp x10, x11");                                // class id within the dense name table?
            emitter.instruction("b.hs __rt_unser_type_object_generic");
            crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_class_name_entries");
            emitter.instruction("add x11, x11, x10, lsl #4");                   // select the (ptr,len) class-name row
            emitter.instruction("ldp x3, x4, [x11]");                           // use the concrete PHP class name
            emitter.instruction("cbnz x4, __rt_unser_type_ready");              // empty metadata falls back to object
            emitter.label("__rt_unser_type_object_generic");
            crate::codegen_support::abi::emit_symbol_address(emitter, "x3", "_unser_type_object");
            emitter.instruction("mov x4, #6");                                  // byte length of object
            emitter.label("__rt_unser_type_ready");
            emitter.instruction("ldp x1, x2, [sp, #16]");                       // diagnostic prefix string
            emitter.instruction("bl __rt_concat");                              // append the resolved PHP type name
            crate::codegen_support::abi::emit_symbol_address(emitter, "x3", "_unser_type_given_suffix");
            emitter.instruction(&format!("mov x4, #{}", UNSER_TYPE_GIVEN_SUFFIX.len())); // suffix byte length
            emitter.instruction("bl __rt_concat");                              // append PHP's ` given` suffix
            emitter.instruction("bl __rt_str_persist");                         // give the Throwable stable message ownership
            emitter.instruction("stp x1, x2, [sp, #48]");                       // preserve message pointer/length across allocation
            emitter.instruction("mov x0, #56");                                 // request the canonical Throwable payload size
            emitter.instruction("bl __rt_heap_alloc");                          // allocate the TypeError object payload
            emitter.instruction("mov x9, #6");                                  // heap kind 6 identifies a throwable object
            emitter.instruction("str x9, [x0, #-8]");                           // stamp the allocation as a runtime object
            emitter.instruction("bl __rt_object_handle_acquire");               // bind the TypeError to its PHP object handle
            crate::codegen_support::abi::emit_load_symbol_to_reg(
                emitter,
                "x9",
                "_spl_type_error_class_id",
                0,
            );
            emitter.instruction("str x9, [x0]");                                // store the built-in TypeError class id
            emitter.instruction("ldr x9, [sp, #48]");                           // recover the persisted TypeError message pointer
            emitter.instruction("str x9, [x0, #8]");                            // store the dynamic TypeError message pointer
            emitter.instruction("ldr x9, [sp, #56]");                           // recover the dynamic message byte length
            emitter.instruction("str x9, [x0, #16]");                           // store the TypeError message byte length
            emitter.instruction("str xzr, [x0, #24]");                          // exception code defaults to zero
            crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(
                emitter, "x0",
            );
            emitter.instruction("str xzr, [x0, #40]");                          // previous Throwable defaults to null
            crate::codegen_support::abi::emit_store_reg_to_symbol(
                emitter,
                "x0",
                "_exc_value",
                0,
            );
            emitter.instruction("ldp x29, x30, [sp, #64]");                     // restore the frame before unwinding
            emitter.instruction("add sp, sp, #80");                             // release error-construction state
            emitter.instruction("b __rt_throw_current");                        // propagate through the standard catchable exception path
        }
        Arch::X86_64 => {
            emitter.blank();
            emitter.comment("--- runtime: catchable unserialize TypeError ---");
            emitter.label_global("__rt_unser_throw_type_error");
            emitter.instruction("push rbp");                                    // preserve the caller frame while building the message
            emitter.instruction("mov rbp, rsp");                                // establish an aligned exception-construction frame
            emitter.instruction("sub rsp, 64");                                 // reserve tag/payload/prefix/type/message state
            emitter.instruction("mov QWORD PTR [rbp - 8], rax");                // save runtime tag
            emitter.instruction("mov QWORD PTR [rbp - 16], rdi");               // save runtime payload
            emitter.instruction("mov QWORD PTR [rbp - 24], rsi");               // save diagnostic prefix pointer
            emitter.instruction("mov QWORD PTR [rbp - 32], rdx");               // save diagnostic prefix length
            emitter.instruction("xor eax, eax");                                // end cleanup ignores the placeholder result
            emitter.instruction("call __rt_unserialize_end");                   // close this opened context exactly once
            emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                 // rejected runtime tag
            emitter.instruction("cmp r8, 0");
            emitter.instruction("je __rt_unser_type_int_x");
            emitter.instruction("cmp r8, 1");
            emitter.instruction("je __rt_unser_type_string_x");
            emitter.instruction("cmp r8, 2");
            emitter.instruction("je __rt_unser_type_float_x");
            emitter.instruction("cmp r8, 3");
            emitter.instruction("je __rt_unser_type_bool_x");
            emitter.instruction("cmp r8, 4");
            emitter.instruction("je __rt_unser_type_array_x");
            emitter.instruction("cmp r8, 5");
            emitter.instruction("je __rt_unser_type_array_x");
            emitter.instruction("cmp r8, 6");
            emitter.instruction("je __rt_unser_type_object_x");
            emitter.instruction("cmp r8, 8");
            emitter.instruction("je __rt_unser_type_null_x");
            emitter.instruction("cmp r8, 9");
            emitter.instruction("je __rt_unser_type_resource_x");
            emitter.instruction("lea rdi, [rip + _unser_type_unknown]");        // unknown runtime tag spelling
            emitter.instruction("mov rsi, 7");                                  // byte length of unknown
            emitter.instruction("jmp __rt_unser_type_ready_x");
            for (label, symbol, len) in [
                ("__rt_unser_type_int_x", "_unser_type_int", 3),
                ("__rt_unser_type_string_x", "_unser_type_string", 6),
                ("__rt_unser_type_float_x", "_unser_type_float", 5),
                ("__rt_unser_type_bool_x", "_unser_type_bool", 4),
                ("__rt_unser_type_array_x", "_unser_type_array", 5),
                ("__rt_unser_type_null_x", "_unser_type_null", 4),
                ("__rt_unser_type_resource_x", "_unser_type_resource", 8),
            ] {
                emitter.label(label);
                emitter.instruction(&format!("lea rdi, [rip + {}]", symbol));   // selected PHP type-name pointer
                emitter.instruction(&format!("mov rsi, {}", len));              // selected PHP type-name length
                emitter.instruction("jmp __rt_unser_type_ready_x");             // join dynamic message construction
            }
            emitter.label("__rt_unser_type_object_x");
            emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                // rejected object payload
            emitter.instruction("test r8, r8");
            emitter.instruction("jz __rt_unser_type_object_generic_x");
            emitter.instruction("mov r9, QWORD PTR [r8]");                      // object class id
            emitter.instruction("mov r10, QWORD PTR [rip + _class_name_count]"); // dense class-name table bound
            emitter.instruction("cmp r9, r10");
            emitter.instruction("jae __rt_unser_type_object_generic_x");
            emitter.instruction("lea r10, [rip + _class_name_entries]");        // class-name metadata base
            emitter.instruction("shl r9, 4");                                   // one pointer/length pair per class id
            emitter.instruction("add r10, r9");
            emitter.instruction("mov rdi, QWORD PTR [r10]");                    // concrete class-name pointer
            emitter.instruction("mov rsi, QWORD PTR [r10 + 8]");                // concrete class-name length
            emitter.instruction("test rsi, rsi");
            emitter.instruction("jnz __rt_unser_type_ready_x");
            emitter.label("__rt_unser_type_object_generic_x");
            emitter.instruction("lea rdi, [rip + _unser_type_object]");         // generic fallback type name
            emitter.instruction("mov rsi, 6");                                  // byte length of object
            emitter.label("__rt_unser_type_ready_x");
            emitter.instruction("mov rax, QWORD PTR [rbp - 24]");               // diagnostic prefix pointer
            emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");               // diagnostic prefix length
            emitter.instruction("call __rt_concat");                            // append the resolved PHP type name
            emitter.instruction("lea rdi, [rip + _unser_type_given_suffix]");   // PHP diagnostic suffix
            emitter.instruction(&format!("mov rsi, {}", UNSER_TYPE_GIVEN_SUFFIX.len())); // suffix byte length
            emitter.instruction("call __rt_concat");                            // append ` given`
            emitter.instruction("call __rt_str_persist");                       // give the Throwable stable message ownership
            emitter.instruction("mov QWORD PTR [rbp - 40], rax");               // save persisted message pointer
            emitter.instruction("mov QWORD PTR [rbp - 48], rdx");               // save persisted message length
            emitter.instruction("mov rax, 56");                                 // request the canonical Throwable payload size
            emitter.instruction("call __rt_heap_alloc");                        // allocate the TypeError object payload
            emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(6))); // materialize the throwable heap-kind marker
            emitter.instruction("mov QWORD PTR [rax - 8], r10");                // stamp the allocation as a runtime object
            emitter.instruction("call __rt_object_handle_acquire");             // bind the TypeError to its PHP object handle
            crate::codegen_support::abi::emit_load_symbol_to_reg(
                emitter,
                "r10",
                "_spl_type_error_class_id",
                0,
            );
            emitter.instruction("mov QWORD PTR [rax], r10");                    // store the built-in TypeError class id
            emitter.instruction("mov r10, QWORD PTR [rbp - 40]");               // persisted TypeError message pointer
            emitter.instruction("mov QWORD PTR [rax + 8], r10");                // store the dynamic TypeError message pointer
            emitter.instruction("mov r10, QWORD PTR [rbp - 48]");               // dynamic TypeError message byte length
            emitter.instruction("mov QWORD PTR [rax + 16], r10");               // store the TypeError message byte length
            emitter.instruction("mov QWORD PTR [rax + 24], 0");                 // exception code defaults to zero
            crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(
                emitter, "rax",
            );
            emitter.instruction("mov QWORD PTR [rax + 40], 0");                 // previous Throwable defaults to null
            crate::codegen_support::abi::emit_store_reg_to_symbol(
                emitter,
                "rax",
                "_exc_value",
                0,
            );
            emitter.instruction("mov rsp, rbp");                                // release the construction frame before unwinding
            emitter.instruction("pop rbp");                                     // restore the caller frame before unwinding
            emitter.instruction("jmp __rt_throw_current");                      // propagate through the standard catchable exception path
        }
    }
}

/// Emits the catchable PHP Error used when an allowed-class entry object has no `__toString()`.
///
/// The concrete class-name pair is resolved while the object is still borrowed from
/// the live options container. The helper then closes the unserialize context once,
/// composes the stable PHP message, and throws a standard `Error` object.
fn emit_unserialize_object_string_error_helper(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.blank();
            emitter.comment("--- runtime: unserialize object string-conversion Error ---");
            emitter.label_global("__rt_unser_throw_object_string_error");
            emitter.instruction("sub sp, sp, #64");                             // reserve class-name and message state
            emitter.instruction("stp x29, x30, [sp, #48]");                     // preserve the caller frame and return address
            emitter.instruction("add x29, sp, #48");                            // establish an aligned Error-construction frame
            emitter.instruction("ldr x13, [x0]");                               // keep class id outside the symbol helper's x9 scratch register
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_class_name_count", 0);
            emitter.instruction("cmp x13, x10");                                // is the class id within the dense name table?
            emitter.instruction("b.hs __rt_unser_object_string_name_fallback"); // malformed ids use the generic object spelling
            crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_class_name_entries");
            emitter.instruction("add x10, x10, x13, lsl #4");                   // select the static class-name row
            emitter.instruction("ldp x11, x12, [x10]");                         // borrow the static class-name pointer and length
            emitter.instruction("cbnz x12, __rt_unser_object_string_name_ready"); // non-empty metadata is safe after context cleanup
            emitter.label("__rt_unser_object_string_name_fallback");
            crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_unser_type_object");
            emitter.instruction("mov x12, #6");                                 // fallback name length for object
            emitter.label("__rt_unser_object_string_name_ready");
            emitter.instruction("stp x11, x12, [sp]");                          // keep only static name metadata across end
            emitter.instruction("mov x0, #0");                                  // cleanup ignores the placeholder parse result
            emitter.instruction("bl __rt_unserialize_end");                     // close this opened context exactly once
            crate::codegen_support::abi::emit_symbol_address(emitter, "x1", "_unser_object_string_error_prefix");
            emitter.instruction(&format!("mov x2, #{}", UNSER_OBJECT_STRING_ERROR_PREFIX.len())); // prefix byte length
            emitter.instruction("ldp x3, x4, [sp]");                            // append the resolved class name
            emitter.instruction("bl __rt_concat");                              // build `Object of class <Class>`
            crate::codegen_support::abi::emit_symbol_address(emitter, "x3", "_unser_object_string_error_suffix");
            emitter.instruction(&format!("mov x4, #{}", UNSER_OBJECT_STRING_ERROR_SUFFIX.len())); // suffix byte length
            emitter.instruction("bl __rt_concat");                              // append PHP's conversion failure suffix
            emitter.instruction("bl __rt_str_persist");                         // give the Error stable message ownership
            emitter.instruction("stp x1, x2, [sp, #16]");                       // preserve message pair across object allocation
            emitter.instruction("mov x0, #56");                                 // request the canonical Throwable payload size
            emitter.instruction("bl __rt_heap_alloc");                          // allocate the Error object payload
            emitter.instruction("mov x9, #6");                                  // heap kind 6 identifies a throwable object
            emitter.instruction("str x9, [x0, #-8]");                           // stamp the allocation as a runtime object
            emitter.instruction("bl __rt_object_handle_acquire");               // bind the Error to its PHP object handle
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x9", "_spl_error_class_id", 0);
            emitter.instruction("str x9, [x0]");                                // store the built-in Error class id
            emitter.instruction("ldp x10, x11, [sp, #16]");                     // recover the persisted message pair
            emitter.instruction("stp x10, x11, [x0, #8]");                      // store Error message pointer and length
            emitter.instruction("str xzr, [x0, #24]");                          // exception code defaults to zero
            crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(emitter, "x0");
            emitter.instruction("str xzr, [x0, #40]");                          // previous Throwable defaults to null
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x0", "_exc_value", 0);
            emitter.instruction("ldp x29, x30, [sp, #48]");                     // restore the caller frame before unwinding
            emitter.instruction("add sp, sp, #64");                             // release Error-construction state
            emitter.instruction("b __rt_throw_current");                        // propagate through the catchable Throwable path
        }
        Arch::X86_64 => {
            emitter.blank();
            emitter.comment("--- runtime: unserialize object string-conversion Error ---");
            emitter.label_global("__rt_unser_throw_object_string_error");
            emitter.instruction("push rbp");                                    // preserve the caller frame while constructing the Error
            emitter.instruction("mov rbp, rsp");                                // establish an aligned construction frame
            emitter.instruction("sub rsp, 48");                                 // reserve class-name and message state
            emitter.instruction("mov r8, QWORD PTR [rax]");                     // resolve class id while the borrowed object is live
            emitter.instruction("test r8, r8");                                 // reject negative synthetic class ids before table indexing
            emitter.instruction("js __rt_unser_object_string_name_fallback_x"); // synthetic ids use the generic object spelling
            emitter.instruction("cmp r8, QWORD PTR [rip + _class_name_count]"); // is the class id within the dense name table?
            emitter.instruction("jae __rt_unser_object_string_name_fallback_x"); // malformed ids use the generic object spelling
            emitter.instruction("lea r9, [rip + _class_name_entries]");         // class-name metadata base
            emitter.instruction("shl r8, 4");                                   // scale id by the pointer/length row width
            emitter.instruction("add r9, r8");                                  // select the static class-name row
            emitter.instruction("mov r10, QWORD PTR [r9]");                     // borrow static class-name pointer
            emitter.instruction("mov r11, QWORD PTR [r9 + 8]");                 // borrow static class-name length
            emitter.instruction("test r11, r11");                               // empty metadata falls back to object
            emitter.instruction("jnz __rt_unser_object_string_name_ready_x");
            emitter.label("__rt_unser_object_string_name_fallback_x");
            emitter.instruction("lea r10, [rip + _unser_type_object]");         // generic object class spelling
            emitter.instruction("mov r11, 6");                                  // fallback name byte length
            emitter.label("__rt_unser_object_string_name_ready_x");
            emitter.instruction("mov QWORD PTR [rbp - 8], r10");                // keep only static name metadata across end
            emitter.instruction("mov QWORD PTR [rbp - 16], r11");               // preserve static class-name length
            emitter.instruction("xor eax, eax");                                // cleanup ignores the placeholder parse result
            emitter.instruction("call __rt_unserialize_end");                   // close this opened context exactly once
            emitter.instruction("lea rax, [rip + _unser_object_string_error_prefix]"); // conversion Error prefix
            emitter.instruction(&format!("mov rdx, {}", UNSER_OBJECT_STRING_ERROR_PREFIX.len())); // prefix byte length
            emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                // resolved class-name pointer
            emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");               // resolved class-name length
            emitter.instruction("call __rt_concat");                            // build `Object of class <Class>`
            emitter.instruction("lea rdi, [rip + _unser_object_string_error_suffix]"); // conversion Error suffix
            emitter.instruction(&format!("mov rsi, {}", UNSER_OBJECT_STRING_ERROR_SUFFIX.len())); // suffix byte length
            emitter.instruction("call __rt_concat");                            // append PHP's conversion failure suffix
            emitter.instruction("call __rt_str_persist");                       // give the Error stable message ownership
            emitter.instruction("mov QWORD PTR [rbp - 24], rax");               // preserve persisted message pointer
            emitter.instruction("mov QWORD PTR [rbp - 32], rdx");               // preserve persisted message length
            emitter.instruction("mov rax, 56");                                 // request the canonical Throwable payload size
            emitter.instruction("call __rt_heap_alloc");                        // allocate the Error object payload
            emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(6))); // throwable heap-kind marker
            emitter.instruction("mov QWORD PTR [rax - 8], r10");                // stamp the allocation as a runtime object
            emitter.instruction("call __rt_object_handle_acquire");             // bind the Error to its PHP object handle
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_spl_error_class_id", 0);
            emitter.instruction("mov QWORD PTR [rax], r10");                    // store the built-in Error class id
            emitter.instruction("mov r10, QWORD PTR [rbp - 24]");               // recover persisted Error message pointer
            emitter.instruction("mov QWORD PTR [rax + 8], r10");                // store message pointer
            emitter.instruction("mov r10, QWORD PTR [rbp - 32]");               // recover persisted Error message length
            emitter.instruction("mov QWORD PTR [rax + 16], r10");               // store message byte length
            emitter.instruction("mov QWORD PTR [rax + 24], 0");                 // exception code defaults to zero
            crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(emitter, "rax");
            emitter.instruction("mov QWORD PTR [rax + 40], 0");                 // previous Throwable defaults to null
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "rax", "_exc_value", 0);
            emitter.instruction("leave");                                       // restore the caller frame before unwinding
            emitter.instruction("jmp __rt_throw_current");                      // propagate through the catchable Throwable path
        }
    }
}

/// Emits a dynamic, exception-safe `__toString()` invocation for allowed-class entries.
///
/// Input is a borrowed object (`x0`/`rax`). A class without `__toString()` returns
/// a null string pointer; otherwise the method's string pair is returned unchanged.
/// A Throwable escaping the user method is caught at this native boundary so the
/// active unserialize context is closed exactly once before propagation resumes.
fn emit_unserialize_object_to_string_helper(emitter: &mut Emitter) {
    let boundary_bytes = TRY_HANDLER_SLOT_SIZE + 32;
    match emitter.target.arch {
        Arch::AArch64 => {
            let frame_link_offset = boundary_bytes - 16;
            let object_offset = TRY_HANDLER_SLOT_SIZE;
            let string_len_offset = object_offset + 8;
            emitter.blank();
            emitter.comment("--- runtime: bounded allowed-class object __toString ---");
            emitter.label_global("__rt_unser_allowed_object_to_string");
            emitter.instruction(&format!("sub sp, sp, #{}", boundary_bytes));   // reserve a complete handler record plus result spills
            emitter.instruction(&format!("stp x29, x30, [sp, #{}]", frame_link_offset)); // preserve the caller frame and return address
            emitter.instruction(&format!("add x29, sp, #{}", frame_link_offset)); // establish the protected invocation frame
            emitter.instruction(&format!("str x0, [sp, #{}]", object_offset));  // preserve the borrowed receiver across setjmp
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_exc_handler_top", 0);
            emitter.instruction("str x10, [sp]");                               // handler.next = previous exception-handler head
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_exc_call_frame_top", 0);
            emitter.instruction("str x10, [sp, #8]");                           // preserve the activation frame surviving this boundary
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_rt_diag_suppression", 0);
            emitter.instruction(&format!("str x10, [sp, #{}]", TRY_HANDLER_DIAG_DEPTH_OFFSET)); // snapshot diagnostic suppression across longjmp
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_runtime_recursion_stack_bytes", 0);
            emitter.instruction(&format!("str x10, [sp, #{}]", TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET)); // snapshot user-stack accounting across longjmp
            emitter.instruction("mov x10, sp");                                 // compute this invocation's handler record address
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_exc_handler_top", 0);
            emitter.instruction(&format!("add x0, sp, #{}", TRY_HANDLER_JMP_BUF_OFFSET)); // pass this boundary's opaque jmp_buf to setjmp
            emitter.bl_c("setjmp"); // catch Throwable control flow escaping __toString
            emitter.instruction("cbnz x0, __rt_unser_allowed_tostring_throw");  // longjmp resumes here for exact context cleanup
            emitter.instruction(&format!("ldr x0, [sp, #{}]", object_offset));  // reload borrowed receiver after setjmp
            emitter.instruction("ldr x11, [x0]");                               // keep class id outside the symbol helper's x9 scratch register
            emitter.instruction("tbnz x11, #63, __rt_unser_allowed_tostring_missing"); // synthetic negative ids cannot index metadata
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_class_tostring_count", 0);
            emitter.instruction("cmp x11, x10");                                // is the id within the dense __toString table?
            emitter.instruction("b.hs __rt_unser_allowed_tostring_missing");    // out-of-range classes have no callable conversion
            crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_class_tostring_ptrs");
            emitter.instruction("ldr x10, [x10, x11, lsl #3]");                 // resolve the concrete or inherited method symbol
            emitter.instruction("cbz x10, __rt_unser_allowed_tostring_missing"); // no __toString means PHP conversion Error
            emitter.instruction("blr x10");                                     // call __toString with borrowed receiver in x0
            emitter.instruction(&format!("stp x1, x2, [sp, #{}]", object_offset)); // preserve returned string pair while popping boundary
            emitter.instruction("b __rt_unser_allowed_tostring_finish");        // share successful boundary teardown
            emitter.label("__rt_unser_allowed_tostring_missing");
            emitter.instruction(&format!("str xzr, [sp, #{}]", object_offset)); // null pointer reports a missing conversion method
            emitter.instruction(&format!("str xzr, [sp, #{}]", string_len_offset)); // missing conversion has no string length
            emitter.label("__rt_unser_allowed_tostring_finish");
            emitter.instruction("ldr x10, [sp]");                               // reload the previous exception-handler head
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_exc_handler_top", 0);
            emitter.instruction(&format!("ldr x10, [sp, #{}]", TRY_HANDLER_DIAG_DEPTH_OFFSET)); // restore diagnostic suppression after the call
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_rt_diag_suppression", 0);
            emitter.instruction(&format!("ldp x1, x2, [sp, #{}]", object_offset)); // recover the conversion string pair
            emitter.instruction(&format!("ldp x29, x30, [sp, #{}]", frame_link_offset)); // restore caller frame and return address
            emitter.instruction(&format!("add sp, sp, #{}", boundary_bytes));   // release the protected invocation frame
            emitter.instruction("ret");                                         // return the string pair or a null pointer
            emitter.label("__rt_unser_allowed_tostring_throw");
            emitter.instruction("ldr x10, [sp]");                               // reload the handler preceding this internal boundary
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_exc_handler_top", 0);
            emitter.instruction(&format!("ldr x10, [sp, #{}]", TRY_HANDLER_DIAG_DEPTH_OFFSET)); // restore diagnostic suppression skipped by longjmp
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_rt_diag_suppression", 0);
            emitter.instruction(&format!("ldr x10, [sp, #{}]", TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET)); // restore stack accounting skipped by longjmp
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_runtime_recursion_stack_bytes", 0);
            emitter.instruction("mov x0, #0");                                  // cleanup ignores the placeholder result on throw
            emitter.instruction("bl __rt_unserialize_end");                     // close this opened context exactly once
            emitter.instruction(&format!("ldp x29, x30, [sp, #{}]", frame_link_offset)); // restore caller frame before rethrow
            emitter.instruction(&format!("add sp, sp, #{}", boundary_bytes));   // discard the protected invocation frame
            emitter.instruction("b __rt_throw_current");                        // resume propagation at the caller's handler
        }
        Arch::X86_64 => {
            let previous_handler_offset = boundary_bytes;
            let survivor_offset = previous_handler_offset - 8;
            emitter.blank();
            emitter.comment("--- runtime: bounded allowed-class object __toString ---");
            emitter.label_global("__rt_unser_allowed_object_to_string");
            emitter.instruction("push rbp");                                    // preserve caller frame across the exception boundary
            emitter.instruction("mov rbp, rsp");                                // establish a stable base for the handler record
            emitter.instruction(&format!("sub rsp, {}", boundary_bytes));       // reserve handler record plus receiver/result spills
            emitter.instruction("mov QWORD PTR [rbp - 8], rax");                // preserve borrowed receiver across setjmp
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_exc_handler_top", 0);
            emitter.instruction(&format!("mov QWORD PTR [rbp - {}], r10", previous_handler_offset)); // handler.next = previous head
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_exc_call_frame_top", 0);
            emitter.instruction(&format!("mov QWORD PTR [rbp - {}], r10", survivor_offset)); // activation frame surviving this boundary
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_rt_diag_suppression", 0);
            emitter.instruction(&format!("mov QWORD PTR [rbp - {}], r10", boundary_bytes - TRY_HANDLER_DIAG_DEPTH_OFFSET)); // snapshot diagnostic suppression
            crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_runtime_recursion_stack_bytes", 0);
            emitter.instruction(&format!("mov QWORD PTR [rbp - {}], r10", boundary_bytes - TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET)); // snapshot stack accounting
            emitter.instruction(&format!("lea r10, [rbp - {}]", previous_handler_offset)); // compute this handler record address
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_exc_handler_top", 0);
            emitter.instruction(&format!("lea rdi, [rbp - {}]", boundary_bytes - TRY_HANDLER_JMP_BUF_OFFSET)); // pass opaque jmp_buf to setjmp
            emitter.bl_c("setjmp"); // catch Throwable control flow escaping __toString
            emitter.instruction("test eax, eax");                               // did control return through longjmp?
            emitter.instruction("jnz __rt_unser_allowed_tostring_throw_x");     // clean runtime state before propagating
            emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                // reload borrowed receiver after setjmp
            emitter.instruction("mov r8, QWORD PTR [rdi]");                     // runtime class id
            emitter.instruction("test r8, r8");                                 // reject negative synthetic class ids
            emitter.instruction("js __rt_unser_allowed_tostring_missing_x");    // synthetic ids cannot index metadata
            emitter.instruction("cmp r8, QWORD PTR [rip + _class_tostring_count]"); // is id within dense method table?
            emitter.instruction("jae __rt_unser_allowed_tostring_missing_x");   // out-of-range classes have no conversion method
            emitter.instruction("lea r10, [rip + _class_tostring_ptrs]");       // dense __toString function-pointer table
            emitter.instruction("mov r10, QWORD PTR [r10 + r8 * 8]");           // resolve concrete or inherited method symbol
            emitter.instruction("test r10, r10");                               // does the class expose __toString?
            emitter.instruction("jz __rt_unser_allowed_tostring_missing_x");    // no method means PHP conversion Error
            emitter.instruction("call r10");                                    // call __toString with borrowed receiver in rdi
            emitter.instruction("mov QWORD PTR [rbp - 8], rax");                // preserve returned string pointer while popping boundary
            emitter.instruction("mov QWORD PTR [rbp - 16], rdx");               // preserve returned string length
            emitter.instruction("jmp __rt_unser_allowed_tostring_finish_x");    // share normal boundary teardown
            emitter.label("__rt_unser_allowed_tostring_missing_x");
            emitter.instruction("mov QWORD PTR [rbp - 8], 0");                  // null pointer reports a missing conversion method
            emitter.instruction("mov QWORD PTR [rbp - 16], 0");                 // missing conversion has no string length
            emitter.label("__rt_unser_allowed_tostring_finish_x");
            emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", previous_handler_offset)); // reload previous exception-handler head
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_exc_handler_top", 0);
            emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", boundary_bytes - TRY_HANDLER_DIAG_DEPTH_OFFSET)); // restore diagnostic suppression
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_rt_diag_suppression", 0);
            emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                // recover conversion string pointer
            emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");               // recover conversion string length
            emitter.instruction("leave");                                       // release protected frame and restore caller frame
            emitter.instruction("ret");                                         // return string pair or null pointer
            emitter.label("__rt_unser_allowed_tostring_throw_x");
            emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", previous_handler_offset)); // reload handler preceding this boundary
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_exc_handler_top", 0);
            emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", boundary_bytes - TRY_HANDLER_DIAG_DEPTH_OFFSET)); // restore diagnostic suppression skipped by longjmp
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_rt_diag_suppression", 0);
            emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", boundary_bytes - TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET)); // restore stack accounting skipped by longjmp
            crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_runtime_recursion_stack_bytes", 0);
            emitter.instruction("xor eax, eax");                                // cleanup ignores placeholder result on throw
            emitter.instruction("call __rt_unserialize_end");                   // close this opened context exactly once
            emitter.instruction("leave");                                       // discard protected invocation frame
            emitter.instruction("jmp __rt_throw_current");                      // resume propagation at caller's handler
        }
    }
}

/// Emits the per-call `allowed_classes` option parser and class-membership gate.
fn emit_unserialize_allowed_classes(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_unserialize_allowed_classes_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: unserialize_allowed_classes ---");
    emitter.label_global("__rt_unserialize_set_options_mixed");
    emitter.instruction("cbnz x0, __rt_unser_options_mixed_nonnull");           // keep the conditional branch inside the mixed-options entry atom
    emitter.instruction("b __rt_unser_options_type_error");                     // route null through the shared TypeError path unconditionally
    emitter.label("__rt_unser_options_mixed_nonnull");
    emitter.instruction("ldr x9, [x0]");                                        // inspect the Mixed runtime tag before any payload dereference
    emitter.instruction("cmp x9, #4");                                          // Mixed(indexed array) is valid default options
    emitter.instruction("b.ne __rt_unser_options_mixed_assoc");                 // keep the conditional branch on an assembler-local target
    emitter.instruction("b __rt_unserialize_set_options_indexed");              // enter the exported indexed-options helper unconditionally
    emitter.label("__rt_unser_options_mixed_assoc");
    emitter.instruction("cmp x9, #5");                                          // Mixed(assoc array) can represent the options map
    emitter.instruction("b.eq __rt_unser_options_mixed_assoc_valid");           // keep the conditional branch inside the mixed-options entry atom
    emitter.instruction("b __rt_unser_options_type_error");                     // route scalar/object shapes through the shared TypeError path
    emitter.label("__rt_unser_options_mixed_assoc_valid");
    emitter.instruction("ldr x0, [x0, #8]");                                    // unwrap the validated associative-hash payload
    emitter.instruction("b __rt_unserialize_set_options");                      // scan a trusted options hash

    emitter.label_global("__rt_unserialize_set_options_indexed");
    emitter.instruction("ret");                                                 // indexed options have no string keys; preserve defaults

    emitter.label_global("__rt_unserialize_set_options");
    emitter.instruction("sub sp, sp, #80");                                     // reserve source, normalized-list, cursor, and conversion spills
    emitter.instruction("stp x29, x30, [sp, #64]");                             // preserve caller state across allocation, iteration, and user conversion
    emitter.instruction("add x29, sp, #64");                                    // retain a conventional aligned options-decoder frame
    emitter.instruction("mov x9, x0");                                          // preserve raw associative options hash
    crate::codegen_support::abi::emit_symbol_address(emitter, "x1", "_unser_allowed_classes_key");
    emitter.instruction("mov x2, #15");                                         // byte length of allowed_classes
    emitter.instruction("mov x0, x9");                                          // hash_get receiver
    emitter.instruction("bl __rt_hash_get");                                    // x0=found, x1=payload, x3=tag
    emitter.instruction("cbz x0, __rt_unser_options_done");                     // absent option preserves default allow-all policy
    emitter.instruction("cmp x3, #7");                                          // heterogeneous option hashes store a boxed Mixed value
    emitter.instruction("b.ne __rt_unser_options_unboxed");                     // direct typed payload needs no unwrap
    emitter.instruction("ldr x3, [x1]");                                        // recover boxed value tag
    emitter.instruction("ldr x1, [x1, #8]");                                    // recover boxed value payload
    emitter.label("__rt_unser_options_unboxed");
    emitter.instruction("cmp x3, #3");                                          // boolean option?
    emitter.instruction("b.ne __rt_unser_options_list");                        // arrays are allow-lists
    emitter.instruction("cbnz x1, __rt_unser_options_done");                    // true preserves allow-all policy
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_mode");
    emitter.instruction("mov x10, #1");                                         // mode 1 = no class hydration
    emitter.instruction("str x10, [x9]");                                       // block every serialized object
    emitter.instruction("b __rt_unser_options_done");                           // restore the helper frame through the shared return path
    emitter.label("__rt_unser_options_list");
    emitter.instruction("cmp x3, #4");                                          // indexed array allow-list?
    emitter.instruction("b.eq __rt_unser_options_indexed_list_setup");          // normalize packed values in index order
    emitter.instruction("cmp x3, #5");                                          // associative array allow-list?
    emitter.instruction("b.eq __rt_unser_options_assoc_list");                  // keep the conditional branch inside the options-decoder atom
    emitter.instruction("b __rt_unser_allowed_classes_policy_type_error");      // reject non-array policies through the shared TypeError path
    emitter.label("__rt_unser_options_assoc_list");
    emitter.instruction("str x1, [sp]");                                        // preserve associative source hash across allocation
    emitter.instruction("mov x0, x1");                                          // hash_count receiver
    emitter.instruction("bl __rt_hash_count");                                  // capacity equals the number of values to normalize
    emitter.instruction("mov x9, #5");                                          // source kind 5 = associative hash
    emitter.instruction("b __rt_unser_options_list_allocate");                  // share normalized string-array allocation
    emitter.label("__rt_unser_options_indexed_list_setup");
    emitter.instruction("str x1, [sp]");                                        // preserve indexed source array across allocation
    emitter.instruction("ldr x0, [x1]");                                        // capacity equals the packed logical length
    emitter.instruction("mov x9, #4");                                          // source kind 4 = indexed array
    emitter.label("__rt_unser_options_list_allocate");
    emitter.instruction("str x9, [sp, #24]");                                   // remember source shape for the iterator dispatch
    emitter.instruction("mov x1, #16");                                         // normalized entries are direct string pointer/length pairs
    emitter.instruction("bl __rt_array_new");                                   // allocate the context-owned normalized allow-list
    emitter.instruction("str x0, [sp, #8]");                                    // retain normalized owner across validation calls
    emitter.instruction("str xzr, [sp, #16]");                                  // start packed index/hash cursor at zero
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list_mixed");
    emitter.instruction("str xzr, [x9]");                                       // direct string pointer/length cells
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_mode");
    emitter.instruction("mov x10, #2");                                         // mode 2 = named allow-list
    emitter.instruction("str x10, [x9]");                                       // publish policy before validation so error cleanup owns the partial list
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list");
    emitter.instruction("str x0, [x9]");                                        // publish the fresh normalized array owner
    emitter.instruction("ldr x9, [sp, #24]");                                   // select packed or associative value traversal
    emitter.instruction("cmp x9, #5");                                          // associative source?
    emitter.instruction("b.eq __rt_unser_options_hash_list_loop");              // hash keys are ignored; values supply class names

    emitter.label("__rt_unser_options_indexed_list_loop");
    emitter.instruction("ldr x9, [sp]");                                        // indexed source array
    emitter.instruction("ldr x10, [sp, #16]");                                  // packed element index
    emitter.instruction("ldr x11, [x9]");                                       // packed logical length
    emitter.instruction("cmp x10, x11");                                        // normalized every packed value?
    emitter.instruction("b.hs __rt_unser_options_done");                        // publish is already complete
    emitter.instruction("add x11, x10, #1");                                    // advance index before any nested helper call
    emitter.instruction("str x11, [sp, #16]");                                  // preserve next index across conversion helpers
    emitter.instruction("ldr x11, [x9, #-8]");                                  // packed array metadata precedes the payload header
    emitter.instruction("ubfx x11, x11, #8, #7");                               // extract the runtime element tag
    emitter.instruction("cmp x11, #1");                                         // direct string payload?
    emitter.instruction("b.eq __rt_unser_options_indexed_string");
    emitter.instruction("cmp x11, #6");                                         // direct object pointer payload?
    emitter.instruction("b.eq __rt_unser_options_indexed_object");
    emitter.instruction("cmp x11, #7");                                         // heterogeneous packed values use boxed Mixed cells
    emitter.instruction("b.eq __rt_unser_options_indexed_mixed");               // keep the conditional branch inside the options-decoder atom
    emitter.instruction("b __rt_unser_allowed_classes_entry_metadata_error");   // reject other homogeneous types through the shared error path
    emitter.label("__rt_unser_options_indexed_mixed");
    emitter.instruction("add x12, x9, #24");                                    // skip indexed header to the packed payload base
    emitter.instruction("ldr x12, [x12, x10, lsl #3]");                         // read the selected boxed Mixed pointer
    emitter.instruction("cbnz x12, __rt_unser_options_indexed_cell_ready");     // keep the conditional branch inside the options-decoder atom
    emitter.instruction("b __rt_unser_allowed_classes_entry_null_error");       // report a malformed/null cell through the shared error path
    emitter.label("__rt_unser_options_indexed_cell_ready");
    emitter.instruction("ldr x13, [x12]");                                      // boxed runtime tag
    emitter.instruction("ldr x1, [x12, #8]");                                   // boxed low payload
    emitter.instruction("ldr x2, [x12, #16]");                                  // boxed high payload
    emitter.instruction("cmp x13, #1");                                         // boxed string entry?
    emitter.instruction("b.eq __rt_unser_options_append_borrowed");
    emitter.instruction("cmp x13, #6");                                         // boxed object entry?
    emitter.instruction("b.eq __rt_unser_options_boxed_object");
    emitter.instruction("mov x3, x13");                                         // forward invalid boxed runtime tag
    emitter.instruction("b __rt_unser_allowed_classes_entry_type_error");
    emitter.label("__rt_unser_options_indexed_string");
    emitter.instruction("add x9, x9, #24");                                     // direct payload begins after indexed header
    emitter.instruction("lsl x10, x10, #4");                                    // string pair stride is sixteen bytes
    emitter.instruction("add x9, x9, x10");                                     // select the current direct string pair
    emitter.instruction("ldp x1, x2, [x9]");                                    // borrow source string pointer and length
    emitter.instruction("b __rt_unser_options_append_borrowed");
    emitter.label("__rt_unser_options_indexed_object");
    emitter.instruction("add x12, x9, #24");                                    // skip indexed header to direct object slots
    emitter.instruction("ldr x0, [x12, x10, lsl #3]");                          // load selected direct object pointer
    emitter.instruction("cbnz x0, __rt_unser_options_indexed_object_ready");    // keep the conditional branch inside the options-decoder atom
    emitter.instruction("b __rt_unser_allowed_classes_entry_null_error");       // report a null object slot through the shared error path
    emitter.label("__rt_unser_options_indexed_object_ready");
    emitter.instruction("b __rt_unser_options_convert_object");
    emitter.label("__rt_unser_options_boxed_object");
    emitter.instruction("mov x0, x1");                                          // boxed low payload is the object pointer
    emitter.instruction("b __rt_unser_options_convert_object");

    emitter.label("__rt_unser_options_hash_list_loop");
    emitter.instruction("ldr x0, [sp]");                                        // associative source hash
    emitter.instruction("ldr x1, [sp, #16]");                                   // insertion-order iterator cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // return next borrowed value tuple; keys are intentionally ignored
    emitter.instruction("cmp x0, #-1");                                         // exhausted every associative value?
    emitter.instruction("b.eq __rt_unser_options_done");                        // normalized policy is ready
    emitter.instruction("str x0, [sp, #16]");                                   // preserve next iterator cursor
    emitter.instruction("cmp x5, #7");                                          // boxed Mixed hash value?
    emitter.instruction("b.ne __rt_unser_options_hash_unboxed");
    emitter.instruction("cbnz x3, __rt_unser_options_hash_cell_ready");         // keep the conditional branch inside the options-decoder atom
    emitter.instruction("b __rt_unser_allowed_classes_entry_null_error");       // reject a null boxed hash cell through the shared error path
    emitter.label("__rt_unser_options_hash_cell_ready");
    emitter.instruction("mov x12, x3");                                         // preserve boxed cell pointer for payload reads
    emitter.instruction("ldr x5, [x12]");                                       // unwrap actual value tag
    emitter.instruction("ldr x3, [x12, #8]");                                   // unwrap low payload
    emitter.instruction("ldr x4, [x12, #16]");                                  // unwrap high payload
    emitter.label("__rt_unser_options_hash_unboxed");
    emitter.instruction("cmp x5, #1");                                          // string hash value?
    emitter.instruction("b.eq __rt_unser_options_hash_string");
    emitter.instruction("cmp x5, #6");                                          // object hash value?
    emitter.instruction("b.eq __rt_unser_options_hash_object");
    emitter.instruction("mov x1, x3");                                          // forward invalid value payload for dynamic type naming
    emitter.instruction("mov x3, x5");                                          // forward invalid value runtime tag
    emitter.instruction("b __rt_unser_allowed_classes_entry_type_error");
    emitter.label("__rt_unser_options_hash_string");
    emitter.instruction("mov x1, x3");                                          // normalize iterator string pointer to push ABI
    emitter.instruction("mov x2, x4");                                          // normalize iterator string length to push ABI
    emitter.instruction("b __rt_unser_options_append_borrowed");
    emitter.label("__rt_unser_options_hash_object");
    emitter.instruction("mov x0, x3");                                          // iterator low payload is the object pointer

    emitter.label("__rt_unser_options_convert_object");
    emitter.instruction("str x0, [sp, #32]");                                   // preserve offending object for Error and result-owner cleanup
    emitter.instruction("bl __rt_unser_allowed_object_to_string");              // invoke __toString under an internal exception boundary
    emitter.instruction("cbnz x1, __rt_unser_options_object_string_ready");     // keep the conditional branch inside the options-decoder atom
    emitter.instruction("b __rt_unser_allowed_classes_entry_object_error");     // absent conversion enters the shared object Error path
    emitter.label("__rt_unser_options_object_string_ready");
    emitter.instruction("stp x1, x2, [sp, #40]");                               // preserve method-return pair across heap-kind classification
    emitter.instruction("mov x0, x1");                                          // classify ownership of the returned string payload
    emitter.instruction("bl __rt_heap_kind");                                   // kind 7 is transferred in place by str_persist inside push_str
    emitter.instruction("str x0, [sp, #56]");                                   // preserve original heap kind across normalized append
    emitter.instruction("ldp x1, x2, [sp, #40]");                               // reload method-return string pair for array_push_str
    emitter.instruction("ldr x0, [sp, #8]");                                    // normalized destination array
    emitter.instruction("bl __rt_array_push_str");                              // persist and append the converted class name
    emitter.instruction("str x0, [sp, #8]");                                    // retain possibly-grown destination
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list");
    emitter.instruction("str x0, [x9]");                                        // keep cleanup owner current after a possible growth
    emitter.instruction("ldr x9, [sp, #56]");                                   // recover original method-return heap kind
    emitter.instruction("cmp x9, #7");                                          // did push_str take over a concat temporary in place?
    emitter.instruction("b.eq __rt_unser_options_list_continue");               // transferred storage is now owned by the normalized array
    emitter.instruction("ldr x0, [sp, #40]");                                   // recover transient __toString return owner
    emitter.instruction("bl __rt_heap_free_safe");                              // release it after array_push_str copied the bytes
    emitter.instruction("b __rt_unser_options_list_continue");                  // process the next source value

    emitter.label("__rt_unser_options_append_borrowed");
    emitter.instruction("ldr x0, [sp, #8]");                                    // normalized destination array
    emitter.instruction("bl __rt_array_push_str");                              // persist and append borrowed source bytes
    emitter.instruction("str x0, [sp, #8]");                                    // retain possibly-grown destination
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list");
    emitter.instruction("str x0, [x9]");                                        // keep cleanup owner current after a possible growth
    emitter.label("__rt_unser_options_list_continue");
    emitter.instruction("ldr x9, [sp, #24]");                                   // resume the matching source traversal
    emitter.instruction("cmp x9, #5");                                          // associative source?
    emitter.instruction("b.eq __rt_unser_options_hash_list_loop");
    emitter.instruction("b __rt_unser_options_indexed_list_loop");              // otherwise continue packed traversal
    emitter.label("__rt_unser_options_done");
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore caller frame and link register after nested helpers
    emitter.instruction("add sp, sp, #80");                                     // release normalized-list decoder state
    emitter.instruction("ret");                                                 // caller proceeds to parsing

    emitter.label_global("__rt_unserialize_class_allowed");
    emitter.instruction("sub sp, sp, #64");                                     // save class name across list scans and comparisons
    emitter.instruction("stp x29, x30, [sp, #48]");                             // preserve helper frame/link
    emitter.instruction("add x29, sp, #48");                                    // establish stable frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // class name pointer
    emitter.instruction("str x1, [sp, #8]");                                    // class name byte length
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_mode");
    emitter.instruction("ldr x9, [x9]");                                        // current policy mode
    emitter.instruction("cbz x9, __rt_unser_class_allowed_yes");                // mode 0 = allow all
    emitter.instruction("cmp x9, #1");                                          // mode 1 = block all
    emitter.instruction("b.eq __rt_unser_class_allowed_no");                    // never hydrate blocked classes
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list");
    emitter.instruction("ldr x9, [x9]");                                        // indexed string-array payload
    emitter.instruction("cbz x9, __rt_unser_class_allowed_no");                 // malformed list fails closed
    emitter.instruction("ldr x10, [x9]");                                       // list length
    emitter.instruction("str x9, [sp, #16]");                                   // save list base across strcasecmp
    emitter.instruction("str x10, [sp, #24]");                                  // save list length
    emitter.instruction("mov x11, #0");                                         // list cursor
    emitter.label("__rt_unser_class_allowed_loop");
    emitter.instruction("ldr x10, [sp, #24]");                                  // list length
    emitter.instruction("cmp x11, x10");                                        // exhausted every allowed name?
    emitter.instruction("b.ge __rt_unser_class_allowed_no");                    // no exact match means incomplete object
    emitter.instruction("ldr x9, [sp, #16]");                                   // list base
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_unser_allowed_list_mixed");
    emitter.instruction("ldr x12, [x12]");                                      // heterogeneous allow-list representation?
    emitter.instruction("cbnz x12, __rt_unser_class_allowed_mixed_cell");
    emitter.instruction("add x9, x9, #24");                                     // skip indexed-array header
    emitter.instruction("lsl x10, x11, #4");                                    // direct string element stride = pointer + length
    emitter.instruction("add x9, x9, x10");
    emitter.instruction("b __rt_unser_class_allowed_compare");
    emitter.label("__rt_unser_class_allowed_mixed_cell");
    emitter.instruction("add x9, x9, #24");
    emitter.instruction("lsl x10, x11, #3");                                    // boxed Mixed elements are pointers
    emitter.instruction("ldr x9, [x9, x10]");
    emitter.instruction("add x9, x9, #8");                                      // string payload pair follows Mixed tag
    emitter.label("__rt_unser_class_allowed_compare");
    emitter.instruction("ldr x1, [sp, #0]");                                    // requested class-name pointer
    emitter.instruction("ldr x2, [sp, #8]");                                    // requested class-name length
    emitter.instruction("ldr x3, [x9]");                                        // allow-list string pointer
    emitter.instruction("ldr x4, [x9, #8]");                                    // allow-list string length
    emitter.instruction("str x11, [sp, #32]");                                  // preserve cursor across the string helper
    emitter.instruction("bl __rt_strcasecmp");                                  // PHP class names compare case-insensitively
    emitter.instruction("cbz x0, __rt_unser_class_allowed_yes");                // matching allow-list entry grants hydration
    emitter.instruction("ldr x11, [sp, #32]");                                  // restore cursor after helper call
    emitter.instruction("add x11, x11, #1");                                    // advance after mismatch
    emitter.instruction("b __rt_unser_class_allowed_loop");                     // scan next allowed class
    emitter.label("__rt_unser_class_allowed_yes");
    emitter.instruction("mov x0, #1");                                          // true = instantiate and hydrate
    emitter.instruction("b __rt_unser_class_allowed_return");                   // shared frame teardown
    emitter.label("__rt_unser_class_allowed_no");
    emitter.instruction("mov x0, #0");                                          // false = incomplete object without hooks
    emitter.label("__rt_unser_class_allowed_return");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore helper frame/link
    emitter.instruction("add sp, sp, #64");                                     // release saved inputs
    emitter.instruction("ret");                                                 // return allow decision

    emitter.label_shared("__rt_unser_options_type_error");
    emitter.instruction("cbz x0, __rt_unser_options_type_null_error");          // a null Mixed pointer reports null
    emitter.instruction("ldr x1, [x0, #8]");                                    // rejected runtime payload
    emitter.instruction("ldr x0, [x0]");                                        // rejected runtime tag
    emitter.instruction("b __rt_unser_options_type_dispatch");
    emitter.label("__rt_unser_options_type_null_error");
    emitter.instruction("mov x0, #8");                                          // runtime null tag
    emitter.instruction("mov x1, #0");                                          // null has no payload
    emitter.label("__rt_unser_options_type_dispatch");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x2", "_unser_options_type_prefix");
    emitter.instruction(&format!("mov x3, #{}", UNSER_OPTIONS_TYPE_PREFIX.len())); // diagnostic prefix byte length
    emitter.instruction("b __rt_unser_throw_type_error");                       // helper closes context and throws exactly once

    emitter.label_shared("__rt_unser_allowed_classes_entry_null_error");
    emitter.instruction("mov x3, #8");                                          // invalid list entry is null
    emitter.instruction("mov x1, #0");                                          // null has no payload
    emitter.instruction("b __rt_unser_allowed_classes_entry_type_error");
    emitter.label_shared("__rt_unser_allowed_classes_entry_metadata_error");
    emitter.instruction("mov x3, x11");                                         // packed element metadata uses the runtime tag numbering
    emitter.instruction("mov x1, #0");                                          // homogeneous metadata has no single payload
    emitter.label_shared("__rt_unser_allowed_classes_entry_type_error");
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // remove the options decoder frame before unwinding
    emitter.instruction("add sp, sp, #80");                                     // restore the caller stack before context cleanup
    emitter.instruction("mov x0, x3");                                          // rejected entry runtime tag
    crate::codegen_support::abi::emit_symbol_address(emitter, "x2", "_unser_allowed_classes_entry_prefix");
    emitter.instruction(&format!("mov x3, #{}", UNSER_ALLOWED_CLASSES_ENTRY_PREFIX.len())); // diagnostic prefix byte length
    emitter.instruction("b __rt_unser_throw_type_error");                       // helper closes context and throws exactly once

    emitter.label_shared("__rt_unser_allowed_classes_entry_object_error");
    emitter.instruction("ldr x0, [sp, #32]");                                   // recover offending object while source storage remains live
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // remove options decoder frame before unwinding
    emitter.instruction("add sp, sp, #80");                                     // restore caller stack before Error construction
    emitter.instruction("b __rt_unser_throw_object_string_error");              // helper resolves class name, closes context once, and throws Error

    emitter.label_shared("__rt_unser_allowed_classes_policy_type_error");
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // remove the options decoder frame before unwinding
    emitter.instruction("add sp, sp, #80");                                     // restore caller stack before the common cleanup helper
    emitter.instruction("mov x0, x3");                                          // rejected policy runtime tag
    crate::codegen_support::abi::emit_symbol_address(emitter, "x2", "_unser_allowed_classes_policy_prefix");
    emitter.instruction(&format!("mov x3, #{}", UNSER_ALLOWED_CLASSES_POLICY_PREFIX.len())); // diagnostic prefix byte length
    emitter.instruction("b __rt_unser_throw_type_error");                       // helper closes context and throws exactly once
}

/// Emits the x86_64 version of the unserialize allowed-class policy helpers.
fn emit_unserialize_allowed_classes_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unserialize_allowed_classes ---");
    emitter.label_global("__rt_unserialize_set_options_mixed");
    emitter.instruction("test rax, rax");                                       // null Mixed cannot carry an options hash
    emitter.instruction("jz __rt_unser_options_type_error_x");                  // null is not a valid options array
    emitter.instruction("cmp QWORD PTR [rax], 4");                              // Mixed(indexed array) is valid default options
    emitter.instruction("jne __rt_unser_options_mixed_assoc_x");                // keep the conditional jump on an assembler-local target
    emitter.instruction("jmp __rt_unserialize_set_options_indexed");            // enter the exported indexed-options helper unconditionally
    emitter.label("__rt_unser_options_mixed_assoc_x");
    emitter.instruction("cmp QWORD PTR [rax], 5");                              // only Mixed(assoc array) can represent the options map
    emitter.instruction("jne __rt_unser_options_type_error_x");                 // reject other tags before payload reads
    emitter.instruction("mov rax, QWORD PTR [rax + 8]");                        // unwrap the validated associative-hash payload
    emitter.instruction("jmp __rt_unserialize_set_options");                    // scan a trusted options hash

    emitter.label_global("__rt_unserialize_set_options_indexed");
    emitter.instruction("ret");                                                 // indexed options have no allowed_classes key

    emitter.label_global("__rt_unserialize_set_options");
    emitter.instruction("push rbp");                                            // preserve the caller frame while options helpers run
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame for normalized policy construction
    emitter.instruction("sub rsp, 64");                                         // reserve source, destination, cursor, kind, and conversion ownership spills
    emitter.instruction("mov rdi, rax");                                        // hash_get receiver
    emitter.instruction("lea rsi, [rip + _unser_allowed_classes_key]");         // options key pointer
    emitter.instruction("mov rdx, 15");                                         // options key length
    emitter.instruction("call __rt_hash_get");                                  // rax=found, rdi=payload, rcx=tag
    emitter.instruction("test rax, rax");                                       // did options include allowed_classes?
    emitter.instruction("jz __rt_unser_options_done_x");                        // absent option preserves allow-all policy
    emitter.instruction("cmp rcx, 7");                                          // heterogeneous option hashes store a boxed Mixed value
    emitter.instruction("jne __rt_unser_options_unboxed_x");                    // direct typed payload needs no unwrap
    emitter.instruction("mov rcx, QWORD PTR [rdi]");                            // recover boxed value tag
    emitter.instruction("mov rdi, QWORD PTR [rdi + 8]");                        // recover boxed value payload
    emitter.label("__rt_unser_options_unboxed_x");
    emitter.instruction("cmp rcx, 3");                                          // boolean option?
    emitter.instruction("jne __rt_unser_options_list_x");                       // arrays are allow-lists
    emitter.instruction("test rdi, rdi");                                       // false blocks all object hydration
    emitter.instruction("jnz __rt_unser_options_done_x");                       // true preserves allow-all policy
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_mode], 1");        // mode 1 = no class hydration
    emitter.instruction("jmp __rt_unser_options_done_x");                       // restore the options helper frame
    emitter.label("__rt_unser_options_list_x");
    emitter.instruction("cmp rcx, 4");                                          // indexed array allow-list?
    emitter.instruction("je __rt_unser_options_indexed_list_setup_x");          // normalize packed values in index order
    emitter.instruction("cmp rcx, 5");                                          // associative array allow-list?
    emitter.instruction("jne __rt_unser_allowed_classes_policy_type_error_x");  // PHP accepts only bool or either PHP array shape
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve associative source hash across allocation
    emitter.instruction("call __rt_hash_count");                                // capacity equals the number of values to normalize
    emitter.instruction("mov rdi, rax");                                        // array_new capacity argument
    emitter.instruction("mov r10, 5");                                          // source kind 5 = associative hash
    emitter.instruction("jmp __rt_unser_options_list_allocate_x");              // share normalized string-array allocation
    emitter.label("__rt_unser_options_indexed_list_setup_x");
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve indexed source array across allocation
    emitter.instruction("mov r10, 4");                                          // source kind 4 = indexed array
    emitter.instruction("mov rdi, QWORD PTR [rdi]");                            // capacity equals packed logical length
    emitter.label("__rt_unser_options_list_allocate_x");
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // remember source shape for iterator dispatch
    emitter.instruction("mov rsi, 16");                                         // normalized entries are direct string pairs
    emitter.instruction("call __rt_array_new");                                 // allocate context-owned normalized allow-list
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // retain normalized owner across validation calls
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // start packed index/hash cursor at zero
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list_mixed], 0");  // membership scans direct string pairs only
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_mode], 2");        // publish named allow-list mode before validation
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list], rax");      // errors now release the partial normalized owner through end
    emitter.instruction("cmp QWORD PTR [rbp - 32], 5");                         // associative source?
    emitter.instruction("je __rt_unser_options_hash_list_loop_x");              // hash keys are ignored; values supply class names

    emitter.label("__rt_unser_options_indexed_list_loop_x");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // indexed source array
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // packed element index
    emitter.instruction("cmp r9, QWORD PTR [r8]");                              // normalized every packed value?
    emitter.instruction("jae __rt_unser_options_done_x");                       // policy is already published
    emitter.instruction("lea r10, [r9 + 1]");                                   // advance index before nested helper calls
    emitter.instruction("mov QWORD PTR [rbp - 24], r10");                       // preserve next index across conversion helpers
    emitter.instruction("mov r11, QWORD PTR [r8 - 8]");                         // packed array metadata precedes payload header
    emitter.instruction("shr r11, 8");                                          // move runtime element tag to low bits
    emitter.instruction("and r11, 0x7f");                                       // isolate seven-bit runtime element tag
    emitter.instruction("cmp r11, 1");                                          // direct string payload?
    emitter.instruction("je __rt_unser_options_indexed_string_x");
    emitter.instruction("cmp r11, 6");                                          // direct object pointer payload?
    emitter.instruction("je __rt_unser_options_indexed_object_x");
    emitter.instruction("cmp r11, 7");                                          // heterogeneous packed values use boxed Mixed cells
    emitter.instruction("jne __rt_unser_allowed_classes_entry_metadata_error_x"); // other homogeneous values are invalid class names
    emitter.instruction("mov r10, QWORD PTR [r8 + r9 * 8 + 24]");               // selected boxed Mixed pointer
    emitter.instruction("test r10, r10");                                       // null/malformed cell?
    emitter.instruction("jz __rt_unser_allowed_classes_entry_null_error_x");    // report PHP null
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // boxed runtime tag
    emitter.instruction("mov rsi, QWORD PTR [r10 + 8]");                        // boxed low payload
    emitter.instruction("mov rdx, QWORD PTR [r10 + 16]");                       // boxed high payload
    emitter.instruction("cmp r11, 1");                                          // boxed string entry?
    emitter.instruction("je __rt_unser_options_append_borrowed_x");
    emitter.instruction("cmp r11, 6");                                          // boxed object entry?
    emitter.instruction("je __rt_unser_options_boxed_object_x");
    emitter.instruction("mov rax, r11");                                        // forward invalid boxed runtime tag
    emitter.instruction("mov rdi, rsi");                                        // forward invalid boxed low payload
    emitter.instruction("jmp __rt_unser_allowed_classes_entry_type_error_x");
    emitter.label("__rt_unser_options_indexed_string_x");
    emitter.instruction("mov r10, r9");                                         // copy index before scaling
    emitter.instruction("shl r10, 4");                                          // direct string-pair stride is sixteen bytes
    emitter.instruction("lea r10, [r8 + r10 + 24]");                            // select current direct string pair
    emitter.instruction("mov rsi, QWORD PTR [r10]");                            // borrow source string pointer
    emitter.instruction("mov rdx, QWORD PTR [r10 + 8]");                        // borrow source string length
    emitter.instruction("jmp __rt_unser_options_append_borrowed_x");
    emitter.label("__rt_unser_options_indexed_object_x");
    emitter.instruction("mov rax, QWORD PTR [r8 + r9 * 8 + 24]");               // load selected direct object pointer
    emitter.instruction("test rax, rax");                                       // null object slot?
    emitter.instruction("jz __rt_unser_allowed_classes_entry_null_error_x");    // report PHP null
    emitter.instruction("jmp __rt_unser_options_convert_object_x");
    emitter.label("__rt_unser_options_boxed_object_x");
    emitter.instruction("mov rax, rsi");                                        // boxed low payload is object pointer
    emitter.instruction("jmp __rt_unser_options_convert_object_x");

    emitter.label("__rt_unser_options_hash_list_loop_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // associative source hash
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // insertion-order iterator cursor
    emitter.instruction("call __rt_hash_iter_next");                            // return next borrowed value tuple; ignore key registers
    emitter.instruction("cmp rax, -1");                                         // exhausted every associative value?
    emitter.instruction("je __rt_unser_options_done_x");                        // normalized policy is ready
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // preserve next iterator cursor
    emitter.instruction("cmp r9, 7");                                           // boxed Mixed hash value?
    emitter.instruction("jne __rt_unser_options_hash_unboxed_x");
    emitter.instruction("test rcx, rcx");                                       // null boxed cell?
    emitter.instruction("jz __rt_unser_allowed_classes_entry_null_error_x");    // report PHP null
    emitter.instruction("mov r10, rcx");                                        // preserve boxed cell pointer for payload reads
    emitter.instruction("mov r9, QWORD PTR [r10]");                             // unwrap actual value tag
    emitter.instruction("mov rcx, QWORD PTR [r10 + 8]");                        // unwrap low payload
    emitter.instruction("mov r8, QWORD PTR [r10 + 16]");                        // unwrap high payload
    emitter.label("__rt_unser_options_hash_unboxed_x");
    emitter.instruction("cmp r9, 1");                                           // string hash value?
    emitter.instruction("je __rt_unser_options_hash_string_x");
    emitter.instruction("cmp r9, 6");                                           // object hash value?
    emitter.instruction("je __rt_unser_options_hash_object_x");
    emitter.instruction("mov rax, r9");                                         // forward invalid value runtime tag
    emitter.instruction("mov rdi, rcx");                                        // forward invalid low payload
    emitter.instruction("jmp __rt_unser_allowed_classes_entry_type_error_x");
    emitter.label("__rt_unser_options_hash_string_x");
    emitter.instruction("mov rsi, rcx");                                        // normalize iterator string pointer to push ABI
    emitter.instruction("mov rdx, r8");                                         // normalize iterator string length to push ABI
    emitter.instruction("jmp __rt_unser_options_append_borrowed_x");
    emitter.label("__rt_unser_options_hash_object_x");
    emitter.instruction("mov rax, rcx");                                        // iterator low payload is object pointer

    emitter.label("__rt_unser_options_convert_object_x");
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // preserve offending object for Error and owner cleanup
    emitter.instruction("call __rt_unser_allowed_object_to_string");            // invoke __toString under internal exception boundary
    emitter.instruction("test rax, rax");                                       // did the class expose __toString?
    emitter.instruction("jz __rt_unser_allowed_classes_entry_object_error_x");  // absent method raises PHP conversion Error
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // preserve transient method-return owner across append
    emitter.instruction("mov QWORD PTR [rbp - 48], rdx");                       // preserve returned byte length while classifying ownership
    emitter.instruction("call __rt_heap_kind");                                 // kind 7 is transferred in place by str_persist inside push_str
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // preserve original heap kind across normalized append
    emitter.instruction("mov rsi, QWORD PTR [rbp - 40]");                       // reload array_push_str string pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // reload array_push_str string length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // normalized destination array
    emitter.instruction("call __rt_array_push_str");                            // persist and append converted class name
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // retain possibly-grown destination
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list], rax");      // keep cleanup owner current after possible growth
    emitter.instruction("cmp QWORD PTR [rbp - 56], 7");                         // did push_str take over a concat temporary in place?
    emitter.instruction("je __rt_unser_options_list_continue_x");               // transferred storage is now owned by the normalized array
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // recover transient __toString return owner
    emitter.instruction("call __rt_heap_free_safe");                            // release it after array_push_str copied the bytes
    emitter.instruction("jmp __rt_unser_options_list_continue_x");

    emitter.label("__rt_unser_options_append_borrowed_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // normalized destination array
    emitter.instruction("call __rt_array_push_str");                            // persist and append borrowed source bytes
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // retain possibly-grown destination
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list], rax");      // keep cleanup owner current after possible growth
    emitter.label("__rt_unser_options_list_continue_x");
    emitter.instruction("cmp QWORD PTR [rbp - 32], 5");                         // resume the matching source traversal
    emitter.instruction("je __rt_unser_options_hash_list_loop_x");              // continue associative values
    emitter.instruction("jmp __rt_unser_options_indexed_list_loop_x");          // otherwise continue packed values
    emitter.label("__rt_unser_options_done_x");
    emitter.instruction("leave");                                               // restore the caller frame after policy normalization helpers
    emitter.instruction("ret");                                                 // caller proceeds to parsing

    emitter.label_global("__rt_unserialize_class_allowed");
    emitter.instruction("push rbp");                                            // preserve caller frame
    emitter.instruction("mov rbp, rsp");                                        // establish helper frame
    emitter.instruction("sub rsp, 48");                                         // class name plus list scan state
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // class name pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // class name length
    emitter.instruction("mov r8, QWORD PTR [rip + _unser_allowed_mode]");       // current policy mode
    emitter.instruction("test r8, r8");                                         // mode 0 = allow all
    emitter.instruction("jz __rt_unser_class_allowed_yes_x");                   // hydrate unrestricted classes
    emitter.instruction("cmp r8, 1");                                           // mode 1 = block all
    emitter.instruction("je __rt_unser_class_allowed_no_x");                    // never hydrate blocked classes
    emitter.instruction("mov r8, QWORD PTR [rip + _unser_allowed_list]");       // indexed string-array payload
    emitter.instruction("test r8, r8");                                         // malformed list fails closed
    emitter.instruction("jz __rt_unser_class_allowed_no_x");                    // no list cannot grant hydration
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // list length
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // save list base across strcmp
    emitter.instruction("mov QWORD PTR [rbp - 32], r9");                        // save list length
    emitter.instruction("xor r10d, r10d");                                      // list cursor
    emitter.label("__rt_unser_class_allowed_loop_x");
    emitter.instruction("cmp r10, QWORD PTR [rbp - 32]");                       // exhausted every allowed name?
    emitter.instruction("jae __rt_unser_class_allowed_no_x");                   // no exact match means incomplete object
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // list base
    emitter.instruction("cmp QWORD PTR [rip + _unser_allowed_list_mixed], 0");  // does this list contain boxed Mixed strings?
    emitter.instruction("jne __rt_unser_class_allowed_mixed_cell_x");
    emitter.instruction("mov rax, r10");                                        // preserve the list cursor while deriving a byte offset
    emitter.instruction("shl rax, 4");                                          // scale the cursor by the 16-byte direct-string pair stride
    emitter.instruction("add r11, rax");                                        // advance the list base to the selected direct-string pair
    emitter.instruction("add r11, 24");                                         // skip the indexed-array header before reading the string pair
    emitter.instruction("jmp __rt_unser_class_allowed_compare_x");
    emitter.label("__rt_unser_class_allowed_mixed_cell_x");
    emitter.instruction("mov r11, QWORD PTR [r11 + r10 * 8 + 24]");             // validated boxed string cell
    emitter.instruction("add r11, 8");                                          // string payload pair follows its Mixed tag
    emitter.label("__rt_unser_class_allowed_compare_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // requested class-name pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // requested class-name length
    emitter.instruction("mov rdx, QWORD PTR [r11]");                            // allow-list string pointer
    emitter.instruction("mov rcx, QWORD PTR [r11 + 8]");                        // allow-list string length
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // preserve cursor across the string helper
    emitter.instruction("call __rt_strcasecmp");                                // PHP class names compare case-insensitively
    emitter.instruction("test rax, rax");                                       // comparison result
    emitter.instruction("jz __rt_unser_class_allowed_yes_x");                   // matching allow-list entry grants hydration
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // restore cursor after helper call
    emitter.instruction("add r10, 1");                                          // advance after mismatch
    emitter.instruction("jmp __rt_unser_class_allowed_loop_x");                 // scan next allowed class
    emitter.label("__rt_unser_class_allowed_yes_x");
    emitter.instruction("mov eax, 1");                                          // true = instantiate and hydrate
    emitter.instruction("jmp __rt_unser_class_allowed_return_x");               // shared frame teardown
    emitter.label("__rt_unser_class_allowed_no_x");
    emitter.instruction("xor eax, eax");                                        // false = incomplete object without hooks
    emitter.label("__rt_unser_class_allowed_return_x");
    emitter.instruction("leave");                                               // restore caller frame and stack
    emitter.instruction("ret");                                                 // return allow decision

    emitter.label("__rt_unser_options_type_error_x");
    emitter.instruction("test rax, rax");                                       // a null Mixed pointer reports null
    emitter.instruction("jz __rt_unser_options_type_null_error_x");
    emitter.instruction("mov rdi, QWORD PTR [rax + 8]");                        // rejected runtime payload
    emitter.instruction("mov rax, QWORD PTR [rax]");                            // rejected runtime tag
    emitter.instruction("jmp __rt_unser_options_type_dispatch_x");
    emitter.label("__rt_unser_options_type_null_error_x");
    emitter.instruction("mov eax, 8");                                          // runtime null tag
    emitter.instruction("xor edi, edi");                                        // null has no payload
    emitter.label("__rt_unser_options_type_dispatch_x");
    emitter.instruction("lea rsi, [rip + _unser_options_type_prefix]");         // options diagnostic prefix
    emitter.instruction(&format!("mov rdx, {}", UNSER_OPTIONS_TYPE_PREFIX.len())); // prefix byte length
    emitter.instruction("jmp __rt_unser_throw_type_error");                     // helper closes context and throws exactly once

    emitter.label("__rt_unser_allowed_classes_entry_null_error_x");
    emitter.instruction("mov eax, 8");                                          // forward the invalid null-entry tag directly
    emitter.instruction("xor edi, edi");                                        // null has no payload
    emitter.instruction("jmp __rt_unser_allowed_classes_entry_type_error_x");
    emitter.label("__rt_unser_allowed_classes_entry_metadata_error_x");
    emitter.instruction("mov rax, r11");                                        // packed element metadata uses runtime tag numbering
    emitter.instruction("xor edi, edi");                                        // homogeneous metadata has no single payload
    emitter.label("__rt_unser_allowed_classes_entry_type_error_x");
    emitter.instruction("leave");                                               // remove options decoder frame before common cleanup
    emitter.instruction("lea rsi, [rip + _unser_allowed_classes_entry_prefix]"); // list-entry diagnostic prefix
    emitter.instruction(&format!("mov rdx, {}", UNSER_ALLOWED_CLASSES_ENTRY_PREFIX.len())); // prefix byte length
    emitter.instruction("jmp __rt_unser_throw_type_error");                     // helper closes context and throws exactly once

    emitter.label("__rt_unser_allowed_classes_entry_object_error_x");
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // recover offending object while source storage remains live
    emitter.instruction("leave");                                               // remove options decoder frame before Error construction
    emitter.instruction("jmp __rt_unser_throw_object_string_error");            // helper resolves class name, closes context once, and throws Error

    emitter.label("__rt_unser_allowed_classes_policy_type_error_x");
    emitter.instruction("mov rax, rcx");                                        // rejected policy runtime tag
    emitter.instruction("leave");                                               // remove options decoder frame before common cleanup
    emitter.instruction("lea rsi, [rip + _unser_allowed_classes_policy_prefix]"); // policy diagnostic prefix
    emitter.instruction(&format!("mov rdx, {}", UNSER_ALLOWED_CLASSES_POLICY_PREFIX.len())); // prefix byte length
    emitter.instruction("jmp __rt_unser_throw_type_error");                     // helper closes context and throws exactly once
}

/// Emits the AArch64 begin/end helpers that isolate one active unserialize call.
///
/// Nested calls snapshot the outer policy, parser depth, and populated reference
/// slots into a linked heap context. The end helper preserves the parsed result,
/// releases the current call's owned allow-list, and restores the outer context.
fn emit_unserialize_context_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unserialize context begin/end ---");
    emitter.label_global("__rt_unserialize_begin");
    emitter.instruction("sub sp, sp, #32");                                     // reserve spills plus an ABI-aligned helper frame
    emitter.instruction("stp x29, x30, [sp, #16]");                             // preserve the caller frame and return address across allocation
    emitter.instruction("add x29, sp, #16");                                    // establish a stable frame for snapshot sizing
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_active");
    emitter.instruction("ldr x10, [x9]");                                       // load the active unserialize nesting count
    emitter.instruction("cbnz x10, __rt_unserialize_begin_nested");             // snapshot state only when a parser is already active
    emitter.instruction("mov x10, #1");                                         // mark the top-level parser active
    emitter.instruction("str x10, [x9]");                                       // publish the top-level nesting count
    emitter.instruction("b __rt_unserialize_begin_reset");                      // initialize the fresh per-call state

    emitter.label("__rt_unserialize_begin_nested");
    emitter.instruction("cmp x10, #256");                                       // has reentrant parser nesting reached its hard limit?
    emitter.instruction("b.lo __rt_unserialize_begin_nested_in_budget");        // keep the conditional branch inside the context helper atom
    emitter.instruction("b __rt_unser_depth_fatal");                            // reject another snapshot through the shared fatal path
    emitter.label("__rt_unserialize_begin_nested_in_budget");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_count");
    emitter.instruction("ldr x10, [x9]");                                       // load the outer registry's logical value count
    emitter.instruction("str x10, [sp]");                                       // preserve the logical count across context allocation
    emitter.instruction("mov x11, #65536");                                     // materialize the fixed registry capacity
    emitter.instruction("cmp x10, x11");                                        // does the logical count exceed the physical registry?
    emitter.instruction("csel x11, x10, x11, lo");                              // copy only the populated in-bounds registry prefix
    emitter.instruction("str x11, [sp, #8]");                                   // preserve the copy count across allocation
    emitter.instruction("lsl x0, x11, #3");                                     // convert the copied slot count to bytes
    emitter.instruction("add x0, x0, #56");                                     // include the seven-word context header
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the linked reentrant context snapshot
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_context");
    emitter.instruction("ldr x12, [x9]");                                       // load the previous context link
    emitter.instruction("str x12, [x0]");                                       // context.prev = previous context
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_unser_allowed_mode");
    emitter.instruction("ldr x13, [x12]");                                      // load the outer allowed-class mode
    emitter.instruction("str x13, [x0, #8]");                                   // snapshot the outer allowed-class mode
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_unser_allowed_list");
    emitter.instruction("ldr x13, [x12]");                                      // load the outer context-owned allow-list
    emitter.instruction("str x13, [x0, #16]");                                  // move the outer allow-list reference into the snapshot
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_unser_allowed_list_mixed");
    emitter.instruction("ldr x13, [x12]");                                      // load the outer list representation flag
    emitter.instruction("str x13, [x0, #24]");                                  // snapshot the outer list representation flag
    emitter.instruction("ldr x13, [sp]");                                       // recover the outer logical registry count
    emitter.instruction("str x13, [x0, #32]");                                  // snapshot the logical registry count
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_unser_depth");
    emitter.instruction("ldr x13, [x12]");                                      // load the outer recursive parser depth
    emitter.instruction("str x13, [x0, #40]");                                  // snapshot the outer parser depth
    emitter.instruction("ldr x11, [sp, #8]");                                   // recover the bounded registry copy count
    emitter.instruction("str x11, [x0, #48]");                                  // record how many registry slots follow the header
    crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_unser_values");
    emitter.instruction("mov x13, #0");                                         // start copying the populated registry prefix
    emitter.label("__rt_unserialize_begin_copy");
    emitter.instruction("cmp x13, x11");                                        // copied every in-bounds outer registry slot?
    emitter.instruction("b.hs __rt_unserialize_begin_copy_done");               // finish once the used prefix is preserved
    emitter.instruction("ldr x14, [x12, x13, lsl #3]");                         // load one outer reference-registry entry
    emitter.instruction("add x15, x0, #56");                                    // derive the snapshot registry payload base
    emitter.instruction("str x14, [x15, x13, lsl #3]");                         // save the outer reference-registry entry
    emitter.instruction("add x13, x13, #1");                                    // advance to the next populated slot
    emitter.instruction("b __rt_unserialize_begin_copy");                       // continue copying the used registry prefix
    emitter.label("__rt_unserialize_begin_copy_done");
    emitter.instruction("str x0, [x9]");                                        // publish this snapshot as the current context link
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_active");
    emitter.instruction("ldr x10, [x9]");                                       // reload the nesting count after allocation
    emitter.instruction("add x10, x10, #1");                                    // account for the nested parser
    emitter.instruction("str x10, [x9]");                                       // publish the incremented nesting count

    emitter.label("__rt_unserialize_begin_reset");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_mode");
    emitter.instruction("str xzr, [x9]");                                       // default this call to allow-all until options are installed
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list");
    emitter.instruction("str xzr, [x9]");                                       // this call starts without an owned allow-list
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list_mixed");
    emitter.instruction("str xzr, [x9]");                                       // default to direct-string list representation
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_count");
    emitter.instruction("str xzr, [x9]");                                       // reset this call's reference-registry count
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_depth");
    emitter.instruction("str xzr, [x9]");                                       // reset this call's recursive parser depth
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore the caller frame and return address
    emitter.instruction("add sp, sp, #32");                                     // release the begin helper frame
    emitter.instruction("ret");                                                 // enter the new isolated unserialize call

    emitter.label_global("__rt_unserialize_end");
    emitter.instruction("sub sp, sp, #48");                                     // reserve result/context spills plus an aligned helper frame
    emitter.instruction("stp x29, x30, [sp, #32]");                             // preserve the caller frame and return address across releases
    emitter.instruction("add x29, sp, #32");                                    // establish a stable frame for restoration
    emitter.instruction("str x0, [sp]");                                        // preserve the parsed Mixed result across cleanup
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list");
    emitter.instruction("ldr x0, [x9]");                                        // load this call's owned allow-list reference
    emitter.instruction("str xzr, [x9]");                                       // unpublish the list before releasing its ownership
    emitter.instruction("cbz x0, __rt_unserialize_end_list_done");              // skip refcount traffic when no allow-list was installed
    emitter.instruction("bl __rt_decref_array");                                // release this call's allow-list ownership
    emitter.label("__rt_unserialize_end_list_done");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_context");
    emitter.instruction("ldr x10, [x9]");                                       // load the outer snapshot, if this call was reentrant
    emitter.instruction("cbz x10, __rt_unserialize_end_top");                   // top-level completion has no outer state to restore
    emitter.instruction("str x10, [sp, #8]");                                   // preserve the snapshot pointer across heap release
    emitter.instruction("ldr x11, [x10]");                                      // load the previous linked context
    emitter.instruction("str x11, [x9]");                                       // pop the current context snapshot
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_unser_allowed_mode");
    emitter.instruction("ldr x12, [x10, #8]");                                  // recover the outer allowed-class mode
    emitter.instruction("str x12, [x11]");                                      // restore the outer allowed-class mode
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_unser_allowed_list");
    emitter.instruction("ldr x12, [x10, #16]");                                 // recover the outer owned allow-list reference
    emitter.instruction("str x12, [x11]");                                      // republish the outer owned allow-list
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_unser_allowed_list_mixed");
    emitter.instruction("ldr x12, [x10, #24]");                                 // recover the outer list representation flag
    emitter.instruction("str x12, [x11]");                                      // restore the outer list representation flag
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_unser_count");
    emitter.instruction("ldr x12, [x10, #32]");                                 // recover the outer logical registry count
    emitter.instruction("str x12, [x11]");                                      // restore the outer logical registry count
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_unser_depth");
    emitter.instruction("ldr x12, [x10, #40]");                                 // recover the suspended outer parser depth
    emitter.instruction("str x12, [x11]");                                      // restore the suspended outer parser depth
    emitter.instruction("ldr x12, [x10, #48]");                                 // load the bounded registry snapshot length
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_unser_values");
    emitter.instruction("mov x13, #0");                                         // start restoring the outer registry prefix
    emitter.label("__rt_unserialize_end_copy");
    emitter.instruction("cmp x13, x12");                                        // restored every snapshotted registry slot?
    emitter.instruction("b.hs __rt_unserialize_end_copy_done");                 // finish when the full used prefix is live again
    emitter.instruction("add x14, x10, #56");                                   // derive the snapshot registry payload base
    emitter.instruction("ldr x15, [x14, x13, lsl #3]");                         // load one saved outer registry entry
    emitter.instruction("str x15, [x11, x13, lsl #3]");                         // restore the outer reference-registry entry
    emitter.instruction("add x13, x13, #1");                                    // advance to the next saved slot
    emitter.instruction("b __rt_unserialize_end_copy");                         // continue restoring the used registry prefix
    emitter.label("__rt_unserialize_end_copy_done");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_active");
    emitter.instruction("ldr x11, [x9]");                                       // load the active nesting count
    emitter.instruction("sub x11, x11, #1");                                    // account for the completed nested parser
    emitter.instruction("str x11, [x9]");                                       // publish the decremented nesting count
    emitter.instruction("ldr x0, [sp, #8]");                                    // pass the consumed snapshot to heap_free
    emitter.instruction("bl __rt_heap_free");                                   // release the temporary reentrancy snapshot
    emitter.instruction("b __rt_unserialize_end_return");                       // preserve the restored outer context

    emitter.label("__rt_unserialize_end_top");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_mode");
    emitter.instruction("str xzr, [x9]");                                       // clear the completed top-level policy mode
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_allowed_list_mixed");
    emitter.instruction("str xzr, [x9]");                                       // clear the completed list representation flag
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_count");
    emitter.instruction("str xzr, [x9]");                                       // retire the completed top-level registry
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_depth");
    emitter.instruction("str xzr, [x9]");                                       // leave no parser depth behind after completion
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_active");
    emitter.instruction("str xzr, [x9]");                                       // mark the unserialize runtime idle
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_context");
    emitter.instruction("str xzr, [x9]");                                       // leave no linked snapshot after top-level completion
    emitter.label("__rt_unserialize_end_return");
    emitter.instruction("ldr x0, [sp]");                                        // restore the parsed Mixed result for the lowering
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore the caller frame and return address
    emitter.instruction("add sp, sp, #48");                                     // release the end helper frame
    emitter.instruction("ret");                                                 // return the unchanged parse result
}

/// Emits the x86_64 begin/end helpers that isolate one active unserialize call.
///
/// This is the SysV counterpart of [`emit_unserialize_context_aarch64`], with the
/// same linked snapshot and allow-list ownership contract.
fn emit_unserialize_context_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unserialize context begin/end ---");
    emitter.label_global("__rt_unserialize_begin");
    emitter.instruction("push rbp");                                            // preserve the caller frame across context allocation
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame for snapshot sizing
    emitter.instruction("sub rsp, 32");                                         // reserve logical-count and copy-count spills
    emitter.instruction("mov r10, QWORD PTR [rip + _unser_active]");            // load the active unserialize nesting count
    emitter.instruction("test r10, r10");                                       // is another parser already active?
    emitter.instruction("jnz __rt_unserialize_begin_nested_x");                 // snapshot the outer call before resetting globals
    emitter.instruction("mov QWORD PTR [rip + _unser_active], 1");              // mark the top-level parser active
    emitter.instruction("jmp __rt_unserialize_begin_reset_x");                  // initialize the fresh per-call state

    emitter.label("__rt_unserialize_begin_nested_x");
    emitter.instruction("cmp r10, 256");                                        // has reentrant parser nesting reached its hard limit?
    emitter.instruction("jae __rt_unser_depth_fatal_x");                        // reject another snapshot before allocating bounded heap state
    emitter.instruction("mov r10, QWORD PTR [rip + _unser_count]");             // load the outer registry's logical value count
    emitter.instruction("mov QWORD PTR [rbp - 8], r10");                        // preserve the logical count across context allocation
    emitter.instruction("mov r11, 65536");                                      // materialize the fixed registry capacity
    emitter.instruction("cmp r10, r11");                                        // does the logical count exceed the physical registry?
    emitter.instruction("cmovb r11, r10");                                      // copy only the populated in-bounds registry prefix
    emitter.instruction("mov QWORD PTR [rbp - 16], r11");                       // preserve the copy count across allocation
    emitter.instruction("mov rax, r11");                                        // start computing the snapshot allocation size
    emitter.instruction("shl rax, 3");                                          // convert copied slots to bytes
    emitter.instruction("add rax, 56");                                         // include the seven-word context header
    emitter.instruction("call __rt_heap_alloc");                                // allocate the linked reentrant context snapshot
    emitter.instruction("mov r10, QWORD PTR [rip + _unser_context]");           // load the previous context link
    emitter.instruction("mov QWORD PTR [rax], r10");                            // context.prev = previous context
    emitter.instruction("mov r10, QWORD PTR [rip + _unser_allowed_mode]");      // load the outer allowed-class mode
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // snapshot the outer allowed-class mode
    emitter.instruction("mov r10, QWORD PTR [rip + _unser_allowed_list]");      // load the outer context-owned allow-list
    emitter.instruction("mov QWORD PTR [rax + 16], r10");                       // move the outer allow-list reference into the snapshot
    emitter.instruction("mov r10, QWORD PTR [rip + _unser_allowed_list_mixed]"); // load the outer list representation flag
    emitter.instruction("mov QWORD PTR [rax + 24], r10");                       // snapshot the outer list representation flag
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // recover the outer logical registry count
    emitter.instruction("mov QWORD PTR [rax + 32], r10");                       // snapshot the logical registry count
    emitter.instruction("mov r10, QWORD PTR [rip + _unser_depth]");             // load the outer recursive parser depth
    emitter.instruction("mov QWORD PTR [rax + 40], r10");                       // snapshot the outer parser depth
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                       // recover the bounded registry copy count
    emitter.instruction("mov QWORD PTR [rax + 48], r11");                       // record how many registry slots follow the header
    emitter.instruction("lea rdx, [rip + _unser_values]");                      // load the outer reference-registry base
    emitter.instruction("xor r10d, r10d");                                      // start copying the populated registry prefix
    emitter.label("__rt_unserialize_begin_copy_x");
    emitter.instruction("cmp r10, r11");                                        // copied every in-bounds outer registry slot?
    emitter.instruction("jae __rt_unserialize_begin_copy_done_x");              // finish once the used prefix is preserved
    emitter.instruction("mov rcx, QWORD PTR [rdx + r10 * 8]");                  // load one outer reference-registry entry
    emitter.instruction("mov QWORD PTR [rax + r10 * 8 + 56], rcx");             // save the outer reference-registry entry
    emitter.instruction("add r10, 1");                                          // advance to the next populated slot
    emitter.instruction("jmp __rt_unserialize_begin_copy_x");                   // continue copying the used registry prefix
    emitter.label("__rt_unserialize_begin_copy_done_x");
    emitter.instruction("mov QWORD PTR [rip + _unser_context], rax");           // publish this snapshot as the current context link
    emitter.instruction("add QWORD PTR [rip + _unser_active], 1");              // account for the nested parser

    emitter.label("__rt_unserialize_begin_reset_x");
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_mode], 0");        // default this call to allow-all until options are installed
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list], 0");        // this call starts without an owned allow-list
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list_mixed], 0");  // default to direct-string list representation
    emitter.instruction("mov QWORD PTR [rip + _unser_count], 0");               // reset this call's reference-registry count
    emitter.instruction("mov QWORD PTR [rip + _unser_depth], 0");               // reset this call's recursive parser depth
    emitter.instruction("leave");                                               // restore the caller frame after begin setup
    emitter.instruction("ret");                                                 // enter the new isolated unserialize call

    emitter.label_global("__rt_unserialize_end");
    emitter.instruction("push rbp");                                            // preserve the caller frame across cleanup calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame for result/context spills
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spills for the result and snapshot pointer
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve the parsed Mixed result across cleanup
    emitter.instruction("mov rax, QWORD PTR [rip + _unser_allowed_list]");      // load this call's owned allow-list reference
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list], 0");        // unpublish the list before releasing its ownership
    emitter.instruction("test rax, rax");                                       // was an allow-list installed for this call?
    emitter.instruction("jz __rt_unserialize_end_list_done_x");                 // skip refcount traffic when no list was installed
    emitter.instruction("call __rt_decref_array");                              // release this call's allow-list ownership
    emitter.label("__rt_unserialize_end_list_done_x");
    emitter.instruction("mov r10, QWORD PTR [rip + _unser_context]");           // load the outer snapshot, if this call was reentrant
    emitter.instruction("test r10, r10");                                       // does an outer parser need restoration?
    emitter.instruction("jz __rt_unserialize_end_top_x");                       // top-level completion has no outer state
    emitter.instruction("mov QWORD PTR [rbp - 16], r10");                       // preserve the snapshot pointer across heap release
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // load the previous linked context
    emitter.instruction("mov QWORD PTR [rip + _unser_context], r11");           // pop the current context snapshot
    emitter.instruction("mov r11, QWORD PTR [r10 + 8]");                        // recover the outer allowed-class mode
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_mode], r11");      // restore the outer allowed-class mode
    emitter.instruction("mov r11, QWORD PTR [r10 + 16]");                       // recover the outer owned allow-list reference
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list], r11");      // republish the outer owned allow-list
    emitter.instruction("mov r11, QWORD PTR [r10 + 24]");                       // recover the outer list representation flag
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list_mixed], r11"); // restore the outer list representation flag
    emitter.instruction("mov r11, QWORD PTR [r10 + 32]");                       // recover the outer logical registry count
    emitter.instruction("mov QWORD PTR [rip + _unser_count], r11");             // restore the outer logical registry count
    emitter.instruction("mov r11, QWORD PTR [r10 + 40]");                       // recover the suspended outer parser depth
    emitter.instruction("mov QWORD PTR [rip + _unser_depth], r11");             // restore the suspended outer parser depth
    emitter.instruction("mov r11, QWORD PTR [r10 + 48]");                       // load the bounded registry snapshot length
    emitter.instruction("lea rdx, [rip + _unser_values]");                      // load the active reference-registry base
    emitter.instruction("xor ecx, ecx");                                        // start restoring the outer registry prefix
    emitter.label("__rt_unserialize_end_copy_x");
    emitter.instruction("cmp rcx, r11");                                        // restored every snapshotted registry slot?
    emitter.instruction("jae __rt_unserialize_end_copy_done_x");                // finish when the full used prefix is live again
    emitter.instruction("mov r8, QWORD PTR [r10 + rcx * 8 + 56]");              // load one saved outer registry entry
    emitter.instruction("mov QWORD PTR [rdx + rcx * 8], r8");                   // restore the outer reference-registry entry
    emitter.instruction("add rcx, 1");                                          // advance to the next saved slot
    emitter.instruction("jmp __rt_unserialize_end_copy_x");                     // continue restoring the used registry prefix
    emitter.label("__rt_unserialize_end_copy_done_x");
    emitter.instruction("sub QWORD PTR [rip + _unser_active], 1");              // account for the completed nested parser
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // pass the consumed snapshot to heap_free
    emitter.instruction("call __rt_heap_free");                                 // release the temporary reentrancy snapshot
    emitter.instruction("jmp __rt_unserialize_end_return_x");                   // preserve the restored outer context

    emitter.label("__rt_unserialize_end_top_x");
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_mode], 0");        // clear the completed top-level policy mode
    emitter.instruction("mov QWORD PTR [rip + _unser_allowed_list_mixed], 0");  // clear the completed list representation flag
    emitter.instruction("mov QWORD PTR [rip + _unser_count], 0");               // retire the completed top-level registry
    emitter.instruction("mov QWORD PTR [rip + _unser_depth], 0");               // leave no parser depth behind after completion
    emitter.instruction("mov QWORD PTR [rip + _unser_active], 0");              // mark the unserialize runtime idle
    emitter.instruction("mov QWORD PTR [rip + _unser_context], 0");             // leave no linked snapshot after top-level completion
    emitter.label("__rt_unserialize_end_return_x");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // restore the parsed Mixed result for the lowering
    emitter.instruction("leave");                                               // restore the caller frame and stack
    emitter.instruction("ret");                                                 // return the unchanged parse result
}

/// Emits the AArch64 allocation-free grammar preflight used before decoding.
///
/// The validator recursively proves every cursor advance and delimiter before the
/// mutating parser can allocate hashes/objects or invoke hydration hooks.
fn emit_unser_validator_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: bounded unserialize grammar preflight ---");

    // uint(base=x0, pos=x1, end=x2, delimiter=w3) -> x0=ok, x1=value, x2=delimiter position
    emitter.label_global("__rt_unser_validate_uint");
    emitter.instruction("add x9, x0, x1");                                      // absolute digit cursor
    emitter.instruction("add x10, x0, x2");                                     // absolute source end
    emitter.instruction("mov x11, #0");                                         // unsigned accumulator
    emitter.instruction("mov x12, #0");                                         // parsed digit count
    emitter.instruction("mov x14, #10");                                        // decimal radix
    emitter.label("__rt_unser_validate_uint_loop");
    emitter.instruction("cmp x9, x10");                                         // is another byte available?
    emitter.instruction("b.hs __rt_unser_validate_uint_fail");                  // truncated digit run has no delimiter
    emitter.instruction("ldrb w13, [x9]");                                      // inspect one bounded byte
    emitter.instruction("cmp w13, #48");                                        // below ASCII zero?
    emitter.instruction("b.lo __rt_unser_validate_uint_done");                  // require the requested delimiter below
    emitter.instruction("cmp w13, #57");                                        // above ASCII nine?
    emitter.instruction("b.hi __rt_unser_validate_uint_done");                  // require the requested delimiter below
    emitter.instruction("sub w13, w13, #48");                                   // convert the byte to a digit
    emitter.instruction("umulh x15, x11, x14");                                 // detect overflow in accumulator * 10
    emitter.instruction("cbnz x15, __rt_unser_validate_uint_fail");             // wrapped lengths/counts are invalid
    emitter.instruction("mul x11, x11, x14");                                   // shift the accumulator by one decimal place
    emitter.instruction("adds x11, x11, x13");                                  // append the current digit and expose carry
    emitter.instruction("b.cs __rt_unser_validate_uint_fail");                  // reject addition overflow
    emitter.instruction("add x12, x12, #1");                                    // record one valid digit
    emitter.instruction("add x9, x9, #1");                                      // advance within the proven source extent
    emitter.instruction("b __rt_unser_validate_uint_loop");                     // scan the remaining digits
    emitter.label("__rt_unser_validate_uint_done");
    emitter.instruction("cbz x12, __rt_unser_validate_uint_fail");              // every numeric field needs at least one digit
    emitter.instruction("cmp w13, w3");                                         // did the run end on its grammar delimiter?
    emitter.instruction("b.ne __rt_unser_validate_uint_fail");                  // arbitrary terminators are rejected
    emitter.instruction("sub x2, x9, x0");                                      // return delimiter position as a source offset
    emitter.instruction("mov x1, x11");                                         // return the parsed unsigned value
    emitter.instruction("mov x0, #1");                                          // report success
    emitter.instruction("ret");                                                 // leaf return
    emitter.label("__rt_unser_validate_uint_fail");
    emitter.instruction("mov x0, #0");                                          // report a bounded numeric failure
    emitter.instruction("ret");                                                 // leaf return

    // key(base=x0, pos=x1, end=x2, depth=x3) -> x0=ok, x1=newpos
    emitter.label_global("__rt_unser_validate_key");
    emitter.instruction("cmp x1, x2");                                          // require the key type byte before loading it
    emitter.instruction("b.hs __rt_unser_validate_key_fail");                   // truncated key
    emitter.instruction("ldrb w9, [x0, x1]");                                   // inspect the bounded key type
    emitter.instruction("cmp w9, #105");                                        // integer key?
    emitter.instruction("b.eq __rt_unser_validate_key_dispatch");               // use an assembler-local conditional target
    emitter.instruction("cmp w9, #115");                                        // string key?
    emitter.instruction("b.ne __rt_unser_validate_key_fail");                   // reject every other key marker
    emitter.label("__rt_unser_validate_key_dispatch");
    emitter.instruction("b __rt_unser_validate_at");                            // main validator owns integer/string grammar
    emitter.label("__rt_unser_validate_key_fail");
    emitter.instruction("mov x0, #0");                                          // only integer and string keys are valid
    emitter.instruction("ret");                                                 // leaf failure return

    // at(base=x0, pos=x1, end=x2, depth=x3) -> x0=ok, x1=newpos
    emitter.label_global("__rt_unser_validate_at");
    emitter.instruction("sub sp, sp, #80");                                     // reserve recursive cursor/count state and an aligned frame
    emitter.instruction("stp x29, x30, [sp, #64]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #64");                                    // establish a stable validator frame
    emitter.instruction("stp x0, x1, [sp]");                                    // save source base and starting position
    emitter.instruction("stp x2, x3, [sp, #16]");                               // save source end and recursion depth
    emitter.instruction("cmp x1, x2");                                          // require a type byte before dispatch
    emitter.instruction("b.hs __rt_unser_validate_at_fail");                    // empty/truncated value
    emitter.instruction("cmp x3, #512");                                        // enforce the same recursion ceiling as the parser
    emitter.instruction("b.hs __rt_unser_validate_at_fail");                    // stop hostile nesting before native-stack exhaustion
    emitter.instruction("ldrb w9, [x0, x1]");                                   // load the bounded type byte
    emitter.instruction("cmp w9, #78");                                         // N
    emitter.instruction("b.eq __rt_unser_validate_null");
    emitter.instruction("cmp w9, #98");                                         // b
    emitter.instruction("b.eq __rt_unser_validate_bool");
    emitter.instruction("cmp w9, #105");                                        // i
    emitter.instruction("b.eq __rt_unser_validate_int");
    emitter.instruction("cmp w9, #100");                                        // d
    emitter.instruction("b.eq __rt_unser_validate_float");
    emitter.instruction("cmp w9, #115");                                        // s
    emitter.instruction("b.eq __rt_unser_validate_string");
    emitter.instruction("cmp w9, #97");                                         // a
    emitter.instruction("b.eq __rt_unser_validate_array");
    emitter.instruction("cmp w9, #79");                                         // O
    emitter.instruction("b.eq __rt_unser_validate_object");
    emitter.instruction("cmp w9, #114");                                        // r
    emitter.instruction("b.eq __rt_unser_validate_ref");
    emitter.instruction("cmp w9, #82");                                         // R
    emitter.instruction("b.eq __rt_unser_validate_ref");
    emitter.instruction("b __rt_unser_validate_at_fail");                       // reject unsupported wire markers

    emitter.label("__rt_unser_validate_null");
    emitter.instruction("sub x9, x2, x1");                                      // bytes remaining from N
    emitter.instruction("cmp x9, #2");                                          // N plus semicolon must fit
    emitter.instruction("b.lo __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x0, x1");                                      // type pointer
    emitter.instruction("ldrb w10, [x9, #1]");                                  // bounded delimiter byte
    emitter.instruction("cmp w10, #59");                                        // semicolon?
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x1, #2");                                      // skip N;
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_bool");
    emitter.instruction("sub x9, x2, x1");                                      // bytes remaining from b
    emitter.instruction("cmp x9, #4");                                          // exact b:<digit>; envelope
    emitter.instruction("b.lo __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x0, x1");                                      // type pointer
    emitter.instruction("ldrb w10, [x9, #1]");
    emitter.instruction("cmp w10, #58");                                        // colon after b
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x9, #2]");
    emitter.instruction("cmp w10, #48");                                        // false digit
    emitter.instruction("b.eq __rt_unser_validate_bool_delim");
    emitter.instruction("cmp w10, #49");                                        // true digit
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.label("__rt_unser_validate_bool_delim");
    emitter.instruction("ldrb w10, [x9, #3]");
    emitter.instruction("cmp w10, #59");                                        // terminating semicolon
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x1, #4");                                      // skip complete boolean
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_int");
    emitter.instruction("add x9, x1, #1");                                      // colon position
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");                                        // require i:
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // first sign/digit position
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #45");                                        // optional minus sign
    emitter.instruction("b.ne __rt_unser_validate_int_digits");
    emitter.instruction("mov x10, #1");                                         // record a negative integer
    emitter.instruction("str x10, [sp, #56]");                                  // preserve sign across the numeric helper
    emitter.instruction("add x9, x9, #1");                                      // skip minus before unsigned digit scan
    emitter.instruction("b __rt_unser_validate_int_scan");
    emitter.label("__rt_unser_validate_int_digits");
    emitter.instruction("str xzr, [sp, #56]");                                  // positive integer
    emitter.label("__rt_unser_validate_int_scan");
    emitter.instruction("ldr x0, [sp]");                                        // base
    emitter.instruction("mov x1, x9");                                          // digit position
    emitter.instruction("ldr x2, [sp, #16]");                                   // end
    emitter.instruction("mov w3, #59");                                         // integer terminator ';'
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    crate::codegen_support::abi::emit_load_int_immediate(emitter, "x9", i64::MAX);
    emitter.instruction("ldr x10, [sp, #56]");                                  // negative-sign flag
    emitter.instruction("cbz x10, __rt_unser_validate_int_positive");
    emitter.instruction("cmp x1, x9");                                          // negative magnitude at most i64::MAX + 1
    emitter.instruction("b.ls __rt_unser_validate_int_range_ok");
    emitter.instruction("sub x10, x1, x9");                                     // only one extra magnitude value is representable
    emitter.instruction("cmp x10, #1");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("b __rt_unser_validate_int_range_ok");
    emitter.label("__rt_unser_validate_int_positive");
    emitter.instruction("cmp x1, x9");                                          // positive magnitude at most i64::MAX
    emitter.instruction("b.hi __rt_unser_validate_at_fail");
    emitter.label("__rt_unser_validate_int_range_ok");
    emitter.instruction("add x1, x2, #1");                                      // skip semicolon
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_float");
    emitter.instruction("add x9, x1, #1");                                      // colon position
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");                                        // require d:
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // first float byte
    emitter.instruction("mov x11, x9");                                         // remember start to reject empty payloads
    emitter.label("__rt_unser_validate_float_loop");
    emitter.instruction("cmp x9, x2");                                          // bound every byte scanned for ';'
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #59");                                        // terminating semicolon?
    emitter.instruction("b.eq __rt_unser_validate_float_done");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("b __rt_unser_validate_float_loop");
    emitter.label("__rt_unser_validate_float_done");
    emitter.instruction("cmp x9, x11");                                         // require at least one float byte
    emitter.instruction("b.eq __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // skip semicolon
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_string");
    emitter.instruction("add x9, x1, #1");                                      // colon after s
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first length digit
    emitter.instruction("mov w3, #58");                                         // length delimiter ':'
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("mov x11, x1");                                         // preserve byte length
    emitter.instruction("add x9, x2, #1");                                      // opening quote position
    emitter.instruction("ldr x10, [sp, #16]");                                  // end
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x12, [sp]");                                       // base
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #34");                                        // opening quote
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // raw payload position
    emitter.instruction("sub x13, x10, x9");                                    // bytes remaining after opening quote
    emitter.instruction("cmp x13, #2");                                         // closing quote plus semicolon
    emitter.instruction("b.lo __rt_unser_validate_at_fail");
    emitter.instruction("sub x13, x13, #2");                                    // maximum safe payload length
    emitter.instruction("cmp x11, x13");                                        // declared bytes fit before delimiters?
    emitter.instruction("b.hi __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, x11");                                     // closing quote position
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #34");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #59");                                        // string terminator
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // position after string
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_array");
    emitter.instruction("add x9, x1, #1");                                      // colon after a
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first count digit
    emitter.instruction("mov w3, #58");                                         // count delimiter ':'
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #32]");                                   // save entry count
    emitter.instruction("add x9, x2, #1");                                      // opening brace position
    emitter.instruction("ldr x10, [sp, #16]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x11, [sp]");
    emitter.instruction("ldrb w12, [x11, x9]");
    emitter.instruction("cmp w12, #123");                                       // opening brace
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // first key position
    emitter.instruction("str x9, [sp, #40]");                                   // current body position
    emitter.instruction("str xzr, [sp, #48]");                                  // entry index
    emitter.label("__rt_unser_validate_array_loop");
    emitter.instruction("ldr x9, [sp, #48]");
    emitter.instruction("ldr x10, [sp, #32]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_container_close");
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x1, [sp, #40]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("ldr x3, [sp, #24]");
    emitter.instruction("add x3, x3, #1");                                      // nested key depth
    emitter.instruction("bl __rt_unser_validate_key");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #40]");                                   // position after key
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("ldr x3, [sp, #24]");
    emitter.instruction("add x3, x3, #1");                                      // nested value depth
    emitter.instruction("bl __rt_unser_validate_at");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #40]");                                   // position after value
    emitter.instruction("ldr x9, [sp, #48]");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("str x9, [sp, #48]");
    emitter.instruction("b __rt_unser_validate_array_loop");

    emitter.label("__rt_unser_validate_object");
    emitter.instruction("add x9, x1, #1");                                      // colon after O
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first class-name length digit
    emitter.instruction("mov w3, #58");
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("mov x11, x1");                                         // class-name byte length
    emitter.instruction("add x9, x2, #1");                                      // opening quote position
    emitter.instruction("ldr x10, [sp, #16]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x12, [sp]");
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #34");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // class-name bytes
    emitter.instruction("sub x13, x10, x9");
    emitter.instruction("cmp x13, #2");                                         // closing quote and colon
    emitter.instruction("b.lo __rt_unser_validate_at_fail");
    emitter.instruction("sub x13, x13, #2");
    emitter.instruction("cmp x11, x13");
    emitter.instruction("b.hi __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, x11");                                     // closing quote
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #34");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // colon before property count
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first property-count digit
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("mov w3, #58");
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #32]");                                   // save property count
    emitter.instruction("add x9, x2, #1");                                      // opening brace position
    emitter.instruction("ldr x10, [sp, #16]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x11, [sp]");
    emitter.instruction("ldrb w12, [x11, x9]");
    emitter.instruction("cmp w12, #123");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("str x9, [sp, #40]");                                   // first property key
    emitter.instruction("str xzr, [sp, #48]");                                  // property index
    emitter.label("__rt_unser_validate_object_loop");
    emitter.instruction("ldr x9, [sp, #48]");
    emitter.instruction("ldr x10, [sp, #32]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_container_close");
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x1, [sp, #40]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("ldr x3, [sp, #24]");
    emitter.instruction("add x3, x3, #1");
    emitter.instruction("bl __rt_unser_validate_key");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #40]");
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("ldr x3, [sp, #24]");
    emitter.instruction("add x3, x3, #1");
    emitter.instruction("bl __rt_unser_validate_at");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #40]");
    emitter.instruction("ldr x9, [sp, #48]");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("str x9, [sp, #48]");
    emitter.instruction("b __rt_unser_validate_object_loop");

    emitter.label("__rt_unser_validate_container_close");
    emitter.instruction("ldr x1, [sp, #40]");                                   // closing-brace position
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("cmp x1, x2");                                          // require the closing brace byte
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x9, [sp]");
    emitter.instruction("ldrb w10, [x9, x1]");
    emitter.instruction("cmp w10, #125");                                       // exact closing brace
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x1, #1");                                      // return after the complete container
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_ref");
    emitter.instruction("add x9, x1, #1");                                      // colon after r/R
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first reference-index digit
    emitter.instruction("mov w3, #59");
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("cbz x1, __rt_unser_validate_at_fail");                 // reference indices are one-based
    emitter.instruction("add x1, x2, #1");                                      // skip semicolon

    emitter.label("__rt_unser_validate_at_ok");
    emitter.instruction("mov x0, #1");                                          // report a fully bounded value
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore recursive validator frame
    emitter.instruction("add sp, sp, #80");                                     // release local validation state
    emitter.instruction("ret");
    emitter.label("__rt_unser_validate_at_fail");
    emitter.instruction("mov x0, #0");                                          // report malformed/truncated wire data
    emitter.instruction("ldr x1, [sp, #8]");                                    // preserve the original position on failure
    emitter.instruction("ldp x29, x30, [sp, #64]");
    emitter.instruction("add sp, sp, #80");
    emitter.instruction("ret");
}

/// Emits the AArch64 unserialize entry, recursive parser, and hydration helpers.
fn emit_unserialize_aarch64(emitter: &mut Emitter) {
    let boundary_bytes = TRY_HANDLER_SLOT_SIZE + 32;
    let frame_link_offset = boundary_bytes - 16;
    let source_offset = TRY_HANDLER_SLOT_SIZE;
    let source_len_offset = source_offset + 8;

    // -- entry wrapper: protect begin/end cleanup across hydration-hook throws --
    emitter.blank();
    emitter.comment("--- runtime: unserialize_mixed (serialize() wire -> boxed Mixed) ---");
    emitter.label_global("__rt_unserialize_mixed");
    emitter.instruction(&format!("sub sp, sp, #{}", boundary_bytes));           // reserve a complete handler record plus input/result spills
    emitter.instruction(&format!("stp x29, x30, [sp, #{}]", frame_link_offset)); // preserve the caller frame and return address across setjmp
    emitter.instruction(&format!("add x29, sp, #{}", frame_link_offset));       // establish the protected unserialize wrapper frame
    emitter.instruction(&format!("str x1, [sp, #{}]", source_offset));          // preserve source pointer across setjmp
    emitter.instruction(&format!("str x2, [sp, #{}]", source_len_offset));      // preserve source length across setjmp
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_exc_handler_top", 0);
    emitter.instruction("str x10, [sp]");                                       // handler.next = previous exception-handler head
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_exc_call_frame_top", 0);
    emitter.instruction("str x10, [sp, #8]");                                   // preserve the activation frame that survives this boundary
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_rt_diag_suppression", 0);
    emitter.instruction(&format!("str x10, [sp, #{}]", TRY_HANDLER_DIAG_DEPTH_OFFSET)); // snapshot diagnostic suppression across longjmp
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "x10", "_runtime_recursion_stack_bytes", 0);
    emitter.instruction(&format!("str x10, [sp, #{}]", TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET)); // snapshot the user-stack budget across longjmp
    emitter.instruction("mov x10, sp");                                         // compute this wrapper's exception-handler record address
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_exc_handler_top", 0);
    emitter.instruction(&format!("add x0, sp, #{}", TRY_HANDLER_JMP_BUF_OFFSET)); // pass this boundary's opaque jmp_buf to setjmp
    emitter.bl_c("setjmp"); // catch Throwable control flow escaping hydration hooks
    emitter.instruction("cbnz x0, __rt_unserialize_mixed_throw");               // longjmp resumes here so runtime state can be cleaned first
    emitter.instruction(&format!("ldr x0, [sp, #{}]", source_offset));          // base = preserved source string pointer
    emitter.instruction("mov x1, #0");                                          // start parsing at position 0
    emitter.instruction(&format!("ldr x2, [sp, #{}]", source_len_offset));      // end = preserved source string length
    emitter.instruction("mov x3, #0");                                          // preflight starts at recursive depth zero
    emitter.instruction("bl __rt_unser_validate_at");                           // reject truncated/overflowing grammar before allocating or running hooks
    emitter.instruction("cbz x0, __rt_unserialize_mixed_invalid");              // malformed input returns PHP false through the normal end path
    emitter.instruction(&format!("ldr x0, [sp, #{}]", source_offset));          // reload base after the validator's caller-clobbered registers
    emitter.instruction("mov x1, #0");                                          // parse the already validated value from the beginning
    emitter.instruction(&format!("ldr x2, [sp, #{}]", source_len_offset));      // restore the validated source extent
    emitter.instruction("bl __rt_unser_at");                                    // parse while the cleanup boundary is active
    emitter.instruction("b __rt_unserialize_mixed_parsed");                     // share exception-boundary teardown with validation failures
    emitter.label("__rt_unserialize_mixed_invalid");
    emitter.instruction("mov x0, #0");                                          // null result signals a bounded parse failure
    emitter.label("__rt_unserialize_mixed_parsed");
    emitter.instruction(&format!("str x0, [sp, #{}]", source_offset));          // preserve the parsed box while popping the boundary
    emitter.instruction("ldr x10, [sp]");                                       // reload the previous exception-handler head
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_exc_handler_top", 0);
    emitter.instruction(&format!("ldr x10, [sp, #{}]", TRY_HANDLER_DIAG_DEPTH_OFFSET)); // reload diagnostic suppression after the protected parse
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_rt_diag_suppression", 0);
    emitter.instruction(&format!("ldr x0, [sp, #{}]", source_offset));          // recover the parsed Mixed result
    emitter.instruction(&format!("ldp x29, x30, [sp, #{}]", frame_link_offset)); // restore the caller frame and return address
    emitter.instruction(&format!("add sp, sp, #{}", boundary_bytes));           // release the exception boundary frame
    emitter.instruction("ret");                                                 // return the parsed box to the lowering's normal end path
    emitter.label("__rt_unserialize_mixed_throw");
    emitter.instruction("ldr x10, [sp]");                                       // reload the handler preceding this internal boundary
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_exc_handler_top", 0);
    emitter.instruction(&format!("ldr x10, [sp, #{}]", TRY_HANDLER_DIAG_DEPTH_OFFSET)); // restore diagnostic suppression skipped by longjmp
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_rt_diag_suppression", 0);
    emitter.instruction(&format!("ldr x10, [sp, #{}]", TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET)); // restore the user-stack budget skipped by longjmp
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "x10", "_runtime_recursion_stack_bytes", 0);
    emitter.instruction("mov x0, #0");                                          // end cleanup ignores the placeholder parse result on throw
    emitter.instruction("bl __rt_unserialize_end");                             // release policy/context state before propagating the Throwable
    emitter.instruction(&format!("ldp x29, x30, [sp, #{}]", frame_link_offset)); // restore the caller frame before rethrowing
    emitter.instruction(&format!("add sp, sp, #{}", boundary_bytes));           // discard the protected parser stack through its boundary
    emitter.instruction("b __rt_throw_current");                                // resume propagation at the caller's exception handler

    emit_unser_validator_aarch64(emitter);

    // -- __rt_unser_at(base=x0, pos=x1, end=x2) -> x0=boxed Mixed (0 on fail), x1=newpos --
    emitter.blank();
    emitter.comment("--- runtime: unser_at (recursive serialize() value parser) ---");
    emitter.label_global("__rt_unser_at");
    // [sp+0]=base [8]=pos [16]=end [24]=hash [32]=count [40]=index [48]=key_lo [56]=key_hi [64]=scratch
    emitter.instruction("sub sp, sp, #112");                                    // recursive parser frame
    emitter.instruction("stp x29, x30, [sp, #96]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #96");                                    // establish the new frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the base pointer
    emitter.instruction("str x1, [sp, #8]");                                    // save the current position
    emitter.instruction("str x2, [sp, #16]");                                   // save the end position
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_depth");
    emitter.instruction("ldr x10, [x9]");                                       // load current recursive unserialize depth
    emitter.instruction("add x10, x10, #1");                                    // account for this parser frame
    emitter.instruction("str x10, [x9]");                                       // publish parser depth before consuming wire bytes
    emitter.instruction("cmp x10, #512");                                       // bound recursive frames before native-stack exhaustion
    emitter.instruction("b.le __rt_unser_at_depth_in_budget");                  // keep the conditional branch inside the recursive parser atom
    emitter.instruction("b __rt_unser_depth_fatal");                            // terminate hostile nesting through the shared fatal path
    emitter.label("__rt_unser_at_depth_in_budget");
    emitter.instruction("cmp x1, x2");                                          // is the cursor already at/past the end?
    emitter.instruction("b.ge __rt_unser_at_fail");                             // nothing left to parse
    emitter.instruction("ldrb w9, [x0, x1]");                                   // load the leading type byte
    // -- back-reference? r:N; (object identity) or R:N; (PHP reference) resolves
    //    to a previously parsed value and consumes no new index --
    emitter.instruction("cmp w9, #114");                                        // ASCII 'r'?
    emitter.instruction("b.eq __rt_unser_at_ref");                              // resolve an object back-reference
    emitter.instruction("cmp w9, #82");                                         // ASCII 'R'?
    emitter.instruction("b.eq __rt_unser_at_ref");                              // resolve a PHP reference
    // -- every other value consumes the next pre-order index, mirroring the
    //    counter serialize() used, so r:/R: targets line up by index --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_unser_count");
    emitter.instruction("ldr x11, [x10]");                                      // current value index
    emitter.instruction("str x11, [sp, #88]");                                  // reserve this value's index
    emitter.instruction("add x11, x11, #1");                                    // advance the registry counter
    emitter.instruction("str x11, [x10]");                                      // publish the advanced counter
    emitter.instruction("sub x12, x11, #1");                                    // recover the reserved zero-based registry slot
    emitter.instruction("mov x13, #65536");                                     // materialize the physical reference-registry capacity
    emitter.instruction("cmp x12, x13");                                        // is the reserved slot inside the fixed registry?
    emitter.instruction("b.hs __rt_unser_at_registry_slot_ready");              // out-of-capacity values remain deliberately unregistered
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_unser_values");
    emitter.instruction("str xzr, [x10, x12, lsl #3]");                         // erase any stale object pointer before parsing this value
    emitter.label("__rt_unser_at_registry_slot_ready");
    emitter.instruction("cmp w9, #78");                                         // ASCII 'N' (null)?
    emitter.instruction("b.eq __rt_unser_at_null");                             // parse null
    emitter.instruction("cmp w9, #98");                                         // ASCII 'b' (bool)?
    emitter.instruction("b.eq __rt_unser_at_bool");                             // parse bool
    emitter.instruction("cmp w9, #105");                                        // ASCII 'i' (int)?
    emitter.instruction("b.eq __rt_unser_at_int");                              // parse int
    emitter.instruction("cmp w9, #100");                                        // ASCII 'd' (float)?
    emitter.instruction("b.eq __rt_unser_at_float");                            // parse float
    emitter.instruction("cmp w9, #115");                                        // ASCII 's' (string)?
    emitter.instruction("b.eq __rt_unser_at_str");                              // parse string
    emitter.instruction("cmp w9, #97");                                         // ASCII 'a' (array)?
    emitter.instruction("b.eq __rt_unser_at_array");                            // parse array
    emitter.instruction("cmp w9, #79");                                         // ASCII 'O' (object)?
    emitter.instruction("b.eq __rt_unser_at_object");                           // parse object
    emitter.instruction("b __rt_unser_at_fail");                                // unsupported wire form

    // -- null: "N;" --
    emitter.label("__rt_unser_at_null");
    emitter.instruction("mov x0, #8");                                          // value tag = null
    emitter.instruction("mov x1, #0");                                          // null payload low word
    emitter.instruction("mov x2, #0");                                          // null payload high word
    emitter.instruction("bl __rt_mixed_from_value");                            // box the null value
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload position
    emitter.instruction("add x1, x1, #2");                                      // newpos skips "N;"
    emitter.instruction("b __rt_unser_at_ret");                                 // return the box and new position

    // -- bool: "b:0;" / "b:1;" --
    emitter.label("__rt_unser_at_bool");
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload base
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload position
    emitter.instruction("add x12, x10, x11");                                   // pointer to the type byte
    emitter.instruction("ldrb w9, [x12, #2]");                                  // load the bool digit at offset 2
    emitter.instruction("sub w9, w9, #48");                                     // ASCII '0'/'1' -> 0/1
    emitter.instruction("and x1, x9, #1");                                      // clamp to a single bool bit
    emitter.instruction("mov x0, #3");                                          // value tag = bool
    emitter.instruction("mov x2, #0");                                          // bool high payload unused
    emitter.instruction("bl __rt_mixed_from_value");                            // box the bool value
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload position
    emitter.instruction("add x1, x1, #4");                                      // newpos skips "b:X;"
    emitter.instruction("b __rt_unser_at_ret");                                 // return the box and new position

    // -- int: "i:" + optional '-' + digits + ";" --
    emitter.label("__rt_unser_at_int");
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload base
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload position
    emitter.instruction("add x10, x10, x11");                                   // pointer to the type byte
    emitter.instruction("add x10, x10, #2");                                    // skip "i:" to the first digit
    emitter.instruction("mov x11, #0");                                         // digit accumulator
    emitter.instruction("mov x12, #0");                                         // negative-sign flag
    emitter.instruction("ldrb w9, [x10]");                                      // first numeric byte
    emitter.instruction("cmp w9, #45");                                         // leading '-'?
    emitter.instruction("b.ne __rt_unser_at_int_loop");                         // no sign
    emitter.instruction("mov x12, #1");                                         // record negative sign
    emitter.instruction("add x10, x10, #1");                                    // skip '-'
    emitter.label("__rt_unser_at_int_loop");
    emitter.instruction("ldrb w9, [x10]");                                      // next numeric byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_at_int_done");                         // terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_at_int_done");                         // terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x13, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x13");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x10, x10, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_at_int_loop");                            // continue
    emitter.label("__rt_unser_at_int_done");
    emitter.instruction("cbz x12, __rt_unser_at_int_box");                      // not signed
    emitter.instruction("neg x11, x11");                                        // apply sign
    emitter.label("__rt_unser_at_int_box");
    emitter.instruction("str x10, [sp, #64]");                                  // save the cursor (at ';') across the box call
    emitter.instruction("mov x1, x11");                                         // value payload = parsed int
    emitter.instruction("mov x0, #0");                                          // value tag = int
    emitter.instruction("mov x2, #0");                                          // int high payload unused
    emitter.instruction("bl __rt_mixed_from_value");                            // box the int value
    emitter.instruction("ldr x10, [sp, #64]");                                  // reload the cursor
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload base
    emitter.instruction("sub x1, x10, x9");                                     // newpos = cursor - base
    emitter.instruction("add x1, x1, #1");                                      // skip the ';'
    emitter.instruction("b __rt_unser_at_ret");                                 // return the box and new position

    // -- float: "d:" + (INF/-INF/NAN | digits) + ";" --
    emitter.label("__rt_unser_at_float");
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload base
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload position
    emitter.instruction("add x0, x10, x11");                                    // pointer to the type byte
    emitter.instruction("add x0, x0, #2");                                      // strtod source = first byte after "d:"
    emitter.instruction("add x1, sp, #64");                                     // strtod endptr = &scratch
    emitter.bl_c("strtod"); // parse the float (stops at ';') -> d0, scratch=endptr
    emitter.instruction("ldr x10, [sp, #64]");                                  // bounded conversion end pointer
    emitter.instruction("ldr x11, [sp, #0]");                                   // source base
    emitter.instruction("ldr x12, [sp, #8]");                                   // original value position
    emitter.instruction("add x11, x11, x12");                                   // pointer to the type byte
    emitter.instruction("add x11, x11, #2");                                    // first float payload byte
    emitter.instruction("cmp x10, x11");                                        // did strtod consume at least one byte?
    emitter.instruction("b.eq __rt_unser_at_fail");                             // invalid numeric payload
    emitter.instruction("ldr x12, [sp, #0]");                                   // source base
    emitter.instruction("ldr x13, [sp, #16]");                                  // source end offset
    emitter.instruction("add x12, x12, x13");                                   // absolute source end
    emitter.instruction("cmp x10, x12");                                        // end pointer must still address a delimiter
    emitter.instruction("b.hs __rt_unser_at_fail");                             // reject a conversion escaping the source extent
    emitter.instruction("ldrb w12, [x10]");                                     // conversion terminator byte
    emitter.instruction("cmp w12, #59");                                        // exact semicolon delimiter
    emitter.instruction("b.ne __rt_unser_at_fail");                             // reject partial conversions such as `1x;`
    emitter.instruction("fmov x9, d0");                                         // move the parsed double into a GPR
    emitter.instruction("mov x1, x9");                                          // value payload = float bits
    emitter.instruction("mov x0, #2");                                          // value tag = float
    emitter.instruction("mov x2, #0");                                          // float high payload unused
    emitter.instruction("bl __rt_mixed_from_value");                            // box the float value
    emitter.instruction("ldr x10, [sp, #64]");                                  // reload the strtod endptr
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload base
    emitter.instruction("sub x1, x10, x9");                                     // newpos = endptr - base
    emitter.instruction("add x1, x1, #1");                                      // skip the ';'
    emitter.instruction("b __rt_unser_at_ret");                                 // return the box and new position

    // -- string: "s:" + bytelen + ":\"" + raw + "\";" --
    emitter.label("__rt_unser_at_str");
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload base
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload position
    emitter.instruction("add x10, x10, x11");                                   // pointer to the type byte
    emitter.instruction("add x10, x10, #2");                                    // skip "s:" to the length digits
    emitter.instruction("mov x11, #0");                                         // length accumulator
    emitter.label("__rt_unser_at_strlen");
    emitter.instruction("ldrb w9, [x10]");                                      // next length byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_at_strlen_done");                      // ':' terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_at_strlen_done");                      // ':' terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x13, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x13");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x10, x10, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_at_strlen");                              // continue
    emitter.label("__rt_unser_at_strlen_done");
    emitter.instruction("add x10, x10, #2");                                    // skip ':' and opening '\"' to the raw bytes
    emitter.instruction("add x9, x10, x11");                                    // raw end = raw + len
    emitter.instruction("str x9, [sp, #64]");                                   // save raw end across the box call
    emitter.instruction("mov x1, x10");                                         // string payload pointer = raw bytes
    emitter.instruction("mov x2, x11");                                         // string payload length
    emitter.instruction("mov x0, #1");                                          // value tag = string (mixed_from_value persists it)
    emitter.instruction("bl __rt_mixed_from_value");                            // box an owned copy of the string
    emitter.instruction("ldr x10, [sp, #64]");                                  // reload raw end
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload base
    emitter.instruction("sub x1, x10, x9");                                     // newpos = raw end - base
    emitter.instruction("add x1, x1, #2");                                      // skip closing '\"' and ';'
    emitter.instruction("b __rt_unser_at_ret");                                 // return the box and new position

    // -- array: "a:" + count + ":{" + count*(key value) + "}" --
    emitter.label("__rt_unser_at_array");
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload base
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload position
    emitter.instruction("add x10, x10, x11");                                   // pointer to the type byte
    emitter.instruction("add x10, x10, #2");                                    // skip "a:" to the count digits
    emitter.instruction("mov x11, #0");                                         // count accumulator
    emitter.label("__rt_unser_at_count");
    emitter.instruction("ldrb w9, [x10]");                                      // next count byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_at_count_done");                       // ':' terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_at_count_done");                       // ':' terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x13, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x13");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x10, x10, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_at_count");                               // continue
    emitter.label("__rt_unser_at_count_done");
    emitter.instruction("str x11, [sp, #32]");                                  // save the entry count
    emitter.instruction("add x10, x10, #2");                                    // skip ':' and '{' to the body
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload base
    emitter.instruction("sub x12, x10, x9");                                    // body position offset
    emitter.instruction("str x12, [sp, #8]");                                   // advance the cursor to the body
    emitter.instruction("mov x0, x11");                                         // hash capacity = entry count
    emitter.instruction("mov x1, #7");                                          // hash value_type = boxed Mixed
    emitter.instruction("bl __rt_hash_new");                                    // allocate the destination hash
    emitter.instruction("str x0, [sp, #24]");                                   // save the hash pointer
    emitter.instruction("str xzr, [sp, #40]");                                  // initialize the entry index
    emitter.label("__rt_unser_at_array_loop");
    emitter.instruction("ldr x4, [sp, #40]");                                   // reload the entry index
    emitter.instruction("ldr x3, [sp, #32]");                                   // reload the entry count
    emitter.instruction("cmp x4, x3");                                          // all entries parsed?
    emitter.instruction("b.ge __rt_unser_at_array_close");                      // box the hash when done
    emitter.instruction("ldr x0, [sp, #0]");                                    // base
    emitter.instruction("ldr x1, [sp, #8]");                                    // current position
    emitter.instruction("ldr x2, [sp, #16]");                                   // end
    emitter.instruction("bl __rt_unser_key");                                   // parse the key -> x0=key_lo, x1=key_hi, x2=newpos
    emitter.instruction("ldr x9, [sp, #16]");                                   // source end for key-result validation
    emitter.instruction("cmp x2, x9");                                          // key parser must not escape the validated source
    emitter.instruction("b.hi __rt_unser_at_array_fail");                       // release the partially built hash on failure
    emitter.instruction("str x0, [sp, #48]");                                   // save key_lo
    emitter.instruction("str x1, [sp, #56]");                                   // save key_hi
    emitter.instruction("str x2, [sp, #8]");                                    // advance past the key
    emitter.instruction("ldr x0, [sp, #0]");                                    // base
    emitter.instruction("ldr x1, [sp, #8]");                                    // position after the key
    emitter.instruction("ldr x2, [sp, #16]");                                   // end
    emitter.instruction("bl __rt_unser_at");                                    // recursively parse the value -> x0=box, x1=newpos
    emitter.instruction("cbz x0, __rt_unser_at_array_fail");                    // child failure invalidates the whole array
    emitter.instruction("str x1, [sp, #8]");                                    // advance past the value
    emitter.instruction("mov x3, x0");                                          // value_lo = parsed value box
    emitter.instruction("ldr x0, [sp, #24]");                                   // hash pointer
    emitter.instruction("ldr x1, [sp, #48]");                                   // key_lo
    emitter.instruction("ldr x2, [sp, #56]");                                   // key_hi (-1 for int keys)
    emitter.instruction("mov x4, #0");                                          // value_hi unused
    emitter.instruction("mov x5, #7");                                          // value tag = boxed Mixed (transfer the box)
    emitter.instruction("bl __rt_hash_set");                                    // insert the entry -> x0 = (possibly new) hash
    emitter.instruction("str x0, [sp, #24]");                                   // save the updated hash pointer
    emitter.instruction("ldr x4, [sp, #40]");                                   // reload the entry index
    emitter.instruction("add x4, x4, #1");                                      // advance the entry index
    emitter.instruction("str x4, [sp, #40]");                                   // persist the entry index
    emitter.instruction("b __rt_unser_at_array_loop");                          // continue with the next entry
    emitter.label("__rt_unser_at_array_close");
    emitter.instruction("ldr x9, [sp, #8]");                                    // closing-brace position
    emitter.instruction("ldr x10, [sp, #16]");                                  // source end
    emitter.instruction("cmp x9, x10");                                         // require the closing delimiter byte
    emitter.instruction("b.hs __rt_unser_at_array_fail");
    emitter.instruction("ldr x10, [sp, #0]");                                   // source base
    emitter.instruction("ldrb w11, [x10, x9]");                                 // bounded closing delimiter
    emitter.instruction("cmp w11, #125");                                       // exact `}`
    emitter.instruction("b.ne __rt_unser_at_array_fail");
    emitter.instruction("mov x0, #24");                                         // box the hash: Mixed cell = tag + two payload words
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the boxed Mixed cell
    emitter.instruction("mov x9, #5");                                          // heap kind 5 = boxed Mixed cell
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the heap header
    emitter.instruction("mov x9, #5");                                          // value tag 5 = associative array (hash)
    emitter.instruction("str x9, [x0]");                                        // store the value tag
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload the hash pointer
    emitter.instruction("str x9, [x0, #8]");                                    // store the hash pointer (ownership transferred, no incref)
    emitter.instruction("str xzr, [x0, #16]");                                  // clear the high payload word
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload position (at the closing '}')
    emitter.instruction("add x1, x1, #1");                                      // newpos skips the '}'
    emitter.instruction("b __rt_unser_at_ret");                                 // return the box and new position
    emitter.label("__rt_unser_at_array_fail");
    emitter.instruction("ldr x0, [sp, #24]");                                   // partially built hash pointer
    emitter.instruction("bl __rt_hash_free_deep");                              // release keys and transferred boxed values locally
    emitter.instruction("mov x0, #0");                                          // report parse failure
    emitter.instruction("ldr x1, [sp, #8]");                                    // preserve current cursor for the caller
    emitter.instruction("b __rt_unser_at_ret");                                 // only the shared return decrements parser depth

    // -- object: "O:" + namelen + ":\"" + class + "\":" + count + ":{" + count*(key value) + "}" --
    emitter.label("__rt_unser_at_object");
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload base
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload position
    emitter.instruction("add x10, x10, x11");                                   // pointer to the type byte
    emitter.instruction("add x10, x10, #2");                                    // skip "O:" to the class-name length digits
    emitter.instruction("mov x11, #0");                                         // class-name length accumulator
    emitter.label("__rt_unser_at_obj_namelen");
    emitter.instruction("ldrb w9, [x10]");                                      // next length byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_at_obj_namelen_done");                 // ':' terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_at_obj_namelen_done");                 // ':' terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x13, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x13");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x10, x10, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_at_obj_namelen");                         // continue
    emitter.label("__rt_unser_at_obj_namelen_done");
    emitter.instruction("add x10, x10, #2");                                    // skip ':' and opening '\"' to the class name bytes
    emitter.instruction("add x12, x10, x11");                                   // class-name end = name + len
    emitter.instruction("str x12, [sp, #64]");                                  // save the class-name end across the call
    emitter.instruction("str x10, [sp, #40]");                                  // save the class-name start across the policy helper
    emitter.instruction("str x11, [sp, #72]");                                  // save the class-name length across the policy helper
    emitter.instruction("mov x0, x10");                                         // class name pointer for allowed_classes policy
    emitter.instruction("mov x1, x11");                                         // class name length for allowed_classes policy
    emitter.instruction("bl __rt_unserialize_class_allowed");                   // decide whether hydration is permitted
    emitter.instruction("str x0, [sp, #80]");                                   // retain policy result until hook/property dispatch
    emitter.instruction("cbz x0, __rt_unser_obj_incomplete");                   // blocked classes become incomplete objects
    emitter.instruction("ldr x10, [sp, #40]");                                  // reload class-name start after helper call
    emitter.instruction("ldr x11, [sp, #72]");                                  // reload class-name length after helper call
    emitter.instruction("mov x1, x10");                                         // class-name pointer (new_by_name arg)
    emitter.instruction("mov x2, x11");                                         // class-name length (new_by_name arg)
    emitter.instruction("bl __rt_new_by_name");                                 // instantiate the class by name (0 on unknown class)
    emitter.instruction("cbz x0, __rt_unser_at_fail");                          // unknown class fails the parse
    emitter.instruction("b __rt_unser_obj_allocated");                          // skip incomplete-object allocation
    emitter.label("__rt_unser_obj_incomplete");
    emitter.instruction("mov x0, #32");                                         // class id, original class name, and opaque property hash
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the incomplete-object payload
    emitter.instruction("mov x9, #4");                                          // heap kind 4 = object instance
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the object heap header
    emitter.instruction("bl __rt_object_handle_acquire");                       // give the incomplete object a normal PHP handle
    emitter.instruction("mov x9, #-2");                                         // reserved class id for __PHP_Incomplete_Class
    emitter.instruction("str x9, [x0]");                                        // publish synthetic class id
    emitter.instruction("str x0, [sp, #24]");                                   // preserve incomplete object across string persistence
    emitter.instruction("ldr x1, [sp, #40]");                                   // original serialized class-name bytes
    emitter.instruction("ldr x2, [sp, #72]");                                   // original serialized class-name length
    emitter.instruction("bl __rt_str_persist");                                 // own the class name independently of the source wire
    emitter.instruction("ldr x0, [sp, #24]");                                   // reload incomplete object payload
    emitter.instruction("str x1, [x0, #8]");                                    // persisted original class-name pointer
    emitter.instruction("str x2, [x0, #16]");                                   // persisted original class-name length
    emitter.instruction("str xzr, [x0, #24]");                                  // property hash is created after its count is parsed
    emitter.label("__rt_unser_obj_allocated");
    emitter.instruction("str x0, [sp, #24]");                                   // save the new object pointer
    emitter.instruction("ldr x12, [sp, #64]");                                  // reload the class-name end
    emitter.instruction("add x12, x12, #2");                                    // skip closing '\"' and ':' to the property count
    emitter.instruction("mov x11, #0");                                         // property-count accumulator
    emitter.label("__rt_unser_at_obj_count");
    emitter.instruction("ldrb w9, [x12]");                                      // next count byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_at_obj_count_done");                   // ':' terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_at_obj_count_done");                   // ':' terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x13, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x13");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x12, x12, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_at_obj_count");                           // continue
    emitter.label("__rt_unser_at_obj_count_done");
    emitter.instruction("str x11, [sp, #32]");                                  // save the property count
    emitter.instruction("add x12, x12, #2");                                    // skip ':' and '{' to the body
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload base
    emitter.instruction("sub x12, x12, x9");                                    // body position offset
    emitter.instruction("str x12, [sp, #8]");                                   // advance the cursor to the body
    emitter.instruction("ldr x9, [sp, #80]");                                   // reload allowed_classes decision
    emitter.instruction("cbz x9, __rt_unser_obj_default");                      // blocked objects must never inspect class hook tables
    // -- __unserialize magic: parse the body into an assoc array, then call
    //    __unserialize($this, $data) instead of injecting properties by name --
    emitter.instruction("ldr x9, [sp, #24]");                                   // object pointer
    emitter.instruction("ldr x9, [x9]");                                        // class id from the object header
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_class_unserialize_ptrs");
    emitter.instruction("ldr x10, [x10, x9, lsl #3]");                          // __unserialize method symbol (0 if none)
    emitter.instruction("cbz x10, __rt_unser_obj_default");                     // no __unserialize → inject properties by name
    emitter.instruction("str x10, [sp, #72]");                                  // park the __unserialize target across the body parse
    emitter.instruction("ldr x0, [sp, #32]");                                   // entry count = hash capacity hint
    emitter.instruction("mov x1, #7");                                          // hash value_type = boxed Mixed
    emitter.instruction("bl __rt_hash_new");                                    // allocate the $data hash
    emitter.instruction("str x0, [sp, #80]");                                   // save the $data hash pointer
    emitter.instruction("str xzr, [sp, #40]");                                  // entry index = 0
    emitter.label("__rt_unser_obj_data_loop");
    emitter.instruction("ldr x4, [sp, #40]");                                   // reload the entry index
    emitter.instruction("ldr x3, [sp, #32]");                                   // reload the entry count
    emitter.instruction("cmp x4, x3");                                          // all entries parsed?
    emitter.instruction("b.ge __rt_unser_obj_data_done");                       // call __unserialize when done
    emitter.instruction("ldr x0, [sp, #0]");                                    // base
    emitter.instruction("ldr x1, [sp, #8]");                                    // current position
    emitter.instruction("ldr x2, [sp, #16]");                                   // end
    emitter.instruction("bl __rt_unser_key");                                   // parse the key -> x0=key_lo, x1=key_hi, x2=newpos
    emitter.instruction("str x0, [sp, #48]");                                   // save key_lo
    emitter.instruction("str x1, [sp, #56]");                                   // save key_hi
    emitter.instruction("str x2, [sp, #8]");                                    // advance past the key
    emitter.instruction("ldr x0, [sp, #0]");                                    // base
    emitter.instruction("ldr x1, [sp, #8]");                                    // position after the key
    emitter.instruction("ldr x2, [sp, #16]");                                   // end
    emitter.instruction("bl __rt_unser_at");                                    // recursively parse the value -> x0=box, x1=newpos
    emitter.instruction("str x1, [sp, #8]");                                    // advance past the value
    emitter.instruction("mov x3, x0");                                          // value_lo = parsed value box
    emitter.instruction("ldr x0, [sp, #80]");                                   // $data hash pointer
    emitter.instruction("ldr x1, [sp, #48]");                                   // key_lo
    emitter.instruction("ldr x2, [sp, #56]");                                   // key_hi (-1 for int keys)
    emitter.instruction("mov x4, #0");                                          // value_hi unused
    emitter.instruction("mov x5, #7");                                          // value tag = boxed Mixed (transfer the box)
    emitter.instruction("bl __rt_hash_set");                                    // insert the entry -> x0 = (possibly new) hash
    emitter.instruction("str x0, [sp, #80]");                                   // save the updated $data hash pointer
    emitter.instruction("ldr x4, [sp, #40]");                                   // reload the entry index
    emitter.instruction("add x4, x4, #1");                                      // advance the entry index
    emitter.instruction("str x4, [sp, #40]");                                   // persist the entry index
    emitter.instruction("b __rt_unser_obj_data_loop");                          // continue with the next entry
    emitter.label("__rt_unser_obj_data_done");
    emitter.instruction("ldr x0, [sp, #24]");                                   // $this receiver = first argument
    emitter.instruction("ldr x1, [sp, #80]");                                   // $data assoc array (bare hash) = second argument
    emitter.instruction("ldr x10, [sp, #72]");                                  // reload the __unserialize target
    emitter.instruction("blr x10");                                             // call __unserialize($this, $data)
    emitter.instruction("b __rt_unser_at_obj_box");                             // box the object (position is at the closing '}')
    emitter.label("__rt_unser_obj_default");
    emitter.instruction("ldr x9, [sp, #80]");                                   // blocked objects own an opaque Mixed property hash
    emitter.instruction("cbnz x9, __rt_unser_obj_default_props");               // hydrated objects use their declared property slots
    emitter.instruction("ldr x0, [sp, #32]");                                   // property count is the hash capacity hint
    emitter.instruction("mov x1, #7");                                          // values are boxed Mixed cells
    emitter.instruction("bl __rt_hash_new");                                    // allocate property hash before parsing values
    emitter.instruction("ldr x9, [sp, #24]");                                   // incomplete-object payload
    emitter.instruction("str x0, [x9, #24]");                                   // transfer hash ownership into incomplete object
    emitter.label("__rt_unser_obj_default_props");
    emitter.instruction("str xzr, [sp, #40]");                                  // initialize the property index
    emitter.label("__rt_unser_at_obj_loop");
    emitter.instruction("ldr x4, [sp, #40]");                                   // reload the property index
    emitter.instruction("ldr x3, [sp, #32]");                                   // reload the property count
    emitter.instruction("cmp x4, x3");                                          // all properties parsed?
    emitter.instruction("b.ge __rt_unser_at_obj_close");                        // box the object when done
    emitter.instruction("ldr x0, [sp, #0]");                                    // base
    emitter.instruction("ldr x1, [sp, #8]");                                    // current position
    emitter.instruction("ldr x2, [sp, #16]");                                   // end
    emitter.instruction("bl __rt_unser_key");                                   // parse the mangled key -> x0=key_ptr, x1=key_len, x2=newpos
    emitter.instruction("str x0, [sp, #48]");                                   // save the key pointer
    emitter.instruction("str x1, [sp, #56]");                                   // save the key length
    emitter.instruction("str x2, [sp, #8]");                                    // advance past the key
    emitter.instruction("ldr x0, [sp, #0]");                                    // base
    emitter.instruction("ldr x1, [sp, #8]");                                    // position after the key
    emitter.instruction("ldr x2, [sp, #16]");                                   // end
    emitter.instruction("bl __rt_unser_at");                                    // recursively parse the value -> x0=box, x1=newpos
    emitter.instruction("str x1, [sp, #8]");                                    // advance past the value
    emitter.instruction("mov x3, x0");                                          // value box
    emitter.instruction("ldr x0, [sp, #24]");                                   // object pointer
    emitter.instruction("ldr x1, [sp, #48]");                                   // key pointer
    emitter.instruction("ldr x2, [sp, #56]");                                   // key length
    emitter.instruction("ldr x9, [sp, #80]");                                   // blocked objects keep their wire properties opaque
    emitter.instruction("cbz x9, __rt_unser_obj_store_opaque_prop");            // blocked objects retain the parsed property semantically
    emitter.instruction("bl __rt_obj_store_prop");                              // store the value into the matching property slot
    emitter.instruction("b __rt_unser_obj_skip_prop_store");                    // transferred value now belongs to the hydrated object
    emitter.label("__rt_unser_obj_store_opaque_prop");
    emitter.instruction("ldr x0, [sp, #24]");                                   // incomplete-object payload
    emitter.instruction("ldr x0, [x0, #24]");                                   // opaque property hash
    emitter.instruction("ldr x1, [sp, #48]");                                   // serialized property key pointer
    emitter.instruction("ldr x2, [sp, #56]");                                   // serialized property key length
    emitter.instruction("mov x4, #0");                                          // boxed Mixed values have no high payload word
    emitter.instruction("mov x5, #7");                                          // transfer the parsed box as a Mixed hash value
    emitter.instruction("bl __rt_hash_set");                                    // insert property, preserving key/value ownership and order
    emitter.instruction("ldr x9, [sp, #24]");                                   // incomplete-object payload after a possible hash grow
    emitter.instruction("str x0, [x9, #24]");                                   // retain updated property hash pointer
    emitter.label("__rt_unser_obj_skip_prop_store");
    emitter.instruction("ldr x4, [sp, #40]");                                   // reload the property index
    emitter.instruction("add x4, x4, #1");                                      // advance the property index
    emitter.instruction("str x4, [sp, #40]");                                   // persist the property index
    emitter.instruction("b __rt_unser_at_obj_loop");                            // continue with the next property
    emitter.label("__rt_unser_at_obj_close");
    // -- __wakeup magic: after default property injection, call __wakeup($this) --
    emitter.instruction("ldr x9, [sp, #80]");                                   // blocked classes cannot run __wakeup
    emitter.instruction("cbz x9, __rt_unser_at_obj_box");                       // incomplete objects never run class hooks
    emitter.label("__rt_unser_obj_wakeup");
    emitter.instruction("ldr x9, [sp, #24]");                                   // object pointer
    emitter.instruction("ldr x9, [x9]");                                        // class id from the object header
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_class_wakeup_ptrs");
    emitter.instruction("ldr x10, [x10, x9, lsl #3]");                          // __wakeup method symbol (0 if none)
    emitter.instruction("cbz x10, __rt_unser_at_obj_box");                      // no __wakeup → box the object directly
    emitter.instruction("ldr x0, [sp, #24]");                                   // $this receiver
    emitter.instruction("blr x10");                                             // call __wakeup($this)
    emitter.label("__rt_unser_at_obj_box");
    emitter.instruction("mov x0, #24");                                         // box the object: Mixed cell = tag + two payload words
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the boxed Mixed cell
    emitter.instruction("mov x9, #5");                                          // heap kind 5 = boxed Mixed cell
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the heap header
    emitter.instruction("mov x9, #6");                                          // value tag 6 = object
    emitter.instruction("str x9, [x0]");                                        // store the value tag
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload the object pointer
    emitter.instruction("str x9, [x0, #8]");                                    // store the object pointer (ownership transferred)
    emitter.instruction("str xzr, [x0, #16]");                                  // clear the high payload word
    // -- register this object box so a later r:<index>; resolves to the same
    //    object (its index was reserved before its properties were parsed) --
    emitter.instruction("ldr x9, [sp, #88]");                                   // reserved value index for this object
    emitter.instruction("mov x10, #65536");                                     // value-registry capacity
    emitter.instruction("cmp x9, x10");                                         // is the registry full?
    emitter.instruction("b.ge __rt_unser_obj_box_noreg");                       // overflow → skip registration
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_unser_values");
    emitter.instruction("str x0, [x10, x9, lsl #3]");                           // values[index] = this object box
    emitter.label("__rt_unser_obj_box_noreg");
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload position (at the closing '}')
    emitter.instruction("add x1, x1, #1");                                      // newpos skips the '}'
    emitter.instruction("b __rt_unser_at_ret");                                 // return the box and new position

    // -- failure: null box, position unchanged --
    emitter.label("__rt_unser_at_fail");
    emitter.instruction("mov x0, #0");                                          // null result signals parse failure
    emitter.instruction("ldr x1, [sp, #8]");                                    // newpos = unchanged position

    emitter.label("__rt_unser_at_ret");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_depth");
    emitter.instruction("ldr x10, [x9]");                                       // load parser depth before returning to the caller
    emitter.instruction("sub x10, x10, #1");                                    // release this completed recursive parser frame
    emitter.instruction("str x10, [x9]");                                       // keep sibling parses independent
    emitter.instruction("ldp x29, x30, [sp, #96]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #112");                                    // deallocate the parser frame
    emitter.instruction("ret");                                                 // return x0=box, x1=newpos

    emitter.label_shared("__rt_unser_depth_fatal");
    emitter.instruction("mov x0, #2");                                          // stderr file descriptor
    crate::codegen_support::abi::emit_symbol_address(emitter, "x1", "_unser_depth_msg");
    emitter.instruction("mov x2, #48");                                         // complete unserialize-depth fatal diagnostic length
    emitter.syscall(4);                                                          // write the fatal diagnostic without recursing further
    emitter.instruction("mov x0, #1");                                          // non-zero failure status
    emitter.syscall(1);                                                          // terminate the hostile parse immediately

    // -- back-reference: r:N; / R:N; -> a fresh box aliasing the Nth parsed value.
    //    N is 1-based (PHP's value index); objects are retained so refcounts stay
    //    balanced. An out-of-range or never-registered index yields null. --
    emitter.label("__rt_unser_at_ref");
    emitter.instruction("ldr x10, [sp, #0]");                                   // base
    emitter.instruction("ldr x11, [sp, #8]");                                   // position
    emitter.instruction("add x10, x10, x11");                                   // pointer to the leading 'r'/'R'
    emitter.instruction("add x10, x10, #2");                                    // skip the marker and ':'
    emitter.instruction("mov x11, #0");                                         // index accumulator
    emitter.label("__rt_unser_at_ref_loop");
    emitter.instruction("ldrb w9, [x10]");                                      // next byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_at_ref_done");                         // terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_at_ref_done");                         // terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x13, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x13");                                   // shift the accumulator
    emitter.instruction("add x11, x11, x9");                                    // add the digit
    emitter.instruction("add x10, x10, #1");                                    // advance the cursor
    emitter.instruction("b __rt_unser_at_ref_loop");                            // continue
    emitter.label("__rt_unser_at_ref_done");
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload base
    emitter.instruction("sub x12, x10, x9");                                    // offset of the ';'
    emitter.instruction("add x12, x12, #1");                                    // newpos skips the ';'
    emitter.instruction("str x12, [sp, #8]");                                   // save the new position
    emitter.instruction("cbz x11, __rt_unser_at_ref_fail");                     // index 0 is invalid
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_count");
    emitter.instruction("ldr x9, [x9]");                                        // number of registered values
    emitter.instruction("cmp x11, x9");                                         // index beyond what was parsed?
    emitter.instruction("b.gt __rt_unser_at_ref_fail");                         // out of range → null
    emitter.instruction("sub x12, x11, #1");                                    // 0-based registry slot
    emitter.instruction("mov x10, #65536");                                     // materialize the physical reference-registry capacity
    emitter.instruction("cmp x12, x10");                                        // would this logical reference index exceed the registry?
    emitter.instruction("b.hs __rt_unser_at_ref_fail");                         // fail closed instead of reading beyond the fixed registry
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_unser_values");
    emitter.instruction("ldr x13, [x9, x12, lsl #3]");                          // the registered value box (0 if none)
    emitter.instruction("cbz x13, __rt_unser_at_ref_fail");                     // nothing registered (e.g. a cycle) → null
    emitter.instruction("str x13, [sp, #64]");                                  // save the source box across the alloc
    emitter.instruction("mov x0, #24");                                         // a fresh boxed Mixed cell
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate it
    emitter.instruction("ldr x13, [sp, #64]");                                  // reload the source box
    emitter.instruction("ldur x9, [x13, #-8]");                                 // source heap header
    emitter.instruction("str x9, [x0, #-8]");                                   // copy the heap header
    emitter.instruction("ldr x9, [x13]");                                       // source value tag
    emitter.instruction("str x9, [x0]");                                        // copy the value tag
    emitter.instruction("ldr x10, [x13, #8]");                                  // source low payload (object pointer)
    emitter.instruction("str x10, [x0, #8]");                                   // copy the low payload
    emitter.instruction("ldr x11, [x13, #16]");                                 // source high payload
    emitter.instruction("str x11, [x0, #16]");                                  // copy the high payload
    emitter.instruction("cmp x9, #6");                                          // does the alias point at an object?
    emitter.instruction("b.ne __rt_unser_at_ref_boxed");                        // non-objects need no retain
    emitter.instruction("str x0, [sp, #64]");                                   // save the fresh box across the retain
    emitter.instruction("mov x0, x10");                                         // object pointer
    emitter.instruction("bl __rt_incref");                                      // retain the shared object
    emitter.instruction("ldr x0, [sp, #64]");                                   // reload the fresh box
    emitter.label("__rt_unser_at_ref_boxed");
    emitter.instruction("ldr x1, [sp, #8]");                                    // newpos past the ';'
    emitter.instruction("b __rt_unser_at_ret");                                 // return the aliasing box
    emitter.label("__rt_unser_at_ref_fail");
    emitter.instruction("mov x0, #0");                                          // unresolved reference → null
    emitter.instruction("ldr x1, [sp, #8]");                                    // newpos past the ';'
    emitter.instruction("b __rt_unser_at_ret");                                 // return the null result

    emit_unserialize_context_aarch64(emitter);

    // -- __rt_obj_store_prop(x0=obj, x1=key_ptr, x2=key_len, x3=valbox): inject a property --
    // Matches the (mangled) key against the class's serialize property-info table and
    // stores the parsed value into the matching object slot per the property's tag.
    emitter.label_global("__rt_obj_store_prop");
    emitter.instruction("ldr x9, [x0]");                                        // class id from the object header
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_class_serprop_ptrs");
    emitter.instruction("ldr x10, [x10, x9, lsl #3]");                          // property-info table for this class
    emitter.instruction("ldr x11, [x10]");                                      // property count
    emitter.instruction("add x12, x10, #8");                                    // rows start (skip the count word)
    emitter.instruction("mov x13, #0");                                         // row index
    emitter.label("__rt_obj_store_prop_loop");
    emitter.instruction("cmp x13, x11");                                        // scanned every row?
    emitter.instruction("b.ge __rt_obj_store_prop_done");                       // unknown key is ignored
    emitter.instruction("add x14, x12, x13, lsl #5");                           // row = rows + index*32
    emitter.instruction("ldr x4, [x14]");                                       // row mangled key pointer
    emitter.instruction("ldr x5, [x14, #8]");                                   // row mangled key length
    emitter.instruction("cmp x5, x2");                                          // same length as the parsed key?
    emitter.instruction("b.ne __rt_obj_store_prop_next");                       // lengths differ, skip
    emitter.instruction("mov x6, #0");                                          // byte compare cursor
    emitter.label("__rt_obj_store_prop_cmp");
    emitter.instruction("cmp x6, x2");                                          // compared all bytes?
    emitter.instruction("b.ge __rt_obj_store_prop_match");                      // full match
    emitter.instruction("ldrb w7, [x4, x6]");                                   // row key byte
    emitter.instruction("ldrb w8, [x1, x6]");                                   // parsed key byte
    emitter.instruction("cmp w7, w8");                                          // bytes equal?
    emitter.instruction("b.ne __rt_obj_store_prop_next");                       // mismatch, skip this row
    emitter.instruction("add x6, x6, #1");                                      // next byte
    emitter.instruction("b __rt_obj_store_prop_cmp");                           // continue comparing
    emitter.label("__rt_obj_store_prop_match");
    emitter.instruction("ldr x6, [x14, #16]");                                  // property byte offset
    emitter.instruction("ldr x7, [x14, #24]");                                  // property value tag
    emitter.instruction("add x8, x0, x6");                                      // address of the property slot
    emitter.instruction("cmp x7, #7");                                          // is this a Mixed/untyped slot?
    emitter.instruction("b.eq __rt_obj_store_prop_mixed");                      // store the boxed cell directly
    emitter.instruction("cmp x7, #1");                                          // is this a string slot?
    emitter.instruction("b.eq __rt_obj_store_prop_str");                        // store pointer and length
    emitter.instruction("cmp x7, #4");                                          // is this an indexed-array slot?
    emitter.instruction("b.eq __rt_obj_store_prop_arr");                        // convert the parsed hash to an indexed array
    emitter.instruction("ldr x9, [x3, #8]");                                    // typed scalar/object/hash: unbox the low word
    emitter.instruction("str x9, [x8]");                                        // store it inline in the slot
    emitter.instruction("ret");                                                 // property stored
    emitter.label("__rt_obj_store_prop_arr");
    emitter.instruction("stp x8, x30, [sp, #-16]!");                            // save the slot address and return address
    emitter.instruction("ldr x0, [x3, #8]");                                    // parsed hash pointer (box low word)
    emitter.instruction("bl __rt_hash_to_indexed_array");                       // materialize a native indexed array
    emitter.instruction("ldp x8, x30, [sp], #16");                              // restore the slot address and return address
    emitter.instruction("str x0, [x8]");                                        // store the indexed-array pointer
    emitter.instruction("ret");                                                 // property stored
    emitter.label("__rt_obj_store_prop_str");
    emitter.instruction("ldr x9, [x3, #8]");                                    // string pointer from the box
    emitter.instruction("str x9, [x8]");                                        // store the string pointer
    emitter.instruction("ldr x9, [x3, #16]");                                   // string length from the box
    emitter.instruction("str x9, [x8, #8]");                                    // store the string length
    emitter.instruction("ret");                                                 // property stored
    emitter.label("__rt_obj_store_prop_mixed");
    emitter.instruction("ldr x9, [x3]");                                        // boxed value tag
    emitter.instruction("cmp x9, #8");                                          // is the boxed value null?
    emitter.instruction("b.eq __rt_obj_store_prop_mixed_null");                 // store the null sentinel
    emitter.instruction("str x3, [x8]");                                        // store the boxed Mixed cell pointer
    emitter.instruction("ret");                                                 // property stored
    emitter.label("__rt_obj_store_prop_mixed_null");
    crate::codegen_support::abi::emit_load_int_immediate(emitter, "x9", crate::codegen_support::NULL_SENTINEL);
    emitter.instruction("str x9, [x8]");                                        // store the in-band null sentinel
    emitter.instruction("str xzr, [x8, #8]");                                   // clear the high word
    emitter.instruction("ret");                                                 // property stored
    emitter.label("__rt_obj_store_prop_next");
    emitter.instruction("add x13, x13, #1");                                    // advance to the next row
    emitter.instruction("b __rt_obj_store_prop_loop");                          // continue scanning
    emitter.label("__rt_obj_store_prop_done");
    emitter.instruction("ret");                                                 // no matching property, ignore the value

    // -- __rt_hash_to_indexed_array(x0=hash) -> x0=indexed array: rebuild a parsed
    // hash (with boxed-Mixed values) as a native value_type-7 indexed array so
    // indexed-array-typed property slots match what property access expects. --
    emitter.label_global("__rt_hash_to_indexed_array");
    emitter.instruction("stp x29, x30, [sp, #-48]!");                           // open the conversion frame
    emitter.instruction("mov x29, sp");                                         // set the frame pointer
    emitter.instruction("stp x19, x20, [sp, #16]");                             // save callee-saved temporaries
    emitter.instruction("str x21, [sp, #32]");                                  // save callee-saved cursor
    emitter.instruction("mov x19, x0");                                         // hash pointer
    emitter.instruction("mov x0, #0");                                          // initial capacity 0
    emitter.instruction("mov x1, #8");                                          // 8-byte element slots
    emitter.instruction("bl __rt_array_new");                                   // allocate an empty indexed array
    emitter.instruction("mov x20, x0");                                         // destination array pointer
    emitter.instruction("mov x21, #0");                                         // hash iteration cursor
    emitter.label("__rt_hash_to_indexed_array_loop");
    emitter.instruction("mov x0, x19");                                         // hash pointer
    emitter.instruction("mov x1, x21");                                         // resume cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // x3=value low, x5=value tag, x0=next cursor
    emitter.instruction("cmn x0, #1");                                          // cursor == -1 (iteration done)?
    emitter.instruction("b.eq __rt_hash_to_indexed_array_done");                // stop when exhausted
    emitter.instruction("mov x21, x0");                                         // save the resume cursor
    emitter.instruction("mov x0, x20");                                         // destination array
    emitter.instruction("mov x1, x3");                                          // boxed-Mixed value pointer (parsed-hash value)
    emitter.instruction("bl __rt_array_push_refcounted");                       // append, transferring ownership
    emitter.instruction("mov x20, x0");                                         // array may move on COW growth
    emitter.instruction("b __rt_hash_to_indexed_array_loop");                   // continue iterating
    emitter.label("__rt_hash_to_indexed_array_done");
    emitter.instruction("mov x0, x20");                                         // return the indexed array
    emitter.instruction("ldr x21, [sp, #32]");                                  // restore the cursor register
    emitter.instruction("ldp x19, x20, [sp, #16]");                             // restore the temporaries
    emitter.instruction("ldp x29, x30, [sp], #48");                             // close the conversion frame
    emitter.instruction("ret");                                                 // return the converted array

    emit_unser_key_aarch64(emitter);
}

/// Emits the AArch64 leaf key parser `__rt_unser_key`.
///
/// Input: `x0`=base, `x1`=pos, `x2`=end. Output: `x0`=key_lo (int value or string
/// pointer), `x1`=key_hi (-1 for an integer key, else the string byte length), `x2`=newpos.
/// String key pointers are borrowed into the source buffer; `__rt_hash_set` persists them.
fn emit_unser_key_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unser_key (serialize() array key parser, leaf) ---");
    emitter.label_global("__rt_unser_key");
    emitter.instruction("cmp x1, x2");                                          // require a key type byte before loading it
    emitter.instruction("b.hs __rt_unser_key_fail");                            // return a sentinel cursor for a truncated key
    emitter.instruction("ldrb w9, [x0, x1]");                                   // load the key type byte
    emitter.instruction("cmp w9, #105");                                        // ASCII 'i' (integer key)?
    emitter.instruction("b.eq __rt_unser_key_int");                             // parse an integer key
    // -- string key: "s:" + bytelen + ":\"" + raw + "\";" --
    emitter.instruction("add x10, x0, x1");                                     // pointer to the type byte
    emitter.instruction("add x10, x10, #2");                                    // skip "s:" to the length digits
    emitter.instruction("mov x11, #0");                                         // length accumulator
    emitter.label("__rt_unser_key_strlen");
    emitter.instruction("ldrb w9, [x10]");                                      // next length byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_key_strlen_done");                     // ':' terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_key_strlen_done");                     // ':' terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x12, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x12");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x10, x10, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_key_strlen");                             // continue
    emitter.label("__rt_unser_key_strlen_done");
    emitter.instruction("add x10, x10, #2");                                    // skip ':' and opening '\"' to the raw bytes
    emitter.instruction("add x12, x10, x11");                                   // raw end = raw + len
    emitter.instruction("add x12, x12, #2");                                    // skip closing '\"' and ';'
    emitter.instruction("sub x2, x12, x0");                                     // newpos = (raw end + 2) - base
    emitter.instruction("mov x1, x11");                                         // key_hi = string byte length
    emitter.instruction("mov x0, x10");                                         // key_lo = borrowed raw string pointer
    emitter.instruction("ret");                                                 // return the string key
    // -- integer key: "i:" + optional '-' + digits + ";" --
    emitter.label("__rt_unser_key_int");
    emitter.instruction("add x10, x0, x1");                                     // pointer to the type byte
    emitter.instruction("add x10, x10, #2");                                    // skip "i:" to the first digit
    emitter.instruction("mov x11, #0");                                         // digit accumulator
    emitter.instruction("mov x13, #0");                                         // negative-sign flag
    emitter.instruction("ldrb w9, [x10]");                                      // first numeric byte
    emitter.instruction("cmp w9, #45");                                         // leading '-'?
    emitter.instruction("b.ne __rt_unser_key_int_loop");                        // no sign
    emitter.instruction("mov x13, #1");                                         // record negative sign
    emitter.instruction("add x10, x10, #1");                                    // skip '-'
    emitter.label("__rt_unser_key_int_loop");
    emitter.instruction("ldrb w9, [x10]");                                      // next numeric byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_key_int_done");                        // ';' terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_key_int_done");                        // ';' terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x12, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x12");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x10, x10, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_key_int_loop");                           // continue
    emitter.label("__rt_unser_key_int_done");
    emitter.instruction("cbz x13, __rt_unser_key_int_pos");                     // not signed
    emitter.instruction("neg x11, x11");                                        // apply sign
    emitter.label("__rt_unser_key_int_pos");
    emitter.instruction("sub x2, x10, x0");                                     // newpos = cursor - base
    emitter.instruction("add x2, x2, #1");                                      // skip the ';'
    emitter.instruction("mov x0, x11");                                         // key_lo = integer key value
    emitter.instruction("mov x1, #-1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("ret");                                                 // return the integer key
    emitter.label("__rt_unser_key_fail");
    emitter.instruction("mov x0, #0");                                          // clear key payload on failure
    emitter.instruction("mov x1, #0");                                          // clear key metadata on failure
    emitter.instruction("add x2, x2, #1");                                      // end+1 is an impossible valid cursor
    emitter.instruction("ret");                                                 // caller/preflight rejects the sentinel
}

/// Emits the x86_64 allocation-free grammar preflight used before decoding.
///
/// This mirrors the AArch64 validator and rejects malformed cursors, overflowing
/// decimal fields, and unterminated containers before the mutating parser runs.
fn emit_unser_validator_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: bounded unserialize grammar preflight ---");

    // uint(base=rdi, pos=rsi, end=rdx, delimiter=cl) -> rax=ok, rsi=value, rdx=delimiter position
    emitter.label_global("__rt_unser_validate_uint");
    emitter.instruction("lea r8, [rdi + rsi]");                                 // absolute digit cursor
    emitter.instruction("lea r9, [rdi + rdx]");                                 // absolute source end
    emitter.instruction("xor r10d, r10d");                                      // unsigned accumulator
    emitter.instruction("xor r11d, r11d");                                      // parsed digit count
    emitter.label("__rt_unser_validate_uint_loop_x");
    emitter.instruction("cmp r8, r9");                                          // is another byte available?
    emitter.instruction("jae __rt_unser_validate_uint_fail_x");                 // truncated digit run has no delimiter
    emitter.instruction("movzx r12d, BYTE PTR [r8]");                           // inspect one bounded byte
    emitter.instruction("cmp r12d, 48");                                        // below ASCII zero?
    emitter.instruction("jb __rt_unser_validate_uint_done_x");                  // require the requested delimiter below
    emitter.instruction("cmp r12d, 57");                                        // above ASCII nine?
    emitter.instruction("ja __rt_unser_validate_uint_done_x");                  // require the requested delimiter below
    emitter.instruction("sub r12d, 48");                                        // convert the byte to a digit
    emitter.instruction("mov r13, 1844674407370955161");                        // floor(u64::MAX / 10)
    emitter.instruction("cmp r10, r13");                                        // would multiplication overflow?
    emitter.instruction("ja __rt_unser_validate_uint_fail_x");
    emitter.instruction("jne __rt_unser_validate_uint_mul_x");
    emitter.instruction("cmp r12d, 5");                                         // final digit limit when accumulator equals the threshold
    emitter.instruction("ja __rt_unser_validate_uint_fail_x");
    emitter.label("__rt_unser_validate_uint_mul_x");
    emitter.instruction("imul r10, r10, 10");                                   // shift the accumulator by one decimal place
    emitter.instruction("add r10, r12");                                        // append the current digit
    emitter.instruction("add r11, 1");                                          // record one valid digit
    emitter.instruction("add r8, 1");                                           // advance within the proven source extent
    emitter.instruction("jmp __rt_unser_validate_uint_loop_x");
    emitter.label("__rt_unser_validate_uint_done_x");
    emitter.instruction("test r11, r11");                                       // was at least one digit parsed?
    emitter.instruction("jz __rt_unser_validate_uint_fail_x");
    emitter.instruction("cmp r12b, cl");                                        // did the run end on its grammar delimiter?
    emitter.instruction("jne __rt_unser_validate_uint_fail_x");
    emitter.instruction("mov rdx, r8");                                         // absolute delimiter cursor
    emitter.instruction("sub rdx, rdi");                                        // return delimiter position as an offset
    emitter.instruction("mov rsi, r10");                                        // return parsed value
    emitter.instruction("mov eax, 1");                                          // report success
    emitter.instruction("ret");
    emitter.label("__rt_unser_validate_uint_fail_x");
    emitter.instruction("xor eax, eax");                                        // report a bounded numeric failure
    emitter.instruction("ret");

    // key(base=rdi, pos=rsi, end=rdx, depth=rcx) -> rax=ok, rdx=newpos
    emitter.label_global("__rt_unser_validate_key");
    emitter.instruction("cmp rsi, rdx");                                        // require the key type byte before loading it
    emitter.instruction("jae __rt_unser_validate_key_fail_x");
    emitter.instruction("movzx r8d, BYTE PTR [rdi + rsi]");                     // inspect the bounded key type
    emitter.instruction("cmp r8d, 105");                                        // integer key?
    emitter.instruction("je __rt_unser_validate_key_dispatch_x");               // join through a local conditional target
    emitter.instruction("cmp r8d, 115");                                        // string key?
    emitter.instruction("jne __rt_unser_validate_key_fail_x");                  // reject every other key marker
    emitter.label("__rt_unser_validate_key_dispatch_x");
    emitter.instruction("jmp __rt_unser_validate_at");                          // main validator owns integer/string grammar
    emitter.label("__rt_unser_validate_key_fail_x");
    emitter.instruction("xor eax, eax");                                        // only integer and string keys are valid
    emitter.instruction("ret");

    // at(base=rdi, pos=rsi, end=rdx, depth=rcx) -> rax=ok, rdx=newpos
    emitter.label_global("__rt_unser_validate_at");
    emitter.instruction("push rbp");                                            // preserve the caller frame
    emitter.instruction("mov rbp, rsp");                                        // establish recursive validator frame
    emitter.instruction("sub rsp, 48");                                         // reserve base/pos/end/depth/count/index state
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // source base
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // starting position
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // source end
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // recursion depth
    emitter.instruction("cmp rsi, rdx");                                        // require a type byte before dispatch
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp rcx, 512");                                        // enforce parser recursion ceiling
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("movzx r8d, BYTE PTR [rdi + rsi]");                     // bounded type byte
    emitter.instruction("cmp r8d, 78");
    emitter.instruction("je __rt_unser_validate_null_x");
    emitter.instruction("cmp r8d, 98");
    emitter.instruction("je __rt_unser_validate_bool_x");
    emitter.instruction("cmp r8d, 105");
    emitter.instruction("je __rt_unser_validate_int_x");
    emitter.instruction("cmp r8d, 100");
    emitter.instruction("je __rt_unser_validate_float_x");
    emitter.instruction("cmp r8d, 115");
    emitter.instruction("je __rt_unser_validate_string_x");
    emitter.instruction("cmp r8d, 97");
    emitter.instruction("je __rt_unser_validate_array_x");
    emitter.instruction("cmp r8d, 79");
    emitter.instruction("je __rt_unser_validate_object_x");
    emitter.instruction("cmp r8d, 114");
    emitter.instruction("je __rt_unser_validate_ref_x");
    emitter.instruction("cmp r8d, 82");
    emitter.instruction("je __rt_unser_validate_ref_x");
    emitter.instruction("jmp __rt_unser_validate_at_fail_x");

    emitter.label("__rt_unser_validate_null_x");
    emitter.instruction("mov r8, rdx");                                         // bytes remaining from N
    emitter.instruction("sub r8, rsi");
    emitter.instruction("cmp r8, 2");                                           // N plus semicolon
    emitter.instruction("jb __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + rsi + 1], 59");                    // exact semicolon
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("mov rdx, rsi");                                        // seed new position from start
    emitter.instruction("add rdx, 2");                                          // skip N;
    emitter.instruction("jmp __rt_unser_validate_at_ok_x");

    emitter.label("__rt_unser_validate_bool_x");
    emitter.instruction("mov r8, rdx");
    emitter.instruction("sub r8, rsi");
    emitter.instruction("cmp r8, 4");                                           // exact b:<digit>; envelope
    emitter.instruction("jb __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + rsi + 1], 58");                    // colon after b
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("movzx r8d, BYTE PTR [rdi + rsi + 2]");
    emitter.instruction("cmp r8d, 48");
    emitter.instruction("je __rt_unser_validate_bool_delim_x");
    emitter.instruction("cmp r8d, 49");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.label("__rt_unser_validate_bool_delim_x");
    emitter.instruction("cmp BYTE PTR [rdi + rsi + 3], 59");                    // terminating semicolon
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("mov rdx, rsi");
    emitter.instruction("add rdx, 4");
    emitter.instruction("jmp __rt_unser_validate_at_ok_x");

    emitter.label("__rt_unser_validate_int_x");
    emitter.instruction("lea r8, [rsi + 1]");                                   // colon position
    emitter.instruction("cmp r8, rdx");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 58");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, 1");                                           // first sign/digit position
    emitter.instruction("cmp r8, rdx");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 45");
    emitter.instruction("jne __rt_unser_validate_int_digits_x");
    emitter.instruction("mov QWORD PTR [rbp - 48], 1");                         // record a negative integer
    emitter.instruction("add r8, 1");                                           // skip optional minus
    emitter.instruction("jmp __rt_unser_validate_int_scan_x");
    emitter.label("__rt_unser_validate_int_digits_x");
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // positive integer
    emitter.label("__rt_unser_validate_int_scan_x");
    emitter.instruction("mov rsi, r8");
    emitter.instruction("mov ecx, 59");                                         // integer terminator
    emitter.instruction("call __rt_unser_validate_uint");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov r8, 9223372036854775807");                         // i64::MAX magnitude
    emitter.instruction("cmp QWORD PTR [rbp - 48], 0");                         // negative-sign flag
    emitter.instruction("je __rt_unser_validate_int_positive_x");
    emitter.instruction("cmp rsi, r8");                                         // negative magnitude at most i64::MAX + 1
    emitter.instruction("jbe __rt_unser_validate_int_range_ok_x");
    emitter.instruction("sub rsi, r8");                                         // only one extra magnitude value is representable
    emitter.instruction("cmp rsi, 1");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("jmp __rt_unser_validate_int_range_ok_x");
    emitter.label("__rt_unser_validate_int_positive_x");
    emitter.instruction("cmp rsi, r8");                                         // positive magnitude at most i64::MAX
    emitter.instruction("ja __rt_unser_validate_at_fail_x");
    emitter.label("__rt_unser_validate_int_range_ok_x");
    emitter.instruction("add rdx, 1");                                          // skip semicolon
    emitter.instruction("jmp __rt_unser_validate_at_ok_x");

    emitter.label("__rt_unser_validate_float_x");
    emitter.instruction("lea r8, [rsi + 1]");                                   // colon position
    emitter.instruction("cmp r8, rdx");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 58");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, 1");                                           // first float byte
    emitter.instruction("mov r9, r8");                                          // remember start
    emitter.label("__rt_unser_validate_float_loop_x");
    emitter.instruction("cmp r8, rdx");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 59");
    emitter.instruction("je __rt_unser_validate_float_done_x");
    emitter.instruction("add r8, 1");
    emitter.instruction("jmp __rt_unser_validate_float_loop_x");
    emitter.label("__rt_unser_validate_float_done_x");
    emitter.instruction("cmp r8, r9");                                          // reject empty float payload
    emitter.instruction("je __rt_unser_validate_at_fail_x");
    emitter.instruction("lea rdx, [r8 + 1]");                                   // position after semicolon
    emitter.instruction("jmp __rt_unser_validate_at_ok_x");

    emitter.label("__rt_unser_validate_string_x");
    emitter.instruction("lea r8, [rsi + 1]");                                   // colon after s
    emitter.instruction("cmp r8, rdx");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 58");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("lea rsi, [r8 + 1]");                                   // first length digit
    emitter.instruction("mov ecx, 58");                                         // length delimiter
    emitter.instruction("call __rt_unser_validate_uint");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov r11, rsi");                                        // declared string length
    emitter.instruction("lea r8, [rdx + 1]");                                   // opening quote position
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // end
    emitter.instruction("cmp r8, r9");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // base after helper call
    emitter.instruction("cmp BYTE PTR [rdi + r8], 34");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, 1");                                           // raw payload position
    emitter.instruction("mov r10, r9");
    emitter.instruction("sub r10, r8");                                         // remaining bytes
    emitter.instruction("cmp r10, 2");                                          // closing quote plus semicolon
    emitter.instruction("jb __rt_unser_validate_at_fail_x");
    emitter.instruction("sub r10, 2");
    emitter.instruction("cmp r11, r10");                                        // declared payload fits?
    emitter.instruction("ja __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, r11");                                         // closing quote position
    emitter.instruction("cmp BYTE PTR [rdi + r8], 34");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, 1");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 59");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("lea rdx, [r8 + 1]");
    emitter.instruction("jmp __rt_unser_validate_at_ok_x");

    emitter.label("__rt_unser_validate_array_x");
    emitter.instruction("lea r8, [rsi + 1]");                                   // colon after a
    emitter.instruction("cmp r8, rdx");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 58");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("lea rsi, [r8 + 1]");                                   // first count digit
    emitter.instruction("mov ecx, 58");
    emitter.instruction("call __rt_unser_validate_uint");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov QWORD PTR [rbp - 40], rsi");                       // entry count
    emitter.instruction("lea r8, [rdx + 1]");                                   // opening brace position
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");
    emitter.instruction("cmp r8, r9");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 123");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, 1");
    emitter.instruction("mov QWORD PTR [rbp - 16], r8");                        // body position
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // entry index
    emitter.label("__rt_unser_validate_array_loop_x");
    emitter.instruction("mov r8, QWORD PTR [rbp - 48]");
    emitter.instruction("cmp r8, QWORD PTR [rbp - 40]");
    emitter.instruction("jae __rt_unser_validate_container_close_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");
    emitter.instruction("add rcx, 1");                                          // nested key depth
    emitter.instruction("call __rt_unser_validate_key");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // position after key
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("mov rsi, rdx");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");
    emitter.instruction("add rcx, 1");                                          // nested value depth
    emitter.instruction("call __rt_unser_validate_at");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // position after value
    emitter.instruction("add QWORD PTR [rbp - 48], 1");
    emitter.instruction("jmp __rt_unser_validate_array_loop_x");

    emitter.label("__rt_unser_validate_object_x");
    emitter.instruction("lea r8, [rsi + 1]");                                   // colon after O
    emitter.instruction("cmp r8, rdx");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 58");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("lea rsi, [r8 + 1]");                                   // first class-name length digit
    emitter.instruction("mov ecx, 58");
    emitter.instruction("call __rt_unser_validate_uint");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov r11, rsi");                                        // class-name byte length
    emitter.instruction("lea r8, [rdx + 1]");                                   // opening quote position
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");
    emitter.instruction("cmp r8, r9");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 34");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, 1");                                           // class-name bytes
    emitter.instruction("mov r10, r9");
    emitter.instruction("sub r10, r8");
    emitter.instruction("cmp r10, 2");                                          // closing quote and colon
    emitter.instruction("jb __rt_unser_validate_at_fail_x");
    emitter.instruction("sub r10, 2");
    emitter.instruction("cmp r11, r10");
    emitter.instruction("ja __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, r11");                                         // closing quote
    emitter.instruction("cmp BYTE PTR [rdi + r8], 34");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, 1");                                           // colon before count
    emitter.instruction("cmp BYTE PTR [rdi + r8], 58");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("lea rsi, [r8 + 1]");                                   // first property-count digit
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");
    emitter.instruction("mov ecx, 58");
    emitter.instruction("call __rt_unser_validate_uint");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov QWORD PTR [rbp - 40], rsi");                       // property count
    emitter.instruction("lea r8, [rdx + 1]");                                   // opening brace
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");
    emitter.instruction("cmp r8, r9");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 123");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add r8, 1");
    emitter.instruction("mov QWORD PTR [rbp - 16], r8");                        // first property key
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // property index
    emitter.label("__rt_unser_validate_object_loop_x");
    emitter.instruction("mov r8, QWORD PTR [rbp - 48]");
    emitter.instruction("cmp r8, QWORD PTR [rbp - 40]");
    emitter.instruction("jae __rt_unser_validate_container_close_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");
    emitter.instruction("add rcx, 1");
    emitter.instruction("call __rt_unser_validate_key");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("mov rsi, rdx");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");
    emitter.instruction("add rcx, 1");
    emitter.instruction("call __rt_unser_validate_at");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");
    emitter.instruction("add QWORD PTR [rbp - 48], 1");
    emitter.instruction("jmp __rt_unser_validate_object_loop_x");

    emitter.label("__rt_unser_validate_container_close_x");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // closing-brace position
    emitter.instruction("cmp rdx, QWORD PTR [rbp - 24]");                       // require the closing brace byte
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("cmp BYTE PTR [rdi + rdx], 125");                       // exact closing brace
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("add rdx, 1");                                          // position after complete container
    emitter.instruction("jmp __rt_unser_validate_at_ok_x");

    emitter.label("__rt_unser_validate_ref_x");
    emitter.instruction("lea r8, [rsi + 1]");                                   // colon after r/R
    emitter.instruction("cmp r8, rdx");
    emitter.instruction("jae __rt_unser_validate_at_fail_x");
    emitter.instruction("cmp BYTE PTR [rdi + r8], 58");
    emitter.instruction("jne __rt_unser_validate_at_fail_x");
    emitter.instruction("lea rsi, [r8 + 1]");                                   // first reference-index digit
    emitter.instruction("mov ecx, 59");
    emitter.instruction("call __rt_unser_validate_uint");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("test rsi, rsi");                                       // reference indices are one-based
    emitter.instruction("jz __rt_unser_validate_at_fail_x");
    emitter.instruction("add rdx, 1");                                          // skip semicolon

    emitter.label("__rt_unser_validate_at_ok_x");
    emitter.instruction("mov eax, 1");                                          // report a fully bounded value
    emitter.instruction("leave");                                               // restore recursive validator frame
    emitter.instruction("ret");
    emitter.label("__rt_unser_validate_at_fail_x");
    emitter.instruction("xor eax, eax");                                        // report malformed/truncated wire data
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // preserve original position on failure
    emitter.instruction("leave");
    emitter.instruction("ret");
}

/// x86_64 implementation of the unserialize entry, recursive parser, and key parser.
fn emit_unserialize_x86_64(emitter: &mut Emitter) {
    let boundary_bytes = TRY_HANDLER_SLOT_SIZE + 32;
    let previous_handler_offset = boundary_bytes;
    let survivor_offset = previous_handler_offset - 8;

    // -- entry wrapper: protect begin/end cleanup across hydration-hook throws --
    emitter.blank();
    emitter.comment("--- runtime: unserialize_mixed (serialize() wire -> boxed Mixed) ---");
    emitter.label_global("__rt_unserialize_mixed");
    emitter.instruction("push rbp");                                            // preserve the caller frame across the exception boundary
    emitter.instruction("mov rbp, rsp");                                        // establish a stable base for the complete handler record
    emitter.instruction(&format!("sub rsp, {}", boundary_bytes));               // reserve the handler record plus source/result spills
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve source pointer across setjmp
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // preserve source length across setjmp
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_exc_handler_top", 0);
    emitter.instruction(&format!("mov QWORD PTR [rbp - {}], r10", previous_handler_offset)); // handler.next = previous exception-handler head
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_exc_call_frame_top", 0);
    emitter.instruction(&format!("mov QWORD PTR [rbp - {}], r10", survivor_offset)); // preserve the activation frame that survives this boundary
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_rt_diag_suppression", 0);
    emitter.instruction(&format!("mov QWORD PTR [rbp - {}], r10", boundary_bytes - TRY_HANDLER_DIAG_DEPTH_OFFSET)); // snapshot diagnostic suppression across longjmp
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_runtime_recursion_stack_bytes", 0);
    emitter.instruction(&format!("mov QWORD PTR [rbp - {}], r10", boundary_bytes - TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET)); // snapshot the user-stack budget across longjmp
    emitter.instruction(&format!("lea r10, [rbp - {}]", previous_handler_offset)); // compute this wrapper's exception-handler record address
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_exc_handler_top", 0);
    emitter.instruction(&format!("lea rdi, [rbp - {}]", boundary_bytes - TRY_HANDLER_JMP_BUF_OFFSET)); // pass this boundary's opaque jmp_buf to setjmp
    emitter.bl_c("setjmp"); // catch Throwable control flow escaping hydration hooks
    emitter.instruction("test eax, eax");                                       // did control return through longjmp?
    emitter.instruction("jnz __rt_unserialize_mixed_throw_x");                  // clean runtime state before propagating the Throwable
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // base = preserved source string pointer
    emitter.instruction("xor esi, esi");                                        // start parsing at position 0
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // end = preserved source string length
    emitter.instruction("xor ecx, ecx");                                        // preflight starts at recursive depth zero
    emitter.instruction("call __rt_unser_validate_at");                         // reject truncated/overflowing grammar before allocating or running hooks
    emitter.instruction("test rax, rax");                                       // did the complete wire value validate?
    emitter.instruction("jz __rt_unserialize_mixed_invalid_x");                 // malformed input returns PHP false through the normal end path
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload base after caller-clobbered validator registers
    emitter.instruction("xor esi, esi");                                        // parse the already validated value from the beginning
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // restore the validated source extent
    emitter.instruction("call __rt_unser_at");                                  // parse while the cleanup boundary is active
    emitter.instruction("jmp __rt_unserialize_mixed_parsed_x");                 // share exception-boundary teardown with validation failures
    emitter.label("__rt_unserialize_mixed_invalid_x");
    emitter.instruction("xor eax, eax");                                        // null result signals a bounded parse failure
    emitter.label("__rt_unserialize_mixed_parsed_x");
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve the parsed box while popping the boundary
    emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", previous_handler_offset)); // reload the previous exception-handler head
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_exc_handler_top", 0);
    emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", boundary_bytes - TRY_HANDLER_DIAG_DEPTH_OFFSET)); // reload diagnostic suppression after the protected parse
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_rt_diag_suppression", 0);
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // recover the parsed Mixed result
    emitter.instruction("leave");                                               // release the exception boundary and restore the caller frame
    emitter.instruction("ret");                                                 // return the parsed box to the lowering's normal end path
    emitter.label("__rt_unserialize_mixed_throw_x");
    emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", previous_handler_offset)); // reload the handler preceding this internal boundary
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_exc_handler_top", 0);
    emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", boundary_bytes - TRY_HANDLER_DIAG_DEPTH_OFFSET)); // restore diagnostic suppression skipped by longjmp
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_rt_diag_suppression", 0);
    emitter.instruction(&format!("mov r10, QWORD PTR [rbp - {}]", boundary_bytes - TRY_HANDLER_RECURSION_STACK_BYTES_OFFSET)); // restore the user-stack budget skipped by longjmp
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_runtime_recursion_stack_bytes", 0);
    emitter.instruction("xor eax, eax");                                        // end cleanup ignores the placeholder parse result on throw
    emitter.instruction("call __rt_unserialize_end");                           // release policy/context state before propagating the Throwable
    emitter.instruction("leave");                                               // discard the protected parser stack through its boundary
    emitter.instruction("jmp __rt_throw_current");                              // resume propagation at the caller's exception handler

    emit_unser_validator_x86_64(emitter);

    // -- __rt_unser_at(base=rdi, pos=rsi, end=rdx) -> rax=box (0 fail), rdx=newpos --
    emitter.blank();
    emitter.comment("--- runtime: unser_at (recursive serialize() value parser) ---");
    emitter.label_global("__rt_unser_at");
    // [rbp-8]=base [16]=pos [24]=end [32]=hash [40]=count [48]=index [56]=key_lo [64]=key_hi [72]=scratch
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 96");                                         // recursive parser frame (with a reference-index slot)
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the base pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the current position
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save the end position
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r8", "_unser_depth", 0); // load current recursive unserialize depth
    emitter.instruction("add r8, 1");                                           // account for this parser frame
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r8", "_unser_depth", 0); // publish parser depth before consuming wire bytes
    emitter.instruction("cmp r8, 512");                                         // bound recursive frames before native-stack exhaustion
    emitter.instruction("jg __rt_unser_depth_fatal_x");                         // terminate hostile deeply nested serialized input
    emitter.instruction("cmp rsi, rdx");                                        // is the cursor already at/past the end?
    emitter.instruction("jge __rt_unser_at_fail");                              // nothing left to parse
    emitter.instruction("movzx r9d, BYTE PTR [rdi + rsi]");                     // load the leading type byte
    // -- back-reference? r:N; / R:N; resolves to a previously parsed value and
    //    consumes no new index --
    emitter.instruction("cmp r9d, 114");                                        // ASCII 'r'?
    emitter.instruction("je __rt_unser_at_ref");                                // resolve an object back-reference
    emitter.instruction("cmp r9d, 82");                                         // ASCII 'R'?
    emitter.instruction("je __rt_unser_at_ref");                                // resolve a PHP reference
    // -- every other value consumes the next pre-order index, mirroring serialize() --
    crate::codegen_support::abi::emit_symbol_address(emitter, "r10", "_unser_count");
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // current value index
    emitter.instruction("mov QWORD PTR [rbp - 88], r11");                       // reserve this value's index
    emitter.instruction("add r11, 1");                                          // advance the registry counter
    emitter.instruction("mov QWORD PTR [r10], r11");                            // publish the advanced counter
    emitter.instruction("mov r10, QWORD PTR [rbp - 88]");                       // recover the reserved zero-based registry slot
    emitter.instruction("cmp r10, 65536");                                      // is the reserved slot inside the fixed registry?
    emitter.instruction("jae __rt_unser_at_registry_slot_ready_x");             // out-of-capacity values remain deliberately unregistered
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_unser_values");
    emitter.instruction("mov QWORD PTR [r11 + r10 * 8], 0");                    // erase any stale object pointer before parsing this value
    emitter.label("__rt_unser_at_registry_slot_ready_x");
    emitter.instruction("cmp r9d, 78");                                         // ASCII 'N' (null)?
    emitter.instruction("je __rt_unser_at_null");                               // parse null
    emitter.instruction("cmp r9d, 98");                                         // ASCII 'b' (bool)?
    emitter.instruction("je __rt_unser_at_bool");                               // parse bool
    emitter.instruction("cmp r9d, 105");                                        // ASCII 'i' (int)?
    emitter.instruction("je __rt_unser_at_int");                                // parse int
    emitter.instruction("cmp r9d, 100");                                        // ASCII 'd' (float)?
    emitter.instruction("je __rt_unser_at_float");                              // parse float
    emitter.instruction("cmp r9d, 115");                                        // ASCII 's' (string)?
    emitter.instruction("je __rt_unser_at_str");                                // parse string
    emitter.instruction("cmp r9d, 97");                                         // ASCII 'a' (array)?
    emitter.instruction("je __rt_unser_at_array");                              // parse array
    emitter.instruction("cmp r9d, 79");                                         // ASCII 'O' (object)?
    emitter.instruction("je __rt_unser_at_object");                             // parse object
    emitter.instruction("jmp __rt_unser_at_fail");                              // unsupported wire form

    // -- null: "N;" --
    emitter.label("__rt_unser_at_null");
    emitter.instruction("mov rax, 8");                                          // value tag = null
    emitter.instruction("mov rdi, 0");                                          // null payload low word
    emitter.instruction("mov rsi, 0");                                          // null payload high word
    emitter.instruction("call __rt_mixed_from_value");                          // box the null value
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload position
    emitter.instruction("add rdx, 2");                                          // newpos skips "N;"
    emitter.instruction("jmp __rt_unser_at_ret");                               // return box and new position

    // -- bool: "b:0;" / "b:1;" --
    emitter.label("__rt_unser_at_bool");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload base
    emitter.instruction("add r10, QWORD PTR [rbp - 16]");                       // pointer to the type byte
    emitter.instruction("movzx r9d, BYTE PTR [r10 + 2]");                       // load the bool digit at offset 2
    emitter.instruction("sub r9d, 48");                                         // ASCII '0'/'1' -> 0/1
    emitter.instruction("and r9, 1");                                           // clamp to a single bool bit
    emitter.instruction("mov rdi, r9");                                         // value payload = bool bit
    emitter.instruction("mov rax, 3");                                          // value tag = bool
    emitter.instruction("mov rsi, 0");                                          // bool high payload unused
    emitter.instruction("call __rt_mixed_from_value");                          // box the bool value
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload position
    emitter.instruction("add rdx, 4");                                          // newpos skips "b:X;"
    emitter.instruction("jmp __rt_unser_at_ret");                               // return box and new position

    // -- int: "i:" + optional '-' + digits + ";" --
    emitter.label("__rt_unser_at_int");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload base
    emitter.instruction("add r10, QWORD PTR [rbp - 16]");                       // pointer to the type byte
    emitter.instruction("add r10, 2");                                          // skip "i:" to the first digit
    emitter.instruction("xor r11, r11");                                        // digit accumulator
    emitter.instruction("xor r8, r8");                                          // negative-sign flag
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // first numeric byte
    emitter.instruction("cmp r9d, 45");                                         // leading '-'?
    emitter.instruction("jne __rt_unser_at_int_loop");                          // no sign
    emitter.instruction("mov r8, 1");                                           // record negative sign
    emitter.instruction("add r10, 1");                                          // skip '-'
    emitter.label("__rt_unser_at_int_loop");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next numeric byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_at_int_done");                           // terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_at_int_done");                           // terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_at_int_loop");                          // continue
    emitter.label("__rt_unser_at_int_done");
    emitter.instruction("test r8, r8");                                         // signed?
    emitter.instruction("jz __rt_unser_at_int_box");                            // not signed
    emitter.instruction("neg r11");                                             // apply sign
    emitter.label("__rt_unser_at_int_box");
    emitter.instruction("mov QWORD PTR [rbp - 72], r10");                       // save the cursor (at ';') across the box call
    emitter.instruction("mov rdi, r11");                                        // value payload = parsed int
    emitter.instruction("mov rax, 0");                                          // value tag = int
    emitter.instruction("mov rsi, 0");                                          // int high payload unused
    emitter.instruction("call __rt_mixed_from_value");                          // box the int value
    emitter.instruction("mov r10, QWORD PTR [rbp - 72]");                       // reload the cursor
    emitter.instruction("sub r10, QWORD PTR [rbp - 8]");                        // newpos = cursor - base
    emitter.instruction("add r10, 1");                                          // skip the ';'
    emitter.instruction("mov rdx, r10");                                        // newpos
    emitter.instruction("jmp __rt_unser_at_ret");                               // return box and new position

    // -- float: "d:" + (INF/-INF/NAN | digits) + ";" --
    emitter.label("__rt_unser_at_float");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload base
    emitter.instruction("add rdi, QWORD PTR [rbp - 16]");                       // pointer to the type byte
    emitter.instruction("add rdi, 2");                                          // strtod source = first byte after "d:"
    emitter.instruction("lea rsi, [rbp - 72]");                                 // strtod endptr = &scratch
    emitter.instruction("call strtod");                                         // parse the float (stops at ';') -> xmm0, scratch=endptr
    emitter.instruction("mov r10, QWORD PTR [rbp - 72]");                       // bounded conversion end pointer
    emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                        // source base
    emitter.instruction("add r11, QWORD PTR [rbp - 16]");                       // pointer to the type byte
    emitter.instruction("add r11, 2");                                          // first float payload byte
    emitter.instruction("cmp r10, r11");                                        // did strtod consume at least one byte?
    emitter.instruction("je __rt_unser_at_fail");                               // invalid numeric payload
    emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                        // source base
    emitter.instruction("add r11, QWORD PTR [rbp - 24]");                       // absolute source end
    emitter.instruction("cmp r10, r11");                                        // end pointer must still address a delimiter
    emitter.instruction("jae __rt_unser_at_fail");                              // reject a conversion escaping the source extent
    emitter.instruction("cmp BYTE PTR [r10], 59");                              // exact semicolon delimiter
    emitter.instruction("jne __rt_unser_at_fail");                              // reject partial conversions such as `1x;`
    emitter.instruction("movq r9, xmm0");                                       // move the parsed double into a GPR
    emitter.instruction("mov rdi, r9");                                         // value payload = float bits
    emitter.instruction("mov rax, 2");                                          // value tag = float
    emitter.instruction("mov rsi, 0");                                          // float high payload unused
    emitter.instruction("call __rt_mixed_from_value");                          // box the float value
    emitter.instruction("mov r10, QWORD PTR [rbp - 72]");                       // reload the strtod endptr
    emitter.instruction("sub r10, QWORD PTR [rbp - 8]");                        // newpos = endptr - base
    emitter.instruction("add r10, 1");                                          // skip the ';'
    emitter.instruction("mov rdx, r10");                                        // newpos
    emitter.instruction("jmp __rt_unser_at_ret");                               // return box and new position

    // -- string: "s:" + bytelen + ":\"" + raw + "\";" --
    emitter.label("__rt_unser_at_str");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload base
    emitter.instruction("add r10, QWORD PTR [rbp - 16]");                       // pointer to the type byte
    emitter.instruction("add r10, 2");                                          // skip "s:" to the length digits
    emitter.instruction("xor r11, r11");                                        // length accumulator
    emitter.label("__rt_unser_at_strlen");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next length byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_at_strlen_done");                        // ':' terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_at_strlen_done");                        // ':' terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_at_strlen");                            // continue
    emitter.label("__rt_unser_at_strlen_done");
    emitter.instruction("add r10, 2");                                          // skip ':' and opening '\"' to the raw bytes
    emitter.instruction("mov r8, r10");                                         // raw end accumulator = raw start
    emitter.instruction("add r8, r11");                                         // raw end = raw + len
    emitter.instruction("mov QWORD PTR [rbp - 72], r8");                        // save raw end across the box call
    emitter.instruction("mov rdi, r10");                                        // string payload pointer = raw bytes
    emitter.instruction("mov rsi, r11");                                        // string payload length
    emitter.instruction("mov rax, 1");                                          // value tag = string (mixed_from_value persists it)
    emitter.instruction("call __rt_mixed_from_value");                          // box an owned copy of the string
    emitter.instruction("mov r10, QWORD PTR [rbp - 72]");                       // reload raw end
    emitter.instruction("sub r10, QWORD PTR [rbp - 8]");                        // newpos = raw end - base
    emitter.instruction("add r10, 2");                                          // skip closing '\"' and ';'
    emitter.instruction("mov rdx, r10");                                        // newpos
    emitter.instruction("jmp __rt_unser_at_ret");                               // return box and new position

    // -- array: "a:" + count + ":{" + count*(key value) + "}" --
    emitter.label("__rt_unser_at_array");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload base
    emitter.instruction("add r10, QWORD PTR [rbp - 16]");                       // pointer to the type byte
    emitter.instruction("add r10, 2");                                          // skip "a:" to the count digits
    emitter.instruction("xor r11, r11");                                        // count accumulator
    emitter.label("__rt_unser_at_count");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next count byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_at_count_done");                         // ':' terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_at_count_done");                         // ':' terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_at_count");                             // continue
    emitter.label("__rt_unser_at_count_done");
    emitter.instruction("mov QWORD PTR [rbp - 40], r11");                       // save the entry count
    emitter.instruction("add r10, 2");                                          // skip ':' and '{' to the body
    emitter.instruction("sub r10, QWORD PTR [rbp - 8]");                        // body position offset
    emitter.instruction("mov QWORD PTR [rbp - 16], r10");                       // advance the cursor to the body
    emitter.instruction("mov rdi, r11");                                        // hash capacity = entry count
    emitter.instruction("mov rsi, 7");                                          // hash value_type = boxed Mixed
    emitter.instruction("call __rt_hash_new");                                  // allocate the destination hash
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the hash pointer
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // initialize the entry index
    emitter.label("__rt_unser_at_array_loop");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the entry index
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 40]");                       // all entries parsed?
    emitter.instruction("jge __rt_unser_at_array_close");                       // box the hash when done
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // base
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // current position
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // end
    emitter.instruction("call __rt_unser_key");                                 // parse the key -> rax=key_lo, rdx=key_hi, rcx=newpos
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 24]");                       // key parser must not escape the validated source
    emitter.instruction("ja __rt_unser_at_array_fail");                         // release the partially built hash on failure
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // save key_lo
    emitter.instruction("mov QWORD PTR [rbp - 64], rdx");                       // save key_hi
    emitter.instruction("mov QWORD PTR [rbp - 16], rcx");                       // advance past the key
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // base
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // position after the key
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // end
    emitter.instruction("call __rt_unser_at");                                  // recursively parse the value -> rax=box, rdx=newpos
    emitter.instruction("test rax, rax");                                       // did the child parse succeed?
    emitter.instruction("jz __rt_unser_at_array_fail");                         // child failure invalidates the whole array
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // advance past the value
    emitter.instruction("mov rcx, rax");                                        // value_lo = parsed value box
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // key_lo
    emitter.instruction("mov rdx, QWORD PTR [rbp - 64]");                       // key_hi (-1 for int keys)
    emitter.instruction("mov r8, 0");                                           // value_hi unused
    emitter.instruction("mov r9, 7");                                           // value tag = boxed Mixed (transfer the box)
    emitter.instruction("call __rt_hash_set");                                  // insert the entry -> rax = (possibly new) hash
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the updated hash pointer
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the entry index
    emitter.instruction("add rcx, 1");                                          // advance the entry index
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // persist the entry index
    emitter.instruction("jmp __rt_unser_at_array_loop");                        // continue with the next entry
    emitter.label("__rt_unser_at_array_close");
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                        // closing-brace position
    emitter.instruction("cmp r8, QWORD PTR [rbp - 24]");                        // require the closing delimiter byte
    emitter.instruction("jae __rt_unser_at_array_fail");
    emitter.instruction("mov r9, QWORD PTR [rbp - 8]");                         // source base
    emitter.instruction("cmp BYTE PTR [r9 + r8], 125");                         // exact `}`
    emitter.instruction("jne __rt_unser_at_array_fail");
    emitter.instruction("mov rax, 24");                                         // box the hash: Mixed cell = tag + two payload words
    emitter.instruction("call __rt_heap_alloc");                                // allocate the boxed Mixed cell
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the hash pointer
    emitter.instruction(&format!("mov r11, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(5))); // materialize the x86_64 boxed-Mixed heap kind word
    emitter.instruction("mov QWORD PTR [rax - 8], r11");                        // stamp the Mixed box without discarding the x86_64 heap marker
    emitter.instruction("mov QWORD PTR [rax], 5");                              // value tag 5 = associative array (hash)
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // store the hash pointer (ownership transferred)
    emitter.instruction("mov QWORD PTR [rax + 16], 0");                         // clear the high payload word
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload position (at the closing '}')
    emitter.instruction("add rdx, 1");                                          // newpos skips the '}'
    emitter.instruction("jmp __rt_unser_at_ret");                               // return box and new position
    emitter.label("__rt_unser_at_array_fail");
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // partially built hash pointer
    emitter.instruction("call __rt_hash_free_deep");                            // release keys and transferred boxed values locally
    emitter.instruction("xor eax, eax");                                        // report parse failure
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // preserve current cursor for the caller
    emitter.instruction("jmp __rt_unser_at_ret");                               // only the shared return decrements parser depth

    // -- object: "O:" + namelen + ":\"" + class + "\":" + count + ":{" + count*(key value) + "}" --
    emitter.label("__rt_unser_at_object");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload base
    emitter.instruction("add r10, QWORD PTR [rbp - 16]");                       // pointer to the type byte
    emitter.instruction("add r10, 2");                                          // skip "O:" to the class-name length digits
    emitter.instruction("xor r11, r11");                                        // class-name length accumulator
    emitter.label("__rt_unser_at_obj_namelen");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next length byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_at_obj_namelen_done");                   // ':' terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_at_obj_namelen_done");                   // ':' terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_at_obj_namelen");                       // continue
    emitter.label("__rt_unser_at_obj_namelen_done");
    emitter.instruction("add r10, 2");                                          // skip ':' and opening '\"' to the class name bytes
    emitter.instruction("mov r8, r10");                                         // class-name end accumulator = name start
    emitter.instruction("add r8, r11");                                         // class-name end = name + len
    emitter.instruction("mov QWORD PTR [rbp - 72], r8");                        // save the class-name end across the call
    emitter.instruction("mov QWORD PTR [rbp - 48], r10");                       // save class-name start across policy helper
    emitter.instruction("mov QWORD PTR [rbp - 56], r11");                       // save class-name length across policy helper
    emitter.instruction("mov rax, r10");                                        // class name pointer for allowed_classes policy
    emitter.instruction("mov rdx, r11");                                        // class name length for allowed_classes policy
    emitter.instruction("call __rt_unserialize_class_allowed");                 // decide whether hydration is permitted
    emitter.instruction("mov QWORD PTR [rbp - 80], rax");                       // retain policy result until hook/property dispatch
    emitter.instruction("test rax, rax");                                       // blocked classes become incomplete objects
    emitter.instruction("jz __rt_unser_obj_incomplete_x");
    emitter.instruction("mov r10, QWORD PTR [rbp - 48]");                       // reload class-name start after helper call
    emitter.instruction("mov r11, QWORD PTR [rbp - 56]");                       // reload class-name length after helper call
    emitter.instruction("mov rax, r10");                                        // class-name pointer (new_by_name arg)
    emitter.instruction("mov rdx, r11");                                        // class-name length (new_by_name arg)
    emitter.instruction("call __rt_new_by_name");                               // instantiate the class by name (0 on unknown class)
    emitter.instruction("test rax, rax");                                       // unknown class?
    emitter.instruction("jz __rt_unser_at_fail");                               // unknown class fails the parse
    emitter.instruction("jmp __rt_unser_obj_allocated_x");                      // skip incomplete-object allocation
    emitter.label("__rt_unser_obj_incomplete_x");
    emitter.instruction("mov rax, 32");                                         // class id, original class name, and opaque property hash
    emitter.instruction("call __rt_heap_alloc");                                // allocate the incomplete-object payload
    emitter.instruction(&format!(
        "mov r10, 0x{:x}",
        crate::codegen_support::sentinels::x86_64_heap_kind_word(4)
    )); // materialize the full-width object heap marker before storing it
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the object header without an unencodable imm64 memory move
    emitter.instruction("mov rdi, rax");                                        // object handle allocator input
    emitter.instruction("call __rt_object_handle_acquire");                     // give the incomplete object a normal PHP handle
    emitter.instruction("mov QWORD PTR [rax], -2");                             // reserved class id for __PHP_Incomplete_Class
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // preserve incomplete object across string persistence
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // original serialized class-name bytes
    emitter.instruction("mov rdx, QWORD PTR [rbp - 56]");                       // original serialized class-name length
    emitter.instruction("call __rt_str_persist");                               // own the class name independently of the source wire
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload incomplete object payload
    emitter.instruction("mov QWORD PTR [r10 + 8], rax");                        // persisted original class-name pointer
    emitter.instruction("mov QWORD PTR [r10 + 16], rdx");                       // persisted original class-name length
    emitter.instruction("mov QWORD PTR [r10 + 24], 0");                         // property hash is created after its count is parsed
    emitter.instruction("mov rax, r10");                                        // restore object pointer for the shared allocated path
    emitter.label("__rt_unser_obj_allocated_x");
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the new object pointer
    emitter.instruction("mov r10, QWORD PTR [rbp - 72]");                       // reload the class-name end
    emitter.instruction("add r10, 2");                                          // skip closing '\"' and ':' to the property count
    emitter.instruction("xor r11, r11");                                        // property-count accumulator
    emitter.label("__rt_unser_at_obj_count");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next count byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_at_obj_count_done");                     // ':' terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_at_obj_count_done");                     // ':' terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_at_obj_count");                         // continue
    emitter.label("__rt_unser_at_obj_count_done");
    emitter.instruction("mov QWORD PTR [rbp - 40], r11");                       // save the property count
    emitter.instruction("add r10, 2");                                          // skip ':' and '{' to the body
    emitter.instruction("sub r10, QWORD PTR [rbp - 8]");                        // body position offset
    emitter.instruction("mov QWORD PTR [rbp - 16], r10");                       // advance the cursor to the body
    emitter.instruction("cmp QWORD PTR [rbp - 80], 0");                         // blocked objects cannot inspect class hook tables
    emitter.instruction("je __rt_unser_obj_default");                           // parse properties without hydration
    // -- __unserialize magic: parse the body into an assoc array, then call
    //    __unserialize($this, $data) instead of injecting properties by name --
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // object pointer
    emitter.instruction("mov rax, QWORD PTR [r10]");                            // class id from the object header
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_class_unserialize_ptrs");
    emitter.instruction("mov r10, QWORD PTR [r11 + rax*8]");                    // __unserialize method symbol (0 if none)
    emitter.instruction("test r10, r10");                                       // does the class define __unserialize?
    emitter.instruction("jz __rt_unser_obj_default");                           // no → inject properties by name
    emitter.instruction("mov QWORD PTR [rbp - 72], r10");                       // park the __unserialize target
    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // entry count = hash capacity hint
    emitter.instruction("mov rsi, 7");                                          // hash value_type = boxed Mixed
    emitter.instruction("call __rt_hash_new");                                  // allocate the $data hash
    emitter.instruction("mov QWORD PTR [rbp - 80], rax");                       // save the $data hash pointer
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // entry index = 0
    emitter.label("__rt_unser_obj_data_loop");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the entry index
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 40]");                       // all entries parsed?
    emitter.instruction("jge __rt_unser_obj_data_done");                        // call __unserialize when done
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // base
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // current position
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // end
    emitter.instruction("call __rt_unser_key");                                 // parse the key -> rax=key_lo, rdx=key_hi, rcx=newpos
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // save key_lo
    emitter.instruction("mov QWORD PTR [rbp - 64], rdx");                       // save key_hi
    emitter.instruction("mov QWORD PTR [rbp - 16], rcx");                       // advance past the key
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // base
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // position after the key
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // end
    emitter.instruction("call __rt_unser_at");                                  // recursively parse the value -> rax=box, rdx=newpos
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // advance past the value
    emitter.instruction("mov rcx, rax");                                        // value_lo = parsed value box
    emitter.instruction("mov rdi, QWORD PTR [rbp - 80]");                       // $data hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // key_lo
    emitter.instruction("mov rdx, QWORD PTR [rbp - 64]");                       // key_hi (-1 for int keys)
    emitter.instruction("mov r8, 0");                                           // value_hi unused
    emitter.instruction("mov r9, 7");                                           // value tag = boxed Mixed (transfer the box)
    emitter.instruction("call __rt_hash_set");                                  // insert the entry -> rax = (possibly new) hash
    emitter.instruction("mov QWORD PTR [rbp - 80], rax");                       // save the updated $data hash pointer
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the entry index
    emitter.instruction("add rcx, 1");                                          // advance the entry index
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // persist the entry index
    emitter.instruction("jmp __rt_unser_obj_data_loop");                        // continue with the next entry
    emitter.label("__rt_unser_obj_data_done");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // $this receiver = first argument
    emitter.instruction("mov rsi, QWORD PTR [rbp - 80]");                       // $data assoc array (bare hash) = second argument
    emitter.instruction("mov r10, QWORD PTR [rbp - 72]");                       // reload the __unserialize target
    emitter.instruction("call r10");                                            // call __unserialize($this, $data)
    emitter.instruction("jmp __rt_unser_at_obj_box");                           // box the object (position is at the closing '}')
    emitter.label("__rt_unser_obj_default");
    emitter.instruction("cmp QWORD PTR [rbp - 80], 0");                         // blocked objects own an opaque Mixed property hash
    emitter.instruction("jne __rt_unser_obj_default_props_x");                  // hydrated objects use their declared property slots
    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // property count is the hash capacity hint
    emitter.instruction("mov rsi, 7");                                          // values are boxed Mixed cells
    emitter.instruction("call __rt_hash_new");                                  // allocate property hash before parsing values
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // incomplete-object payload
    emitter.instruction("mov QWORD PTR [r10 + 24], rax");                       // transfer hash ownership into incomplete object
    emitter.label("__rt_unser_obj_default_props_x");
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // initialize the property index
    emitter.label("__rt_unser_at_obj_loop");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the property index
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 40]");                       // all properties parsed?
    emitter.instruction("jge __rt_unser_at_obj_close");                         // box the object when done
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // base
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // current position
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // end
    emitter.instruction("call __rt_unser_key");                                 // parse the mangled key -> rax=key_ptr, rdx=key_len, rcx=newpos
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // save the key pointer
    emitter.instruction("mov QWORD PTR [rbp - 64], rdx");                       // save the key length
    emitter.instruction("mov QWORD PTR [rbp - 16], rcx");                       // advance past the key
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // base
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // position after the key
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // end
    emitter.instruction("call __rt_unser_at");                                  // recursively parse the value -> rax=box, rdx=newpos
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // advance past the value
    emitter.instruction("mov rcx, rax");                                        // value box
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // object pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // key pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 64]");                       // key length
    emitter.instruction("cmp QWORD PTR [rbp - 80], 0");                         // blocked objects keep their wire properties opaque
    emitter.instruction("je __rt_unser_obj_store_opaque_prop_x");               // blocked objects retain the parsed property semantically
    emitter.instruction("call __rt_obj_store_prop");                            // store the value into the matching property slot
    emitter.instruction("jmp __rt_unser_obj_skip_prop_store_x");                // transferred value now belongs to the hydrated object
    emitter.label("__rt_unser_obj_store_opaque_prop_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // incomplete-object payload
    emitter.instruction("mov rdi, QWORD PTR [rdi + 24]");                       // opaque property hash
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // serialized property key pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 64]");                       // serialized property key length
    emitter.instruction("mov r8, 0");                                           // boxed Mixed values have no high payload word
    emitter.instruction("mov r9, 7");                                           // transfer the parsed box as a Mixed hash value
    emitter.instruction("call __rt_hash_set");                                  // insert property, preserving key/value ownership and order
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // incomplete-object payload after a possible hash grow
    emitter.instruction("mov QWORD PTR [r10 + 24], rax");                       // retain updated property hash pointer
    emitter.label("__rt_unser_obj_skip_prop_store_x");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the property index
    emitter.instruction("add rcx, 1");                                          // advance the property index
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // persist the property index
    emitter.instruction("jmp __rt_unser_at_obj_loop");                          // continue with the next property
    emitter.label("__rt_unser_at_obj_close");
    // -- __wakeup magic: after default property injection, call __wakeup($this) --
    emitter.instruction("cmp QWORD PTR [rbp - 80], 0");                         // blocked classes cannot run __wakeup
    emitter.instruction("je __rt_unser_at_obj_box");                            // incomplete objects never run class hooks
    emitter.label("__rt_unser_obj_wakeup_x");
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // object pointer
    emitter.instruction("mov rax, QWORD PTR [r10]");                            // class id from the object header
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_class_wakeup_ptrs");
    emitter.instruction("mov r10, QWORD PTR [r11 + rax*8]");                    // __wakeup method symbol (0 if none)
    emitter.instruction("test r10, r10");                                       // does the class define __wakeup?
    emitter.instruction("jz __rt_unser_at_obj_box");                            // no → box the object directly
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // $this receiver
    emitter.instruction("call r10");                                            // call __wakeup($this)
    emitter.label("__rt_unser_at_obj_box");
    emitter.instruction("mov rax, 24");                                         // box the object: Mixed cell = tag + two payload words
    emitter.instruction("call __rt_heap_alloc");                                // allocate the boxed Mixed cell
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the object pointer
    emitter.instruction(&format!("mov r11, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(5))); // materialize the x86_64 boxed-Mixed heap kind word
    emitter.instruction("mov QWORD PTR [rax - 8], r11");                        // stamp the Mixed box without discarding the x86_64 heap marker
    emitter.instruction("mov QWORD PTR [rax], 6");                              // value tag 6 = object
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // store the object pointer (ownership transferred)
    emitter.instruction("mov QWORD PTR [rax + 16], 0");                         // clear the high payload word
    // -- register this object box so a later r:<index>; resolves to the same object --
    emitter.instruction("mov r10, QWORD PTR [rbp - 88]");                       // reserved value index for this object
    emitter.instruction("cmp r10, 65536");                                      // is the value registry full?
    emitter.instruction("jge __rt_unser_obj_box_noreg");                        // overflow → skip registration
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_unser_values");
    emitter.instruction("mov QWORD PTR [r11 + r10*8], rax");                    // values[index] = this object box
    emitter.label("__rt_unser_obj_box_noreg");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload position (at the closing '}')
    emitter.instruction("add rdx, 1");                                          // newpos skips the '}'
    emitter.instruction("jmp __rt_unser_at_ret");                               // return box and new position

    // -- failure: null box, position unchanged --
    emitter.label("__rt_unser_at_fail");
    emitter.instruction("xor eax, eax");                                        // null result signals parse failure
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // newpos = unchanged position

    emitter.label("__rt_unser_at_ret");
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r8", "_unser_depth", 0); // load parser depth before returning to the caller
    emitter.instruction("sub r8, 1");                                           // release this completed recursive parser frame
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r8", "_unser_depth", 0); // keep sibling parses independent
    emitter.instruction("add rsp, 96");                                         // deallocate the parser frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return rax=box, rdx=newpos

    emitter.label("__rt_unser_depth_fatal_x");
    emitter.instruction("mov edi, 2");                                          // stderr file descriptor
    crate::codegen_support::abi::emit_symbol_address(emitter, "rsi", "_unser_depth_msg");
    emitter.instruction("mov edx, 48");                                         // complete unserialize-depth fatal diagnostic length
    emitter.instruction("mov eax, 1");                                          // Linux write syscall number
    emitter.instruction("syscall");                                             // report the recursive parser limit
    emitter.instruction("mov edi, 1");                                          // non-zero failure status
    emitter.instruction("mov eax, 60");                                         // Linux exit syscall number
    emitter.instruction("syscall");                                             // terminate without returning to the overflowing caller

    // -- back-reference: r:N; / R:N; -> a fresh box aliasing the Nth parsed value
    //    (1-based); objects are retained. Out-of-range/unregistered index -> null. --
    emitter.label("__rt_unser_at_ref");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // base
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                       // position
    emitter.instruction("add r10, r11");                                        // pointer to the leading 'r'/'R'
    emitter.instruction("add r10, 2");                                          // skip the marker and ':'
    emitter.instruction("xor r11, r11");                                        // index accumulator
    emitter.label("__rt_unser_at_ref_loop");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_at_ref_done");                           // terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_at_ref_done");                           // terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift the accumulator
    emitter.instruction("add r11, r9");                                         // add the digit
    emitter.instruction("add r10, 1");                                          // advance the cursor
    emitter.instruction("jmp __rt_unser_at_ref_loop");                          // continue
    emitter.label("__rt_unser_at_ref_done");
    emitter.instruction("mov r9, QWORD PTR [rbp - 8]");                         // reload base
    emitter.instruction("sub r10, r9");                                         // offset of the ';'
    emitter.instruction("add r10, 1");                                          // newpos skips the ';'
    emitter.instruction("mov QWORD PTR [rbp - 16], r10");                       // save the new position
    emitter.instruction("test r11, r11");                                       // index 0 is invalid
    emitter.instruction("jz __rt_unser_at_ref_fail");                           // bail to null
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_unser_count");
    emitter.instruction("mov r9, QWORD PTR [r9]");                              // number of registered values
    emitter.instruction("cmp r11, r9");                                         // index beyond what was parsed?
    emitter.instruction("jg __rt_unser_at_ref_fail");                           // out of range → null
    emitter.instruction("sub r11, 1");                                          // 0-based registry slot
    emitter.instruction("cmp r11, 65536");                                      // would this logical reference index exceed the registry?
    emitter.instruction("jae __rt_unser_at_ref_fail");                          // fail closed instead of reading beyond the fixed registry
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_unser_values");
    emitter.instruction("mov r9, QWORD PTR [r9 + r11*8]");                      // the registered value box (0 if none)
    emitter.instruction("test r9, r9");                                         // nothing registered (e.g. a cycle)?
    emitter.instruction("jz __rt_unser_at_ref_fail");                           // → null
    emitter.instruction("mov QWORD PTR [rbp - 72], r9");                        // save the source box across the alloc
    emitter.instruction("mov rax, 24");                                         // a fresh boxed Mixed cell
    emitter.instruction("call __rt_heap_alloc");                                // allocate it
    emitter.instruction("mov r9, QWORD PTR [rbp - 72]");                        // reload the source box
    emitter.instruction("mov r10, QWORD PTR [r9 - 8]");                         // source heap header
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // copy the heap header
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // source value tag
    emitter.instruction("mov QWORD PTR [rax], r10");                            // copy the value tag
    emitter.instruction("mov r10, QWORD PTR [r9 + 8]");                         // source low payload (object pointer)
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // copy the low payload
    emitter.instruction("mov r10, QWORD PTR [r9 + 16]");                        // source high payload
    emitter.instruction("mov QWORD PTR [rax + 16], r10");                       // copy the high payload
    emitter.instruction("cmp QWORD PTR [rax], 6");                              // does the alias point at an object?
    emitter.instruction("jne __rt_unser_at_ref_boxed");                         // non-objects need no retain
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // save the fresh box across the retain
    emitter.instruction("mov rax, QWORD PTR [rax + 8]");                        // move the object pointer into incref's x86_64 input register
    emitter.instruction("call __rt_incref");                                    // retain the shared object before the source box releases it
    emitter.instruction("mov rax, QWORD PTR [rbp - 72]");                       // reload the fresh box
    emitter.label("__rt_unser_at_ref_boxed");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // newpos past the ';'
    emitter.instruction("jmp __rt_unser_at_ret");                               // return the aliasing box
    emitter.label("__rt_unser_at_ref_fail");
    emitter.instruction("xor eax, eax");                                        // unresolved reference → null
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // newpos past the ';'
    emitter.instruction("jmp __rt_unser_at_ret");                               // return the null result

    emit_unserialize_context_x86_64(emitter);

    // -- __rt_obj_store_prop(rdi=obj, rsi=key_ptr, rdx=key_len, rcx=valbox): inject a property --
    emitter.label_global("__rt_obj_store_prop");
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the store frame
    emitter.instruction("sub rsp, 64");                                         // reserve frame slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the object pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the key pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save the key length
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // save the value box
    emitter.instruction("mov rax, QWORD PTR [rdi]");                            // class id from the object header
    crate::codegen_support::abi::emit_symbol_address(emitter, "r10", "_class_serprop_ptrs");
    emitter.instruction("shl rax, 3");                                          // class_id * 8 (pointer stride)
    emitter.instruction("add r10, rax");                                        // slot = base + class_id*8
    emitter.instruction("mov r10, QWORD PTR [r10]");                            // property-info table for this class
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // save the property-info table
    emitter.instruction("mov rax, QWORD PTR [r10]");                            // property count
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // save the property count
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // row index = 0
    emitter.label("__rt_obj_store_prop_loop");
    emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                       // reload the row index
    emitter.instruction("cmp rax, QWORD PTR [rbp - 48]");                       // scanned every row?
    emitter.instruction("jge __rt_obj_store_prop_done");                        // unknown key is ignored
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // property-info table
    emitter.instruction("shl rax, 5");                                          // index * 32 (row stride)
    emitter.instruction("add rax, r10");                                        // table + index*32
    emitter.instruction("add rax, 8");                                          // skip the count word to the row
    emitter.instruction("mov QWORD PTR [rbp - 64], rax");                       // save the row pointer
    emitter.instruction("mov r9, QWORD PTR [rax]");                             // row mangled key pointer
    emitter.instruction("mov rdx, QWORD PTR [rax + 8]");                        // row mangled key length
    emitter.instruction("cmp rdx, QWORD PTR [rbp - 24]");                       // same length as the parsed key?
    emitter.instruction("jne __rt_obj_store_prop_next");                        // lengths differ, skip
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // parsed key pointer
    emitter.instruction("xor r8, r8");                                          // byte compare cursor
    emitter.label("__rt_obj_store_prop_cmp");
    emitter.instruction("cmp r8, rdx");                                         // compared all bytes?
    emitter.instruction("jge __rt_obj_store_prop_match");                       // full match
    emitter.instruction("mov al, BYTE PTR [r9 + r8]");                          // row key byte
    emitter.instruction("mov cl, BYTE PTR [rsi + r8]");                         // parsed key byte
    emitter.instruction("cmp al, cl");                                          // bytes equal?
    emitter.instruction("jne __rt_obj_store_prop_next");                        // mismatch, skip this row
    emitter.instruction("add r8, 1");                                           // next byte
    emitter.instruction("jmp __rt_obj_store_prop_cmp");                         // continue comparing
    emitter.label("__rt_obj_store_prop_match");
    emitter.instruction("mov rax, QWORD PTR [rbp - 64]");                       // reload the row pointer
    emitter.instruction("mov r8, QWORD PTR [rax + 16]");                        // property byte offset
    emitter.instruction("mov r9, QWORD PTR [rax + 24]");                        // property value tag
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // object pointer
    emitter.instruction("add r10, r8");                                         // address of the property slot
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // value box
    emitter.instruction("cmp r9, 7");                                           // is this a Mixed/untyped slot?
    emitter.instruction("je __rt_obj_store_prop_mixed");                        // store the boxed cell directly
    emitter.instruction("cmp r9, 1");                                           // is this a string slot?
    emitter.instruction("je __rt_obj_store_prop_str");                          // store pointer and length
    emitter.instruction("cmp r9, 4");                                           // is this an indexed-array slot?
    emitter.instruction("je __rt_obj_store_prop_arr");                          // convert the parsed hash to an indexed array
    emitter.instruction("mov rax, QWORD PTR [rcx + 8]");                        // typed scalar/object/hash: unbox the low word
    emitter.instruction("mov QWORD PTR [r10], rax");                            // store it inline in the slot
    emitter.instruction("jmp __rt_obj_store_prop_ret");                         // property stored
    emitter.label("__rt_obj_store_prop_arr");
    emitter.instruction("mov QWORD PTR [rbp - 64], r8");                        // save the property byte offset across the call
    emitter.instruction("mov rdi, QWORD PTR [rcx + 8]");                        // parsed hash pointer (box low word)
    emitter.instruction("call __rt_hash_to_indexed_array");                     // materialize a native indexed array
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // object pointer
    emitter.instruction("add r10, QWORD PTR [rbp - 64]");                       // slot = object + byte offset
    emitter.instruction("mov QWORD PTR [r10], rax");                            // store the indexed-array pointer
    emitter.instruction("jmp __rt_obj_store_prop_ret");                         // property stored
    emitter.label("__rt_obj_store_prop_str");
    emitter.instruction("mov rax, QWORD PTR [rcx + 8]");                        // string pointer from the box
    emitter.instruction("mov QWORD PTR [r10], rax");                            // store the string pointer
    emitter.instruction("mov rax, QWORD PTR [rcx + 16]");                       // string length from the box
    emitter.instruction("mov QWORD PTR [r10 + 8], rax");                        // store the string length
    emitter.instruction("jmp __rt_obj_store_prop_ret");                         // property stored
    emitter.label("__rt_obj_store_prop_mixed");
    emitter.instruction("mov rax, QWORD PTR [rcx]");                            // boxed value tag
    emitter.instruction("cmp rax, 8");                                          // is the boxed value null?
    emitter.instruction("je __rt_obj_store_prop_mixed_null");                   // store the null sentinel
    emitter.instruction("mov QWORD PTR [r10], rcx");                            // store the boxed Mixed cell pointer
    emitter.instruction("jmp __rt_obj_store_prop_ret");                         // property stored
    emitter.label("__rt_obj_store_prop_mixed_null");
    crate::codegen_support::abi::emit_load_int_immediate(emitter, "r11", crate::codegen_support::NULL_SENTINEL);
    emitter.instruction("mov QWORD PTR [r10], r11");                            // store the in-band null sentinel
    emitter.instruction("mov QWORD PTR [r10 + 8], 0");                          // clear the high word
    emitter.instruction("jmp __rt_obj_store_prop_ret");                         // property stored
    emitter.label("__rt_obj_store_prop_next");
    emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                       // reload the row index
    emitter.instruction("add rax, 1");                                          // advance to the next row
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // persist the row index
    emitter.instruction("jmp __rt_obj_store_prop_loop");                        // continue scanning
    emitter.label("__rt_obj_store_prop_done");
    emitter.label("__rt_obj_store_prop_ret");
    emitter.instruction("add rsp, 64");                                         // deallocate the store frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the caller

    // -- __rt_hash_to_indexed_array(rdi=hash) -> rax=indexed array: rebuild a parsed
    // hash (boxed-Mixed values) as a native value_type-7 indexed array. --
    emitter.label_global("__rt_hash_to_indexed_array");
    emitter.instruction("push rbp");                                            // open the conversion frame
    emitter.instruction("mov rbp, rsp");                                        // set the frame pointer
    emitter.instruction("sub rsp, 32");                                         // reserve callee-saved spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rbx");                        // save rbx
    emitter.instruction("mov QWORD PTR [rbp - 16], r12");                       // save r12
    emitter.instruction("mov QWORD PTR [rbp - 24], r13");                       // save r13
    emitter.instruction("mov rbx, rdi");                                        // hash pointer
    emitter.instruction("mov rdi, 0");                                          // initial capacity 0
    emitter.instruction("mov rsi, 8");                                          // 8-byte element slots
    emitter.instruction("call __rt_array_new");                                 // allocate an empty indexed array
    emitter.instruction("mov r12, rax");                                        // destination array pointer
    emitter.instruction("xor r13, r13");                                        // hash iteration cursor
    emitter.label("__rt_hash_to_indexed_array_loop");
    emitter.instruction("mov rdi, rbx");                                        // hash pointer
    emitter.instruction("mov rsi, r13");                                        // resume cursor
    emitter.instruction("call __rt_hash_iter_next");                            // rcx=value low, rax=next cursor
    emitter.instruction("cmp rax, -1");                                         // iteration done?
    emitter.instruction("je __rt_hash_to_indexed_array_done");                  // stop when exhausted
    emitter.instruction("mov r13, rax");                                        // save the resume cursor
    emitter.instruction("mov rdi, r12");                                        // destination array
    emitter.instruction("mov rsi, rcx");                                        // boxed-Mixed value pointer (parsed-hash value)
    emitter.instruction("call __rt_array_push_refcounted");                     // append, transferring ownership
    emitter.instruction("mov r12, rax");                                        // array may move on COW growth
    emitter.instruction("jmp __rt_hash_to_indexed_array_loop");                 // continue iterating
    emitter.label("__rt_hash_to_indexed_array_done");
    emitter.instruction("mov rax, r12");                                        // return the indexed array
    emitter.instruction("mov rbx, QWORD PTR [rbp - 8]");                        // restore rbx
    emitter.instruction("mov r12, QWORD PTR [rbp - 16]");                       // restore r12
    emitter.instruction("mov r13, QWORD PTR [rbp - 24]");                       // restore r13
    emitter.instruction("add rsp, 32");                                         // close the conversion frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the converted array

    emit_unser_key_x86_64(emitter);
}

/// Emits the x86_64 leaf key parser `__rt_unser_key`.
///
/// Input: `rdi`=base, `rsi`=pos, `rdx`=end. Output: `rax`=key_lo, `rdx`=key_hi (-1 for
/// an integer key, else the string byte length), `rcx`=newpos. String key pointers are
/// borrowed into the source buffer; `__rt_hash_set` persists them.
fn emit_unser_key_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unser_key (serialize() array key parser, leaf) ---");
    emitter.label_global("__rt_unser_key");
    emitter.instruction("cmp rsi, rdx");                                        // require a key type byte before loading it
    emitter.instruction("jae __rt_unser_key_fail_x");                           // return a sentinel cursor for a truncated key
    emitter.instruction("movzx r9d, BYTE PTR [rdi + rsi]");                     // load the key type byte
    emitter.instruction("cmp r9d, 105");                                        // ASCII 'i' (integer key)?
    emitter.instruction("je __rt_unser_key_int");                               // parse an integer key
    // -- string key: "s:" + bytelen + ":\"" + raw + "\";" --
    emitter.instruction("mov r10, rdi");                                        // base copy for cursor math
    emitter.instruction("add r10, rsi");                                        // pointer to the type byte
    emitter.instruction("add r10, 2");                                          // skip "s:" to the length digits
    emitter.instruction("xor r11, r11");                                        // length accumulator
    emitter.label("__rt_unser_key_strlen");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next length byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_key_strlen_done");                       // ':' terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_key_strlen_done");                       // ':' terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_key_strlen");                           // continue
    emitter.label("__rt_unser_key_strlen_done");
    emitter.instruction("add r10, 2");                                          // skip ':' and opening '\"' to the raw bytes
    emitter.instruction("mov r8, r10");                                         // raw end accumulator = raw start
    emitter.instruction("add r8, r11");                                         // raw end = raw + len
    emitter.instruction("add r8, 2");                                           // skip closing '\"' and ';'
    emitter.instruction("sub r8, rdi");                                         // newpos = (raw end + 2) - base
    emitter.instruction("mov rcx, r8");                                         // key newpos
    emitter.instruction("mov rdx, r11");                                        // key_hi = string byte length
    emitter.instruction("mov rax, r10");                                        // key_lo = borrowed raw string pointer
    emitter.instruction("ret");                                                 // return the string key
    // -- integer key: "i:" + optional '-' + digits + ";" --
    emitter.label("__rt_unser_key_int");
    emitter.instruction("mov r10, rdi");                                        // base copy for cursor math
    emitter.instruction("add r10, rsi");                                        // pointer to the type byte
    emitter.instruction("add r10, 2");                                          // skip "i:" to the first digit
    emitter.instruction("xor r11, r11");                                        // digit accumulator
    emitter.instruction("xor r8, r8");                                          // negative-sign flag
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // first numeric byte
    emitter.instruction("cmp r9d, 45");                                         // leading '-'?
    emitter.instruction("jne __rt_unser_key_int_loop");                         // no sign
    emitter.instruction("mov r8, 1");                                           // record negative sign
    emitter.instruction("add r10, 1");                                          // skip '-'
    emitter.label("__rt_unser_key_int_loop");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next numeric byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_key_int_done");                          // ';' terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_key_int_done");                          // ';' terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_key_int_loop");                         // continue
    emitter.label("__rt_unser_key_int_done");
    emitter.instruction("test r8, r8");                                         // signed?
    emitter.instruction("jz __rt_unser_key_int_pos");                           // not signed
    emitter.instruction("neg r11");                                             // apply sign
    emitter.label("__rt_unser_key_int_pos");
    emitter.instruction("mov rcx, r10");                                        // cursor copy
    emitter.instruction("sub rcx, rdi");                                        // newpos = cursor - base
    emitter.instruction("add rcx, 1");                                          // skip the ';'
    emitter.instruction("mov rax, r11");                                        // key_lo = integer key value
    emitter.instruction("mov rdx, -1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("ret");                                                 // return the integer key
    emitter.label("__rt_unser_key_fail_x");
    emitter.instruction("lea rcx, [rdx + 1]");                                  // end+1 is an impossible valid cursor
    emitter.instruction("xor eax, eax");                                        // clear key payload on failure
    emitter.instruction("xor edx, edx");                                        // clear key metadata on failure
    emitter.instruction("ret");                                                 // caller/preflight rejects the sentinel
}
