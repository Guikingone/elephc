//! Purpose:
//! Emits `__rt_array_sum_int`, the reduction helper used by the EIR typed-array
//! sum-loop rewrite.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::managed::emit_managed_runtime`.
//!
//! Key details:
//! - Arguments are a dense int-shaped indexed array, an inclusive start, an
//!   exclusive end and a borrowed Mixed seed cell.
//! - The helper reproduces PHP's left-to-right `+` sequence: int additions wrap
//!   into a float exactly at the overflow, and the remaining elements continue as
//!   doubles. Empty ranges return the retained seed cell without allocating.
//! - The result is a freshly boxed Mixed int/float cell owned by the caller. The
//!   helper is a leaf except for the terminal boxing/retain helpers.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_array_sum_int` helper for the active target.
pub fn emit_array_sum_int(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_sum_int_x86_64(emitter);
        return;
    }
    emit_array_sum_int_aarch64(emitter);
}

/// Emits the AArch64 reduction helper.
fn emit_array_sum_int_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_sum_int ---");
    emitter.label_global("__rt_array_sum_int");
    // -- an empty range returns the seed cell with its own reference --
    emitter.instruction("subs x9, x2, x1");                                     // count = end - start and flag an empty range
    emitter.instruction("b.le __rt_array_sum_int_empty");                       // an empty or reversed range keeps the seed unchanged
    emitter.instruction("ldr x10, [x3]");                                       // load the seed Mixed tag
    emitter.instruction("ldr x11, [x3, #8]");                                   // load the seed payload word
    emitter.instruction("add x12, x0, #24");                                    // skip the indexed-array header to the payload base
    emitter.instruction("add x12, x12, x1, lsl #3");                            // address the first element in the requested range
    // -- integer mode until a PHP integer addition overflows --
    emitter.instruction("cbz x10, __rt_array_sum_int_int_loop");                // tag 0 starts the checked integer sequence
    emitter.instruction("cmp x10, #2");                                         // tag 2 is a float seed
    emitter.instruction("b.eq __rt_array_sum_int_float_init");                  // float seeds accumulate directly as doubles
    emitter.instruction("mov x11, xzr");                                        // the numeric proof excludes other tags; null reads as zero
    emitter.label("__rt_array_sum_int_int_loop");
    emitter.instruction("ldr x13, [x12], #8");                                  // load the next int element and advance the cursor
    emitter.instruction("adds x14, x11, x13");                                  // perform the checked PHP integer addition
    emitter.instruction("b.vs __rt_array_sum_int_promote");                     // overflow switches the running sum to a double
    emitter.instruction("mov x11, x14");                                        // commit the in-range integer sum
    emitter.instruction("subs x9, x9, #1");                                     // one fewer element remains
    emitter.instruction("b.ne __rt_array_sum_int_int_loop");                    // continue while the range is non-empty
    emitter.instruction("mov x0, #0");                                          // int Mixed tag for the result
    emitter.instruction("mov x1, x11");                                         // int payload for the result
    emitter.instruction("mov x2, xzr");                                         // the high payload word is unused for scalars
    emitter.instruction("b __rt_mixed_from_value");                             // box the integer sum and return
    // -- overflow promotion matches PHP's int-to-float conversion for this step --
    emitter.label("__rt_array_sum_int_promote");
    emitter.instruction("scvtf d0, x11");                                       // convert the pre-overflow sum to double
    emitter.instruction("scvtf d1, x13");                                       // convert the overflowing element to double
    emitter.instruction("fadd d0, d0, d1");                                     // add them as PHP does after the promotion
    emitter.instruction("subs x9, x9, #1");                                     // the promoting element is consumed
    emitter.instruction("b.eq __rt_array_sum_int_box_float");                   // stop when no elements remain
    emitter.label("__rt_array_sum_int_float_loop");
    emitter.instruction("ldr x13, [x12], #8");                                  // load the next int element and advance the cursor
    emitter.instruction("scvtf d1, x13");                                       // convert the element to double
    emitter.instruction("fadd d0, d0, d1");                                     // accumulate in PHP double mode
    emitter.instruction("subs x9, x9, #1");                                     // one fewer element remains
    emitter.instruction("b.ne __rt_array_sum_int_float_loop");                  // continue while the range is non-empty
    emitter.label("__rt_array_sum_int_box_float");
    emitter.instruction("fmov x1, d0");                                         // float payload for the result
    emitter.instruction("mov x0, #2");                                          // float Mixed tag for the result
    emitter.instruction("mov x2, xzr");                                         // the high payload word is unused for scalars
    emitter.instruction("b __rt_mixed_from_value");                             // box the float sum and return
    // -- float seed: start the double accumulation from its payload bits --
    emitter.label("__rt_array_sum_int_float_init");
    emitter.instruction("fmov d0, x11");                                        // seed the double accumulator from the float payload
    emitter.instruction("b __rt_array_sum_int_float_loop");                     // reuse the double accumulation loop
    // -- empty range: retain the seed cell for the caller's new slot owner --
    emitter.label("__rt_array_sum_int_empty");
    emitter.instruction("mov x0, x3");                                          // return the seed cell itself
    emitter.instruction("b __rt_incref");                                       // add the reference consumed by the replacing store
}

/// Emits the x86_64 reduction helper.
fn emit_array_sum_int_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_sum_int ---");
    emitter.label_global("__rt_array_sum_int");
    // -- an empty range returns the seed cell with its own reference --
    emitter.instruction("mov r10, rdx");                                        // copy the exclusive end before subtracting the start
    emitter.instruction("sub r10, rsi");                                        // count = end - start
    emitter.instruction("jle __rt_array_sum_int_empty");                        // an empty or reversed range keeps the seed unchanged
    emitter.instruction("mov r11, QWORD PTR [rcx]");                            // load the seed Mixed tag
    emitter.instruction("mov r8, QWORD PTR [rcx + 8]");                         // load the seed payload word
    emitter.instruction("lea r9, [rdi + 24]");                                  // skip the indexed-array header to the payload base
    emitter.instruction("lea r9, [r9 + rsi * 8]");                              // address the first element in the requested range
    // -- integer mode until a PHP integer addition overflows --
    emitter.instruction("test r11, r11");                                       // tag 0 starts the checked integer sequence
    emitter.instruction("jz __rt_array_sum_int_int_loop");                      // enter integer accumulation for an int seed
    emitter.instruction("cmp r11, 2");                                          // tag 2 is a float seed
    emitter.instruction("je __rt_array_sum_int_float_init");                    // float seeds accumulate directly as doubles
    emitter.instruction("xor r8d, r8d");                                        // the numeric proof excludes other tags; null reads as zero
    emitter.label("__rt_array_sum_int_int_loop");
    emitter.instruction("mov rax, QWORD PTR [r9]");                             // load the next int element
    emitter.instruction("add r9, 8");                                           // advance the element cursor
    emitter.instruction("mov r11, r8");                                         // preserve the pre-overflow sum for the float promotion
    emitter.instruction("add r8, rax");                                         // perform the checked PHP integer addition
    emitter.instruction("jo __rt_array_sum_int_promote");                       // overflow switches the running sum to a double
    emitter.instruction("dec r10");                                             // one fewer element remains
    emitter.instruction("jnz __rt_array_sum_int_int_loop");                     // continue while the range is non-empty
    emitter.instruction("xor eax, eax");                                        // int Mixed tag for the result
    emitter.instruction("mov rdi, r8");                                         // int payload for the result
    emitter.instruction("xor esi, esi");                                        // the high payload word is unused for scalars
    emitter.instruction("jmp __rt_mixed_from_value");                           // box the integer sum and return
    // -- overflow promotion matches PHP's int-to-float conversion for this step --
    emitter.label("__rt_array_sum_int_promote");
    emitter.instruction("cvtsi2sd xmm0, r11");                                  // convert the pre-overflow sum to double
    emitter.instruction("cvtsi2sd xmm1, rax");                                  // convert the overflowing element to double
    emitter.instruction("addsd xmm0, xmm1");                                    // add them as PHP does after the promotion
    emitter.instruction("dec r10");                                             // the promoting element is consumed
    emitter.instruction("jz __rt_array_sum_int_box_float");                     // stop when no elements remain
    emitter.label("__rt_array_sum_int_float_loop");
    emitter.instruction("mov rax, QWORD PTR [r9]");                             // load the next int element
    emitter.instruction("add r9, 8");                                           // advance the element cursor
    emitter.instruction("cvtsi2sd xmm1, rax");                                  // convert the element to double
    emitter.instruction("addsd xmm0, xmm1");                                    // accumulate in PHP double mode
    emitter.instruction("dec r10");                                             // one fewer element remains
    emitter.instruction("jnz __rt_array_sum_int_float_loop");                   // continue while the range is non-empty
    emitter.label("__rt_array_sum_int_box_float");
    emitter.instruction("movq rdi, xmm0");                                      // float payload for the result
    emitter.instruction("mov eax, 2");                                          // float Mixed tag for the result
    emitter.instruction("xor esi, esi");                                        // the high payload word is unused for scalars
    emitter.instruction("jmp __rt_mixed_from_value");                           // box the float sum and return
    // -- float seed: start the double accumulation from its payload bits --
    emitter.label("__rt_array_sum_int_float_init");
    emitter.instruction("movq xmm0, r8");                                       // seed the double accumulator from the float payload
    emitter.instruction("jmp __rt_array_sum_int_float_loop");                   // reuse the double accumulation loop
    // -- empty range: retain the seed cell for the caller's new slot owner --
    emitter.label("__rt_array_sum_int_empty");
    emitter.instruction("mov rax, rcx");                                        // return the seed cell itself
    emitter.instruction("jmp __rt_incref");                                     // add the reference consumed by the replacing store
}
