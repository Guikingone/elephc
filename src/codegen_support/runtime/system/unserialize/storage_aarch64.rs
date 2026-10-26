//! Purpose:
//! Emits AArch64 parsed-hash conversion and serialized-key parsing.
//!
//! Called from:
//! - `super::emit_unserialize()` after the recursive decoder and per-call context helpers.
//!
//! Key details:
//! - Object slot replacement is emitted into user assembly with concrete class metadata.
//! - Converted indexed arrays acquire their own element references from the parsed hash.

use crate::codegen_support::emit::Emitter;


/// Emits AArch64 parsed-hash conversion helpers.
pub(super) fn emit_object_storage(emitter: &mut Emitter) {

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
    emitter.instruction("bl __rt_hash_iter_next_value");                        // x3=value low, x5=value tag, x0=next cursor
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::Target;

    /// Every AArch64 target rejects a hydrated value whose boxed tag the declared property
    /// type does not accept, routing it through the catchable unserialize TypeError helper
    /// instead of writing the mismatched payload.
    #[test]
    fn store_prop_rejects_a_mismatched_typed_property_on_every_aarch64_target() {
        for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64"] {
            let mut emitter = Emitter::new(Target::parse(name).unwrap());
            emit_object_storage(&mut emitter);
            let asm = emitter.output();
            assert!(asm.contains("_class_serpdiag_ptrs"), "{name}: {asm}");
            assert!(asm.contains("__rt_obj_store_prop_type_error"), "{name}: {asm}");
            assert!(asm.contains("__rt_unser_throw_property_type_error"), "{name}: {asm}");
            assert!(asm.contains("_unser_property_assign_prefix"), "{name}: {asm}");
            // An accepted int into a float slot is widened, not reinterpreted.
            assert!(asm.contains("scvtf d0, x9"), "{name}: {asm}");
        }
    }
}
