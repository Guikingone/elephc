//! Purpose:
//! Emits the `__rt_php_compare` runtime helper implementing PHP 8 three-way ordered
//! comparison (`<`, `<=`, `>`, `>=`, `<=>`) over two boxed Mixed operands.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - Reuses `__rt_mixed_unbox`/`__rt_str_to_number` for PHP numeric-string detection,
//!   `__rt_mixed_cast_float` for the numeric branch, and `__rt_mixed_cast_string` +
//!   `__rt_strcmp` for the lexicographic fallback. Returns a normalized sign in x0/rax.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits `__rt_php_compare`: compares two boxed Mixed operands with PHP 8 ordering rules.
///
/// PHP 8 semantics: when both operands are numbers or numeric strings the comparison is
/// numeric; otherwise (at least one non-numeric string, or an object/array payload) both
/// sides are stringified and compared lexicographically. Null/bool fold through the numeric
/// branch (null → 0, bool → truthiness) via `__rt_mixed_cast_float`.
///
/// Register contract:
/// - ARM64: input x0 = left boxed Mixed, x1 = right boxed Mixed; output x0 = -1/0/+1.
/// - x86_64 System V: input rdi = left boxed Mixed, rsi = right boxed Mixed; output rax = -1/0/+1.
///
/// Dispatches to the x86_64 variant on that target; otherwise emits ARM64 assembly inline.
pub fn emit_php_compare(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_php_compare_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: php_compare ---");
    emitter.label_global("__rt_php_compare");

    // -- set up helper frame and save both boxed operands --
    emitter.instruction("sub sp, sp, #48");                                     // allocate frame slots for operands plus scratch
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the helper frame pointer
    emitter.instruction("stp x0, x1, [sp, #0]");                                // save left operand at +0 and right operand at +8

    // -- classify the left operand as numeric or non-numeric --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the left boxed operand
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0=tag, x1=value_lo, x2=value_hi
    emitter.instruction("cmp x0, #1");                                          // is the left payload a string?
    emitter.instruction("b.ne __rt_php_compare_l_nonstr");                      // non-string payloads classify from their tag
    emitter.instruction("bl __rt_str_to_number");                               // numeric-string detection: x0=1 when numeric
    emitter.instruction("b __rt_php_compare_l_done");                           // left numericness computed

    emitter.label("__rt_php_compare_l_nonstr");
    emitter.instruction("mov x9, x0");                                          // preserve the unboxed left runtime tag
    emitter.instruction("mov x0, #1");                                          // assume the non-string left payload is numeric
    emitter.instruction("cmp x9, #0");                                          // tag 0 = int (numeric)
    emitter.instruction("b.eq __rt_php_compare_l_done");                        // ints participate in the numeric branch
    emitter.instruction("cmp x9, #2");                                          // tag 2 = float (numeric)
    emitter.instruction("b.eq __rt_php_compare_l_done");                        // floats participate in the numeric branch
    emitter.instruction("cmp x9, #3");                                          // tag 3 = bool (numeric via truthiness)
    emitter.instruction("b.eq __rt_php_compare_l_done");                        // bools fold to 0/1 in the numeric branch
    emitter.instruction("cmp x9, #8");                                          // tag 8 = null (numeric as 0)
    emitter.instruction("b.eq __rt_php_compare_l_done");                        // null folds to 0 in the numeric branch
    emitter.instruction("mov x0, #0");                                          // arrays/objects compare lexicographically

    emitter.label("__rt_php_compare_l_done");
    emitter.instruction("str x0, [sp, #16]");                                   // save the left numericness flag

    // -- classify the right operand as numeric or non-numeric --
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the right boxed operand
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0=tag, x1=value_lo, x2=value_hi
    emitter.instruction("cmp x0, #1");                                          // is the right payload a string?
    emitter.instruction("b.ne __rt_php_compare_r_nonstr");                      // non-string payloads classify from their tag
    emitter.instruction("bl __rt_str_to_number");                               // numeric-string detection: x0=1 when numeric
    emitter.instruction("b __rt_php_compare_r_done");                           // right numericness computed

    emitter.label("__rt_php_compare_r_nonstr");
    emitter.instruction("mov x9, x0");                                          // preserve the unboxed right runtime tag
    emitter.instruction("mov x0, #1");                                          // assume the non-string right payload is numeric
    emitter.instruction("cmp x9, #0");                                          // tag 0 = int (numeric)
    emitter.instruction("b.eq __rt_php_compare_r_done");                        // ints participate in the numeric branch
    emitter.instruction("cmp x9, #2");                                          // tag 2 = float (numeric)
    emitter.instruction("b.eq __rt_php_compare_r_done");                        // floats participate in the numeric branch
    emitter.instruction("cmp x9, #3");                                          // tag 3 = bool (numeric via truthiness)
    emitter.instruction("b.eq __rt_php_compare_r_done");                        // bools fold to 0/1 in the numeric branch
    emitter.instruction("cmp x9, #8");                                          // tag 8 = null (numeric as 0)
    emitter.instruction("b.eq __rt_php_compare_r_done");                        // null folds to 0 in the numeric branch
    emitter.instruction("mov x0, #0");                                          // arrays/objects compare lexicographically

    emitter.label("__rt_php_compare_r_done");
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload the left numericness flag
    emitter.instruction("cbz x0, __rt_php_compare_strcmp");                     // a non-numeric right side forces string comparison
    emitter.instruction("cbz x9, __rt_php_compare_strcmp");                     // a non-numeric left side forces string comparison

    // -- numeric branch: compare the two operands as doubles --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the left boxed operand
    emitter.instruction("bl __rt_mixed_cast_float");                            // d0 = left value as a double
    emitter.instruction("str d0, [sp, #24]");                                   // save the left double across the next call
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the right boxed operand
    emitter.instruction("bl __rt_mixed_cast_float");                            // d0 = right value as a double
    emitter.instruction("ldr d1, [sp, #24]");                                   // d1 = left double
    emitter.instruction("fcmp d1, d0");                                         // compare the left and right doubles
    emitter.instruction("cset x0, gt");                                         // 1 when left is greater than right
    emitter.instruction("csinv x0, x0, xzr, ge");                               // keep 1/0 for greater/equal, otherwise -1 for less
    emitter.instruction("b __rt_php_compare_done");                             // return the numeric comparison sign

    // -- lexicographic branch: stringify both operands and byte-compare --
    emitter.label("__rt_php_compare_strcmp");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the left boxed operand
    emitter.instruction("bl __rt_mixed_cast_string");                           // x1=ptr, x2=len for the left string
    emitter.instruction("stp x1, x2, [sp, #16]");                               // save the left string pointer and length
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the right boxed operand
    emitter.instruction("bl __rt_mixed_cast_string");                           // x1=ptr, x2=len for the right string
    emitter.instruction("mov x3, x1");                                          // right pointer into the third strcmp argument
    emitter.instruction("mov x4, x2");                                          // right length into the fourth strcmp argument
    emitter.instruction("ldp x1, x2, [sp, #16]");                               // reload left pointer/length into the first two strcmp arguments
    emitter.instruction("bl __rt_strcmp");                                      // x0 = byte/length difference
    emitter.instruction("cmp x0, #0");                                          // normalize the difference to a comparison sign
    emitter.instruction("cset x0, gt");                                         // 1 when left is greater than right
    emitter.instruction("csinv x0, x0, xzr, ge");                               // keep 1/0 for greater/equal, otherwise -1 for less

    emitter.label("__rt_php_compare_done");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return the three-way comparison sign in x0
}

/// Emits `__rt_php_compare` for the Linux x86_64 target.
///
/// Mirrors the ARM64 logic using System V conventions: the mixed-cell helpers take their
/// boxed pointer in rax, `__rt_str_to_number` takes the string in rax/rdx, and `__rt_strcmp`
/// takes rdi/rsi/rdx/rcx. Returns the normalized -1/0/+1 sign in rax.
fn emit_php_compare_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: php_compare ---");
    emitter.label_global("__rt_php_compare");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across nested calls
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    emitter.instruction("sub rsp, 48");                                         // allocate aligned slots for operands and scratch
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the left boxed operand
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the right boxed operand

    // -- classify the left operand as numeric or non-numeric --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the left boxed operand for unboxing
    emitter.instruction("call __rt_mixed_unbox");                               // rax=tag, rdi=value_lo, rdx=value_hi
    emitter.instruction("cmp rax, 1");                                          // is the left payload a string?
    emitter.instruction("jne __rt_php_compare_l_nonstr_linux_x86_64");          // non-string payloads classify from their tag
    emitter.instruction("mov rax, rdi");                                        // move the unboxed string pointer into the cstr input register
    emitter.instruction("call __rt_str_to_number");                             // numeric-string detection: rax=1 when numeric
    emitter.instruction("jmp __rt_php_compare_l_done_linux_x86_64");            // left numericness computed

    emitter.label("__rt_php_compare_l_nonstr_linux_x86_64");
    emitter.instruction("mov r10, rax");                                        // preserve the unboxed left runtime tag
    emitter.instruction("mov rax, 1");                                          // assume the non-string left payload is numeric
    emitter.instruction("cmp r10, 0");                                          // tag 0 = int (numeric)
    emitter.instruction("je __rt_php_compare_l_done_linux_x86_64");             // ints participate in the numeric branch
    emitter.instruction("cmp r10, 2");                                          // tag 2 = float (numeric)
    emitter.instruction("je __rt_php_compare_l_done_linux_x86_64");             // floats participate in the numeric branch
    emitter.instruction("cmp r10, 3");                                          // tag 3 = bool (numeric via truthiness)
    emitter.instruction("je __rt_php_compare_l_done_linux_x86_64");             // bools fold to 0/1 in the numeric branch
    emitter.instruction("cmp r10, 8");                                          // tag 8 = null (numeric as 0)
    emitter.instruction("je __rt_php_compare_l_done_linux_x86_64");             // null folds to 0 in the numeric branch
    emitter.instruction("xor rax, rax");                                        // arrays/objects compare lexicographically

    emitter.label("__rt_php_compare_l_done_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the left numericness flag

    // -- classify the right operand as numeric or non-numeric --
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the right boxed operand for unboxing
    emitter.instruction("call __rt_mixed_unbox");                               // rax=tag, rdi=value_lo, rdx=value_hi
    emitter.instruction("cmp rax, 1");                                          // is the right payload a string?
    emitter.instruction("jne __rt_php_compare_r_nonstr_linux_x86_64");          // non-string payloads classify from their tag
    emitter.instruction("mov rax, rdi");                                        // move the unboxed string pointer into the cstr input register
    emitter.instruction("call __rt_str_to_number");                             // numeric-string detection: rax=1 when numeric
    emitter.instruction("jmp __rt_php_compare_r_done_linux_x86_64");            // right numericness computed

    emitter.label("__rt_php_compare_r_nonstr_linux_x86_64");
    emitter.instruction("mov r10, rax");                                        // preserve the unboxed right runtime tag
    emitter.instruction("mov rax, 1");                                          // assume the non-string right payload is numeric
    emitter.instruction("cmp r10, 0");                                          // tag 0 = int (numeric)
    emitter.instruction("je __rt_php_compare_r_done_linux_x86_64");             // ints participate in the numeric branch
    emitter.instruction("cmp r10, 2");                                          // tag 2 = float (numeric)
    emitter.instruction("je __rt_php_compare_r_done_linux_x86_64");             // floats participate in the numeric branch
    emitter.instruction("cmp r10, 3");                                          // tag 3 = bool (numeric via truthiness)
    emitter.instruction("je __rt_php_compare_r_done_linux_x86_64");             // bools fold to 0/1 in the numeric branch
    emitter.instruction("cmp r10, 8");                                          // tag 8 = null (numeric as 0)
    emitter.instruction("je __rt_php_compare_r_done_linux_x86_64");             // null folds to 0 in the numeric branch
    emitter.instruction("xor rax, rax");                                        // arrays/objects compare lexicographically

    emitter.label("__rt_php_compare_r_done_linux_x86_64");
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // reload the left numericness flag
    emitter.instruction("test rax, rax");                                       // right numericness flag
    emitter.instruction("je __rt_php_compare_strcmp_linux_x86_64");             // a non-numeric right side forces string comparison
    emitter.instruction("test r11, r11");                                       // left numericness flag
    emitter.instruction("je __rt_php_compare_strcmp_linux_x86_64");             // a non-numeric left side forces string comparison

    // -- numeric branch: compare the two operands as doubles --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the left boxed operand
    emitter.instruction("call __rt_mixed_cast_float");                          // xmm0 = left value as a double
    emitter.instruction("movsd QWORD PTR [rbp - 32], xmm0");                    // save the left double across the next call
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the right boxed operand
    emitter.instruction("call __rt_mixed_cast_float");                          // xmm0 = right value as a double
    emitter.instruction("movsd xmm1, QWORD PTR [rbp - 32]");                    // xmm1 = left double
    emitter.instruction("ucomisd xmm1, xmm0");                                  // compare the left and right doubles
    emitter.instruction("jp __rt_php_compare_gt_linux_x86_64");                 // PHP treats an unordered NaN comparison as greater
    emitter.instruction("ja __rt_php_compare_gt_linux_x86_64");                 // left greater than right
    emitter.instruction("jb __rt_php_compare_lt_linux_x86_64");                 // left less than right
    emitter.instruction("xor rax, rax");                                        // equal operands produce sign 0
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");              // return the equal sign

    emitter.label("__rt_php_compare_gt_linux_x86_64");
    emitter.instruction("mov rax, 1");                                          // greater operands produce sign 1
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");              // return the greater sign

    emitter.label("__rt_php_compare_lt_linux_x86_64");
    emitter.instruction("mov rax, -1");                                         // lesser operands produce sign -1
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");              // return the lesser sign

    // -- lexicographic branch: stringify both operands and byte-compare --
    emitter.label("__rt_php_compare_strcmp_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the left boxed operand
    emitter.instruction("call __rt_mixed_cast_string");                         // rax=ptr, rdx=len for the left string
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the left string pointer
    emitter.instruction("mov QWORD PTR [rbp - 32], rdx");                       // save the left string length
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the right boxed operand
    emitter.instruction("call __rt_mixed_cast_string");                         // rax=ptr, rdx=len for the right string
    emitter.instruction("mov rcx, rdx");                                        // right length into the fourth strcmp argument
    emitter.instruction("mov rdx, rax");                                        // right pointer into the third strcmp argument
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // left pointer into the first strcmp argument
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // left length into the second strcmp argument
    emitter.instruction("call __rt_strcmp");                                    // rax = byte/length difference
    emitter.instruction("test rax, rax");                                       // normalize the difference to a comparison sign
    emitter.instruction("jg __rt_php_compare_strgt_linux_x86_64");              // left greater than right
    emitter.instruction("jl __rt_php_compare_strlt_linux_x86_64");              // left less than right
    emitter.instruction("xor rax, rax");                                        // equal strings produce sign 0
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");              // return the equal sign
    emitter.label("__rt_php_compare_strgt_linux_x86_64");
    emitter.instruction("mov rax, 1");                                          // greater strings produce sign 1
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");              // return the greater sign
    emitter.label("__rt_php_compare_strlt_linux_x86_64");
    emitter.instruction("mov rax, -1");                                         // lesser strings produce sign -1

    emitter.label("__rt_php_compare_done_linux_x86_64");
    emitter.instruction("add rsp, 48");                                         // release the helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the three-way comparison sign in rax
}
