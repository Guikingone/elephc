//! Purpose:
//! Emits the `__rt_mixed_array_get` runtime helper for `$mixed[$key]` access.
//! Routes boxed strings and JSON-style values to string, indexed-array, hash,
//! or object lookup paths.
//!
//! Called from:
//! - `crate::codegen_support::runtime::objects::emit_mixed_array_get()`.
//!
//! Key details:
//! - The key tuple matches `emit_normalized_hash_key`: int keys use `key_hi = -1`.
//! - Callers pass an explicit warning flag so normal reads diagnose missing/null
//!   offsets while `isset()`/`??` keep the same helper quiet.
//! - A container payload that is null or the in-band null-container sentinel
//!   (`NULL_SENTINEL`, materialized by a missed read forwarded through a ternary merge)
//!   is treated as an absent container and also returns `Mixed(null)` (issue #585).
//! - Every successful return is an owned `Mixed*`; borrowed array/hash slots are retained first.

use crate::codegen_support::abi;
use crate::codegen_support::callable_invoker_args::{
    ARRAY_GLOBAL_REF_CELL_TAG, ARRAY_LOCAL_REF_CELL_TAG, INVOKER_ARG_REF_CELL_TAG,
};
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::sentinels::emit_branch_if_null_container;

/// Dispatches to the target-specific `__rt_mixed_array_get` emitter.
///
/// Checks `emitter.target.arch` and routes to either `emit_mixed_array_get_x86_64`
/// (SysV ABI) or `emit_mixed_array_get_aarch64` (AAPCS64). The helper is emitted
/// once into the runtime object and is called by generated code for `$mixed[$key]`
/// access on a boxed `Mixed` value.
pub fn emit_mixed_array_get(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_array_get_x86_64(emitter);
        return;
    }
    emit_mixed_array_get_aarch64(emitter);
}

/// Emits `__rt_mixed_array_get` for ARM64 (AAPCS64 ABI).
///
/// Inputs arrive in `x0` = mixed_ptr, `x1` = key_lo, `x2` = key_hi,
/// `x3` = nonzero when missing/null-offset warnings are enabled.
/// Returns an owned pointer to a boxed `Mixed` cell in `x0`.
///
/// The function dispatches on the mixed value's tag:
/// - Tag 1 → string-offset path
/// - Tag 4 → indexed array path
/// - Tag 5 → associative array path
/// - Tag 6 → stdClass object path
/// - All others → null (boxed `Mixed(null)`)
///
/// For indexed arrays the key must be integer (`key_hi == -1` sentinel); string keys
/// return null. For objects only `stdClass` with a string key is supported; int keys
/// and non-stdClass objects return null. Missing keys return null. All paths that
/// produce a value box it through `__rt_mixed_from_value` except when storage already
/// holds a boxed `Mixed` pointer (tag 7), which is retained before returning.
fn emit_mixed_array_get_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_get ---");
    emitter.label_global("__rt_mixed_array_get");

    // Stack:
    //   [sp, #0]  = mixed_ptr
    //   [sp, #8]  = key_lo
    //   [sp, #16] = key_hi
    //   [sp, #24] = saved x29
    //   [sp, #32] = saved x30
    //   [sp, #40] = warn_on_missing
    emitter.instruction("sub sp, sp, #48");                                     // reserve frame: 3 inputs + saved fp/lr (16-byte aligned)
    emitter.instruction("stp x29, x30, [sp, #24]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #24");                                    // set new frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save mixed_ptr
    emitter.instruction("str x1, [sp, #8]");                                    // save key_lo
    emitter.instruction("str x2, [sp, #16]");                                   // save key_hi
    emitter.instruction("str x3, [sp, #40]");                                   // save whether this read should emit PHP offset warnings

    emitter.instruction("cbz x0, __rt_mixed_array_get_null_container");         // null Mixed pointers behave as PHP null receivers
    emitter.instruction("ldr x9, [x0]");                                        // load tag from mixed[0]
    emitter.instruction("cmp x9, #1");                                          // tag = 1 (string)?
    emitter.instruction("b.eq __rt_mixed_array_get_string");                    // read string offsets from the boxed pointer/length pair
    emitter.instruction("cmp x9, #4");                                          // tag = 4 (indexed array)?
    emitter.instruction("b.eq __rt_mixed_array_get_indexed");                   // branch on the current JSON decoder condition
    emitter.instruction("cmp x9, #5");                                          // tag = 5 (associative array)?
    emitter.instruction("b.eq __rt_mixed_array_get_assoc");                     // branch on the current JSON decoder condition
    emitter.instruction("cmp x9, #6");                                          // tag = 6 (object)?
    emitter.instruction("b.eq __rt_mixed_array_get_object");                    // branch on the current JSON decoder condition
    emitter.instruction("cmp x9, #8");                                          // tag = 8 (canonical PHP null)?
    emitter.instruction("b.eq __rt_mixed_array_get_null_container");            // null receivers warn only for ordinary reads
    emitter.instruction("b __rt_mixed_array_get_null");                         // any other payload → null

    // String: integer keys select one byte; negative offsets are relative to the end.
    emitter.label("__rt_mixed_array_get_string");
    emitter.instruction("ldr x10, [sp, #16]");                                  // load the normalized key high word
    emitter.instruction("cmn x10, #1");                                         // integer keys carry the -1 high-word sentinel
    emitter.instruction("b.ne __rt_mixed_array_get_null");                      // non-integer string offsets are not readable on this fallback path
    emitter.instruction("ldr x10, [x0, #8]");                                   // load the boxed string pointer
    emitter.instruction("ldr x11, [x0, #16]");                                  // load the boxed string byte length
    emitter.instruction("ldr x12, [sp, #8]");                                   // load the requested integer offset
    emitter.instruction("cmp x12, #0");                                         // test whether the offset is relative to the end
    emitter.instruction("b.ge __rt_mixed_array_get_string_non_negative");       // preserve non-negative offsets
    emitter.instruction("add x12, x11, x12");                                   // convert a negative offset to length plus offset
    emitter.instruction("cmp x12, #0");                                         // reject offsets that still precede the string
    emitter.instruction("b.lt __rt_mixed_array_get_string_empty");              // out-of-bounds string reads return an empty string
    emitter.label("__rt_mixed_array_get_string_non_negative");
    emitter.instruction("cmp x12, x11");                                        // compare the normalized offset with the byte length
    emitter.instruction("b.ge __rt_mixed_array_get_string_empty");              // offsets at or beyond the end return an empty string
    emitter.instruction("add x1, x10, x12");                                    // point at the selected byte
    emitter.instruction("mov x2, #1");                                          // an in-bounds string offset has one byte
    emitter.instruction("b __rt_mixed_array_get_string_box");                   // box a detached one-byte string result
    emitter.label("__rt_mixed_array_get_string_empty");
    emitter.instruction("mov x1, x10");                                         // preserve a valid source pointer for the empty string
    emitter.instruction("mov x2, #0");                                          // out-of-bounds string offsets stringify to empty
    emitter.label("__rt_mixed_array_get_string_box");
    emitter.instruction("mov x0, #1");                                          // runtime tag 1 identifies a string payload
    emitter.instruction("bl __rt_mixed_from_value");                            // persist and box the selected byte for the caller
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the owned boxed string in x0

    // Indexed array: integer key only. key_hi == -1 marks int keys.
    emitter.label("__rt_mixed_array_get_indexed");
    emitter.instruction("ldr x10, [x0, #8]");                                   // x10 = array pointer
    // treat a null or in-band null-container sentinel payload as an absent container (issue #585)
    emit_branch_if_null_container(
        emitter,
        "x10",
        "x9",
        "__rt_mixed_array_get_null_container",
    );
    emitter.instruction("ldr x11, [sp, #16]");                                  // load key_hi
    emitter.instruction("cmn x11, #1");                                         // compare with -1 (int-key sentinel)
    emitter.instruction("b.ne __rt_mixed_array_get_indexed_missing_string");    // missing string keys use PHP's string-key warning
    emitter.instruction("ldr x12, [sp, #8]");                                   // x12 = key_lo (int index)
    emitter.instruction("ldr x9, [x10]");                                       // x9 = array length (header offset 0)
    emitter.instruction("cmp x12, #0");                                         // negative index → null
    emitter.instruction("b.lt __rt_mixed_array_get_indexed_missing");           // warn and return null for a negative indexed-array key
    emitter.instruction("cmp x12, x9");                                         // index >= length → null
    emitter.instruction("b.ge __rt_mixed_array_get_indexed_missing");           // warn and return null for an out-of-bounds indexed-array key
    emitter.instruction("ldr x13, [x10, #-8]");                                 // load packed indexed-array kind metadata
    emitter.instruction("ubfx x13, x13, #8, #7");                               // extract the runtime element value_type tag
    emitter.instruction("add x10, x10, #24");                                   // skip the 24-byte array header to reach the contiguous payload
    emitter.instruction("cmp x13, #7");                                         // are indexed slots already boxed Mixed pointers?
    emitter.instruction("b.eq __rt_mixed_array_get_indexed_boxed");             // boxed slots must be retained before returning
    emitter.instruction("cmp x13, #1");                                         // do indexed slots contain string pointer/length pairs?
    emitter.instruction("b.eq __rt_mixed_array_get_indexed_string");            // string slots need a 16-byte load before boxing
    emitter.instruction("cmp x13, #8");                                         // do indexed slots represent null payloads?
    emitter.instruction("b.eq __rt_mixed_array_get_indexed_null");              // null slots have no payload to read
    emitter.instruction("ldr x1, [x10, x12, lsl #3]");                          // load scalar or pointer payload from the typed indexed slot
    emitter.instruction("mov x2, #0");                                          // typed indexed slots use one payload word except strings
    emitter.instruction("mov x0, x13");                                         // x0 = runtime value_type tag for the boxed result
    emitter.instruction("bl __rt_mixed_from_value");                            // box the typed indexed-array element into a Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0
    emitter.label("__rt_mixed_array_get_indexed_boxed");
    emitter.instruction("ldr x0, [x10, x12, lsl #3]");                          // load the boxed Mixed pointer from the indexed slot
    emitter.instruction("cbz x0, __rt_mixed_array_get_indexed_missing");        // zero-filled gaps are undefined keys, not present null values
    emitter.instruction("ldr x9, [x0]");                                        // inspect the stored Mixed tag before returning the cell
    emitter.instruction(&format!("cmp x9, #{}", INVOKER_ARG_REF_CELL_TAG));     // is this a borrowed invoker reference marker?
    emitter.instruction("b.eq __rt_mixed_array_get_ref_marker");                // expose the referenced PHP value instead of the internal marker
    emitter.instruction(&format!("cmp x9, #{}", ARRAY_GLOBAL_REF_CELL_TAG));    // is this an owning global-reference marker?
    emitter.instruction("b.eq __rt_mixed_array_get_ref_marker");                // all marker kinds share the referenced-cell payload layout
    emitter.instruction(&format!("cmp x9, #{}", ARRAY_LOCAL_REF_CELL_TAG));     // is this an owning local-reference marker?
    emitter.instruction("b.eq __rt_mixed_array_get_ref_marker");                // expose the retained local cell's current PHP value
    emitter.instruction("bl __rt_incref");                                      // retain the stored Mixed cell so the caller owns the returned result
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0

    emitter.label("__rt_mixed_array_get_ref_marker");
    emitter.instruction("ldr x10, [x0, #8]");                                   // load the referenced cell pointer carried by the marker
    emitter.instruction("ldr x9, [x0, #16]");                                   // load the referenced value's runtime tag
    emitter.instruction("ldr x1, [x10]");                                       // load the referenced low payload word
    emitter.instruction("mov x2, #0");                                          // non-string referenced values have no high payload word
    emitter.instruction("cmp x9, #7");                                          // does the reference cell store a boxed Mixed handle?
    emitter.instruction("b.eq __rt_mixed_array_get_ref_marker_mixed");          // clone the nested Mixed cell instead of boxing its pointer
    emitter.instruction("cmp x9, #1");                                          // does the reference cell store a string pair?
    emitter.instruction("b.ne __rt_mixed_array_get_ref_marker_box");            // scalar and heap values can be boxed immediately
    emitter.instruction("ldr x2, [x10, #8]");                                   // load the referenced string length
    emitter.label("__rt_mixed_array_get_ref_marker_box");
    emitter.instruction("mov x0, x9");                                          // pass the referenced runtime tag to the boxing helper
    emitter.instruction("bl __rt_mixed_from_value");                            // return an ordinary owned PHP-visible Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the dereferenced Mixed value in x0
    emitter.label("__rt_mixed_array_get_ref_marker_mixed");
    emitter.instruction("mov x0, x1");                                          // pass the referenced boxed Mixed handle to the clone helper
    emitter.instruction("bl __rt_mixed_clone");                                 // detach the referenced value for the caller
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the cloned referenced Mixed cell in x0
    emitter.label("__rt_mixed_array_get_indexed_string");
    emitter.instruction("lsl x12, x12, #4");                                    // convert the element index to a 16-byte string slot offset
    emitter.instruction("add x10, x10, x12");                                   // x10 = address of the selected string slot
    emitter.instruction("ldr x1, [x10]");                                       // load string pointer from the selected slot
    emitter.instruction("ldr x2, [x10, #8]");                                   // load string length from the selected slot
    emitter.instruction("mov x0, #1");                                          // x0 = string runtime value_type tag
    emitter.instruction("bl __rt_mixed_from_value");                            // box the string indexed-array element into a Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0
    emitter.label("__rt_mixed_array_get_indexed_null");
    emitter.instruction("mov x0, #8");                                          // x0 = null runtime value_type tag
    emitter.instruction("mov x1, #0");                                          // value_lo = 0 for null
    emitter.instruction("mov x2, #0");                                          // value_hi = 0 for null
    emitter.instruction("bl __rt_mixed_from_value");                            // box the null indexed-array element into a Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0
    emitter.label("__rt_mixed_array_get_indexed_missing");
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload whether ordinary read warnings are enabled
    emitter.instruction("cbz x9, __rt_mixed_array_get_null");                   // `isset()`/`??` suppress undefined-key warnings
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the missing integer key for the PHP warning
    emitter.instruction("bl __rt_warn_undefined_array_key_int");                // emit or suppress the undefined-array-key warning
    emitter.instruction("b __rt_mixed_array_get_null");                         // return boxed Mixed(null) after the warning
    emitter.label("__rt_mixed_array_get_indexed_missing_string");
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload whether ordinary read warnings are enabled
    emitter.instruction("cbz x9, __rt_mixed_array_get_null");                   // `isset()`/`??` suppress undefined-key warnings
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload the missing string key pointer
    emitter.instruction("ldr x2, [sp, #16]");                                   // reload the missing string key length
    emitter.instruction("bl __rt_warn_undefined_array_key_str");                // emit the PHP warning for a missing string key
    emitter.instruction("b __rt_mixed_array_get_null");                         // return boxed Mixed(null) after the warning

    // Associative array: hash_get with normalized key.
    emitter.label("__rt_mixed_array_get_assoc");
    emitter.instruction("ldr x10, [x0, #8]");                                   // x10 = hash pointer
    // treat a null or in-band null-container sentinel payload as an absent container (issue #585)
    emit_branch_if_null_container(
        emitter,
        "x10",
        "x9",
        "__rt_mixed_array_get_null_container",
    );
    emitter.instruction("mov x0, x10");                                         // x0 = hash pointer for hash_get
    emitter.instruction("ldr x1, [sp, #8]");                                    // x1 = key_lo
    emitter.instruction("ldr x2, [sp, #16]");                                   // x2 = key_hi
    emitter.instruction("bl __rt_hash_get");                                    // x0=found, x1=value_lo, x2=value_hi, x3=value_tag
    emitter.instruction("cbz x0, __rt_mixed_array_get_assoc_missing");          // diagnose an absent hash key for ordinary reads
    // For value_tag == 7 the entry already holds a boxed Mixed pointer
    // (json_decode and stdClass populate hashes this way). Anything else
    // (typed string/int/array entries from non-Mixed assoc arrays passing
    // through a Mixed receiver) needs to be re-boxed via mixed_from_value
    // so callers always see a uniform Mixed cell.
    emitter.instruction("cmp x3, #7");                                          // is the hash entry already a boxed Mixed?
    emitter.instruction("b.ne __rt_mixed_array_get_assoc_box");                 // no → box (lo, hi, tag) into a fresh Mixed cell
    emitter.instruction("mov x0, x1");                                          // yes → move the stored Mixed cell into the return register
    emitter.instruction("ldr x9, [x0]");                                        // inspect the stored Mixed tag before returning the cell
    emitter.instruction(&format!("cmp x9, #{}", INVOKER_ARG_REF_CELL_TAG));     // is this a borrowed invoker reference marker?
    emitter.instruction("b.eq __rt_mixed_array_get_ref_marker");                // expose the referenced PHP value instead of the internal marker
    emitter.instruction(&format!("cmp x9, #{}", ARRAY_GLOBAL_REF_CELL_TAG));    // is this an owning global-reference marker?
    emitter.instruction("b.eq __rt_mixed_array_get_ref_marker");                // all marker kinds share the referenced-cell payload layout
    emitter.instruction(&format!("cmp x9, #{}", ARRAY_LOCAL_REF_CELL_TAG));     // is this an owning local-reference marker?
    emitter.instruction("b.eq __rt_mixed_array_get_ref_marker");                // expose the retained local cell's current PHP value
    emitter.instruction("bl __rt_incref");                                      // retain the stored Mixed cell so the caller owns the returned result
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0

    emitter.label("__rt_mixed_array_get_assoc_missing");
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload whether ordinary read warnings are enabled
    emitter.instruction("cbz x9, __rt_mixed_array_get_null");                   // `isset()`/`??` suppress undefined-key warnings
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload the normalized key high word
    emitter.instruction("cmn x9, #1");                                          // does the missing key use the integer-key sentinel?
    emitter.instruction("b.eq __rt_mixed_array_get_assoc_missing_int");         // integer keys use the decimal warning formatter
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload the missing string key pointer
    emitter.instruction("ldr x2, [sp, #16]");                                   // reload the missing string key length
    emitter.instruction("bl __rt_warn_undefined_array_key_str");                // emit the PHP warning for a missing string key
    emitter.instruction("b __rt_mixed_array_get_null");                         // return boxed Mixed(null) after the warning
    emitter.label("__rt_mixed_array_get_assoc_missing_int");
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the missing integer key
    emitter.instruction("bl __rt_warn_undefined_array_key_int");                // emit the PHP warning for a missing integer key
    emitter.instruction("b __rt_mixed_array_get_null");                         // return boxed Mixed(null) after the warning
    emitter.label("__rt_mixed_array_get_assoc_box");
    // mixed_from_value(tag, lo, hi). Move (x1, x2, x3) into (x1, x2, x0).
    emitter.instruction("mov x0, x3");                                          // x0 = value_tag (mixed_from_value first arg)
    // x1 already holds value_lo; x2 already holds value_hi.
    emitter.instruction("bl __rt_mixed_from_value");                            // box the typed entry into a Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0

    // Object: SPL ArrayAccess containers or stdClass with string key.
    emitter.label("__rt_mixed_array_get_object");
    emitter.instruction("ldr x10, [x0, #8]");                                   // x10 = obj pointer
    // treat a null or in-band null-container sentinel payload as an absent container (issue #585)
    emit_branch_if_null_container(
        emitter,
        "x10",
        "x9",
        "__rt_mixed_array_get_null_container",
    );
    emitter.instruction("ldr x11, [x10]");                                      // x11 = class_id
    abi::emit_symbol_address(emitter, "x12", "_spl_fixed_array_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // x12 = compile-time SplFixedArray class_id
    emitter.instruction("cmp x11, x12");                                        // is the receiver a SplFixedArray instance?
    emitter.instruction("b.eq __rt_mixed_array_get_spl_fixed");                 // dispatch mixed object indexing to SplFixedArray::offsetGet
    abi::emit_symbol_address(emitter, "x12", "_spl_dll_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // x12 = compile-time SplDoublyLinkedList class_id
    emitter.instruction("cmp x11, x12");                                        // is the receiver a SplDoublyLinkedList instance?
    emitter.instruction("b.eq __rt_mixed_array_get_spl_dll");                   // dispatch mixed object indexing to the list ArrayAccess helper
    abi::emit_symbol_address(emitter, "x12", "_spl_stack_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // x12 = compile-time SplStack class_id
    emitter.instruction("cmp x11, x12");                                        // is the receiver a SplStack instance?
    emitter.instruction("b.eq __rt_mixed_array_get_spl_dll");                   // SplStack shares the list ArrayAccess helper
    abi::emit_symbol_address(emitter, "x12", "_spl_queue_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // x12 = compile-time SplQueue class id
    emitter.instruction("cmp x11, x12");                                        // is the receiver a SplQueue instance?
    emitter.instruction("b.eq __rt_mixed_array_get_spl_dll");                   // SplQueue shares the list ArrayAccess helper
    abi::emit_symbol_address(emitter, "x12", "_stdclass_class_id");
    emitter.instruction("ldr x12, [x12]");                                      // x12 = compile-time stdClass class_id
    emitter.instruction("cmp x11, x12");                                        // is the receiver a stdClass instance?
    emitter.instruction("b.ne __rt_mixed_array_get_null");                      // unrelated class → null
    emitter.instruction("ldr x11, [sp, #16]");                                  // load key_hi
    emitter.instruction("cmn x11, #1");                                         // compare with -1 (int-key sentinel)
    emitter.instruction("b.eq __rt_mixed_array_get_null");                      // int keys on objects → null
    emitter.instruction("mov x0, x10");                                         // x0 = stdClass pointer
    emitter.instruction("ldr x1, [sp, #8]");                                    // x1 = key_lo (str ptr)
    emitter.instruction("ldr x2, [sp, #16]");                                   // x2 = key_hi (str len)
    emitter.instruction("bl __rt_stdclass_get");                                // delegate to the dynamic-property reader
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0
    emitter.label("__rt_mixed_array_get_spl_fixed");
    emitter.instruction("str x10, [sp, #0]");                                   // save unboxed SplFixedArray receiver while boxing the key
    emitter.instruction("ldr x11, [sp, #16]");                                  // reload normalized key high word
    emitter.instruction("cmn x11, #1");                                         // does key_hi carry the integer-key sentinel?
    emitter.instruction("b.eq __rt_mixed_array_get_spl_fixed_int_key");         // integer keys box as Mixed int
    emitter.instruction("mov x0, #1");                                          // tag = string for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key string pointer
    emitter.instruction("ldr x2, [sp, #16]");                                   // key string length
    emitter.instruction("b __rt_mixed_array_get_spl_fixed_box_key");            // share the offsetGet call after key boxing
    emitter.label("__rt_mixed_array_get_spl_fixed_int_key");
    emitter.instruction("mov x0, #0");                                          // tag = int for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key integer payload
    emitter.instruction("mov x2, #0");                                          // integer keys have no high payload
    emitter.label("__rt_mixed_array_get_spl_fixed_box_key");
    emitter.instruction("bl __rt_mixed_from_value");                            // allocate the boxed ArrayAccess offset
    emitter.instruction("mov x1, x0");                                          // pass boxed offset as argument 1
    emitter.instruction("ldr x0, [sp, #0]");                                    // pass unboxed SplFixedArray receiver as argument 0
    emitter.instruction("bl __rt_spl_fixed_offset_get");                        // read through SplFixedArray::offsetGet
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0
    emitter.label("__rt_mixed_array_get_spl_dll");
    emitter.instruction("str x10, [sp, #0]");                                   // save unboxed SPL list receiver while boxing the key
    emitter.instruction("ldr x11, [sp, #16]");                                  // reload normalized key high word
    emitter.instruction("cmn x11, #1");                                         // does key_hi carry the integer-key sentinel?
    emitter.instruction("b.eq __rt_mixed_array_get_spl_dll_int_key");           // integer keys box as Mixed int
    emitter.instruction("mov x0, #1");                                          // tag = string for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key string pointer
    emitter.instruction("ldr x2, [sp, #16]");                                   // key string length
    emitter.instruction("b __rt_mixed_array_get_spl_dll_box_key");              // share the offsetGet call after key boxing
    emitter.label("__rt_mixed_array_get_spl_dll_int_key");
    emitter.instruction("mov x0, #0");                                          // tag = int for mixed_from_value
    emitter.instruction("ldr x1, [sp, #8]");                                    // key integer payload
    emitter.instruction("mov x2, #0");                                          // integer keys have no high payload
    emitter.label("__rt_mixed_array_get_spl_dll_box_key");
    emitter.instruction("bl __rt_mixed_from_value");                            // allocate the boxed ArrayAccess offset
    emitter.instruction("mov x1, x0");                                          // pass boxed offset as argument 1
    emitter.instruction("ldr x0, [sp, #0]");                                    // pass unboxed SPL list receiver as argument 0
    emitter.instruction("bl __rt_spl_dll_offset_get");                          // read through the shared SPL list offsetGet helper
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0

    emitter.label("__rt_mixed_array_get_null_container");
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload whether ordinary read warnings are enabled
    emitter.instruction("cbz x9, __rt_mixed_array_get_null");                   // quiet read contexts suppress the null-offset warning
    emitter.instruction("bl __rt_warn_array_offset_on_null");                   // emit PHP's warning for indexing a null receiver
    emitter.label("__rt_mixed_array_get_null");
    emitter.instruction("mov x0, #8");                                          // tag = 8 (null)
    emitter.instruction("mov x1, #0");                                          // value_lo = 0
    emitter.instruction("mov x2, #0");                                          // value_hi = 0
    emitter.instruction("bl __rt_mixed_from_value");                            // box null into a fresh Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #24]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the local frame
    emitter.instruction("ret");                                                 // return Mixed* in x0
}

/// Emits `__rt_mixed_array_get` for x86_64 (SysV ABI).
///
/// Inputs arrive in `rdi` = mixed_ptr, `rsi` = key_lo, `rdx` = key_hi,
/// `rcx` = nonzero when missing/null-offset warnings are enabled.
/// Returns an owned pointer to a boxed `Mixed` cell in `rax`.
///
/// Same dispatch and return semantics as `emit_mixed_array_get_aarch64`:
/// - Tag 4 → indexed array, tag 5 → associative array, tag 6 → stdClass object
/// - Integer keys on indexed arrays required (`key_hi == -1`); string keys return null
/// - Objects: only `stdClass` with string key supported; int keys return null
/// - Missing keys and unsupported payloads return boxed `Mixed(null)`
/// - Slots already holding a boxed `Mixed` (tag 7) are retained before return;
///   all other values are boxed through `__rt_mixed_from_value`
fn emit_mixed_array_get_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_get ---");
    emitter.label_global("__rt_mixed_array_get");

    // Inputs (SysV): rdi = mixed_ptr, rsi = key_lo, rdx = key_hi, rcx = warn_on_missing.
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 32");                                         // reserve slots for the 3 saved inputs (16-byte aligned)
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save mixed_ptr
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save key_lo
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save key_hi
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // save whether this read should emit PHP offset warnings

    emitter.instruction("test rdi, rdi");                                       // null Mixed → null
    emitter.instruction("je __rt_mixed_array_get_null_container");              // null Mixed pointers behave as PHP null receivers
    emitter.instruction("mov r10, QWORD PTR [rdi]");                            // load tag from mixed[0]
    emitter.instruction("cmp r10, 1");                                          // tag = 1 (string)?
    emitter.instruction("je __rt_mixed_array_get_string");                      // read string offsets from the boxed pointer/length pair
    emitter.instruction("cmp r10, 4");                                          // tag = 4 (indexed array)?
    emitter.instruction("je __rt_mixed_array_get_indexed");                     // branch on the current JSON decoder condition
    emitter.instruction("cmp r10, 5");                                          // tag = 5 (associative array)?
    emitter.instruction("je __rt_mixed_array_get_assoc");                       // branch on the current JSON decoder condition
    emitter.instruction("cmp r10, 6");                                          // tag = 6 (object)?
    emitter.instruction("je __rt_mixed_array_get_object");                      // branch on the current JSON decoder condition
    emitter.instruction("cmp r10, 8");                                          // tag = 8 (canonical PHP null)?
    emitter.instruction("je __rt_mixed_array_get_null_container");              // null receivers warn only for ordinary reads
    emitter.instruction("jmp __rt_mixed_array_get_null");                       // any other payload → null

    emitter.label("__rt_mixed_array_get_string");
    emitter.instruction("cmp QWORD PTR [rbp - 24], -1");                        // integer keys carry the -1 high-word sentinel
    emitter.instruction("jne __rt_mixed_array_get_null");                       // non-integer string offsets are not readable on this fallback path
    emitter.instruction("mov r10, QWORD PTR [rdi + 8]");                        // load the boxed string pointer
    emitter.instruction("mov r11, QWORD PTR [rdi + 16]");                       // load the boxed string byte length
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                        // load the requested integer offset
    emitter.instruction("cmp r8, 0");                                           // test whether the offset is relative to the end
    emitter.instruction("jge __rt_mixed_array_get_string_non_negative");        // preserve non-negative offsets
    emitter.instruction("add r8, r11");                                         // convert a negative offset to length plus offset
    emitter.instruction("cmp r8, 0");                                           // reject offsets that still precede the string
    emitter.instruction("jl __rt_mixed_array_get_string_empty");                // out-of-bounds string reads return an empty string
    emitter.label("__rt_mixed_array_get_string_non_negative");
    emitter.instruction("cmp r8, r11");                                         // compare the normalized offset with the byte length
    emitter.instruction("jge __rt_mixed_array_get_string_empty");               // offsets at or beyond the end return an empty string
    emitter.instruction("add r10, r8");                                         // point at the selected byte
    emitter.instruction("mov rsi, 1");                                          // an in-bounds string offset has one byte
    emitter.instruction("jmp __rt_mixed_array_get_string_box");                 // box a detached one-byte string result
    emitter.label("__rt_mixed_array_get_string_empty");
    emitter.instruction("xor esi, esi");                                        // out-of-bounds string offsets stringify to empty
    emitter.label("__rt_mixed_array_get_string_box");
    emitter.instruction("mov rax, 1");                                          // runtime tag 1 identifies a string payload
    emitter.instruction("mov rdi, r10");                                        // pass the selected byte pointer to the boxing helper
    emitter.instruction("call __rt_mixed_from_value");                          // persist and box the selected byte for the caller
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return the owned boxed string in rax

    emitter.label("__rt_mixed_array_get_indexed");
    emitter.instruction("mov r10, QWORD PTR [rdi + 8]");                        // r10 = array pointer
    emit_branch_if_null_container(
        emitter,
        "r10",
        "r11",
        "__rt_mixed_array_get_null_container",
    );
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // load key_hi
    emitter.instruction("cmp r11, -1");                                         // int-key sentinel?
    emitter.instruction("jne __rt_mixed_array_get_indexed_missing_string");     // missing string keys use PHP's string-key warning
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                        // r8 = key_lo (int index)
    emitter.instruction("mov r9, QWORD PTR [r10]");                             // r9 = array length
    emitter.instruction("cmp r8, 0");                                           // negative index → null
    emitter.instruction("jl __rt_mixed_array_get_indexed_missing");             // warn and return null for a negative indexed-array key
    emitter.instruction("cmp r8, r9");                                          // index >= length → null
    emitter.instruction("jge __rt_mixed_array_get_indexed_missing");            // warn and return null for an out-of-bounds indexed-array key
    emitter.instruction("mov r9, QWORD PTR [r10 - 8]");                         // load packed indexed-array kind metadata
    emitter.instruction("shr r9, 8");                                           // shift the runtime element value_type tag into the low bits
    emitter.instruction("and r9, 0x7f");                                        // remove the persistent COW flag from the extracted tag
    emitter.instruction("lea r10, [r10 + 24]");                                 // skip the 24-byte array header to reach the contiguous payload
    emitter.instruction("cmp r9, 7");                                           // are indexed slots already boxed Mixed pointers?
    emitter.instruction("je __rt_mixed_array_get_indexed_boxed");               // boxed slots must be retained before returning
    emitter.instruction("cmp r9, 1");                                           // do indexed slots contain string pointer/length pairs?
    emitter.instruction("je __rt_mixed_array_get_indexed_string");              // string slots need a 16-byte load before boxing
    emitter.instruction("cmp r9, 8");                                           // do indexed slots represent null payloads?
    emitter.instruction("je __rt_mixed_array_get_indexed_null");                // null slots have no payload to read
    emitter.instruction("mov rax, r9");                                         // rax = runtime value_type tag for mixed_from_value
    emitter.instruction("mov rdi, QWORD PTR [r10 + r8 * 8]");                   // rdi = scalar or pointer payload from the typed indexed slot
    emitter.instruction("xor esi, esi");                                        // typed indexed slots use one payload word except strings
    emitter.instruction("call __rt_mixed_from_value");                          // box the typed indexed-array element into a Mixed cell
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax
    emitter.label("__rt_mixed_array_get_indexed_boxed");
    emitter.instruction("mov rax, QWORD PTR [r10 + r8 * 8]");                   // load the boxed Mixed pointer from the indexed slot
    emitter.instruction("test rax, rax");                                       // empty slot → null
    emitter.instruction("je __rt_mixed_array_get_indexed_missing");             // zero-filled gaps are undefined keys, not present null values
    emitter.instruction("mov r11, QWORD PTR [rax]");                            // inspect the stored Mixed tag before returning the cell
    emitter.instruction(&format!("cmp r11, {}", INVOKER_ARG_REF_CELL_TAG));     // is this a borrowed invoker reference marker?
    emitter.instruction("je __rt_mixed_array_get_ref_marker");                  // expose the referenced PHP value instead of the internal marker
    emitter.instruction(&format!("cmp r11, {}", ARRAY_GLOBAL_REF_CELL_TAG));    // is this an owning global-reference marker?
    emitter.instruction("je __rt_mixed_array_get_ref_marker");                  // all marker kinds share the referenced-cell payload layout
    emitter.instruction(&format!("cmp r11, {}", ARRAY_LOCAL_REF_CELL_TAG));     // is this an owning local-reference marker?
    emitter.instruction("je __rt_mixed_array_get_ref_marker");                  // expose the retained local cell's current PHP value
    abi::emit_push_reg(emitter, "rax");
    emitter.instruction("call __rt_incref");                                    // retain the stored Mixed cell so the caller owns the returned result
    abi::emit_pop_reg(emitter, "rax");
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax

    emitter.label("__rt_mixed_array_get_ref_marker");
    emitter.instruction("mov r10, QWORD PTR [rax + 8]");                        // load the referenced cell pointer carried by the marker
    emitter.instruction("mov r11, QWORD PTR [rax + 16]");                       // load the referenced value's runtime tag
    emitter.instruction("mov rdi, QWORD PTR [r10]");                            // load the referenced low payload word
    emitter.instruction("xor esi, esi");                                        // non-string referenced values have no high payload word
    emitter.instruction("cmp r11, 7");                                          // does the reference cell store a boxed Mixed handle?
    emitter.instruction("je __rt_mixed_array_get_ref_marker_mixed");            // clone the nested Mixed cell instead of boxing its pointer
    emitter.instruction("cmp r11, 1");                                          // does the reference cell store a string pair?
    emitter.instruction("jne __rt_mixed_array_get_ref_marker_box");             // scalar and heap values can be boxed immediately
    emitter.instruction("mov rsi, QWORD PTR [r10 + 8]");                        // load the referenced string length
    emitter.label("__rt_mixed_array_get_ref_marker_box");
    emitter.instruction("mov rax, r11");                                        // pass the referenced runtime tag to the boxing helper
    emitter.instruction("call __rt_mixed_from_value");                          // return an ordinary owned PHP-visible Mixed cell
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return the dereferenced Mixed value in rax
    emitter.label("__rt_mixed_array_get_ref_marker_mixed");
    emitter.instruction("mov rax, rdi");                                        // pass the referenced boxed Mixed handle to the clone helper
    emitter.instruction("call __rt_mixed_clone");                               // detach the referenced value for the caller
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return the cloned referenced Mixed cell in rax
    emitter.label("__rt_mixed_array_get_indexed_string");
    emitter.instruction("shl r8, 4");                                           // convert the element index to a 16-byte string slot offset
    emitter.instruction("add r10, r8");                                         // r10 = address of the selected string slot
    emitter.instruction("mov rax, 1");                                          // rax = string runtime value_type tag
    emitter.instruction("mov rdi, QWORD PTR [r10]");                            // rdi = selected string pointer
    emitter.instruction("mov rsi, QWORD PTR [r10 + 8]");                        // rsi = selected string length
    emitter.instruction("call __rt_mixed_from_value");                          // box the string indexed-array element into a Mixed cell
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax
    emitter.label("__rt_mixed_array_get_indexed_null");
    emitter.instruction("mov rax, 8");                                          // rax = null runtime value_type tag
    emitter.instruction("mov rdi, 0");                                          // value_lo = 0 for null
    emitter.instruction("mov rsi, 0");                                          // value_hi = 0 for null
    emitter.instruction("call __rt_mixed_from_value");                          // box the null indexed-array element into a Mixed cell
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax
    emitter.label("__rt_mixed_array_get_indexed_missing");
    emitter.instruction("cmp QWORD PTR [rbp - 32], 0");                         // are ordinary read warnings enabled?
    emitter.instruction("je __rt_mixed_array_get_null");                        // `isset()`/`??` suppress undefined-key warnings
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the missing integer key for the PHP warning
    emitter.instruction("call __rt_warn_undefined_array_key_int");              // emit or suppress the undefined-array-key warning
    emitter.instruction("jmp __rt_mixed_array_get_null");                       // return boxed Mixed(null) after the warning
    emitter.label("__rt_mixed_array_get_indexed_missing_string");
    emitter.instruction("cmp QWORD PTR [rbp - 32], 0");                         // are ordinary read warnings enabled?
    emitter.instruction("je __rt_mixed_array_get_null");                        // `isset()`/`??` suppress undefined-key warnings
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the missing string key pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // reload the missing string key length
    emitter.instruction("call __rt_warn_undefined_array_key_str");              // emit the PHP warning for a missing string key
    emitter.instruction("jmp __rt_mixed_array_get_null");                       // return boxed Mixed(null) after the warning

    emitter.label("__rt_mixed_array_get_assoc");
    emitter.instruction("mov r10, QWORD PTR [rdi + 8]");                        // r10 = hash pointer
    emit_branch_if_null_container(
        emitter,
        "r10",
        "r11",
        "__rt_mixed_array_get_null_container",
    );
    emitter.instruction("mov rdi, r10");                                        // rdi = hash pointer for hash_get
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // rsi = key_lo
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // rdx = key_hi
    emitter.instruction("call __rt_hash_get");                                  // rax=found, rdi=value_lo, rsi=value_hi, rcx=value_tag
    emitter.instruction("test rax, rax");                                       // miss → null
    emitter.instruction("je __rt_mixed_array_get_assoc_missing");               // diagnose an absent hash key for ordinary reads
    // For value_tag == 7 the entry is already a boxed Mixed pointer; for
    // any other tag (typed string/int/array entries from non-Mixed assoc
    // arrays passing through a Mixed receiver) re-box (lo, hi, tag) so
    // callers always see a uniform Mixed cell.
    emitter.instruction("cmp rcx, 7");                                          // is the hash entry already a boxed Mixed?
    emitter.instruction("jne __rt_mixed_array_get_assoc_box");                  // no → box (lo, hi, tag) into a fresh Mixed cell
    emitter.instruction("mov rax, rdi");                                        // yes → move the stored Mixed cell into the return register
    emitter.instruction("mov r11, QWORD PTR [rax]");                            // inspect the stored Mixed tag before returning the cell
    emitter.instruction(&format!("cmp r11, {}", INVOKER_ARG_REF_CELL_TAG));     // is this a borrowed invoker reference marker?
    emitter.instruction("je __rt_mixed_array_get_ref_marker");                  // expose the referenced PHP value instead of the internal marker
    emitter.instruction(&format!("cmp r11, {}", ARRAY_GLOBAL_REF_CELL_TAG));    // is this an owning global-reference marker?
    emitter.instruction("je __rt_mixed_array_get_ref_marker");                  // all marker kinds share the referenced-cell payload layout
    emitter.instruction(&format!("cmp r11, {}", ARRAY_LOCAL_REF_CELL_TAG));     // is this an owning local-reference marker?
    emitter.instruction("je __rt_mixed_array_get_ref_marker");                  // expose the retained local cell's current PHP value
    abi::emit_push_reg(emitter, "rax");
    emitter.instruction("call __rt_incref");                                    // retain the stored Mixed cell so the caller owns the returned result
    abi::emit_pop_reg(emitter, "rax");
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax

    emitter.label("__rt_mixed_array_get_assoc_missing");
    emitter.instruction("cmp QWORD PTR [rbp - 32], 0");                         // are ordinary read warnings enabled?
    emitter.instruction("je __rt_mixed_array_get_null");                        // `isset()`/`??` suppress undefined-key warnings
    emitter.instruction("cmp QWORD PTR [rbp - 24], -1");                        // does the missing key use the integer-key sentinel?
    emitter.instruction("je __rt_mixed_array_get_assoc_missing_int");           // integer keys use the decimal warning formatter
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the missing string key pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // reload the missing string key length
    emitter.instruction("call __rt_warn_undefined_array_key_str");              // emit the PHP warning for a missing string key
    emitter.instruction("jmp __rt_mixed_array_get_null");                       // return boxed Mixed(null) after the warning
    emitter.label("__rt_mixed_array_get_assoc_missing_int");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the missing integer key
    emitter.instruction("call __rt_warn_undefined_array_key_int");              // emit the PHP warning for a missing integer key
    emitter.instruction("jmp __rt_mixed_array_get_null");                       // return boxed Mixed(null) after the warning
    emitter.label("__rt_mixed_array_get_assoc_box");
    // mixed_from_value(tag, lo, hi). Helper expects rax=tag, rdi=lo, rsi=hi.
    emitter.instruction("mov rax, rcx");                                        // rax = value_tag
    // rdi and rsi already hold value_lo and value_hi.
    emitter.instruction("call __rt_mixed_from_value");                          // box the typed entry into a Mixed cell
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax

    emitter.label("__rt_mixed_array_get_object");
    emitter.instruction("mov r10, QWORD PTR [rdi + 8]");                        // r10 = obj pointer
    emit_branch_if_null_container(
        emitter,
        "r10",
        "r11",
        "__rt_mixed_array_get_null_container",
    );
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // r11 = class_id
    abi::emit_load_symbol_to_reg(emitter, "r12", "_spl_fixed_array_class_id", 0);
    emitter.instruction("cmp r11, r12");                                        // is the receiver a SplFixedArray instance?
    emitter.instruction("je __rt_mixed_array_get_spl_fixed");                   // dispatch mixed object indexing to SplFixedArray::offsetGet
    abi::emit_load_symbol_to_reg(emitter, "r12", "_spl_dll_class_id", 0);
    emitter.instruction("cmp r11, r12");                                        // is the receiver a SplDoublyLinkedList instance?
    emitter.instruction("je __rt_mixed_array_get_spl_dll");                     // dispatch mixed object indexing to the list ArrayAccess helper
    abi::emit_load_symbol_to_reg(emitter, "r12", "_spl_stack_class_id", 0);
    emitter.instruction("cmp r11, r12");                                        // is the receiver a SplStack instance?
    emitter.instruction("je __rt_mixed_array_get_spl_dll");                     // SplStack shares the list ArrayAccess helper
    abi::emit_load_symbol_to_reg(emitter, "r12", "_spl_queue_class_id", 0);
    emitter.instruction("cmp r11, r12");                                        // is the receiver a SplQueue instance?
    emitter.instruction("je __rt_mixed_array_get_spl_dll");                     // SplQueue shares the list ArrayAccess helper
    abi::emit_load_symbol_to_reg(emitter, "r12", "_stdclass_class_id", 0);      // r12 = compile-time stdClass class_id
    emitter.instruction("cmp r11, r12");                                        // is the receiver a stdClass instance?
    emitter.instruction("jne __rt_mixed_array_get_null");                       // unrelated class → null
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // load key_hi
    emitter.instruction("cmp r11, -1");                                         // int-key sentinel?
    emitter.instruction("je __rt_mixed_array_get_null");                        // int key on object → null
    emitter.instruction("mov rdi, r10");                                        // rdi = stdClass pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // rsi = key_lo (str ptr)
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // rdx = key_hi (str len)
    emitter.instruction("call __rt_stdclass_get");                              // delegate to the dynamic-property reader
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax
    emitter.label("__rt_mixed_array_get_spl_fixed");
    emitter.instruction("mov QWORD PTR [rbp - 8], r10");                        // save unboxed SplFixedArray receiver while boxing the key
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // reload normalized key high word
    emitter.instruction("cmp r11, -1");                                         // does key_hi carry the integer-key sentinel?
    emitter.instruction("je __rt_mixed_array_get_spl_fixed_int_key");           // integer keys box as Mixed int
    emitter.instruction("mov rax, 1");                                          // tag = string for mixed_from_value
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // key string pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // key string length
    emitter.instruction("jmp __rt_mixed_array_get_spl_fixed_box_key");          // share the offsetGet call after key boxing
    emitter.label("__rt_mixed_array_get_spl_fixed_int_key");
    emitter.instruction("mov rax, 0");                                          // tag = int for mixed_from_value
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // key integer payload
    emitter.instruction("xor esi, esi");                                        // integer keys have no high payload
    emitter.label("__rt_mixed_array_get_spl_fixed_box_key");
    emitter.instruction("call __rt_mixed_from_value");                          // allocate the boxed ArrayAccess offset
    emitter.instruction("mov rsi, rax");                                        // pass boxed offset as argument 1
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass unboxed SplFixedArray receiver as argument 0
    emitter.instruction("call __rt_spl_fixed_offset_get");                      // read through SplFixedArray::offsetGet
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax
    emitter.label("__rt_mixed_array_get_spl_dll");
    emitter.instruction("mov QWORD PTR [rbp - 8], r10");                        // save unboxed SPL list receiver while boxing the key
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // reload normalized key high word
    emitter.instruction("cmp r11, -1");                                         // does key_hi carry the integer-key sentinel?
    emitter.instruction("je __rt_mixed_array_get_spl_dll_int_key");             // integer keys box as Mixed int
    emitter.instruction("mov rax, 1");                                          // tag = string for mixed_from_value
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // key string pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // key string length
    emitter.instruction("jmp __rt_mixed_array_get_spl_dll_box_key");            // share the offsetGet call after key boxing
    emitter.label("__rt_mixed_array_get_spl_dll_int_key");
    emitter.instruction("mov rax, 0");                                          // tag = int for mixed_from_value
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // key integer payload
    emitter.instruction("xor esi, esi");                                        // integer keys have no high payload
    emitter.label("__rt_mixed_array_get_spl_dll_box_key");
    emitter.instruction("call __rt_mixed_from_value");                          // allocate the boxed ArrayAccess offset
    emitter.instruction("mov rsi, rax");                                        // pass boxed offset as argument 1
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass unboxed SPL list receiver as argument 0
    emitter.instruction("call __rt_spl_dll_offset_get");                        // read through the shared SPL list offsetGet helper
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax

    emitter.label("__rt_mixed_array_get_null_container");
    emitter.instruction("cmp QWORD PTR [rbp - 32], 0");                         // are ordinary read warnings enabled?
    emitter.instruction("je __rt_mixed_array_get_null");                        // quiet read contexts suppress the null-offset warning
    emitter.instruction("call __rt_warn_array_offset_on_null");                 // emit PHP's warning for indexing a null receiver
    emitter.label("__rt_mixed_array_get_null");
    emitter.instruction("mov rax, 8");                                          // tag = 8 (null) for mixed_from_value
    emitter.instruction("mov rdi, 0");                                          // value_lo = 0
    emitter.instruction("mov rsi, 0");                                          // value_hi = 0
    emitter.instruction("call __rt_mixed_from_value");                          // box null into a fresh Mixed cell
    emitter.instruction("mov rsp, rbp");                                        // restore stack pointer
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return Mixed* in rax
}
