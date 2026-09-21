//! Purpose:
//! Emits AArch64 object-property hydration, hash conversion, and serialized-key parsing.
//!
//! Called from:
//! - `super::emit_unserialize()` after the recursive decoder and per-call context helpers.
//!
//! Key details:
//! - Parsed Mixed ownership is transferred into object slots or rebuilt indexed arrays without extra retains.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::sentinels::PROP_DESC_TAG_TAGGED_SCALAR;

/// Emits AArch64 object-property storage and parsed-hash conversion helpers.
pub(super) fn emit_object_storage(emitter: &mut Emitter) {
    // -- __rt_obj_store_prop(x0=obj, x1=key_ptr, x2=key_len, x3=valbox): inject a property --
    // Matches the key against the class's serialize property-info table and stores the parsed
    // value into the matching object slot per the property's tag.
    //
    // TWO PASSES, because a serialized key is not always mangled. `serialize()` writes a
    // private property as "\0Class\0name" and a protected one as "\0*\0name", but a class with
    // a `__serialize()` chooses its own keys, and the natural spelling there is the DECLARED
    // name: Symfony's `FileResource::__serialize()` returns `['resource' => …]` for a
    // `private string $resource`. PHP resolves such a key against the class's declared
    // properties, so it lands in the private slot; matching only the mangled row left the
    // typed property uninitialized and every later read raised "must not be accessed before
    // initialization". Exact keys are matched FIRST across all rows so an explicit mangled key
    // still wins over an unmangled alias when a class has both spellings.
    emitter.label_global("__rt_obj_store_prop");
    emitter.instruction("ldr x9, [x0]");                                        // class id from the object header
    crate::codegen_support::abi::emit_symbol_address(emitter, "x10", "_class_serprop_ptrs");
    emitter.instruction("ldr x10, [x10, x9, lsl #3]");                          // property-info table for this class
    emitter.instruction("ldr x11, [x10]");                                      // property count
    emitter.instruction("add x12, x10, #8");                                    // rows start (skip the count word)
    emitter.instruction("mov x15, #0");                                         // pass 0 = key as stored, pass 1 = unmangled tail
    emitter.label("__rt_obj_store_prop_pass");
    emitter.instruction("mov x13, #0");                                         // row index
    emitter.label("__rt_obj_store_prop_loop");
    emitter.instruction("cmp x13, x11");                                        // scanned every row?
    emitter.instruction("b.ge __rt_obj_store_prop_pass_end");                   // this pass found nothing
    emitter.instruction("add x14, x12, x13, lsl #5");                           // row = rows + index*32
    emitter.instruction("ldr x4, [x14]");                                       // row mangled key pointer
    emitter.instruction("ldr x5, [x14, #8]");                                   // row mangled key length
    emitter.instruction("cbz x15, __rt_obj_store_prop_have_key");               // pass 0 compares the row key verbatim
    // Narrow the row key to whatever follows its LAST NUL, which is the declared property
    // name. A row with no NUL is a public property whose key is already the declared name and
    // was compared verbatim in pass 0, so it is skipped rather than compared twice.
    emitter.instruction("mov x6, #0");                                          // scan cursor
    emitter.instruction("mov x7, #-1");                                         // index of the last NUL seen
    emitter.label("__rt_obj_store_prop_tail_scan");
    emitter.instruction("cmp x6, x5");                                          // scanned the whole row key?
    emitter.instruction("b.ge __rt_obj_store_prop_tail_done");                  // stop at the end of the key
    emitter.instruction("ldrb w8, [x4, x6]");                                   // row key byte
    emitter.instruction("cbnz w8, __rt_obj_store_prop_tail_next");              // only NUL separators matter
    emitter.instruction("mov x7, x6");                                          // remember this separator
    emitter.label("__rt_obj_store_prop_tail_next");
    emitter.instruction("add x6, x6, #1");                                      // next byte
    emitter.instruction("b __rt_obj_store_prop_tail_scan");                     // continue scanning
    emitter.label("__rt_obj_store_prop_tail_done");
    emitter.instruction("cmn x7, #1");                                          // was any separator found?
    emitter.instruction("b.eq __rt_obj_store_prop_next");                       // an unmangled row has no alias to try
    emitter.instruction("add x7, x7, #1");                                      // the name starts after the separator
    emitter.instruction("add x4, x4, x7");                                      // declared-name pointer
    emitter.instruction("sub x5, x5, x7");                                      // declared-name length
    emitter.label("__rt_obj_store_prop_have_key");
    emitter.instruction("cmp x5, x2");                                          // same length as the parsed key?
    emitter.instruction("b.ne __rt_obj_store_prop_next");                       // lengths differ, skip
    emitter.instruction("mov x6, #0");                                          // byte compare cursor
    emitter.label("__rt_obj_store_prop_cmp");
    emitter.instruction("cmp x6, x2");                                          // compared all bytes?
    emitter.instruction("b.ge __rt_obj_store_prop_match");                      // full match
    emitter.instruction("ldrb w7, [x4, x6]");                                   // row key byte
    emitter.instruction("ldrb w8, [x1, x6]");                                   // parsed key byte
    emitter.instruction("cmp w7, w8");                                          // bytes equal?
    emitter.instruction("b.ne __rt_obj_store_prop_next");                       // mismatch, skip this row
    emitter.instruction("add x6, x6, #1");                                      // next byte
    emitter.instruction("b __rt_obj_store_prop_cmp");                           // continue comparing
    emitter.label("__rt_obj_store_prop_match");
    emitter.instruction("ldr x6, [x14, #16]");                                  // property byte offset
    emitter.instruction("ldr x7, [x14, #24]");                                  // property value tag
    emitter.instruction("add x8, x0, x6");                                      // address of the property slot
    emitter.instruction("cmp x7, #7");                                          // is this a Mixed/untyped slot?
    emitter.instruction("b.eq __rt_obj_store_prop_mixed");                      // store the boxed cell directly
    emitter.instruction("cmp x7, #1");                                          // is this a string slot?
    emitter.instruction("b.eq __rt_obj_store_prop_str");                        // store pointer and length
    emitter.instruction("cmp x7, #4");                                          // is this an indexed-array slot?
    emitter.instruction("b.eq __rt_obj_store_prop_arr");                        // convert the parsed hash to an indexed array
    emitter.instruction(&format!("cmp x7, #{}", PROP_DESC_TAG_TAGGED_SCALAR));  // is this an inline tagged-scalar slot?
    emitter.instruction("b.eq __rt_obj_store_prop_tagged");                     // rebuild both the payload and the slot's own runtime tag
    emitter.instruction("ldr x9, [x3, #8]");                                    // typed scalar/object/hash: unbox the low word
    emitter.instruction("str x9, [x8]");                                        // store it inline in the slot
    // A single-payload typed slot keeps its initialization marker in the HIGH word, and a
    // property DECLARED WITHOUT A DEFAULT still carries the uninitialized sentinel there when
    // unserialize() hydrates it. Writing only the payload left `private array $p;` reading back
    // as "must not be accessed before initialization" while `private array $p = [];` worked --
    // which is exactly how Symfony's second request died in ContainerParametersResource.
    emitter.instruction("str xzr, [x8, #8]");                                   // an unserialized property is initialized
    emitter.instruction("ret");                                                 // property stored
    // An `array`-typed slot carries the INDEXED descriptor tag whatever it ends up holding, so
    // the parsed hash may only be flattened into an indexed array when its keys really are the
    // list 0..n-1. Converting unconditionally renumbered every string key: Symfony's
    // `['kernel.debug' => true]` came back as `[0 => true]`. A non-list keeps hash storage, the
    // same thing `$obj->arr['k'] = v` leaves in the slot.
    emitter.label("__rt_obj_store_prop_arr");
    emitter.instruction("ldr x9, [x3, #8]");                                    // parsed hash pointer (box low word)
    emitter.instruction("stp x8, x30, [sp, #-16]!");                            // save the slot address and return address
    emitter.instruction("str x9, [sp, #-16]!");                                 // keep the parsed hash across the classification
    emitter.instruction("mov x0, x9");                                          // classify the parsed hash
    emitter.instruction("bl __rt_array_is_list");                               // are its keys exactly 0..n-1 in order?
    emitter.instruction("ldr x9, [sp], #16");                                   // restore the parsed hash pointer
    emitter.instruction("cbz x0, __rt_obj_store_prop_arr_keep_hash");           // a keyed array stays hash-backed
    emitter.instruction("mov x0, x9");                                          // a list is flattened for indexed property access
    emitter.instruction("bl __rt_hash_to_indexed_array");                       // materialize a native indexed array
    emitter.instruction("b __rt_obj_store_prop_arr_store");                     // store whichever container was chosen
    emitter.label("__rt_obj_store_prop_arr_keep_hash");
    emitter.instruction("mov x0, x9");                                          // store the parsed hash exactly as it was decoded
    emitter.label("__rt_obj_store_prop_arr_store");
    emitter.instruction("ldp x8, x30, [sp], #16");                              // restore the slot address and return address
    emitter.instruction("str x0, [x8]");                                        // store the array pointer
    emitter.instruction("str xzr, [x8, #8]");                                   // an unserialized property is initialized
    emitter.instruction("ret");                                                 // property stored
    emitter.label("__rt_obj_store_prop_str");
    emitter.instruction("ldr x9, [x3, #8]");                                    // string pointer from the box
    emitter.instruction("str x9, [x8]");                                        // store the string pointer
    emitter.instruction("ldr x9, [x3, #16]");                                   // string length from the box
    emitter.instruction("str x9, [x8, #8]");                                    // store the string length
    emitter.instruction("ret");                                                 // property stored
    // A `?int` / `int|null` slot is two inline words: the payload, then the runtime value
    // tag that says which of int/null the payload is. Writing only the low word (the plain
    // typed-scalar arm above) would leave a stale tag behind and make `n:null` read back as
    // the integer `NULL_SENTINEL`, so the tag word is restored from the parsed box too. The
    // canonical null payload is the in-band sentinel, matching `emit_tagged_scalar_null`.
    emitter.label("__rt_obj_store_prop_tagged");
    emitter.instruction("ldr x9, [x3]");                                        // boxed value tag
    emitter.instruction("ldr x10, [x3, #8]");                                   // boxed payload low word
    emitter.instruction("cmp x9, #8");                                          // is the parsed value PHP null?
    emitter.instruction("b.ne __rt_obj_store_prop_tagged_store");               // a non-null payload is stored exactly as parsed
    crate::codegen_support::abi::emit_load_int_immediate(emitter, "x10", crate::codegen_support::NULL_SENTINEL);
    emitter.label("__rt_obj_store_prop_tagged_store");
    emitter.instruction("str x10, [x8]");                                       // store the tagged-scalar payload word
    emitter.instruction("str x9, [x8, #8]");                                    // store the tagged-scalar runtime tag word
    emitter.instruction("ret");                                                 // property stored
    // PHP null goes into the slot as a boxed cell like every other value. Writing the bare
    // in-band `NULL_SENTINEL` instead would be a shape NO other producer of a Mixed property
    // slot emits — the constructor's null default and `$o->p = null` both store a cell — and
    // the unguarded readers (`var_dump`, `===`, `isset`, the `(array)` cast) dereference the
    // slot's low word on sight, so a sentinel there is a segfault, not a null. It also leaks:
    // the caller transfers the parsed box unconditionally and never frees it.
    emitter.label("__rt_obj_store_prop_mixed");
    emitter.instruction("str x3, [x8]");                                        // store the parsed boxed Mixed cell, PHP null included
    emitter.instruction("str xzr, [x8, #8]");                                   // an unserialized property is initialized: clear any uninitialized marker
    emitter.instruction("ret");                                                 // property stored
    emitter.label("__rt_obj_store_prop_next");
    emitter.instruction("add x13, x13, #1");                                    // advance to the next row
    emitter.instruction("b __rt_obj_store_prop_loop");                          // continue scanning
    emitter.label("__rt_obj_store_prop_pass_end");
    emitter.instruction("cbnz x15, __rt_obj_store_prop_done");                  // both passes are spent
    emitter.instruction("mov x15, #1");                                         // retry against declared names
    emitter.instruction("b __rt_obj_store_prop_pass");                          // rescan the rows unmangled
    emitter.label("__rt_obj_store_prop_done");
    emitter.instruction("ret");                                                 // no matching property, ignore the value

    // -- __rt_hash_to_indexed_array(x0=hash) -> x0=indexed array: rebuild a parsed
    // hash (with boxed-Mixed values) as a native value_type-7 indexed array so
    // indexed-array-typed property slots match what property access expects. --
    emitter.label_global("__rt_hash_to_indexed_array");
    emitter.instruction("stp x29, x30, [sp, #-48]!");                           // open the conversion frame
    emitter.instruction("mov x29, sp");                                         // set the frame pointer
    emitter.instruction("stp x19, x20, [sp, #16]");                             // save callee-saved temporaries
    emitter.instruction("str x21, [sp, #32]");                                  // save callee-saved cursor
    emitter.instruction("mov x19, x0");                                         // hash pointer
    emitter.instruction("mov x0, #0");                                          // initial capacity 0
    emitter.instruction("mov x1, #8");                                          // 8-byte element slots
    emitter.instruction("bl __rt_array_new");                                   // allocate an empty indexed array
    emitter.instruction("mov x20, x0");                                         // destination array pointer
    emitter.instruction("mov x21, #0");                                         // hash iteration cursor
    emitter.label("__rt_hash_to_indexed_array_loop");
    emitter.instruction("mov x0, x19");                                         // hash pointer
    emitter.instruction("mov x1, x21");                                         // resume cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // x3=value low, x5=value tag, x0=next cursor
    emitter.instruction("cmn x0, #1");                                          // cursor == -1 (iteration done)?
    emitter.instruction("b.eq __rt_hash_to_indexed_array_done");                // stop when exhausted
    emitter.instruction("mov x21, x0");                                         // save the resume cursor
    emitter.instruction("mov x0, x20");                                         // destination array
    emitter.instruction("mov x1, x3");                                          // boxed-Mixed value pointer (parsed-hash value)
    emitter.instruction("bl __rt_array_push_refcounted");                       // append, transferring ownership
    emitter.instruction("mov x20, x0");                                         // array may move on COW growth
    emitter.instruction("b __rt_hash_to_indexed_array_loop");                   // continue iterating
    emitter.label("__rt_hash_to_indexed_array_done");
    emitter.instruction("mov x0, x20");                                         // return the indexed array
    emitter.instruction("ldr x21, [sp, #32]");                                  // restore the cursor register
    emitter.instruction("ldp x19, x20, [sp, #16]");                             // restore the temporaries
    emitter.instruction("ldp x29, x30, [sp], #48");                             // close the conversion frame
    emitter.instruction("ret");                                                 // return the converted array
}

/// Emits the AArch64 leaf key parser `__rt_unser_key`.
///
/// Input: `x0`=base, `x1`=pos, `x2`=end. Output: `x0`=key_lo (int value or string
/// pointer), `x1`=key_hi (-1 for an integer key, else the string byte length), `x2`=newpos.
/// String key pointers are borrowed into the source buffer; `__rt_hash_set` persists them.
pub(super) fn emit_key(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unser_key (serialize() array key parser, leaf) ---");
    emitter.label_global("__rt_unser_key");
    emitter.instruction("cmp x1, x2");                                          // require a key type byte before loading it
    emitter.instruction("b.hs __rt_unser_key_fail");                            // return a sentinel cursor for a truncated key
    emitter.instruction("ldrb w9, [x0, x1]");                                   // load the key type byte
    emitter.instruction("cmp w9, #105");                                        // ASCII 'i' (integer key)?
    emitter.instruction("b.eq __rt_unser_key_int");                             // parse an integer key
    // -- string key: "s:" + bytelen + ":\"" + raw + "\";" --
    emitter.instruction("add x10, x0, x1");                                     // pointer to the type byte
    emitter.instruction("add x10, x10, #2");                                    // skip "s:" to the length digits
    emitter.instruction("mov x11, #0");                                         // length accumulator
    emitter.label("__rt_unser_key_strlen");
    emitter.instruction("ldrb w9, [x10]");                                      // next length byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_key_strlen_done");                     // ':' terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_key_strlen_done");                     // ':' terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x12, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x12");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x10, x10, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_key_strlen");                             // continue
    emitter.label("__rt_unser_key_strlen_done");
    emitter.instruction("add x10, x10, #2");                                    // skip ':' and opening '\"' to the raw bytes
    emitter.instruction("add x12, x10, x11");                                   // raw end = raw + len
    emitter.instruction("add x12, x12, #2");                                    // skip closing '\"' and ';'
    emitter.instruction("sub x2, x12, x0");                                     // newpos = (raw end + 2) - base
    emitter.instruction("mov x1, x11");                                         // key_hi = string byte length
    emitter.instruction("mov x0, x10");                                         // key_lo = borrowed raw string pointer
    emitter.instruction("ret");                                                 // return the string key
    // -- integer key: "i:" + optional '-' + digits + ";" --
    emitter.label("__rt_unser_key_int");
    emitter.instruction("add x10, x0, x1");                                     // pointer to the type byte
    emitter.instruction("add x10, x10, #2");                                    // skip "i:" to the first digit
    emitter.instruction("mov x11, #0");                                         // digit accumulator
    emitter.instruction("mov x13, #0");                                         // negative-sign flag
    emitter.instruction("ldrb w9, [x10]");                                      // first numeric byte
    emitter.instruction("cmp w9, #45");                                         // leading '-'?
    emitter.instruction("b.ne __rt_unser_key_int_loop");                        // no sign
    emitter.instruction("mov x13, #1");                                         // record negative sign
    emitter.instruction("add x10, x10, #1");                                    // skip '-'
    emitter.label("__rt_unser_key_int_loop");
    emitter.instruction("ldrb w9, [x10]");                                      // next numeric byte
    emitter.instruction("cmp w9, #48");                                         // below '0'?
    emitter.instruction("b.lt __rt_unser_key_int_done");                        // ';' terminator reached
    emitter.instruction("cmp w9, #57");                                         // above '9'?
    emitter.instruction("b.gt __rt_unser_key_int_done");                        // ';' terminator reached
    emitter.instruction("sub w9, w9, #48");                                     // digit value
    emitter.instruction("mov x12, #10");                                        // decimal base
    emitter.instruction("mul x11, x11, x12");                                   // shift accumulator
    emitter.instruction("add x11, x11, x9");                                    // add digit
    emitter.instruction("add x10, x10, #1");                                    // advance cursor
    emitter.instruction("b __rt_unser_key_int_loop");                           // continue
    emitter.label("__rt_unser_key_int_done");
    emitter.instruction("cbz x13, __rt_unser_key_int_pos");                     // not signed
    emitter.instruction("neg x11, x11");                                        // apply sign
    emitter.label("__rt_unser_key_int_pos");
    emitter.instruction("sub x2, x10, x0");                                     // newpos = cursor - base
    emitter.instruction("add x2, x2, #1");                                      // skip the ';'
    emitter.instruction("mov x0, x11");                                         // key_lo = integer key value
    emitter.instruction("mov x1, #-1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("ret");                                                 // return the integer key
    emitter.label("__rt_unser_key_fail");
    emitter.instruction("mov x0, #0");                                          // clear key payload on failure
    emitter.instruction("mov x1, #0");                                          // clear key metadata on failure
    emitter.instruction("add x2, x2, #1");                                      // end+1 is an impossible valid cursor
    emitter.instruction("ret");                                                 // caller/preflight rejects the sentinel
}
