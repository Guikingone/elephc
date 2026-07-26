//! Purpose:
//! Emits the `__rt_mixed_bitwise` runtime helper assembly for PHP's runtime-
//! polymorphic bitwise operators (`&`/`|`/`^`) when at least one operand is a
//! dynamic `Mixed`/union value and the other is a string or also dynamic. The
//! string-vs-integer choice can only be made at runtime, so this helper unboxes
//! both operands, inspects their tags, and dispatches to the correct PHP path.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Both operands are boxed `Mixed` cells on entry. When both hold strings the
//!   helper computes a bytewise string via the shared `__rt_str_bitwise` and boxes
//!   the persisted result; a runtime array/object operand is a PHP `TypeError`
//!   (loud fatal); every other pairing coerces both operands to int and boxes the
//!   integer bitwise result, matching PHP (`&`/`|`/`^` are bytewise only when BOTH
//!   operands are strings, otherwise integer with string→int coercion).
//! - The operator mode matches `crate::ir::StrBitKind::as_mode` and
//!   `__rt_str_bitwise` (0 = And, 1 = Or, 2 = Xor).

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::{abi, platform::Arch};

/// Emits the `__rt_mixed_bitwise` runtime helper for PHP runtime-polymorphic bitwise ops.
///
/// Input:  AArch64 x0=left Mixed*, x1=right Mixed*, x2=mode
///         x86_64  rax=left Mixed*, rdi=right Mixed*, rsi=mode
/// Output: boxed Mixed pointer in the integer result register (x0 / rax)
pub fn emit_mixed_bitwise(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_bitwise_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: mixed_bitwise ---");
    emitter.label_global("__rt_mixed_bitwise");

    emitter.instruction("sub sp, sp, #80");                                     // allocate a helper frame for operands, tags, and payloads
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #64");                                    // establish a stable helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the boxed left operand pointer for casts
    emitter.instruction("str x1, [sp, #8]");                                    // save the boxed right operand pointer for casts
    emitter.instruction("str x2, [sp, #16]");                                   // save the selected bitwise operator mode

    // -- unbox the left operand and reject array/object payloads --
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0=tag, x1=value_lo, x2=value_hi for the left operand
    emitter.instruction("str x0, [sp, #24]");                                   // save the left runtime tag for the string/int dispatch
    emitter.instruction("str x1, [sp, #32]");                                   // save the left string pointer for the bytewise path
    emitter.instruction("str x2, [sp, #40]");                                   // save the left string length for the bytewise path
    emitter.instruction("cmp x0, #4");                                          // does the left operand hold an indexed array?
    emitter.instruction("b.eq __rt_mixed_bitwise_type_error");                  // arrays are never valid bitwise operands
    emitter.instruction("cmp x0, #5");                                          // does the left operand hold an associative array?
    emitter.instruction("b.eq __rt_mixed_bitwise_type_error");                  // arrays are never valid bitwise operands
    emitter.instruction("cmp x0, #6");                                          // does the left operand hold an object?
    emitter.instruction("b.eq __rt_mixed_bitwise_type_error");                  // objects are never valid bitwise operands

    // -- unbox the right operand and reject array/object payloads --
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the boxed right operand pointer
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0=tag, x1=value_lo, x2=value_hi for the right operand
    emitter.instruction("str x1, [sp, #48]");                                   // save the right string pointer for the bytewise path
    emitter.instruction("str x2, [sp, #56]");                                   // save the right string length for the bytewise path
    emitter.instruction("cmp x0, #4");                                          // does the right operand hold an indexed array?
    emitter.instruction("b.eq __rt_mixed_bitwise_type_error");                  // arrays are never valid bitwise operands
    emitter.instruction("cmp x0, #5");                                          // does the right operand hold an associative array?
    emitter.instruction("b.eq __rt_mixed_bitwise_type_error");                  // arrays are never valid bitwise operands
    emitter.instruction("cmp x0, #6");                                          // does the right operand hold an object?
    emitter.instruction("b.eq __rt_mixed_bitwise_type_error");                  // objects are never valid bitwise operands

    // -- both operands strings -> PHP bytewise string result --
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload the left runtime tag
    emitter.instruction("cmp x9, #1");                                          // is the left operand a string?
    emitter.instruction("b.ne __rt_mixed_bitwise_int");                         // any non-string operand forces the integer path
    emitter.instruction("cmp x0, #1");                                          // is the right operand a string? (tag still in x0)
    emitter.instruction("b.ne __rt_mixed_bitwise_int");                         // any non-string operand forces the integer path
    emitter.instruction("ldr x1, [sp, #32]");                                   // left string pointer
    emitter.instruction("ldr x2, [sp, #40]");                                   // left string length
    emitter.instruction("ldr x3, [sp, #48]");                                   // right string pointer
    emitter.instruction("ldr x4, [sp, #56]");                                   // right string length
    emitter.instruction("ldr x5, [sp, #16]");                                   // bitwise operator mode
    emitter.instruction("bl __rt_str_bitwise");                                 // x1=result pointer, x2=result length in concat scratch
    emitter.instruction("mov x0, #1");                                          // runtime tag 1 = string
    emitter.instruction("bl __rt_mixed_from_value");                            // persist and box the bytewise string result
    emitter.instruction("b __rt_mixed_bitwise_done");                           // return the boxed string result

    // -- integer path: coerce both operands to int, then box the integer result --
    emitter.label("__rt_mixed_bitwise_int");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the boxed left operand for the int cast
    emitter.instruction("bl __rt_mixed_cast_int");                              // coerce the left operand to int via PHP rules
    emitter.instruction("str x0, [sp, #32]");                                   // reuse the left string slot to save the left int
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the boxed right operand for the int cast
    emitter.instruction("bl __rt_mixed_cast_int");                              // coerce the right operand to int via PHP rules
    emitter.instruction("mov x2, x0");                                          // keep the right int operand in x2
    emitter.instruction("ldr x1, [sp, #32]");                                   // reload the left int operand into x1
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload the bitwise operator mode
    emitter.instruction("cmp x9, #1");                                          // mode 1 selects OR
    emitter.instruction("b.eq __rt_mixed_bitwise_int_or");                      // branch to the OR combine
    emitter.instruction("cmp x9, #2");                                          // mode 2 selects XOR
    emitter.instruction("b.eq __rt_mixed_bitwise_int_xor");                     // branch to the XOR combine
    emitter.instruction("and x1, x1, x2");                                      // mode 0 = AND: combine the two int operands
    emitter.instruction("b __rt_mixed_bitwise_int_box");                        // box the integer AND result
    emitter.label("__rt_mixed_bitwise_int_or");
    emitter.instruction("orr x1, x1, x2");                                      // combine the two int operands with OR
    emitter.instruction("b __rt_mixed_bitwise_int_box");                        // box the integer OR result
    emitter.label("__rt_mixed_bitwise_int_xor");
    emitter.instruction("eor x1, x1, x2");                                      // combine the two int operands with XOR
    emitter.label("__rt_mixed_bitwise_int_box");
    emitter.instruction("mov x0, #0");                                          // runtime tag 0 = integer
    emitter.instruction("mov x2, #0");                                          // integer payloads do not use a high word
    emitter.instruction("bl __rt_mixed_from_value");                            // box the integer bitwise result
    emitter.instruction("b __rt_mixed_bitwise_done");                           // return the boxed integer result

    // -- runtime array/object operand: PHP TypeError (loud fatal) --
    emitter.label("__rt_mixed_bitwise_type_error");
    abi::emit_symbol_address(emitter, "x1", "_array_arg_type_error_msg");
    emitter.instruction("mov x2, #78");                                         // pass the shared gradual operand TypeError message length
    emitter.instruction("mov x0, #2");                                          // write the invalid operand diagnostic to stderr
    emitter.syscall(4);
    emitter.instruction("mov x0, #70");                                         // use EX_SOFTWARE after the invalid operand pairing
    emitter.syscall(1);

    emitter.label("__rt_mixed_bitwise_done");
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // release the helper stack frame
    emitter.instruction("ret");                                                 // return the boxed Mixed result in x0
}

/// Emits the x86_64 Linux variant of `__rt_mixed_bitwise`.
///
/// Uses the System V AMD64 ABI as used by the sibling Mixed helpers: left boxed
/// pointer in rax, right in rdi, mode in rsi. The result is a boxed Mixed pointer
/// returned in rax.
fn emit_mixed_bitwise_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_bitwise ---");
    emitter.label_global("__rt_mixed_bitwise");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame base
    emitter.instruction("sub rsp, 64");                                         // reserve aligned slots for operands, tags, and payloads
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the boxed left operand pointer for casts
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // save the boxed right operand pointer for casts
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // save the selected bitwise operator mode

    // -- unbox the left operand and reject array/object payloads --
    emitter.instruction("call __rt_mixed_unbox");                               // rax=tag, rdi=value_lo, rdx=value_hi for the left operand
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the left runtime tag
    emitter.instruction("mov QWORD PTR [rbp - 40], rdi");                       // save the left string pointer for the bytewise path
    emitter.instruction("mov QWORD PTR [rbp - 48], rdx");                       // save the left string length for the bytewise path
    emitter.instruction("cmp rax, 4");                                          // does the left operand hold an indexed array?
    emitter.instruction("je __rt_mixed_bitwise_type_error_linux_x86_64");       // arrays are never valid bitwise operands
    emitter.instruction("cmp rax, 5");                                          // does the left operand hold an associative array?
    emitter.instruction("je __rt_mixed_bitwise_type_error_linux_x86_64");       // arrays are never valid bitwise operands
    emitter.instruction("cmp rax, 6");                                          // does the left operand hold an object?
    emitter.instruction("je __rt_mixed_bitwise_type_error_linux_x86_64");       // objects are never valid bitwise operands

    // -- unbox the right operand and reject array/object payloads --
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the boxed right operand pointer
    emitter.instruction("call __rt_mixed_unbox");                               // rax=tag, rdi=value_lo, rdx=value_hi for the right operand
    emitter.instruction("mov QWORD PTR [rbp - 56], rdi");                       // save the right string pointer for the bytewise path
    emitter.instruction("mov QWORD PTR [rbp - 64], rdx");                       // save the right string length for the bytewise path
    emitter.instruction("cmp rax, 4");                                          // does the right operand hold an indexed array?
    emitter.instruction("je __rt_mixed_bitwise_type_error_linux_x86_64");       // arrays are never valid bitwise operands
    emitter.instruction("cmp rax, 5");                                          // does the right operand hold an associative array?
    emitter.instruction("je __rt_mixed_bitwise_type_error_linux_x86_64");       // arrays are never valid bitwise operands
    emitter.instruction("cmp rax, 6");                                          // does the right operand hold an object?
    emitter.instruction("je __rt_mixed_bitwise_type_error_linux_x86_64");       // objects are never valid bitwise operands

    // -- both operands strings -> PHP bytewise string result --
    emitter.instruction("cmp QWORD PTR [rbp - 32], 1");                         // is the left operand a string?
    emitter.instruction("jne __rt_mixed_bitwise_int_linux_x86_64");             // any non-string operand forces the integer path
    emitter.instruction("cmp rax, 1");                                          // is the right operand a string? (tag still in rax)
    emitter.instruction("jne __rt_mixed_bitwise_int_linux_x86_64");             // any non-string operand forces the integer path
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // left string pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // left string length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 56]");                       // right string pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 64]");                       // right string length
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                       // bitwise operator mode
    emitter.instruction("call __rt_str_bitwise");                              // rax=result pointer, rdx=result length in concat scratch
    emitter.instruction("mov rdi, rax");                                        // move the result pointer into the from_value low payload register
    emitter.instruction("mov rsi, rdx");                                        // move the result length into the from_value high payload register
    emitter.instruction("mov rax, 1");                                          // runtime tag 1 = string
    emitter.instruction("call __rt_mixed_from_value");                         // persist and box the bytewise string result
    emitter.instruction("jmp __rt_mixed_bitwise_done_linux_x86_64");            // return the boxed string result

    // -- integer path: coerce both operands to int, then box the integer result --
    emitter.label("__rt_mixed_bitwise_int_linux_x86_64");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the boxed left operand for the int cast
    emitter.instruction("call __rt_mixed_cast_int");                            // coerce the left operand to int via PHP rules
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // reuse the left string slot to save the left int
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the boxed right operand for the int cast
    emitter.instruction("call __rt_mixed_cast_int");                            // coerce the right operand to int via PHP rules
    emitter.instruction("mov rcx, rax");                                        // keep the right int operand in rcx
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // reload the left int operand into rax
    emitter.instruction("cmp QWORD PTR [rbp - 24], 1");                         // mode 1 selects OR
    emitter.instruction("je __rt_mixed_bitwise_int_or_linux_x86_64");           // branch to the OR combine
    emitter.instruction("cmp QWORD PTR [rbp - 24], 2");                         // mode 2 selects XOR
    emitter.instruction("je __rt_mixed_bitwise_int_xor_linux_x86_64");          // branch to the XOR combine
    emitter.instruction("and rax, rcx");                                        // mode 0 = AND: combine the two int operands
    emitter.instruction("jmp __rt_mixed_bitwise_int_box_linux_x86_64");         // box the integer AND result
    emitter.label("__rt_mixed_bitwise_int_or_linux_x86_64");
    emitter.instruction("or rax, rcx");                                         // combine the two int operands with OR
    emitter.instruction("jmp __rt_mixed_bitwise_int_box_linux_x86_64");         // box the integer OR result
    emitter.label("__rt_mixed_bitwise_int_xor_linux_x86_64");
    emitter.instruction("xor rax, rcx");                                        // combine the two int operands with XOR
    emitter.label("__rt_mixed_bitwise_int_box_linux_x86_64");
    emitter.instruction("mov rdi, rax");                                        // move the integer result into the from_value low payload register
    emitter.instruction("xor rsi, rsi");                                        // integer payloads do not use a high word
    emitter.instruction("mov rax, 0");                                          // runtime tag 0 = integer
    emitter.instruction("call __rt_mixed_from_value");                         // box the integer bitwise result
    emitter.instruction("jmp __rt_mixed_bitwise_done_linux_x86_64");            // return the boxed integer result

    // -- runtime array/object operand: PHP TypeError (loud fatal) --
    emitter.label("__rt_mixed_bitwise_type_error_linux_x86_64");
    emitter.instruction("mov edi, 2");                                          // write the invalid operand diagnostic to stderr
    abi::emit_symbol_address(emitter, "rsi", "_array_arg_type_error_msg");
    emitter.instruction("mov edx, 78");                                         // pass the shared gradual operand TypeError message length
    emitter.instruction("mov eax, 1");                                          // select the Linux write syscall
    emitter.instruction("syscall");                                             // emit the invalid operand diagnostic
    emitter.instruction("mov edi, 70");                                         // use EX_SOFTWARE after the invalid operand pairing
    emitter.instruction("mov eax, 60");                                         // select the Linux exit syscall
    emitter.instruction("syscall");                                             // terminate after the operand TypeError

    emitter.label("__rt_mixed_bitwise_done_linux_x86_64");
    emitter.instruction("add rsp, 64");                                         // release the helper stack frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the boxed Mixed result in rax
}

#[cfg(test)]
mod tests {
    use crate::codegen::platform::{Arch, Platform, Target};

    use super::*;

    /// Verifies the AArch64 helper emits the global symbol, the string and integer
    /// dispatch paths, and the array/object TypeError fatal.
    #[test]
    fn test_emit_mixed_bitwise_arm64_emits_all_paths() {
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_mixed_bitwise(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_mixed_bitwise:\n"));
        assert!(asm.contains("bl __rt_str_bitwise"));
        assert!(asm.contains("bl __rt_mixed_cast_int"));
        assert!(asm.contains("__rt_mixed_bitwise_type_error:\n"));
        assert!(asm.contains("and x1, x1, x2"));
        assert!(asm.contains("orr x1, x1, x2"));
        assert!(asm.contains("eor x1, x1, x2"));
    }

    /// Verifies the x86_64 helper emits the shared dispatch and returns the boxed
    /// result via the SysV integer result register.
    #[test]
    fn test_emit_mixed_bitwise_linux_x86_64_emits_all_paths() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_mixed_bitwise(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_mixed_bitwise:\n"));
        assert!(asm.contains("call __rt_str_bitwise"));
        assert!(asm.contains("call __rt_mixed_cast_int"));
        assert!(asm.contains("and rax, rcx\n"));
        assert!(asm.contains("or rax, rcx\n"));
        assert!(asm.contains("xor rax, rcx\n"));
    }
}
