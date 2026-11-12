//! Purpose:
//! Emits the `__rt_gc_collect_cycles`, `__rt_gc_collect_cycles_done` runtime helper assembly for gc collect cycles Linux x86 64.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - GC helpers must honor cycle-collection suppression, mark bits, and parent/child references without double-releasing values.
//! - Incoming edges are counted in one linear heap scan (the previous per-candidate full-heap
//!   rescan was quadratic in the number of live blocks).

use crate::codegen_support::emit::Emitter;

/// Emits the `__rt_gc_collect_cycles` and `__rt_gc_collect_cycles_done` runtime helpers for Linux x86_64.
///
/// This is a four-pass mark-sweep collector tailored to PHP array/hash/object storage on the managed heap:
/// - **Pass 1 (clear):** clears the transient metadata (bits 16..31) on every live heap block while preserving kind, value_type, and heap marker bits.
/// - **Pass 2 (count):** one heap scan adds every outgoing child edge to the child's saturating incoming-edge counter (bits 17..31) through `__rt_gc_note_child_ref_x86`.
/// - **Pass 3 (roots):** nodes whose refcount exceeds their incoming heap edges are externally rooted and marked reachable via `__rt_gc_mark_reachable`.
/// - **Pass 4 (free):** frees every still-unreachable live refcounted node by dispatching to `__rt_array_free_deep`, `__rt_hash_free_deep`, `__rt_mixed_free_deep`, or `__rt_object_free_deep`.
///
/// Re-entry is guarded by the `_gc_collecting` flag — nested collection attempts are silently skipped.
///
/// Uses cdecl calling convention; all registers are caller-saved except the frame pointer.
pub(super) fn emit_gc_collect_cycles_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: gc_collect_cycles ---");
    emitter.label_global("__rt_gc_collect_cycles");

    // -- avoid recursive re-entry while the collector is already running --
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_gc_collecting");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // load the current collector-active flag before starting a new x86_64 collection pass
    emitter.instruction("test r9, r9");                                         // is the collector already running?
    emitter.instruction("jnz __rt_gc_collect_cycles_done");                     // nested collection attempts are ignored to avoid recursive frees
    emitter.instruction("mov QWORD PTR [r8], 1");                               // mark the collector active for the duration of this x86_64 cycle pass

    // -- set up a collector frame --
    // Stack layout:
    //   [rbp - 8]  = heap base
    //   [rbp - 16] = initial heap end
    //   [rbp - 24] = outer scan header
    //   [rbp - 32] = (unused)
    //   [rbp - 40] = scratch / saved next header
    //   [rbp - 48] = (unused)
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving x86_64 collector locals
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame pointer for the x86_64 collector locals
    emitter.instruction("sub rsp, 48");                                         // reserve collector locals for heap bounds, scan pointers, and incoming counts

    // -- capture heap bounds once for the current collection pass --
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_heap_buf");
    emitter.instruction("mov QWORD PTR [rbp - 8], r8");                         // save the heap base so every collector pass can restart from the same managed heap window
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_heap_off");
    emitter.instruction("mov r9, QWORD PTR [r9]");                              // load the current heap bump offset before capturing the initial heap end
    emitter.instruction("lea r9, [r8 + r9]");                                   // compute the initial heap end from the heap base plus bump offset
    emitter.instruction("mov QWORD PTR [rbp - 16], r9");                        // save the initial heap end for all x86_64 metadata, root, and free scans
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // initialize the outer scan pointer to the heap base for the clear pass

    // -- pass 1: clear the x86_64 reachable bit while preserving kind + array value_type + heap marker --
    emitter.label("__rt_gc_collect_cycles_clear_loop");
    emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                        // reload the current heap header scan pointer for the metadata-clear pass
    emitter.instruction("cmp r8, QWORD PTR [rbp - 16]");                        // have we reached the initial heap end for this collection pass?
    emitter.instruction("jae __rt_gc_collect_cycles_count_init");               // yes — count incoming edges once every live block has its transient metadata cleared
    emitter.instruction("mov r9d, DWORD PTR [r8]");                             // load this block payload size before advancing to the next heap header
    emitter.instruction("mov r10d, DWORD PTR [r8 + 4]");                        // load this block refcount so free-list blocks can be skipped during metadata clearing
    emitter.instruction("test r10d, r10d");                                     // is this heap block currently live?
    emitter.instruction("jz __rt_gc_collect_cycles_clear_next");                // free-list blocks already keep transient GC metadata cleared
    emitter.instruction("mov r11, QWORD PTR [r8 + 8]");                         // load the full kind word with any stale x86_64 reachable metadata
    emitter.instruction("mov rcx, 0xffffffff0000ffff");                         // preserve the high-word heap marker and low 16 bits while clearing the transient x86_64 mark range
    emitter.instruction("and r11, rcx");                                        // clear the x86_64 transient reachable metadata while preserving kind and value_type bits
    emitter.instruction("mov QWORD PTR [r8 + 8], r11");                         // persist the cleared x86_64 kind word back into the heap header
    emitter.label("__rt_gc_collect_cycles_clear_next");
    emitter.instruction("add r8, r9");                                          // advance by the current block payload size
    emitter.instruction("add r8, 16");                                          // skip the uniform 16-byte heap header to reach the next block header
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // save the next heap header scan pointer for the following clear iteration
    emitter.instruction("jmp __rt_gc_collect_cycles_clear_loop");               // continue clearing transient metadata across the managed heap window

    emitter.label("__rt_gc_collect_cycles_count_init");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 8]");                        // heap base
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // start the edge-count scan at the heap base
    // -- pass 2: count incoming heap edges for every block in one scan (saturating counter in
    //    kind-word bits 17..31; bit 16 stays the reachable mark) --
    emitter.label("__rt_gc_collect_cycles_count_loop");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // reload the current source heap header for the incoming-edge rescan
    emitter.instruction("cmp rdx, QWORD PTR [rbp - 16]");                       // have we finished rescanning the initial heap window?
    emitter.instruction("jae __rt_gc_collect_cycles_root_init");                // every edge counted: find the externally rooted nodes
    emitter.instruction("mov ecx, DWORD PTR [rdx]");                            // load this source block payload size before deciding whether it emits outgoing edges
    emitter.instruction("mov rdi, rcx");                                        // preserve the source block payload size across the outgoing-edge dispatch loops
    emitter.instruction("mov r8d, DWORD PTR [rdx + 4]");                        // load the source block refcount so free-list blocks can be skipped
    emitter.instruction("test r8d, r8d");                                       // is this source block live?
    emitter.instruction("jz __rt_gc_collect_cycles_count_next");                // free blocks contribute no outgoing graph edges
    emitter.instruction("mov r9, QWORD PTR [rdx + 8]");                         // load the source kind word before dispatching on its outgoing edge layout
    emitter.instruction("mov r10, r9");                                         // preserve the full source kind word while isolating the low-byte heap kind tag
    emitter.instruction("and r10, 0xff");                                       // isolate the source heap kind tag for outgoing-edge dispatch
    emitter.instruction("cmp r10, 2");                                          // is this source at least an indexed array?
    emitter.instruction("jb __rt_gc_collect_cycles_count_next");                // strings and raw buffers contribute no outgoing cycle edges
    emitter.instruction("cmp r10, 7");                                          // is this source within the array/hash/object/mixed/refcell range?
    emitter.instruction("ja __rt_gc_collect_cycles_count_next");                // unknown/raw heap kinds are ignored by the collector
    emitter.instruction("cmp r10, 2");                                          // is the source block an indexed array?
    emitter.instruction("je __rt_gc_collect_cycles_count_array");               // yes — scan array child slots
    emitter.instruction("cmp r10, 3");                                          // is the source block an associative array / hash?
    emitter.instruction("je __rt_gc_collect_cycles_count_hash");                // yes — scan hash entry child payloads
    emitter.instruction("cmp r10, 5");                                          // is the source block a boxed mixed cell?
    emitter.instruction("je __rt_gc_collect_cycles_count_mixed");               // yes — compare the boxed child pointer against the candidate
    emitter.instruction("cmp r10, 7");                                          // is the source block a refcounted reference cell?
    emitter.instruction("je __rt_gc_collect_cycles_count_refcell");             // yes — compare the cell's inner-value pointer against the candidate
    emitter.instruction("jmp __rt_gc_collect_cycles_count_object");             // the remaining refcounted heap kind is an object instance

    emitter.label("__rt_gc_collect_cycles_count_array");
    emitter.instruction("mov r10, r9");                                         // preserve the full array kind word while unpacking the runtime array value_type tag
    emitter.instruction("shr r10, 8");                                          // move the packed array value_type tag into the low bits
    emitter.instruction("and r10, 0x7f");                                       // isolate the array value_type without the persistent COW flag bit
    emitter.instruction("cmp r10, 4");                                          // does this array contain refcounted element payloads?
    emitter.instruction("jb __rt_gc_collect_cycles_count_next");                // scalar and string arrays emit no refcounted outgoing edges
    emitter.instruction("cmp r10, 7");                                          // is the array value_type within the supported refcounted range?
    emitter.instruction("ja __rt_gc_collect_cycles_count_next");                // unknown array payload tags are ignored by the collector
    emitter.instruction("lea r9, [rdx + 16]");                                  // compute the source array user pointer from its heap header
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // load the source array length before iterating its child slots
    emitter.instruction("xor r11, r11");                                        // initialize the source array index to zero for the incoming-edge comparison loop
    emitter.label("__rt_gc_collect_cycles_count_array_loop");
    emitter.instruction("cmp r11, r10");                                        // have we inspected every child slot in this source array?
    emitter.instruction("jae __rt_gc_collect_cycles_count_next");               // yes — move on to the next source block in the rescan
    emitter.instruction("mov r8, r11");                                         // preserve the logical array index while scaling it into a byte offset
    emitter.instruction("shl r8, 3");                                           // scale the array index by eight bytes per child pointer slot
    emitter.instruction("add r8, 24");                                          // skip the 24-byte array header to reach the selected child slot
    emitter.instruction("push QWORD PTR [r9 + r8]");                             // pass the child pointer on the stack
    emitter.instruction("call __rt_gc_note_child_ref_x86"); // count one incoming heap edge into the child
    emitter.instruction("add rsp, 8"); // drop the argument
    emitter.label("__rt_gc_collect_cycles_count_array_next");
    emitter.instruction("add r11, 1");                                          // advance to the next child slot in the source array
    emitter.instruction("jmp __rt_gc_collect_cycles_count_array_loop");         // continue comparing source array children against the current candidate

    emitter.label("__rt_gc_collect_cycles_count_hash");
    emitter.instruction("lea r9, [rdx + 16]");                                  // compute the source hash user pointer from its heap header
    emitter.instruction("mov r10, QWORD PTR [r9 + 8]");                         // load the source hash capacity before iterating its entry slots
    emitter.instruction("xor r11, r11");                                        // initialize the source hash slot index to zero for the incoming-edge rescan
    emitter.label("__rt_gc_collect_cycles_count_hash_loop");
    emitter.instruction("cmp r11, r10");                                        // have we inspected every hash entry slot?
    emitter.instruction("jae __rt_gc_collect_cycles_count_next");               // yes — move on to the next source block in the rescan
    emitter.instruction("mov r8, r11");                                         // preserve the logical hash slot index while scaling it into an entry byte offset
    emitter.instruction("imul r8, 64");                                         // scale the slot index by 64 bytes per hash entry
    emitter.instruction("add r8, 40");                                          // skip the 40-byte hash header to reach the selected entry
    emitter.instruction("lea r8, [r9 + r8]");                                   // compute the address of the selected hash entry inside the source hash table
    emitter.instruction("cmp QWORD PTR [r8], 1");                               // is this hash entry occupied?
    emitter.instruction("jne __rt_gc_collect_cycles_count_hash_next");          // skip empty and tombstone entries that carry no outgoing edge
    emitter.instruction("mov rax, QWORD PTR [r8 + 40]");                        // load the runtime value_tag stored for this hash entry
    emitter.instruction("cmp rax, 11");                                         // does this hash entry hold a reference cell?
    emitter.instruction("je __rt_gc_collect_cycles_count_hash_child");          // reference cells still emit an incoming edge into the candidate
    emitter.instruction("cmp rax, 4");                                          // does this hash entry hold a heap-backed child?
    emitter.instruction("jb __rt_gc_collect_cycles_count_hash_next");           // scalar and string entries contribute no incoming edge to the candidate
    emitter.instruction("cmp rax, 7");                                          // is the runtime value_tag within the supported heap-backed range?
    emitter.instruction("ja __rt_gc_collect_cycles_count_hash_next");           // unknown runtime tags are ignored by the collector
    emitter.label("__rt_gc_collect_cycles_count_hash_child");
    emitter.instruction("push QWORD PTR [r8 + 24]");                             // pass the child pointer on the stack
    emitter.instruction("call __rt_gc_note_child_ref_x86"); // count one incoming heap edge into the child
    emitter.instruction("add rsp, 8"); // drop the argument
    emitter.label("__rt_gc_collect_cycles_count_hash_next");
    emitter.instruction("add r11, 1");                                          // advance to the next hash entry slot in the source table
    emitter.instruction("jmp __rt_gc_collect_cycles_count_hash_loop");          // continue comparing source hash children against the current candidate

    emitter.label("__rt_gc_collect_cycles_count_mixed");
    emitter.instruction("lea r9, [rdx + 16]");                                  // compute the source mixed-box user pointer from its heap header
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // load the boxed mixed runtime value_tag before comparing the boxed child pointer
    emitter.instruction("cmp r10, 4");                                          // does this boxed mixed value hold a heap-backed child?
    emitter.instruction("jb __rt_gc_collect_cycles_count_next");                // scalar, string, and null boxed values contribute no incoming edge
    emitter.instruction("cmp r10, 7");                                          // is the boxed runtime tag within the supported heap-backed range?
    emitter.instruction("ja __rt_gc_collect_cycles_count_next");                // unknown boxed runtime tags are ignored by the collector
    emitter.instruction("push QWORD PTR [r9 + 8]");                              // pass the child pointer on the stack
    emitter.instruction("call __rt_gc_note_child_ref_x86"); // count one incoming heap edge into the child
    emitter.instruction("add rsp, 8"); // drop the argument
    emitter.instruction("jmp __rt_gc_collect_cycles_count_next");               // boxed mixed-child comparison is complete for this source block

    emitter.label("__rt_gc_collect_cycles_count_refcell");
    emitter.instruction("lea r9, [rdx + 16]");                                  // compute the source reference-cell user pointer from its heap header
    emitter.instruction("mov r10, QWORD PTR [r9 + 8]");                         // load the reference cell's inner value-tag before comparing its inner pointer
    emitter.instruction("cmp r10, 4");                                          // does the cell's inner value hold a heap-backed child?
    emitter.instruction("jb __rt_gc_collect_cycles_count_next");                // scalar, string, and null inner values contribute no incoming edge
    emitter.instruction("cmp r10, 7");                                          // is the inner value-tag within the supported heap-backed range?
    emitter.instruction("ja __rt_gc_collect_cycles_count_next");                // unknown inner value-tags are ignored by the collector
    emitter.instruction("push QWORD PTR [r9]");                                  // pass the child pointer on the stack
    emitter.instruction("call __rt_gc_note_child_ref_x86"); // count one incoming heap edge into the child
    emitter.instruction("add rsp, 8"); // drop the argument
    emitter.instruction("jmp __rt_gc_collect_cycles_count_next");               // reference-cell inner-child comparison is complete for this source block

    emitter.label("__rt_gc_collect_cycles_count_object");
    emitter.instruction("lea r9, [rdx + 16]");                                  // compute the source object user pointer from its heap header
    emitter.instruction("mov eax, DWORD PTR [rdx]");                            // load the source object payload size before deriving the property count
    emitter.instruction("sub rax, 8");                                          // subtract the leading class_id field from the object payload size
    emitter.instruction("shr rax, 4");                                          // divide by 16 to get the source object property count
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // load the runtime class_id stored at the start of the source object payload
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_class_gc_desc_count");
    emitter.instruction("mov r11, QWORD PTR [r11]");                            // load the number of emitted class GC descriptors for bounds checking
    emitter.instruction("cmp r10, r11");                                        // is the runtime class_id within the emitted descriptor table range?
    emitter.instruction("jae __rt_gc_collect_cycles_count_next");               // invalid class ids contribute no traversable property metadata
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_class_gc_desc_ptrs");
    emitter.instruction("mov r11, QWORD PTR [r11 + r10 * 8]");                  // load the per-class property-tag descriptor pointer for the source object
    emitter.instruction("xor r10, r10");                                        // initialize the source object property index to zero for the incoming-edge scan
    emitter.label("__rt_gc_collect_cycles_count_object_loop");
    emitter.instruction("cmp r10, rax");                                        // have we inspected every property slot in this source object?
    emitter.instruction("jae __rt_gc_collect_cycles_count_next");               // yes — move on to the next source block in the rescan
    emitter.instruction("movzx ecx, BYTE PTR [r11 + r10]");                     // load the compile-time property tag for the selected source object property
    emitter.instruction("mov r8, r10");                                         // preserve the logical property index while scaling it into a byte offset
    emitter.instruction("imul r8, 16");                                         // scale the property index by 16 bytes per object property slot
    emitter.instruction("add r8, 8");                                           // skip the leading class_id field to reach the selected property slot
    emitter.instruction("cmp rcx, 4");                                          // is this property statically typed as an indexed array?
    emitter.instruction("je __rt_gc_collect_cycles_count_object_child");        // yes — compare the direct property child pointer against the current candidate
    emitter.instruction("cmp rcx, 5");                                          // is this property statically typed as an associative array?
    emitter.instruction("je __rt_gc_collect_cycles_count_object_child");        // yes — compare the direct property child pointer against the current candidate
    emitter.instruction("cmp rcx, 6");                                          // is this property statically typed as an object?
    emitter.instruction("je __rt_gc_collect_cycles_count_object_child");        // yes — compare the direct property child pointer against the current candidate
    emitter.instruction("cmp rcx, 7");                                          // is this property statically typed as a mixed slot?
    emitter.instruction("jne __rt_gc_collect_cycles_count_object_next");        // scalar and string properties contribute no incoming heap edges
    emitter.instruction("mov rcx, QWORD PTR [r9 + r8 + 8]");                    // load the runtime tag stored alongside the mixed property payload
    emitter.instruction("cmp rcx, 4");                                          // does the mixed property currently hold a heap-backed child?
    emitter.instruction("jb __rt_gc_collect_cycles_count_object_next");         // scalar, string, and null mixed payloads contribute no incoming edge
    emitter.instruction("cmp rcx, 7");                                          // is the mixed runtime tag within the supported heap-backed range?
    emitter.instruction("ja __rt_gc_collect_cycles_count_object_next");         // unknown mixed runtime tags are ignored by the collector
    emitter.label("__rt_gc_collect_cycles_count_object_child");
    emitter.instruction("push QWORD PTR [r9 + r8]");                             // pass the child pointer on the stack
    emitter.instruction("call __rt_gc_note_child_ref_x86"); // count one incoming heap edge into the child
    emitter.instruction("add rsp, 8"); // drop the argument
    emitter.label("__rt_gc_collect_cycles_count_object_next");
    emitter.instruction("add r10, 1");                                          // advance to the next object property slot in the source object
    emitter.instruction("jmp __rt_gc_collect_cycles_count_object_loop");        // continue comparing source object property children against the candidate

    emitter.label("__rt_gc_collect_cycles_count_next");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // reload the current source heap header after any nested child scans
    emitter.instruction("add rdx, rdi");                                        // advance by the preserved source block payload size to reach the next header candidate
    emitter.instruction("add rdx, 16");                                         // skip the uniform 16-byte heap header to reach the next source block header
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // persist the next source heap header for the incoming-edge rescan loop
    emitter.instruction("jmp __rt_gc_collect_cycles_count_loop");               // continue rescanning heap edges into the current candidate node

    // -- pass 3: a block whose refcount exceeds its incoming heap edges is externally rooted --
    emitter.label("__rt_gc_collect_cycles_root_init");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the heap base before starting the externally rooted node scan
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // restart the outer scan pointer at the heap base
    emitter.label("__rt_gc_collect_cycles_root_loop");
    emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                        // reload the current candidate heap header for the x86_64 root scan
    emitter.instruction("cmp r8, QWORD PTR [rbp - 16]");                        // have we scanned every block in the initial heap window?
    emitter.instruction("jae __rt_gc_collect_cycles_free_init");                // yes — move on to freeing the still-unreachable graph nodes
    emitter.instruction("mov r9d, DWORD PTR [r8]");                             // load this candidate block payload size before any nested rescans
    emitter.instruction("mov r10d, DWORD PTR [r8 + 4]");                        // load this candidate block refcount from the heap header
    emitter.instruction("test r10d, r10d");                                     // is this candidate block live?
    emitter.instruction("jz __rt_gc_collect_cycles_root_next");                 // free-list blocks cannot be GC roots
    emitter.instruction("mov r11, QWORD PTR [r8 + 8]");                         // load the candidate kind word before deciding whether it participates in cycle collection
    emitter.instruction("mov rcx, r11");                                        // preserve the full kind word while isolating the low-byte heap kind tag
    emitter.instruction("and rcx, 0xff");                                       // isolate the low-byte heap kind tag for candidate dispatch
    emitter.instruction("cmp rcx, 2");                                          // is this candidate at least an indexed array?
    emitter.instruction("jb __rt_gc_collect_cycles_root_next");                 // strings and raw buffers never participate in cycle collection
    emitter.instruction("cmp rcx, 7");                                          // is this candidate within the array/hash/object/mixed/refcell range?
    emitter.instruction("ja __rt_gc_collect_cycles_root_next");                 // unknown/raw heap kinds are ignored by the collector
    emitter.instruction("cmp rcx, 2");                                          // is this candidate an indexed array?
    emitter.instruction("jne __rt_gc_collect_cycles_root_candidate_ready");     // hashes, objects, mixed boxes, and reference cells remain collector candidates
    emitter.instruction("mov rdx, r11");                                        // preserve the full array kind word while unpacking the runtime array value_type tag
    emitter.instruction("shr rdx, 8");                                          // move the packed array value_type tag into the low bits
    emitter.instruction("and rdx, 0x7f");                                       // isolate the array value_type without the persistent COW flag bit
    emitter.instruction("cmp rdx, 4");                                          // does this array contain refcounted element payloads?
    emitter.instruction("jb __rt_gc_collect_cycles_root_next");                 // scalar and string arrays cannot participate in heap cycles
    emitter.instruction("cmp rdx, 7");                                          // is the array value_type within the supported refcounted range?
    emitter.instruction("ja __rt_gc_collect_cycles_root_next");                 // unknown array payload tags are ignored by the collector
    emitter.label("__rt_gc_collect_cycles_root_candidate_ready");
    emitter.instruction("mov rdx, r11");                                        // kind word with the incoming-edge counter
    emitter.instruction("shr rdx, 17");                                         // move the counter to the low bits
    emitter.instruction("and edx, 0x7fff");                                     // isolate the 15-bit saturating counter
    emitter.instruction("cmp r10, rdx");                                        // refcount beyond the heap-internal edges?
    emitter.instruction("jbe __rt_gc_collect_cycles_root_next");                // no: only reachable from inside the heap
    emitter.instruction("lea rax, [r8 + 16]");                                  // user pointer of the externally rooted node
    emitter.instruction("call __rt_gc_mark_reachable");                         // mark everything reachable from it
    emitter.label("__rt_gc_collect_cycles_root_next");
    emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                        // reload the current candidate heap header after any nested mark traversal
    emitter.instruction("mov r9d, DWORD PTR [r8]");                             // reload the candidate payload size so the outer scan can advance correctly
    emitter.instruction("add r8, r9");                                          // advance by the candidate payload size to the next heap header
    emitter.instruction("add r8, 16");                                          // skip the uniform 16-byte heap header to reach the next outer candidate
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // persist the next candidate heap header for the outer root scan
    emitter.instruction("jmp __rt_gc_collect_cycles_root_loop");                // continue looking for externally rooted graph nodes

    // -- pass 4: free every still-unreachable live refcounted node --
    emitter.label("__rt_gc_collect_cycles_free_init");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the heap base before starting the unreachable-node free scan
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // restart the outer scan pointer at the heap base for the free pass
    emitter.label("__rt_gc_collect_cycles_free_loop");
    emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                        // reload the current heap header for the unreachable-node free scan
    emitter.instruction("cmp r8, QWORD PTR [rbp - 16]");                        // have we scanned every block in the initial heap window?
    emitter.instruction("jae __rt_gc_collect_cycles_finish");                   // yes — finish once every initial live block has been checked
    emitter.instruction("mov r9d, DWORD PTR [r8]");                             // load this block payload size before saving the next header across any deep frees
    emitter.instruction("lea r10, [r8 + r9 + 16]");                             // compute the next heap header before freeing the current node can mutate allocator state
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // save the next heap header so the scan can continue after a deep free
    emitter.instruction("mov r10d, DWORD PTR [r8 + 4]");                        // load this block refcount to skip already-free nodes during the free pass
    emitter.instruction("test r10d, r10d");                                     // is this block still live?
    emitter.instruction("jz __rt_gc_collect_cycles_free_next");                 // free-list blocks are already reclaimed and need no collector action
    emitter.instruction("mov r11, QWORD PTR [r8 + 8]");                         // load the current kind word before deciding whether this block is a collector candidate
    emitter.instruction("mov rcx, r11");                                        // preserve the full kind word while isolating the low-byte heap kind tag
    emitter.instruction("and rcx, 0xff");                                       // isolate the low-byte heap kind tag for free-pass dispatch
    emitter.instruction("cmp rcx, 2");                                          // is this block at least an indexed array?
    emitter.instruction("jb __rt_gc_collect_cycles_free_next");                 // strings and raw buffers are outside the cycle collector set
    emitter.instruction("cmp rcx, 7");                                          // is this block within the array/hash/object/mixed/refcell range?
    emitter.instruction("ja __rt_gc_collect_cycles_free_next");                 // unknown/raw heap kinds are ignored by the collector
    emitter.instruction("cmp rcx, 2");                                          // is this block an indexed array?
    emitter.instruction("jne __rt_gc_collect_cycles_free_candidate_ready");     // hashes, objects, mixed boxes, and reference cells remain collector candidates
    emitter.instruction("mov rdx, r11");                                        // preserve the full array kind word while unpacking the runtime array value_type tag
    emitter.instruction("shr rdx, 8");                                          // move the packed array value_type tag into the low bits
    emitter.instruction("and rdx, 0x7f");                                       // isolate the array value_type without the persistent COW flag bit
    emitter.instruction("cmp rdx, 4");                                          // does this array contain refcounted element payloads?
    emitter.instruction("jb __rt_gc_collect_cycles_free_next");                 // scalar and string arrays cannot participate in heap cycles
    emitter.instruction("cmp rdx, 7");                                          // is the array value_type within the supported refcounted range?
    emitter.instruction("ja __rt_gc_collect_cycles_free_next");                 // unknown array payload tags are ignored by the collector
    emitter.label("__rt_gc_collect_cycles_free_candidate_ready");
    emitter.instruction("test r11, 0x10000");                                   // was this live block marked reachable from an external root during the root pass?
    emitter.instruction("jnz __rt_gc_collect_cycles_free_next");                // yes — reachable graph nodes remain live
    emitter.instruction("mov DWORD PTR [r8 + 4], 0");                           // pre-clear the doomed node refcount so back-edges released during deep-free cannot recursively reclaim it again
    emitter.instruction("lea rax, [r8 + 16]");                                  // compute the current user pointer before dispatching to the deep-free helper
    emitter.instruction("cmp rcx, 2");                                          // is this unreachable node an indexed array?
    emitter.instruction("je __rt_gc_collect_cycles_free_array");                // yes — deep-free the unreachable array and its child payloads
    emitter.instruction("cmp rcx, 3");                                          // is this unreachable node an associative array / hash?
    emitter.instruction("je __rt_gc_collect_cycles_free_hash");                 // yes — deep-free the unreachable hash and its owned entries
    emitter.instruction("cmp rcx, 5");                                          // is this unreachable node a boxed mixed cell?
    emitter.instruction("je __rt_gc_collect_cycles_free_mixed");                // yes — deep-free the unreachable mixed box and its boxed child
    emitter.instruction("cmp rcx, 7");                                          // is this unreachable node a refcounted reference cell?
    emitter.instruction("je __rt_gc_collect_cycles_free_refcell");              // yes — deep-free the unreachable reference cell and release its inner value
    emitter.instruction("call __rt_object_free_deep");                          // deep-free the remaining unreachable object node and its properties
    emitter.instruction("jmp __rt_gc_collect_cycles_free_next");                // continue scanning from the saved next header after freeing the object node
    emitter.label("__rt_gc_collect_cycles_free_refcell");
    emitter.instruction("call __rt_ref_cell_free_deep");                        // deep-free the unreachable reference cell and release its owned inner value
    emitter.instruction("jmp __rt_gc_collect_cycles_free_next");                // continue scanning from the saved next header after freeing the reference cell
    emitter.label("__rt_gc_collect_cycles_free_array");
    emitter.instruction("call __rt_array_free_deep");                           // deep-free the unreachable array node and its nested payloads
    emitter.instruction("jmp __rt_gc_collect_cycles_free_next");                // continue scanning from the saved next header after freeing the array node
    emitter.label("__rt_gc_collect_cycles_free_hash");
    emitter.instruction("call __rt_hash_free_deep");                            // deep-free the unreachable hash node and its owned entries
    emitter.instruction("jmp __rt_gc_collect_cycles_free_next");                // continue scanning from the saved next header after freeing the hash node
    emitter.label("__rt_gc_collect_cycles_free_mixed");
    emitter.instruction("call __rt_mixed_free_deep");                           // deep-free the unreachable mixed box and its boxed child

    emitter.label("__rt_gc_collect_cycles_free_next");
    emitter.instruction("mov r8, QWORD PTR [rbp - 40]");                        // reload the next saved heap header after any deep free mutated allocator state
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // persist the next heap header so the free pass can continue scanning
    emitter.instruction("jmp __rt_gc_collect_cycles_free_loop");                // continue scanning the initial heap window for unreachable graph nodes

    emitter.label("__rt_gc_collect_cycles_finish");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_gc_collecting");
    emitter.instruction("mov QWORD PTR [r8], 0");                               // clear the collector-active flag now that the x86_64 cycle pass is complete
    emitter.instruction("leave");                                               // tear down the x86_64 collector frame before returning to generated code

    emitter.label("__rt_gc_collect_cycles_done");
    emitter.instruction("ret");                                                 // return immediately when collection is skipped or after a full x86_64 cycle pass

    emit_gc_note_child_ref_x86_64(emitter);
}

/// Emits the x86_64 `__rt_gc_note_child_ref_x86` edge counter used by the collector.
///
/// The child user pointer is passed on the stack (`[rsp + 8]`) so the caller's scan registers
/// survive; the helper preserves every register it touches. Pointers outside the managed heap
/// or without the elephc heap marker are ignored. The counter lives in kind-word bits 17..31
/// and saturates at 0x7fff: a saturated count only makes a node look externally referenced,
/// which keeps it alive (never frees a live node).
pub(super) fn emit_gc_note_child_ref_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.label("__rt_gc_note_child_ref_x86");
    emitter.instruction("push rax");                                            // preserve scratch
    emitter.instruction("push rcx");                                            // preserve scratch
    emitter.instruction("push rdx");                                            // preserve scratch
    emitter.instruction("mov rax, QWORD PTR [rsp + 32]");                       // child user pointer
    emitter.instruction("test rax, rax");                                       // null child?
    emitter.instruction("jz __rt_gc_note_child_ref_x86_done");                  // nothing to count
    crate::codegen_support::abi::emit_symbol_address(emitter, "rcx", "_heap_buf");
    emitter.instruction("cmp rax, rcx");                                        // below the managed heap?
    emitter.instruction("jb __rt_gc_note_child_ref_x86_done");                  // foreign pointer
    crate::codegen_support::abi::emit_load_symbol_to_reg(emitter, "rdx", "_heap_off", 0);
    emitter.instruction("add rdx, rcx");                                        // heap end
    emitter.instruction("cmp rax, rdx");                                        // beyond the managed heap?
    emitter.instruction("jae __rt_gc_note_child_ref_x86_done");                 // foreign pointer
    emitter.instruction("mov rcx, QWORD PTR [rax - 8]");                        // child kind word
    emitter.instruction("mov rdx, rcx");                                        // copy for the marker check
    emitter.instruction("shr rdx, 32");                                         // high-word heap marker
    emitter.instruction(&format!("cmp edx, 0x{:x}", crate::codegen_support::sentinels::X86_64_HEAP_MAGIC_HI32)); // elephc heap block?
    emitter.instruction("jne __rt_gc_note_child_ref_x86_done");                 // not an elephc block
    emitter.instruction("mov rdx, rcx");                                        // copy for the counter
    emitter.instruction("shr rdx, 17");                                         // counter to the low bits
    emitter.instruction("and edx, 0x7fff");                                     // isolate it
    emitter.instruction("cmp edx, 0x7fff");                                     // saturated?
    emitter.instruction("je __rt_gc_note_child_ref_x86_done");                  // keep the saturated value
    emitter.instruction("add rcx, 0x20000");                                    // one more incoming edge
    emitter.instruction("mov QWORD PTR [rax - 8], rcx");                        // store the kind word back
    emitter.label("__rt_gc_note_child_ref_x86_done");
    emitter.instruction("pop rdx");                                             // restore scratch
    emitter.instruction("pop rcx");                                             // restore scratch
    emitter.instruction("pop rax");                                             // restore scratch
    emitter.instruction("ret");                                                 // return to the edge scan
}
