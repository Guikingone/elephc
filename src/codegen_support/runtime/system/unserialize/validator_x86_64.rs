//! Purpose:
//! Emits the x86_64 allocation-free grammar preflight for serialized input.
//!
//! Called from:
//! - `super::emit_unserialize()` after the public entry wrapper and before the mutating decoder.
//!
//! Key details:
//! - Every cursor, delimiter, length, and recursive child is bounded before allocation or hooks.

use crate::codegen_support::emit::Emitter;

/// Emits the x86_64 allocation-free grammar preflight used before decoding.
///
/// This mirrors the AArch64 validator and rejects malformed cursors, overflowing
/// decimal fields, and unterminated containers before the mutating parser runs.
pub(super) fn emit(emitter: &mut Emitter) {
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
