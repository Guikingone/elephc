//! Purpose:
//! Emits AArch64 native comparison policy and storage entry matching.
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
    abi::emit_load_symbol_to_reg(emitter, "x10", "_class_gc_desc_count", 0);
    emitter.instruction("ldr x9, [x0]");                                        // inspect the borrowed left receiver class
    emitter.instruction("cmp x9, x10");                                         // bound the dense native descriptor table
    emitter.instruction("b.hs __rt_onc_standard");                              // unknown classes retain standard property behavior
    abi::emit_symbol_address(emitter, "x11", "_class_cmp_native_descs");
    abi::emit_load_int_immediate(emitter, "x10", NATIVE_COMPARISON_DESC_BYTES);
    emitter.instruction("mul x9, x9, x10");                                     // scale the runtime class id by the native descriptor width
    emitter.instruction("add x11, x11, x9");                                    // locate this class native comparison policy
    emitter.instruction("ldr x12, [x11]");                                      // load the native comparison kind
    abi::emit_load_int_immediate(emitter, "x13", NativeObjectComparisonKind::Uncomparable as i64);
    emitter.instruction("cmp x12, x13");                                        // distinct storage subclasses are not comparable
    emitter.instruction("b.eq __rt_onc_uncomparable");                          // return false without visiting subclass properties
    abi::emit_load_int_immediate(emitter, "x13", NativeObjectComparisonKind::ObjectStorage as i64);
    emitter.instruction("cmp x12, x13");                                        // only exact native storage owns entry comparison
    emitter.instruction("b.ne __rt_onc_standard");                              // all other objects compare their PHP properties
    emitter.instruction("ldp x3, x4, [x11, #8]");                               // load the object and info field offsets
    emitter.instruction("sub sp, sp, #192");                                    // reserve borrowed cells, indices and saved frame state
    emitter.instruction("stp x29, x30, [sp, #176]");                            // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #176");                                   // establish the native comparison frame
    emitter.instruction("str x2, [sp, #96]");                                   // preserve the recursion depth before unboxing
    emitter.instruction("ldr x9, [x0, x3]");                                    // borrow the native array payload from its owning receiver
    emitter.instruction("ldr x10, [x0, x4]");                                   // borrow the native array payload from its owning receiver
    emitter.instruction("ldr x11, [x1, x3]");                                   // borrow the native array payload from its owning receiver
    emitter.instruction("ldr x12, [x1, x4]");                                   // borrow the native array payload from its owning receiver
    emitter.instruction("mov x13, #4");                                         // tag four denotes an indexed array cell
    emitter.instruction("stp x13, x9, [sp, #0]");                               // publish a borrowed stack cell without acquiring an owner
    emitter.instruction("str xzr, [sp, #16]");                                  // the array cell carries no high payload word
    emitter.instruction("stp x13, x10, [sp, #24]");                             // publish a borrowed stack cell without acquiring an owner
    emitter.instruction("str xzr, [sp, #40]");                                  // the array cell carries no high payload word
    emitter.instruction("stp x13, x11, [sp, #48]");                             // publish a borrowed stack cell without acquiring an owner
    emitter.instruction("str xzr, [sp, #64]");                                  // the array cell carries no high payload word
    emitter.instruction("stp x13, x12, [sp, #72]");                             // publish a borrowed stack cell without acquiring an owner
    emitter.instruction("str xzr, [sp, #88]");                                  // the array cell carries no high payload word
    emitter.instruction("add x0, sp, #0");                                      // normalize empty or sentinel array payloads in the borrowed cell
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("stp x0, x1, [sp, #0]");                                // store the normalized tag and array pointer
    emitter.instruction("str x2, [sp, #16]");                                   // store the normalized high payload word
    emitter.instruction("add x0, sp, #24");                                     // normalize empty or sentinel array payloads in the borrowed cell
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("stp x0, x1, [sp, #24]");                               // store the normalized tag and array pointer
    emitter.instruction("str x2, [sp, #40]");                                   // store the normalized high payload word
    emitter.instruction("add x0, sp, #48");                                     // normalize empty or sentinel array payloads in the borrowed cell
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("stp x0, x1, [sp, #48]");                               // store the normalized tag and array pointer
    emitter.instruction("str x2, [sp, #64]");                                   // store the normalized high payload word
    emitter.instruction("add x0, sp, #72");                                     // normalize empty or sentinel array payloads in the borrowed cell
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("stp x0, x1, [sp, #72]");                               // store the normalized tag and array pointer
    emitter.instruction("str x2, [sp, #88]");                                   // store the normalized high payload word
    emitter.instruction("mov x0, sp");                                          // pass the borrowed left object-array cell
    abi::emit_call_label(emitter, "__rt_mixed_count");
    emitter.instruction("str x0, [sp, #104]");                                  // save the left entry count
    emitter.instruction("add x0, sp, #48");                                     // pass the borrowed right object-array cell
    abi::emit_call_label(emitter, "__rt_mixed_count");
    emitter.instruction("ldr x9, [sp, #104]");                                  // reload the left entry count
    emitter.instruction("cmp x0, x9");                                          // storage cardinalities must agree
    emitter.instruction("b.ne __rt_onc_false");                                 // different entry counts are unequal
    emitter.instruction("cbz x9, __rt_onc_true");                               // two empty exact storage instances are equal
    emitter.instruction("str xzr, [sp, #112]");                                 // start with the first left entry
    emitter.label("__rt_onc_left");
    emitter.instruction("ldr x9, [sp, #112]");                                  // reload the left entry ordinal
    emitter.instruction("ldr x10, [sp, #104]");                                 // reload the shared entry count
    emitter.instruction("cmp x9, x10");                                         // has every left entry matched
    emitter.instruction("b.ge __rt_onc_true");                                  // all keys and info values matched
    emitter.instruction("mov x0, sp");                                          // read from the borrowed left object array
    emitter.instruction("mov x1, x9");                                          // select the left object ordinal
    emitter.instruction("mov x2, #-1");                                         // the ordinal is an integer key
    emitter.instruction("mov x3, #0");                                          // comparison reads never emit missing-key warnings
    abi::emit_call_label(emitter, "__rt_mixed_array_get");
    emitter.instruction("str x0, [sp, #128]");                                  // preserve the owned key read until it is unboxed
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("str x1, [sp, #160]");                                  // retain only borrowed object identity for matching
    emitter.instruction("ldr x0, [sp, #128]");                                  // reload the owned left key read
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    emitter.instruction("ldr x9, [sp, #112]");                                  // try the matching ordinal before a full key search
    emitter.instruction("str x9, [sp, #120]");                                  // save the candidate right ordinal
    emitter.instruction("mov x9, #1");                                          // mark the same-order fast candidate
    emitter.instruction("str x9, [sp, #168]");                                  // save the candidate phase
    emitter.label("__rt_onc_right");
    emitter.instruction("ldr x1, [sp, #120]");                                  // reload the candidate right ordinal
    emitter.instruction("ldr x9, [sp, #104]");                                  // reload the right entry count
    emitter.instruction("cmp x1, x9");                                          // has the full key search exhausted its candidates
    emitter.instruction("b.ge __rt_onc_false");                                 // a missing object key makes storage unequal
    emitter.instruction("add x0, sp, #48");                                     // read from the borrowed right object array
    emitter.instruction("mov x2, #-1");                                         // the candidate ordinal is an integer key
    emitter.instruction("mov x3, #0");                                          // comparison reads remain quiet
    abi::emit_call_label(emitter, "__rt_mixed_array_get");
    emitter.instruction("str x0, [sp, #128]");                                  // preserve the owned right key read
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    emitter.instruction("ldr x9, [sp, #160]");                                  // reload the borrowed left object identity
    emitter.instruction("cmp x1, x9");                                          // default storage keys compare by object identity
    emitter.instruction("cset x9, eq");                                         // remember whether this key matched
    emitter.instruction("str x9, [sp, #152]");                                  // preserve the match across release
    emitter.instruction("ldr x0, [sp, #128]");                                  // reload the owned right key read
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    emitter.instruction("ldr x9, [sp, #152]");                                  // reload the candidate match
    emitter.instruction("cbnz x9, __rt_onc_info");                              // matching identities select corresponding info values
    emitter.instruction("ldr x9, [sp, #168]");                                  // was this only the same-order fast candidate
    emitter.instruction("cbz x9, __rt_onc_next_candidate");                     // continue a search already in progress
    emitter.instruction("str xzr, [sp, #168]");                                 // switch to the complete order-independent search
    emitter.instruction("str xzr, [sp, #120]");                                 // restart the candidate search at ordinal zero
    emitter.instruction("b __rt_onc_right");                                    // look up the object identity independently of insertion order
    emitter.label("__rt_onc_next_candidate");
    emitter.instruction("ldr x9, [sp, #120]");                                  // reload the current candidate ordinal
    emitter.instruction("add x9, x9, #1");                                      // advance to the next right entry
    emitter.instruction("str x9, [sp, #120]");                                  // save the advanced candidate ordinal
    emitter.instruction("b __rt_onc_right");                                    // continue matching this left object key
    emitter.label("__rt_onc_info");
    emitter.instruction("add x0, sp, #24");                                     // select the borrowed native info array
    emitter.instruction("ldr x1, [sp, #112]");                                  // select the matching object info ordinal
    emitter.instruction("mov x2, #-1");                                         // info ordinals are integer keys
    emitter.instruction("mov x3, #0");                                          // read the info value without diagnostics
    abi::emit_call_label(emitter, "__rt_mixed_array_get");
    emitter.instruction("str x0, [sp, #136]");                                  // preserve the owned info read
    emitter.instruction("add x0, sp, #72");                                     // select the borrowed native info array
    emitter.instruction("ldr x1, [sp, #120]");                                  // select the matching object info ordinal
    emitter.instruction("mov x2, #-1");                                         // info ordinals are integer keys
    emitter.instruction("mov x3, #0");                                          // read the info value without diagnostics
    abi::emit_call_label(emitter, "__rt_mixed_array_get");
    emitter.instruction("str x0, [sp, #144]");                                  // preserve the owned info read
    emitter.instruction("ldr x0, [sp, #136]");                                  // load the owned left info value
    emitter.instruction("ldr x1, [sp, #144]");                                  // load the owned right info value
    emitter.instruction("ldr x2, [sp, #96]");                                   // restore the shared comparison recursion depth
    abi::emit_call_label(emitter, "__rt_mixed_loose_eq_d");
    emitter.instruction("str x0, [sp, #152]");                                  // preserve the info comparison result
    emitter.instruction("ldr x0, [sp, #136]");                                  // reload an owned info read
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    emitter.instruction("ldr x0, [sp, #144]");                                  // reload an owned info read
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    emitter.instruction("ldr x9, [sp, #152]");                                  // reload the info comparison result
    emitter.instruction("cbz x9, __rt_onc_false");                              // unequal info values end storage comparison
    emitter.instruction("ldr x9, [sp, #112]");                                  // reload the left entry ordinal
    emitter.instruction("add x9, x9, #1");                                      // advance after matching both key and info
    emitter.instruction("str x9, [sp, #112]");                                  // save the next left entry ordinal
    emitter.instruction("b __rt_onc_left");                                     // continue the native storage walk
    emitter.label("__rt_onc_true");
    emitter.instruction("mov x0, #1");                                          // the exact native storage contents are equal
    emitter.instruction("b __rt_onc_done");                                     // return after all owned reads have been retired
    emitter.label("__rt_onc_false");
    emitter.instruction("mov x0, #0");                                          // native storage contents differ
    emitter.label("__rt_onc_done");
    emitter.instruction("ldp x29, x30, [sp, #176]");                            // restore the caller frame and return address
    emitter.instruction("add sp, sp, #192");                                    // release only the borrowed stack cells and scalar state
    emitter.instruction("ret");                                                 // return the native comparison boolean
    emitter.label("__rt_onc_standard");
    emitter.instruction("mov x0, #-1");                                         // delegate to ordinary PHP property comparison
    emitter.instruction("ret");                                                 // the standard path never allocated a native frame
    emitter.label("__rt_onc_uncomparable");
    emitter.instruction("mov x0, #0");                                          // distinct storage subclass instances are uncomparable
    emitter.instruction("ret");                                                 // return false without reading internal or user properties
}
