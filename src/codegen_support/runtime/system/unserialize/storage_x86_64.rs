//! Purpose:
//! Emits x86_64 parsed-hash conversion and serialized-key parsing.
//!
//! Called from:
//! - `super::emit_unserialize()` after the recursive decoder and per-call context helpers.
//!
//! Key details:
//! - Object slot replacement is emitted into user assembly with concrete class metadata.
//! - Converted indexed arrays acquire their own element references from the parsed hash.

use crate::codegen_support::emit::Emitter;


/// Emits x86_64 parsed-hash conversion helpers.
pub(super) fn emit_object_storage(emitter: &mut Emitter) {

    // -- __rt_hash_to_indexed_array(rdi=hash) -> rax=indexed array: rebuild a parsed
    // hash (boxed-Mixed values) as a native value_type-7 indexed array. --
    emitter.label_global("__rt_hash_to_indexed_array");
    emitter.instruction("push rbp");                                            // open the conversion frame
    emitter.instruction("mov rbp, rsp");                                        // set the frame pointer
    emitter.instruction("sub rsp, 32");                                         // reserve callee-saved spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rbx");                        // save rbx
    emitter.instruction("mov QWORD PTR [rbp - 16], r12");                       // save r12
    emitter.instruction("mov QWORD PTR [rbp - 24], r13");                       // save r13
    emitter.instruction("mov rbx, rdi");                                        // hash pointer
    emitter.instruction("mov rdi, 0");                                          // initial capacity 0
    emitter.instruction("mov rsi, 8");                                          // 8-byte element slots
    emitter.instruction("call __rt_array_new");                                 // allocate an empty indexed array
    emitter.instruction("mov r12, rax");                                        // destination array pointer
    emitter.instruction("xor r13, r13");                                        // hash iteration cursor
    emitter.label("__rt_hash_to_indexed_array_loop");
    emitter.instruction("mov rdi, rbx");                                        // hash pointer
    emitter.instruction("mov rsi, r13");                                        // resume cursor
    emitter.instruction("call __rt_hash_iter_next_value");                      // rcx=value low, rax=next cursor
    emitter.instruction("cmp rax, -1");                                         // iteration done?
    emitter.instruction("je __rt_hash_to_indexed_array_done");                  // stop when exhausted
    emitter.instruction("mov r13, rax");                                        // save the resume cursor
    emitter.instruction("mov rdi, r12");                                        // destination array
    emitter.instruction("mov rsi, rcx");                                        // boxed-Mixed value pointer (parsed-hash value)
    emitter.instruction("call __rt_array_push_refcounted");                     // append, transferring ownership
    emitter.instruction("mov r12, rax");                                        // array may move on COW growth
    emitter.instruction("jmp __rt_hash_to_indexed_array_loop");                 // continue iterating
    emitter.label("__rt_hash_to_indexed_array_done");
    emitter.instruction("mov rax, r12");                                        // return the indexed array
    emitter.instruction("mov rbx, QWORD PTR [rbp - 8]");                        // restore rbx
    emitter.instruction("mov r12, QWORD PTR [rbp - 16]");                       // restore r12
    emitter.instruction("mov r13, QWORD PTR [rbp - 24]");                       // restore r13
    emitter.instruction("add rsp, 32");                                         // close the conversion frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the converted array
}

/// Emits the x86_64 leaf key parser `__rt_unser_key`.
///
/// Input: `rdi`=base, `rsi`=pos, `rdx`=end. Output: `rax`=key_lo, `rdx`=key_hi (-1 for
/// an integer key, else the string byte length), `rcx`=newpos. String key pointers are
/// borrowed into the source buffer; `__rt_hash_set` persists them.
pub(super) fn emit_key(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unser_key (serialize() array key parser, leaf) ---");
    emitter.label_global("__rt_unser_key");
    emitter.instruction("cmp rsi, rdx");                                        // require a key type byte before loading it
    emitter.instruction("jae __rt_unser_key_fail_x");                           // return a sentinel cursor for a truncated key
    emitter.instruction("movzx r9d, BYTE PTR [rdi + rsi]");                     // load the key type byte
    emitter.instruction("cmp r9d, 105");                                        // ASCII 'i' (integer key)?
    emitter.instruction("je __rt_unser_key_int");                               // parse an integer key
    // -- string key: "s:" + bytelen + ":\"" + raw + "\";" --
    emitter.instruction("mov r10, rdi");                                        // base copy for cursor math
    emitter.instruction("add r10, rsi");                                        // pointer to the type byte
    emitter.instruction("add r10, 2");                                          // skip "s:" to the length digits
    emitter.instruction("xor r11, r11");                                        // length accumulator
    emitter.label("__rt_unser_key_strlen");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next length byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_key_strlen_done");                       // ':' terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_key_strlen_done");                       // ':' terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_key_strlen");                           // continue
    emitter.label("__rt_unser_key_strlen_done");
    emitter.instruction("add r10, 2");                                          // skip ':' and opening '\"' to the raw bytes
    emitter.instruction("mov r8, r10");                                         // raw end accumulator = raw start
    emitter.instruction("add r8, r11");                                         // raw end = raw + len
    emitter.instruction("add r8, 2");                                           // skip closing '\"' and ';'
    emitter.instruction("sub r8, rdi");                                         // newpos = (raw end + 2) - base
    emitter.instruction("mov rcx, r8");                                         // key newpos
    emitter.instruction("mov rdx, r11");                                        // key_hi = string byte length
    emitter.instruction("mov rax, r10");                                        // key_lo = borrowed raw string pointer
    emitter.instruction("ret");                                                 // return the string key
    // -- integer key: "i:" + optional '-' + digits + ";" --
    emitter.label("__rt_unser_key_int");
    emitter.instruction("mov r10, rdi");                                        // base copy for cursor math
    emitter.instruction("add r10, rsi");                                        // pointer to the type byte
    emitter.instruction("add r10, 2");                                          // skip "i:" to the first digit
    emitter.instruction("xor r11, r11");                                        // digit accumulator
    emitter.instruction("xor r8, r8");                                          // negative-sign flag
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // first numeric byte
    emitter.instruction("cmp r9d, 45");                                         // leading '-'?
    emitter.instruction("jne __rt_unser_key_int_loop");                         // no sign
    emitter.instruction("mov r8, 1");                                           // record negative sign
    emitter.instruction("add r10, 1");                                          // skip '-'
    emitter.label("__rt_unser_key_int_loop");
    emitter.instruction("movzx r9d, BYTE PTR [r10]");                           // next numeric byte
    emitter.instruction("cmp r9d, 48");                                         // below '0'?
    emitter.instruction("jl __rt_unser_key_int_done");                          // ';' terminator reached
    emitter.instruction("cmp r9d, 57");                                         // above '9'?
    emitter.instruction("jg __rt_unser_key_int_done");                          // ';' terminator reached
    emitter.instruction("sub r9d, 48");                                         // digit value
    emitter.instruction("imul r11, r11, 10");                                   // shift accumulator
    emitter.instruction("add r11, r9");                                         // add digit
    emitter.instruction("add r10, 1");                                          // advance cursor
    emitter.instruction("jmp __rt_unser_key_int_loop");                         // continue
    emitter.label("__rt_unser_key_int_done");
    emitter.instruction("test r8, r8");                                         // signed?
    emitter.instruction("jz __rt_unser_key_int_pos");                           // not signed
    emitter.instruction("neg r11");                                             // apply sign
    emitter.label("__rt_unser_key_int_pos");
    emitter.instruction("mov rcx, r10");                                        // cursor copy
    emitter.instruction("sub rcx, rdi");                                        // newpos = cursor - base
    emitter.instruction("add rcx, 1");                                          // skip the ';'
    emitter.instruction("mov rax, r11");                                        // key_lo = integer key value
    emitter.instruction("mov rdx, -1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("ret");                                                 // return the integer key
    emitter.label("__rt_unser_key_fail_x");
    emitter.instruction("lea rcx, [rdx + 1]");                                  // end+1 is an impossible valid cursor
    emitter.instruction("xor eax, eax");                                        // clear key payload on failure
    emitter.instruction("xor edx, edx");                                        // clear key metadata on failure
    emitter.instruction("ret");                                                 // caller/preflight rejects the sentinel
}
