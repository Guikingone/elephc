//! Purpose:
//! Emits the `__rt_preg_match`, `__rt_preg_strip` runtime helper assembly for preg match.
//! Keeps PHP builtin semantics, libc/syscall boundaries, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - Regex helpers preserve PHP PCRE-flavored inputs for PCRE2 and must preserve match array construction.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// __rt_preg_match: check if a PCRE regex matches a subject string.
/// Input:  x1=pattern ptr, x2=pattern len, x3=subject ptr, x4=subject len,
///         x5=PHP flags, x6=starting byte offset
/// Output: x0=1 if match found, 0 if not
pub(crate) fn emit_preg_match(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_preg_match_linux_x86_64(emitter);
        emit_preg_match_capture_linux_x86_64(emitter);
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
    let regexec_result_off = subject_cstr_off + 8;
    let offset_off = regexec_result_off + 8;
    let stack_size = (offset_off + 40 + 15) & !15;
    let save_off = stack_size - 16;

    emitter.blank();
    emitter.comment("--- runtime: preg_match ---");
    emitter.label_global("__rt_preg_match");

    // -- set up stack frame --
    emitter.instruction(&format!("sub sp, sp, #{}", stack_size));               // allocate preg_match stack frame
    emitter.instruction(&format!("stp x29, x30, [sp, #{}]", save_off));         // save frame pointer and return address
    emitter.instruction(&format!("add x29, sp, #{}", save_off));                // set new frame pointer

    // -- save inputs --
    emitter.instruction(&format!("str x1, [sp, #{}]", pattern_ptr_off));        // save pattern ptr
    emitter.instruction(&format!("str x2, [sp, #{}]", pattern_len_off));        // save pattern len
    emitter.instruction(&format!("str x3, [sp, #{}]", subject_ptr_off));        // save subject ptr
    emitter.instruction(&format!("str x4, [sp, #{}]", subject_len_off));        // save subject len
    emitter.instruction(&format!("str x6, [sp, #{}]", offset_off));             // save requested starting offset

    // -- strip delimiters from pattern --
    emitter.instruction("bl __rt_preg_strip");                                  // → x1=stripped, x2=len, x3=flags
    emitter.instruction(&format!("str x3, [sp, #{}]", flags_off));              // save flags

    // -- materialize the PCRE pattern as a C string --
    emitter.instruction("bl __rt_pcre_to_posix");                               // materialize PCRE pattern as a C string
    emitter.instruction(&format!("str x0, [sp, #{}]", pattern_cstr_off));       // save pattern C string

    // -- prepare locale state for regex helpers --
    super::emit_prepare_regex_locale(emitter);

    // -- compile regex through the opaque Elephc shim --
    emitter.instruction(&format!("add x0, sp, #{}", handle_off));               // pass opaque-handle output storage
    emitter.instruction(&format!("ldr x1, [sp, #{}]", pattern_cstr_off));       // pass null-terminated pattern
    emitter.instruction(&format!("ldr x2, [sp, #{}]", flags_off));              // pass PCRE2 POSIX compile flags from delimiter parsing
    emitter.instruction(&format!("add x3, sp, #{}", match_slot_count_off));     // receive the compiled match-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile without exposing PCRE2-owned layouts
    emitter.instruction("cbnz x0, __rt_preg_match_no");                         // compile failed → no match

    // -- null-terminate subject --
    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_ptr_off));        // load subject ptr
    emitter.instruction(&format!("ldr x2, [sp, #{}]", subject_len_off));        // load subject len
    emitter.instruction("bl __rt_cstr2");                                       // → x0=subject C string
    emitter.instruction(&format!("str x0, [sp, #{}]", subject_cstr_off));       // save subject C string

    // -- normalize the PHP byte offset and seed REG_STARTEND bounds --
    emitter.instruction(&format!("ldr x9, [sp, #{}]", offset_off));             // load requested starting offset
    emitter.instruction("cmp x9, #0");                                          // negative offsets are relative to the subject end
    emitter.instruction("b.ge __rt_preg_match_offset_nonnegative");             // keep non-negative offsets unchanged
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_len_off));       // load subject byte length for negative normalization
    emitter.instruction("add x9, x10, x9");                                     // convert negative offset to an absolute byte offset
    emitter.label("__rt_preg_match_offset_nonnegative");
    emitter.instruction("cmp x9, #0");                                          // reject offsets before the subject start
    emitter.instruction("b.lt __rt_preg_match_cleanup_no");                     // invalid offset frees the compiled handle and reports no match
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_len_off));       // load subject byte length for upper-bound validation
    emitter.instruction("cmp x9, x10");                                         // offset may point at the trailing byte boundary
    emitter.instruction("b.gt __rt_preg_match_cleanup_no");                     // reject offsets beyond the subject after freeing the compiled handle
    emitter.instruction(&format!("str x9, [sp, #{}]", match_pair_off));         // REG_STARTEND start bound
    emitter.instruction(&format!("str x10, [sp, #{}]", match_pair_off + 8));    // REG_STARTEND end bound

    // -- execute regex through the opaque Elephc shim --
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // pass compiled opaque handle
    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_cstr_off));       // pass null-terminated subject
    emitter.instruction("mov x2, #1");                                          // request only the full-match pair
    emitter.instruction(&format!("add x3, sp, #{}", match_pair_off));           // receive one fixed signed-64-bit offset pair
    emitter.instruction("mov x4, #128");                                        // REG_STARTEND preserves full-subject anchor semantics with offsets
    emitter.instruction(&format!("ldr x9, [sp, #{}]", match_pair_off));         // reload the normalized starting offset
    emitter.instruction("cmp x9, #0");                                          // a non-zero start is not the beginning of the full subject
    emitter.instruction("orr x10, x4, #4");                                     // REG_NOTBOL keeps ^ anchored to the full subject
    emitter.instruction("csel x4, x10, x4, gt");                                // add REG_NOTBOL only for positive offsets
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // execute without exposing PCRE2-owned layouts
    emitter.instruction(&format!("str x0, [sp, #{}]", regexec_result_off));     // save regexec result

    // -- free compiled regex --
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // reload compiled opaque handle
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources

    // -- return result --
    emitter.instruction(&format!("ldr x0, [sp, #{}]", regexec_result_off));     // reload regexec result
    emitter.instruction("cbnz x0, __rt_preg_match_no");                         // non-zero = no match
    emitter.instruction("mov x0, #1");                                          // matched → return 1
    emitter.instruction("b __rt_preg_match_ret");                               // return

    emitter.label("__rt_preg_match_cleanup_no");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // reload compiled handle for invalid-offset cleanup
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources before reporting no match
    emitter.instruction("b __rt_preg_match_no");                                // share the zero-result path

    emitter.label("__rt_preg_match_no");
    emitter.instruction("mov x0, #0");                                          // no match → return 0

    emitter.label("__rt_preg_match_ret");
    emitter.instruction(&format!("ldp x29, x30, [sp, #{}]", save_off));         // restore frame pointer and return address
    emitter.instruction(&format!("add sp, sp, #{}", stack_size));               // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    emit_preg_match_capture_arm64(emitter);
}

/// Emits ARM64 `__rt_preg_match_capture`, returning the match flag and `$matches` array.
fn emit_preg_match_capture_arm64(emitter: &mut Emitter) {
    let handle_off = 0;
    let regmatches_ptr_off = handle_off + 8;
    let nmatch_off = regmatches_ptr_off + 8;
    let pattern_ptr_off = nmatch_off + 8;
    let pattern_len_off = pattern_ptr_off + 8;
    let subject_ptr_off = pattern_len_off + 8;
    let subject_len_off = subject_ptr_off + 8;
    let flags_off = subject_len_off + 8;
    let pattern_cstr_off = flags_off + 8;
    let subject_cstr_off = pattern_cstr_off + 8;
    let regexec_result_off = subject_cstr_off + 8;
    let matches_array_off = regexec_result_off + 8;
    let group_idx_off = matches_array_off + 8;
    let max_group_off = group_idx_off + 8;
    let offset_off = max_group_off + 8;
    let name_ptr_off = offset_off + 8;
    let name_len_off = name_ptr_off + 8;
    let value_ptr_off = name_len_off + 8;
    let value_len_off = value_ptr_off + 8;
    let key_lo_off = value_len_off + 8;
    let key_hi_off = key_lo_off + 8;
    let stack_size = (key_hi_off + 8 + 48 + 15) & !15;
    let save_off = stack_size - 16;

    emitter.blank();
    emitter.comment("--- runtime: preg_match_capture ---");
    emitter.label_global("__rt_preg_match_capture");

    // -- set up stack frame --
    emitter.instruction(&format!("sub sp, sp, #{}", stack_size));               // allocate preg_match capture stack frame
    emitter.instruction(&format!("add x9, sp, #{}", save_off));                 // compute save-slot address beyond ARM64 pair-store range
    emitter.instruction("stp x29, x30, [x9]");                                  // save frame pointer and return address
    emitter.instruction(&format!("add x29, sp, #{}", save_off));                // establish the preg_match capture frame pointer

    // -- save inputs --
    emitter.instruction(&format!("str x1, [sp, #{}]", pattern_ptr_off));        // save pattern ptr
    emitter.instruction(&format!("str x2, [sp, #{}]", pattern_len_off));        // save pattern len
    emitter.instruction(&format!("str x3, [sp, #{}]", subject_ptr_off));        // save subject ptr
    emitter.instruction(&format!("str x4, [sp, #{}]", subject_len_off));        // save subject len
    emitter.instruction(&format!("str x6, [sp, #{}]", offset_off));             // save requested starting offset

    // -- strip delimiters and compile PCRE regex --
    emitter.instruction("bl __rt_preg_strip");                                  // strip delimiters and expose regex flags
    emitter.instruction(&format!("str x3, [sp, #{}]", flags_off));              // save stripped regex flags
    emitter.instruction("bl __rt_pcre_to_posix");                               // materialize PCRE pattern as a C string
    emitter.instruction(&format!("str x0, [sp, #{}]", pattern_cstr_off));       // save null-terminated PCRE pattern
    super::emit_prepare_regex_locale(emitter);
    emitter.instruction(&format!("add x0, sp, #{}", handle_off));               // pass opaque-handle output storage
    emitter.instruction(&format!("ldr x1, [sp, #{}]", pattern_cstr_off));       // pass null-terminated PCRE pattern
    emitter.instruction(&format!("ldr x2, [sp, #{}]", flags_off));              // pass PCRE2 POSIX compile flags from delimiter parsing
    emitter.instruction(&format!("add x3, sp, #{}", nmatch_off));               // receive the compiled match-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile without exposing PCRE2-owned layouts
    emitter.instruction("cbnz x0, __rt_preg_match_capture_empty");              // compile failure produces no match and an empty matches array

    // -- allocate fixed Elephc offset pairs for every compiled capture --
    emitter.instruction(&format!("ldr x9, [sp, #{}]", nmatch_off));             // load match-slot count returned by the shim
    emitter.instruction("lsr x10, x9, #60");                                    // reject slot counts whose 16-byte size would overflow
    emitter.instruction("cbnz x10, __rt_preg_match_capture_malloc_fail");       // free the handle instead of allocating a wrapped size
    emitter.instruction("lsl x0, x9, #4");                                      // allocate one 16-byte signed-64-bit pair per slot
    emitter.bl_c("malloc");                                                     // allocate the fixed offset-pair vector
    emitter.instruction("cbz x0, __rt_preg_match_capture_malloc_fail");         // allocation failure returns no match after freeing the handle
    emitter.instruction(&format!("str x0, [sp, #{}]", regmatches_ptr_off));     // save dynamic offset-pair buffer pointer

    // -- null-terminate subject and run regex --
    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_ptr_off));        // reload subject ptr
    emitter.instruction(&format!("ldr x2, [sp, #{}]", subject_len_off));        // reload subject len
    emitter.instruction("bl __rt_cstr2");                                       // materialize null-terminated subject copy
    emitter.instruction(&format!("str x0, [sp, #{}]", subject_cstr_off));       // save subject C string
    emitter.instruction(&format!("ldr x9, [sp, #{}]", offset_off));             // load requested starting offset
    emitter.instruction("cmp x9, #0");                                          // negative offsets are relative to the subject end
    emitter.instruction("b.ge __rt_preg_match_capture_offset_nonnegative");     // keep non-negative offsets unchanged
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_len_off));       // load subject length for negative normalization
    emitter.instruction("add x9, x10, x9");                                     // convert negative offset to absolute byte position
    emitter.label("__rt_preg_match_capture_offset_nonnegative");
    emitter.instruction("cmp x9, #0");                                          // reject positions before the subject
    emitter.instruction("b.lt __rt_preg_match_capture_invalid_offset");         // release capture resources on invalid offsets
    emitter.instruction(&format!("ldr x10, [sp, #{}]", subject_len_off));       // load subject length for upper-bound validation
    emitter.instruction("cmp x9, x10");                                         // allow the trailing byte boundary only
    emitter.instruction("b.gt __rt_preg_match_capture_invalid_offset");         // reject offsets beyond the subject
    emitter.instruction(&format!("str x9, [sp, #{}]", offset_off));             // persist the normalized offset for execution flags and output offsets
    emitter.instruction(&format!("ldr x11, [sp, #{}]", regmatches_ptr_off));    // load dynamic pair buffer
    emitter.instruction("str x9, [x11]");                                       // seed REG_STARTEND start bound
    emitter.instruction("str x10, [x11, #8]");                                  // seed REG_STARTEND end bound
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // pass compiled opaque handle
    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_cstr_off));       // pass subject C string to regexec
    emitter.instruction(&format!("ldr x2, [sp, #{}]", nmatch_off));             // request one regmatch slot for every capture group
    emitter.instruction(&format!("ldr x3, [sp, #{}]", regmatches_ptr_off));     // pass dynamic fixed offset-pair buffer
    emitter.instruction("mov x4, #128");                                        // REG_STARTEND searches from the normalized byte offset
    emitter.instruction(&format!("ldr x9, [sp, #{}]", offset_off));             // reload the normalized starting offset
    emitter.instruction("cmp x9, #0");                                          // a non-zero start is not the beginning of the full subject
    emitter.instruction("orr x10, x4, #4");                                     // REG_NOTBOL keeps ^ anchored to the full subject
    emitter.instruction("csel x4, x10, x4, gt");                                // add REG_NOTBOL only for positive offsets
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // execute and initialize every requested offset pair
    emitter.instruction(&format!("str x0, [sp, #{}]", regexec_result_off));     // save regexec status across the matches construction
    emitter.instruction("cbnz x0, __rt_preg_match_capture_no_match");           // no match releases the pattern and capture storage

    // -- find highest populated capture so trailing unmatched groups are omitted --
    emitter.instruction(&format!("ldr x12, [sp, #{}]", nmatch_off));            // reload the dynamic regmatch count
    emitter.instruction("sub x12, x12, #1");                                    // start scanning from the last compiled capture slot
    emitter.label("__rt_preg_match_capture_scan");
    emitter.instruction("lsl x14, x12, #4");                                    // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("ldr x15, [sp, #{}]", regmatches_ptr_off));    // load dynamic offset-pair buffer base
    emitter.instruction("add x14, x15, x14");                                   // compute address of this offset pair
    emitter.instruction("ldr x13, [x14]");                                      // load signed-64-bit capture start
    emitter.instruction("cmp x13, #0");                                         // check whether this capture participated
    emitter.instruction("b.ge __rt_preg_match_capture_scan_found");             // use this as the last emitted capture
    emitter.instruction("cbz x12, __rt_preg_match_capture_scan_found");         // keep at least the full match slot after a successful regexec
    emitter.instruction("sub x12, x12, #1");                                    // move to the previous capture slot
    emitter.instruction("b __rt_preg_match_capture_scan");                      // continue searching for the highest populated capture
    emitter.label("__rt_preg_match_capture_scan_found");
    emitter.instruction(&format!("str x12, [sp, #{}]", max_group_off));         // save highest capture index to materialize

    // -- a compiled name table means PHP's $matches is an ordered hash, not a list --
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // the name table lives inside the compiled pattern
    emitter.bl_c("elephc_pcre2_v1_name_count");                                 // ask PCRE2 how many capture groups were named
    emitter.instruction("cbnz x0, __rt_preg_match_capture_named");              // any named group forces the hash-backed matches array

    // -- allocate and fill matches array --
    emitter.instruction(&format!("ldr x0, [sp, #{}]", nmatch_off));             // allocate enough slots for every compiled capture
    emitter.instruction("mov x1, #16");                                         // string arrays use pointer/length payload slots
    emitter.instruction("bl __rt_array_new");                                   // allocate indexed string matches array
    emitter.instruction(&format!("str x0, [sp, #{}]", matches_array_off));      // save matches array pointer across pushes
    emitter.instruction(&format!("str xzr, [sp, #{}]", group_idx_off));         // start with capture index zero
    emitter.label("__rt_preg_match_capture_group_loop");
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload current capture index
    emitter.instruction(&format!("ldr x13, [sp, #{}]", max_group_off));         // reload highest capture index
    emitter.instruction("cmp x12, x13");                                        // have all required captures been materialized?
    emitter.instruction("b.gt __rt_preg_match_capture_success");                // finish after the highest populated capture
    emitter.instruction("lsl x14, x12, #4");                                    // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("ldr x17, [sp, #{}]", regmatches_ptr_off));    // load dynamic offset-pair buffer base
    emitter.instruction("add x14, x17, x14");                                   // compute address of this offset pair
    emitter.instruction("ldr x15, [x14]");                                      // load signed-64-bit capture start
    emitter.instruction("ldr x16, [x14, #8]");                                  // load signed-64-bit capture end
    emitter.instruction("cmp x15, #0");                                         // detect unmatched captures before the highest populated slot
    emitter.instruction("b.lt __rt_preg_match_capture_empty_string");           // emit PHP's empty string for an interior unmatched capture
    emitter.instruction("sub x2, x16, x15");                                    // capture length = rm_eo - rm_so
    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_cstr_off));       // reload subject C string base
    emitter.instruction("add x1, x1, x15");                                     // compute capture string pointer
    emitter.instruction("b __rt_preg_match_capture_push");                      // append this capture string
    emitter.label("__rt_preg_match_capture_empty_string");
    emitter.instruction("mov x1, #0");                                          // empty unmatched capture has a null pointer
    emitter.instruction("mov x2, #0");                                          // empty unmatched capture has zero length
    emitter.label("__rt_preg_match_capture_push");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", matches_array_off));      // reload matches array pointer
    emitter.instruction("bl __rt_array_push_str");                              // append capture string to matches array
    emitter.instruction(&format!("str x0, [sp, #{}]", matches_array_off));      // save possibly-grown matches array pointer
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload capture index after helper calls
    emitter.instruction("add x12, x12, #1");                                    // advance to next capture index
    emitter.instruction(&format!("str x12, [sp, #{}]", group_idx_off));         // save next capture index
    emitter.instruction("b __rt_preg_match_capture_group_loop");                // continue materializing captures


    // -- named captures: PHP writes each name immediately before its own numeric key --
    emitter.label("__rt_preg_match_capture_named");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", nmatch_off));             // size the table from the compiled capture count
    emitter.instruction("lsl x0, x0, #1");                                      // reserve one bucket per numeric key and one per name
    emitter.instruction("mov x1, #1");                                          // string-valued hash: the shape a PHP string array literal builds
    emitter.instruction("bl __rt_hash_new");                                    // allocate the ordered hash that carries both key kinds
    emitter.instruction(&format!("str x0, [sp, #{}]", matches_array_off));      // save matches hash pointer across inserts
    emitter.instruction(&format!("str xzr, [sp, #{}]", group_idx_off));         // start with capture index zero
    emitter.label("__rt_preg_match_capture_named_loop");
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload current capture index
    emitter.instruction(&format!("ldr x13, [sp, #{}]", max_group_off));         // reload highest capture index
    emitter.instruction("cmp x12, x13");                                        // have all required captures been materialized?
    emitter.instruction("b.gt __rt_preg_match_capture_success");                // finish after the highest populated capture
    emitter.instruction("lsl x14, x12, #4");                                    // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("ldr x15, [sp, #{}]", regmatches_ptr_off));    // load dynamic offset-pair buffer base
    emitter.instruction("add x14, x15, x14");                                   // compute address of this offset pair
    emitter.instruction("ldr x15, [x14]");                                      // load signed-64-bit capture start
    emitter.instruction("ldr x13, [x14, #8]");                                  // load signed-64-bit capture end
    emitter.instruction("cmp x15, #0");                                         // detect unmatched captures before the highest populated slot
    emitter.instruction("b.lt __rt_preg_match_capture_named_empty");            // emit PHP's empty string for an interior unmatched capture
    emitter.instruction("sub x2, x13, x15");                                    // capture length = rm_eo - rm_so
    emitter.instruction(&format!("ldr x1, [sp, #{}]", subject_cstr_off));       // reload subject C string base
    emitter.instruction("add x1, x1, x15");                                     // compute capture string pointer
    emitter.instruction("b __rt_preg_match_capture_named_value");               // insert this capture under both key kinds
    emitter.label("__rt_preg_match_capture_named_empty");
    emitter.instruction("mov x1, #0");                                          // empty unmatched capture has a null pointer
    emitter.instruction("mov x2, #0");                                          // empty unmatched capture has zero length
    emitter.label("__rt_preg_match_capture_named_value");
    emitter.instruction(&format!("str x1, [sp, #{}]", value_ptr_off));          // hold the capture pointer across helper calls
    emitter.instruction(&format!("str x2, [sp, #{}]", value_len_off));          // hold the capture length across helper calls
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // pass the compiled pattern holding the name table
    emitter.instruction(&format!("ldr x1, [sp, #{}]", group_idx_off));          // ask for this capture group's declared name
    emitter.instruction(&format!("add x2, sp, #{}", name_ptr_off));             // receive the name pointer into pattern storage
    emitter.instruction(&format!("add x3, sp, #{}", name_len_off));             // receive the name length
    emitter.bl_c("elephc_pcre2_v1_group_name");                                 // resolve the name without exposing PCRE2 table layouts
    emitter.instruction("cbnz w0, __rt_preg_match_capture_named_int");          // an unnamed group only gets its numeric key
    emitter.instruction(&format!("ldr x1, [sp, #{}]", name_ptr_off));           // load the resolved group name pointer
    emitter.instruction(&format!("ldr x2, [sp, #{}]", name_len_off));           // load the resolved group name length
    emitter.instruction("bl __rt_hash_normalize_key");                          // apply PHP's numeric-string key normalization
    emitter.instruction(&format!("str x1, [sp, #{}]", key_lo_off));             // hold the normalized key across the value persist
    emitter.instruction(&format!("str x2, [sp, #{}]", key_hi_off));             // hold the key discriminant across the value persist
    emitter.instruction(&format!("ldr x1, [sp, #{}]", value_ptr_off));          // reload the capture pointer to persist
    emitter.instruction(&format!("ldr x2, [sp, #{}]", value_len_off));          // reload the capture length to persist
    emitter.instruction("bl __rt_str_persist");                                 // the named entry owns its own copy of the capture bytes
    emitter.instruction("mov x3, x1");                                          // pass the persisted capture as the entry value payload
    emitter.instruction("mov x4, x2");                                          // pass the persisted capture length
    emitter.instruction(&format!("ldr x1, [sp, #{}]", key_lo_off));             // reload the normalized key
    emitter.instruction(&format!("ldr x2, [sp, #{}]", key_hi_off));             // reload the key discriminant
    emitter.instruction(&format!("ldr x0, [sp, #{}]", matches_array_off));      // reload matches hash pointer
    emitter.instruction("mov x5, #1");                                          // runtime value tag 1 = string
    emitter.instruction("bl __rt_hash_set");                                    // insert the named key ahead of its numeric twin
    emitter.instruction(&format!("str x0, [sp, #{}]", matches_array_off));      // save possibly-grown matches hash pointer
    emitter.label("__rt_preg_match_capture_named_int");
    emitter.instruction(&format!("ldr x1, [sp, #{}]", value_ptr_off));          // reload the capture pointer to persist
    emitter.instruction(&format!("ldr x2, [sp, #{}]", value_len_off));          // reload the capture length to persist
    emitter.instruction("bl __rt_str_persist");                                 // the numeric entry owns its own copy of the capture bytes
    emitter.instruction("mov x3, x1");                                          // pass the persisted capture as the entry value payload
    emitter.instruction("mov x4, x2");                                          // pass the persisted capture length
    emitter.instruction(&format!("ldr x1, [sp, #{}]", group_idx_off));          // the capture index is the numeric key
    emitter.instruction("mov x2, #-1");                                         // a key discriminant of -1 marks an integer key
    emitter.instruction(&format!("ldr x0, [sp, #{}]", matches_array_off));      // reload matches hash pointer
    emitter.instruction("mov x5, #1");                                          // runtime value tag 1 = string
    emitter.instruction("bl __rt_hash_set");                                    // insert this capture under its numeric key
    emitter.instruction(&format!("str x0, [sp, #{}]", matches_array_off));      // save possibly-grown matches hash pointer
    emitter.instruction(&format!("ldr x12, [sp, #{}]", group_idx_off));         // reload capture index after helper calls
    emitter.instruction("add x12, x12, #1");                                    // advance to next capture index
    emitter.instruction(&format!("str x12, [sp, #{}]", group_idx_off));         // save next capture index
    emitter.instruction("b __rt_preg_match_capture_named_loop");                // continue materializing captures

    emitter.label("__rt_preg_match_capture_success");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // reload the pattern now every group name has been read
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources
    emitter.instruction(&format!("ldr x0, [sp, #{}]", regmatches_ptr_off));     // reload dynamic offset-pair buffer for cleanup
    emitter.bl_c("free");                                                       // free the dynamic offset-pair vector before returning matches
    emitter.instruction("mov x0, #1");                                          // report that preg_match found a match
    emitter.instruction(&format!("ldr x1, [sp, #{}]", matches_array_off));      // return the matches array pointer in x1
    emitter.instruction("b __rt_preg_match_capture_ret");                       // share helper epilogue

    emitter.label("__rt_preg_match_capture_no_match");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // reload compiled handle for the no-match cleanup path
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources
    emitter.instruction("b __rt_preg_match_capture_free_pairs");                // share the capture-buffer cleanup

    emitter.label("__rt_preg_match_capture_invalid_offset");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // reload compiled handle for invalid-offset cleanup
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources

    emitter.label("__rt_preg_match_capture_free_pairs");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", regmatches_ptr_off));     // reload dynamic capture buffer for cleanup
    emitter.bl_c("free");                                                       // free the dynamic offset-pair vector before returning an empty matches array
    emitter.instruction("b __rt_preg_match_capture_empty");                     // allocate and return the empty matches array

    emitter.label("__rt_preg_match_capture_malloc_fail");
    emitter.instruction(&format!("ldr x0, [sp, #{}]", handle_off));             // reload opaque handle after capture-buffer allocation failed
    emitter.bl_c("elephc_pcre2_v1_free");                                       // free compiled regex resources before returning no match

    emitter.label("__rt_preg_match_capture_empty");
    emitter.instruction("mov x0, #0");                                          // empty array capacity for no-match or compile-failure paths
    emitter.instruction("mov x1, #16");                                         // empty matches array still has string slot metadata
    emitter.instruction("bl __rt_array_new");                                   // allocate empty indexed string matches array
    emitter.instruction("mov x1, x0");                                          // return empty matches array in x1
    emitter.instruction("mov x0, #0");                                          // report no match

    emitter.label("__rt_preg_match_capture_ret");
    emitter.instruction(&format!("add x9, sp, #{}", save_off));                 // compute save-slot address for epilogue restore
    emitter.instruction("ldp x29, x30, [x9]");                                  // restore frame pointer and return address
    emitter.instruction(&format!("add sp, sp, #{}", stack_size));               // deallocate preg_match capture stack frame
    emitter.instruction("ret");                                                 // return match flag in x0 and matches array in x1
}

/// Emits the x86_64 Linux variant of `__rt_preg_match`.
/// Called from `emit_preg_match` when `target.arch == Arch::X86_64`.
/// Uses System V AMD64 ABI: pattern in rdi/rsi, subject in rdx/rcx, result in eax.
/// Compiles and executes through the versioned opaque PCRE2 shim, then frees its handle.
fn emit_preg_match_linux_x86_64(emitter: &mut Emitter) {
    let handle_off = 0;
    let match_slot_count_off = handle_off + 8;
    let match_pair_off = match_slot_count_off + 8;
    let subject_ptr_off = match_pair_off + 16;
    let subject_len_off = subject_ptr_off + 8;
    let flags_off = subject_len_off + 8;
    let pattern_cstr_off = flags_off + 8;
    let subject_cstr_off = pattern_cstr_off + 8;
    let regexec_result_off = subject_cstr_off + 8;
    let offset_off = regexec_result_off + 8;
    let stack_size = (offset_off + 16 + 15) & !15;

    emitter.blank();
    emitter.comment("--- runtime: preg_match ---");
    emitter.label_global("__rt_preg_match");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving regex compilation scratch storage
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the regex object, regmatch buffer, and subject spill slots
    emitter.instruction(&format!("sub rsp, {}", stack_size));                   // reserve aligned local storage for the opaque handle, pair, and spill fields
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdx", subject_ptr_off)); // preserve the elephc subject pointer across delimiter stripping and regex compilation helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", subject_len_off)); // preserve the elephc subject length across delimiter stripping and regex compilation helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", offset_off));  // preserve the requested starting offset across helper calls
    emitter.instruction("mov rax, rdi");                                        // move the elephc pattern pointer into the preg-strip helper input register
    emitter.instruction("mov rdx, rsi");                                        // move the elephc pattern length into the preg-strip helper input register
    emitter.instruction("call __rt_preg_strip");                                // strip slash delimiters and collect supported regex flags from the elephc pattern payload
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", flags_off));  // preserve the regex flag word returned by the delimiter-strip helper for the later regcomp() call
    emitter.instruction("call __rt_pcre_to_posix");                             // materialize PCRE pattern as a null-terminated C string
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", pattern_cstr_off)); // preserve the null-terminated PCRE pattern pointer for the upcoming regcomp() call
    super::emit_prepare_regex_locale(emitter);
    emitter.instruction(&format!("lea rdi, [rsp + {}]", handle_off));           // pass opaque-handle output storage
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", pattern_cstr_off)); // pass the null-terminated PCRE pattern C string as the second regcomp() argument
    emitter.instruction(&format!("mov edx, DWORD PTR [rsp + {}]", flags_off));  // pass PCRE2 POSIX compile flags from delimiter parsing
    emitter.instruction(&format!("lea rcx, [rsp + {}]", match_slot_count_off)); // receive the compiled match-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile without exposing PCRE2-owned layouts
    emitter.instruction("test eax, eax");                                       // did regcomp() succeed and produce a compiled regex object?
    emitter.instruction("jnz __rt_preg_match_no_linux_x86_64");                 // failed regex compilation maps to a PHP false-style no-match result
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", subject_ptr_off)); // reload the elephc subject pointer before null-terminating it in the secondary scratch buffer
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", subject_len_off)); // reload the elephc subject length before null-terminating it in the secondary scratch buffer
    emitter.instruction("call __rt_cstr2");                                     // materialize a null-terminated C version of the subject string for PCRE2 regex execution
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", subject_cstr_off)); // preserve the subject C string pointer for the regexec() call and later cleanup path
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", offset_off));  // load requested starting offset
    emitter.instruction("cmp r9, 0");                                           // negative offsets are relative to the subject end
    emitter.instruction("jge __rt_preg_match_offset_nonnegative_linux_x86_64"); // keep non-negative offsets unchanged
    emitter.instruction(&format!("add r9, QWORD PTR [rsp + {}]", subject_len_off)); // convert negative offset to an absolute byte position
    emitter.label("__rt_preg_match_offset_nonnegative_linux_x86_64");
    emitter.instruction("cmp r9, 0");                                           // reject positions before the subject start
    emitter.instruction("jl __rt_preg_match_cleanup_no_linux_x86_64");          // release the compiled handle on invalid offsets
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", subject_len_off)); // load subject byte length for upper-bound validation
    emitter.instruction("cmp r9, r10");                                         // allow the trailing byte boundary only
    emitter.instruction("jg __rt_preg_match_cleanup_no_linux_x86_64");          // reject positions beyond the subject
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", match_pair_off)); // seed REG_STARTEND start bound
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r10", match_pair_off + 8)); // seed REG_STARTEND end bound
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // pass compiled opaque handle
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", subject_cstr_off)); // pass the null-terminated subject C string as the second regexec() argument
    emitter.instruction("mov edx, 1");                                          // request only the full-match pair
    emitter.instruction(&format!("lea rcx, [rsp + {}]", match_pair_off));       // receive one fixed signed-64-bit offset pair
    emitter.instruction("mov r8d, 128");                                        // REG_STARTEND preserves full-subject anchor semantics with offsets
    emitter.instruction(&format!("cmp QWORD PTR [rsp + {}], 0", match_pair_off)); // a non-zero start is not the beginning of the full subject
    emitter.instruction("jle __rt_preg_match_exec_flags_ready_linux_x86_64");   // retain only REG_STARTEND at offset zero
    emitter.instruction("or r8d, 4");                                           // REG_NOTBOL keeps ^ anchored to the full subject
    emitter.label("__rt_preg_match_exec_flags_ready_linux_x86_64");
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // execute without exposing PCRE2-owned layouts
    emitter.instruction(&format!("mov DWORD PTR [rsp + {}], eax", regexec_result_off)); // preserve the regexec() result code across the mandatory regfree() cleanup call
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // reload compiled opaque handle
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources
    emitter.instruction(&format!("mov eax, DWORD PTR [rsp + {}]", regexec_result_off)); // reload the saved regexec() status code after regfree() clobbered caller-saved registers
    emitter.instruction("test eax, eax");                                       // interpret a zero regexec() result as a successful regex match
    emitter.instruction("jnz __rt_preg_match_no_linux_x86_64");                 // return zero when PCRE2 regex execution reports no match
    emitter.instruction("mov eax, 1");                                          // return one when PCRE2 regex execution reports a successful match
    emitter.instruction("jmp __rt_preg_match_ret_linux_x86_64");                // share the common epilogue after materializing the successful match result

    emitter.label("__rt_preg_match_cleanup_no_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // reload compiled handle for invalid-offset cleanup
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources before reporting no match
    emitter.instruction("jmp __rt_preg_match_no_linux_x86_64");                 // share the zero-result path

    emitter.label("__rt_preg_match_no_linux_x86_64");
    emitter.instruction("xor eax, eax");                                        // return zero for compile failures and subjects that do not match the regex

    emitter.label("__rt_preg_match_ret_linux_x86_64");
    emitter.instruction(&format!("add rsp, {}", stack_size));                   // release the opaque-handle, pair, and subject spill storage before returning
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the regex helper completes
    emitter.instruction("ret");                                                 // return the preg_match() integer result in the x86_64 integer result register
}

/// Emits x86_64 `__rt_preg_match_capture`, returning the match flag and `$matches` array.
fn emit_preg_match_capture_linux_x86_64(emitter: &mut Emitter) {
    let handle_off = 0;
    let regmatches_ptr_off = handle_off + 8;
    let nmatch_off = regmatches_ptr_off + 8;
    let subject_ptr_off = nmatch_off + 8;
    let subject_len_off = subject_ptr_off + 8;
    let flags_off = subject_len_off + 8;
    let pattern_cstr_off = flags_off + 8;
    let subject_cstr_off = pattern_cstr_off + 8;
    let regexec_result_off = subject_cstr_off + 8;
    let matches_array_off = regexec_result_off + 8;
    let group_idx_off = matches_array_off + 8;
    let max_group_off = group_idx_off + 8;
    let offset_off = max_group_off + 8;
    let name_ptr_off = offset_off + 8;
    let name_len_off = name_ptr_off + 8;
    let value_ptr_off = name_len_off + 8;
    let value_len_off = value_ptr_off + 8;
    let key_lo_off = value_len_off + 8;
    let key_hi_off = key_lo_off + 8;
    let stack_size = (key_hi_off + 8 + 32 + 15) & !15;

    emitter.blank();
    emitter.comment("--- runtime: preg_match_capture ---");
    emitter.label_global("__rt_preg_match_capture");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving capture helper storage
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for regex and capture spill slots
    emitter.instruction(&format!("sub rsp, {}", stack_size));                   // reserve local storage for the opaque handle, pair buffer, and matches state
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdx", subject_ptr_off)); // preserve the elephc subject pointer across pattern helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", subject_len_off)); // preserve the elephc subject length across pattern helper calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", offset_off));  // preserve requested starting offset across pattern helper calls
    emitter.instruction("mov rax, rdi");                                        // move pattern pointer into preg-strip helper input register
    emitter.instruction("mov rdx, rsi");                                        // move pattern length into preg-strip helper input register
    emitter.instruction("call __rt_preg_strip");                                // strip slash delimiters and collect supported regex flags
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rcx", flags_off));  // save stripped regex flags for regcomp
    emitter.instruction("call __rt_pcre_to_posix");                             // materialize PCRE pattern as a C string
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", pattern_cstr_off)); // save null-terminated PCRE pattern for regcomp
    super::emit_prepare_regex_locale(emitter);
    emitter.instruction(&format!("lea rdi, [rsp + {}]", handle_off));           // pass opaque-handle output storage
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", pattern_cstr_off)); // pass null-terminated PCRE pattern to regcomp
    emitter.instruction(&format!("mov edx, DWORD PTR [rsp + {}]", flags_off));  // pass PCRE2 POSIX compile flags from delimiter parsing
    emitter.instruction(&format!("lea rcx, [rsp + {}]", nmatch_off));           // receive the compiled match-slot count
    emitter.bl_c("elephc_pcre2_v1_compile");                                    // compile without exposing PCRE2-owned layouts
    emitter.instruction("test eax, eax");                                       // did regex compilation succeed?
    emitter.instruction("jnz __rt_preg_match_capture_empty_linux_x86_64");      // compile failure returns no match and an empty matches array
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", nmatch_off)); // load match-slot count returned by the shim
    emitter.instruction("mov r10, rdi");                                        // copy slot count for allocation-overflow validation
    emitter.instruction("shr r10, 60");                                         // detect a wrapped 16-byte pair-vector size
    emitter.instruction("jnz __rt_preg_match_capture_malloc_fail_linux_x86_64"); // free the handle instead of allocating a wrapped size
    emitter.instruction("shl rdi, 4");                                          // allocate one 16-byte signed-64-bit pair per slot
    emitter.bl_c("malloc");                                                     // allocate the fixed offset-pair vector
    emitter.instruction("test rax, rax");                                       // did malloc return a capture buffer?
    emitter.instruction("jz __rt_preg_match_capture_malloc_fail_linux_x86_64"); // allocation failure frees the opaque handle and returns no match
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", regmatches_ptr_off)); // save dynamic offset-pair buffer pointer
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", subject_ptr_off)); // reload subject pointer before C-string conversion
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", subject_len_off)); // reload subject length before C-string conversion
    emitter.instruction("call __rt_cstr2");                                     // materialize null-terminated subject for regexec
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", subject_cstr_off)); // save subject C string across regexec and pushes
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", offset_off));  // load requested starting offset
    emitter.instruction("cmp r9, 0");                                           // negative offsets are relative to the subject end
    emitter.instruction("jge __rt_preg_match_capture_offset_nonnegative_linux_x86_64"); // keep non-negative offsets unchanged
    emitter.instruction(&format!("add r9, QWORD PTR [rsp + {}]", subject_len_off)); // convert negative offset to an absolute byte position
    emitter.label("__rt_preg_match_capture_offset_nonnegative_linux_x86_64");
    emitter.instruction("cmp r9, 0");                                           // reject positions before the subject start
    emitter.instruction("jl __rt_preg_match_capture_invalid_offset_linux_x86_64"); // release capture resources on invalid offsets
    emitter.instruction(&format!("mov r10, QWORD PTR [rsp + {}]", subject_len_off)); // load subject length for upper-bound validation
    emitter.instruction("cmp r9, r10");                                         // allow the trailing byte boundary only
    emitter.instruction("jg __rt_preg_match_capture_invalid_offset_linux_x86_64"); // reject offsets beyond the subject
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", offset_off));  // persist the normalized offset for execution flags and output offsets
    emitter.instruction(&format!("mov r11, QWORD PTR [rsp + {}]", regmatches_ptr_off)); // load dynamic pair buffer
    emitter.instruction("mov QWORD PTR [r11], r9");                             // seed REG_STARTEND start bound
    emitter.instruction("mov QWORD PTR [r11 + 8], r10");                        // seed REG_STARTEND end bound
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // pass compiled opaque handle
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", subject_cstr_off)); // pass subject C string to regexec
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", nmatch_off)); // request one regmatch slot for every capture group
    emitter.instruction(&format!("mov rcx, QWORD PTR [rsp + {}]", regmatches_ptr_off)); // pass dynamic fixed offset-pair buffer
    emitter.instruction("mov r8d, 128");                                        // REG_STARTEND searches from the normalized byte offset
    emitter.instruction(&format!("cmp QWORD PTR [rsp + {}], 0", offset_off));   // a non-zero start is not the beginning of the full subject
    emitter.instruction("jle __rt_preg_match_capture_exec_flags_ready_linux_x86_64"); // retain only REG_STARTEND at offset zero
    emitter.instruction("or r8d, 4");                                           // REG_NOTBOL keeps ^ anchored to the full subject
    emitter.label("__rt_preg_match_capture_exec_flags_ready_linux_x86_64");
    emitter.bl_c("elephc_pcre2_v1_exec");                                       // execute and initialize every requested offset pair
    emitter.instruction(&format!("mov DWORD PTR [rsp + {}], eax", regexec_result_off)); // save regexec status across the matches construction
    emitter.instruction("test eax, eax");                                       // was there a successful regex match?
    emitter.instruction("jnz __rt_preg_match_capture_no_match_linux_x86_64");   // no match releases the pattern and capture storage

    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", nmatch_off));  // reload the dynamic regmatch count
    emitter.instruction("sub r9, 1");                                           // start scanning from the last compiled capture slot
    emitter.label("__rt_preg_match_capture_scan_linux_x86_64");
    emitter.instruction("mov r10, r9");                                         // copy capture index before scaling
    emitter.instruction("shl r10, 4");                                          // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("mov r12, QWORD PTR [rsp + {}]", regmatches_ptr_off)); // load dynamic offset-pair buffer base
    emitter.instruction("add r10, r12");                                        // compute address of this offset pair
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // load signed-64-bit capture start
    emitter.instruction("cmp r11, 0");                                          // check whether this capture participated
    emitter.instruction("jge __rt_preg_match_capture_scan_found_linux_x86_64"); // use this as the highest emitted capture
    emitter.instruction("test r9, r9");                                         // have we reached the full-match slot?
    emitter.instruction("jz __rt_preg_match_capture_scan_found_linux_x86_64");  // keep at least the full match after successful regexec
    emitter.instruction("sub r9, 1");                                           // move to the previous capture slot
    emitter.instruction("jmp __rt_preg_match_capture_scan_linux_x86_64");       // continue searching for the highest populated capture
    emitter.label("__rt_preg_match_capture_scan_found_linux_x86_64");
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", max_group_off)); // save highest capture index to materialize

    // -- a compiled name table means PHP's $matches is an ordered hash, not a list --
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // the name table lives inside the compiled pattern
    emitter.bl_c("elephc_pcre2_v1_name_count");                                 // ask PCRE2 how many capture groups were named
    emitter.instruction("test rax, rax");                                       // did the pattern declare any group name?
    emitter.instruction("jnz __rt_preg_match_capture_named_linux_x86_64");      // any named group forces the hash-backed matches array

    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", nmatch_off)); // allocate enough slots for every compiled capture
    emitter.instruction("mov rsi, 16");                                         // string arrays use pointer/length payload slots
    emitter.instruction("call __rt_array_new");                                 // allocate indexed string matches array
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", matches_array_off)); // save matches array pointer across pushes
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", group_idx_off)); // start with capture index zero
    emitter.label("__rt_preg_match_capture_group_loop_linux_x86_64");
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx_off)); // reload current capture index
    emitter.instruction(&format!("mov r8, QWORD PTR [rsp + {}]", max_group_off)); // reload highest capture index
    emitter.instruction("cmp r9, r8");                                          // have all required captures been materialized?
    emitter.instruction("jg __rt_preg_match_capture_success_linux_x86_64");     // finish after the highest populated capture
    emitter.instruction("mov r10, r9");                                         // copy capture index before scaling
    emitter.instruction("shl r10, 4");                                          // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("mov r12, QWORD PTR [rsp + {}]", regmatches_ptr_off)); // load dynamic offset-pair buffer base
    emitter.instruction("add r10, r12");                                        // compute address of this offset pair
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // load signed-64-bit capture start
    emitter.instruction("mov rcx, QWORD PTR [r10 + 8]");                        // load signed-64-bit capture end
    emitter.instruction("cmp r11, 0");                                          // detect unmatched captures before the highest populated slot
    emitter.instruction("jl __rt_preg_match_capture_empty_string_linux_x86_64"); // emit PHP's empty string for an interior unmatched capture
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", subject_cstr_off)); // reload subject C string base
    emitter.instruction("add rsi, r11");                                        // compute capture string pointer
    emitter.instruction("mov rdx, rcx");                                        // copy capture end offset before subtracting start
    emitter.instruction("sub rdx, r11");                                        // capture length = rm_eo - rm_so
    emitter.instruction("jmp __rt_preg_match_capture_push_linux_x86_64");       // append this capture string
    emitter.label("__rt_preg_match_capture_empty_string_linux_x86_64");
    emitter.instruction("xor esi, esi");                                        // empty unmatched capture has a null pointer
    emitter.instruction("xor edx, edx");                                        // empty unmatched capture has zero length
    emitter.label("__rt_preg_match_capture_push_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", matches_array_off)); // reload matches array pointer
    emitter.instruction("call __rt_array_push_str");                            // append capture string to matches array
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", matches_array_off)); // save possibly-grown matches array pointer
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx_off)); // reload capture index after helper calls
    emitter.instruction("add r9, 1");                                           // advance to next capture index
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", group_idx_off)); // save next capture index
    emitter.instruction("jmp __rt_preg_match_capture_group_loop_linux_x86_64"); // continue materializing captures


    // -- named captures: PHP writes each name immediately before its own numeric key --
    emitter.label("__rt_preg_match_capture_named_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", nmatch_off)); // size the table from the compiled capture count
    emitter.instruction("shl rdi, 1");                                          // reserve one bucket per numeric key and one per name
    emitter.instruction("mov esi, 1");                                          // string-valued hash: the shape a PHP string array literal builds
    emitter.instruction("call __rt_hash_new");                                  // allocate the ordered hash that carries both key kinds
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", matches_array_off)); // save matches hash pointer across inserts
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], 0", group_idx_off)); // start with capture index zero
    emitter.label("__rt_preg_match_capture_named_loop_linux_x86_64");
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx_off)); // reload current capture index
    emitter.instruction(&format!("mov r8, QWORD PTR [rsp + {}]", max_group_off)); // reload highest capture index
    emitter.instruction("cmp r9, r8");                                          // have all required captures been materialized?
    emitter.instruction("jg __rt_preg_match_capture_success_linux_x86_64");     // finish after the highest populated capture
    emitter.instruction("mov r10, r9");                                         // copy capture index before scaling
    emitter.instruction("shl r10, 4");                                          // scale capture index by the fixed 16-byte pair stride
    emitter.instruction(&format!("add r10, QWORD PTR [rsp + {}]", regmatches_ptr_off)); // compute address of this offset pair
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // load signed-64-bit capture start
    emitter.instruction("mov rcx, QWORD PTR [r10 + 8]");                        // load signed-64-bit capture end
    emitter.instruction("cmp r11, 0");                                          // detect unmatched captures before the highest populated slot
    emitter.instruction("jl __rt_preg_match_capture_named_empty_linux_x86_64"); // emit PHP's empty string for an interior unmatched capture
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", subject_cstr_off)); // reload subject C string base
    emitter.instruction("add rsi, r11");                                        // compute capture string pointer
    emitter.instruction("mov rdx, rcx");                                        // copy capture end offset before subtracting start
    emitter.instruction("sub rdx, r11");                                        // capture length = rm_eo - rm_so
    emitter.instruction("jmp __rt_preg_match_capture_named_value_linux_x86_64"); // insert this capture under both key kinds
    emitter.label("__rt_preg_match_capture_named_empty_linux_x86_64");
    emitter.instruction("xor esi, esi");                                        // empty unmatched capture has a null pointer
    emitter.instruction("xor edx, edx");                                        // empty unmatched capture has zero length
    emitter.label("__rt_preg_match_capture_named_value_linux_x86_64");
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rsi", value_ptr_off)); // hold the capture pointer across calls
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdx", value_len_off)); // hold the capture length across calls
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // pass the compiled pattern holding the name table
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", group_idx_off)); // ask for this capture group's declared name
    emitter.instruction(&format!("lea rdx, [rsp + {}]", name_ptr_off));         // receive the name pointer into pattern storage
    emitter.instruction(&format!("lea rcx, [rsp + {}]", name_len_off));         // receive the name length
    emitter.bl_c("elephc_pcre2_v1_group_name");                                 // resolve the name without exposing PCRE2 table layouts
    emitter.instruction("test eax, eax");                                       // did this capture group carry a declared name?
    emitter.instruction("jnz __rt_preg_match_capture_named_int_linux_x86_64");  // an unnamed group only gets its numeric key
    emitter.instruction(&format!("mov rax, QWORD PTR [rsp + {}]", name_ptr_off)); // load the resolved group name pointer
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", name_len_off)); // load the resolved group name length
    emitter.instruction("call __rt_hash_normalize_key");                        // apply PHP's numeric-string key normalization
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", key_lo_off)); // hold the normalized key across the persist
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rdx", key_hi_off)); // hold the key discriminant across the persist
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", value_ptr_off)); // reload the capture pointer to persist
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", value_len_off)); // reload the capture length to persist
    emitter.instruction("call __rt_str_persist");                               // the named entry owns its own copy of the capture bytes
    emitter.instruction("mov rcx, rax");                                        // pass the persisted capture as the entry value payload
    emitter.instruction("mov r8, rdx");                                         // pass the persisted capture length
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", key_lo_off)); // reload the normalized key
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", key_hi_off)); // reload the key discriminant
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", matches_array_off)); // reload matches hash pointer
    emitter.instruction("mov r9d, 1");                                          // runtime value tag 1 = string
    emitter.instruction("call __rt_hash_set");                                  // insert the named key ahead of its numeric twin
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", matches_array_off)); // save possibly-grown hash pointer
    emitter.label("__rt_preg_match_capture_named_int_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", value_ptr_off)); // reload the capture pointer to persist
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", value_len_off)); // reload the capture length to persist
    emitter.instruction("call __rt_str_persist");                               // the numeric entry owns its own copy of the capture bytes
    emitter.instruction("mov rcx, rax");                                        // pass the persisted capture as the entry value payload
    emitter.instruction("mov r8, rdx");                                         // pass the persisted capture length
    emitter.instruction(&format!("mov rsi, QWORD PTR [rsp + {}]", group_idx_off)); // the capture index is the numeric key
    emitter.instruction("mov rdx, -1");                                         // a key discriminant of -1 marks an integer key
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", matches_array_off)); // reload matches hash pointer
    emitter.instruction("mov r9d, 1");                                          // runtime value tag 1 = string
    emitter.instruction("call __rt_hash_set");                                  // insert this capture under its numeric key
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], rax", matches_array_off)); // save possibly-grown hash pointer
    emitter.instruction(&format!("mov r9, QWORD PTR [rsp + {}]", group_idx_off)); // reload capture index after helper calls
    emitter.instruction("add r9, 1");                                           // advance to next capture index
    emitter.instruction(&format!("mov QWORD PTR [rsp + {}], r9", group_idx_off)); // save next capture index
    emitter.instruction("jmp __rt_preg_match_capture_named_loop_linux_x86_64"); // continue materializing captures

    emitter.label("__rt_preg_match_capture_success_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // reload the pattern now every group name has been read
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", regmatches_ptr_off)); // reload dynamic capture buffer for cleanup
    emitter.bl_c("free");                                                       // free the dynamic offset-pair vector before returning matches
    emitter.instruction(&format!("mov rdx, QWORD PTR [rsp + {}]", matches_array_off)); // return matches array pointer in rdx
    emitter.instruction("mov eax, 1");                                          // report that preg_match found a match
    emitter.instruction("jmp __rt_preg_match_capture_ret_linux_x86_64");        // share helper epilogue

    emitter.label("__rt_preg_match_capture_no_match_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // reload compiled handle for the no-match cleanup path
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources
    emitter.instruction("jmp __rt_preg_match_capture_free_pairs_linux_x86_64"); // share the capture-buffer cleanup

    emitter.label("__rt_preg_match_capture_invalid_offset_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // reload compiled handle for invalid-offset cleanup
    emitter.bl_c("elephc_pcre2_v1_free");                                       // release compiled regex resources

    emitter.label("__rt_preg_match_capture_free_pairs_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", regmatches_ptr_off)); // reload dynamic capture buffer for cleanup
    emitter.bl_c("free");                                                       // free the dynamic offset-pair vector before returning an empty matches array
    emitter.instruction("jmp __rt_preg_match_capture_empty_linux_x86_64");      // allocate and return the empty matches array

    emitter.label("__rt_preg_match_capture_malloc_fail_linux_x86_64");
    emitter.instruction(&format!("mov rdi, QWORD PTR [rsp + {}]", handle_off)); // reload opaque handle after capture-buffer allocation failed
    emitter.bl_c("elephc_pcre2_v1_free");                                       // free compiled regex resources before returning no match

    emitter.label("__rt_preg_match_capture_empty_linux_x86_64");
    emitter.instruction("xor edi, edi");                                        // empty array capacity for no-match or compile-failure paths
    emitter.instruction("mov esi, 16");                                         // empty matches array still uses string slot metadata
    emitter.instruction("call __rt_array_new");                                 // allocate empty indexed string matches array
    emitter.instruction("mov rdx, rax");                                        // return empty matches array pointer in rdx
    emitter.instruction("xor eax, eax");                                        // report no match

    emitter.label("__rt_preg_match_capture_ret_linux_x86_64");
    emitter.instruction(&format!("add rsp, {}", stack_size));                   // release capture helper local storage
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return match flag in rax and matches array in rdx
}
