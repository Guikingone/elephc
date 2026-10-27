//! Purpose:
//! Emits the typed int-array reduction helpers used by the EIR sum-loop rewrite
//! and the `array_sum()` builtin fast path.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::managed::emit_managed_runtime`.
//!
//! Key details:
//! - `__rt_array_sum_int` takes a borrowed Mixed seed cell for the loop rewrite.
//! - `__rt_array_sum_int_i64` takes a raw int seed for `array_sum()` and boxes it
//!   for an empty range.
//! - Both entries reproduce PHP's left-to-right `+` sequence: the first integer
//!   overflow promotes the running sum and that element to doubles, later elements
//!   stay double, and the result is a fresh Mixed int/float cell owned by the caller.
//! - The two entries are self-contained so Mach-O dead stripping can remove either
//!   one without leaving a branch into removed code.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits both `__rt_array_sum_int` reducer entries for the active target.
pub fn emit_array_sum_int(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_sum_int_x86_64(emitter, false);
        emit_array_sum_int_x86_64(emitter, true);
        return;
    }
    emit_array_sum_int_aarch64(emitter, false);
    emit_array_sum_int_aarch64(emitter, true);
}

/// Emits one self-contained AArch64 reducer entry; `raw_seed` selects the integer entry.
fn emit_array_sum_int_aarch64(emitter: &mut Emitter, raw_seed: bool) {
    let prefix = if raw_seed {
        "__rt_array_sum_int_i64"
    } else {
        "__rt_array_sum_int"
    };
    let loop_label = format!("{prefix}_int_loop");
    let empty_label = format!("{prefix}_empty");
    let promote_label = format!("{prefix}_promote");
    let float_loop = format!("{prefix}_float_loop");
    let box_float = format!("{prefix}_box_float");
    let float_init = format!("{prefix}_float_init");
    emitter.blank();
    emitter.comment("--- runtime: array_sum int reducer ---");
    emitter.label_global(prefix);
    // -- empty range: raw seeds are boxed; borrowed cells are retained --
    emitter.instruction("subs x9, x2, x1");                                     // count = end - start and flag an empty range
    emitter.instruction(&format!("b.le {empty_label}"));                        // empty or reversed ranges keep the seed value
    if raw_seed {
        emitter.instruction("mov x11, x3");                                    // seed the integer accumulator from the raw seed
        emitter.instruction("mov x10, #0");                                    // the raw seed starts in integer mode
    } else {
        emitter.instruction("ldr x10, [x3]");                                   // load the seed Mixed tag
        emitter.instruction("ldr x11, [x3, #8]");                               // load the seed payload word
    }
    emitter.instruction("add x12, x0, #24");                                    // skip the indexed-array header to the payload base
    emitter.instruction("add x12, x12, x1, lsl #3");                            // address the first element in the requested range
    emitter.instruction("cbz x10, 1f");                                        // tag 0 starts the checked integer sequence
    emitter.instruction("cmp x10, #2");                                        // tag 2 is a float seed
    emitter.instruction(&format!("b.eq {float_init}"));                        // float seeds accumulate directly as doubles
    emitter.instruction("mov x11, xzr");                                        // the numeric proof excludes other tags; null reads as zero
    emitter.label("1");
    emitter.label(&loop_label);
    emitter.instruction("ldr x13, [x12], #8");                                  // load the next int element and advance the cursor
    emitter.instruction("adds x14, x11, x13");                                  // perform the checked PHP integer addition
    emitter.instruction(&format!("b.vs {promote_label}"));                     // overflow switches the running sum to a double
    emitter.instruction("mov x11, x14");                                        // commit the in-range integer sum
    emitter.instruction("subs x9, x9, #1");                                     // one fewer element remains
    emitter.instruction(&format!("b.ne {loop_label}"));                        // continue while the range is non-empty
    emitter.instruction("mov x0, #0");                                          // int Mixed tag for the result
    emitter.instruction("mov x1, x11");                                        // int payload for the result
    emitter.instruction("mov x2, xzr");                                        // the high payload word is unused for scalars
    emitter.instruction("b __rt_mixed_from_value");                            // box the integer sum and return
    // -- overflow promotion matches PHP's int-to-float conversion for this step --
    emitter.label(&promote_label);
    emitter.instruction("scvtf d0, x11");                                       // convert the pre-overflow sum to double
    emitter.instruction("scvtf d1, x13");                                       // convert the overflowing element to double
    emitter.instruction("fadd d0, d0, d1");                                     // add them as PHP does after the promotion
    emitter.instruction("subs x9, x9, #1");                                     // the promoting element is consumed
    emitter.instruction(&format!("b.eq {box_float}"));                         // stop when no elements remain
    emitter.label(&float_loop);
    emitter.instruction("ldr x13, [x12], #8");                                  // load the next int element and advance the cursor
    emitter.instruction("scvtf d1, x13");                                       // convert the element to double
    emitter.instruction("fadd d0, d0, d1");                                     // accumulate in PHP double mode
    emitter.instruction("subs x9, x9, #1");                                     // one fewer element remains
    emitter.instruction(&format!("b.ne {float_loop}"));                        // continue while the range is non-empty
    emitter.label(&box_float);
    emitter.instruction("fmov x1, d0");                                         // float payload for the result
    emitter.instruction("mov x0, #2");                                          // float Mixed tag for the result
    emitter.instruction("mov x2, xzr");                                        // the high payload word is unused for scalars
    emitter.instruction("b __rt_mixed_from_value");                            // box the float sum and return
    // -- float seed: start the double accumulation from its payload bits --
    emitter.label(&float_init);
    emitter.instruction("fmov d0, x11");                                        // seed the double accumulator from the float payload
    emitter.instruction(&format!("b {float_loop}"));                           // reuse the double accumulation loop
    // -- empty range --
    emitter.label(&empty_label);
    if raw_seed {
        emitter.instruction("mov x0, #0");                                      // int Mixed tag for the empty sum
        emitter.instruction("mov x1, x3");                                      // the raw seed becomes the integer payload
        emitter.instruction("mov x2, xzr");                                    // the high payload word is unused for scalars
        emitter.instruction("b __rt_mixed_from_value");                        // box the raw seed and return
    } else {
        emitter.instruction("mov x0, x3");                                      // return the seed cell itself
        emitter.instruction("b __rt_incref");                                   // add the reference consumed by the replacing store
    }
}

/// Emits one self-contained x86_64 reducer entry; `raw_seed` selects the integer entry.
fn emit_array_sum_int_x86_64(emitter: &mut Emitter, raw_seed: bool) {
    let prefix = if raw_seed {
        "__rt_array_sum_int_i64"
    } else {
        "__rt_array_sum_int"
    };
    let loop_label = format!("{prefix}_int_loop");
    let empty_label = format!("{prefix}_empty");
    let promote_label = format!("{prefix}_promote");
    let float_loop = format!("{prefix}_float_loop");
    let box_float = format!("{prefix}_box_float");
    let float_init = format!("{prefix}_float_init");
    emitter.blank();
    emitter.comment("--- runtime: array_sum int reducer ---");
    emitter.label_global(prefix);
    // -- empty range: raw seeds are boxed; borrowed cells are retained --
    emitter.instruction("mov r10, rdx");                                        // copy the exclusive end before subtracting the start
    emitter.instruction("sub r10, rsi");                                        // count = end - start
    emitter.instruction(&format!("jle {empty_label}"));                         // empty or reversed ranges keep the seed value
    if raw_seed {
        emitter.instruction("mov r8, rcx");                                     // seed the integer accumulator from the raw seed
        emitter.instruction("xor r11d, r11d");                                  // the raw seed starts in integer mode
    } else {
        emitter.instruction("mov r11, QWORD PTR [rcx]");                        // load the seed Mixed tag
        emitter.instruction("mov r8, QWORD PTR [rcx + 8]");                     // load the seed payload word
    }
    emitter.instruction("lea r9, [rdi + 24]");                                  // skip the indexed-array header to the payload base
    emitter.instruction("lea r9, [r9 + rsi * 8]");                              // address the first element in the requested range
    emitter.instruction("test r11, r11");                                       // tag 0 starts the checked integer sequence
    emitter.instruction(&format!("jz {loop_label}"));                           // enter integer accumulation for an int seed
    emitter.instruction("cmp r11, 2");                                          // tag 2 is a float seed
    emitter.instruction(&format!("je {float_init}"));                          // float seeds accumulate directly as doubles
    emitter.instruction("xor r8d, r8d");                                        // the numeric proof excludes other tags; null reads as zero
    emitter.label(&loop_label);
    emitter.instruction("mov rax, QWORD PTR [r9]");                             // load the next int element
    emitter.instruction("add r9, 8");                                           // advance the element cursor
    emitter.instruction("mov r11, r8");                                         // preserve the pre-overflow sum for the float promotion
    emitter.instruction("add r8, rax");                                         // perform the checked PHP integer addition
    emitter.instruction(&format!("jo {promote_label}"));                       // overflow switches the running sum to a double
    emitter.instruction("dec r10");                                             // one fewer element remains
    emitter.instruction(&format!("jnz {loop_label}"));                         // continue while the range is non-empty
    emitter.instruction("xor eax, eax");                                        // int Mixed tag for the result
    emitter.instruction("mov rdi, r8");                                         // int payload for the result
    emitter.instruction("xor esi, esi");                                        // the high payload word is unused for scalars
    emitter.instruction("jmp __rt_mixed_from_value");                          // box the integer sum and return
    // -- overflow promotion matches PHP's int-to-float conversion for this step --
    emitter.label(&promote_label);
    emitter.instruction("cvtsi2sd xmm0, r11");                                  // convert the pre-overflow sum to double
    emitter.instruction("cvtsi2sd xmm1, rax");                                  // convert the overflowing element to double
    emitter.instruction("addsd xmm0, xmm1");                                    // add them as PHP does after the promotion
    emitter.instruction("dec r10");                                             // the promoting element is consumed
    emitter.instruction(&format!("jz {box_float}"));                            // stop when no elements remain
    emitter.label(&float_loop);
    emitter.instruction("mov rax, QWORD PTR [r9]");                             // load the next int element
    emitter.instruction("add r9, 8");                                           // advance the element cursor
    emitter.instruction("cvtsi2sd xmm1, rax");                                  // convert the element to double
    emitter.instruction("addsd xmm0, xmm1");                                    // accumulate in PHP double mode
    emitter.instruction("dec r10");                                             // one fewer element remains
    emitter.instruction(&format!("jnz {float_loop}"));                         // continue while the range is non-empty
    emitter.label(&box_float);
    emitter.instruction("movq rdi, xmm0");                                      // float payload for the result
    emitter.instruction("mov eax, 2");                                          // float Mixed tag for the result
    emitter.instruction("xor esi, esi");                                        // the high payload word is unused for scalars
    emitter.instruction("jmp __rt_mixed_from_value");                          // box the float sum and return
    // -- float seed: start the double accumulation from its payload bits --
    emitter.label(&float_init);
    emitter.instruction("movq xmm0, r8");                                       // seed the double accumulator from the float payload
    emitter.instruction(&format!("jmp {float_loop}"));                         // reuse the double accumulation loop
    // -- empty range --
    emitter.label(&empty_label);
    if raw_seed {
        emitter.instruction("xor eax, eax");                                    // int Mixed tag for the empty sum
        emitter.instruction("mov rdi, rcx");                                    // the raw seed becomes the integer payload
        emitter.instruction("xor esi, esi");                                    // the high payload word is unused for scalars
        emitter.instruction("jmp __rt_mixed_from_value");                      // box the raw seed and return
    } else {
        emitter.instruction("mov rax, rcx");                                    // return the seed cell itself
        emitter.instruction("jmp __rt_incref");                                 // add the reference consumed by the replacing store
    }
}
