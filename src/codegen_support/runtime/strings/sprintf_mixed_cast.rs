//! Purpose:
//! Emits the printf-family coercion helpers for deferred non-scalar operands.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via
//!   `crate::codegen_support::runtime::strings`.
//! - `__rt_sprintf` after a packed argument record preserves either a boxed `Mixed`
//!   value or a statically typed non-scalar payload.
//!
//! Key details:
//! - Numeric formatting follows PHP's non-scalar cast rules: empty arrays become 0,
//!   non-empty arrays, objects, and callable descriptors become 1, and resources use
//!   their public resource id rather than their native handle.
//! - String formatting returns `"Array"`, resource display text, or a dynamic
//!   `__toString()` result, including eval-declared classes when an eval context is active.
//!   A null result pair tells `__rt_sprintf` to take its controlled object-to-string fatal
//!   path; raw heap pointers never become numeric output, and owned results are returned
//!   separately so the formatter can release them after copying.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits the deferred non-scalar coercion helpers used by `__rt_sprintf`.
pub fn emit_sprintf_mixed_casts(emitter: &mut Emitter, eval_bridge: bool) {
    if emitter.target.arch == Arch::X86_64 {
        emit_sprintf_mixed_casts_linux_x86_64(emitter, eval_bridge);
        return;
    }

    emit_sprintf_mixed_to_int_aarch64(emitter);
    emit_sprintf_mixed_to_string_aarch64(emitter, eval_bridge);
}

/// Emits AArch64 `__rt_sprintf_mixed_to_int(tag, payload) -> int`.
fn emit_sprintf_mixed_to_int_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf mixed non-scalar to int ---");
    emitter.label_global("__rt_sprintf_mixed_to_int");

    emitter.instruction("sub sp, sp, #32");                                     // reserve an aligned helper frame for nested runtime calls
    emitter.instruction("stp x29, x30, [sp, #16]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #16");                                    // establish this helper's frame pointer
    emitter.instruction("cmp x0, #7");                                          // is the record payload a boxed Mixed cell?
    emitter.instruction("b.eq __rt_sprintf_mixed_int_unbox");                   // inspect the concrete boxed tag
    emitter.instruction("cmp x0, #11");                                         // type-erased Iterable record?
    emitter.instruction("b.eq __rt_sprintf_mixed_int_iterable");                // classify it by heap kind
    emitter.instruction("b __rt_sprintf_mixed_int_dispatch");                   // other raw tags are already concrete
    emitter.label("__rt_sprintf_mixed_int_unbox");
    emitter.instruction("mov x0, x1");                                          // pass the boxed payload to mixed_unbox
    emitter.instruction("bl __rt_mixed_unbox");                                 // unwrap nested Mixed cells before inspecting the concrete tag
    emitter.instruction("b __rt_sprintf_mixed_int_dispatch");
    emitter.label("__rt_sprintf_mixed_int_iterable");
    emitter.instruction("str x1, [sp, #0]");                                    // preserve erased payload across heap classification
    emitter.instruction("mov x0, x1");
    emitter.instruction("bl __rt_heap_kind");
    emitter.instruction("ldr x1, [sp, #0]");
    emitter.instruction("cmp x0, #2");
    emitter.instruction("b.eq __rt_sprintf_mixed_int_array");
    emitter.instruction("cmp x0, #3");
    emitter.instruction("b.eq __rt_sprintf_mixed_int_array");
    emitter.instruction("cmp x0, #4");
    emitter.instruction("b.eq __rt_sprintf_mixed_int_one");
    emitter.instruction("cmp x0, #6");
    emitter.instruction("b.eq __rt_sprintf_mixed_int_one");
    emitter.instruction("b __rt_sprintf_mixed_int_zero");
    emitter.label("__rt_sprintf_mixed_int_dispatch");
    emitter.instruction("cmp x0, #4");                                          // indexed array?
    emitter.instruction("b.eq __rt_sprintf_mixed_int_array");                   // arrays cast to zero or one by emptiness
    emitter.instruction("cmp x0, #5");                                          // associative array?
    emitter.instruction("b.eq __rt_sprintf_mixed_int_array");                   // hashes share PHP's array numeric cast
    emitter.instruction("cmp x0, #6");                                          // object?
    emitter.instruction("b.eq __rt_sprintf_mixed_int_one");                     // every object casts to integer one
    emitter.instruction("cmp x0, #9");                                          // resource?
    emitter.instruction("b.eq __rt_sprintf_mixed_int_resource");                // resources format through their public resource id
    emitter.instruction("cmp x0, #10");                                         // callable descriptor (Closure shape)?
    emitter.instruction("b.eq __rt_sprintf_mixed_int_one");                     // callable objects cast to integer one
    emitter.instruction("mov x0, #0");                                          // unknown non-scalars normalize to zero, never their raw payload
    emitter.instruction("b __rt_sprintf_mixed_int_done");                       // restore the helper frame and return

    emitter.label("__rt_sprintf_mixed_int_array");
    emitter.instruction("cbz x1, __rt_sprintf_mixed_int_zero");                 // null container pointers behave like empty arrays
    emitter.instruction("ldr x9, [x1]");                                        // read the live element count from the container header
    emitter.instruction("cmp x9, #0");                                          // is the array empty?
    emitter.instruction("cset x0, ne");                                         // PHP casts empty arrays to 0 and non-empty arrays to 1
    emitter.instruction("b __rt_sprintf_mixed_int_done");                       // return the normalized boolean-sized integer

    emitter.label("__rt_sprintf_mixed_int_one");
    emitter.instruction("mov x0, #1");                                          // object and callable numeric conversion result
    emitter.instruction("b __rt_sprintf_mixed_int_done");                       // share the frame teardown

    emitter.label("__rt_sprintf_mixed_int_resource");
    emitter.instruction("mov x0, x1");                                          // pass the native resource payload to the id registry
    emitter.instruction("bl __rt_resource_id_of");                              // return PHP's stable resource id instead of a handle/pointer
    emitter.instruction("b __rt_sprintf_mixed_int_done");                       // share the frame teardown

    emitter.label("__rt_sprintf_mixed_int_zero");
    emitter.instruction("mov x0, #0");                                          // empty/null container numeric conversion result
    emitter.label("__rt_sprintf_mixed_int_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore the caller frame and return address
    emitter.instruction("add sp, sp, #32");                                     // release this helper's aligned frame
    emitter.instruction("ret");                                                 // return the normalized integer in x0
}

/// Emits AArch64 `__rt_sprintf_mixed_to_string(tag, payload, eval_ctx) -> (owner, ptr, len)`.
fn emit_sprintf_mixed_to_string_aarch64(emitter: &mut Emitter, eval_bridge: bool) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf mixed non-scalar to string ---");
    emitter.label_global("__rt_sprintf_mixed_to_string");

    // Locals: tag/payload/eval-context at 0/8/16, output ptr/len/owner at 24/32/40,
    // boxed eval input at 48, ownership flag at 56, eval result struct at 64..87.
    emitter.instruction("sub sp, sp, #112");                                    // reserve spills and an eval result structure
    emitter.instruction("stp x29, x30, [sp, #96]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #96");                                    // establish this helper's frame pointer
    emitter.instruction("stp x0, x1, [sp, #0]");                               // save record tag and payload
    emitter.instruction("str x2, [sp, #16]");                                   // save the optional eval context
    emitter.instruction("stp xzr, xzr, [sp, #48]");                            // no eval input box and no ownership yet
    emitter.instruction("cmp x0, #7");                                          // boxed Mixed record?
    emitter.instruction("b.ne __rt_sprintf_mixed_string_raw");                  // raw tags are already concrete
    emitter.instruction("str x1, [sp, #48]");                                   // preserve the borrowed box for eval fallback
    emitter.instruction("mov x0, x1");                                          // mixed_unbox consumes the box in x0
    emitter.instruction("bl __rt_mixed_unbox");                                 // expose concrete tag and payload
    emitter.instruction("b __rt_sprintf_mixed_string_dispatch");                // apply concrete semantics
    emitter.label("__rt_sprintf_mixed_string_raw");
    emitter.instruction("cmp x0, #11");                                         // type-erased Iterable record?
    emitter.instruction("b.ne __rt_sprintf_mixed_string_dispatch");             // other raw tags are concrete
    emitter.instruction("mov x0, x1");                                          // classify the erased payload by heap kind
    emitter.instruction("bl __rt_heap_kind");                                   // 2/3 arrays, 4/6 objects
    emitter.instruction("ldr x1, [sp, #8]");                                    // restore the erased payload
    emitter.instruction("cmp x0, #2");
    emitter.instruction("b.eq __rt_sprintf_mixed_string_array");
    emitter.instruction("cmp x0, #3");
    emitter.instruction("b.eq __rt_sprintf_mixed_string_array");
    emitter.instruction("cmp x0, #4");
    emitter.instruction("b.eq __rt_sprintf_mixed_string_object");
    emitter.instruction("cmp x0, #6");
    emitter.instruction("b.eq __rt_sprintf_mixed_string_object");
    emitter.instruction("b __rt_sprintf_mixed_string_missing");
    emitter.label("__rt_sprintf_mixed_string_dispatch");
    emitter.instruction("cmp x0, #4");                                          // indexed array?
    emitter.instruction("b.eq __rt_sprintf_mixed_string_array");                // arrays stringify to the literal "Array"
    emitter.instruction("cmp x0, #5");                                          // associative array?
    emitter.instruction("b.eq __rt_sprintf_mixed_string_array");                // hashes use the same PHP placeholder
    emitter.instruction("cmp x0, #6");                                          // object?
    emitter.instruction("b.eq __rt_sprintf_mixed_string_object");               // dispatch dynamically through the class __toString table
    emitter.instruction("cmp x0, #9");                                          // resource?
    emitter.instruction("b.eq __rt_sprintf_mixed_string_resource");             // resources use the public Resource id #N rendering
    emitter.instruction("b __rt_sprintf_mixed_string_missing");                 // callable/unknown values cannot be converted to string

    emitter.label("__rt_sprintf_mixed_string_array");
    abi::emit_symbol_address(emitter, "x1", "_iterable_array_str");
    emitter.instruction("mov x2, #5");                                          // byte length of the literal "Array"
    emitter.instruction("mov x0, #0");                                          // fixed data is borrowed
    emitter.instruction("b __rt_sprintf_mixed_string_done");                    // return the borrowed fixed-data string pair

    emitter.label("__rt_sprintf_mixed_string_resource");
    emitter.instruction("mov x0, x1");                                          // pass the native resource payload to the display helper
    emitter.instruction("bl __rt_resource_to_string");                          // return borrowed Resource id #N text in x1/x2
    emitter.instruction("mov x0, #0");                                          // resource display storage is borrowed
    emitter.instruction("b __rt_sprintf_mixed_string_done");                    // share the frame teardown

    emitter.label("__rt_sprintf_mixed_string_object");
    emitter.instruction("str x1, [sp, #8]");                                    // preserve the receiver for eval fallback
    emitter.instruction("mov x0, x1");                                          // keep the borrowed object as the method receiver
    emitter.instruction("ldr x11, [x0]");                                       // load the object's dense runtime class id
    emitter.instruction("tbnz x11, #63, __rt_sprintf_mixed_string_eval");       // synthetic negative ids belong to eval
    abi::emit_load_symbol_to_reg(emitter, "x10", "_class_tostring_count", 0);
    emitter.instruction("cmp x11, x10");                                        // is the class id inside the generated table?
    emitter.instruction("b.hs __rt_sprintf_mixed_string_eval");                 // out-of-range classes may belong to eval
    abi::emit_symbol_address(emitter, "x10", "_class_tostring_ptrs");
    emitter.instruction("ldr x10, [x10, x11, lsl #3]");                         // resolve the concrete or inherited __toString symbol
    emitter.instruction("cbz x10, __rt_sprintf_mixed_string_eval");             // a missing native hook may still be eval-backed
    emitter.instruction("blr x10");                                             // call __toString with the borrowed receiver in x0
    emitter.instruction("b __rt_sprintf_mixed_string_own");                     // stabilize and own the method result

    emitter.label("__rt_sprintf_mixed_string_eval");
    if eval_bridge {
        emitter.instruction("ldr x0, [sp, #16]");                               // persistent eval context
        emitter.instruction("cbz x0, __rt_sprintf_mixed_string_missing");       // no dynamic method table without it
        emitter.instruction("mov x0, #6");                                      // runtime object tag
        emitter.instruction("ldr x1, [sp, #8]");                                // borrowed raw receiver
        emitter.instruction("mov x2, #0");                                      // objects have no high payload
        emitter.instruction("bl __rt_mixed_from_value");                        // retain receiver in a temporary box
        emitter.instruction("str x0, [sp, #48]");                               // save the owned box
        emitter.instruction("mov x9, x0");                                      // bridge object argument
        emitter.instruction("mov x10, #1");
        emitter.instruction("str x10, [sp, #56]");                              // remember that the helper owns it
        emitter.label("__rt_sprintf_mixed_string_eval_call");
        emitter.instruction("stp xzr, xzr, [sp, #64]");                         // clear kind/padding/value
        emitter.instruction("str xzr, [sp, #80]");                              // clear throwable pointer
        emitter.instruction("ldr x0, [sp, #16]");                               // context ABI arg 0
        emitter.instruction("mov x1, x9");                                      // boxed value ABI arg 1
        emitter.instruction("add x2, sp, #64");                                 // result ABI arg 2
        let symbol = emitter.target.extern_symbol("__elephc_eval_string_context");
        abi::emit_call_label(emitter, &symbol);
        emitter.instruction("str x0, [sp, #0]");                                // preserve bridge status during input cleanup
        emitter.instruction("ldr x9, [sp, #56]");
        emitter.instruction("cbz x9, __rt_sprintf_mixed_string_eval_status");
        emitter.instruction("ldr x0, [sp, #48]");                               // owned temporary input box
        emitter.instruction("bl __rt_decref_any");                              // balance its retained receiver
        emitter.label("__rt_sprintf_mixed_string_eval_status");
        emitter.instruction("ldr x0, [sp, #0]");                                // bridge status
        emitter.instruction("cbz x0, __rt_sprintf_mixed_string_eval_ok");
        emitter.instruction("cmp x0, #3");                                      // uncaught Throwable?
        emitter.instruction("b.eq __rt_sprintf_mixed_string_eval_throw");
        emitter.instruction("b __rt_sprintf_mixed_string_missing");
        emitter.label("__rt_sprintf_mixed_string_eval_ok");
        emitter.instruction("ldr x0, [sp, #72]");                               // boxed string result
        emitter.instruction("str x0, [sp, #48]");                               // preserve it through persistence
        emitter.instruction("bl __rt_mixed_unbox");
        emitter.instruction("cmp x0, #1");                                      // bridge must return a string cell
        emitter.instruction("b.ne __rt_sprintf_mixed_string_eval_bad");
        emitter.instruction("stp x1, x2, [sp, #24]");                           // source pair for persistence
        emitter.instruction("bl __rt_str_persist");                             // formatter-owned stable copy
        emitter.instruction("stp x1, x2, [sp, #24]");
        emitter.instruction("str x1, [sp, #40]");                               // owner released after copy
        emitter.instruction("ldr x0, [sp, #48]");
        emitter.instruction("bl __rt_decref_any");                              // release the bridge-owned result cell
        emitter.instruction("b __rt_sprintf_mixed_string_return_owned");
        emitter.label("__rt_sprintf_mixed_string_eval_bad");
        emitter.instruction("ldr x0, [sp, #48]");
        emitter.instruction("bl __rt_decref_any");
        emitter.instruction("b __rt_sprintf_mixed_string_missing");
        emitter.label("__rt_sprintf_mixed_string_eval_throw");
        emitter.instruction("ldr x0, [sp, #80]");                               // boxed Throwable
        emitter.instruction("bl __rt_mixed_unbox");                             // raw object pointer in x1
        abi::emit_store_reg_to_symbol(emitter, "x1", "_exc_value", 0);
        emitter.instruction("b __rt_throw_current");                            // unwind to the nearest active PHP catch handler
    } else {
        emitter.instruction("b __rt_sprintf_mixed_string_missing");             // no eval bridge in this runtime
    }

    emitter.label("__rt_sprintf_mixed_string_own");
    emitter.instruction("str x1, [sp, #88]");                                  // preserve the original method pointer
    emitter.instruction("bl __rt_str_persist");                                // make it independent of method scratch/ownership
    emitter.instruction("stp x1, x2, [sp, #24]");
    emitter.instruction("str x1, [sp, #40]");                                  // formatter owns the stabilized result
    emitter.instruction("ldr x9, [sp, #88]");                                  // original method result pointer
    emitter.instruction("cmp x1, x9");                                         // did persist take over the same concat block?
    emitter.instruction("b.eq __rt_sprintf_mixed_string_return_owned");         // yes, it is already the formatter owner
    emitter.instruction("mov x0, x9");                                          // release an independently owned method result
    emitter.instruction("bl __rt_heap_free_safe");                             // borrowed/static pointers are ignored safely
    emitter.instruction("b __rt_sprintf_mixed_string_return_owned");

    emitter.label("__rt_sprintf_mixed_string_return_owned");
    emitter.instruction("ldp x1, x2, [sp, #24]");                              // stabilized result pair
    emitter.instruction("ldr x0, [sp, #40]");                                  // owned pointer for post-copy release
    emitter.instruction("b __rt_sprintf_mixed_string_done");

    emitter.label("__rt_sprintf_mixed_string_missing");
    emitter.instruction("mov x0, #0");                                          // no owned result
    emitter.instruction("mov x1, #0");                                          // null result pointer asks sprintf for a controlled fatal
    emitter.instruction("mov x2, #0");                                          // missing conversions have no result bytes
    emitter.label("__rt_sprintf_mixed_string_done");
    emitter.instruction("ldp x29, x30, [sp, #96]");                             // restore the caller frame and return address
    emitter.instruction("add sp, sp, #112");                                    // release this helper's aligned frame
    emitter.instruction("ret");                                                 // return owner/string pair in x0/x1/x2
}

/// Emits both Linux x86_64 deferred non-scalar coercion helpers.
fn emit_sprintf_mixed_casts_linux_x86_64(emitter: &mut Emitter, eval_bridge: bool) {
    emit_sprintf_mixed_to_int_linux_x86_64(emitter);
    emit_sprintf_mixed_to_string_linux_x86_64(emitter, eval_bridge);
}

/// Emits x86_64 `__rt_sprintf_mixed_to_int(tag, payload) -> int`.
fn emit_sprintf_mixed_to_int_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf mixed non-scalar to int ---");
    emitter.label_global("__rt_sprintf_mixed_to_int");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame
    emitter.instruction("sub rsp, 16");                                         // keep nested SysV calls 16-byte aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // preserve raw payload across heap classification
    emitter.instruction("cmp rdi, 7");                                          // is the record payload a boxed Mixed cell?
    emitter.instruction("je __rt_sprintf_mixed_int_unbox_x64");                 // inspect the concrete boxed tag
    emitter.instruction("cmp rdi, 11");                                         // type-erased Iterable record?
    emitter.instruction("je __rt_sprintf_mixed_int_iterable_x64");              // classify it by heap kind
    emitter.instruction("jmp __rt_sprintf_mixed_int_dispatch_x64");             // other raw tags are concrete
    emitter.label("__rt_sprintf_mixed_int_unbox_x64");
    emitter.instruction("mov rax, rsi");                                        // __rt_mixed_unbox reads its boxed pointer from rax
    emitter.instruction("call __rt_mixed_unbox");                               // unwrap nested Mixed cells before inspecting the concrete tag
    emitter.instruction("jmp __rt_sprintf_mixed_int_dispatch_ready_x64");       // rax/rdi now carry tag/payload
    emitter.label("__rt_sprintf_mixed_int_iterable_x64");
    emitter.instruction("mov rax, rsi");
    emitter.instruction("call __rt_heap_kind");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("cmp rax, 2");
    emitter.instruction("je __rt_sprintf_mixed_int_array_x64");
    emitter.instruction("cmp rax, 3");
    emitter.instruction("je __rt_sprintf_mixed_int_array_x64");
    emitter.instruction("cmp rax, 4");
    emitter.instruction("je __rt_sprintf_mixed_int_one_x64");
    emitter.instruction("cmp rax, 6");
    emitter.instruction("je __rt_sprintf_mixed_int_one_x64");
    emitter.instruction("jmp __rt_sprintf_mixed_int_zero_x64");
    emitter.label("__rt_sprintf_mixed_int_dispatch_x64");
    emitter.instruction("mov rax, rdi");                                        // raw record tag
    emitter.instruction("mov rdi, rsi");                                        // raw record payload
    emitter.label("__rt_sprintf_mixed_int_dispatch_ready_x64");
    emitter.instruction("cmp rax, 4");                                          // indexed array?
    emitter.instruction("je __rt_sprintf_mixed_int_array_x64");                 // arrays cast to zero or one by emptiness
    emitter.instruction("cmp rax, 5");                                          // associative array?
    emitter.instruction("je __rt_sprintf_mixed_int_array_x64");                 // hashes share PHP's array numeric cast
    emitter.instruction("cmp rax, 6");                                          // object?
    emitter.instruction("je __rt_sprintf_mixed_int_one_x64");                   // every object casts to integer one
    emitter.instruction("cmp rax, 9");                                          // resource?
    emitter.instruction("je __rt_sprintf_mixed_int_resource_x64");              // resources format through their public resource id
    emitter.instruction("cmp rax, 10");                                         // callable descriptor (Closure shape)?
    emitter.instruction("je __rt_sprintf_mixed_int_one_x64");                   // callable objects cast to integer one
    emitter.instruction("xor eax, eax");                                        // unknown non-scalars normalize to zero, never their raw payload
    emitter.instruction("jmp __rt_sprintf_mixed_int_done_x64");                 // restore the helper frame and return

    emitter.label("__rt_sprintf_mixed_int_array_x64");
    emitter.instruction("test rdi, rdi");                                       // null container pointers behave like empty arrays
    emitter.instruction("jz __rt_sprintf_mixed_int_zero_x64");                  // skip the header load for a null container
    emitter.instruction("cmp QWORD PTR [rdi], 0");                              // is the live container element count zero?
    emitter.instruction("setne al");                                            // non-empty arrays cast to one
    emitter.instruction("movzx rax, al");                                       // normalize the boolean byte to a full integer result
    emitter.instruction("jmp __rt_sprintf_mixed_int_done_x64");                 // share the frame teardown

    emitter.label("__rt_sprintf_mixed_int_one_x64");
    emitter.instruction("mov eax, 1");                                          // object and callable numeric conversion result
    emitter.instruction("jmp __rt_sprintf_mixed_int_done_x64");                 // share the frame teardown

    emitter.label("__rt_sprintf_mixed_int_resource_x64");
    emitter.instruction("mov rax, rdi");                                        // pass the native resource payload to the id registry
    emitter.instruction("call __rt_resource_id_of");                            // return PHP's stable resource id instead of a handle/pointer
    emitter.instruction("jmp __rt_sprintf_mixed_int_done_x64");                 // share the frame teardown

    emitter.label("__rt_sprintf_mixed_int_zero_x64");
    emitter.instruction("xor eax, eax");                                        // empty/null container numeric conversion result
    emitter.label("__rt_sprintf_mixed_int_done_x64");
    emitter.instruction("leave");                                               // release the helper frame and restore rbp
    emitter.instruction("ret");                                                 // return the normalized integer in rax
}

/// Emits x86_64 `__rt_sprintf_mixed_to_string(tag, payload, eval_ctx) -> (ptr, len, owner)`.
fn emit_sprintf_mixed_to_string_linux_x86_64(emitter: &mut Emitter, eval_bridge: bool) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf mixed non-scalar to string ---");
    emitter.label_global("__rt_sprintf_mixed_to_string");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame
    emitter.instruction("sub rsp, 96");                                         // aligned spills and eval result structure
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save record tag
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save record payload
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save optional eval context
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // no eval input box yet
    emitter.instruction("mov QWORD PTR [rbp - 64], 0");                         // eval input is borrowed by default
    emitter.instruction("cmp rdi, 7");                                          // boxed Mixed record?
    emitter.instruction("jne __rt_sprintf_mixed_string_raw_x64");               // raw tags are already concrete
    emitter.instruction("mov QWORD PTR [rbp - 56], rsi");                       // preserve borrowed box for eval fallback
    emitter.instruction("mov rax, rsi");                                        // mixed_unbox consumes the box in rax
    emitter.instruction("call __rt_mixed_unbox");                               // concrete tag rax, payload rdi
    emitter.instruction("jmp __rt_sprintf_mixed_string_dispatch_x64");          // apply concrete semantics
    emitter.label("__rt_sprintf_mixed_string_raw_x64");
    emitter.instruction("cmp rdi, 11");                                         // type-erased Iterable record?
    emitter.instruction("jne __rt_sprintf_mixed_string_raw_ready_x64");
    emitter.instruction("mov rax, rsi");                                        // erased payload for heap classification
    emitter.instruction("call __rt_heap_kind");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // restore erased payload
    emitter.instruction("cmp rax, 2");
    emitter.instruction("je __rt_sprintf_mixed_string_array_x64");
    emitter.instruction("cmp rax, 3");
    emitter.instruction("je __rt_sprintf_mixed_string_array_x64");
    emitter.instruction("cmp rax, 4");
    emitter.instruction("je __rt_sprintf_mixed_string_object_x64");
    emitter.instruction("cmp rax, 6");
    emitter.instruction("je __rt_sprintf_mixed_string_object_x64");
    emitter.instruction("jmp __rt_sprintf_mixed_string_missing_x64");
    emitter.label("__rt_sprintf_mixed_string_raw_ready_x64");
    emitter.instruction("mov rax, rdi");                                        // raw record tag
    emitter.instruction("mov rdi, rsi");                                        // raw record payload
    emitter.label("__rt_sprintf_mixed_string_dispatch_x64");
    emitter.instruction("cmp rax, 4");                                          // indexed array?
    emitter.instruction("je __rt_sprintf_mixed_string_array_x64");              // arrays stringify to the literal "Array"
    emitter.instruction("cmp rax, 5");                                          // associative array?
    emitter.instruction("je __rt_sprintf_mixed_string_array_x64");              // hashes use the same PHP placeholder
    emitter.instruction("cmp rax, 6");                                          // object?
    emitter.instruction("je __rt_sprintf_mixed_string_object_x64");             // dispatch dynamically through the class __toString table
    emitter.instruction("cmp rax, 9");                                          // resource?
    emitter.instruction("je __rt_sprintf_mixed_string_resource_x64");           // resources use the public Resource id #N rendering
    emitter.instruction("jmp __rt_sprintf_mixed_string_missing_x64");           // callable/unknown values cannot be converted to string

    emitter.label("__rt_sprintf_mixed_string_array_x64");
    abi::emit_symbol_address(emitter, "rax", "_iterable_array_str");
    emitter.instruction("mov edx, 5");                                          // byte length of the literal "Array"
    emitter.instruction("xor ecx, ecx");                                        // fixed data is borrowed
    emitter.instruction("jmp __rt_sprintf_mixed_string_done_x64");              // return the borrowed fixed-data string pair

    emitter.label("__rt_sprintf_mixed_string_resource_x64");
    emitter.instruction("mov rax, rdi");                                        // pass the native resource payload to the display helper
    emitter.instruction("call __rt_resource_to_string");                        // return borrowed Resource id #N text in rax/rdx
    emitter.instruction("xor ecx, ecx");                                        // resource display storage is borrowed
    emitter.instruction("jmp __rt_sprintf_mixed_string_done_x64");              // share the frame teardown

    emitter.label("__rt_sprintf_mixed_string_object_x64");
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // preserve receiver for eval fallback
    emitter.instruction("mov r8, QWORD PTR [rdi]");                             // load the object's dense runtime class id
    emitter.instruction("test r8, r8");                                         // reject synthetic negative class ids
    emitter.instruction("js __rt_sprintf_mixed_string_eval_x64");               // synthetic ids belong to eval
    emitter.instruction("cmp r8, QWORD PTR [rip + _class_tostring_count]");     // is the class id inside the generated table?
    emitter.instruction("jae __rt_sprintf_mixed_string_eval_x64");              // out-of-range classes may belong to eval
    emitter.instruction("lea r10, [rip + _class_tostring_ptrs]");               // address the dense __toString function-pointer table
    emitter.instruction("mov r10, QWORD PTR [r10 + r8 * 8]");                   // resolve the concrete or inherited __toString symbol
    emitter.instruction("test r10, r10");                                       // did the class publish a conversion method?
    emitter.instruction("jz __rt_sprintf_mixed_string_eval_x64");               // a missing native hook may still be eval-backed
    emitter.instruction("call r10");                                            // call __toString with the borrowed receiver in rdi
    emitter.instruction("jmp __rt_sprintf_mixed_string_own_x64");               // stabilize and own the method result

    emitter.label("__rt_sprintf_mixed_string_eval_x64");
    if eval_bridge {
        emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                   // persistent eval context
        emitter.instruction("test rdi, rdi");
        emitter.instruction("jz __rt_sprintf_mixed_string_missing_x64");        // no dynamic method table without it
        emitter.instruction("mov eax, 6");                                      // runtime object tag
        emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                   // borrowed raw receiver
        emitter.instruction("xor esi, esi");                                    // objects have no high payload
        emitter.instruction("call __rt_mixed_from_value");                      // retain receiver in a temporary box
        emitter.instruction("mov QWORD PTR [rbp - 56], rax");                   // save owned input box
        emitter.instruction("mov r9, rax");
        emitter.instruction("mov QWORD PTR [rbp - 64], 1");                     // helper owns this box
        emitter.label("__rt_sprintf_mixed_string_eval_call_x64");
        emitter.instruction("mov QWORD PTR [rbp - 96], 0");                     // clear result kind/padding
        emitter.instruction("mov QWORD PTR [rbp - 88], 0");                     // clear value result pointer
        emitter.instruction("mov QWORD PTR [rbp - 80], 0");                     // clear throwable result pointer
        emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                   // context ABI arg 0
        emitter.instruction("mov rsi, r9");                                     // boxed value ABI arg 1
        emitter.instruction("lea rdx, [rbp - 96]");                             // result ABI arg 2
        let symbol = emitter.target.extern_symbol("__elephc_eval_string_context");
        abi::emit_call_label(emitter, &symbol);
        emitter.instruction("mov QWORD PTR [rbp - 8], rax");                    // preserve bridge status during input cleanup
        emitter.instruction("cmp QWORD PTR [rbp - 64], 0");
        emitter.instruction("je __rt_sprintf_mixed_string_eval_status_x64");
        emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                   // owned temporary input box
        emitter.instruction("call __rt_decref_any");                            // balance retained receiver
        emitter.label("__rt_sprintf_mixed_string_eval_status_x64");
        emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                    // bridge status
        emitter.instruction("test rax, rax");
        emitter.instruction("jz __rt_sprintf_mixed_string_eval_ok_x64");
        emitter.instruction("cmp rax, 3");                                      // uncaught Throwable?
        emitter.instruction("je __rt_sprintf_mixed_string_eval_throw_x64");
        emitter.instruction("jmp __rt_sprintf_mixed_string_missing_x64");
        emitter.label("__rt_sprintf_mixed_string_eval_ok_x64");
        emitter.instruction("mov rax, QWORD PTR [rbp - 88]");                   // boxed string result
        emitter.instruction("mov QWORD PTR [rbp - 56], rax");                   // preserve through persistence
        emitter.instruction("call __rt_mixed_unbox");
        emitter.instruction("cmp rax, 1");                                      // bridge must return a string cell
        emitter.instruction("jne __rt_sprintf_mixed_string_eval_bad_x64");
        emitter.instruction("mov rax, rdi");                                    // standard string pointer register
        emitter.instruction("call __rt_str_persist");                           // formatter-owned stable copy
        emitter.instruction("mov QWORD PTR [rbp - 32], rax");
        emitter.instruction("mov QWORD PTR [rbp - 40], rdx");
        emitter.instruction("mov QWORD PTR [rbp - 48], rax");                   // owner released after copy
        emitter.instruction("mov rax, QWORD PTR [rbp - 56]");
        emitter.instruction("call __rt_decref_any");                            // release bridge-owned result cell
        emitter.instruction("jmp __rt_sprintf_mixed_string_return_owned_x64");
        emitter.label("__rt_sprintf_mixed_string_eval_bad_x64");
        emitter.instruction("mov rax, QWORD PTR [rbp - 56]");
        emitter.instruction("call __rt_decref_any");
        emitter.instruction("jmp __rt_sprintf_mixed_string_missing_x64");
        emitter.label("__rt_sprintf_mixed_string_eval_throw_x64");
        emitter.instruction("mov rax, QWORD PTR [rbp - 80]");                   // boxed Throwable
        emitter.instruction("call __rt_mixed_unbox");                           // raw object pointer in rdi
        abi::emit_store_reg_to_symbol(emitter, "rdi", "_exc_value", 0);
        emitter.instruction("jmp __rt_throw_current");                          // unwind to the nearest active PHP catch handler
    } else {
        emitter.instruction("jmp __rt_sprintf_mixed_string_missing_x64");        // no eval bridge in this runtime
    }

    emitter.label("__rt_sprintf_mixed_string_own_x64");
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // preserve original method result pointer
    emitter.instruction("call __rt_str_persist");                               // stabilize concat/heap/static results
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // formatter owns stabilized result
    emitter.instruction("cmp rax, QWORD PTR [rbp - 72]");                       // did persist take over the same block?
    emitter.instruction("je __rt_sprintf_mixed_string_return_owned_x64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 72]");                       // independently owned original result
    emitter.instruction("call __rt_heap_free_safe");                            // borrowed/static pointers are ignored safely
    emitter.label("__rt_sprintf_mixed_string_return_owned_x64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // stabilized result pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // stabilized result length
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // owner for post-copy release
    emitter.instruction("jmp __rt_sprintf_mixed_string_done_x64");

    emitter.label("__rt_sprintf_mixed_string_missing_x64");
    emitter.instruction("xor eax, eax");                                        // null result pointer asks sprintf for a controlled fatal
    emitter.instruction("xor edx, edx");                                        // missing conversions have no result bytes
    emitter.instruction("xor ecx, ecx");                                        // no owned result
    emitter.label("__rt_sprintf_mixed_string_done_x64");
    emitter.instruction("leave");                                               // release the helper frame and restore rbp
    emitter.instruction("ret");                                                 // return the string pair in rax/rdx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::{Platform, Target};

    /// Emits both helpers for one target and returns their assembly text.
    fn emit_for(target: Target) -> String {
        let mut emitter = Emitter::new(target);
        emit_sprintf_mixed_casts(&mut emitter, false);
        emitter.output()
    }

    /// Pins non-scalar casts on every supported architecture so arrays, resources,
    /// objects, and callable descriptors never fall back to raw payload bits.
    #[test]
    fn test_sprintf_mixed_casts_preserve_php_non_scalar_semantics_on_both_architectures() {
        let arm = emit_for(Target::new(Platform::MacOS, Arch::AArch64));
        assert!(arm.contains("cset x0, ne"), "{arm}");
        assert!(arm.contains("bl __rt_resource_id_of"), "{arm}");
        assert!(arm.contains("_class_tostring_ptrs"), "{arm}");
        assert!(arm.contains("_iterable_array_str"), "{arm}");

        let x64 = emit_for(Target::new(Platform::Linux, Arch::X86_64));
        assert!(x64.contains("setne al"), "{x64}");
        assert!(x64.contains("call __rt_resource_id_of"), "{x64}");
        assert!(x64.contains("_class_tostring_ptrs"), "{x64}");
        assert!(x64.contains("_iterable_array_str"), "{x64}");
    }

    /// Both target emitters route eval-declared stringification through the bridge and unwinder.
    #[test]
    fn test_sprintf_eval_string_conversion_uses_owned_results_and_standard_unwinding() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            emit_sprintf_mixed_casts(&mut emitter, true);
            let asm = emitter.output();
            assert!(asm.contains("__elephc_eval_string_context"), "{target:?}\n{asm}");
            assert!(asm.contains("__rt_str_persist"), "{target:?}\n{asm}");
            assert!(asm.contains("__rt_decref_any"), "{target:?}\n{asm}");
            assert!(asm.contains("__rt_throw_current"), "{target:?}\n{asm}");
        }
    }
}
