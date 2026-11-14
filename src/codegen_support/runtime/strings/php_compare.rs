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
//! - The stringified operands of the lexicographic branch are owned copies for string payloads
//!   and are released after the comparison (`__rt_decref_any` skips scratch and static strings).

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
    emit_php_compare_bool_pairs(emitter);
    if emitter.target.arch == Arch::X86_64 {
        emit_php_compare_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: php_compare ---");
    emitter.label_global("__rt_php_compare_core");

    // -- set up helper frame and save both boxed operands --
    emitter.instruction("sub sp, sp, #96");                                     // allocate frame slots for operands, unboxed payloads and scratch
    emitter.instruction("stp x29, x30, [sp, #80]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #80");                                    // establish the helper frame pointer
    emitter.instruction("stp x0, x1, [sp, #0]");                                // save left operand at +0 and right operand at +8

    // -- classify the left operand as numeric or non-numeric --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the left boxed operand
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0=tag, x1=value_lo, x2=value_hi
    emitter.instruction("stp x0, x1, [sp, #32]");                               // keep the left tag and low payload word
    emitter.instruction("str x2, [sp, #48]");                                   // keep the left high payload word
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
    emitter.instruction("stp x0, x1, [sp, #56]");                               // keep the right tag and low payload word
    emitter.instruction("str x2, [sp, #72]");                                   // keep the right high payload word
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
    emitter.instruction("str x0, [sp, #24]");                                   // keep the right numericness flag across the type rules
    // -- PHP 8 type rules that precede the numeric/string split --
    emitter.instruction("ldr x11, [sp, #32]");                                  // left tag
    emitter.instruction("ldr x12, [sp, #56]");                                  // right tag
    emitter.instruction("orr x13, x11, x12");                                   // both ints exactly when the tags OR to zero
    emitter.instruction("cbnz x13, __rt_php_compare_not_ints");                 // not two ints
    emitter.instruction("ldr x13, [sp, #40]");                                  // left int
    emitter.instruction("ldr x14, [sp, #64]");                                  // right int
    emitter.instruction("cmp x13, x14");                                        // exact integer comparison (no double rounding)
    emitter.instruction("cset x0, gt");                                         // 1 when left is greater than right
    emitter.instruction("csinv x0, x0, xzr, ge");                               // keep 1/0 for greater/equal, otherwise -1 for less
    emitter.instruction("b __rt_php_compare_done");                             // return the integer comparison sign
    emitter.label("__rt_php_compare_not_ints");
    emitter.instruction("cmp x11, #3");                                         // a bool operand compares both sides as bools
    emitter.instruction("b.eq __rt_php_compare_bools");
    emitter.instruction("cmp x12, #3");
    emitter.instruction("b.eq __rt_php_compare_bools");
    emitter.instruction("cmp x11, #8");                                         // left null?
    emitter.instruction("b.ne __rt_php_compare_left_not_null");
    emitter.instruction("cmp x12, #8");                                         // null <=> null
    emitter.instruction("b.eq __rt_php_compare_zero");
    emitter.instruction("cmp x12, #1");                                         // null <=> string compares "" with the string
    emitter.instruction("b.ne __rt_php_compare_bools");                         // null <=> anything else compares as bools
    emitter.instruction("ldr x9, [sp, #72]");                                   // right string length
    emitter.instruction("cmp x9, #0");                                          // "" equals only the empty string
    emitter.instruction("csetm x0, ne");                                        // otherwise "" sorts first
    emitter.instruction("b __rt_php_compare_done");
    emitter.label("__rt_php_compare_left_not_null");
    emitter.instruction("cmp x12, #8");                                         // right null?
    emitter.instruction("b.ne __rt_php_compare_typed_done");
    emitter.instruction("cmp x11, #1");                                         // string <=> null compares the string with ""
    emitter.instruction("b.ne __rt_php_compare_bools");                         // anything else <=> null compares as bools
    emitter.instruction("ldr x9, [sp, #48]");                                   // left string length
    emitter.instruction("cmp x9, #0");                                          // "" equals only the empty string
    emitter.instruction("cset x0, ne");                                         // otherwise the string sorts after ""
    emitter.instruction("b __rt_php_compare_done");
    emitter.label("__rt_php_compare_bools");
    emitter.instruction("ldr x0, [sp, #0]");                                    // left boxed operand
    emitter.instruction("bl __rt_mixed_cast_bool");                             // left truthiness
    emitter.instruction("str x0, [sp, #16]");                                   // keep it across the next call
    emitter.instruction("ldr x0, [sp, #8]");                                    // right boxed operand
    emitter.instruction("bl __rt_mixed_cast_bool");                             // right truthiness
    emitter.instruction("ldr x9, [sp, #16]");                                   // left truthiness
    emitter.instruction("sub x0, x9, x0");                                      // false < true: the difference is the sign
    emitter.instruction("b __rt_php_compare_done");
    emitter.label("__rt_php_compare_one");
    emitter.instruction("mov x0, #1");                                          // uncomparable operands
    emitter.instruction("b __rt_php_compare_done");
    emitter.label("__rt_php_compare_zero");
    emitter.instruction("mov x0, #0");                                          // equal operands
    emitter.instruction("b __rt_php_compare_done");
    emitter.label("__rt_php_compare_typed_done");
    // -- arrays: an array is greater than any non-array except an object; two arrays compare by size first --
    emitter.instruction("sub x13, x11, #4");                                    // tags 4/5 are indexed/associative arrays
    emitter.instruction("cmp x13, #1");
    emitter.instruction("cset x13, ls");                                        // x13 = left is an array
    emitter.instruction("sub x14, x12, #4");
    emitter.instruction("cmp x14, #1");
    emitter.instruction("cset x14, ls");                                        // x14 = right is an array
    emitter.instruction("orr x9, x13, x14");
    emitter.instruction("cbz x9, __rt_php_compare_not_arrays");                 // neither side is an array
    emitter.instruction("sub x0, x13, x14");                                    // one array: it is the greater side
    emitter.instruction("cmp x13, x14");
    emitter.instruction("b.eq 4f");                                             // two arrays
    emitter.instruction("cmp x11, #6");                                         // an object is greater than an array
    emitter.instruction("ccmp x12, #6, #4, ne");                                // (either side)
    emitter.instruction("cneg x0, x0, eq");                                     // object side wins: flip the sign
    emitter.instruction("b __rt_php_compare_done");
    emitter.label("4");
    emitter.instruction("ldr x9, [sp, #40]");                                   // two arrays: left container
    emitter.instruction("cbz x9, 2f");                                          // a null container counts as empty
    emitter.instruction("ldr x9, [x9]");                                        // left element count
    emitter.label("2");
    emitter.instruction("ldr x10, [sp, #64]");                                  // right container
    emitter.instruction("cbz x10, 3f");                                         // a null container counts as empty
    emitter.instruction("ldr x10, [x10]");                                      // right element count
    emitter.label("3");
    emitter.instruction("cmp x9, x10");                                         // the smaller array sorts first
    emitter.instruction("cset x0, gt");
    emitter.instruction("csinv x0, x0, xzr, ge");
    emitter.instruction("b __rt_php_compare_done");                             // equal sizes keep the size verdict (0)
    emitter.label("__rt_php_compare_not_arrays");
    // -- a NAN operand is uncomparable: PHP answers 1 --
    for (tag_slot, lo_slot) in [(32, 40), (56, 64)] {
        emitter.instruction(&format!("ldr x9, [sp, #{}]", tag_slot));          // operand tag
        emitter.instruction("cmp x9, #2");                                      // a float?
        emitter.instruction("b.ne 1f");
        emitter.instruction(&format!("ldr x9, [sp, #{}]", lo_slot));           // its bits
        emitter.instruction("lsl x9, x9, #1");                                  // drop the sign
        emitter.instruction("mov x10, #0xffe0000000000000");                    // exponent all ones, zero mantissa (shifted)
        emitter.instruction("cmp x9, x10");                                     // NAN has a non-zero mantissa
        emitter.instruction("b.hi __rt_php_compare_one");
        emitter.label("1");
    }
    emitter.instruction("ldr x0, [sp, #24]");                                   // restore the right numericness flag
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
    emitter.instruction("cset x0, ne");                                         // 1 when greater or unordered (PHP: NAN compares greater)
    emitter.instruction("csinv x0, x0, xzr, pl");                               // -1 only for an ordered less-than (N flag set)
    emitter.instruction("b __rt_php_compare_done");                             // return the numeric comparison sign

    // -- lexicographic branch: stringify both operands and byte-compare --
    emitter.label("__rt_php_compare_strcmp");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the left boxed operand
    emitter.instruction("bl __rt_mixed_cast_string");                           // x1=ptr, x2=len for the left string
    emitter.instruction("stp x1, x2, [sp, #16]");                               // save the left string pointer and length
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the right boxed operand
    emitter.instruction("bl __rt_mixed_cast_string");                           // x1=ptr, x2=len for the right string
    emitter.instruction("str x1, [sp, #8]");                                    // keep the right string pointer for its release
    emitter.instruction("mov x3, x1");                                          // right pointer into the third strcmp argument
    emitter.instruction("mov x4, x2");                                          // right length into the fourth strcmp argument
    emitter.instruction("ldp x1, x2, [sp, #16]");                               // reload left pointer/length into the first two strcmp arguments
    emitter.instruction("bl __rt_strcmp");                                      // x0 = byte/length difference
    emitter.instruction("cmp x0, #0");                                          // normalize the difference to a comparison sign
    emitter.instruction("cset x0, gt");                                         // 1 when left is greater than right
    emitter.instruction("csinv x0, x0, xzr, ge");                               // keep 1/0 for greater/equal, otherwise -1 for less
    emitter.instruction("str x0, [sp, #0]");                                    // keep the sign across the releases
    // String payloads were stringified into owned persisted copies: release both.
    emitter.instruction("ldr x0, [sp, #16]");                                   // left string
    emitter.instruction("bl __rt_decref_any");                                  // release it (scratch/static strings are skipped)
    emitter.instruction("ldr x0, [sp, #8]");                                    // right string
    emitter.instruction("bl __rt_decref_any");                                  // release it (scratch/static strings are skipped)
    emitter.instruction("ldr x0, [sp, #0]");                                    // comparison sign

    emitter.label("__rt_php_compare_done");
    emitter.instruction("ldp x29, x30, [sp, #80]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #96");                                     // release the helper frame
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
    emitter.label_global("__rt_php_compare_core");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across nested calls
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    emitter.instruction("sub rsp, 96");                                         // allocate aligned slots for operands, unboxed payloads and scratch
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the left boxed operand
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the right boxed operand

    // -- classify the left operand as numeric or non-numeric --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the left boxed operand for unboxing
    emitter.instruction("call __rt_mixed_unbox");                               // rax=tag, rdi=value_lo, rdx=value_hi
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // keep the left tag
    emitter.instruction("mov QWORD PTR [rbp - 56], rdi");                       // keep the left low payload word
    emitter.instruction("mov QWORD PTR [rbp - 64], rdx");                       // keep the left high payload word
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
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // keep the right tag
    emitter.instruction("mov QWORD PTR [rbp - 80], rdi");                       // keep the right low payload word
    emitter.instruction("mov QWORD PTR [rbp - 88], rdx");                       // keep the right high payload word
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
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // keep the right numericness flag across the type rules
    // -- PHP 8 type rules that precede the numeric/string split --
    emitter.instruction("mov r10, QWORD PTR [rbp - 48]");                       // left tag
    emitter.instruction("mov r11, QWORD PTR [rbp - 72]");                       // right tag
    emitter.instruction("mov rcx, r10");                                        // both ints exactly when the tags OR to zero
    emitter.instruction("or rcx, r11");
    emitter.instruction("jnz __rt_php_compare_not_ints_linux_x86_64");          // not two ints
    emitter.instruction("mov rcx, QWORD PTR [rbp - 56]");                       // left int
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 80]");                       // exact integer comparison (no double rounding)
    emitter.instruction("jg __rt_php_compare_gt_linux_x86_64");
    emitter.instruction("jl __rt_php_compare_lt_linux_x86_64");
    emitter.instruction("xor eax, eax");                                        // equal ints
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");
    emitter.label("__rt_php_compare_not_ints_linux_x86_64");
    emitter.instruction("cmp r10, 3");                                          // a bool operand compares both sides as bools
    emitter.instruction("je __rt_php_compare_bools_linux_x86_64");
    emitter.instruction("cmp r11, 3");
    emitter.instruction("je __rt_php_compare_bools_linux_x86_64");
    emitter.instruction("cmp r10, 8");                                          // left null?
    emitter.instruction("jne __rt_php_compare_left_not_null_linux_x86_64");
    emitter.instruction("cmp r11, 8");                                          // null <=> null
    emitter.instruction("je __rt_php_compare_zero_linux_x86_64");
    emitter.instruction("cmp r11, 1");                                          // null <=> string compares "" with the string
    emitter.instruction("jne __rt_php_compare_bools_linux_x86_64");             // null <=> anything else compares as bools
    emitter.instruction("cmp QWORD PTR [rbp - 88], 0");                         // "" equals only the empty string
    emitter.instruction("jne __rt_php_compare_lt_linux_x86_64");                // otherwise "" sorts first
    emitter.instruction("jmp __rt_php_compare_zero_linux_x86_64");
    emitter.label("__rt_php_compare_left_not_null_linux_x86_64");
    emitter.instruction("cmp r11, 8");                                          // right null?
    emitter.instruction("jne __rt_php_compare_typed_done_linux_x86_64");
    emitter.instruction("cmp r10, 1");                                          // string <=> null compares the string with ""
    emitter.instruction("jne __rt_php_compare_bools_linux_x86_64");             // anything else <=> null compares as bools
    emitter.instruction("cmp QWORD PTR [rbp - 64], 0");                         // "" equals only the empty string
    emitter.instruction("jne __rt_php_compare_gt_linux_x86_64");                // otherwise the string sorts after ""
    emitter.instruction("jmp __rt_php_compare_zero_linux_x86_64");
    emitter.label("__rt_php_compare_bools_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // left boxed operand
    emitter.instruction("mov rdi, rax");
    emitter.instruction("call __rt_mixed_cast_bool");                           // left truthiness
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // keep it across the next call
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // right boxed operand
    emitter.instruction("mov rdi, rax");
    emitter.instruction("call __rt_mixed_cast_bool");                           // right truthiness
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                       // left truthiness
    emitter.instruction("sub rcx, rax");                                        // false < true: the difference is the sign
    emitter.instruction("mov rax, rcx");
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");
    emitter.label("__rt_php_compare_zero_linux_x86_64");
    emitter.instruction("xor eax, eax");                                        // equal operands
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");
    emitter.label("__rt_php_compare_typed_done_linux_x86_64");
    // -- arrays: an array is greater than any non-array except an object; two arrays compare by size first --
    emitter.instruction("lea rcx, [r10 - 4]");                                  // tags 4/5 are indexed/associative arrays
    emitter.instruction("cmp rcx, 1");
    emitter.instruction("setbe cl");
    emitter.instruction("movzx ecx, cl");                                       // rcx = left is an array
    emitter.instruction("lea rdx, [r11 - 4]");
    emitter.instruction("cmp rdx, 1");
    emitter.instruction("setbe dl");
    emitter.instruction("movzx edx, dl");                                       // rdx = right is an array
    emitter.instruction("mov rax, rcx");
    emitter.instruction("or rax, rdx");
    emitter.instruction("jz __rt_php_compare_not_arrays_linux_x86_64");         // neither side is an array
    emitter.instruction("mov rax, rcx");
    emitter.instruction("sub rax, rdx");                                        // one array: it is the greater side
    emitter.instruction("cmp rcx, rdx");
    emitter.instruction("je 4f");                                               // two arrays
    emitter.instruction("cmp r10, 6");                                          // an object is greater than an array
    emitter.instruction("je 5f");
    emitter.instruction("cmp r11, 6");
    emitter.instruction("jne __rt_php_compare_done_linux_x86_64");
    emitter.label("5");
    emitter.instruction("neg rax");                                             // object side wins: flip the sign
    emitter.instruction("jmp __rt_php_compare_done_linux_x86_64");
    emitter.label("4");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 56]");                       // two arrays: left container
    emitter.instruction("test rcx, rcx");                                       // a null container counts as empty
    emitter.instruction("jz 2f");
    emitter.instruction("mov rcx, QWORD PTR [rcx]");                            // left element count
    emitter.label("2");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 80]");                       // right container
    emitter.instruction("test rdx, rdx");                                       // a null container counts as empty
    emitter.instruction("jz 3f");
    emitter.instruction("mov rdx, QWORD PTR [rdx]");                            // right element count
    emitter.label("3");
    emitter.instruction("cmp rcx, rdx");                                        // the smaller array sorts first
    emitter.instruction("jg __rt_php_compare_gt_linux_x86_64");
    emitter.instruction("jl __rt_php_compare_lt_linux_x86_64");
    emitter.instruction("jmp __rt_php_compare_zero_linux_x86_64");              // equal sizes keep the size verdict (0)
    emitter.label("__rt_php_compare_not_arrays_linux_x86_64");
    // -- a NAN operand is uncomparable: PHP answers 1 --
    for (tag_slot, lo_slot) in [(48, 56), (72, 80)] {
        emitter.instruction(&format!("cmp QWORD PTR [rbp - {}], 2", tag_slot)); // a float operand?
        emitter.instruction("jne 1f");
        emitter.instruction(&format!("mov rcx, QWORD PTR [rbp - {}]", lo_slot)); // its bits
        emitter.instruction("shl rcx, 1");                                      // drop the sign
        emitter.instruction("mov rdx, 0xffe0000000000000");                     // exponent all ones, zero mantissa (shifted)
        emitter.instruction("cmp rcx, rdx");                                    // NAN has a non-zero mantissa
        emitter.instruction("ja __rt_php_compare_gt_linux_x86_64");
        emitter.label("1");
    }
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // restore the right numericness flag
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
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // keep the right string pointer for its release
    emitter.instruction("mov rcx, rdx");                                        // right length into the fourth strcmp argument
    emitter.instruction("mov rdx, rax");                                        // right pointer into the third strcmp argument
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // left pointer into the first strcmp argument
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // left length into the second strcmp argument
    emitter.instruction("call __rt_strcmp");                                    // rax = byte/length difference
    emitter.instruction("xor ecx, ecx");                                        // equal strings produce sign 0
    emitter.instruction("test rax, rax");                                       // normalize the difference to a comparison sign
    emitter.instruction("setg cl");                                             // greater strings produce sign 1
    emitter.instruction("mov rdx, -1");                                         // lesser strings produce sign -1
    emitter.instruction("cmovl rcx, rdx");                                      // select -1 for lesser strings
    emitter.instruction("mov QWORD PTR [rbp - 40], rcx");                       // keep the sign across the releases
    // String payloads were stringified into owned persisted copies: release both.
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // left string
    emitter.instruction("call __rt_decref_any");                                // release it (scratch/static strings are skipped)
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // right string
    emitter.instruction("call __rt_decref_any");                                // release it (scratch/static strings are skipped)
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // comparison sign

    emitter.label("__rt_php_compare_done_linux_x86_64");
    emitter.instruction("add rsp, 96");                                         // release the helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the three-way comparison sign in rax
}

/// Emits the `__rt_php_compare` entry: PHP 8 compares a bool with anything, and `null` with an
/// int, float or `null`, as two booleans (`null < -1` is `false < true`); every other pair goes
/// to `__rt_php_compare_core`. Same register contract as the core.
fn emit_php_compare_bool_pairs(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: php_compare (bool/null pairs) ---");
    emitter.label_global("__rt_php_compare");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("stp x29, x30, [sp, #-48]!");                   // frame for the tag probes
            emitter.instruction("mov x29, sp");
            emitter.instruction("stp x0, x1, [sp, #16]");                       // keep both boxes
            emitter.instruction("bl __rt_mixed_unbox");                         // left tag
            emitter.instruction("str x0, [sp, #32]");
            emitter.instruction("ldr x0, [sp, #24]");
            emitter.instruction("bl __rt_mixed_unbox");                         // right tag in x0
            emitter.instruction("ldr x9, [sp, #32]");                           // left tag in x9
            emitter.instruction("cmp x9, #3");                                  // a bool operand: compare as bools
            emitter.instruction("b.eq 5f");
            emitter.instruction("cmp x0, #3");
            emitter.instruction("b.eq 5f");
            emitter.instruction("cmp x9, #8");                                  // left null?
            emitter.instruction("b.ne 2f");
            emitter.instruction("cmp x0, #0");                                  // with an int
            emitter.instruction("b.eq 5f");
            emitter.instruction("cmp x0, #2");                                  // with a float
            emitter.instruction("b.eq 5f");
            emitter.instruction("cmp x0, #8");                                  // with null
            emitter.instruction("b.eq 5f");
            emitter.instruction("b 9f");
            emitter.label("2");
            emitter.instruction("cmp x0, #8");                                  // right null?
            emitter.instruction("b.ne 9f");
            emitter.instruction("cmp x9, #0");                                  // with an int
            emitter.instruction("b.eq 5f");
            emitter.instruction("cmp x9, #2");                                  // with a float
            emitter.instruction("b.eq 5f");
            emitter.instruction("b 9f");
            emitter.label("5");
            emitter.instruction("ldr x0, [sp, #16]");
            emitter.instruction("bl __rt_mixed_cast_bool");                     // left truthiness
            emitter.instruction("str x0, [sp, #32]");
            emitter.instruction("ldr x0, [sp, #24]");
            emitter.instruction("bl __rt_mixed_cast_bool");                     // right truthiness
            emitter.instruction("ldr x9, [sp, #32]");
            emitter.instruction("cmp x9, x0");
            emitter.instruction("cset x10, gt");
            emitter.instruction("cset x11, lt");
            emitter.instruction("sub x0, x10, x11");                            // -1 / 0 / +1
            emitter.instruction("ldp x29, x30, [sp], #48");
            emitter.instruction("ret");
            emitter.label("9");
            emitter.instruction("ldp x0, x1, [sp, #16]");                       // restore both boxes
            emitter.instruction("ldp x29, x30, [sp], #48");
            emitter.instruction("b __rt_php_compare_core");                     // general PHP 8 rules
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");                                    // frame for the tag probes
            emitter.instruction("mov rbp, rsp");
            emitter.instruction("sub rsp, 32");
            emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                // keep both boxes
            emitter.instruction("mov QWORD PTR [rbp - 16], rsi");
            emitter.instruction("mov rax, rdi");
            emitter.instruction("call __rt_mixed_unbox");                       // left tag
            emitter.instruction("mov QWORD PTR [rbp - 24], rax");
            emitter.instruction("mov rax, QWORD PTR [rbp - 16]");
            emitter.instruction("call __rt_mixed_unbox");                       // right tag in rax
            emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");               // left tag in rcx
            emitter.instruction("cmp rcx, 3");                                  // a bool operand: compare as bools
            emitter.instruction("je 5f");
            emitter.instruction("cmp rax, 3");
            emitter.instruction("je 5f");
            emitter.instruction("cmp rcx, 8");                                  // left null?
            emitter.instruction("jne 2f");
            emitter.instruction("cmp rax, 0");                                  // with an int
            emitter.instruction("je 5f");
            emitter.instruction("cmp rax, 2");                                  // with a float
            emitter.instruction("je 5f");
            emitter.instruction("cmp rax, 8");                                  // with null
            emitter.instruction("je 5f");
            emitter.instruction("jmp 9f");
            emitter.label("2");
            emitter.instruction("cmp rax, 8");                                  // right null?
            emitter.instruction("jne 9f");
            emitter.instruction("cmp rcx, 0");                                  // with an int
            emitter.instruction("je 5f");
            emitter.instruction("cmp rcx, 2");                                  // with a float
            emitter.instruction("je 5f");
            emitter.instruction("jmp 9f");
            emitter.label("5");
            emitter.instruction("mov rax, QWORD PTR [rbp - 8]");
            emitter.instruction("mov rdi, rax");
            emitter.instruction("call __rt_mixed_cast_bool");                   // left truthiness
            emitter.instruction("mov QWORD PTR [rbp - 24], rax");
            emitter.instruction("mov rax, QWORD PTR [rbp - 16]");
            emitter.instruction("mov rdi, rax");
            emitter.instruction("call __rt_mixed_cast_bool");                   // right truthiness
            emitter.instruction("mov r8, QWORD PTR [rbp - 24]");
            emitter.instruction("xor ecx, ecx");
            emitter.instruction("xor edx, edx");
            emitter.instruction("cmp r8, rax");
            emitter.instruction("setg cl");
            emitter.instruction("setl dl");
            emitter.instruction("mov rax, rcx");
            emitter.instruction("sub rax, rdx");                                // -1 / 0 / +1
            emitter.instruction("leave");
            emitter.instruction("ret");
            emitter.label("9");
            emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                // restore both boxes
            emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");
            emitter.instruction("leave");
            emitter.instruction("jmp __rt_php_compare_core");                   // general PHP 8 rules
        }
    }
}
