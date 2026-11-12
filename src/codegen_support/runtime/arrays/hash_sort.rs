//! Purpose:
//! Emits `__rt_hash_sort`, the stable key/value sort behind `ksort`, `krsort`, `asort` and
//! `arsort` on associative arrays.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - `crate::codegen::lower_inst::builtins::arrays` (hash receivers of the four sorts).
//!
//! Key details:
//! - The caller passes a unique (copy-on-write split) table. Entries keep their slots and
//!   key/value payloads; only the insertion-order chain (`head`/`tail`, per-entry
//!   `prev`/`next`) is rewritten, so keys are preserved like PHP.
//! - Each entry's key (int key → tag 0, string key → tag 1) or value is boxed once into a
//!   Mixed cell, then `(slot, box)` records are merge-sorted bottom-up. Merge sort is stable,
//!   matching PHP 8's stable sorts; descending order compares `(right, left)` so ties keep
//!   their original order. Comparisons use `__rt_php_compare` (PHP 8 `<=>` rules) with an
//!   inline fast path when both boxes hold ints.
//! - Each comparison runs between a save and a restore of `_concat_off`, so the scratch
//!   strings produced by stringifying operands do not accumulate over n·log n comparisons.
//! - Mode bits: bit 0 = sort by value (else by key), bit 1 = descending, bit 2 = `SORT_STRING`
//!   (compare the string forms byte-wise through `__rt_hash_sort_strcmp`).

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Mode bit selecting a value sort (`asort`/`arsort`); clear for key sorts.
pub(crate) const HASH_SORT_BY_VALUE: i64 = 1;
/// Mode bit selecting descending order (`krsort`/`arsort`).
pub(crate) const HASH_SORT_DESCENDING: i64 = 2;
/// Mode bit selecting `SORT_STRING` comparison (byte-wise on the string forms).
pub(crate) const HASH_SORT_STRING: i64 = 4;

/// Emits `__rt_hash_sort` and its merge helper.
///
/// - AArch64: x0 = unique hash table, x1 = mode.
/// - x86_64: rdi = unique hash table, rsi = mode.
pub fn emit_hash_sort(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_hash_sort_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hash_sort ---");
    emitter.label_global("__rt_hash_sort");
    // frame: [0] mode, [8] dst records, [16..64] x19-x24, [64] x29/x30
    emitter.instruction("sub sp, sp, #80");                                     // allocate the sort frame
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #64");                                    // establish frame pointer
    emitter.instruction("stp x19, x20, [sp, #16]");                             // save callee-saved registers
    emitter.instruction("stp x21, x22, [sp, #32]");                             // save callee-saved registers
    emitter.instruction("stp x23, x24, [sp, #48]");                             // save callee-saved registers
    emitter.instruction("mov x19, x0");                                         // x19 = hash table
    emitter.instruction("str x1, [sp, #0]");                                    // save mode
    emitter.instruction("cbz x19, __rt_hash_sort_ret");                         // null table: nothing to sort
    emitter.instruction("ldr x21, [x19]");                                      // x21 = live entry count
    emitter.instruction("cmp x21, #2");                                         // fewer than two entries?
    emitter.instruction("b.lt __rt_hash_sort_ret");                             // already sorted

    // -- allocate the record buffers: 16 bytes (slot, box) per entry --
    emitter.instruction("lsl x0, x21, #4");                                     // record buffer size
    emitter.instruction("bl __rt_heap_alloc");                                  // source records
    emitter.instruction("mov x20, x0");                                         // x20 = source records
    emitter.instruction("lsl x0, x21, #4");                                     // record buffer size
    emitter.instruction("bl __rt_heap_alloc");                                  // destination records
    emitter.instruction("mov x24, x0");                                         // x24 = destination records

    // -- collect (slot, boxed key or value) in insertion order --
    emitter.instruction("ldr x22, [x19, #24]");                                 // x22 = head slot
    emitter.instruction("mov x23, #0");                                         // x23 = record index
    emitter.label("__rt_hash_sort_collect");
    emitter.instruction("cmn x22, #1");                                         // end of the insertion chain?
    emitter.instruction("b.eq __rt_hash_sort_collected");                       // all entries collected
    emitter.instruction("add x10, x19, x22, lsl #6");                           // table + slot * 64
    emitter.instruction("add x10, x10, #40");                                   // entry address
    emitter.instruction("ldr x11, [sp, #0]");                                   // mode
    emitter.instruction(&format!("tst x11, #{}", HASH_SORT_BY_VALUE));          // value sort?
    emitter.instruction("b.ne __rt_hash_sort_collect_value");                   // box the value
    emitter.instruction("ldr x1, [x10, #8]");                                   // key low word
    emitter.instruction("ldr x2, [x10, #16]");                                  // key high word (-1 = int key)
    emitter.instruction("cmn x2, #1");                                          // integer key?
    emitter.instruction("cset x0, ne");                                         // tag 0 = int, 1 = string
    emitter.instruction("b __rt_hash_sort_collect_box");                        // box the key
    emitter.label("__rt_hash_sort_collect_value");
    emitter.instruction("ldr x1, [x10, #24]");                                  // value low word
    emitter.instruction("ldr x2, [x10, #32]");                                  // value high word
    emitter.instruction("ldr x0, [x10, #40]");                                  // value tag
    emitter.label("__rt_hash_sort_collect_box");
    emitter.instruction("bl __rt_mixed_from_value");                            // x0 = boxed sort operand
    emitter.instruction("add x11, x20, x23, lsl #4");                           // record address
    emitter.instruction("stp x22, x0, [x11]");                                  // record = (slot, box)
    emitter.instruction("add x10, x19, x22, lsl #6");                           // table + slot * 64
    emitter.instruction("ldr x22, [x10, #96]");                                 // next slot (entry + 56)
    emitter.instruction("add x23, x23, #1");                                    // next record
    emitter.instruction("b __rt_hash_sort_collect");                            // continue the walk
    emitter.label("__rt_hash_sort_collected");
    emitter.instruction("mov x21, x23");                                        // x21 = record count

    // -- bottom-up merge sort: x22 = run width, x23 = run start --
    emitter.instruction("mov x22, #1");                                         // width = 1
    emitter.label("__rt_hash_sort_pass");
    emitter.instruction("cmp x22, x21");                                        // width covers everything?
    emitter.instruction("b.ge __rt_hash_sort_relink");                          // sorted
    emitter.instruction("mov x23, #0");                                         // run start = 0
    emitter.label("__rt_hash_sort_run");
    emitter.instruction("cmp x23, x21");                                        // past the last run?
    emitter.instruction("b.ge __rt_hash_sort_pass_done");                       // finish this pass
    emitter.instruction("add x3, x23, x22");                                    // mid = start + width
    emitter.instruction("cmp x3, x21");                                         // clamp mid
    emitter.instruction("csel x3, x3, x21, lt");                                // mid = min(mid, n)
    emitter.instruction("add x4, x3, x22");                                     // hi = mid + width
    emitter.instruction("cmp x4, x21");                                         // clamp hi
    emitter.instruction("csel x4, x4, x21, lt");                                // hi = min(hi, n)
    emitter.instruction("mov x0, x20");                                         // source records
    emitter.instruction("mov x1, x24");                                         // destination records
    emitter.instruction("mov x2, x23");                                         // lo
    emitter.instruction("ldr x5, [sp, #0]");                                    // mode
    emitter.instruction("bl __rt_hash_sort_merge");                             // merge [lo, mid) and [mid, hi)
    emitter.instruction("add x23, x23, x22, lsl #1");                           // next run start
    emitter.instruction("b __rt_hash_sort_run");                                // merge the next pair
    emitter.label("__rt_hash_sort_pass_done");
    emitter.instruction("mov x9, x20");                                         // swap source and destination
    emitter.instruction("mov x20, x24");                                        // sorted runs become the source
    emitter.instruction("mov x24, x9");                                         // old source becomes scratch
    emitter.instruction("lsl x22, x22, #1");                                    // double the width
    emitter.instruction("b __rt_hash_sort_pass");                               // next pass

    // -- relink the insertion-order chain in sorted order --
    emitter.label("__rt_hash_sort_relink");
    emitter.instruction("ldr x9, [x20]");                                       // first slot
    emitter.instruction("str x9, [x19, #24]");                                  // head = first slot
    emitter.instruction("sub x10, x21, #1");                                    // last record index
    emitter.instruction("add x10, x20, x10, lsl #4");                           // last record
    emitter.instruction("ldr x9, [x10]");                                       // last slot
    emitter.instruction("str x9, [x19, #32]");                                  // tail = last slot
    emitter.instruction("mov x23, #0");                                         // record index
    emitter.instruction("mov x12, #-1");                                        // previous slot = none
    emitter.label("__rt_hash_sort_relink_loop");
    emitter.instruction("cmp x23, x21");                                        // all records relinked?
    emitter.instruction("b.ge __rt_hash_sort_release");                         // yes
    emitter.instruction("add x10, x20, x23, lsl #4");                           // record address
    emitter.instruction("ldr x9, [x10]");                                       // this slot
    emitter.instruction("mov x13, #-1");                                        // next slot = none by default
    emitter.instruction("add x14, x23, #1");                                    // next record index
    emitter.instruction("cmp x14, x21");                                        // is there a next record?
    emitter.instruction("b.ge __rt_hash_sort_relink_store");                    // no
    emitter.instruction("ldr x13, [x10, #16]");                                 // next record slot
    emitter.label("__rt_hash_sort_relink_store");
    emitter.instruction("add x11, x19, x9, lsl #6");                            // table + slot * 64
    emitter.instruction("add x11, x11, #40");                                   // entry address
    emitter.instruction("str x12, [x11, #48]");                                 // entry.prev
    emitter.instruction("str x13, [x11, #56]");                                 // entry.next
    emitter.instruction("mov x12, x9");                                         // this slot is the next prev
    emitter.instruction("add x23, x23, #1");                                    // next record
    emitter.instruction("b __rt_hash_sort_relink_loop");                        // continue

    // -- release the boxed operands and both record buffers --
    emitter.label("__rt_hash_sort_release");
    emitter.instruction("mov x23, #0");                                         // record index
    emitter.label("__rt_hash_sort_release_loop");
    emitter.instruction("cmp x23, x21");                                        // all boxes released?
    emitter.instruction("b.ge __rt_hash_sort_free");                            // yes
    emitter.instruction("add x10, x20, x23, lsl #4");                           // record address
    emitter.instruction("ldr x0, [x10, #8]");                                   // boxed operand
    emitter.instruction("bl __rt_decref_mixed");                                // release the box and its payload reference
    emitter.instruction("add x23, x23, #1");                                    // next record
    emitter.instruction("b __rt_hash_sort_release_loop");                       // continue
    emitter.label("__rt_hash_sort_free");
    emitter.instruction("mov x0, x20");                                         // sorted records
    emitter.instruction("bl __rt_heap_free");                                   // free
    emitter.instruction("mov x0, x24");                                         // scratch records
    emitter.instruction("bl __rt_heap_free");                                   // free

    emitter.label("__rt_hash_sort_ret");
    emitter.instruction("ldp x19, x20, [sp, #16]");                             // restore callee-saved registers
    emitter.instruction("ldp x21, x22, [sp, #32]");                             // restore callee-saved registers
    emitter.instruction("ldp x23, x24, [sp, #48]");                             // restore callee-saved registers
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // release the frame
    emitter.instruction("ret");                                                 // return

    // -- merge(x0 src, x1 dst, x2 lo, x3 mid, x4 hi, x5 mode): stable, records are 16 bytes --
    emitter.blank();
    emitter.label("__rt_hash_sort_merge");
    emitter.instruction("sub sp, sp, #96");                                     // frame for callee-saved registers and the scratch offset
    emitter.instruction("stp x29, x30, [sp, #80]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #80");                                    // establish frame pointer
    emitter.instruction("stp x19, x20, [sp, #0]");                              // save callee-saved registers
    emitter.instruction("stp x21, x22, [sp, #16]");                             // save callee-saved registers
    emitter.instruction("stp x23, x24, [sp, #32]");                             // save callee-saved registers
    emitter.instruction("stp x25, x26, [sp, #48]");                             // save callee-saved registers
    emitter.instruction("mov x19, x0");                                         // src
    emitter.instruction("mov x20, x1");                                         // dst
    emitter.instruction("mov x21, x2");                                         // i = lo
    emitter.instruction("mov x22, x3");                                         // mid
    emitter.instruction("mov x23, x3");                                         // j = mid
    emitter.instruction("mov x24, x4");                                         // hi
    emitter.instruction("mov x25, x2");                                         // k = lo
    emitter.instruction("mov x26, x5");                                         // mode
    emitter.label("__rt_hash_sort_merge_loop");
    emitter.instruction("cmp x21, x22");                                        // left run exhausted?
    emitter.instruction("b.ge __rt_hash_sort_merge_right");                     // copy the rest of the right run
    emitter.instruction("cmp x23, x24");                                        // right run exhausted?
    emitter.instruction("b.ge __rt_hash_sort_merge_left");                      // copy the rest of the left run
    emitter.instruction("add x9, x19, x21, lsl #4");                            // left record
    emitter.instruction("ldr x0, [x9, #8]");                                    // left box
    emitter.instruction("add x9, x19, x23, lsl #4");                            // right record
    emitter.instruction("ldr x1, [x9, #8]");                                    // right box
    emitter.instruction(&format!("tst x26, #{}", HASH_SORT_DESCENDING));        // descending?
    emitter.instruction("b.eq __rt_hash_sort_merge_cmp");                       // ascending: compare (left, right)
    emitter.instruction("mov x9, x0");                                          // swap the operands
    emitter.instruction("mov x0, x1");                                          // descending compares (right, left)
    emitter.instruction("mov x1, x9");                                          // so ties keep their order
    emitter.label("__rt_hash_sort_merge_cmp");
    // Comparisons may stringify operands into the concat scratch: rewind it afterwards.
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [x9]");                                       // scratch offset before the comparison
    emitter.instruction("str x10, [sp, #64]");                                  // save it
    emitter.instruction(&format!("tst x26, #{}", HASH_SORT_STRING));            // SORT_STRING?
    emitter.instruction("b.eq __rt_hash_sort_merge_regular");                   // no: SORT_REGULAR rules
    emitter.instruction("bl __rt_hash_sort_strcmp");                            // x0 = -1/0/+1 on the string forms
    emitter.instruction("b __rt_hash_sort_merge_pick");                         // pick the next record
    emitter.label("__rt_hash_sort_merge_regular");
    emitter.instruction("ldr x9, [x0]");                                        // left box tag
    emitter.instruction("ldr x10, [x1]");                                       // right box tag
    emitter.instruction("orr x11, x9, x10");                                    // both int (tag 0)?
    emitter.instruction("cbnz x11, __rt_hash_sort_merge_php_cmp");              // no: PHP comparison
    emitter.instruction("ldr x9, [x0, #8]");                                    // left int
    emitter.instruction("ldr x10, [x1, #8]");                                   // right int
    emitter.instruction("cmp x9, x10");                                         // compare ints
    emitter.instruction("cset x0, gt");                                         // 1 when left > right
    emitter.instruction("b __rt_hash_sort_merge_pick");                         // pick the next record
    emitter.label("__rt_hash_sort_merge_php_cmp");
    emitter.instruction("bl __rt_php_compare");                                 // x0 = -1/0/+1
    emitter.label("__rt_hash_sort_merge_pick");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [sp, #64]");                                  // scratch offset before the comparison
    emitter.instruction("str x10, [x9]");                                       // drop the comparison's scratch strings
    emitter.instruction("cmp x0, #0");                                          // left after right?
    emitter.instruction("b.gt __rt_hash_sort_merge_take_right");                // yes: take the right record
    emitter.instruction("add x9, x19, x21, lsl #4");                            // left record
    emitter.instruction("add x21, x21, #1");                                    // i++
    emitter.instruction("b __rt_hash_sort_merge_store");                        // store it
    emitter.label("__rt_hash_sort_merge_take_right");
    emitter.instruction("add x9, x19, x23, lsl #4");                            // right record
    emitter.instruction("add x23, x23, #1");                                    // j++
    emitter.label("__rt_hash_sort_merge_store");
    emitter.instruction("ldp x10, x11, [x9]");                                  // record
    emitter.instruction("add x12, x20, x25, lsl #4");                           // destination record
    emitter.instruction("stp x10, x11, [x12]");                                 // store it
    emitter.instruction("add x25, x25, #1");                                    // k++
    emitter.instruction("b __rt_hash_sort_merge_loop");                         // continue
    emitter.label("__rt_hash_sort_merge_left");
    emitter.instruction("cmp x21, x22");                                        // left records left?
    emitter.instruction("b.ge __rt_hash_sort_merge_right");                     // no
    emitter.instruction("add x9, x19, x21, lsl #4");                            // left record
    emitter.instruction("ldp x10, x11, [x9]");                                  // record
    emitter.instruction("add x12, x20, x25, lsl #4");                           // destination record
    emitter.instruction("stp x10, x11, [x12]");                                 // store it
    emitter.instruction("add x21, x21, #1");                                    // i++
    emitter.instruction("add x25, x25, #1");                                    // k++
    emitter.instruction("b __rt_hash_sort_merge_left");                         // continue
    emitter.label("__rt_hash_sort_merge_right");
    emitter.instruction("cmp x23, x24");                                        // right records left?
    emitter.instruction("b.ge __rt_hash_sort_merge_done");                      // no
    emitter.instruction("add x9, x19, x23, lsl #4");                            // right record
    emitter.instruction("ldp x10, x11, [x9]");                                  // record
    emitter.instruction("add x12, x20, x25, lsl #4");                           // destination record
    emitter.instruction("stp x10, x11, [x12]");                                 // store it
    emitter.instruction("add x23, x23, #1");                                    // j++
    emitter.instruction("add x25, x25, #1");                                    // k++
    emitter.instruction("b __rt_hash_sort_merge_right");                        // continue
    emitter.label("__rt_hash_sort_merge_done");
    emitter.instruction("ldp x19, x20, [sp, #0]");                              // restore callee-saved registers
    emitter.instruction("ldp x21, x22, [sp, #16]");                             // restore callee-saved registers
    emitter.instruction("ldp x23, x24, [sp, #32]");                             // restore callee-saved registers
    emitter.instruction("ldp x25, x26, [sp, #48]");                             // restore callee-saved registers
    emitter.instruction("ldp x29, x30, [sp, #80]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #96");                                     // release the frame
    emitter.instruction("ret");                                                 // return to the sort driver

    // -- strcmp(x0 left box, x1 right box): SORT_STRING sign of the stringified operands --
    emitter.blank();
    emitter.label("__rt_hash_sort_strcmp");
    emitter.instruction("sub sp, sp, #48");                                     // frame: right box, left string, right pointer, sign
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the right box
    emitter.instruction("bl __rt_mixed_cast_string");                           // x1/x2 = left string
    emitter.instruction("stp x1, x2, [sp, #8]");                                // save the left string
    emitter.instruction("ldr x0, [sp, #0]");                                    // right box
    emitter.instruction("bl __rt_mixed_cast_string");                           // x1/x2 = right string
    emitter.instruction("str x1, [sp, #0]");                                    // keep the right pointer for its release
    emitter.instruction("mov x3, x1");                                          // right pointer
    emitter.instruction("mov x4, x2");                                          // right length
    emitter.instruction("ldp x1, x2, [sp, #8]");                                // left string
    emitter.instruction("bl __rt_strcmp");                                      // x0 = byte/length difference
    emitter.instruction("cmp x0, #0");                                          // normalize to a sign
    emitter.instruction("cset x0, gt");                                         // 1 when left > right
    emitter.instruction("csinv x0, x0, xzr, ge");                               // -1 when left < right
    emitter.instruction("str x0, [sp, #24]");                                   // keep the sign across the releases
    emitter.instruction("ldr x0, [sp, #8]");                                    // left string
    emitter.instruction("bl __rt_decref_any");                                  // release the owned copy (scratch strings are skipped)
    emitter.instruction("ldr x0, [sp, #0]");                                    // right string
    emitter.instruction("bl __rt_decref_any");                                  // release the owned copy (scratch strings are skipped)
    emitter.instruction("ldr x0, [sp, #24]");                                   // sign
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the frame
    emitter.instruction("ret");                                                 // return the sign
}

/// Emits the x86_64 `__rt_hash_sort` (see [`emit_hash_sort`]).
fn emit_hash_sort_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_sort ---");
    emitter.label_global("__rt_hash_sort");
    // frame: rbx, r12-r15 pushed; [rbp - 48] mode, [rbp - 56] dst records
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame pointer
    emitter.instruction("push rbx");                                            // callee-saved: hash table
    emitter.instruction("push r12");                                            // callee-saved: source records
    emitter.instruction("push r13");                                            // callee-saved: record count
    emitter.instruction("push r14");                                            // callee-saved: width / slot cursor
    emitter.instruction("push r15");                                            // callee-saved: run start / record index
    emitter.instruction("sub rsp, 24");                                         // mode, destination records (keeps 16-byte alignment)
    emitter.instruction("mov rbx, rdi");                                        // rbx = hash table
    emitter.instruction("mov QWORD PTR [rbp - 48], rsi");                       // save mode
    emitter.instruction("test rbx, rbx");                                       // null table?
    emitter.instruction("jz __rt_hash_sort_ret");                               // nothing to sort
    emitter.instruction("mov r13, QWORD PTR [rbx]");                            // r13 = live entry count
    emitter.instruction("cmp r13, 2");                                          // fewer than two entries?
    emitter.instruction("jl __rt_hash_sort_ret");                               // already sorted

    // -- allocate the record buffers: 16 bytes (slot, box) per entry --
    emitter.instruction("mov rax, r13");                                        // entry count
    emitter.instruction("shl rax, 4");                                          // record buffer size
    emitter.instruction("call __rt_heap_alloc");                                // source records
    emitter.instruction("mov r12, rax");                                        // r12 = source records
    emitter.instruction("mov rax, r13");                                        // entry count
    emitter.instruction("shl rax, 4");                                          // record buffer size
    emitter.instruction("call __rt_heap_alloc");                                // destination records
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // save destination records

    // -- collect (slot, boxed key or value) in insertion order --
    emitter.instruction("mov r14, QWORD PTR [rbx + 24]");                       // r14 = head slot
    emitter.instruction("xor r15d, r15d");                                      // r15 = record index
    emitter.label("__rt_hash_sort_collect");
    emitter.instruction("cmp r14, -1");                                         // end of the insertion chain?
    emitter.instruction("je __rt_hash_sort_collected");                         // all entries collected
    emitter.instruction("mov r10, r14");                                        // slot
    emitter.instruction("shl r10, 6");                                          // slot * 64
    emitter.instruction("lea r10, [rbx + r10 + 40]");                           // entry address
    emitter.instruction(&format!("test QWORD PTR [rbp - 48], {}", HASH_SORT_BY_VALUE)); // value sort?
    emitter.instruction("jnz __rt_hash_sort_collect_value");                    // box the value
    emitter.instruction("mov rdi, QWORD PTR [r10 + 8]");                        // key low word
    emitter.instruction("mov rsi, QWORD PTR [r10 + 16]");                       // key high word (-1 = int key)
    emitter.instruction("xor eax, eax");                                        // tag 0 = int
    emitter.instruction("cmp rsi, -1");                                         // integer key?
    emitter.instruction("setne al");                                            // tag 1 = string
    emitter.instruction("jmp __rt_hash_sort_collect_box");                      // box the key
    emitter.label("__rt_hash_sort_collect_value");
    emitter.instruction("mov rdi, QWORD PTR [r10 + 24]");                       // value low word
    emitter.instruction("mov rsi, QWORD PTR [r10 + 32]");                       // value high word
    emitter.instruction("mov rax, QWORD PTR [r10 + 40]");                       // value tag
    emitter.label("__rt_hash_sort_collect_box");
    emitter.instruction("call __rt_mixed_from_value");                          // rax = boxed sort operand
    emitter.instruction("mov r10, r15");                                        // record index
    emitter.instruction("shl r10, 4");                                          // record offset
    emitter.instruction("mov QWORD PTR [r12 + r10], r14");                      // record.slot
    emitter.instruction("mov QWORD PTR [r12 + r10 + 8], rax");                  // record.box
    emitter.instruction("mov r10, r14");                                        // slot
    emitter.instruction("shl r10, 6");                                          // slot * 64
    emitter.instruction("mov r14, QWORD PTR [rbx + r10 + 96]");                 // next slot (entry + 56)
    emitter.instruction("add r15, 1");                                          // next record
    emitter.instruction("jmp __rt_hash_sort_collect");                          // continue the walk
    emitter.label("__rt_hash_sort_collected");
    emitter.instruction("mov r13, r15");                                        // r13 = record count

    // -- bottom-up merge sort: r14 = run width, r15 = run start --
    emitter.instruction("mov r14, 1");                                          // width = 1
    emitter.label("__rt_hash_sort_pass");
    emitter.instruction("cmp r14, r13");                                        // width covers everything?
    emitter.instruction("jge __rt_hash_sort_relink");                           // sorted
    emitter.instruction("xor r15d, r15d");                                      // run start = 0
    emitter.label("__rt_hash_sort_run");
    emitter.instruction("cmp r15, r13");                                        // past the last run?
    emitter.instruction("jge __rt_hash_sort_pass_done");                        // finish this pass
    emitter.instruction("lea rcx, [r15 + r14]");                                // mid = start + width
    emitter.instruction("cmp rcx, r13");                                        // clamp mid
    emitter.instruction("cmovg rcx, r13");                                      // mid = min(mid, n)
    emitter.instruction("lea r8, [rcx + r14]");                                 // hi = mid + width
    emitter.instruction("cmp r8, r13");                                         // clamp hi
    emitter.instruction("cmovg r8, r13");                                       // hi = min(hi, n)
    emitter.instruction("mov rdi, r12");                                        // source records
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // destination records
    emitter.instruction("mov rdx, r15");                                        // lo
    emitter.instruction("mov r9, QWORD PTR [rbp - 48]");                        // mode
    emitter.instruction("call __rt_hash_sort_merge");                           // merge [lo, mid) and [mid, hi)
    emitter.instruction("lea r15, [r15 + r14 * 2]");                            // next run start
    emitter.instruction("jmp __rt_hash_sort_run");                              // merge the next pair
    emitter.label("__rt_hash_sort_pass_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                       // sorted runs
    emitter.instruction("mov QWORD PTR [rbp - 56], r12");                       // old source becomes scratch
    emitter.instruction("mov r12, rax");                                        // sorted runs become the source
    emitter.instruction("add r14, r14");                                        // double the width
    emitter.instruction("jmp __rt_hash_sort_pass");                             // next pass

    // -- relink the insertion-order chain in sorted order --
    emitter.label("__rt_hash_sort_relink");
    emitter.instruction("mov rax, QWORD PTR [r12]");                            // first slot
    emitter.instruction("mov QWORD PTR [rbx + 24], rax");                       // head = first slot
    emitter.instruction("mov rax, r13");                                        // record count
    emitter.instruction("shl rax, 4");                                          // past-the-end offset
    emitter.instruction("mov rax, QWORD PTR [r12 + rax - 16]");                 // last slot
    emitter.instruction("mov QWORD PTR [rbx + 32], rax");                       // tail = last slot
    emitter.instruction("xor r15d, r15d");                                      // record index
    emitter.instruction("mov rdx, -1");                                         // previous slot = none
    emitter.label("__rt_hash_sort_relink_loop");
    emitter.instruction("cmp r15, r13");                                        // all records relinked?
    emitter.instruction("jge __rt_hash_sort_release");                          // yes
    emitter.instruction("mov r10, r15");                                        // record index
    emitter.instruction("shl r10, 4");                                          // record offset
    emitter.instruction("mov rax, QWORD PTR [r12 + r10]");                      // this slot
    emitter.instruction("mov rcx, -1");                                         // next slot = none by default
    emitter.instruction("lea r11, [r15 + 1]");                                  // next record index
    emitter.instruction("cmp r11, r13");                                        // is there a next record?
    emitter.instruction("jge __rt_hash_sort_relink_store");                     // no
    emitter.instruction("mov rcx, QWORD PTR [r12 + r10 + 16]");                 // next record slot
    emitter.label("__rt_hash_sort_relink_store");
    emitter.instruction("mov r11, rax");                                        // slot
    emitter.instruction("shl r11, 6");                                          // slot * 64
    emitter.instruction("mov QWORD PTR [rbx + r11 + 88], rdx");                 // entry.prev (entry + 48)
    emitter.instruction("mov QWORD PTR [rbx + r11 + 96], rcx");                 // entry.next (entry + 56)
    emitter.instruction("mov rdx, rax");                                        // this slot is the next prev
    emitter.instruction("add r15, 1");                                          // next record
    emitter.instruction("jmp __rt_hash_sort_relink_loop");                      // continue

    // -- release the boxed operands and both record buffers --
    emitter.label("__rt_hash_sort_release");
    emitter.instruction("xor r15d, r15d");                                      // record index
    emitter.label("__rt_hash_sort_release_loop");
    emitter.instruction("cmp r15, r13");                                        // all boxes released?
    emitter.instruction("jge __rt_hash_sort_free");                             // yes
    emitter.instruction("mov r10, r15");                                        // record index
    emitter.instruction("shl r10, 4");                                          // record offset
    emitter.instruction("mov rax, QWORD PTR [r12 + r10 + 8]");                  // boxed operand
    emitter.instruction("call __rt_decref_mixed");                              // release the box and its payload reference
    emitter.instruction("add r15, 1");                                          // next record
    emitter.instruction("jmp __rt_hash_sort_release_loop");                     // continue
    emitter.label("__rt_hash_sort_free");
    emitter.instruction("mov rax, r12");                                        // sorted records
    emitter.instruction("call __rt_heap_free");                                 // free
    emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                       // scratch records
    emitter.instruction("call __rt_heap_free");                                 // free

    emitter.label("__rt_hash_sort_ret");
    emitter.instruction("add rsp, 24");                                         // release the locals
    emitter.instruction("pop r15");                                             // restore callee-saved registers
    emitter.instruction("pop r14");                                             // restore callee-saved registers
    emitter.instruction("pop r13");                                             // restore callee-saved registers
    emitter.instruction("pop r12");                                             // restore callee-saved registers
    emitter.instruction("pop rbx");                                             // restore callee-saved registers
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return

    // -- merge(rdi src, rsi dst, rdx lo, rcx mid, r8 hi, r9 mode): stable, 16-byte records --
    emitter.blank();
    emitter.label("__rt_hash_sort_merge");
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame pointer
    emitter.instruction("push rbx");                                            // callee-saved: src
    emitter.instruction("push r12");                                            // callee-saved: dst
    emitter.instruction("push r13");                                            // callee-saved: i
    emitter.instruction("push r14");                                            // callee-saved: j
    emitter.instruction("push r15");                                            // callee-saved: k
    emitter.instruction("sub rsp, 40");                                         // mid, hi, mode, scratch offset (keeps 16-byte alignment)
    emitter.instruction("mov rbx, rdi");                                        // src
    emitter.instruction("mov r12, rsi");                                        // dst
    emitter.instruction("mov r13, rdx");                                        // i = lo
    emitter.instruction("mov r14, rcx");                                        // j = mid
    emitter.instruction("mov r15, rdx");                                        // k = lo
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // mid
    emitter.instruction("mov QWORD PTR [rbp - 56], r8");                        // hi
    emitter.instruction("mov QWORD PTR [rbp - 64], r9");                        // mode
    emitter.label("__rt_hash_sort_merge_loop");
    emitter.instruction("cmp r13, QWORD PTR [rbp - 48]");                       // left run exhausted?
    emitter.instruction("jge __rt_hash_sort_merge_right");                      // copy the rest of the right run
    emitter.instruction("cmp r14, QWORD PTR [rbp - 56]");                       // right run exhausted?
    emitter.instruction("jge __rt_hash_sort_merge_left");                       // copy the rest of the left run
    emitter.instruction("mov r10, r13");                                        // i
    emitter.instruction("shl r10, 4");                                          // left record offset
    emitter.instruction("mov rdi, QWORD PTR [rbx + r10 + 8]");                  // left box
    emitter.instruction("mov r10, r14");                                        // j
    emitter.instruction("shl r10, 4");                                          // right record offset
    emitter.instruction("mov rsi, QWORD PTR [rbx + r10 + 8]");                  // right box
    emitter.instruction(&format!("test QWORD PTR [rbp - 64], {}", HASH_SORT_DESCENDING)); // descending?
    emitter.instruction("jz __rt_hash_sort_merge_cmp");                         // ascending: compare (left, right)
    emitter.instruction("xchg rdi, rsi");                                       // descending compares (right, left) so ties keep their order
    emitter.label("__rt_hash_sort_merge_cmp");
    // Comparisons may stringify operands into the concat scratch: rewind it afterwards.
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "r10", "_concat_off", 0);
    emitter.instruction("mov QWORD PTR [rbp - 72], r10");                       // save the scratch offset
    emitter.instruction(&format!("test QWORD PTR [rbp - 64], {}", HASH_SORT_STRING)); // SORT_STRING?
    emitter.instruction("jz __rt_hash_sort_merge_regular");                     // no: SORT_REGULAR rules
    emitter.instruction("call __rt_hash_sort_strcmp");                          // rax = -1/0/+1 on the string forms
    emitter.instruction("jmp __rt_hash_sort_merge_pick");                       // pick the next record
    emitter.label("__rt_hash_sort_merge_regular");
    emitter.instruction("mov rax, QWORD PTR [rdi]");                            // left box tag
    emitter.instruction("or rax, QWORD PTR [rsi]");                             // both int (tag 0)?
    emitter.instruction("jnz __rt_hash_sort_merge_php_cmp");                    // no: PHP comparison
    emitter.instruction("mov rax, QWORD PTR [rdi + 8]");                        // left int
    emitter.instruction("xor ecx, ecx");                                        // assume left <= right
    emitter.instruction("cmp rax, QWORD PTR [rsi + 8]");                        // compare ints
    emitter.instruction("setg cl");                                             // 1 when left > right
    emitter.instruction("mov rax, rcx");                                        // comparison result
    emitter.instruction("jmp __rt_hash_sort_merge_pick");                       // pick the next record
    emitter.label("__rt_hash_sort_merge_php_cmp");
    emitter.instruction("call __rt_php_compare");                               // rax = -1/0/+1
    emitter.label("__rt_hash_sort_merge_pick");
    emitter.instruction("mov r10, QWORD PTR [rbp - 72]");                       // scratch offset before the comparison
    crate::codegen_support::abi::emit_store_reg_to_symbol(emitter, "r10", "_concat_off", 0);
    emitter.instruction("test rax, rax");                                       // left after right?
    emitter.instruction("jg __rt_hash_sort_merge_take_right");                  // yes: take the right record
    emitter.instruction("mov r10, r13");                                        // i
    emitter.instruction("add r13, 1");                                          // i++
    emitter.instruction("jmp __rt_hash_sort_merge_store");                      // store it
    emitter.label("__rt_hash_sort_merge_take_right");
    emitter.instruction("mov r10, r14");                                        // j
    emitter.instruction("add r14, 1");                                          // j++
    emitter.label("__rt_hash_sort_merge_store");
    emitter.instruction("shl r10, 4");                                          // source record offset
    emitter.instruction("mov r11, r15");                                        // k
    emitter.instruction("shl r11, 4");                                          // destination record offset
    emitter.instruction("mov rax, QWORD PTR [rbx + r10]");                      // record.slot
    emitter.instruction("mov QWORD PTR [r12 + r11], rax");                      // store slot
    emitter.instruction("mov rax, QWORD PTR [rbx + r10 + 8]");                  // record.box
    emitter.instruction("mov QWORD PTR [r12 + r11 + 8], rax");                  // store box
    emitter.instruction("add r15, 1");                                          // k++
    emitter.instruction("jmp __rt_hash_sort_merge_loop");                       // continue
    emitter.label("__rt_hash_sort_merge_left");
    emitter.instruction("cmp r13, QWORD PTR [rbp - 48]");                       // left records left?
    emitter.instruction("jge __rt_hash_sort_merge_right");                      // no
    emitter.instruction("mov r10, r13");                                        // i
    emitter.instruction("add r13, 1");                                          // i++
    emitter.instruction("jmp __rt_hash_sort_merge_tail_store");                 // copy it
    emitter.label("__rt_hash_sort_merge_right");
    emitter.instruction("cmp r14, QWORD PTR [rbp - 56]");                       // right records left?
    emitter.instruction("jge __rt_hash_sort_merge_done");                       // no
    emitter.instruction("mov r10, r14");                                        // j
    emitter.instruction("add r14, 1");                                          // j++
    emitter.label("__rt_hash_sort_merge_tail_store");
    emitter.instruction("shl r10, 4");                                          // source record offset
    emitter.instruction("mov r11, r15");                                        // k
    emitter.instruction("shl r11, 4");                                          // destination record offset
    emitter.instruction("mov rax, QWORD PTR [rbx + r10]");                      // record.slot
    emitter.instruction("mov QWORD PTR [r12 + r11], rax");                      // store slot
    emitter.instruction("mov rax, QWORD PTR [rbx + r10 + 8]");                  // record.box
    emitter.instruction("mov QWORD PTR [r12 + r11 + 8], rax");                  // store box
    emitter.instruction("add r15, 1");                                          // k++
    emitter.instruction("jmp __rt_hash_sort_merge_left");                       // left first, then right
    emitter.label("__rt_hash_sort_merge_done");
    emitter.instruction("add rsp, 40");                                         // release the locals
    emitter.instruction("pop r15");                                             // restore callee-saved registers
    emitter.instruction("pop r14");                                             // restore callee-saved registers
    emitter.instruction("pop r13");                                             // restore callee-saved registers
    emitter.instruction("pop r12");                                             // restore callee-saved registers
    emitter.instruction("pop rbx");                                             // restore callee-saved registers
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the sort driver

    // -- strcmp(rdi left box, rsi right box): SORT_STRING sign of the stringified operands --
    emitter.blank();
    emitter.label("__rt_hash_sort_strcmp");
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame pointer
    emitter.instruction("sub rsp, 32");                                         // right box/pointer, left string, sign
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // save the right box
    emitter.instruction("mov rax, rdi");                                        // left box
    emitter.instruction("call __rt_mixed_cast_string");                         // rax/rdx = left string
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // left pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // left length
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // right box
    emitter.instruction("call __rt_mixed_cast_string");                         // rax/rdx = right string
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // keep the right pointer for its release
    emitter.instruction("mov rcx, rdx");                                        // right length
    emitter.instruction("mov rdx, rax");                                        // right pointer
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // left pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // left length
    emitter.instruction("call __rt_strcmp");                                    // rax = byte/length difference
    emitter.instruction("xor ecx, ecx");                                        // equal: 0
    emitter.instruction("test rax, rax");                                       // normalize to a sign
    emitter.instruction("setg cl");                                             // greater: 1
    emitter.instruction("mov rdx, -1");                                         // lesser: -1
    emitter.instruction("cmovl rcx, rdx");                                      // select -1 for lesser
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // keep the sign across the releases
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // left string
    emitter.instruction("call __rt_decref_any");                                // release the owned copy (scratch strings are skipped)
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // right string
    emitter.instruction("call __rt_decref_any");                                // release the owned copy (scratch strings are skipped)
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // sign
    emitter.instruction("add rsp, 32");                                         // release the frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the sign
}
