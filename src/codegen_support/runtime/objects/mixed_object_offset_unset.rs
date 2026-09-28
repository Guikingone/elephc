//! Purpose:
//! Emits `__rt_mixed_object_offset_unset`, which removes an offset from an object held in a
//! boxed `mixed` cell: `unset($o->bag[$k])` where `bag` is untyped or `mixed` and holds an
//! `ArrayAccess` object.
//!
//! Called from:
//! - `crate::codegen::lower_inst::offset_unset`'s object branch.
//!
//! Key details:
//! - Dispatch mirrors `__rt_mixed_array_get`. The runtime's own SPL containers are recognised
//!   by class id: `SplFixedArray` goes to `__rt_spl_fixed_offset_unset`, and
//!   `SplDoublyLinkedList`, `SplStack` and `SplQueue` to `__rt_spl_dll_offset_unset`. Any other
//!   class reaches its `ArrayAccess::offsetUnset` through the dense `_class_offsetunset_ptrs`
//!   table.
//! - A class with no `offsetUnset` entry (a plain object, `stdClass`) raises PHP's
//!   `Error("Cannot use object of type C as array")` through `__rt_throw_object_not_array`.
//! - The key arrives as a normalized hash key (`key_hi == -1` marks an integer) and is boxed
//!   into a `Mixed` offset. A PHP `offsetUnset()` BORROWS that box, so this frame frees it after
//!   the call. The SPL helpers CONSUME it, as they do for `__rt_mixed_array_get`, so their paths
//!   skip the release.
//! - The table is read twice, once to decide and once after boxing, because boxing allocates
//!   and no register survives the call to hold the resolved pointer.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Selector values stored in the frame between the dispatch and the shared boxing step.
const SELECT_SPL_FIXED: u64 = 1;
const SELECT_SPL_LIST: u64 = 2;
const SELECT_PHP: u64 = 3;

/// Emits `__rt_mixed_object_offset_unset` for the current target.
///
/// Inputs: the unboxed object (`x0` / `rdi`), the key's low word (`x1` / `rsi`) and high word
/// (`x2` / `rdx`). Returns nothing; it may throw.
pub fn emit_mixed_object_offset_unset(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_object_offset_unset_x86_64(emitter);
        return;
    }
    emit_mixed_object_offset_unset_aarch64(emitter);
}

/// Emits the ARM64 helper. Frame: `[sp]` receiver, `[sp, #8]` key_lo then the boxed offset,
/// `[sp, #16]` key_hi, `[sp, #24]` dispatch selector, `[sp, #32]` saved fp/lr.
fn emit_mixed_object_offset_unset_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_object_offset_unset ---");
    emitter.label_global("__rt_mixed_object_offset_unset");
    emitter.instruction("sub sp, sp, #48");                                     // reserve receiver, key, selector and frame linkage
    emitter.instruction("stp x29, x30, [sp, #32]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #32");                                    // establish a stable frame
    emitter.instruction("str x0, [sp, #0]");                                    // save the unboxed receiver
    emitter.instruction("str x1, [sp, #8]");                                    // save the normalized key low word
    emitter.instruction("str x2, [sp, #16]");                                   // save the normalized key high word
    emitter.instruction("ldr x11, [x0]");                                       // load the receiver's class id
    for (symbol, selector) in [
        ("_spl_fixed_array_class_id", SELECT_SPL_FIXED),
        ("_spl_dll_class_id", SELECT_SPL_LIST),
        ("_spl_stack_class_id", SELECT_SPL_LIST),
        ("_spl_queue_class_id", SELECT_SPL_LIST),
    ] {
        abi::emit_symbol_address(emitter, "x12", symbol);
        emitter.instruction("ldr x12, [x12]");                                  // load the runtime container's class id
        emitter.instruction(&format!("mov x9, #{selector}"));                   // select this container's own unset helper
        emitter.instruction("cmp x11, x12");                                    // is the receiver this runtime container?
        emitter.instruction("b.eq __rt_mixed_object_offset_unset_box");         // box the key, then dispatch by selector
    }
    emitter.instruction("tbnz x11, #63, __rt_mixed_object_offset_unset_not_indexable"); // synthetic negative ids cannot index metadata
    abi::emit_load_symbol_to_reg(emitter, "x12", "_class_iface_method_count", 0);
    emitter.instruction("cmp x11, x12");                                        // is the id within the dense table?
    emitter.instruction("b.hs __rt_mixed_object_offset_unset_not_indexable");   // out-of-range ids have no entry
    abi::emit_symbol_address(emitter, "x12", "_class_offsetunset_ptrs");
    emitter.instruction("ldr x12, [x12, x11, lsl #3]");                         // resolve the concrete or inherited offsetUnset
    emitter.instruction("cbz x12, __rt_mixed_object_offset_unset_not_indexable"); // 0 means the class is not ArrayAccess
    emitter.instruction(&format!("mov x9, #{SELECT_PHP}"));                     // select the PHP-class method call

    emitter.label("__rt_mixed_object_offset_unset_box");
    emitter.instruction("str x9, [sp, #24]");                                   // keep the selector across the boxing call
    emitter.instruction("ldr x11, [sp, #16]");                                  // reload the normalized key high word
    emitter.instruction("cmn x11, #1");                                         // does key_hi carry the integer-key sentinel?
    emitter.instruction("b.eq __rt_mixed_object_offset_unset_int_key");         // integer keys box as Mixed int
    emitter.instruction("mov x0, #1");                                          // tag = string for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key string pointer
    emitter.instruction("ldr x2, [sp, #16]");                                   // key string length
    emitter.instruction("b __rt_mixed_object_offset_unset_boxed");              // share the call after boxing
    emitter.label("__rt_mixed_object_offset_unset_int_key");
    emitter.instruction("mov x0, #0");                                          // tag = int for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key integer payload
    emitter.instruction("mov x2, #0");                                          // integer keys have no high payload
    emitter.label("__rt_mixed_object_offset_unset_boxed");
    emitter.instruction("bl __rt_mixed_from_value");                            // allocate the boxed ArrayAccess offset
    emitter.instruction("str x0, [sp, #8]");                                    // keep the box for the release after the call
    emitter.instruction("mov x1, x0");                                          // pass the boxed offset as argument 1
    emitter.instruction("ldr x0, [sp, #0]");                                    // pass the unboxed receiver as argument 0
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload the dispatch selector
    emitter.instruction(&format!("cmp x9, #{SELECT_SPL_FIXED}"));               // is this an SplFixedArray?
    emitter.instruction("b.eq __rt_mixed_object_offset_unset_spl_fixed");       // remove through the fixed-array helper
    emitter.instruction(&format!("cmp x9, #{SELECT_SPL_LIST}"));                // is this an SPL list?
    emitter.instruction("b.eq __rt_mixed_object_offset_unset_spl_list");        // remove through the shared list helper
    emitter.instruction("ldr x11, [x0]");                                       // reload the class id, clobbered across boxing
    abi::emit_symbol_address(emitter, "x12", "_class_offsetunset_ptrs");
    emitter.instruction("ldr x12, [x12, x11, lsl #3]");                         // re-resolve offsetUnset for the same class
    emitter.instruction("blr x12");                                             // remove through PHP's ArrayAccess::offsetUnset
    emitter.instruction("b __rt_mixed_object_offset_unset_release");            // free the boxed offset
    emitter.label("__rt_mixed_object_offset_unset_spl_fixed");
    emitter.instruction("bl __rt_spl_fixed_offset_unset");                      // SplFixedArray::offsetUnset, which consumes the box
    emitter.instruction("b __rt_mixed_object_offset_unset_done");               // nothing left to release
    emitter.label("__rt_mixed_object_offset_unset_spl_list");
    emitter.instruction("bl __rt_spl_dll_offset_unset");                        // SPL list offsetUnset, which consumes the box
    emitter.instruction("b __rt_mixed_object_offset_unset_done");               // nothing left to release
    emitter.label("__rt_mixed_object_offset_unset_release");
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the boxed offset
    emitter.instruction("bl __rt_decref_mixed");                                // a PHP method BORROWED it, so this frame frees it
    emitter.label("__rt_mixed_object_offset_unset_done");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return to the unset site

    emitter.label("__rt_mixed_object_offset_unset_not_indexable");
    emitter.instruction("ldr x0, [sp, #0]");                                    // pass the receiver so the Error names its class
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame before the tail-call
    emitter.instruction("b __rt_throw_object_not_array");                       // never returns
}

/// Emits the x86_64 helper. Frame: `[rbp - 8]` receiver, `[rbp - 16]` key_lo then the boxed
/// offset, `[rbp - 24]` key_hi, `[rbp - 32]` dispatch selector.
fn emit_mixed_object_offset_unset_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_object_offset_unset ---");
    emitter.label_global("__rt_mixed_object_offset_unset");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 32");                                         // reserve receiver, key and selector slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the unboxed receiver
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the normalized key low word
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save the normalized key high word
    emitter.instruction("mov r11, QWORD PTR [rdi]");                            // load the receiver's class id
    for (symbol, selector) in [
        ("_spl_fixed_array_class_id", SELECT_SPL_FIXED),
        ("_spl_dll_class_id", SELECT_SPL_LIST),
        ("_spl_stack_class_id", SELECT_SPL_LIST),
        ("_spl_queue_class_id", SELECT_SPL_LIST),
    ] {
        abi::emit_load_symbol_to_reg(emitter, "r12", symbol, 0);
        emitter.instruction(&format!("mov r9, {selector}"));                    // select this container's own unset helper
        emitter.instruction("cmp r11, r12");                                    // is the receiver this runtime container?
        emitter.instruction("je __rt_mixed_object_offset_unset_box");           // box the key, then dispatch by selector
    }
    emitter.instruction("test r11, r11");                                       // reject negative synthetic class ids
    emitter.instruction("js __rt_mixed_object_offset_unset_not_indexable");     // synthetic ids cannot index metadata
    emitter.instruction("cmp r11, QWORD PTR [rip + _class_iface_method_count]"); // is the id within the dense table?
    emitter.instruction("jae __rt_mixed_object_offset_unset_not_indexable");    // out-of-range ids have no entry
    emitter.instruction("lea r12, [rip + _class_offsetunset_ptrs]");            // dense ArrayAccess::offsetUnset table
    emitter.instruction("mov r12, QWORD PTR [r12 + r11 * 8]");                  // resolve the concrete or inherited offsetUnset
    emitter.instruction("test r12, r12");                                       // 0 means the class is not ArrayAccess
    emitter.instruction("jz __rt_mixed_object_offset_unset_not_indexable");     // raise PHP's object-as-array Error
    emitter.instruction(&format!("mov r9, {SELECT_PHP}"));                      // select the PHP-class method call

    emitter.label("__rt_mixed_object_offset_unset_box");
    emitter.instruction("mov QWORD PTR [rbp - 32], r9");                        // keep the selector across the boxing call
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // reload the normalized key high word
    emitter.instruction("cmp r11, -1");                                         // does key_hi carry the integer-key sentinel?
    emitter.instruction("je __rt_mixed_object_offset_unset_int_key");           // integer keys box as Mixed int
    emitter.instruction("mov rax, 1");                                          // tag = string for mixed_from_value
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // key string pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // key string length
    emitter.instruction("jmp __rt_mixed_object_offset_unset_boxed");            // share the call after boxing
    emitter.label("__rt_mixed_object_offset_unset_int_key");
    emitter.instruction("mov rax, 0");                                          // tag = int for mixed_from_value
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // key integer payload
    emitter.instruction("xor esi, esi");                                        // integer keys have no high payload
    emitter.label("__rt_mixed_object_offset_unset_boxed");
    emitter.instruction("call __rt_mixed_from_value");                          // allocate the boxed ArrayAccess offset
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // keep the box for the release after the call
    emitter.instruction("mov rsi, rax");                                        // pass the boxed offset as argument 1
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the unboxed receiver as argument 0
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // reload the dispatch selector
    emitter.instruction(&format!("cmp r9, {SELECT_SPL_FIXED}"));                // is this an SplFixedArray?
    emitter.instruction("je __rt_mixed_object_offset_unset_spl_fixed");         // remove through the fixed-array helper
    emitter.instruction(&format!("cmp r9, {SELECT_SPL_LIST}"));                 // is this an SPL list?
    emitter.instruction("je __rt_mixed_object_offset_unset_spl_list");          // remove through the shared list helper
    emitter.instruction("mov r11, QWORD PTR [rdi]");                            // reload the class id, clobbered across boxing
    emitter.instruction("lea r12, [rip + _class_offsetunset_ptrs]");            // dense ArrayAccess::offsetUnset table
    emitter.instruction("mov r12, QWORD PTR [r12 + r11 * 8]");                  // re-resolve offsetUnset for the same class
    emitter.instruction("call r12");                                            // remove through PHP's ArrayAccess::offsetUnset
    emitter.instruction("jmp __rt_mixed_object_offset_unset_release");          // free the boxed offset
    emitter.label("__rt_mixed_object_offset_unset_spl_fixed");
    emitter.instruction("call __rt_spl_fixed_offset_unset");                    // SplFixedArray::offsetUnset, which consumes the box
    emitter.instruction("jmp __rt_mixed_object_offset_unset_done");             // nothing left to release
    emitter.label("__rt_mixed_object_offset_unset_spl_list");
    emitter.instruction("call __rt_spl_dll_offset_unset");                      // SPL list offsetUnset, which consumes the box
    emitter.instruction("jmp __rt_mixed_object_offset_unset_done");             // nothing left to release
    emitter.label("__rt_mixed_object_offset_unset_release");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the boxed offset
    emitter.instruction("call __rt_decref_mixed");                              // a PHP method BORROWED it, so this frame frees it
    emitter.label("__rt_mixed_object_offset_unset_done");
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the unset site

    emitter.label("__rt_mixed_object_offset_unset_not_indexable");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the receiver so the Error names its class
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before the tail-call
    emitter.instruction("jmp __rt_throw_object_not_array");                     // never returns
}
