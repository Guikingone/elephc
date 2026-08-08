//! Purpose:
//! Emits the `__rt_explode` runtime helper assembly for PHP's `explode()`.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.
//! - The helper implements PHP's full `$limit` contract: a positive limit caps the element
//!   count and lets the last element absorb the remaining suffix, `0` behaves like `1`, and a
//!   negative limit drops that many trailing segments. Negative limits therefore need the
//!   segment total up front, which is why the delimiter scan is factored into a local
//!   subroutine shared by a counting pass and the emitting pass.
//! - A zero-length separator returns "no match" instead of matching everywhere. Reference PHP
//!   raises `ValueError` for it and the EIR lowering guard does the same before this helper is
//!   reached; the check here only keeps the scan from looping forever if it ever is.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits the `__rt_explode` runtime helper for splitting a string by a delimiter.
///
/// Dispatches to `emit_explode_linux_x86_64` on x86_64; falls through to the ARM64
/// implementation on all other targets. Uses target ABI registers for the pointer/length
/// pairs: x1/x2 = delimiter ptr/length, x3/x4 = subject ptr/length, x5 = `$limit`, x0 =
/// result array pointer. Allocates an initial indexed array with 16 string slots and pushes
/// each retained segment via `__rt_array_push_str`. Stack frame is 112 bytes on ARM64.
pub fn emit_explode(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_explode_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: explode ---");
    emitter.label_global("__rt_explode");

    // -- set up stack frame (112 bytes) --
    // [sp+0]  delimiter ptr   [sp+8]  delimiter len
    // [sp+16] subject ptr     [sp+24] subject len
    // [sp+32] result array    [sp+40] scan position
    // [sp+48] segment start   [sp+56] element cap
    // [sp+64] extend-last     [sp+72] emitted count
    // [sp+80] segment total   [sp+88] delimiter-scan start
    // [sp+96] saved x29, x30
    emitter.instruction("sub sp, sp, #112");                                    // allocate the explode() scan frame
    emitter.instruction("stp x29, x30, [sp, #96]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #96");                                    // establish new frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save delimiter ptr and length
    emitter.instruction("stp x3, x4, [sp, #16]");                               // save subject ptr and length
    emitter.instruction("str x5, [sp, #56]");                                   // save the raw PHP $limit before it becomes an element cap

    // -- create a new string array --
    emitter.instruction("mov x0, #16");                                         // initial array capacity = 16 elements
    emitter.instruction("mov x1, #16");                                         // element size = 16 bytes (ptr + len)
    emitter.instruction("bl __rt_array_new");                                   // call array constructor, returns array in x0
    emitter.instruction("str x0, [sp, #32]");                                   // save array pointer on stack

    // -- translate PHP's $limit into an element cap plus a last-element rule --
    emitter.instruction("ldr x9, [sp, #56]");                                   // reload the raw $limit
    emitter.instruction("cmp x9, #0");                                          // classify the limit as positive, zero, or negative
    emitter.instruction("b.gt __rt_explode_cap_positive");                      // a positive limit is already the element cap
    emitter.instruction("b.lt __rt_explode_cap_negative");                      // a negative limit drops that many trailing segments
    emitter.instruction("mov x9, #1");                                          // PHP treats $limit === 0 exactly like $limit === 1
    emitter.label("__rt_explode_cap_positive");
    emitter.instruction("str x9, [sp, #56]");                                   // publish the element cap
    emitter.instruction("mov x10, #1");                                         // positive limits let the final element absorb the rest of the subject
    emitter.instruction("str x10, [sp, #64]");                                  // publish the extend-last-element rule
    emitter.instruction("b __rt_explode_scan_init");                            // start emitting segments

    // -- negative limit: count the segments first so the cap can drop the tail --
    emitter.label("__rt_explode_cap_negative");
    emitter.instruction("mov x10, #1");                                         // a subject with no delimiter still holds one segment
    emitter.instruction("str x10, [sp, #80]");                                  // seed the running segment total
    emitter.instruction("str xzr, [sp, #88]");                                  // count from the start of the subject
    emitter.label("__rt_explode_count_loop");
    emitter.instruction("bl __rt_explode_find");                                // locate the next delimiter occurrence
    emitter.instruction("cmp x0, #0");                                          // did the scan run out of delimiters?
    emitter.instruction("b.lt __rt_explode_count_done");                        // stop counting once no delimiter remains
    emitter.instruction("ldr x10, [sp, #80]");                                  // reload the running segment total
    emitter.instruction("add x10, x10, #1");                                    // one delimiter introduces one more segment
    emitter.instruction("str x10, [sp, #80]");                                  // publish the updated segment total
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload the delimiter length
    emitter.instruction("add x0, x0, x11");                                     // resume counting after the matched delimiter
    emitter.instruction("str x0, [sp, #88]");                                   // publish the next delimiter-scan start
    emitter.instruction("b __rt_explode_count_loop");                           // continue counting delimiter occurrences
    emitter.label("__rt_explode_count_done");
    emitter.instruction("ldr x10, [sp, #80]");                                  // reload the final segment total
    emitter.instruction("ldr x9, [sp, #56]");                                   // reload the negative $limit
    emitter.instruction("add x9, x10, x9");                                     // cap = segment total + negative limit
    emitter.instruction("cmp x9, #0");                                          // does the limit drop every segment?
    emitter.instruction("b.le __rt_explode_return_array");                      // PHP returns an empty array when it does
    emitter.instruction("str x9, [sp, #56]");                                   // publish the element cap
    emitter.instruction("str xzr, [sp, #64]");                                  // negative limits never extend the final element

    // -- emit the retained segments --
    emitter.label("__rt_explode_scan_init");
    emitter.instruction("str xzr, [sp, #40]");                                  // scan position starts at the beginning of the subject
    emitter.instruction("str xzr, [sp, #48]");                                  // first segment starts at the beginning of the subject
    emitter.instruction("str xzr, [sp, #72]");                                  // no elements have been emitted yet

    emitter.label("__rt_explode_loop");
    emitter.instruction("ldr x9, [sp, #72]");                                   // reload the emitted element count
    emitter.instruction("ldr x10, [sp, #56]");                                  // reload the element cap
    emitter.instruction("cmp x9, x10");                                         // has the limit already been reached?
    emitter.instruction("b.ge __rt_explode_return_array");                      // stop without a trailing element when it has
    emitter.instruction("ldr x11, [sp, #64]");                                  // reload the extend-last-element rule
    emitter.instruction("cbz x11, __rt_explode_next_delim");                    // negative limits always emit plain segments
    emitter.instruction("add x9, x9, #1");                                      // would this element be the last one the limit allows?
    emitter.instruction("cmp x9, x10");                                         // compare the prospective count against the cap
    emitter.instruction("b.ge __rt_explode_last");                              // the last allowed element absorbs the remaining suffix

    emitter.label("__rt_explode_next_delim");
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload the current scan position
    emitter.instruction("str x9, [sp, #88]");                                   // hand it to the delimiter-scan subroutine
    emitter.instruction("bl __rt_explode_find");                                // locate the next delimiter occurrence
    emitter.instruction("cmp x0, #0");                                          // did the scan run out of delimiters?
    emitter.instruction("b.lt __rt_explode_last");                              // the remaining suffix becomes the final element
    emitter.instruction("str x0, [sp, #40]");                                   // remember where the matched delimiter starts

    // -- push the segment that precedes the matched delimiter --
    emitter.instruction("ldr x0, [sp, #32]");                                   // load array pointer
    emitter.instruction("ldr x3, [sp, #16]");                                   // reload the subject pointer
    emitter.instruction("ldr x16, [sp, #48]");                                  // load segment start position
    emitter.instruction("ldr x17, [sp, #40]");                                  // load the matched delimiter position
    emitter.instruction("add x1, x3, x16");                                     // segment ptr = subject + segment_start
    emitter.instruction("sub x2, x17, x16");                                    // segment len = match_pos - segment_start
    emitter.instruction("bl __rt_array_push_str");                              // push segment string to array
    emitter.instruction("str x0, [sp, #32]");                                   // update array pointer after possible realloc
    emitter.instruction("ldr x9, [sp, #72]");                                   // reload the emitted element count
    emitter.instruction("add x9, x9, #1");                                      // one more element has been emitted
    emitter.instruction("str x9, [sp, #72]");                                   // publish the updated emitted count

    // -- advance past delimiter, update segment start --
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload delimiter length
    emitter.instruction("ldr x17, [sp, #40]");                                  // reload the matched delimiter position
    emitter.instruction("add x17, x17, x11");                                   // skip past delimiter
    emitter.instruction("str x17, [sp, #40]");                                  // save new scan position
    emitter.instruction("str x17, [sp, #48]");                                  // update segment start to after delimiter
    emitter.instruction("b __rt_explode_loop");                                 // continue scanning

    // -- push final segment (from last delimiter to end of string) --
    emitter.label("__rt_explode_last");
    emitter.instruction("ldr x0, [sp, #32]");                                   // load array pointer
    emitter.instruction("ldp x3, x4, [sp, #16]");                               // reload subject ptr and length
    emitter.instruction("ldr x16, [sp, #48]");                                  // load segment start position
    emitter.instruction("add x1, x3, x16");                                     // segment ptr = subject + segment_start
    emitter.instruction("sub x2, x4, x16");                                     // segment len = subject_len - segment_start
    emitter.instruction("bl __rt_array_push_str");                              // push final segment to array
    emitter.instruction("str x0, [sp, #32]");                                   // update array pointer after possible realloc

    // -- return array and restore frame --
    emitter.label("__rt_explode_return_array");
    emitter.instruction("ldr x0, [sp, #32]");                                   // return array pointer in x0
    emitter.instruction("ldp x29, x30, [sp, #96]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #112");                                    // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    // -- local subroutine: first delimiter at or after [sp+88], or -1 --
    emitter.comment("--- runtime: explode delimiter scan (local subroutine) ---");
    emitter.label("__rt_explode_find");
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload delimiter ptr and length
    emitter.instruction("cbz x2, __rt_explode_find_none");                      // a zero-length separator can never match
    emitter.instruction("ldp x3, x4, [sp, #16]");                               // reload subject ptr and length
    emitter.instruction("ldr x9, [sp, #88]");                                   // load the requested scan start position
    emitter.label("__rt_explode_find_loop");
    emitter.instruction("sub x10, x4, x9");                                     // remaining = subject_len - scan_pos
    emitter.instruction("cmp x2, x10");                                         // check if delimiter still fits in the remainder
    emitter.instruction("b.gt __rt_explode_find_none");                         // delimiter longer than remaining, no match
    emitter.instruction("mov x11, #0");                                         // delimiter comparison index = 0
    emitter.label("__rt_explode_find_cmp");
    emitter.instruction("cmp x11, x2");                                         // check if all delimiter bytes matched
    emitter.instruction("b.ge __rt_explode_find_hit");                          // full match, delimiter found
    emitter.instruction("add x12, x9, x11");                                    // compute subject index = scan_pos + cmp_idx
    emitter.instruction("ldrb w14, [x3, x12]");                                 // load subject byte at computed index
    emitter.instruction("ldrb w15, [x1, x11]");                                 // load delimiter byte at cmp index
    emitter.instruction("cmp w14, w15");                                        // compare subject and delimiter bytes
    emitter.instruction("b.ne __rt_explode_find_next");                         // mismatch, advance by 1
    emitter.instruction("add x11, x11, #1");                                    // advance delimiter index
    emitter.instruction("b __rt_explode_find_cmp");                             // continue comparing
    emitter.label("__rt_explode_find_next");
    emitter.instruction("add x9, x9, #1");                                      // move scan position forward by 1
    emitter.instruction("b __rt_explode_find_loop");                            // continue scanning
    emitter.label("__rt_explode_find_hit");
    emitter.instruction("mov x0, x9");                                          // return the matched delimiter position
    emitter.instruction("ret");                                                 // return to the explode() scan loop
    emitter.label("__rt_explode_find_none");
    emitter.instruction("mov x0, #-1");                                         // report that no delimiter remains
    emitter.instruction("ret");                                                 // return to the explode() scan loop
}

/// Emits the x86_64 implementation of `__rt_explode`.
///
/// Dispatches from `emit_explode` when targeting x86_64. Uses the AMD64 System V ABI
/// registers elephc's string lowering materializes: delimiter pointer/length in rax/rdx,
/// subject pointer/length in rdi/rsi, `$limit` in rcx, result array pointer returned in
/// rax. Uses an rbp-relative frame (delimiter pair at `[rbp-8]`/`[rbp-16]`, subject pair at
/// `[rbp-24]`/`[rbp-32]`, array pointer at `[rbp-40]`, scan position at `[rbp-48]`, segment
/// start at `[rbp-56]`, element cap at `[rbp-64]`, extend-last rule at `[rbp-72]`, emitted
/// count at `[rbp-80]`, segment total at `[rbp-88]`, delimiter-scan start at `[rbp-96]`) so
/// every value survives the helper calls that clobber caller-saved registers.
fn emit_explode_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: explode ---");
    emitter.label_global("__rt_explode");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before the splitter uses stack-backed scan state
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the saved delimiter, subject string, and scan cursors
    emitter.instruction("sub rsp, 112");                                        // reserve aligned local storage for the saved strings, limit bookkeeping, and scan indices
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the delimiter pointer so every scan iteration can reload it without depending on caller-saved registers
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the delimiter length so the fit check survives helper calls and loop back-edges
    emitter.instruction("mov QWORD PTR [rbp - 24], rdi");                       // save the subject-string pointer so every scan iteration can reload it without depending on caller-saved registers
    emitter.instruction("mov QWORD PTR [rbp - 32], rsi");                       // save the subject-string length so the fit and final-segment checks survive helper calls
    emitter.instruction("mov QWORD PTR [rbp - 64], rcx");                       // save the raw PHP $limit before it becomes an element cap

    emitter.instruction("mov rdi, 16");                                         // request an initial indexed-array capacity of sixteen string slots for explode()
    emitter.instruction("mov rsi, 16");                                         // declare that each explode() element occupies sixteen bytes as a ptr+len string slot
    emitter.instruction("call __rt_array_new");                                 // allocate the initial indexed array that will receive each extracted string segment
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the indexed-array pointer because every push helper may reallocate it

    emitter.instruction("mov rcx, QWORD PTR [rbp - 64]");                       // reload the raw $limit before classifying it
    emitter.instruction("cmp rcx, 0");                                          // classify the limit as positive, zero, or negative
    emitter.instruction("jg __rt_explode_cap_positive_linux_x86_64");           // a positive limit is already the element cap
    emitter.instruction("jl __rt_explode_cap_negative_linux_x86_64");           // a negative limit drops that many trailing segments
    emitter.instruction("mov rcx, 1");                                          // PHP treats $limit === 0 exactly like $limit === 1
    emitter.label("__rt_explode_cap_positive_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 64], rcx");                       // publish the element cap
    emitter.instruction("mov QWORD PTR [rbp - 72], 1");                         // positive limits let the final element absorb the rest of the subject
    emitter.instruction("jmp __rt_explode_scan_init_linux_x86_64");             // start emitting segments

    emitter.label("__rt_explode_cap_negative_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 88], 1");                         // a subject with no delimiter still holds one segment
    emitter.instruction("mov QWORD PTR [rbp - 96], 0");                         // count from the start of the subject
    emitter.label("__rt_explode_count_loop_linux_x86_64");
    emitter.instruction("call __rt_explode_find_linux_x86_64");                 // locate the next delimiter occurrence
    emitter.instruction("cmp rax, 0");                                          // did the scan run out of delimiters?
    emitter.instruction("jl __rt_explode_count_done_linux_x86_64");             // stop counting once no delimiter remains
    emitter.instruction("mov rcx, QWORD PTR [rbp - 88]");                       // reload the running segment total
    emitter.instruction("add rcx, 1");                                          // one delimiter introduces one more segment
    emitter.instruction("mov QWORD PTR [rbp - 88], rcx");                       // publish the updated segment total
    emitter.instruction("add rax, QWORD PTR [rbp - 16]");                       // resume counting after the matched delimiter
    emitter.instruction("mov QWORD PTR [rbp - 96], rax");                       // publish the next delimiter-scan start
    emitter.instruction("jmp __rt_explode_count_loop_linux_x86_64");            // continue counting delimiter occurrences
    emitter.label("__rt_explode_count_done_linux_x86_64");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 88]");                       // reload the final segment total
    emitter.instruction("add rcx, QWORD PTR [rbp - 64]");                       // cap = segment total + negative limit
    emitter.instruction("cmp rcx, 0");                                          // does the limit drop every segment?
    emitter.instruction("jle __rt_explode_return_array_linux_x86_64");          // PHP returns an empty array when it does
    emitter.instruction("mov QWORD PTR [rbp - 64], rcx");                       // publish the element cap
    emitter.instruction("mov QWORD PTR [rbp - 72], 0");                         // negative limits never extend the final element

    emitter.label("__rt_explode_scan_init_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // scan position starts at the beginning of the subject
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // first segment starts at the beginning of the subject
    emitter.instruction("mov QWORD PTR [rbp - 80], 0");                         // no elements have been emitted yet

    emitter.label("__rt_explode_loop_linux_x86_64");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 80]");                       // reload the emitted element count
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 64]");                       // has the limit already been reached?
    emitter.instruction("jge __rt_explode_return_array_linux_x86_64");          // stop without a trailing element when it has
    emitter.instruction("cmp QWORD PTR [rbp - 72], 0");                         // reload the extend-last-element rule
    emitter.instruction("je __rt_explode_next_delim_linux_x86_64");             // negative limits always emit plain segments
    emitter.instruction("add rcx, 1");                                          // would this element be the last one the limit allows?
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 64]");                       // compare the prospective count against the cap
    emitter.instruction("jge __rt_explode_last_linux_x86_64");                  // the last allowed element absorbs the remaining suffix

    emitter.label("__rt_explode_next_delim_linux_x86_64");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the current scan position
    emitter.instruction("mov QWORD PTR [rbp - 96], rcx");                       // hand it to the delimiter-scan subroutine
    emitter.instruction("call __rt_explode_find_linux_x86_64");                 // locate the next delimiter occurrence
    emitter.instruction("cmp rax, 0");                                          // did the scan run out of delimiters?
    emitter.instruction("jl __rt_explode_last_linux_x86_64");                   // the remaining suffix becomes the final element
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // remember where the matched delimiter starts

    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // move the indexed-array pointer into the x86_64 receiver register expected by the string-append helper
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // reload the subject-string pointer before forming the segment substring pointer
    emitter.instruction("mov r8, QWORD PTR [rbp - 56]");                        // reload the current segment start position before computing the substring pointer and length
    emitter.instruction("add rsi, r8");                                         // compute the segment substring pointer from the subject base plus the segment start offset
    emitter.instruction("mov rdx, rax");                                        // seed the segment length with the matched delimiter position
    emitter.instruction("sub rdx, r8");                                         // convert that position into the segment length by subtracting the segment start offset
    emitter.instruction("call __rt_array_push_str");                            // append the subject segment that precedes the matched delimiter to the indexed result array
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the possibly-reallocated indexed-array pointer returned by the string-append helper
    emitter.instruction("mov rcx, QWORD PTR [rbp - 80]");                       // reload the emitted element count
    emitter.instruction("add rcx, 1");                                          // one more element has been emitted
    emitter.instruction("mov QWORD PTR [rbp - 80], rcx");                       // publish the updated emitted count
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the matched delimiter position
    emitter.instruction("add rcx, QWORD PTR [rbp - 16]");                       // advance past the full matched delimiter
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // publish the advanced scan position
    emitter.instruction("mov QWORD PTR [rbp - 56], rcx");                       // start the next segment immediately after the matched delimiter
    emitter.instruction("jmp __rt_explode_loop_linux_x86_64");                  // continue scanning for subsequent delimiter occurrences

    emitter.label("__rt_explode_last_linux_x86_64");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // move the indexed-array pointer into the x86_64 receiver register expected by the string-append helper
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // reload the subject-string pointer before forming the trailing substring pointer
    emitter.instruction("mov r8, QWORD PTR [rbp - 56]");                        // reload the trailing segment start position saved after the last delimiter match
    emitter.instruction("add rsi, r8");                                         // compute the trailing segment pointer from the subject base plus the segment start offset
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // seed the trailing segment length with the full subject-string length
    emitter.instruction("sub rdx, r8");                                         // compute the trailing segment length from the full subject length minus the segment start offset
    emitter.instruction("call __rt_array_push_str");                            // append the trailing subject segment to the indexed result array
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the possibly-reallocated indexed-array pointer returned by the string-append helper

    emitter.label("__rt_explode_return_array_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // return the indexed explode() result array pointer
    emitter.instruction("add rsp, 112");                                        // release the splitter locals after the result array is final
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the indexed explode() result array
    emitter.instruction("ret");                                                 // return the indexed explode() result array pointer in the standard x86_64 integer result register

    emitter.comment("--- runtime: explode delimiter scan (local subroutine) ---");
    emitter.label("__rt_explode_find_linux_x86_64");
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                        // reload the delimiter length before scanning
    emitter.instruction("test r8, r8");                                         // is the separator zero-length?
    emitter.instruction("jz __rt_explode_find_none_linux_x86_64");              // a zero-length separator can never match
    emitter.instruction("mov rax, QWORD PTR [rbp - 96]");                       // load the requested scan start position
    emitter.label("__rt_explode_find_loop_linux_x86_64");
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // reload the subject-string length before computing the remaining scan window
    emitter.instruction("sub r9, rax");                                         // compute the number of subject bytes remaining at the current scan position
    emitter.instruction("cmp r8, r9");                                          // does the delimiter still fit in the remaining suffix?
    emitter.instruction("jg __rt_explode_find_none_linux_x86_64");              // report no match once the delimiter no longer fits
    emitter.instruction("xor r10, r10");                                        // start the delimiter-comparison byte index at zero
    emitter.label("__rt_explode_find_cmp_linux_x86_64");
    emitter.instruction("cmp r10, r8");                                         // stop comparing once every delimiter byte has matched
    emitter.instruction("jae __rt_explode_find_hit_linux_x86_64");              // treat the current scan position as a delimiter hit
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // reload the subject-string pointer before reading the candidate byte
    emitter.instruction("add r11, rax");                                        // advance to the current scan position inside the subject
    emitter.instruction("movzx ecx, BYTE PTR [r11 + r10]");                     // load the subject byte that should match the delimiter byte
    emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                        // reload the delimiter pointer before reading the delimiter byte
    emitter.instruction("movzx r9d, BYTE PTR [r11 + r10]");                     // load the delimiter byte for the current comparison index
    emitter.instruction("cmp ecx, r9d");                                        // compare the subject and delimiter bytes
    emitter.instruction("jne __rt_explode_find_next_linux_x86_64");             // abandon the current scan position on any mismatch
    emitter.instruction("add r10, 1");                                          // advance the delimiter-comparison byte index
    emitter.instruction("jmp __rt_explode_find_cmp_linux_x86_64");              // continue comparing the remaining delimiter bytes
    emitter.label("__rt_explode_find_next_linux_x86_64");
    emitter.instruction("add rax, 1");                                          // advance the scan position by one subject byte
    emitter.instruction("jmp __rt_explode_find_loop_linux_x86_64");             // continue scanning the subject for the next delimiter occurrence
    emitter.label("__rt_explode_find_hit_linux_x86_64");
    emitter.instruction("ret");                                                 // return the matched delimiter position already held in rax
    emitter.label("__rt_explode_find_none_linux_x86_64");
    emitter.instruction("mov rax, -1");                                         // report that no delimiter remains
    emitter.instruction("ret");                                                 // return to the explode() scan loop
}
