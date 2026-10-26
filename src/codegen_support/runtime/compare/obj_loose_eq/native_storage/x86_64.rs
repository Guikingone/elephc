//! Purpose:
//! Emits x86_64 native comparison policy and storage entry matching.
//!
//! Called from:
//! - `super::emit_native_object_comparison()`.
//!
//! Key details:
//! - The 192-byte frame holds four borrowed array cells, indices and owned read results.
//! - Each key/info read is retired before another entry or return.
//! - Object identity matches keys; info values use the existing depth-aware loose comparator.

use crate::codegen_support::{abi, emit::Emitter};
use crate::codegen_support::runtime::compare::{
    NativeObjectComparisonKind, NATIVE_COMPARISON_DESC_BYTES,
};

/// Emits the native handler, returning -1 for standard comparison or a native boolean.
pub(super) fn emit(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: native object comparison ---");
    emitter.label_global("__rt_obj_native_loose_eq");
    abi::emit_load_symbol_to_reg(emitter, "r10", "_class_gc_desc_count", 0);
    emitter.instruction("mov r9, QWORD PTR [rdi]");                             // inspect the borrowed left receiver class
    emitter.instruction("cmp r9, r10");                                         // bound the dense native descriptor table
    emitter.instruction("jae __rt_onc_standard");                               // unknown classes retain standard property behavior
    abi::emit_symbol_address(emitter, "r11", "_class_cmp_native_descs");
    abi::emit_load_int_immediate(emitter, "r10", NATIVE_COMPARISON_DESC_BYTES);
    emitter.instruction("imul r9, r10");                                        // scale the class id by the native descriptor width
    emitter.instruction("add r11, r9");                                         // locate this class native comparison policy
    emitter.instruction("mov r10, QWORD PTR [r11]");                            // load the native comparison kind
    abi::emit_load_int_immediate(emitter, "r9", NativeObjectComparisonKind::Uncomparable as i64);
    emitter.instruction("cmp r10, r9");                                         // distinct storage subclasses are not comparable
    emitter.instruction("je __rt_onc_uncomparable");                            // return false without visiting subclass properties
    abi::emit_load_int_immediate(emitter, "r9", NativeObjectComparisonKind::ObjectStorage as i64);
    emitter.instruction("cmp r10, r9");                                         // only exact native storage owns entry comparison
    emitter.instruction("jne __rt_onc_standard");                               // all other objects compare their PHP properties
    emitter.instruction("mov rcx, QWORD PTR [r11 + 8]");                        // load the native object-array field offset
    emitter.instruction("mov r8, QWORD PTR [r11 + 16]");                        // load the native info-array field offset
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the native comparison frame
    emitter.instruction("sub rsp, 192");                                        // reserve borrowed cells, indices and owned-read state
    emitter.instruction("mov QWORD PTR [rsp + 96], rdx");                       // preserve the recursion depth before unboxing
    emitter.instruction("mov r9, QWORD PTR [rdi + rcx]");                       // borrow the native array payload from its owning receiver
    emitter.instruction("mov r10, QWORD PTR [rdi + r8]");                       // borrow the native array payload from its owning receiver
    emitter.instruction("mov r11, QWORD PTR [rsi + rcx]");                      // borrow the native array payload from its owning receiver
    emitter.instruction("mov rax, QWORD PTR [rsi + r8]");                       // borrow the native array payload from its owning receiver
    emitter.instruction("mov QWORD PTR [rsp + 0], 4");                          // tag four denotes a borrowed indexed array cell
    emitter.instruction("mov QWORD PTR [rsp + 8], r9");                         // store the borrowed array payload pointer
    emitter.instruction("mov QWORD PTR [rsp + 16], 0");                         // the array cell carries no high payload word
    emitter.instruction("mov QWORD PTR [rsp + 24], 4");                         // tag four denotes a borrowed indexed array cell
    emitter.instruction("mov QWORD PTR [rsp + 32], r10");                       // store the borrowed array payload pointer
    emitter.instruction("mov QWORD PTR [rsp + 40], 0");                         // the array cell carries no high payload word
    emitter.instruction("mov QWORD PTR [rsp + 48], 4");                         // tag four denotes a borrowed indexed array cell
    emitter.instruction("mov QWORD PTR [rsp + 56], r11");                       // store the borrowed array payload pointer
    emitter.instruction("mov QWORD PTR [rsp + 64], 0");                         // the array cell carries no high payload word
    emitter.instruction("mov QWORD PTR [rsp + 72], 4");                         // tag four denotes a borrowed indexed array cell
    emitter.instruction("mov QWORD PTR [rsp + 80], rax");                       // store the borrowed array payload pointer
    emitter.instruction("mov QWORD PTR [rsp + 88], 0");                         // the array cell carries no high payload word
    emitter.instruction("lea rax, [rsp + 0]");                                  // normalize empty or sentinel arrays in the borrowed cell
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("mov QWORD PTR [rsp + 0], rax");                        // store the normalized runtime tag
    emitter.instruction("mov QWORD PTR [rsp + 8], rdi");                        // store the normalized array payload
    emitter.instruction("mov QWORD PTR [rsp + 16], rdx");                       // store the normalized high payload word
    emitter.instruction("lea rax, [rsp + 24]");                                 // normalize empty or sentinel arrays in the borrowed cell
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("mov QWORD PTR [rsp + 24], rax");                       // store the normalized runtime tag
    emitter.instruction("mov QWORD PTR [rsp + 32], rdi");                       // store the normalized array payload
    emitter.instruction("mov QWORD PTR [rsp + 40], rdx");                       // store the normalized high payload word
    emitter.instruction("lea rax, [rsp + 48]");                                 // normalize empty or sentinel arrays in the borrowed cell
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("mov QWORD PTR [rsp + 48], rax");                       // store the normalized runtime tag
    emitter.instruction("mov QWORD PTR [rsp + 56], rdi");                       // store the normalized array payload
    emitter.instruction("mov QWORD PTR [rsp + 64], rdx");                       // store the normalized high payload word
    emitter.instruction("lea rax, [rsp + 72]");                                 // normalize empty or sentinel arrays in the borrowed cell
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("mov QWORD PTR [rsp + 72], rax");                       // store the normalized runtime tag
    emitter.instruction("mov QWORD PTR [rsp + 80], rdi");                       // store the normalized array payload
    emitter.instruction("mov QWORD PTR [rsp + 88], rdx");                       // store the normalized high payload word
    emitter.instruction("mov rax, rsp");                                        // pass the borrowed left object-array cell
    abi::emit_call_label(emitter, "__rt_mixed_count");
    emitter.instruction("mov QWORD PTR [rsp + 104], rax");                      // save the left entry count
    emitter.instruction("lea rax, [rsp + 48]");                                 // pass the borrowed right object-array cell
    abi::emit_call_label(emitter, "__rt_mixed_count");
    emitter.instruction("cmp rax, QWORD PTR [rsp + 104]");                      // storage cardinalities must agree
    emitter.instruction("jne __rt_onc_false");                                  // different entry counts are unequal
    emitter.instruction("test rax, rax");                                       // are both exact storage instances empty
    emitter.instruction("jz __rt_onc_true");                                    // empty exact storage instances are equal
    emitter.instruction("mov QWORD PTR [rsp + 112], 0");                        // start with the first left entry
    emitter.label("__rt_onc_left");
    emitter.instruction("mov rsi, QWORD PTR [rsp + 112]");                      // reload the left entry ordinal
    emitter.instruction("cmp rsi, QWORD PTR [rsp + 104]");                      // has every left entry matched
    emitter.instruction("jge __rt_onc_true");                                   // all keys and info values matched
    emitter.instruction("mov rdi, rsp");                                        // read from the borrowed left object array
    emitter.instruction("mov rdx, -1");                                         // the ordinal is an integer key
    emitter.instruction("xor ecx, ecx");                                        // comparison reads never emit missing-key warnings
    abi::emit_call_label(emitter, "__rt_mixed_array_get");
    emitter.instruction("mov QWORD PTR [rsp + 128], rax");                      // preserve the owned key read until it is unboxed
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("mov QWORD PTR [rsp + 160], rdi");                      // retain only borrowed object identity for matching
    emitter.instruction("mov rax, QWORD PTR [rsp + 128]");                      // reload the owned left key read
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    emitter.instruction("mov r9, QWORD PTR [rsp + 112]");                       // try the matching ordinal before a full key search
    emitter.instruction("mov QWORD PTR [rsp + 120], r9");                       // save the candidate right ordinal
    emitter.instruction("mov QWORD PTR [rsp + 168], 1");                        // mark the same-order fast candidate
    emitter.label("__rt_onc_right");
    emitter.instruction("mov rsi, QWORD PTR [rsp + 120]");                      // reload the candidate right ordinal
    emitter.instruction("cmp rsi, QWORD PTR [rsp + 104]");                      // has the full key search exhausted its candidates
    emitter.instruction("jge __rt_onc_false");                                  // a missing object key makes storage unequal
    emitter.instruction("lea rdi, [rsp + 48]");                                 // read from the borrowed right object array
    emitter.instruction("mov rdx, -1");                                         // the candidate ordinal is an integer key
    emitter.instruction("xor ecx, ecx");                                        // comparison reads remain quiet
    abi::emit_call_label(emitter, "__rt_mixed_array_get");
    emitter.instruction("mov QWORD PTR [rsp + 128], rax");                      // preserve the owned right key read
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("cmp rdi, QWORD PTR [rsp + 160]");                      // default storage keys compare by object identity
    emitter.instruction("sete r9b");                                            // remember whether this key matched
    emitter.instruction("movzx r9, r9b");                                       // normalize the saved comparison result
    emitter.instruction("mov QWORD PTR [rsp + 152], r9");                       // preserve the match across release
    emitter.instruction("mov rax, QWORD PTR [rsp + 128]");                      // reload the owned right key read
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    emitter.instruction("cmp QWORD PTR [rsp + 152], 0");                        // reload the candidate match
    emitter.instruction("jne __rt_onc_info");                                   // matching identities select corresponding info values
    emitter.instruction("cmp QWORD PTR [rsp + 168], 0");                        // was this only the same-order fast candidate
    emitter.instruction("je __rt_onc_next_candidate");                          // continue a search already in progress
    emitter.instruction("mov QWORD PTR [rsp + 168], 0");                        // switch to the complete order-independent search
    emitter.instruction("mov QWORD PTR [rsp + 120], 0");                        // restart the candidate search at ordinal zero
    emitter.instruction("jmp __rt_onc_right");                                  // look up the object independently of insertion order
    emitter.label("__rt_onc_next_candidate");
    emitter.instruction("add QWORD PTR [rsp + 120], 1");                        // advance to the next right entry
    emitter.instruction("jmp __rt_onc_right");                                  // continue matching this left object key
    emitter.label("__rt_onc_info");
    emitter.instruction("lea rdi, [rsp + 24]");                                 // select the borrowed native info array
    emitter.instruction("mov rsi, QWORD PTR [rsp + 112]");                      // select the matching object info ordinal
    emitter.instruction("mov rdx, -1");                                         // info ordinals are integer keys
    emitter.instruction("xor ecx, ecx");                                        // read the info value without diagnostics
    abi::emit_call_label(emitter, "__rt_mixed_array_get");
    emitter.instruction("mov QWORD PTR [rsp + 136], rax");                      // preserve the owned info read
    emitter.instruction("lea rdi, [rsp + 72]");                                 // select the borrowed native info array
    emitter.instruction("mov rsi, QWORD PTR [rsp + 120]");                      // select the matching object info ordinal
    emitter.instruction("mov rdx, -1");                                         // info ordinals are integer keys
    emitter.instruction("xor ecx, ecx");                                        // read the info value without diagnostics
    abi::emit_call_label(emitter, "__rt_mixed_array_get");
    emitter.instruction("mov QWORD PTR [rsp + 144], rax");                      // preserve the owned info read
    emitter.instruction("mov rdi, QWORD PTR [rsp + 136]");                      // load the owned left info value
    emitter.instruction("mov rsi, QWORD PTR [rsp + 144]");                      // load the owned right info value
    emitter.instruction("mov rdx, QWORD PTR [rsp + 96]");                       // restore the shared comparison recursion depth
    abi::emit_call_label(emitter, "__rt_mixed_loose_eq_d");
    emitter.instruction("mov QWORD PTR [rsp + 152], rax");                      // preserve the info comparison result
    emitter.instruction("mov rax, QWORD PTR [rsp + 136]");                      // reload an owned info read
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    emitter.instruction("mov rax, QWORD PTR [rsp + 144]");                      // reload an owned info read
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    emitter.instruction("cmp QWORD PTR [rsp + 152], 0");                        // reload the info comparison result
    emitter.instruction("je __rt_onc_false");                                   // unequal info values end storage comparison
    emitter.instruction("add QWORD PTR [rsp + 112], 1");                        // advance after matching both key and info
    emitter.instruction("jmp __rt_onc_left");                                   // continue the native storage walk
    emitter.label("__rt_onc_true");
    emitter.instruction("mov eax, 1");                                          // the exact native storage contents are equal
    emitter.instruction("jmp __rt_onc_done");                                   // return after all owned reads have been retired
    emitter.label("__rt_onc_false");
    emitter.instruction("xor eax, eax");                                        // native storage contents differ
    emitter.label("__rt_onc_done");
    emitter.instruction("add rsp, 192");                                        // release only the borrowed cells and scalar state
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the native comparison boolean
    emitter.label("__rt_onc_standard");
    emitter.instruction("mov rax, -1");                                         // delegate to ordinary PHP property comparison
    emitter.instruction("ret");                                                 // the standard path never allocated a native frame
    emitter.label("__rt_onc_uncomparable");
    emitter.instruction("xor eax, eax");                                        // distinct storage subclass instances are uncomparable
    emitter.instruction("ret");                                                 // return false without reading internal or user properties
}
