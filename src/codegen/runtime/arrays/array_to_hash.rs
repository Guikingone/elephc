//! Purpose:
//! Emits the `__rt_array_to_hash` runtime helper assembly that promotes an indexed
//! array to a fresh associative-array (hash) representation so that mixed-kind
//! `array + array` union (indexed + assoc / assoc + indexed) can reuse `__rt_hash_union`.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::arrays`.
//!
//! Key details:
//! - The promoted hash owns refcount-retained payloads: refcounted slots are `__rt_incref`-ed
//!   and string slots are persisted via `__rt_str_persist` before being inserted, so callers
//!   must `__rt_decref_hash` the result once they are done with the temporary.
//! - The source indexed array is left untouched; the caller retains its normal ownership.

use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;

/// array_to_hash: promote a dense indexed array to a fresh hash table with int keys 0..count-1.
/// Reuses the existing string-persistence and incref helpers so the resulting hash owns its
/// payloads regardless of the source value_type tag. The returned hash has refcount 1 and is
/// suitable to feed directly into `__rt_hash_union`; the caller must decref it once finished.
/// Input:  x0=indexed_array_ptr
/// Output: x0=fresh_hash_table_ptr
pub fn emit_array_to_hash(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_to_hash_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_to_hash ---");
    emitter.label_global("__rt_array_to_hash");

    // -- set up stack frame and capture source array metadata --
    // Stack layout:
    //   [sp, #0]  = source indexed array pointer (saved across helper calls)
    //   [sp, #8]  = evolving hash table pointer
    //   [sp, #16] = source value_type tag
    //   [sp, #24] = source element count
    //   [sp, #32] = loop index i
    //   [sp, #40] = staged value_lo for __rt_hash_set
    //   [sp, #48] = staged value_hi for __rt_hash_set
    //   [sp, #56] = staged value_tag for __rt_hash_set
    //   [sp, #64] = saved x29
    //   [sp, #72] = saved x30
    emitter.instruction("sub sp, sp, #80");                                     // reserve spill slots for the indexed-to-hash promotion walk
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #64");                                    // establish a stable frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the source indexed-array pointer for slot loads
    emitter.instruction("ldr x9, [x0]");                                        // load the indexed-array length from header[0]
    emitter.instruction("str x9, [sp, #24]");                                   // save the slot count for the promotion loop
    emitter.instruction("ldr x10, [x0, #-8]");                                  // load the packed heap-kind word that carries the value_type tag
    emitter.instruction("lsr x10, x10, #8");                                    // shift the value_type tag into the low bits
    emitter.instruction("and x10, x10, #0x7f");                                 // isolate the source value_type tag for dispatch
    emitter.instruction("str x10, [sp, #16]");                                  // save the source value_type tag for the promotion loop

    // -- allocate a fresh hash table sized to avoid mid-fill growth --
    emitter.instruction("lsl x0, x9, #1");                                      // start with capacity = count * 2 to stay well under the 75% load factor
    emitter.instruction("add x0, x0, #8");                                      // add a small floor so empty/short arrays still get a usable table
    emitter.instruction("mov x1, x10");                                         // pass the source value_type tag through to the new hash
    emitter.instruction("bl __rt_hash_new");                                    // allocate a fresh hash with refcount 1 to receive the promoted entries
    emitter.instruction("str x0, [sp, #8]");                                    // save the evolving hash-table pointer for hash_set calls
    emitter.instruction("str xzr, [sp, #32]");                                  // initialize the promotion loop index i = 0

    // -- promotion loop: insert each indexed slot under integer key i --
    emitter.label("__rt_array_to_hash_loop");
    emitter.instruction("ldr x9, [sp, #32]");                                   // reload the current loop index
    emitter.instruction("ldr x10, [sp, #24]");                                  // reload the source slot count
    emitter.instruction("cmp x9, x10");                                         // have all source slots been promoted?
    emitter.instruction("b.ge __rt_array_to_hash_done");                        // finish once every source slot has been inserted

    emitter.instruction("ldr x11, [sp, #16]");                                  // reload the source value_type tag
    emitter.instruction("cmp x11, #1");                                         // is the source storing string slots (16-byte ptr+len cells)?
    emitter.instruction("b.eq __rt_array_to_hash_str_value");                   // string slots need pointer+length loads plus persistence
    emitter.instruction("cmp x11, #4");                                         // is the source payload in the refcounted tag range?
    emitter.instruction("b.lo __rt_array_to_hash_scalar_value");                // scalar tags (int/float/bool) copy directly without retention
    emitter.instruction("cmp x11, #7");                                         // is the source payload still a supported refcounted tag?
    emitter.instruction("b.hi __rt_array_to_hash_scalar_value");                // unknown high tags fall back to scalar copying

    // -- refcounted payloads (tags 4..=7): incref before transferring ownership --
    emitter.instruction("ldr x12, [sp, #0]");                                   // reload the source indexed-array pointer
    emitter.instruction("add x12, x12, #24");                                   // advance to the indexed-array payload base
    emitter.instruction("ldr x0, [x12, x9, lsl #3]");                           // load the refcounted heap pointer at slot i (8-byte stride)
    emitter.instruction("str x0, [sp, #40]");                                   // stage the heap pointer as the hash value_lo word
    emitter.instruction("bl __rt_incref");                                      // retain the heap payload so the hash owns its reference
    emitter.instruction("str xzr, [sp, #48]");                                  // refcounted payloads use a single word; clear the high word
    emitter.instruction("ldr x11, [sp, #16]");                                  // reload the source value_type tag for the per-entry stored tag
    emitter.instruction("str x11, [sp, #56]");                                  // stage the refcounted tag as the hash value_tag word
    emitter.instruction("b __rt_array_to_hash_insert");                         // proceed to the shared hash_set insertion path

    // -- string payloads (tag 1): persist the borrowed bytes for hash ownership --
    emitter.label("__rt_array_to_hash_str_value");
    emitter.instruction("ldr x12, [sp, #0]");                                   // reload the source indexed-array pointer
    emitter.instruction("lsl x13, x9, #4");                                     // compute the byte offset for a 16-byte string slot
    emitter.instruction("add x12, x12, x13");                                   // advance to the string slot inside the indexed-array payload
    emitter.instruction("add x12, x12, #24");                                   // skip the 24-byte indexed-array header
    emitter.instruction("ldp x1, x2, [x12]");                                   // load the borrowed string pointer (x1) and length (x2)
    emitter.instruction("bl __rt_str_persist");                                 // duplicate the string so the hash owns its key/value bytes
    emitter.instruction("str x1, [sp, #40]");                                   // stage the owned string pointer as the hash value_lo word
    emitter.instruction("str x2, [sp, #48]");                                   // stage the owned string length as the hash value_hi word
    emitter.instruction("mov x11, #1");                                         // the staged payload is a runtime string value
    emitter.instruction("str x11, [sp, #56]");                                  // stage the string runtime tag as the hash value_tag word
    emitter.instruction("b __rt_array_to_hash_insert");                         // proceed to the shared hash_set insertion path

    // -- scalar payloads (tags 0/2/3 and unknown high tags): copy directly --
    emitter.label("__rt_array_to_hash_scalar_value");
    emitter.instruction("ldr x12, [sp, #0]");                                   // reload the source indexed-array pointer
    emitter.instruction("add x12, x12, #24");                                   // advance to the indexed-array payload base
    emitter.instruction("ldr x13, [x12, x9, lsl #3]");                          // load the scalar payload at slot i (8-byte stride)
    emitter.instruction("str x13, [sp, #40]");                                  // stage the scalar payload as the hash value_lo word
    emitter.instruction("str xzr, [sp, #48]");                                  // scalar payloads use a single word; clear the high word
    emitter.instruction("ldr x11, [sp, #16]");                                  // reload the source value_type tag
    emitter.instruction("str x11, [sp, #56]");                                  // stage the scalar runtime tag as the hash value_tag word

    // -- insert the staged payload under integer key i in the promoted hash --
    emitter.label("__rt_array_to_hash_insert");
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the evolving hash-table pointer
    emitter.instruction("ldr x1, [sp, #32]");                                   // pass the current index i as the integer key low word
    emitter.instruction("mov x2, #-1");                                         // x2 = -1 marks the key as an integer key in __rt_hash_set
    emitter.instruction("ldr x3, [sp, #40]");                                   // pass the staged hash value_lo word
    emitter.instruction("ldr x4, [sp, #48]");                                   // pass the staged hash value_hi word
    emitter.instruction("ldr x5, [sp, #56]");                                   // pass the staged hash value_tag word
    emitter.instruction("bl __rt_hash_set");                                    // insert the integer-keyed entry into the promoted hash
    emitter.instruction("str x0, [sp, #8]");                                    // save the possibly grown hash-table pointer for subsequent inserts

    // -- advance to the next source slot --
    emitter.instruction("ldr x9, [sp, #32]");                                   // reload the current loop index
    emitter.instruction("add x9, x9, #1");                                      // advance to the next source slot
    emitter.instruction("str x9, [sp, #32]");                                   // save the updated loop index
    emitter.instruction("b __rt_array_to_hash_loop");                           // continue promoting subsequent source slots

    // -- tear down stack frame and return the promoted hash pointer --
    emitter.label("__rt_array_to_hash_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // return the promoted hash-table pointer
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // release the promotion spill slots
    emitter.instruction("ret");                                                 // return to generated code
}

fn emit_array_to_hash_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_to_hash ---");
    emitter.label_global("__rt_array_to_hash");

    // Stack layout (after `push rbp` + `sub rsp, 64`):
    //   [rbp - 8]  = source indexed array pointer (saved across helper calls)
    //   [rbp - 16] = evolving hash table pointer
    //   [rbp - 24] = source value_type tag
    //   [rbp - 32] = source element count
    //   [rbp - 40] = loop index i
    //   [rbp - 48] = staged value_lo for __rt_hash_set
    //   [rbp - 56] = staged value_hi for __rt_hash_set
    //   [rbp - 64] = staged value_tag for __rt_hash_set
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving promotion spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the saved source pointer and staged value
    emitter.instruction("sub rsp, 64");                                         // reserve local storage while keeping nested calls 16-byte aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the source indexed-array pointer for slot loads
    emitter.instruction("mov rax, QWORD PTR [rdi]");                            // load the indexed-array length from header[0]
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the slot count for the promotion loop
    emitter.instruction("mov r10, QWORD PTR [rdi - 8]");                        // load the packed heap-kind word that carries the value_type tag
    emitter.instruction("shr r10, 8");                                          // shift the value_type tag into the low bits
    emitter.instruction("and r10, 0x7f");                                       // isolate the source value_type tag for dispatch
    emitter.instruction("mov QWORD PTR [rbp - 24], r10");                       // save the source value_type tag for the promotion loop

    // -- allocate a fresh hash table sized to avoid mid-fill growth --
    emitter.instruction("mov rdi, rax");                                        // start with capacity = count
    emitter.instruction("shl rdi, 1");                                          // multiply capacity by two to stay well under the 75% load factor
    emitter.instruction("add rdi, 8");                                          // add a small floor so empty/short arrays still get a usable table
    emitter.instruction("mov rsi, r10");                                        // pass the source value_type tag through to the new hash
    emitter.instruction("call __rt_hash_new");                                  // allocate a fresh hash with refcount 1 to receive the promoted entries
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the evolving hash-table pointer for hash_set calls
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // initialize the promotion loop index i = 0

    // -- promotion loop: insert each indexed slot under integer key i --
    emitter.label("__rt_array_to_hash_x86_loop");
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the current loop index
    emitter.instruction("mov r11, QWORD PTR [rbp - 32]");                       // reload the source slot count
    emitter.instruction("cmp r10, r11");                                        // have all source slots been promoted?
    emitter.instruction("jae __rt_array_to_hash_x86_done");                     // finish once every source slot has been inserted

    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the source value_type tag
    emitter.instruction("cmp r10, 1");                                          // is the source storing string slots (16-byte ptr+len cells)?
    emitter.instruction("je __rt_array_to_hash_x86_str_value");                 // string slots need pointer+length loads plus persistence
    emitter.instruction("cmp r10, 4");                                          // is the source payload in the refcounted tag range?
    emitter.instruction("jb __rt_array_to_hash_x86_scalar_value");              // scalar tags (int/float/bool) copy directly without retention
    emitter.instruction("cmp r10, 7");                                          // is the source payload still a supported refcounted tag?
    emitter.instruction("ja __rt_array_to_hash_x86_scalar_value");              // unknown high tags fall back to scalar copying

    // -- refcounted payloads (tags 4..=7): incref before transferring ownership --
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the current loop index for the slot address calculation
    emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                        // reload the source indexed-array pointer
    emitter.instruction("mov rax, QWORD PTR [r11 + 24 + r10 * 8]");             // load the refcounted heap pointer at slot i
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // stage the heap pointer as the hash value_lo word
    emitter.instruction("call __rt_incref");                                    // retain the heap payload so the hash owns its reference
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // refcounted payloads use a single word; clear the high word
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the source value_type tag for the per-entry stored tag
    emitter.instruction("mov QWORD PTR [rbp - 64], r10");                       // stage the refcounted tag as the hash value_tag word
    emitter.instruction("jmp __rt_array_to_hash_x86_insert");                   // proceed to the shared hash_set insertion path

    // -- string payloads (tag 1): persist the borrowed bytes for hash ownership --
    emitter.label("__rt_array_to_hash_x86_str_value");
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the current loop index
    emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                        // reload the source indexed-array pointer
    emitter.instruction("shl r10, 4");                                          // compute the byte offset for a 16-byte string slot
    emitter.instruction("lea r11, [r11 + r10 + 24]");                           // address the string slot inside the indexed-array payload
    emitter.instruction("mov rax, QWORD PTR [r11]");                            // load the borrowed string pointer
    emitter.instruction("mov rdx, QWORD PTR [r11 + 8]");                        // load the borrowed string length
    emitter.instruction("call __rt_str_persist");                               // duplicate the string so the hash owns its bytes
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // stage the owned string pointer as the hash value_lo word
    emitter.instruction("mov QWORD PTR [rbp - 56], rdx");                       // stage the owned string length as the hash value_hi word
    emitter.instruction("mov QWORD PTR [rbp - 64], 1");                         // stage the string runtime tag as the hash value_tag word
    emitter.instruction("jmp __rt_array_to_hash_x86_insert");                   // proceed to the shared hash_set insertion path

    // -- scalar payloads (tags 0/2/3 and unknown high tags): copy directly --
    emitter.label("__rt_array_to_hash_x86_scalar_value");
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the current loop index for the slot address calculation
    emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                        // reload the source indexed-array pointer
    emitter.instruction("mov rax, QWORD PTR [r11 + 24 + r10 * 8]");             // load the scalar payload at slot i
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // stage the scalar payload as the hash value_lo word
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // scalar payloads use a single word; clear the high word
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the source value_type tag
    emitter.instruction("mov QWORD PTR [rbp - 64], r10");                       // stage the scalar runtime tag as the hash value_tag word

    // -- insert the staged payload under integer key i in the promoted hash --
    emitter.label("__rt_array_to_hash_x86_insert");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the evolving hash-table pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 40]");                       // pass the current index i as the integer key low word
    emitter.instruction("mov rdx, -1");                                         // rdx = -1 marks the key as an integer key in __rt_hash_set
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // pass the staged hash value_lo word
    emitter.instruction("mov r8, QWORD PTR [rbp - 56]");                        // pass the staged hash value_hi word
    emitter.instruction("mov r9, QWORD PTR [rbp - 64]");                        // pass the staged hash value_tag word
    emitter.instruction("call __rt_hash_set");                                  // insert the integer-keyed entry into the promoted hash
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the possibly grown hash-table pointer for subsequent inserts

    // -- advance to the next source slot --
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the current loop index
    emitter.instruction("add r10, 1");                                          // advance to the next source slot
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // save the updated loop index
    emitter.instruction("jmp __rt_array_to_hash_x86_loop");                     // continue promoting subsequent source slots

    // -- tear down stack frame and return the promoted hash pointer --
    emitter.label("__rt_array_to_hash_x86_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return the promoted hash-table pointer
    emitter.instruction("add rsp, 64");                                         // release the promotion spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to generated code
}
