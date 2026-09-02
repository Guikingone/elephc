//! Purpose:
//! Emits the `__rt_preg_match_all`, `__rt_preg_strip` runtime helper assembly for preg match all.
//! Keeps PHP builtin semantics, libc/syscall boundaries, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - Regex helpers preserve PHP PCRE-flavored inputs for PCRE2 and must preserve match array construction.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// __rt_preg_match_all: count all non-overlapping matches of regex in subject.
/// Input:  x1=pattern ptr, x2=pattern len, x3=subject ptr, x4=subject len
/// Output: x0=match count
pub(crate) fn emit_preg_match_all(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_preg_match_all_linux_x86_64(emitter);
        emit_preg_match_all_capture_linux_x86_64(emitter);
        return;
    }

    let handle_off = 0;
    let match_slot_count_off = handle_off + 8;
    let match_pair_off = match_slot_count_off + 8;
    let pattern_ptr_off = match_pair_off + 16;
    let pattern_len_off = pattern_ptr_off + 8;
    let subject_ptr_off = pattern_len_off + 8;
    let subject_len_off = subject_ptr_off + 8;
    let flags_off = subject_len_off + 8;
    let pattern_cstr_off = flags_off + 8;
    let subject_cstr_off = pattern_cstr_off + 8;
    let match_count_off = subject_cstr_off + 8;
    let current_pos_off = match_count_off + 8;
    let stack_size = (current_pos_off + 48 + 15) & !15;
    let save_off = stack_size - 16;

    emitter.blank();
    emitter.comment("--- runtime: preg_match_all ---");
    emitter.label_global("__rt_preg_match_all");

    // -- set up stack frame --
    emitter.instruction(&format!("sub sp, sp, #{}", stack_size));               // allocate preg_match_all stack frame
    emitter.instruction(&format!("stp x29, x30, [sp, #{}]", save_off));         // save frame pointer and return address
    emitter.instruction(&format!("add x29, sp, #{}", save_off));                // set new frame pointer

    // -- save inputs --
    emitter.instruction(&format!("str x1, [sp, #{}]", pattern_ptr_off));        // save pattern ptr
    emitter.instruction(&format!("str x2, [sp, #{}]", pattern_len_off));        // save pattern len
    emitter.instruction(&format!("str x3, [sp, #{}]", subject_ptr_off));        // save subject ptr
    emitter.instruction(&format!("str x4, [sp, #{}]", subject_len_off));        // save subject len

    // -- strip delimiters --
    emitter.instruction("bl __rt_preg_strip");                                  // → x1, x2, x3=flags
    emitter.instruction(&format!("str x3, [sp, #{}]", flags_off));              // save flags

    // -- materialize the PCRE pattern as a C string --
    emitter.instruction("bl __rt_pcre_to_posix");                               // materialize PCRE pattern as a C string
    emitter.instruction(&format!("str x0, [sp, #{}]", pattern_cstr_off));       // save pattern C string

    // -- prepare locale state for regex helpers --
    super::emit_prepare_regex_locale(emitter);

    // -- compile regex --
    emitter.instruction(&format!("add x0, sp, #{}", handle_off));               // pass opaque-handle output storage
    emitter.instruction(&format!("ldr x1, [sp, #{}]", pattern_cstr_off));       // pass null-terminated pattern
    emitter.instruction(&format!("ldr x2, [sp, #{}]", flags_off));              // pass PCRE2 POSIX compile flags from delimiter parsing
    emitter.instruction(&format!("add x3, sp, #{}", match_slot_count_off));     // receive the compiled match-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile without exposing PCRE2-owned layouts
    emitter.instruction("cbnz x0, __rt_preg_match_all_fail");                   // fail

    // -- null-terminate subject --
    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_ptr_off));        // subject ptr
    emitter.instruction(&format!("ldr x2, [sp, #{}]", subject_len_off));        // subject len
    emitter.instruction("bl __rt_cstr2");                                       // → x0=subject C string
    emitter.instruction(&format!("str x0, [sp, #{}]", subject_cstr_off));       // save subject C string

    // -- count matches loop --
    emitter.instruction(&format!("str xzr, [sp, #{}]", match_count_off));       // match count = 0
    emitter.instruction(&format!("ldr x9, [sp, #{}]", subject_cstr_off));       // current position = start
    emitter.instruction(&format!("str x9, [sp, #{}]", current_pos_off));        // save current pos

    emitter.label("__rt_preg_match_all_loop");
    emitter.instruction(&format!("ldr x1, [sp, #{}]", current_pos_off));        // current subject position
    emitter.instruction("ldrb w9, [x1]");                                       // load byte at current pos
    emitter.instruction("cbz w9, __rt_preg_match_all_done");                    // null terminator = done
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // pass compiled opaque handle
    emitter.instruction("mov x2, #1");                                          // request only the full-match pair
    emitter.instruction(&format!("add x3, sp, #{}", match_pair_off));           // receive one fixed signed-64-bit offset pair
    emitter.instruction("mov x4, #0");                                          // use default execution flags
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // execute without exposing PCRE2-owned layouts
    emitter.instruction("cbnz x0, __rt_preg_match_all_done");                   // no more matches

    // -- found a match, increment count --
    emitter.instruction(&format!("ldr x9, [sp, #{}]", match_count_off));        // load count
    emitter.instruction("add x9, x9, #1");                                      // increment
    emitter.instruction(&format!("str x9, [sp, #{}]", match_count_off));        // save count

    // -- advance past this match --
    emitter.instruction(&format!("ldr x10, [sp, #{}]", current_pos_off));       // current pos
    emitter.instruction(&format!("ldr x11, [sp, #{}]", match_pair_off + 8));    // load fixed signed-64-bit match end
    emitter.instruction("cmp x11, #0");                                         // check for zero-length match
    emitter.instruction("b.gt __rt_preg_match_all_adv");                        // non-zero advance
    emitter.instruction("mov x11, #1");                                         // advance by at least 1
    emitter.label("__rt_preg_match_all_adv");
    emitter.instruction("add x10, x10, x11");                                   // advance position
    emitter.instruction(&format!("str x10, [sp, #{}]", current_pos_off));       // save new position
    emitter.instruction("b __rt_preg_match_all_loop");                          // continue

    emitter.label("__rt_preg_match_all_done");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // reload compiled opaque handle
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources
    emitter.instruction(&format!("ldr x0, [sp, #{}]", match_count_off));        // return count
    emitter.instruction("b __rt_preg_match_all_ret");                           // return

    emitter.label("__rt_preg_match_all_fail");
    emitter.instruction("mov x0, #0");                                          // return 0 on compile failure

    emitter.label("__rt_preg_match_all_ret");
    emitter.instruction(&format!("ldp x29, x30, [sp, #{}]", save_off));         // restore frame pointer and return address
    emitter.instruction(&format!("add sp, sp, #{}", stack_size));               // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    emit_preg_match_all_capture_aarch64(emitter);
}

/// Emits the capture-producing `preg_match_all()` path for AArch64.
///
/// The count-only helper above deliberately keeps its small ABI for callers that
/// omit `$matches`. This sibling consumes the same pattern/subject ABI and
/// returns the pattern-order full-match column in `x1`, while `x0` retains the
/// ordinary match count. Every row is persisted through the ordinary array
/// helpers, so strings and the nested boxed array follow the normal ownership
/// and copy-on-write rules rather than using a regex-specific representation.
fn emit_preg_match_all_capture_aarch64(emitter: &mut Emitter) {
    let handle_off = 0;
    let match_pair_off = handle_off + 8;
    let pattern_ptr_off = match_pair_off + 16;
    let pattern_len_off = pattern_ptr_off + 8;
    let subject_ptr_off = pattern_len_off + 8;
    let subject_len_off = subject_ptr_off + 8;
    let pattern_flags_off = subject_len_off + 8;
    let subject_cstr_off = pattern_flags_off + 8;
    let match_count_off = subject_cstr_off + 8;
    let current_pos_off = match_count_off + 8;
    let outer_matches_off = current_pos_off + 8;
    let full_matches_off = outer_matches_off + 8;
    let full_matches_box_off = full_matches_off + 8;
    let requested_offset_off = full_matches_box_off + 8;
    let stack_size = (requested_offset_off + 32 + 15) & !15;
    let save_off = stack_size - 16;

    emitter.blank();
    emitter.comment("--- runtime: preg_match_all capture ---");
    emitter.label_global("__rt_preg_match_all_capture");
    emitter.instruction(&format!("sub sp, sp, #{}", stack_size));               // reserve regex state plus stable nested-array owners
    emitter.instruction(&format!("stp x29, x30, [sp, #{}]", save_off));         // preserve the generated caller frame
    emitter.instruction(&format!("add x29, sp, #{}", save_off));                // establish a stable helper frame
    emitter.instruction(&format!("str x1, [sp, #{}]", pattern_ptr_off));        // retain pattern bytes across compile and match calls
    emitter.instruction(&format!("str x2, [sp, #{}]", pattern_len_off));        // retain pattern length across compile and match calls
    emitter.instruction(&format!("str x3, [sp, #{}]", subject_ptr_off));        // retain subject bytes across compile and match calls
    emitter.instruction(&format!("str x4, [sp, #{}]", subject_len_off));        // retain subject length across compile and match calls
    emitter.instruction(&format!("str x6, [sp, #{}]", requested_offset_off));  // preserve the public byte offset for the first search

    // PHP always exposes a column for the full match, including the no-match
    // case. Build that `array{0: list<string>}` shape before compiling so every
    // exit path returns a real, ownership-balanced capture matrix.
    emitter.instruction("mov x0, #1");                                          // one outer pattern-order column: capture group zero
    emitter.instruction("mov x1, #8");                                          // outer slots hold boxed Mixed cells
    emitter.instruction("bl __rt_array_new");                                   // allocate the outer `array<mixed>` result
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_matches_off));      // preserve outer storage across nested construction
    emitter.instruction("mov x0, #8");                                          // reserve a modest first capacity for full-match strings
    emitter.instruction("mov x1, #16");                                         // inner slots hold string pointer/length pairs
    emitter.instruction("bl __rt_array_new");                                   // allocate the `matches[0]` string array
    emitter.instruction(&format!("str x0, [sp, #{}]", full_matches_off));       // preserve the raw inner owner while boxing it
    emitter.instruction("mov x1, x0");                                          // pass the inner array as the Mixed payload
    emitter.instruction("mov x2, xzr");                                         // arrays use one payload word
    emitter.instruction("mov x0, #4");                                          // runtime tag 4 identifies indexed arrays
    emitter.instruction("bl __rt_mixed_from_value");                            // create the outer-owned boxed inner array
    emitter.instruction(&format!("str x0, [sp, #{}]", full_matches_box_off));   // save the stable box before consuming it into the outer array
    emitter.instruction(&format!("ldr x0, [sp, #{}]", full_matches_off));       // release the raw construction owner now retained by the box
    emitter.instruction("bl __rt_decref_array");                                // leave the boxed outer slot as the sole owner
    emitter.instruction(&format!("ldr x0, [sp, #{}]", outer_matches_off));      // reload the outer matrix receiver
    emitter.instruction("mov x1, #0");                                          // write the full-match column at numeric key zero
    emitter.instruction(&format!("ldr x2, [sp, #{}]", full_matches_box_off));   // transfer the boxed inner array into the outer matrix
    emitter.instruction("bl __rt_array_set_mixed");                             // publish the first Mixed column and retain COW invariants
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_matches_off));      // preserve a possible outer-array relocation
    emitter.instruction(&format!("str xzr, [sp, #{}]", match_count_off));       // compile failures still return the ordinary zero match count

    emitter.instruction(&format!("ldr x1, [sp, #{}]", pattern_ptr_off));        // restore pattern ABI input for delimiter stripping
    emitter.instruction(&format!("ldr x2, [sp, #{}]", pattern_len_off));        // restore pattern ABI length for delimiter stripping
    emitter.instruction("bl __rt_preg_strip");                                  // normalize delimiters and compile flags
    emitter.instruction(&format!("str x3, [sp, #{}]", pattern_flags_off));      // retain PCRE compilation flags
    emitter.instruction("bl __rt_pcre_to_posix");                               // materialize a C-compatible pattern string
    emitter.instruction(&format!("str x0, [sp, #{}]", pattern_ptr_off));        // reuse pattern storage for the C-string pointer
    super::emit_prepare_regex_locale(emitter);
    emitter.instruction(&format!("add x0, sp, #{}", handle_off));               // provide opaque compiled-pattern output storage
    emitter.instruction(&format!("ldr x1, [sp, #{}]", pattern_ptr_off));        // pass the C-compatible pattern to the managed PCRE bridge
    emitter.instruction(&format!("ldr x2, [sp, #{}]", pattern_flags_off));      // pass delimiter-derived compilation flags
    emitter.instruction("add x3, sp, #24");                                     // count-only execution needs one full-match offset pair
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile the pattern through the managed PCRE bridge
    emitter.instruction("cbnz x0, __rt_preg_match_all_capture_return_empty");   // invalid patterns return zero and the initialized capture matrix

    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_ptr_off));        // reload subject bytes for C-string materialization
    emitter.instruction(&format!("ldr x2, [sp, #{}]", subject_len_off));        // reload subject length for C-string materialization
    emitter.instruction("bl __rt_cstr2");                                       // make one null-terminated subject copy for repeated searches
    emitter.instruction(&format!("str x0, [sp, #{}]", subject_cstr_off));       // preserve the subject base across PCRE calls
    emitter.instruction(&format!("ldr x9, [sp, #{}]", requested_offset_off));  // load the public offset before normalizing it
    emitter.instruction("cmp x9, #0");                                          // negative offsets are relative to the end of the byte string
    emitter.instruction("b.ge __rt_preg_match_all_capture_offset_ready");       // positive offsets are already absolute
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_len_off));       // load subject length for negative offset normalization
    emitter.instruction("add x9, x10, x9");                                     // convert a negative offset into a byte position
    emitter.label("__rt_preg_match_all_capture_offset_ready");
    emitter.instruction("cmp x9, #0");                                          // PHP rejects offsets before the start of the subject
    emitter.instruction("b.lt __rt_preg_match_all_capture_done");               // no match when the requested start lies before byte zero
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_len_off));       // load the upper bound for offset validation
    emitter.instruction("cmp x9, x10");                                         // the trailing byte boundary is valid
    emitter.instruction("b.gt __rt_preg_match_all_capture_done");               // no match when the offset is beyond the subject
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_cstr_off));      // recover the subject C-string base
    emitter.instruction("add x10, x10, x9");                                    // compute the first PCRE search cursor
    emitter.instruction(&format!("str x10, [sp, #{}]", current_pos_off));       // persist the cursor across loop calls
    emitter.instruction(&format!("str xzr, [sp, #{}]", match_count_off));       // initialize the completed-match count

    emitter.label("__rt_preg_match_all_capture_loop");
    emitter.instruction(&format!("ldr x1, [sp, #{}]", current_pos_off));        // load the current subject suffix cursor
    emitter.instruction("ldrb w9, [x1]");                                       // detect the trailing null terminator
    emitter.instruction("cbz w9, __rt_preg_match_all_capture_done");            // no bytes left means no further match
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // pass the compiled pattern handle
    emitter.instruction("mov x2, #1");                                          // request only the full-match capture pair
    emitter.instruction("add x3, sp, #8");                                      // receive the pair after the opaque handle slot
    emitter.instruction("mov x4, #0");                                          // search the current suffix with ordinary PCRE flags
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // find the next non-overlapping full match
    emitter.instruction("cbnz x0, __rt_preg_match_all_capture_done");           // a non-zero status ends the global search
    emitter.instruction("ldr x9, [sp, #8]");                                    // load the match start offset within the current suffix
    emitter.instruction("ldr x10, [sp, #16]");                                  // load the match end offset within the current suffix
    emitter.instruction("sub x2, x10, x9");                                     // derive the matched byte length before helper calls
    emitter.instruction(&format!("ldr x1, [sp, #{}]", current_pos_off));        // reload the current suffix base
    emitter.instruction("add x1, x1, x9");                                      // derive the matched string pointer
    emitter.instruction(&format!("ldr x0, [sp, #{}]", full_matches_off));       // reload the inner full-match array receiver
    emitter.instruction("bl __rt_array_push_str");                              // persist and append the captured full-match string
    emitter.instruction(&format!("str x0, [sp, #{}]", full_matches_off));       // preserve a possible inner-array relocation
    emitter.instruction(&format!("ldr x9, [sp, #{}]", outer_matches_off));      // reload the fixed outer matrix for box writeback
    emitter.instruction("ldr x9, [x9, #24]");                                   // locate the box stored at outer numeric key zero
    emitter.instruction("str x0, [x9, #8]");                                    // publish the relocated inner array in its owning Mixed cell
    emitter.instruction(&format!("ldr x9, [sp, #{}]", match_count_off));       // reload the completed match count
    emitter.instruction("add x9, x9, #1");                                      // count the newly appended match
    emitter.instruction(&format!("str x9, [sp, #{}]", match_count_off));       // save the updated count across the next PCRE call
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the match end after the array append clobbered caller registers
    emitter.instruction("cmp x10, #0");                                         // zero-length matches must still make progress
    emitter.instruction("b.gt __rt_preg_match_all_capture_advance");            // a positive end offset is the normal next suffix boundary
    emitter.instruction("mov x10, #1");                                         // force one-byte progress for a zero-length match
    emitter.label("__rt_preg_match_all_capture_advance");
    emitter.instruction(&format!("ldr x9, [sp, #{}]", current_pos_off));        // reload the current suffix pointer
    emitter.instruction("add x9, x9, x10");                                     // move past the complete current match
    emitter.instruction(&format!("str x9, [sp, #{}]", current_pos_off));       // persist the next PCRE suffix cursor
    emitter.instruction("b __rt_preg_match_all_capture_loop");                  // continue until PCRE finds no next match

    emitter.label("__rt_preg_match_all_capture_done");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // release the compiled PCRE resource on every successful compile path
    emitter.bl_c("elephc_pcre2_v1_free");                                       // dispose of the opaque compiled pattern
    emitter.label("__rt_preg_match_all_capture_return_empty");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", match_count_off));       // return the integer match count
    emitter.instruction(&format!("ldr x1, [sp, #{}]", outer_matches_off));     // return the initialized pattern-order matrix beside the count
    emitter.instruction(&format!("ldp x29, x30, [sp, #{}]", save_off));         // restore the generated caller frame
    emitter.instruction(&format!("add sp, sp, #{}", stack_size));               // release all helper scratch space
    emitter.instruction("ret");                                                 // return count and capture matrix to lowering
}

/// Target-specific implementation of `__rt_preg_match_all` for Linux x86_64.
/// Uses the System V AMD64 ABI: pattern ptr in rdi, pattern len in rsi, subject ptr in rdx, subject len in rcx.
/// Returns the non-overlapping match count in rax.
/// Handles zero-length matches by advancing at least one byte to avoid infinite loops.
fn emit_preg_match_all_linux_x86_64(emitter: &mut Emitter) {
    let handle_off = 0;
    let match_slot_count_off = handle_off + 8;
    let match_pair_off = match_slot_count_off + 8;
    let subject_ptr_off = match_pair_off + 16;
    let subject_len_off = subject_ptr_off + 8;
    let flags_off = subject_len_off + 8;
    let pattern_cstr_off = flags_off + 8;
    let subject_cstr_off = pattern_cstr_off + 8;
    let match_count_off = subject_cstr_off + 8;
    let current_pos_off = match_count_off + 8;
    let stack_size = (current_pos_off + 16 + 15) & !15;
    emitter.blank();
    emitter.comment("--- runtime: preg_match_all ---");
    emitter.label_global("__rt_preg_match_all");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving regex-counting scratch storage
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the regex object, regmatch buffer, and loop spill slots
    emitter.instruction(&format!("sub rsp, {}", stack_size));                   // reserve aligned local storage for the opaque handle, pair, and match count
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdx", subject_ptr_off)); // preserve the elephc subject pointer across delimiter stripping and regex compilation helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", subject_len_off)); // preserve the elephc subject length across delimiter stripping and regex compilation helper calls
    emitter.instruction("mov rax, rdi");                                        // move the elephc pattern pointer into the delimiter-strip helper input register
    emitter.instruction("mov rdx, rsi");                                        // move the elephc pattern length into the delimiter-strip helper input register
    emitter.instruction("call __rt_preg_strip");                                // strip slash delimiters and gather supported regex flags from the pattern literal
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", flags_off));  // preserve the delimiter-strip helper flags for the later regcomp() call
    emitter.instruction("call __rt_pcre_to_posix");                             // materialize PCRE pattern as a null-terminated C string
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", pattern_cstr_off)); // preserve the null-terminated PCRE pattern C string across compilation and loop setup
    super::emit_prepare_regex_locale(emitter);
    emitter.instruction(&format!("lea rdi, [rsp + {}]", handle_off));           // pass opaque-handle output storage
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", pattern_cstr_off)); // pass the null-terminated PCRE pattern C string as the second regcomp() argument
    emitter.instruction(&format!("mov edx, DWORD PTR [rsp + {}]", flags_off));  // pass PCRE2 POSIX compile flags from delimiter parsing
    emitter.instruction(&format!("lea rcx, [rsp + {}]", match_slot_count_off)); // receive the compiled match-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile without exposing PCRE2-owned layouts
    emitter.instruction("test eax, eax");                                       // did regcomp() succeed and produce a compiled regex object?
    emitter.instruction("jnz __rt_preg_match_all_fail_linux_x86_64");           // failed regex compilation maps to a zero-count result
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", subject_ptr_off)); // reload the elephc subject pointer before null-terminating it in the secondary scratch buffer
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", subject_len_off)); // reload the elephc subject length before null-terminating it in the secondary scratch buffer
    emitter.instruction("call __rt_cstr2");                                     // materialize a null-terminated subject C string for repeated PCRE2 regex execution probes
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", subject_cstr_off)); // preserve the subject C string pointer across the full match-counting loop
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", match_count_off)); // initialize the running non-overlapping match count at zero
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", current_pos_off)); // start the current subject cursor at the beginning of the null-terminated subject C string

    emitter.label("__rt_preg_match_all_loop_linux_x86_64");
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", current_pos_off)); // reload the current subject C-string cursor before the next regexec() probe
    emitter.instruction("movzx r9d, BYTE PTR [rsi]");                           // check whether the current subject cursor already points at the trailing null terminator
    emitter.instruction("test r9d, r9d");                                       // treat the terminating null byte as the loop completion condition
    emitter.instruction("jz __rt_preg_match_all_done_linux_x86_64");            // stop counting once the full null-terminated subject has been consumed
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // pass compiled opaque handle
    emitter.instruction("mov edx, 1");                                          // request only the full-match pair
    emitter.instruction(&format!("lea rcx, [rsp + {}]", match_pair_off));       // receive one fixed signed-64-bit offset pair
    emitter.instruction("xor r8d, r8d");                                        // use default execution flags
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // execute without exposing PCRE2-owned layouts
    emitter.instruction("test eax, eax");                                       // did regexec() find another match at or after the current cursor?
    emitter.instruction("jnz __rt_preg_match_all_done_linux_x86_64");           // stop counting when regexec() reports no further matches
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", match_count_off)); // reload the running non-overlapping match count before incrementing it
    emitter.instruction("add r9, 1");                                           // count the newly discovered regex match
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", match_count_off)); // preserve the updated match count for the next loop iteration
    emitter.instruction(&format!("mov r11, QWORD PTR [rsp + {}]", match_pair_off + 8)); // load fixed signed-64-bit match end
    emitter.instruction("cmp r11, 0");                                          // detect zero-length regex matches so the loop still makes forward progress
    emitter.instruction("jg __rt_preg_match_all_adv_linux_x86_64");             // use the reported end offset directly when the regex consumed at least one byte
    emitter.instruction("mov r11, 1");                                          // force zero-length matches to advance by one byte and avoid infinite loops
    emitter.label("__rt_preg_match_all_adv_linux_x86_64");
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", current_pos_off)); // reload the current subject cursor before advancing it past the latest regex match
    emitter.instruction("add r10, r11");                                        // advance the current subject cursor by rm_eo bytes or the forced one-byte fallback
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r10", current_pos_off)); // preserve the advanced subject cursor for the next loop iteration
    emitter.instruction("jmp __rt_preg_match_all_loop_linux_x86_64");           // continue counting the remaining non-overlapping regex matches

    emitter.label("__rt_preg_match_all_done_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // reload compiled opaque handle
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources before returning the match count
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", match_count_off)); // return the total number of non-overlapping regex matches discovered in the subject
    emitter.instruction("jmp __rt_preg_match_all_ret_linux_x86_64");            // share the common epilogue after the successful match-count path

    emitter.label("__rt_preg_match_all_fail_linux_x86_64");
    emitter.instruction("xor eax, eax");                                        // return zero when regex compilation fails for preg_match_all()

    emitter.label("__rt_preg_match_all_ret_linux_x86_64");
    emitter.instruction(&format!("add rsp, {}", stack_size));                   // release the opaque-handle, pair, and counting spill storage before returning
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the regex-count helper completes
    emitter.instruction("ret");                                                 // return the preg_match_all() integer count in the x86_64 integer result register
}

/// Emits the capture-producing `preg_match_all()` path for Linux x86_64.
///
/// It is the System-V counterpart of [`emit_preg_match_all_capture_aarch64`]:
/// callers that supply `$matches` receive a pattern-order full-match column in
/// `rdx` in addition to the ordinary count in `rax`.
fn emit_preg_match_all_capture_linux_x86_64(emitter: &mut Emitter) {
    let handle = 0;
    let pair = handle + 8;
    let pattern = pair + 16;
    let pattern_len = pattern + 8;
    let subject = pattern_len + 8;
    let subject_len = subject + 8;
    let flags = subject_len + 8;
    let subject_cstr = flags + 8;
    let count = subject_cstr + 8;
    let cursor = count + 8;
    let outer = cursor + 8;
    let inner = outer + 8;
    let boxed_inner = inner + 8;
    let requested_offset = boxed_inner + 8;
    let stack_size = (requested_offset + 16 + 15) & !15;

    emitter.blank();
    emitter.comment("--- runtime: preg_match_all capture ---");
    emitter.label_global("__rt_preg_match_all_capture");
    emitter.instruction("push rbp");                                            // preserve the generated caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame
    emitter.instruction(&format!("sub rsp, {}", stack_size));                   // reserve PCRE state and nested-array owners
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdi", pattern));    // retain the pattern bytes across helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rsi", pattern_len)); // retain the pattern length across helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdx", subject));    // retain the subject bytes across helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", subject_len)); // retain the subject length across helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", requested_offset)); // retain the public starting offset

    emitter.instruction("mov edi, 1");                                          // one pattern-order column for the full match
    emitter.instruction("mov esi, 8");                                          // outer slots are boxed Mixed cells
    emitter.instruction("call __rt_array_new");                                 // allocate outer `array<mixed>` storage
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // preserve the outer result across nested setup
    emitter.instruction("mov edi, 8");                                          // reserve initial space for full-match strings
    emitter.instruction("mov esi, 16");                                         // inner slots hold string pointer/length pairs
    emitter.instruction("call __rt_array_new");                                 // allocate `matches[0]`
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", inner));      // preserve its raw construction owner
    emitter.instruction("mov rdi, rax");                                        // pass the inner array as the Mixed payload
    emitter.instruction("xor esi, esi");                                        // arrays have a single payload word
    emitter.instruction("mov eax, 4");                                          // indexed-array runtime tag
    emitter.instruction("call __rt_mixed_from_value");                          // create the outer-owned boxed column
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", boxed_inner)); // keep the box before the consuming array write
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", inner));      // drop the raw construction owner now retained by the box
    emitter.instruction("call __rt_decref_array");                              // leave the box as the sole inner-array owner
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", outer));      // reload the outer matrix receiver
    emitter.instruction("xor esi, esi");                                        // set numeric key zero
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", boxed_inner)); // transfer the Mixed column into the matrix
    emitter.instruction("call __rt_array_set_mixed");                           // publish the nested column with normal COW semantics
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // preserve a possible outer relocation
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", count));        // all early exits return a zero match count

    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", pattern));    // restore delimiter-strip input pattern
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", pattern_len)); // restore delimiter-strip input length
    emitter.instruction("call __rt_preg_strip");                                // normalize delimiters and extract compile flags
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", flags));      // preserve the derived compile flags
    emitter.instruction("call __rt_pcre_to_posix");                             // materialize a C-compatible pattern string
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", pattern));    // reuse the pattern slot for its C-string pointer
    super::emit_prepare_regex_locale(emitter);
    emitter.instruction(&format!("lea rdi, [rsp + {}]", handle));               // provide compiled-pattern output storage
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", pattern));    // pass the C-compatible pattern
    emitter.instruction(&format!("mov edx, DWORD PTR [rsp + {}]", flags));     // pass delimiter-derived flags
    emitter.instruction(&format!("lea rcx, [rsp + {}]", pattern));              // the old pattern slot receives an unused match-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile via the managed PCRE bridge
    emitter.instruction("test eax, eax");                                       // did compilation succeed?
    emitter.instruction("jnz __rt_preg_match_all_capture_return_empty_linux_x86_64"); // invalid patterns retain the initialized empty matrix

    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", subject));    // reload subject bytes for C-string materialization
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", subject_len)); // reload subject length for C-string materialization
    emitter.instruction("call __rt_cstr2");                                     // construct one stable null-terminated subject copy
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", subject_cstr)); // preserve its base across PCRE calls
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", requested_offset)); // load the requested byte offset
    emitter.instruction("test r9, r9");                                         // negative offsets are relative to subject end
    emitter.instruction("jns __rt_preg_match_all_capture_offset_ready_linux_x86_64"); // keep non-negative offsets unchanged
    emitter.instruction(&format!("add r9, QWORD PTR [rsp + {}]", subject_len)); // normalize a negative byte offset
    emitter.label("__rt_preg_match_all_capture_offset_ready_linux_x86_64");
    emitter.instruction("test r9, r9");                                         // reject offsets before byte zero
    emitter.instruction("js __rt_preg_match_all_capture_done_linux_x86_64");   // invalid offsets yield no match
    emitter.instruction(&format!("cmp r9, QWORD PTR [rsp + {}]", subject_len)); // the trailing boundary remains valid
    emitter.instruction("jg __rt_preg_match_all_capture_done_linux_x86_64");   // reject offsets beyond the subject
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", subject_cstr)); // load the subject C-string base
    emitter.instruction("add rax, r9");                                         // form the first search suffix
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", cursor));     // preserve the PCRE search cursor

    emitter.label("__rt_preg_match_all_capture_loop_linux_x86_64");
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", cursor));     // load the current subject suffix
    emitter.instruction("cmp BYTE PTR [rsi], 0");                               // stop at the terminating null byte
    emitter.instruction("je __rt_preg_match_all_capture_done_linux_x86_64");   // there are no more candidate bytes
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle));     // pass the compiled pattern handle
    emitter.instruction("mov edx, 1");                                          // request the full-match pair only
    emitter.instruction(&format!("lea rcx, [rsp + {}]", pair));                 // receive start/end offsets in the current suffix
    emitter.instruction("xor r8d, r8d");                                        // use ordinary PCRE execution flags
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // search for the next non-overlapping match
    emitter.instruction("test eax, eax");                                       // did PCRE find a match?
    emitter.instruction("jnz __rt_preg_match_all_capture_done_linux_x86_64");  // a non-zero result ends the global search
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", pair));        // load match start within the current suffix
    emitter.instruction(&format!("mov rcx, QWORD PTR [rsp + {}]", pair + 8));   // load match end within the current suffix
    emitter.instruction("mov rdx, rcx");                                        // preserve the end while deriving the captured length
    emitter.instruction("sub rdx, r9");                                         // calculate full-match byte length
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", cursor));     // reload the current suffix base
    emitter.instruction("add rsi, r9");                                         // calculate the captured string pointer
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", inner));      // reload the full-match string-array receiver
    emitter.instruction("call __rt_array_push_str");                            // persist and append the captured full match
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", inner));      // preserve a possible inner-array relocation
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", outer));      // reload the fixed outer matrix
    emitter.instruction("mov r9, QWORD PTR [r9 + 24]");                         // locate the owning Mixed cell at key zero
    emitter.instruction("mov QWORD PTR [r9 + 8], rax");                         // update the boxed inner payload after COW relocation
    emitter.instruction(&format!("add QWORD PTR [rsp + {}], 1", count));        // count the newly appended match
    emitter.instruction(&format!("mov rcx, QWORD PTR [rsp + {}]", pair + 8));   // reload match end clobbered by the append helper
    emitter.instruction("test rcx, rcx");                                       // zero-length matches need forced progress
    emitter.instruction("jg __rt_preg_match_all_capture_advance_linux_x86_64"); // ordinary matches advance to their end offset
    emitter.instruction("mov ecx, 1");                                          // force one-byte progress for an empty match
    emitter.label("__rt_preg_match_all_capture_advance_linux_x86_64");
    emitter.instruction(&format!("add QWORD PTR [rsp + {}], rcx", cursor));     // move the suffix cursor past the full match
    emitter.instruction("jmp __rt_preg_match_all_capture_loop_linux_x86_64");  // continue until PCRE reports no next match

    emitter.label("__rt_preg_match_all_capture_done_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle));     // reload the compiled pattern after a successful compile
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release the opaque PCRE resource
    emitter.label("__rt_preg_match_all_capture_return_empty_linux_x86_64");
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", count));      // return the standard integer match count
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", outer));      // return the pattern-order full-match matrix beside it
    emitter.instruction(&format!("add rsp, {}", stack_size));                   // release helper scratch storage
    emitter.instruction("pop rbp");                                             // restore the generated caller frame
    emitter.instruction("ret");                                                 // return count and capture matrix to lowering
}
