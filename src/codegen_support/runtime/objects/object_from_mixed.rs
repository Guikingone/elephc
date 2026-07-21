//! Purpose:
//! Emits the `__rt_object_from_mixed` runtime dispatcher for the PHP `(object)` cast.
//! Inspects the boxed source tag and builds the matching `stdClass` result.
//!
//! Called from:
//! - `crate::codegen_support::runtime::objects` via the top-level runtime emitter.
//! - The EIR `object_cast` lowering, which boxes any non-object source first.
//!
//! Key details:
//! - null/empty -> empty stdClass; object -> retained passthrough (same instance);
//!   assoc array -> `__rt_object_from_hash`; indexed array -> promote to a Mixed hash
//!   then `__rt_object_from_hash`; scalar -> stdClass with one `scalar` property.

use crate::codegen::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits `__rt_object_from_mixed(mixed_ptr) -> obj_ptr` for the active target.
///
/// Dispatches on the boxed runtime tag to produce an owned `stdClass`: PHP null
/// becomes an empty object, an object payload is returned (retained) unchanged,
/// arrays build a property map, and scalars are wrapped in a single `scalar`
/// property. The boxed Mixed argument is only borrowed.
pub fn emit_object_from_mixed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_object_from_mixed_x86_64(emitter);
        return;
    }
    emit_object_from_mixed_aarch64(emitter);
}

/// ARM64 implementation of `__rt_object_from_mixed` (input `x0`, result `x0`).
fn emit_object_from_mixed_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: object_from_mixed ---");
    emitter.label_global("__rt_object_from_mixed");

    // Frame: [sp,#0]=mixed [sp,#8]=temp/obj [sp,#16]=temp hash
    //        [sp,#32]=fp [sp,#40]=lr
    emitter.instruction("sub sp, sp, #48");                                     // reserve the dispatch frame and saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // set the new frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the boxed Mixed pointer
    emitter.instruction("cbz x0, __rt_object_from_mixed_null");                 // a null pointer yields an empty stdClass
    emitter.instruction("ldr x9, [x0]");                                        // load the runtime tag from mixed[0]
    emitter.instruction("cmp x9, #8");                                          // is the payload PHP null?
    emitter.instruction("b.eq __rt_object_from_mixed_null");                    // null casts to an empty stdClass
    emitter.instruction("cmp x9, #6");                                          // is the payload already an object?
    emitter.instruction("b.eq __rt_object_from_mixed_object");                  // objects pass through as the same instance
    emitter.instruction("cmp x9, #5");                                          // is the payload an associative array?
    emitter.instruction("b.eq __rt_object_from_mixed_hash");                    // assoc arrays build properties from entries
    emitter.instruction("cmp x9, #4");                                          // is the payload an indexed array?
    emitter.instruction("b.eq __rt_object_from_mixed_array");                   // indexed arrays are promoted to a hash first
    emitter.instruction("b __rt_object_from_mixed_scalar");                     // scalars wrap in a single `scalar` property
    emitter.label("__rt_object_from_mixed_null");
    emitter.instruction("bl __rt_stdclass_new");                                // allocate an empty stdClass
    emitter.instruction("b __rt_object_from_mixed_ret");                        // return the empty object
    emitter.label("__rt_object_from_mixed_object");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the boxed Mixed pointer
    emitter.instruction("ldr x0, [x0, #8]");                                    // load the object pointer from mixed[8]
    emitter.instruction("bl __rt_incref");                                      // retain it so the cast result owns a reference
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the boxed Mixed pointer
    emitter.instruction("ldr x0, [x0, #8]");                                    // reload the object pointer for the return value
    emitter.instruction("b __rt_object_from_mixed_ret");                        // return the same object instance
    emitter.label("__rt_object_from_mixed_hash");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the boxed Mixed pointer
    emitter.instruction("ldr x0, [x0, #8]");                                    // load the hash pointer from mixed[8]
    emitter.instruction("bl __rt_object_from_hash");                            // build a stdClass from the hash entries
    emitter.instruction("b __rt_object_from_mixed_ret");                        // return the populated object
    emitter.label("__rt_object_from_mixed_array");
    emitter.instruction("mov x0, #16");                                         // initial temporary-hash capacity
    emitter.instruction("mov x1, #7");                                          // value_type 7 = boxed Mixed entries
    emitter.instruction("bl __rt_hash_new");                                    // x0 = empty temporary hash
    emitter.instruction("str x0, [sp, #16]");                                   // remember the empty hash so it can be freed
    emitter.instruction("mov x1, x0");                                          // pass the empty hash as the union right operand
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the boxed Mixed pointer
    emitter.instruction("ldr x0, [x9, #8]");                                    // x0 = indexed array (union left operand)
    emitter.instruction("bl __rt_array_hash_union");                            // x0 = fresh hash holding the array elements
    emitter.instruction("str x0, [sp, #8]");                                    // remember the converted hash
    emitter.instruction("ldr x0, [sp, #16]");                                   // reload the empty temporary hash
    emitter.instruction("bl __rt_decref_hash");                                 // release the empty hash consumed by the union
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the converted hash
    emitter.instruction("bl __rt_hash_to_mixed");                               // ensure every entry is a boxed Mixed cell
    emitter.instruction("str x0, [sp, #16]");                                   // remember the normalized converted hash
    emitter.instruction("bl __rt_object_from_hash");                            // build a stdClass from the converted hash
    emitter.instruction("str x0, [sp, #8]");                                    // stash the result object
    emitter.instruction("ldr x0, [sp, #16]");                                   // reload the converted hash
    emitter.instruction("bl __rt_decref_hash");                                 // release the temporary converted hash
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the result object
    emitter.instruction("b __rt_object_from_mixed_ret");                        // return the populated object
    emitter.label("__rt_object_from_mixed_scalar");
    emitter.instruction("bl __rt_stdclass_new");                                // x0 = fresh empty stdClass
    emitter.instruction("str x0, [sp, #8]");                                    // save the new object
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the boxed scalar Mixed pointer
    emitter.instruction("bl __rt_incref");                                      // retain it for the `scalar` property owner
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the object (stdclass_set arg 0)
    abi::emit_symbol_address(emitter, "x1", "_objcast_scalar_prop");
    emitter.instruction("mov x2, #6");                                          // property name length of "scalar"
    emitter.instruction("ldr x3, [sp, #0]");                                    // the boxed scalar Mixed pointer (property value)
    emitter.instruction("bl __rt_stdclass_set");                                // set $obj->scalar = the boxed scalar value
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the object for the return value
    emitter.label("__rt_object_from_mixed_ret");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the stdClass pointer in x0
}

/// x86_64 implementation of `__rt_object_from_mixed` (input `rdi`, result `rax`).
fn emit_object_from_mixed_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: object_from_mixed ---");
    emitter.label_global("__rt_object_from_mixed");

    // Frame: [rbp-8]=mixed [rbp-16]=temp hash [rbp-24]=obj
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 32");                                         // reserve the dispatch spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the boxed Mixed pointer
    emitter.instruction("test rdi, rdi");                                       // is the boxed Mixed pointer null?
    emitter.instruction("je __rt_object_from_mixed_null");                      // a null pointer yields an empty stdClass
    emitter.instruction("mov r10, QWORD PTR [rdi]");                            // load the runtime tag from mixed[0]
    emitter.instruction("cmp r10, 8");                                          // is the payload PHP null?
    emitter.instruction("je __rt_object_from_mixed_null");                      // null casts to an empty stdClass
    emitter.instruction("cmp r10, 6");                                          // is the payload already an object?
    emitter.instruction("je __rt_object_from_mixed_object");                    // objects pass through as the same instance
    emitter.instruction("cmp r10, 5");                                          // is the payload an associative array?
    emitter.instruction("je __rt_object_from_mixed_hash");                      // assoc arrays build properties from entries
    emitter.instruction("cmp r10, 4");                                          // is the payload an indexed array?
    emitter.instruction("je __rt_object_from_mixed_array");                     // indexed arrays are promoted to a hash first
    emitter.instruction("jmp __rt_object_from_mixed_scalar");                   // scalars wrap in a `scalar` property
    emitter.label("__rt_object_from_mixed_null");
    emitter.instruction("call __rt_stdclass_new");                              // allocate an empty stdClass
    emitter.instruction("jmp __rt_object_from_mixed_ret");                      // return the empty object
    emitter.label("__rt_object_from_mixed_object");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the boxed Mixed pointer
    emitter.instruction("mov rax, QWORD PTR [rax + 8]");                        // load the object pointer from mixed[8]
    emitter.instruction("call __rt_incref");                                    // retain it so the cast result owns a reference
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the boxed Mixed pointer
    emitter.instruction("mov rax, QWORD PTR [rax + 8]");                        // reload the object pointer for the return value
    emitter.instruction("jmp __rt_object_from_mixed_ret");                      // return the same object instance
    emitter.label("__rt_object_from_mixed_hash");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the boxed Mixed pointer
    emitter.instruction("mov rdi, QWORD PTR [rax + 8]");                        // load the hash pointer (from_hash first arg)
    emitter.instruction("call __rt_object_from_hash");                          // build a stdClass from the hash entries
    emitter.instruction("jmp __rt_object_from_mixed_ret");                      // return the populated object
    emitter.label("__rt_object_from_mixed_array");
    emitter.instruction("mov rax, 16");                                         // initial temporary-hash capacity
    emitter.instruction("mov rdi, 7");                                          // value_type 7 = boxed Mixed entries
    emitter.instruction("call __rt_hash_new");                                  // rax = empty temporary hash
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // remember the empty hash so it can be freed
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the boxed Mixed pointer
    emitter.instruction("mov rdi, QWORD PTR [rax + 8]");                        // rdi = indexed array (union left operand)
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // rsi = empty hash (union right operand)
    emitter.instruction("call __rt_array_hash_union");                          // rax = fresh hash holding the array elements
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // remember the converted hash
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the empty temporary hash
    emitter.instruction("call __rt_decref_hash");                               // release the empty hash consumed by the union
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // reload the converted hash
    emitter.instruction("call __rt_hash_to_mixed");                             // ensure every entry is a boxed Mixed cell
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // remember the normalized converted hash
    emitter.instruction("mov rdi, rax");                                        // from_hash takes the hash in the first arg
    emitter.instruction("call __rt_object_from_hash");                          // build a stdClass from the converted hash
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // stash the result object
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // reload the converted hash
    emitter.instruction("call __rt_decref_hash");                               // release the temporary converted hash
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the result object
    emitter.instruction("jmp __rt_object_from_mixed_ret");                      // return the populated object
    emitter.label("__rt_object_from_mixed_scalar");
    emitter.instruction("call __rt_stdclass_new");                              // rax = fresh empty stdClass
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the new object
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the boxed scalar Mixed pointer
    emitter.instruction("call __rt_incref");                                    // retain it for the `scalar` property owner
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the object (stdclass_set arg 0)
    abi::emit_symbol_address(emitter, "rsi", "_objcast_scalar_prop");
    emitter.instruction("mov rdx, 6");                                          // property name length of "scalar"
    emitter.instruction("mov rcx, QWORD PTR [rbp - 8]");                        // the boxed scalar Mixed pointer (value)
    emitter.instruction("call __rt_stdclass_set");                              // set $obj->scalar = the boxed scalar value
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the object for the return value
    emitter.label("__rt_object_from_mixed_ret");
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the stdClass pointer in rax
}
