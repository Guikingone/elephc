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
    // The END position is a legal search start, not a stop condition: PHP reports a zero-length
    // match there, so `preg_match_all('/x*/', 'axb')` counts four, not three. Stopping at the
    // null byte dropped that last one.
    emitter.instruction(&format!("ldr x9, [sp, #{}]", subject_cstr_off));       // load the subject base for the bound
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_len_off));       // load the subject length for the bound
    emitter.instruction("add x9, x9, x10");                                     // the last legal search start is one past the last byte
    emitter.instruction("cmp x1, x9");                                          // has the cursor moved beyond it?
    emitter.instruction("b.hi __rt_preg_match_all_done");                       // stop only once the cursor is past the end
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
/// The count-only helper above keeps its small ABI for callers that omit `$matches`. This
/// sibling asks PCRE2 for the pattern's REAL capture-slot count and name table, then builds the
/// same `$matches` PHP does:
///
/// * `PREG_PATTERN_ORDER` (the default) gives every compiled group its own column, including a
///   group that never participates — PHP still emits its column, filled with empty strings —
///   and a declared group name is written immediately before the numeric key it doubles.
/// * `PREG_SET_ORDER` gives one row per match, trimmed after the highest group that actually
///   participated in THAT match, which is exactly what `__rt_preg_match_row` builds.
///
/// Every row is persisted through the ordinary array helpers, so strings and nested arrays
/// follow the normal ownership and copy-on-write rules rather than a regex-specific layout.
///
/// Input:  x1=pattern ptr, x2=pattern len, x3=subject ptr, x4=subject len,
///         x5=PHP `$flags`, x6=starting byte offset
/// Output: x0=match count, x1=`$matches` storage owned by the caller
fn emit_preg_match_all_capture_aarch64(emitter: &mut Emitter) {
    let handle_off = 0;
    let pairs_ptr_off = handle_off + 8;
    let nmatch_off = pairs_ptr_off + 8;
    let pattern_ptr_off = nmatch_off + 8;
    let pattern_len_off = pattern_ptr_off + 8;
    let subject_ptr_off = pattern_len_off + 8;
    let subject_len_off = subject_ptr_off + 8;
    let php_flags_off = subject_len_off + 8;
    let pattern_flags_off = php_flags_off + 8;
    let subject_cstr_off = pattern_flags_off + 8;
    let match_count_off = subject_cstr_off + 8;
    let current_pos_off = match_count_off + 8;
    let requested_offset_off = current_pos_off + 8;
    let outer_off = requested_offset_off + 8;
    let columns_off = outer_off + 8;
    let name_count_off = columns_off + 8;
    let group_idx_off = name_count_off + 8;
    let max_group_off = group_idx_off + 8;
    let row_off = max_group_off + 8;
    let row_tag_off = row_off + 8;
    let box_off = row_tag_off + 8;
    let match_end_off = box_off + 8;
    let name_ptr_off = match_end_off + 8;
    let name_len_off = name_ptr_off + 8;
    let key_lo_off = name_len_off + 8;
    let key_hi_off = key_lo_off + 8;
    let stack_size = (key_hi_off + 8 + 48 + 15) & !15;
    let save_off = stack_size - 16;

    emitter.blank();
    emitter.comment("--- runtime: preg_match_all capture ---");
    emitter.label_global("__rt_preg_match_all_capture");
    emitter.instruction(&format!("sub sp, sp, #{}", stack_size));               // reserve regex state plus stable nested-array owners
    emitter.instruction(&format!("add x9, sp, #{}", save_off));                 // compute the save slot beyond the pair-store immediate range
    emitter.instruction("stp x29, x30, [x9]");                                  // preserve the generated caller frame
    emitter.instruction(&format!("add x29, sp, #{}", save_off));                // establish a stable helper frame
    emitter.instruction(&format!("str x1, [sp, #{}]", pattern_ptr_off));        // retain pattern bytes across compile and match calls
    emitter.instruction(&format!("str x2, [sp, #{}]", pattern_len_off));        // retain pattern length across compile and match calls
    emitter.instruction(&format!("str x3, [sp, #{}]", subject_ptr_off));        // retain subject bytes across compile and match calls
    emitter.instruction(&format!("str x4, [sp, #{}]", subject_len_off));        // retain subject length across compile and match calls
    // `__rt_preg_strip` reuses x5 for its own output, so PHP's `$flags` is spilled first.
    emitter.instruction(&format!("str x5, [sp, #{}]", php_flags_off));          // preserve PHP's order and offset-capture flags
    emitter.instruction(&format!("str x6, [sp, #{}]", requested_offset_off));   // preserve the public byte offset for the first search
    emitter.instruction(&format!("str xzr, [sp, #{}]", handle_off));            // an uncompiled pattern owns no handle to release
    emitter.instruction(&format!("str xzr, [sp, #{}]", pairs_ptr_off));         // no offset-pair vector has been allocated yet
    emitter.instruction(&format!("str xzr, [sp, #{}]", columns_off));           // no pattern-order column vector has been allocated yet
    emitter.instruction(&format!("str xzr, [sp, #{}]", outer_off));             // a null container means the epilogue builds PHP's empty one
    emitter.instruction(&format!("str xzr, [sp, #{}]", match_count_off));       // every failure path still returns the ordinary zero count

    // -- strip delimiters and compile --
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
    emitter.instruction(&format!("add x3, sp, #{}", nmatch_off));               // receive the pattern's real capture-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile the pattern through the managed PCRE bridge
    emitter.instruction("cbnz x0, __rt_preg_match_all_capture_cleanup");        // an invalid pattern leaves no handle and no matches

    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // the name table lives inside the compiled pattern
    emitter.bl_c("elephc_pcre2_v1_name_count");                                 // ask PCRE2 how many capture groups were named
    emitter.instruction(&format!("str x0, [sp, #{}]", name_count_off));         // a named group makes pattern order build hash storage

    // -- one 16-byte offset pair per compiled capture slot --
    emitter.instruction(&format!("ldr x9, [sp, #{}]", nmatch_off));             // load the compiled capture-slot count
    emitter.instruction("lsr x10, x9, #59");                                    // reject counts whose 16-byte size would overflow
    emitter.instruction("cbnz x10, __rt_preg_match_all_capture_cleanup");       // refuse the wrapped allocation instead of making it
    emitter.instruction("lsl x0, x9, #4");                                      // allocate one signed-64-bit pair per slot
    emitter.bl_c("malloc");                                                     // allocate the fixed offset-pair vector
    emitter.instruction("cbz x0, __rt_preg_match_all_capture_cleanup");         // allocation failure returns the empty capture matrix
    emitter.instruction(&format!("str x0, [sp, #{}]", pairs_ptr_off));          // save the offset-pair vector for every search

    // -- PHP's PREG_SET_ORDER is bit 2 of `$flags`; anything else is pattern order --
    emitter.instruction(&format!("ldr x9, [sp, #{}]", php_flags_off));          // reload PHP's order flags
    emitter.instruction("tst x9, #2");                                          // is PREG_SET_ORDER requested?
    emitter.instruction("b.eq __rt_preg_match_all_capture_columns_init");       // pattern order pre-creates one column per group
    emitter.instruction("mov x0, #4");                                          // rows accumulate in a modest first capacity
    emitter.instruction("mov x1, #8");                                          // outer slots hold boxed Mixed rows
    emitter.instruction("bl __rt_array_new");                                   // allocate the set-order row list
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_off));              // set order fills its container as it matches
    emitter.instruction("b __rt_preg_match_all_capture_subject");               // continue with subject preparation

    // -- pattern order: every compiled group owns a column, even one that never participates --
    emitter.label("__rt_preg_match_all_capture_columns_init");
    emitter.instruction(&format!("ldr x9, [sp, #{}]", nmatch_off));             // one column pointer per compiled capture slot
    emitter.instruction("lsl x0, x9, #3");                                      // eight bytes per retained column pointer
    emitter.bl_c("malloc");                                                     // allocate the column-pointer vector
    emitter.instruction("cbz x0, __rt_preg_match_all_capture_cleanup");         // allocation failure returns the empty capture matrix
    emitter.instruction(&format!("str x0, [sp, #{}]", columns_off));            // save the column vector for the append loop
    emitter.instruction(&format!("str xzr, [sp, #{}]", group_idx_off));         // start with capture index zero
    emitter.label("__rt_preg_match_all_capture_column_new");
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index being created
    emitter.instruction(&format!("ldr x13, [sp, #{}]", nmatch_off));            // reload the compiled capture-slot count
    emitter.instruction("cmp x12, x13");                                        // has every column been allocated?
    emitter.instruction("b.ge __rt_preg_match_all_capture_subject");            // continue with subject preparation
    emitter.instruction("mov x0, #4");                                          // reserve a modest first capacity per column
    emitter.instruction("mov x1, #16");                                         // columns hold string pointer/length pairs
    emitter.instruction("bl __rt_array_new");                                   // allocate this group's column
    emitter.instruction(&format!("ldr x9, [sp, #{}]", columns_off));            // reload the column vector base
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index after the helper call
    emitter.instruction("str x0, [x9, x12, lsl #3]");                           // retain the new column at its group index
    emitter.instruction("add x12, x12, #1");                                    // advance to the next column
    emitter.instruction(&format!("str x12, [sp, #{}]", group_idx_off));         // save the advanced column index
    emitter.instruction("b __rt_preg_match_all_capture_column_new");            // continue allocating columns

    // -- one null-terminated subject copy drives every search --
    emitter.label("__rt_preg_match_all_capture_subject");
    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_ptr_off));        // reload subject bytes for C-string materialization
    emitter.instruction(&format!("ldr x2, [sp, #{}]", subject_len_off));        // reload subject length for C-string materialization
    emitter.instruction("bl __rt_cstr2");                                       // make one null-terminated subject copy for repeated searches
    emitter.instruction(&format!("str x0, [sp, #{}]", subject_cstr_off));       // preserve the subject base across PCRE calls
    emitter.instruction(&format!("ldr x9, [sp, #{}]", requested_offset_off));   // load the public offset before normalizing it
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

    emitter.label("__rt_preg_match_all_capture_loop");
    emitter.instruction(&format!("ldr x1, [sp, #{}]", current_pos_off));        // load the current subject suffix cursor
    // The END position is a legal search start: PHP reports a zero-length match there.
    emitter.instruction(&format!("ldr x9, [sp, #{}]", subject_cstr_off));       // load the subject base for the bound
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_len_off));       // load the subject length for the bound
    emitter.instruction("add x9, x9, x10");                                     // the last legal search start is one past the last byte
    emitter.instruction("cmp x1, x9");                                          // has the cursor moved beyond it?
    emitter.instruction("b.hi __rt_preg_match_all_capture_done");               // stop only once the cursor is past the end
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // pass the compiled pattern handle
    emitter.instruction(&format!("ldr x2, [sp, #{}]", nmatch_off));             // request every compiled capture, not just the full match
    emitter.instruction(&format!("ldr x3, [sp, #{}]", pairs_ptr_off));          // receive the pairs in the dedicated vector
    emitter.instruction("mov x4, #0");                                          // search the current suffix with ordinary PCRE flags
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // find the next non-overlapping match
    emitter.instruction("cbnz x0, __rt_preg_match_all_capture_done");           // a non-zero status ends the global search
    emitter.instruction(&format!("ldr x9, [sp, #{}]", pairs_ptr_off));          // the full match occupies the first pair
    emitter.instruction("ldr x10, [x9, #8]");                                   // load the match end offset within the current suffix
    emitter.instruction(&format!("str x10, [sp, #{}]", match_end_off));         // the cursor advances past it after the row is built
    emitter.instruction(&format!("ldr x9, [sp, #{}]", php_flags_off));          // reload PHP's order flags
    emitter.instruction("tst x9, #2");                                          // is PREG_SET_ORDER requested?
    emitter.instruction("b.ne __rt_preg_match_all_capture_row");                // set order appends one trimmed row per match

    // -- pattern order: append this match's capture to every column --
    emitter.instruction(&format!("str xzr, [sp, #{}]", group_idx_off));         // start with capture index zero
    emitter.label("__rt_preg_match_all_capture_column_loop");
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload current capture index
    emitter.instruction(&format!("ldr x13, [sp, #{}]", nmatch_off));            // reload the compiled capture-slot count
    emitter.instruction("cmp x12, x13");                                        // has every column received this match?
    emitter.instruction("b.ge __rt_preg_match_all_capture_advance");            // move on to the next match
    emitter.instruction("lsl x14, x12, #4");                                    // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("ldr x15, [sp, #{}]", pairs_ptr_off));         // load the offset-pair vector base
    emitter.instruction("add x14, x15, x14");                                   // compute address of this offset pair
    emitter.instruction("ldr x15, [x14]");                                      // load signed-64-bit capture start
    emitter.instruction("ldr x16, [x14, #8]");                                  // load signed-64-bit capture end
    emitter.instruction("cmp x15, #0");                                         // detect captures that did not participate
    emitter.instruction("b.lt __rt_preg_match_all_capture_column_empty");       // PHP still emits an empty string in that column
    emitter.instruction("sub x2, x16, x15");                                    // capture length = rm_eo - rm_so
    emitter.instruction(&format!("ldr x1, [sp, #{}]", current_pos_off));        // the pair offsets index into the current suffix
    emitter.instruction("add x1, x1, x15");                                     // compute capture string pointer
    emitter.instruction("b __rt_preg_match_all_capture_column_push");           // append this capture string
    emitter.label("__rt_preg_match_all_capture_column_empty");
    emitter.instruction("mov x1, #0");                                          // an unmatched capture has a null pointer
    emitter.instruction("mov x2, #0");                                          // an unmatched capture has zero length
    emitter.label("__rt_preg_match_all_capture_column_push");
    emitter.instruction(&format!("ldr x9, [sp, #{}]", columns_off));            // reload the column vector base
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index
    emitter.instruction("ldr x0, [x9, x12, lsl #3]");                           // load this group's column receiver
    emitter.instruction("bl __rt_array_push_str");                              // persist and append the capture string
    emitter.instruction(&format!("ldr x9, [sp, #{}]", columns_off));            // reload the column vector base after the helper call
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index after the helper call
    emitter.instruction("str x0, [x9, x12, lsl #3]");                           // publish a possibly-relocated column
    emitter.instruction("add x12, x12, #1");                                    // advance to the next column
    emitter.instruction(&format!("str x12, [sp, #{}]", group_idx_off));         // save the advanced column index
    emitter.instruction("b __rt_preg_match_all_capture_column_loop");           // continue filling columns

    // -- set order: one row per match, trimmed after its highest participating group --
    emitter.label("__rt_preg_match_all_capture_row");
    emitter.instruction(&format!("ldr x12, [sp, #{}]", nmatch_off));            // start the scan at the last compiled capture
    emitter.instruction("sub x12, x12, #1");                                    // the slot count is exclusive
    emitter.label("__rt_preg_match_all_capture_scan");
    emitter.instruction("lsl x14, x12, #4");                                    // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("ldr x15, [sp, #{}]", pairs_ptr_off));         // load the offset-pair vector base
    emitter.instruction("add x14, x15, x14");                                   // compute address of this offset pair
    emitter.instruction("ldr x13, [x14]");                                      // load signed-64-bit capture start
    emitter.instruction("cmp x13, #0");                                         // did this capture participate in this match?
    emitter.instruction("b.ge __rt_preg_match_all_capture_scan_found");         // trim the row after this capture
    emitter.instruction("cbz x12, __rt_preg_match_all_capture_scan_found");     // the full match slot always survives
    emitter.instruction("sub x12, x12, #1");                                    // keep looking at the previous capture slot
    emitter.instruction("b __rt_preg_match_all_capture_scan");                  // continue the trailing-group scan
    emitter.label("__rt_preg_match_all_capture_scan_found");
    emitter.instruction(&format!("str x12, [sp, #{}]", max_group_off));         // save the highest capture index this row emits
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // the row builder resolves declared group names
    emitter.instruction(&format!("ldr x1, [sp, #{}]", pairs_ptr_off));          // pass the populated offset-pair vector
    emitter.instruction(&format!("ldr x2, [sp, #{}]", max_group_off));          // pass the highest capture index to materialize
    emitter.instruction(&format!("ldr x3, [sp, #{}]", current_pos_off));        // the pair offsets index into the current suffix
    emitter.instruction("bl __rt_preg_match_row");                              // build the same row `preg_match()` would
    emitter.instruction(&format!("str x0, [sp, #{}]", row_off));                // hold the row across boxing
    emitter.instruction(&format!("str x1, [sp, #{}]", row_tag_off));            // hold the storage tag the row builder chose
    emitter.instruction(&format!("ldr x0, [sp, #{}]", row_tag_off));            // a row is boxed under its own storage tag
    emitter.instruction(&format!("ldr x1, [sp, #{}]", row_off));                // pass the row as the Mixed payload
    emitter.instruction("mov x2, xzr");                                         // arrays use one payload word
    emitter.instruction("bl __rt_mixed_from_value");                            // create the outer-owned boxed row
    emitter.instruction(&format!("str x0, [sp, #{}]", box_off));                // save the box before consuming it into the row list
    // Boxing RETAINS the payload, so this frame still owns the construction reference the row
    // builder returned. Keeping it left every row and its strings alive for the life of the
    // process — eight thousand capture calls leaked six megabytes and exhausted the heap.
    emitter.instruction(&format!("ldr x0, [sp, #{}]", row_off));                // drop the construction reference the box now holds
    emitter.instruction("bl __rt_decref_any");                                  // the box is the row's only owner from here
    emitter.instruction(&format!("ldr x0, [sp, #{}]", outer_off));              // reload the row list receiver
    emitter.instruction(&format!("ldr x1, [sp, #{}]", match_count_off));        // rows are appended at the running match index
    emitter.instruction(&format!("ldr x2, [sp, #{}]", box_off));                // transfer the boxed row into the list
    emitter.instruction("bl __rt_array_set_mixed");                             // publish the row and retain COW invariants
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_off));              // preserve a possible row-list relocation

    emitter.label("__rt_preg_match_all_capture_advance");
    emitter.instruction(&format!("ldr x9, [sp, #{}]", match_count_off));        // reload the completed match count
    emitter.instruction("add x9, x9, #1");                                      // count the match just materialized
    emitter.instruction(&format!("str x9, [sp, #{}]", match_count_off));        // save the updated count across the next PCRE call
    emitter.instruction(&format!("ldr x10, [sp, #{}]", match_end_off));         // reload the match end recorded before the row was built
    emitter.instruction("cmp x10, #0");                                         // zero-length matches must still make progress
    emitter.instruction("b.gt __rt_preg_match_all_capture_step");               // a positive end offset is the normal next suffix boundary
    emitter.instruction("mov x10, #1");                                         // force one-byte progress for a zero-length match
    emitter.label("__rt_preg_match_all_capture_step");
    emitter.instruction(&format!("ldr x9, [sp, #{}]", current_pos_off));        // reload the current suffix pointer
    emitter.instruction("add x9, x9, x10");                                     // move past the complete current match
    emitter.instruction(&format!("str x9, [sp, #{}]", current_pos_off));        // persist the next PCRE suffix cursor
    emitter.instruction("b __rt_preg_match_all_capture_loop");                  // continue until PCRE finds no next match

    // -- publish pattern-order columns; set order already holds its finished list --
    emitter.label("__rt_preg_match_all_capture_done");
    emitter.instruction(&format!("ldr x9, [sp, #{}]", php_flags_off));          // reload PHP's order flags
    emitter.instruction("tst x9, #2");                                          // is PREG_SET_ORDER requested?
    emitter.instruction("b.ne __rt_preg_match_all_capture_cleanup");            // set order needs no column publication
    emitter.instruction(&format!("ldr x9, [sp, #{}]", name_count_off));         // reload the declared group-name count
    emitter.instruction("cbnz x9, __rt_preg_match_all_capture_named_columns");  // a named group forces hash storage
    emitter.instruction(&format!("ldr x0, [sp, #{}]", nmatch_off));             // one outer slot per compiled capture column
    emitter.instruction("mov x1, #8");                                          // outer slots hold boxed Mixed columns
    emitter.instruction("bl __rt_array_new");                                   // allocate the pattern-order matrix
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_off));              // save the matrix across column publication
    emitter.instruction(&format!("str xzr, [sp, #{}]", group_idx_off));         // start with capture index zero
    emitter.label("__rt_preg_match_all_capture_publish");
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index being published
    emitter.instruction(&format!("ldr x13, [sp, #{}]", nmatch_off));            // reload the compiled capture-slot count
    emitter.instruction("cmp x12, x13");                                        // has every column been published?
    emitter.instruction("b.ge __rt_preg_match_all_capture_cleanup");            // release the scratch vectors and return
    emitter.instruction(&format!("ldr x9, [sp, #{}]", columns_off));            // reload the column vector base
    emitter.instruction("ldr x1, [x9, x12, lsl #3]");                           // pass this column as the Mixed payload
    emitter.instruction("mov x2, xzr");                                         // arrays use one payload word
    emitter.instruction("mov x0, #4");                                          // runtime tag 4 identifies indexed arrays
    emitter.instruction("bl __rt_mixed_from_value");                            // create the matrix-owned boxed column
    emitter.instruction(&format!("str x0, [sp, #{}]", box_off));                // save the box before consuming it into the matrix
    // Boxing RETAINS the column, so the construction reference held by the column vector has to
    // go; see the set-order row path for the leak this caused.
    emitter.instruction(&format!("ldr x9, [sp, #{}]", columns_off));            // reload the column vector base
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index
    emitter.instruction("ldr x0, [x9, x12, lsl #3]");                           // drop the construction reference the box now holds
    emitter.instruction("bl __rt_decref_any");                                  // the box is the column's only owner from here
    emitter.instruction(&format!("ldr x0, [sp, #{}]", outer_off));              // reload the matrix receiver
    emitter.instruction(&format!("ldr x1, [sp, #{}]", group_idx_off));          // the capture index is the numeric key
    emitter.instruction(&format!("ldr x2, [sp, #{}]", box_off));                // transfer the boxed column into the matrix
    emitter.instruction("bl __rt_array_set_mixed");                             // publish the column and retain COW invariants
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_off));              // preserve a possible matrix relocation
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index after helper calls
    emitter.instruction("add x12, x12, #1");                                    // advance to the next column
    emitter.instruction(&format!("str x12, [sp, #{}]", group_idx_off));         // save the advanced column index
    emitter.instruction("b __rt_preg_match_all_capture_publish");               // continue publishing columns

    // -- named pattern order: each declared name precedes the numeric key it doubles --
    emitter.label("__rt_preg_match_all_capture_named_columns");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", nmatch_off));             // size the table from the compiled capture count
    emitter.instruction("lsl x0, x0, #1");                                      // reserve one bucket per numeric key and one per name
    emitter.instruction("mov x1, #7");                                          // mixed-valued hash: every entry holds a column array
    emitter.instruction("bl __rt_hash_new");                                    // allocate the ordered hash carrying both key kinds
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_off));              // save the matrix hash across inserts
    emitter.instruction(&format!("str xzr, [sp, #{}]", group_idx_off));         // start with capture index zero
    emitter.label("__rt_preg_match_all_capture_named_publish");
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index being published
    emitter.instruction(&format!("ldr x13, [sp, #{}]", nmatch_off));            // reload the compiled capture-slot count
    emitter.instruction("cmp x12, x13");                                        // has every column been published?
    emitter.instruction("b.ge __rt_preg_match_all_capture_cleanup");            // release the scratch vectors and return
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // pass the compiled pattern holding the name table
    emitter.instruction(&format!("ldr x1, [sp, #{}]", group_idx_off));          // ask for this capture group's declared name
    emitter.instruction(&format!("add x2, sp, #{}", name_ptr_off));             // receive the name pointer into pattern storage
    emitter.instruction(&format!("add x3, sp, #{}", name_len_off));             // receive the name length
    emitter.bl_c("elephc_pcre2_v1_group_name");                                 // resolve the name without exposing PCRE2 table layouts
    emitter.instruction("cbnz w0, __rt_preg_match_all_capture_named_int");      // an unnamed group only gets its numeric key
    // The name and its numeric twin are two hash entries over ONE column array, so the column
    // is retained once more before the first insert consumes a reference.
    emitter.instruction(&format!("ldr x9, [sp, #{}]", columns_off));            // reload the column vector base
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index
    emitter.instruction("ldr x0, [x9, x12, lsl #3]");                           // load the column both keys will share
    emitter.instruction("bl __rt_incref");                                      // the second entry owns its own reference
    emitter.instruction(&format!("ldr x1, [sp, #{}]", name_ptr_off));           // load the resolved group name pointer
    emitter.instruction(&format!("ldr x2, [sp, #{}]", name_len_off));           // load the resolved group name length
    emitter.instruction("bl __rt_hash_normalize_key");                          // apply PHP's numeric-string key normalization
    emitter.instruction(&format!("str x1, [sp, #{}]", key_lo_off));             // hold the normalized key across the insert
    emitter.instruction(&format!("str x2, [sp, #{}]", key_hi_off));             // hold the key discriminant across the insert
    emitter.instruction(&format!("ldr x9, [sp, #{}]", columns_off));            // reload the column vector base
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index
    emitter.instruction("ldr x3, [x9, x12, lsl #3]");                           // pass the column as the entry value payload
    emitter.instruction("mov x4, xzr");                                         // an array payload has no high word
    emitter.instruction(&format!("ldr x1, [sp, #{}]", key_lo_off));             // reload the normalized key
    emitter.instruction(&format!("ldr x2, [sp, #{}]", key_hi_off));             // reload the key discriminant
    emitter.instruction("mov x5, #4");                                          // runtime value tag 4 = indexed array
    emitter.instruction(&format!("ldr x0, [sp, #{}]", outer_off));              // reload the matrix hash pointer
    emitter.instruction("bl __rt_hash_set");                                    // insert the named key ahead of its numeric twin
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_off));              // save a possibly-grown matrix hash pointer
    emitter.label("__rt_preg_match_all_capture_named_int");
    emitter.instruction(&format!("ldr x9, [sp, #{}]", columns_off));            // reload the column vector base
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index
    emitter.instruction("ldr x3, [x9, x12, lsl #3]");                           // pass the column as the entry value payload
    emitter.instruction("mov x4, xzr");                                         // an array payload has no high word
    emitter.instruction(&format!("ldr x1, [sp, #{}]", group_idx_off));          // the capture index is the numeric key
    emitter.instruction("mov x2, #-1");                                         // a key discriminant of -1 marks an integer key
    emitter.instruction("mov x5, #4");                                          // runtime value tag 4 = indexed array
    emitter.instruction(&format!("ldr x0, [sp, #{}]", outer_off));              // reload the matrix hash pointer
    emitter.instruction("bl __rt_hash_set");                                    // insert this column under its numeric key
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_off));              // save a possibly-grown matrix hash pointer
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload the column index after helper calls
    emitter.instruction("add x12, x12, #1");                                    // advance to the next column
    emitter.instruction(&format!("str x12, [sp, #{}]", group_idx_off));         // save the advanced column index
    emitter.instruction("b __rt_preg_match_all_capture_named_publish");         // continue publishing columns

    // -- one exit releases the scratch vectors, the handle, and materializes PHP's empty case --
    emitter.label("__rt_preg_match_all_capture_cleanup");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", columns_off));            // the column ARRAYS are owned by the matrix now
    emitter.instruction("cbz x0, __rt_preg_match_all_capture_free_pairs");      // pattern order may never have allocated the vector
    emitter.bl_c("free");                                                       // release only the pointer vector itself
    emitter.label("__rt_preg_match_all_capture_free_pairs");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", pairs_ptr_off));          // release the offset-pair vector
    emitter.instruction("cbz x0, __rt_preg_match_all_capture_free_handle");     // a failed compile never allocated it
    emitter.bl_c("free");                                                       // release the fixed pair storage
    emitter.label("__rt_preg_match_all_capture_free_handle");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // release the compiled PCRE resource
    emitter.instruction("cbz x0, __rt_preg_match_all_capture_empty");           // an invalid pattern produced no handle
    emitter.bl_c("elephc_pcre2_v1_free");                                       // dispose of the opaque compiled pattern
    emitter.label("__rt_preg_match_all_capture_empty");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", outer_off));              // did any path build a capture container?
    emitter.instruction("cbnz x0, __rt_preg_match_all_capture_return");         // return the container that was built
    emitter.instruction("mov x0, #0");                                          // PHP leaves `$matches` an empty array here
    emitter.instruction("mov x1, #8");                                          // outer slots hold boxed Mixed cells
    emitter.instruction("bl __rt_array_new");                                   // allocate the empty capture container
    emitter.instruction(&format!("str x0, [sp, #{}]", outer_off));              // return it beside the zero match count

    emitter.label("__rt_preg_match_all_capture_return");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", match_count_off));        // return the integer match count
    emitter.instruction(&format!("ldr x1, [sp, #{}]", outer_off));              // return the capture container beside the count
    emitter.instruction(&format!("add x9, sp, #{}", save_off));                 // recompute the save slot for the pair load
    emitter.instruction("ldp x29, x30, [x9]");                                  // restore the generated caller frame
    emitter.instruction(&format!("add sp, sp, #{}", stack_size));               // release all helper scratch space
    emitter.instruction("ret");                                                 // return count and capture container to lowering
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
    // The END position is a legal search start, not a stop condition: PHP reports a zero-length
    // match there, so `preg_match_all('/x*/', 'axb')` counts four, not three.
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", subject_cstr_off)); // load the subject base for the bound
    emitter.instruction(&format!("add r9, QWORD PTR [rsp + {}]", subject_len_off)); // the last legal search start is one past the last byte
    emitter.instruction("cmp rsi, r9");                                         // has the cursor moved beyond it?
    emitter.instruction("ja __rt_preg_match_all_done_linux_x86_64");            // stop only once the cursor is past the end
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
/// Emits the capture-producing `preg_match_all()` path for Linux x86_64.
///
/// See the AArch64 sibling for the shape contract; this is the same algorithm under the System V
/// ABI. PHP's `$flags` arrives in r8 and is spilled in the prologue: `__rt_preg_strip` reuses the
/// argument registers, and the previous revision never saved it at all, which is why
/// `PREG_SET_ORDER` could not be honoured here even in principle.
///
/// Input:  rdi=pattern ptr, rsi=pattern len, rdx=subject ptr, rcx=subject len,
///         r8=PHP `$flags`, r9=starting byte offset
/// Output: rax=match count, rdx=`$matches` storage owned by the caller
fn emit_preg_match_all_capture_linux_x86_64(emitter: &mut Emitter) {
    let handle = 0;
    let pairs_ptr = handle + 8;
    let nmatch = pairs_ptr + 8;
    let pattern = nmatch + 8;
    let pattern_len = pattern + 8;
    let subject = pattern_len + 8;
    let subject_len = subject + 8;
    let php_flags = subject_len + 8;
    let flags = php_flags + 8;
    let subject_cstr = flags + 8;
    let count = subject_cstr + 8;
    let cursor = count + 8;
    let requested_offset = cursor + 8;
    let outer = requested_offset + 8;
    let columns = outer + 8;
    let name_count = columns + 8;
    let group_idx = name_count + 8;
    let max_group = group_idx + 8;
    let row = max_group + 8;
    let row_tag = row + 8;
    let boxed = row_tag + 8;
    let match_end = boxed + 8;
    let name_ptr = match_end + 8;
    let name_len = name_ptr + 8;
    let key_lo = name_len + 8;
    let key_hi = key_lo + 8;
    let stack_size = (key_hi + 8 + 32 + 15) & !15;

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
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r8", php_flags));   // PHP's order flags, before the strip helper reuses r8
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", requested_offset)); // retain the public starting offset
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", handle));       // an uncompiled pattern owns no handle to release
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", pairs_ptr));    // no offset-pair vector has been allocated yet
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", columns));      // no pattern-order column vector has been allocated yet
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", outer));        // a null container means the epilogue builds PHP's empty one
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", count));        // every failure path still returns the ordinary zero count

    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", pattern));    // restore delimiter-strip input pattern
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", pattern_len)); // restore delimiter-strip input length
    emitter.instruction("call __rt_preg_strip");                                // normalize delimiters and extract compile flags
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", flags));      // preserve the derived compile flags
    emitter.instruction("call __rt_pcre_to_posix");                             // materialize a C-compatible pattern string
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", pattern));    // reuse the pattern slot for its C-string pointer
    super::emit_prepare_regex_locale(emitter);
    emitter.instruction(&format!("lea rdi, [rsp + {}]", handle));               // provide compiled-pattern output storage
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", pattern));    // pass the C-compatible pattern
    emitter.instruction(&format!("mov edx, DWORD PTR [rsp + {}]", flags));      // pass delimiter-derived flags
    emitter.instruction(&format!("lea rcx, [rsp + {}]", nmatch));               // receive the pattern's real capture-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile via the managed PCRE bridge
    emitter.instruction("test eax, eax");                                       // did compilation succeed?
    emitter.instruction("jnz __rt_preg_match_all_capture_cleanup_linux_x86_64"); // an invalid pattern leaves no handle and no matches

    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle));     // the name table lives inside the compiled pattern
    emitter.bl_c("elephc_pcre2_v1_name_count");                                 // ask PCRE2 how many capture groups were named
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", name_count)); // a named group makes pattern order build hash storage

    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", nmatch));     // load the compiled capture-slot count
    emitter.instruction("mov rcx, rax");                                        // copy it before the overflow probe
    emitter.instruction("shr rcx, 59");                                         // reject counts whose 16-byte size would overflow
    emitter.instruction("jnz __rt_preg_match_all_capture_cleanup_linux_x86_64"); // refuse the wrapped allocation instead of making it
    emitter.instruction("shl rax, 4");                                          // allocate one signed-64-bit pair per slot
    emitter.instruction("mov rdi, rax");                                        // pass the byte size to the allocator
    emitter.bl_c("malloc");                                                     // allocate the fixed offset-pair vector
    emitter.instruction("test rax, rax");                                       // did the allocation succeed?
    emitter.instruction("jz __rt_preg_match_all_capture_cleanup_linux_x86_64"); // allocation failure returns the empty capture matrix
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", pairs_ptr));  // save the offset-pair vector for every search

    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", php_flags));  // reload PHP's order flags
    emitter.instruction("test rax, 2");                                         // PHP's PREG_SET_ORDER is bit 2 of `$flags`
    emitter.instruction("jz __rt_preg_match_all_capture_columns_init_linux_x86_64"); // pattern order pre-creates one column per group
    emitter.instruction("mov edi, 4");                                          // rows accumulate in a modest first capacity
    emitter.instruction("mov esi, 8");                                          // outer slots hold boxed Mixed rows
    emitter.instruction("call __rt_array_new");                                 // allocate the set-order row list
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // set order fills its container as it matches
    emitter.instruction("jmp __rt_preg_match_all_capture_subject_linux_x86_64"); // continue with subject preparation

    emitter.label("__rt_preg_match_all_capture_columns_init_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", nmatch));     // one column pointer per compiled capture slot
    emitter.instruction("shl rdi, 3");                                          // eight bytes per retained column pointer
    emitter.bl_c("malloc");                                                     // allocate the column-pointer vector
    emitter.instruction("test rax, rax");                                       // did the allocation succeed?
    emitter.instruction("jz __rt_preg_match_all_capture_cleanup_linux_x86_64"); // allocation failure returns the empty capture matrix
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", columns));    // save the column vector for the append loop
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", group_idx));    // start with capture index zero
    emitter.label("__rt_preg_match_all_capture_column_new_linux_x86_64");
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index being created
    emitter.instruction(&format!("cmp r9, QWORD PTR [rsp + {}]", nmatch));      // has every column been allocated?
    emitter.instruction("jge __rt_preg_match_all_capture_subject_linux_x86_64"); // continue with subject preparation
    emitter.instruction("mov edi, 4");                                          // reserve a modest first capacity per column
    emitter.instruction("mov esi, 16");                                         // columns hold string pointer/length pairs
    emitter.instruction("call __rt_array_new");                                 // allocate this group's column
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", columns));    // reload the column vector base
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index after the helper call
    emitter.instruction("mov QWORD PTR [r10 + r9 * 8], rax");                   // retain the new column at its group index
    emitter.instruction("add r9, 1");                                           // advance to the next column
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", group_idx));   // save the advanced column index
    emitter.instruction("jmp __rt_preg_match_all_capture_column_new_linux_x86_64"); // continue allocating columns

    emitter.label("__rt_preg_match_all_capture_subject_linux_x86_64");
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
    emitter.instruction("js __rt_preg_match_all_capture_done_linux_x86_64");    // invalid offsets yield no match
    emitter.instruction(&format!("cmp r9, QWORD PTR [rsp + {}]", subject_len)); // the trailing boundary remains valid
    emitter.instruction("jg __rt_preg_match_all_capture_done_linux_x86_64");    // reject offsets beyond the subject
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", subject_cstr)); // load the subject C-string base
    emitter.instruction("add rax, r9");                                         // form the first search suffix
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", cursor));     // preserve the PCRE search cursor

    emitter.label("__rt_preg_match_all_capture_loop_linux_x86_64");
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", cursor));     // load the current subject suffix
    // The END position is a legal search start: PHP reports a zero-length match there.
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", subject_cstr)); // load the subject base for the bound
    emitter.instruction(&format!("add r9, QWORD PTR [rsp + {}]", subject_len)); // the last legal search start is one past the last byte
    emitter.instruction("cmp rsi, r9");                                         // has the cursor moved beyond it?
    emitter.instruction("ja __rt_preg_match_all_capture_done_linux_x86_64");    // stop only once the cursor is past the end
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle));     // pass the compiled pattern handle
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", nmatch));     // request every compiled capture, not just the full match
    emitter.instruction(&format!("mov rcx, QWORD PTR [rsp + {}]", pairs_ptr));  // receive the pairs in the dedicated vector
    emitter.instruction("xor r8d, r8d");                                        // use ordinary PCRE execution flags
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // search for the next non-overlapping match
    emitter.instruction("test eax, eax");                                       // did PCRE find a match?
    emitter.instruction("jnz __rt_preg_match_all_capture_done_linux_x86_64");   // a non-zero result ends the global search
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", pairs_ptr));  // the full match occupies the first pair
    emitter.instruction("mov rcx, QWORD PTR [r10 + 8]");                        // load the match end within the current suffix
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", match_end));  // the cursor advances past it after the row is built
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", php_flags));  // reload PHP's order flags
    emitter.instruction("test rax, 2");                                         // is PREG_SET_ORDER requested?
    emitter.instruction("jnz __rt_preg_match_all_capture_row_linux_x86_64");    // set order appends one trimmed row per match

    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", group_idx));    // start with capture index zero
    emitter.label("__rt_preg_match_all_capture_column_loop_linux_x86_64");
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload current capture index
    emitter.instruction(&format!("cmp r9, QWORD PTR [rsp + {}]", nmatch));      // has every column received this match?
    emitter.instruction("jge __rt_preg_match_all_capture_advance_linux_x86_64"); // move on to the next match
    emitter.instruction("mov r10, r9");                                         // copy capture index before scaling
    emitter.instruction("shl r10, 4");                                          // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("add r10, QWORD PTR [rsp + {}]", pairs_ptr));  // compute address of this offset pair
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // load signed-64-bit capture start
    emitter.instruction("mov rcx, QWORD PTR [r10 + 8]");                        // load signed-64-bit capture end
    emitter.instruction("cmp r11, 0");                                          // detect captures that did not participate
    emitter.instruction("jl __rt_preg_match_all_capture_column_empty_linux_x86_64"); // PHP still emits an empty string in that column
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", cursor));     // the pair offsets index into the current suffix
    emitter.instruction("add rsi, r11");                                        // compute capture string pointer
    emitter.instruction("mov rdx, rcx");                                        // copy capture end offset before subtracting start
    emitter.instruction("sub rdx, r11");                                        // capture length = rm_eo - rm_so
    emitter.instruction("jmp __rt_preg_match_all_capture_column_push_linux_x86_64"); // append this capture string
    emitter.label("__rt_preg_match_all_capture_column_empty_linux_x86_64");
    emitter.instruction("xor esi, esi");                                        // an unmatched capture has a null pointer
    emitter.instruction("xor edx, edx");                                        // an unmatched capture has zero length
    emitter.label("__rt_preg_match_all_capture_column_push_linux_x86_64");
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", columns));    // reload the column vector base
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index
    emitter.instruction("mov rdi, QWORD PTR [r10 + r9 * 8]");                   // load this group's column receiver
    emitter.instruction("call __rt_array_push_str");                            // persist and append the capture string
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", columns));    // reload the column vector base after the helper call
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index after the helper call
    emitter.instruction("mov QWORD PTR [r10 + r9 * 8], rax");                   // publish a possibly-relocated column
    emitter.instruction("add r9, 1");                                           // advance to the next column
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", group_idx));   // save the advanced column index
    emitter.instruction("jmp __rt_preg_match_all_capture_column_loop_linux_x86_64"); // continue filling columns

    emitter.label("__rt_preg_match_all_capture_row_linux_x86_64");
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", nmatch));      // start the scan at the last compiled capture
    emitter.instruction("sub r9, 1");                                           // the slot count is exclusive
    emitter.label("__rt_preg_match_all_capture_scan_linux_x86_64");
    emitter.instruction("mov r10, r9");                                         // copy capture index before scaling
    emitter.instruction("shl r10, 4");                                          // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("add r10, QWORD PTR [rsp + {}]", pairs_ptr));  // compute address of this offset pair
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // load signed-64-bit capture start
    emitter.instruction("cmp r11, 0");                                          // did this capture participate in this match?
    emitter.instruction("jge __rt_preg_match_all_capture_scan_found_linux_x86_64"); // trim the row after this capture
    emitter.instruction("test r9, r9");                                         // the full match slot always survives
    emitter.instruction("jz __rt_preg_match_all_capture_scan_found_linux_x86_64"); // stop at capture group zero
    emitter.instruction("sub r9, 1");                                           // keep looking at the previous capture slot
    emitter.instruction("jmp __rt_preg_match_all_capture_scan_linux_x86_64");   // continue the trailing-group scan
    emitter.label("__rt_preg_match_all_capture_scan_found_linux_x86_64");
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", max_group));   // save the highest capture index this row emits
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle));     // the row builder resolves declared group names
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", pairs_ptr));  // pass the populated offset-pair vector
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", max_group));  // pass the highest capture index to materialize
    emitter.instruction(&format!("mov rcx, QWORD PTR [rsp + {}]", cursor));     // the pair offsets index into the current suffix
    emitter.instruction("call __rt_preg_match_row");                            // build the same row `preg_match()` would
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", row));        // hold the row across boxing
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdx", row_tag));    // hold the storage tag the row builder chose
    emitter.instruction("mov rdi, rax");                                        // pass the row as the Mixed payload
    emitter.instruction("xor esi, esi");                                        // arrays have a single payload word
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", row_tag));    // a row is boxed under its own storage tag
    emitter.instruction("call __rt_mixed_from_value");                          // create the outer-owned boxed row
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", boxed));      // keep the box before the consuming array write
    // Boxing RETAINS the payload; see the AArch64 sibling for the leak this frame's construction
    // reference caused when it was kept.
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", row));        // drop the construction reference the box now holds
    emitter.instruction("call __rt_decref_any");                                // the box is the row's only owner from here
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", outer));      // reload the row list receiver
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", count));      // rows are appended at the running match index
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", boxed));      // transfer the boxed row into the list
    emitter.instruction("call __rt_array_set_mixed");                           // publish the row and retain COW invariants
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // preserve a possible row-list relocation

    emitter.label("__rt_preg_match_all_capture_advance_linux_x86_64");
    emitter.instruction(&format!("add QWORD PTR [rsp + {}], 1", count));        // count the match just materialized
    emitter.instruction(&format!("mov rcx, QWORD PTR [rsp + {}]", match_end));  // reload the match end recorded before the row was built
    emitter.instruction("test rcx, rcx");                                       // zero-length matches need forced progress
    emitter.instruction("jg __rt_preg_match_all_capture_step_linux_x86_64");    // ordinary matches advance to their end offset
    emitter.instruction("mov ecx, 1");                                          // force one-byte progress for an empty match
    emitter.label("__rt_preg_match_all_capture_step_linux_x86_64");
    emitter.instruction(&format!("add QWORD PTR [rsp + {}], rcx", cursor));     // move the suffix cursor past the full match
    emitter.instruction("jmp __rt_preg_match_all_capture_loop_linux_x86_64");   // continue until PCRE reports no next match

    emitter.label("__rt_preg_match_all_capture_done_linux_x86_64");
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", php_flags));  // reload PHP's order flags
    emitter.instruction("test rax, 2");                                         // is PREG_SET_ORDER requested?
    emitter.instruction("jnz __rt_preg_match_all_capture_cleanup_linux_x86_64"); // set order needs no column publication
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", name_count)); // reload the declared group-name count
    emitter.instruction("test rax, rax");                                       // did the pattern declare any group name?
    emitter.instruction("jnz __rt_preg_match_all_capture_named_columns_linux_x86_64"); // a named group forces hash storage
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", nmatch));     // one outer slot per compiled capture column
    emitter.instruction("mov esi, 8");                                          // outer slots hold boxed Mixed columns
    emitter.instruction("call __rt_array_new");                                 // allocate the pattern-order matrix
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // save the matrix across column publication
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", group_idx));    // start with capture index zero
    emitter.label("__rt_preg_match_all_capture_publish_linux_x86_64");
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index being published
    emitter.instruction(&format!("cmp r9, QWORD PTR [rsp + {}]", nmatch));      // has every column been published?
    emitter.instruction("jge __rt_preg_match_all_capture_cleanup_linux_x86_64"); // release the scratch vectors and return
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", columns));    // reload the column vector base
    emitter.instruction("mov rdi, QWORD PTR [r10 + r9 * 8]");                   // pass this column as the Mixed payload
    emitter.instruction("xor esi, esi");                                        // arrays have a single payload word
    emitter.instruction("mov eax, 4");                                          // runtime tag 4 identifies indexed arrays
    emitter.instruction("call __rt_mixed_from_value");                          // create the matrix-owned boxed column
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", boxed));      // keep the box before the consuming array write
    // Boxing RETAINS the column; the column vector's construction reference has to go.
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", columns));    // reload the column vector base
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index
    emitter.instruction("mov rax, QWORD PTR [r10 + r9 * 8]");                   // drop the construction reference the box now holds
    emitter.instruction("call __rt_decref_any");                                // the box is the column's only owner from here
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", outer));      // reload the matrix receiver
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", group_idx));  // the capture index is the numeric key
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", boxed));      // transfer the boxed column into the matrix
    emitter.instruction("call __rt_array_set_mixed");                           // publish the column and retain COW invariants
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // preserve a possible matrix relocation
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index after helper calls
    emitter.instruction("add r9, 1");                                           // advance to the next column
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", group_idx));   // save the advanced column index
    emitter.instruction("jmp __rt_preg_match_all_capture_publish_linux_x86_64"); // continue publishing columns

    emitter.label("__rt_preg_match_all_capture_named_columns_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", nmatch));     // size the table from the compiled capture count
    emitter.instruction("shl rdi, 1");                                          // reserve one bucket per numeric key and one per name
    emitter.instruction("mov esi, 7");                                          // mixed-valued hash: every entry holds a column array
    emitter.instruction("call __rt_hash_new");                                  // allocate the ordered hash carrying both key kinds
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // save the matrix hash across inserts
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", group_idx));    // start with capture index zero
    emitter.label("__rt_preg_match_all_capture_named_publish_linux_x86_64");
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index being published
    emitter.instruction(&format!("cmp r9, QWORD PTR [rsp + {}]", nmatch));      // has every column been published?
    emitter.instruction("jge __rt_preg_match_all_capture_cleanup_linux_x86_64"); // release the scratch vectors and return
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle));     // pass the compiled pattern holding the name table
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", group_idx));  // ask for this capture group's declared name
    emitter.instruction(&format!("lea rdx, [rsp + {}]", name_ptr));             // receive the name pointer into pattern storage
    emitter.instruction(&format!("lea rcx, [rsp + {}]", name_len));             // receive the name length
    emitter.bl_c("elephc_pcre2_v1_group_name");                                 // resolve the name without exposing PCRE2 table layouts
    emitter.instruction("test eax, eax");                                       // did this capture group carry a declared name?
    emitter.instruction("jnz __rt_preg_match_all_capture_named_int_linux_x86_64"); // an unnamed group only gets its numeric key
    // The name and its numeric twin are two hash entries over ONE column array, so the column
    // is retained once more before the first insert consumes a reference.
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", columns));    // reload the column vector base
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index
    emitter.instruction("mov rax, QWORD PTR [r10 + r9 * 8]");                   // load the column both keys will share
    emitter.instruction("call __rt_incref");                                    // the second entry owns its own reference
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", name_ptr));   // load the resolved group name pointer
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", name_len));   // load the resolved group name length
    emitter.instruction("call __rt_hash_normalize_key");                        // apply PHP's numeric-string key normalization
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", key_lo));     // hold the normalized key across the insert
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdx", key_hi));     // hold the key discriminant across the insert
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", columns));    // reload the column vector base
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index
    emitter.instruction("mov rcx, QWORD PTR [r10 + r9 * 8]");                   // pass the column as the entry value payload
    emitter.instruction("xor r8d, r8d");                                        // an array payload has no high word
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", key_lo));     // reload the normalized key
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", key_hi));     // reload the key discriminant
    emitter.instruction("mov r9d, 4");                                          // runtime value tag 4 = indexed array
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", outer));      // reload the matrix hash pointer
    emitter.instruction("call __rt_hash_set");                                  // insert the named key ahead of its numeric twin
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // save a possibly-grown matrix hash pointer
    emitter.label("__rt_preg_match_all_capture_named_int_linux_x86_64");
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", columns));    // reload the column vector base
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index
    emitter.instruction("mov rcx, QWORD PTR [r10 + r9 * 8]");                   // pass the column as the entry value payload
    emitter.instruction("xor r8d, r8d");                                        // an array payload has no high word
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", group_idx));  // the capture index is the numeric key
    emitter.instruction("mov rdx, -1");                                         // a key discriminant of -1 marks an integer key
    emitter.instruction("mov r9d, 4");                                          // runtime value tag 4 = indexed array
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", outer));      // reload the matrix hash pointer
    emitter.instruction("call __rt_hash_set");                                  // insert this column under its numeric key
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // save a possibly-grown matrix hash pointer
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx));   // reload the column index after helper calls
    emitter.instruction("add r9, 1");                                           // advance to the next column
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", group_idx));   // save the advanced column index
    emitter.instruction("jmp __rt_preg_match_all_capture_named_publish_linux_x86_64"); // continue publishing columns

    emitter.label("__rt_preg_match_all_capture_cleanup_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", columns));    // the column ARRAYS are owned by the matrix now
    emitter.instruction("test rdi, rdi");                                       // pattern order may never have allocated the vector
    emitter.instruction("jz __rt_preg_match_all_capture_free_pairs_linux_x86_64"); // nothing to release
    emitter.bl_c("free");                                                       // release only the pointer vector itself
    emitter.label("__rt_preg_match_all_capture_free_pairs_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", pairs_ptr));  // release the offset-pair vector
    emitter.instruction("test rdi, rdi");                                       // a failed compile never allocated it
    emitter.instruction("jz __rt_preg_match_all_capture_free_handle_linux_x86_64"); // nothing to release
    emitter.bl_c("free");                                                       // release the fixed pair storage
    emitter.label("__rt_preg_match_all_capture_free_handle_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle));     // release the compiled PCRE resource
    emitter.instruction("test rdi, rdi");                                       // an invalid pattern produced no handle
    emitter.instruction("jz __rt_preg_match_all_capture_empty_linux_x86_64");   // nothing to release
    emitter.bl_c("elephc_pcre2_v1_free");                                       // dispose of the opaque compiled pattern
    emitter.label("__rt_preg_match_all_capture_empty_linux_x86_64");
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", outer));      // did any path build a capture container?
    emitter.instruction("test rax, rax");                                       // a null container means PHP's empty array
    emitter.instruction("jnz __rt_preg_match_all_capture_return_linux_x86_64"); // return the container that was built
    emitter.instruction("xor edi, edi");                                        // PHP leaves `$matches` an empty array here
    emitter.instruction("mov esi, 8");                                          // outer slots hold boxed Mixed cells
    emitter.instruction("call __rt_array_new");                                 // allocate the empty capture container
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", outer));      // return it beside the zero match count

    emitter.label("__rt_preg_match_all_capture_return_linux_x86_64");
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", count));      // return the standard integer match count
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", outer));      // return the capture container beside it
    emitter.instruction(&format!("add rsp, {}", stack_size));                   // release helper scratch storage
    emitter.instruction("pop rbp");                                             // restore the generated caller frame
    emitter.instruction("ret");                                                 // return count and capture container to lowering
}
