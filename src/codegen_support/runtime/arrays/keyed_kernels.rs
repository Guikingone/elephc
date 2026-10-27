//! Purpose:
//! Emits fused scalar kernels for `array_sum(array_map(fn($x) => $x + C, $a))` and
//! `count(array_filter($a, fn($x) => $x % C === 0))` over packed int arrays.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::managed::emit_managed_runtime`.
//!
//! Key details:
//! - `__rt_array_map_sum_add` takes the array pointer and the literal offset, then
//!   boxes the exact PHP map-then-sum result (int until the first int-to-float
//!   promotion) as a fresh Mixed cell.
//! - `__rt_array_filter_count_mod` takes the array pointer and the literal divisor
//!   and returns a raw PHP int count; powers of two use a mask instead of division.
//! - Both walk the packed payload at `array + 24` and read the element count at
//!   `array + 0`, matching every other indexed-array runtime helper.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits both fused scalar kernels for the active target.
pub fn emit_keyed_kernels(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_map_sum_add_x86_64(emitter);
        emit_filter_count_mod_x86_64(emitter);
        return;
    }
    emit_map_sum_add_aarch64(emitter);
    emit_filter_count_mod_aarch64(emitter);
}

/// Emits `__rt_array_map_sum_add` for AArch64.
fn emit_map_sum_add_aarch64(emitter: &mut Emitter) {
    let loop_label = "__rt_array_map_sum_add_loop";
    let mapped_overflow = "__rt_array_map_sum_add_mapped_overflow";
    let acc_overflow = "__rt_array_map_sum_add_acc_overflow";
    let float_acc = "__rt_array_map_sum_add_float_acc";
    let add_float = "__rt_array_map_sum_add_add_float";
    let done = "__rt_array_map_sum_add_done";
    let box_float = "__rt_array_map_sum_add_box_float";

    emitter.blank();
    emitter.comment("--- runtime: array_map_sum_add (fused int kernel) ---");
    emitter.label_global("__rt_array_map_sum_add");
    emitter.instruction("ldr x9, [x0]");                                        // load the packed element count
    emitter.instruction("add x10, x0, #24");                                    // skip the indexed-array header to the payload base
    emitter.instruction("mov x11, xzr");                                        // start the integer accumulator at zero
    emitter.instruction("mov x12, xzr");                                        // integer mode: promote to double on the first overflow

    emitter.label(loop_label);
    emitter.instruction(&format!("cbz x9, {done}"));                            // stop when every element has been mapped and summed
    emitter.instruction("ldr x13, [x10], #8");                                  // load the next source element
    emitter.instruction("adds x14, x13, x1");                                   // map the element with PHP's checked integer addition
    emitter.instruction(&format!("b.vs {mapped_overflow}"));                    // an overflowed map result becomes a double
    emitter.instruction(&format!("cbnz x12, {float_acc}"));                     // float accumulation converts the mapped integer first
    emitter.instruction("adds x15, x11, x14");                                  // sum the mapped integer with the running accumulator
    emitter.instruction(&format!("b.vs {acc_overflow}"));                       // overflow switches the running sum to a double
    emitter.instruction("mov x11, x15");                                        // commit the in-range integer sum
    emitter.instruction("sub x9, x9, #1");                                      // one fewer element remains
    emitter.instruction(&format!("b {loop_label}"));                            // continue through the packed payload

    emitter.label(acc_overflow);
    emitter.instruction("scvtf d0, x11");                                       // convert the pre-overflow accumulator to double
    emitter.instruction("scvtf d1, x14");                                       // convert the mapped integer to double
    emitter.instruction("fadd d0, d0, d1");                                     // add them as PHP does after the promotion
    emitter.instruction("mov x12, #1");                                         // remain in double mode for the rest of the range
    emitter.instruction("sub x9, x9, #1");                                      // the promoting element is consumed
    emitter.instruction(&format!("b {loop_label}"));                            // continue accumulating as a double

    emitter.label(mapped_overflow);
    emitter.instruction("scvtf d1, x13");                                       // convert the source element to double
    emitter.instruction("scvtf d2, x1");                                        // convert the literal map offset to double
    emitter.instruction("fadd d1, d1, d2");                                     // build the float map result PHP would return
    emitter.instruction(&format!("cbnz x12, {add_float}"));                     // a float accumulator already holds the running sum
    emitter.instruction("scvtf d0, x11");                                       // promote the integer accumulator at this step
    emitter.instruction("mov x12, #1");                                         // remain in double mode for the rest of the range

    emitter.label(add_float);
    emitter.instruction("fadd d0, d0, d1");                                     // add the float map result to the running sum
    emitter.instruction("sub x9, x9, #1");                                      // the promoting element is consumed
    emitter.instruction(&format!("b {loop_label}"));                            // continue accumulating as a double

    emitter.label(float_acc);
    emitter.instruction("scvtf d1, x14");                                       // convert the in-range mapped integer to double
    emitter.instruction("fadd d0, d0, d1");                                     // accumulate it in PHP double mode
    emitter.instruction("sub x9, x9, #1");                                      // one fewer element remains
    emitter.instruction(&format!("b {loop_label}"));                            // continue through the packed payload

    emitter.label(done);
    emitter.instruction(&format!("cbnz x12, {box_float}"));                     // a promoted sum is returned as a float cell
    emitter.instruction("mov x0, #0");                                          // int Mixed tag for the integer sum
    emitter.instruction("mov x1, x11");                                         // int payload for the integer sum
    emitter.instruction("mov x2, xzr");                                         // the high payload word is unused for scalars
    emitter.instruction("b __rt_mixed_from_value");                             // box the integer sum and return

    emitter.label(box_float);
    emitter.instruction("fmov x1, d0");                                         // float payload for the promoted sum
    emitter.instruction("mov x0, #2");                                          // float Mixed tag for the promoted sum
    emitter.instruction("mov x2, xzr");                                         // the high payload word is unused for scalars
    emitter.instruction("b __rt_mixed_from_value");                             // box the float sum and return
}

/// Emits `__rt_array_map_sum_add` for Linux x86_64.
fn emit_map_sum_add_x86_64(emitter: &mut Emitter) {
    let loop_label = "__rt_array_map_sum_add_loop_x86";
    let mapped_overflow = "__rt_array_map_sum_add_mapped_overflow_x86";
    let acc_overflow = "__rt_array_map_sum_add_acc_overflow_x86";
    let float_acc = "__rt_array_map_sum_add_float_acc_x86";
    let add_float = "__rt_array_map_sum_add_add_float_x86";
    let done = "__rt_array_map_sum_add_done_x86";
    let box_float = "__rt_array_map_sum_add_box_float_x86";

    emitter.blank();
    emitter.comment("--- runtime: array_map_sum_add (fused int kernel) ---");
    emitter.label_global("__rt_array_map_sum_add");
    emitter.instruction("mov r10, QWORD PTR [rdi]");                            // load the packed element count
    emitter.instruction("lea r9, [rdi + 24]");                                  // skip the indexed-array header to the payload base
    emitter.instruction("xor r8d, r8d");                                        // start the integer accumulator at zero
    emitter.instruction("xor r11d, r11d");                                      // integer mode: promote to double on the first overflow

    emitter.label(loop_label);
    emitter.instruction("test r10, r10");                                       // stop when every element has been mapped and summed
    emitter.instruction(&format!("jz {done}"));                                 // the range is exhausted
    emitter.instruction("mov rax, QWORD PTR [r9]");                             // load the next source element
    emitter.instruction("add r9, 8");                                           // advance the element cursor
    emitter.instruction("mov rcx, rax");                                        // copy the element before mapping it
    emitter.instruction("add rcx, rsi");                                        // map the element with PHP's checked integer addition
    emitter.instruction(&format!("jo {mapped_overflow}"));                      // an overflowed map result becomes a double
    emitter.instruction("test r11d, r11d");                                     // is the running sum already a double?
    emitter.instruction(&format!("jnz {float_acc}"));                           // float accumulation converts the mapped integer first
    emitter.instruction("mov rdx, r8");                                         // copy the integer accumulator before the checked add
    emitter.instruction("add rdx, rcx");                                        // sum the mapped integer with the running accumulator
    emitter.instruction(&format!("jo {acc_overflow}"));                         // overflow switches the running sum to a double
    emitter.instruction("mov r8, rdx");                                         // commit the in-range integer sum
    emitter.instruction("dec r10");                                             // one fewer element remains
    emitter.instruction(&format!("jmp {loop_label}"));                          // continue through the packed payload

    emitter.label(acc_overflow);
    emitter.instruction("cvtsi2sd xmm0, r8");                                   // convert the pre-overflow accumulator to double
    emitter.instruction("cvtsi2sd xmm1, rcx");                                  // convert the mapped integer to double
    emitter.instruction("addsd xmm0, xmm1");                                    // add them as PHP does after the promotion
    emitter.instruction("mov r11d, 1");                                         // remain in double mode for the rest of the range
    emitter.instruction("dec r10");                                             // the promoting element is consumed
    emitter.instruction(&format!("jmp {loop_label}"));                          // continue accumulating as a double

    emitter.label(mapped_overflow);
    emitter.instruction("cvtsi2sd xmm1, rax");                                  // convert the source element to double
    emitter.instruction("cvtsi2sd xmm2, rsi");                                  // convert the literal map offset to double
    emitter.instruction("addsd xmm1, xmm2");                                    // build the float map result PHP would return
    emitter.instruction("test r11d, r11d");                                     // is the running sum already a double?
    emitter.instruction(&format!("jnz {add_float}"));                           // a float accumulator already holds the running sum
    emitter.instruction("cvtsi2sd xmm0, r8");                                   // promote the integer accumulator at this step
    emitter.instruction("mov r11d, 1");                                         // remain in double mode for the rest of the range

    emitter.label(add_float);
    emitter.instruction("addsd xmm0, xmm1");                                    // add the float map result to the running sum
    emitter.instruction("dec r10");                                             // the promoting element is consumed
    emitter.instruction(&format!("jmp {loop_label}"));                          // continue accumulating as a double

    emitter.label(float_acc);
    emitter.instruction("cvtsi2sd xmm1, rcx");                                  // convert the in-range mapped integer to double
    emitter.instruction("addsd xmm0, xmm1");                                    // accumulate it in PHP double mode
    emitter.instruction("dec r10");                                             // one fewer element remains
    emitter.instruction(&format!("jmp {loop_label}"));                          // continue through the packed payload

    emitter.label(done);
    emitter.instruction("test r11d, r11d");                                     // a promoted sum is returned as a float cell
    emitter.instruction(&format!("jnz {box_float}"));                           // box the promoted double
    emitter.instruction("xor eax, eax");                                        // int Mixed tag for the integer sum
    emitter.instruction("mov rdi, r8");                                         // int payload for the integer sum
    emitter.instruction("xor esi, esi");                                        // the high payload word is unused for scalars
    emitter.instruction("jmp __rt_mixed_from_value");                           // box the integer sum and return

    emitter.label(box_float);
    emitter.instruction("movq rdi, xmm0");                                      // float payload for the promoted sum
    emitter.instruction("mov eax, 2");                                          // float Mixed tag for the promoted sum
    emitter.instruction("xor esi, esi");                                        // the high payload word is unused for scalars
    emitter.instruction("jmp __rt_mixed_from_value");                           // box the float sum and return
}

/// Emits `__rt_array_filter_count_mod` for AArch64.
fn emit_filter_count_mod_aarch64(emitter: &mut Emitter) {
    let normalize = "__rt_array_filter_count_mod_normalize";
    let power_check = "__rt_array_filter_count_mod_power_check";
    let mask_loop = "__rt_array_filter_count_mod_mask_loop";
    let mask_skip = "__rt_array_filter_count_mod_mask_skip";
    let mod_loop = "__rt_array_filter_count_mod_loop";
    let mod_skip = "__rt_array_filter_count_mod_skip";
    let done = "__rt_array_filter_count_mod_done";

    emitter.blank();
    emitter.comment("--- runtime: array_filter_count_mod (fused int kernel) ---");
    emitter.label_global("__rt_array_filter_count_mod");
    emitter.instruction("ldr x9, [x0]");                                        // load the packed element count
    emitter.instruction("add x10, x0, #24");                                    // skip the indexed-array header to the payload base
    emitter.instruction("mov x11, xzr");                                        // count elements divisible by the literal divisor
    emitter.instruction(&format!("cbz x9, {done}"));                            // an empty array counts zero matches
    emitter.instruction("mov x12, x1");                                         // copy the divisor before normalizing its sign
    emitter.instruction(&format!("tbnz x12, #63, {normalize}"));                // negative divisors use their absolute value

    emitter.label(power_check);
    emitter.instruction("sub x14, x12, #1");                                    // mask for divisibility by a power of two
    emitter.instruction("tst x14, x12");                                        // is the normalized divisor a power of two?
    emitter.instruction(&format!("b.ne {mod_loop}"));                           // non-powers of two need a real remainder

    emitter.label(mask_loop);
    emitter.instruction("ldr x13, [x10], #8");                                  // load the next source element
    emitter.instruction("tst x13, x14");                                        // is the element a multiple of the power-of-two divisor?
    emitter.instruction(&format!("b.ne {mask_skip}"));                          // non-multiples do not increment the count
    emitter.instruction("add x11, x11, #1");                                    // record a divisible element
    emitter.label(mask_skip);
    emitter.instruction("subs x9, x9, #1");                                     // one fewer element remains
    emitter.instruction(&format!("b.ne {mask_loop}"));                          // continue through the packed payload
    emitter.instruction(&format!("b {done}"));                                  // return the completed count

    emitter.label(normalize);
    emitter.instruction("neg x12, x12");                                        // divisibility is unchanged by the divisor's sign
    emitter.instruction(&format!("b {power_check}"));                           // retry the power-of-two recognition

    emitter.label(mod_loop);
    emitter.instruction("ldr x13, [x10], #8");                                  // load the next source element
    emitter.instruction("sdiv x15, x13, x12");                                  // form the truncated quotient
    emitter.instruction("msub x15, x15, x12, x13");                             // derive PHP's truncated remainder
    emitter.instruction(&format!("cbnz x15, {mod_skip}"));                      // non-zero remainders do not increment the count
    emitter.instruction("add x11, x11, #1");                                    // record a divisible element
    emitter.label(mod_skip);
    emitter.instruction("subs x9, x9, #1");                                     // one fewer element remains
    emitter.instruction(&format!("b.ne {mod_loop}"));                           // continue through the packed payload

    emitter.label(done);
    emitter.instruction("mov x0, x11");                                         // return the raw PHP int count
    emitter.instruction("ret");                                                 // transfer the count to the caller
}

/// Emits `__rt_array_filter_count_mod` for Linux x86_64.
fn emit_filter_count_mod_x86_64(emitter: &mut Emitter) {
    let mask_loop = "__rt_array_filter_count_mod_mask_loop_x86";
    let mask_skip = "__rt_array_filter_count_mod_mask_skip_x86";
    let mod_loop = "__rt_array_filter_count_mod_loop_x86";
    let mod_skip = "__rt_array_filter_count_mod_skip_x86";
    let done = "__rt_array_filter_count_mod_done_x86";

    emitter.blank();
    emitter.comment("--- runtime: array_filter_count_mod (fused int kernel) ---");
    emitter.label_global("__rt_array_filter_count_mod");
    emitter.instruction("mov r10, QWORD PTR [rdi]");                            // load the packed element count
    emitter.instruction("lea r9, [rdi + 24]");                                  // skip the indexed-array header to the payload base
    emitter.instruction("xor r8d, r8d");                                        // count elements divisible by the literal divisor
    emitter.instruction("mov r11, rsi");                                        // copy the divisor before normalizing its sign
    emitter.instruction("test r11, r11");                                       // is the divisor negative?
    emitter.instruction("jns __rt_array_filter_count_mod_positive_x86");        // positive divisors already name their absolute value
    emitter.instruction("neg r11");                                             // divisibility is unchanged by the divisor's sign
    emitter.label("__rt_array_filter_count_mod_positive_x86");
    emitter.instruction("test r10, r10");                                       // an empty array counts zero matches
    emitter.instruction(&format!("jz {done}"));                                 // return before touching the payload
    emitter.instruction("lea rcx, [r11 - 1]");                                  // mask for divisibility by a power of two
    emitter.instruction("test rcx, r11");                                       // is the normalized divisor a power of two?
    emitter.instruction(&format!("jnz {mod_loop}"));                            // non-powers of two need a real remainder

    emitter.label(mask_loop);
    emitter.instruction("mov rax, QWORD PTR [r9]");                             // load the next source element
    emitter.instruction("add r9, 8");                                           // advance the element cursor
    emitter.instruction("test rax, rcx");                                       // is the element a multiple of the power-of-two divisor?
    emitter.instruction(&format!("jnz {mask_skip}"));                           // non-multiples do not increment the count
    emitter.instruction("inc r8");                                              // record a divisible element
    emitter.label(mask_skip);
    emitter.instruction("dec r10");                                             // one fewer element remains
    emitter.instruction(&format!("jnz {mask_loop}"));                           // continue through the packed payload
    emitter.instruction(&format!("jmp {done}"));                                // return the completed count

    emitter.label(mod_loop);
    emitter.instruction("mov rax, QWORD PTR [r9]");                             // load the next source element
    emitter.instruction("add r9, 8");                                           // advance the element cursor
    emitter.instruction("cqo");                                                 // sign-extend the dividend into the high division word
    emitter.instruction("idiv r11");                                            // derive the truncated quotient and remainder
    emitter.instruction(&format!("test rdx, rdx"));                             // is the remainder zero?
    emitter.instruction(&format!("jnz {mod_skip}"));                            // non-zero remainders do not increment the count
    emitter.instruction("inc r8");                                              // record a divisible element
    emitter.label(mod_skip);
    emitter.instruction("dec r10");                                             // one fewer element remains
    emitter.instruction(&format!("jnz {mod_loop}"));                            // continue through the packed payload

    emitter.label(done);
    emitter.instruction("mov rax, r8");                                         // return the raw PHP int count
    emitter.instruction("ret");                                                 // transfer the count to the caller
}
