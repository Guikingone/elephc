//! Purpose:
//! Emits the AArch64 allocation-free grammar preflight for serialized input.
//!
//! Called from:
//! - `super::emit_unserialize()` after the public entry wrapper and before the mutating decoder.
//!
//! Key details:
//! - Every cursor, delimiter, length, and recursive child is bounded before allocation or hooks.

use crate::codegen_support::emit::Emitter;

/// Emits the AArch64 allocation-free grammar preflight used before decoding.
///
/// The validator recursively proves every cursor advance and delimiter before the
/// mutating parser can allocate hashes/objects or invoke hydration hooks.
pub(super) fn emit(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: bounded unserialize grammar preflight ---");

    // uint(base=x0, pos=x1, end=x2, delimiter=w3) -> x0=ok, x1=value, x2=delimiter position
    emitter.label_global("__rt_unser_validate_uint");
    emitter.instruction("add x9, x0, x1");                                      // absolute digit cursor
    emitter.instruction("add x10, x0, x2");                                     // absolute source end
    emitter.instruction("mov x11, #0");                                         // unsigned accumulator
    emitter.instruction("mov x12, #0");                                         // parsed digit count
    emitter.instruction("mov x14, #10");                                        // decimal radix
    emitter.label("__rt_unser_validate_uint_loop");
    emitter.instruction("cmp x9, x10");                                         // is another byte available?
    emitter.instruction("b.hs __rt_unser_validate_uint_fail");                  // truncated digit run has no delimiter
    emitter.instruction("ldrb w13, [x9]");                                      // inspect one bounded byte
    emitter.instruction("cmp w13, #48");                                        // below ASCII zero?
    emitter.instruction("b.lo __rt_unser_validate_uint_done");                  // require the requested delimiter below
    emitter.instruction("cmp w13, #57");                                        // above ASCII nine?
    emitter.instruction("b.hi __rt_unser_validate_uint_done");                  // require the requested delimiter below
    emitter.instruction("sub w13, w13, #48");                                   // convert the byte to a digit
    emitter.instruction("umulh x15, x11, x14");                                 // detect overflow in accumulator * 10
    emitter.instruction("cbnz x15, __rt_unser_validate_uint_fail");             // wrapped lengths/counts are invalid
    emitter.instruction("mul x11, x11, x14");                                   // shift the accumulator by one decimal place
    emitter.instruction("adds x11, x11, x13");                                  // append the current digit and expose carry
    emitter.instruction("b.cs __rt_unser_validate_uint_fail");                  // reject addition overflow
    emitter.instruction("add x12, x12, #1");                                    // record one valid digit
    emitter.instruction("add x9, x9, #1");                                      // advance within the proven source extent
    emitter.instruction("b __rt_unser_validate_uint_loop");                     // scan the remaining digits
    emitter.label("__rt_unser_validate_uint_done");
    emitter.instruction("cbz x12, __rt_unser_validate_uint_fail");              // every numeric field needs at least one digit
    emitter.instruction("cmp w13, w3");                                         // did the run end on its grammar delimiter?
    emitter.instruction("b.ne __rt_unser_validate_uint_fail");                  // arbitrary terminators are rejected
    emitter.instruction("sub x2, x9, x0");                                      // return delimiter position as a source offset
    emitter.instruction("mov x1, x11");                                         // return the parsed unsigned value
    emitter.instruction("mov x0, #1");                                          // report success
    emitter.instruction("ret");                                                 // leaf return
    emitter.label("__rt_unser_validate_uint_fail");
    emitter.instruction("mov x0, #0");                                          // report a bounded numeric failure
    emitter.instruction("ret");                                                 // leaf return

    // key(base=x0, pos=x1, end=x2, depth=x3) -> x0=ok, x1=newpos
    emitter.label_global("__rt_unser_validate_key");
    emitter.instruction("cmp x1, x2");                                          // require the key type byte before loading it
    emitter.instruction("b.hs __rt_unser_validate_key_fail");                   // truncated key
    emitter.instruction("ldrb w9, [x0, x1]");                                   // inspect the bounded key type
    emitter.instruction("cmp w9, #105");                                        // integer key?
    emitter.instruction("b.eq __rt_unser_validate_key_dispatch");               // use an assembler-local conditional target
    emitter.instruction("cmp w9, #115");                                        // string key?
    emitter.instruction("b.ne __rt_unser_validate_key_fail");                   // reject every other key marker
    emitter.label("__rt_unser_validate_key_dispatch");
    emitter.instruction("b __rt_unser_validate_at");                            // main validator owns integer/string grammar
    emitter.label("__rt_unser_validate_key_fail");
    emitter.instruction("mov x0, #0");                                          // only integer and string keys are valid
    emitter.instruction("ret");                                                 // leaf failure return

    // at(base=x0, pos=x1, end=x2, depth=x3) -> x0=ok, x1=newpos
    emitter.label_global("__rt_unser_validate_at");
    emitter.instruction("sub sp, sp, #80");                                     // reserve recursive cursor/count state and an aligned frame
    emitter.instruction("stp x29, x30, [sp, #64]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #64");                                    // establish a stable validator frame
    emitter.instruction("stp x0, x1, [sp]");                                    // save source base and starting position
    emitter.instruction("stp x2, x3, [sp, #16]");                               // save source end and recursion depth
    emitter.instruction("cmp x1, x2");                                          // require a type byte before dispatch
    emitter.instruction("b.hs __rt_unser_validate_at_fail");                    // empty/truncated value
    emitter.instruction("cmp x3, #512");                                        // enforce the same recursion ceiling as the parser
    emitter.instruction("b.hs __rt_unser_validate_at_fail");                    // stop hostile nesting before native-stack exhaustion
    emitter.instruction("ldrb w9, [x0, x1]");                                   // load the bounded type byte
    emitter.instruction("cmp w9, #78");                                         // N
    emitter.instruction("b.eq __rt_unser_validate_null");
    emitter.instruction("cmp w9, #98");                                         // b
    emitter.instruction("b.eq __rt_unser_validate_bool");
    emitter.instruction("cmp w9, #105");                                        // i
    emitter.instruction("b.eq __rt_unser_validate_int");
    emitter.instruction("cmp w9, #100");                                        // d
    emitter.instruction("b.eq __rt_unser_validate_float");
    emitter.instruction("cmp w9, #115");                                        // s
    emitter.instruction("b.eq __rt_unser_validate_string");
    emitter.instruction("cmp w9, #97");                                         // a
    emitter.instruction("b.eq __rt_unser_validate_array");
    emitter.instruction("cmp w9, #79");                                         // O
    emitter.instruction("b.eq __rt_unser_validate_object");
    emitter.instruction("cmp w9, #114");                                        // r
    emitter.instruction("b.eq __rt_unser_validate_ref");
    emitter.instruction("cmp w9, #82");                                         // R
    emitter.instruction("b.eq __rt_unser_validate_ref");
    emitter.instruction("b __rt_unser_validate_at_fail");                       // reject unsupported wire markers

    emitter.label("__rt_unser_validate_null");
    emitter.instruction("sub x9, x2, x1");                                      // bytes remaining from N
    emitter.instruction("cmp x9, #2");                                          // N plus semicolon must fit
    emitter.instruction("b.lo __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x0, x1");                                      // type pointer
    emitter.instruction("ldrb w10, [x9, #1]");                                  // bounded delimiter byte
    emitter.instruction("cmp w10, #59");                                        // semicolon?
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x1, #2");                                      // skip N;
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_bool");
    emitter.instruction("sub x9, x2, x1");                                      // bytes remaining from b
    emitter.instruction("cmp x9, #4");                                          // exact b:<digit>; envelope
    emitter.instruction("b.lo __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x0, x1");                                      // type pointer
    emitter.instruction("ldrb w10, [x9, #1]");
    emitter.instruction("cmp w10, #58");                                        // colon after b
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x9, #2]");
    emitter.instruction("cmp w10, #48");                                        // false digit
    emitter.instruction("b.eq __rt_unser_validate_bool_delim");
    emitter.instruction("cmp w10, #49");                                        // true digit
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.label("__rt_unser_validate_bool_delim");
    emitter.instruction("ldrb w10, [x9, #3]");
    emitter.instruction("cmp w10, #59");                                        // terminating semicolon
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x1, #4");                                      // skip complete boolean
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_int");
    emitter.instruction("add x9, x1, #1");                                      // colon position
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");                                        // require i:
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // first sign/digit position
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #45");                                        // optional minus sign
    emitter.instruction("b.ne __rt_unser_validate_int_digits");
    emitter.instruction("mov x10, #1");                                         // record a negative integer
    emitter.instruction("str x10, [sp, #56]");                                  // preserve sign across the numeric helper
    emitter.instruction("add x9, x9, #1");                                      // skip minus before unsigned digit scan
    emitter.instruction("b __rt_unser_validate_int_scan");
    emitter.label("__rt_unser_validate_int_digits");
    emitter.instruction("str xzr, [sp, #56]");                                  // positive integer
    emitter.label("__rt_unser_validate_int_scan");
    emitter.instruction("ldr x0, [sp]");                                        // base
    emitter.instruction("mov x1, x9");                                          // digit position
    emitter.instruction("ldr x2, [sp, #16]");                                   // end
    emitter.instruction("mov w3, #59");                                         // integer terminator ';'
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    crate::codegen_support::abi::emit_load_int_immediate(emitter, "x9", i64::MAX);
    emitter.instruction("ldr x10, [sp, #56]");                                  // negative-sign flag
    emitter.instruction("cbz x10, __rt_unser_validate_int_positive");
    emitter.instruction("cmp x1, x9");                                          // negative magnitude at most i64::MAX + 1
    emitter.instruction("b.ls __rt_unser_validate_int_range_ok");
    emitter.instruction("sub x10, x1, x9");                                     // only one extra magnitude value is representable
    emitter.instruction("cmp x10, #1");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("b __rt_unser_validate_int_range_ok");
    emitter.label("__rt_unser_validate_int_positive");
    emitter.instruction("cmp x1, x9");                                          // positive magnitude at most i64::MAX
    emitter.instruction("b.hi __rt_unser_validate_at_fail");
    emitter.label("__rt_unser_validate_int_range_ok");
    emitter.instruction("add x1, x2, #1");                                      // skip semicolon
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_float");
    emitter.instruction("add x9, x1, #1");                                      // colon position
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");                                        // require d:
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // first float byte
    emitter.instruction("mov x11, x9");                                         // remember start to reject empty payloads
    emitter.label("__rt_unser_validate_float_loop");
    emitter.instruction("cmp x9, x2");                                          // bound every byte scanned for ';'
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #59");                                        // terminating semicolon?
    emitter.instruction("b.eq __rt_unser_validate_float_done");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("b __rt_unser_validate_float_loop");
    emitter.label("__rt_unser_validate_float_done");
    emitter.instruction("cmp x9, x11");                                         // require at least one float byte
    emitter.instruction("b.eq __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // skip semicolon
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_string");
    emitter.instruction("add x9, x1, #1");                                      // colon after s
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first length digit
    emitter.instruction("mov w3, #58");                                         // length delimiter ':'
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("mov x11, x1");                                         // preserve byte length
    emitter.instruction("add x9, x2, #1");                                      // opening quote position
    emitter.instruction("ldr x10, [sp, #16]");                                  // end
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x12, [sp]");                                       // base
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #34");                                        // opening quote
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // raw payload position
    emitter.instruction("sub x13, x10, x9");                                    // bytes remaining after opening quote
    emitter.instruction("cmp x13, #2");                                         // closing quote plus semicolon
    emitter.instruction("b.lo __rt_unser_validate_at_fail");
    emitter.instruction("sub x13, x13, #2");                                    // maximum safe payload length
    emitter.instruction("cmp x11, x13");                                        // declared bytes fit before delimiters?
    emitter.instruction("b.hi __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, x11");                                     // closing quote position
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #34");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #59");                                        // string terminator
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // position after string
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_array");
    emitter.instruction("add x9, x1, #1");                                      // colon after a
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first count digit
    emitter.instruction("mov w3, #58");                                         // count delimiter ':'
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #32]");                                   // save entry count
    emitter.instruction("add x9, x2, #1");                                      // opening brace position
    emitter.instruction("ldr x10, [sp, #16]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x11, [sp]");
    emitter.instruction("ldrb w12, [x11, x9]");
    emitter.instruction("cmp w12, #123");                                       // opening brace
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // first key position
    emitter.instruction("str x9, [sp, #40]");                                   // current body position
    emitter.instruction("str xzr, [sp, #48]");                                  // entry index
    emitter.label("__rt_unser_validate_array_loop");
    emitter.instruction("ldr x9, [sp, #48]");
    emitter.instruction("ldr x10, [sp, #32]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_container_close");
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x1, [sp, #40]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("ldr x3, [sp, #24]");
    emitter.instruction("add x3, x3, #1");                                      // nested key depth
    emitter.instruction("bl __rt_unser_validate_key");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #40]");                                   // position after key
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("ldr x3, [sp, #24]");
    emitter.instruction("add x3, x3, #1");                                      // nested value depth
    emitter.instruction("bl __rt_unser_validate_at");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #40]");                                   // position after value
    emitter.instruction("ldr x9, [sp, #48]");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("str x9, [sp, #48]");
    emitter.instruction("b __rt_unser_validate_array_loop");

    emitter.label("__rt_unser_validate_object");
    emitter.instruction("add x9, x1, #1");                                      // colon after O
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first class-name length digit
    emitter.instruction("mov w3, #58");
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("mov x11, x1");                                         // class-name byte length
    emitter.instruction("add x9, x2, #1");                                      // opening quote position
    emitter.instruction("ldr x10, [sp, #16]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x12, [sp]");
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #34");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // class-name bytes
    emitter.instruction("sub x13, x10, x9");
    emitter.instruction("cmp x13, #2");                                         // closing quote and colon
    emitter.instruction("b.lo __rt_unser_validate_at_fail");
    emitter.instruction("sub x13, x13, #2");
    emitter.instruction("cmp x11, x13");
    emitter.instruction("b.hi __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, x11");                                     // closing quote
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #34");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");                                      // colon before property count
    emitter.instruction("ldrb w13, [x12, x9]");
    emitter.instruction("cmp w13, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first property-count digit
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("mov w3, #58");
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #32]");                                   // save property count
    emitter.instruction("add x9, x2, #1");                                      // opening brace position
    emitter.instruction("ldr x10, [sp, #16]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x11, [sp]");
    emitter.instruction("ldrb w12, [x11, x9]");
    emitter.instruction("cmp w12, #123");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("str x9, [sp, #40]");                                   // first property key
    emitter.instruction("str xzr, [sp, #48]");                                  // property index
    emitter.label("__rt_unser_validate_object_loop");
    emitter.instruction("ldr x9, [sp, #48]");
    emitter.instruction("ldr x10, [sp, #32]");
    emitter.instruction("cmp x9, x10");
    emitter.instruction("b.hs __rt_unser_validate_container_close");
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x1, [sp, #40]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("ldr x3, [sp, #24]");
    emitter.instruction("add x3, x3, #1");
    emitter.instruction("bl __rt_unser_validate_key");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #40]");
    emitter.instruction("ldr x0, [sp]");
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("ldr x3, [sp, #24]");
    emitter.instruction("add x3, x3, #1");
    emitter.instruction("bl __rt_unser_validate_at");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("str x1, [sp, #40]");
    emitter.instruction("ldr x9, [sp, #48]");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("str x9, [sp, #48]");
    emitter.instruction("b __rt_unser_validate_object_loop");

    emitter.label("__rt_unser_validate_container_close");
    emitter.instruction("ldr x1, [sp, #40]");                                   // closing-brace position
    emitter.instruction("ldr x2, [sp, #16]");
    emitter.instruction("cmp x1, x2");                                          // require the closing brace byte
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldr x9, [sp]");
    emitter.instruction("ldrb w10, [x9, x1]");
    emitter.instruction("cmp w10, #125");                                       // exact closing brace
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x1, #1");                                      // return after the complete container
    emitter.instruction("b __rt_unser_validate_at_ok");

    emitter.label("__rt_unser_validate_ref");
    emitter.instruction("add x9, x1, #1");                                      // colon after r/R
    emitter.instruction("cmp x9, x2");
    emitter.instruction("b.hs __rt_unser_validate_at_fail");
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #58");
    emitter.instruction("b.ne __rt_unser_validate_at_fail");
    emitter.instruction("add x1, x9, #1");                                      // first reference-index digit
    emitter.instruction("mov w3, #59");
    emitter.instruction("bl __rt_unser_validate_uint");
    emitter.instruction("cbz x0, __rt_unser_validate_at_fail");
    emitter.instruction("cbz x1, __rt_unser_validate_at_fail");                 // reference indices are one-based
    emitter.instruction("add x1, x2, #1");                                      // skip semicolon

    emitter.label("__rt_unser_validate_at_ok");
    emitter.instruction("mov x0, #1");                                          // report a fully bounded value
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore recursive validator frame
    emitter.instruction("add sp, sp, #80");                                     // release local validation state
    emitter.instruction("ret");
    emitter.label("__rt_unser_validate_at_fail");
    emitter.instruction("mov x0, #0");                                          // report malformed/truncated wire data
    emitter.instruction("ldr x1, [sp, #8]");                                    // preserve the original position on failure
    emitter.instruction("ldp x29, x30, [sp, #64]");
    emitter.instruction("add sp, sp, #80");
    emitter.instruction("ret");
}
