//! Purpose:
//! Emits the printf-family coercion helpers for non-scalar boxed `Mixed` operands.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via
//!   `crate::codegen_support::runtime::strings`.
//! - `__rt_sprintf` after a packed argument record preserves a boxed value with tag 7.
//!
//! Key details:
//! - Numeric formatting follows PHP's non-scalar cast rules: empty arrays become 0,
//!   non-empty arrays, objects, and callable descriptors become 1, and resources use
//!   their public resource id rather than their native handle.
//! - String formatting returns `"Array"`, resource display text, or a dynamic
//!   `__toString()` result. A null result pair tells `__rt_sprintf` to take its controlled
//!   object-to-string fatal path; raw heap pointers never become numeric output.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits the non-scalar boxed-Mixed coercion helpers used by `__rt_sprintf`.
pub fn emit_sprintf_mixed_casts(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_sprintf_mixed_casts_linux_x86_64(emitter);
        return;
    }

    emit_sprintf_mixed_to_int_aarch64(emitter);
    emit_sprintf_mixed_to_string_aarch64(emitter);
}

/// Emits AArch64 `__rt_sprintf_mixed_to_int(boxed_cell) -> int`.
fn emit_sprintf_mixed_to_int_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf mixed non-scalar to int ---");
    emitter.label_global("__rt_sprintf_mixed_to_int");

    emitter.instruction("sub sp, sp, #32");                                     // reserve an aligned helper frame for nested runtime calls
    emitter.instruction("stp x29, x30, [sp, #16]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #16");                                    // establish this helper's frame pointer
    emitter.instruction("bl __rt_mixed_unbox");                                 // unwrap nested Mixed cells before inspecting the concrete tag
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

/// Emits AArch64 `__rt_sprintf_mixed_to_string(boxed_cell) -> (ptr, len)`.
fn emit_sprintf_mixed_to_string_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf mixed non-scalar to string ---");
    emitter.label_global("__rt_sprintf_mixed_to_string");

    emitter.instruction("sub sp, sp, #32");                                     // reserve an aligned frame for resource and method calls
    emitter.instruction("stp x29, x30, [sp, #16]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #16");                                    // establish this helper's frame pointer
    emitter.instruction("bl __rt_mixed_unbox");                                 // unwrap nested boxes and expose the concrete non-scalar tag
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
    emitter.instruction("b __rt_sprintf_mixed_string_done");                    // return the borrowed fixed-data string pair

    emitter.label("__rt_sprintf_mixed_string_resource");
    emitter.instruction("mov x0, x1");                                          // pass the native resource payload to the display helper
    emitter.instruction("bl __rt_resource_to_string");                          // return borrowed Resource id #N text in x1/x2
    emitter.instruction("b __rt_sprintf_mixed_string_done");                    // share the frame teardown

    emitter.label("__rt_sprintf_mixed_string_object");
    emitter.instruction("mov x0, x1");                                          // keep the borrowed object as the method receiver
    emitter.instruction("ldr x11, [x0]");                                       // load the object's dense runtime class id
    emitter.instruction("tbnz x11, #63, __rt_sprintf_mixed_string_missing");    // synthetic negative ids cannot index generated metadata
    abi::emit_load_symbol_to_reg(emitter, "x10", "_class_tostring_count", 0);
    emitter.instruction("cmp x11, x10");                                        // is the class id inside the generated table?
    emitter.instruction("b.hs __rt_sprintf_mixed_string_missing");              // out-of-range classes have no callable conversion
    abi::emit_symbol_address(emitter, "x10", "_class_tostring_ptrs");
    emitter.instruction("ldr x10, [x10, x11, lsl #3]");                         // resolve the concrete or inherited __toString symbol
    emitter.instruction("cbz x10, __rt_sprintf_mixed_string_missing");          // a zero entry means the object is not stringable
    emitter.instruction("blr x10");                                             // call __toString with the borrowed receiver in x0
    emitter.instruction("b __rt_sprintf_mixed_string_done");                    // return the method's string pair

    emitter.label("__rt_sprintf_mixed_string_missing");
    emitter.instruction("mov x1, #0");                                          // null result pointer asks sprintf for a controlled fatal
    emitter.instruction("mov x2, #0");                                          // missing conversions have no result bytes
    emitter.label("__rt_sprintf_mixed_string_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore the caller frame and return address
    emitter.instruction("add sp, sp, #32");                                     // release this helper's aligned frame
    emitter.instruction("ret");                                                 // return the string pair in x1/x2
}

/// Emits both Linux x86_64 non-scalar boxed-Mixed coercion helpers.
fn emit_sprintf_mixed_casts_linux_x86_64(emitter: &mut Emitter) {
    emit_sprintf_mixed_to_int_linux_x86_64(emitter);
    emit_sprintf_mixed_to_string_linux_x86_64(emitter);
}

/// Emits x86_64 `__rt_sprintf_mixed_to_int(boxed_cell) -> int`.
fn emit_sprintf_mixed_to_int_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf mixed non-scalar to int ---");
    emitter.label_global("__rt_sprintf_mixed_to_int");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame
    emitter.instruction("sub rsp, 16");                                         // keep nested SysV calls 16-byte aligned
    emitter.instruction("mov rax, rdi");                                        // __rt_mixed_unbox reads its boxed pointer from rax
    emitter.instruction("call __rt_mixed_unbox");                               // unwrap nested Mixed cells before inspecting the concrete tag
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

/// Emits x86_64 `__rt_sprintf_mixed_to_string(boxed_cell) -> (ptr, len)`.
fn emit_sprintf_mixed_to_string_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf mixed non-scalar to string ---");
    emitter.label_global("__rt_sprintf_mixed_to_string");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame
    emitter.instruction("sub rsp, 16");                                         // keep nested SysV calls 16-byte aligned
    emitter.instruction("mov rax, rdi");                                        // __rt_mixed_unbox reads its boxed pointer from rax
    emitter.instruction("call __rt_mixed_unbox");                               // unwrap nested boxes and expose the concrete non-scalar tag
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
    emitter.instruction("jmp __rt_sprintf_mixed_string_done_x64");              // return the borrowed fixed-data string pair

    emitter.label("__rt_sprintf_mixed_string_resource_x64");
    emitter.instruction("mov rax, rdi");                                        // pass the native resource payload to the display helper
    emitter.instruction("call __rt_resource_to_string");                        // return borrowed Resource id #N text in rax/rdx
    emitter.instruction("jmp __rt_sprintf_mixed_string_done_x64");              // share the frame teardown

    emitter.label("__rt_sprintf_mixed_string_object_x64");
    emitter.instruction("mov r8, QWORD PTR [rdi]");                             // load the object's dense runtime class id
    emitter.instruction("test r8, r8");                                         // reject synthetic negative class ids
    emitter.instruction("js __rt_sprintf_mixed_string_missing_x64");            // synthetic ids cannot index generated metadata
    emitter.instruction("cmp r8, QWORD PTR [rip + _class_tostring_count]");     // is the class id inside the generated table?
    emitter.instruction("jae __rt_sprintf_mixed_string_missing_x64");           // out-of-range classes have no callable conversion
    emitter.instruction("lea r10, [rip + _class_tostring_ptrs]");               // address the dense __toString function-pointer table
    emitter.instruction("mov r10, QWORD PTR [r10 + r8 * 8]");                   // resolve the concrete or inherited __toString symbol
    emitter.instruction("test r10, r10");                                       // did the class publish a conversion method?
    emitter.instruction("jz __rt_sprintf_mixed_string_missing_x64");            // a zero entry means the object is not stringable
    emitter.instruction("call r10");                                            // call __toString with the borrowed receiver in rdi
    emitter.instruction("jmp __rt_sprintf_mixed_string_done_x64");              // return the method's string pair

    emitter.label("__rt_sprintf_mixed_string_missing_x64");
    emitter.instruction("xor eax, eax");                                        // null result pointer asks sprintf for a controlled fatal
    emitter.instruction("xor edx, edx");                                        // missing conversions have no result bytes
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
        emit_sprintf_mixed_casts(&mut emitter);
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
}
