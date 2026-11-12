//! Purpose:
//! Emits the `__rt_rsort_int`, `__rt_sort_int` runtime helper assembly for sort int.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Sort helpers mutate array payload order in place and must preserve PHP comparison behavior for supported value kinds.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_sort_int` (ascending) or `__rt_rsort_int` (descending) runtime helper.
/// Uses ARM64 instructions on non-x86_64 targets; delegates to `emit_sort_int_linux_x86_64` for x86_64.
///
/// # Arguments
/// * `emitter` — the assembly emitter with target and context
/// * `reverse` — `false` for ascending sort, `true` for descending sort
///
/// # ABI
/// * Input: array pointer in `x0`
/// * Output: sorted array in-place (no return value register)
/// * Temporaries: `x1`–`x9` (caller-saved)
///
/// # Algorithm
/// In-place heapsort, O(n log n): build a max-heap (a min-heap for descending order) by
/// sifting down every internal node, then repeatedly swap the root with the last heap slot
/// and sift the new root down. Signed integer comparisons match PHP's integer ordering, and
/// equal integers are indistinguishable, so stability does not matter.
pub fn emit_sort_int(emitter: &mut Emitter, reverse: bool) {
    if emitter.target.arch == Arch::X86_64 {
        emit_sort_int_linux_x86_64(emitter, reverse);
        return;
    }

    let label = if reverse { "__rt_rsort_int" } else { "__rt_sort_int" };
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} (heapsort) ---", label));
    emitter.label_global(label);

    emitter.instruction("ldr x1, [x0]");                                        // x1 = array length from header
    emitter.instruction("add x2, x0, #24");                                     // x2 = base of data region (skip header)
    emitter.instruction("cmp x1, #2");                                          // fewer than two elements?
    emitter.instruction(&format!("b.lo {}_done", label));                       // already sorted

    // -- phase 1: heapify, sifting down every internal node from n/2 - 1 to 0 --
    emitter.instruction("lsr x3, x1, #1");                                      // x3 = n / 2 (one past the last internal node)
    emitter.instruction("mov x4, x1");                                          // x4 = heap limit = n
    emitter.label(&format!("{}_build", label));
    emitter.instruction(&format!("cbz x3, {}_extract_start", label));           // every internal node sifted
    emitter.instruction("sub x3, x3, #1");                                      // next internal node
    emitter.instruction("mov x5, x3");                                          // root = node
    emit_sift_down_aarch64(emitter, &format!("{}_bsift", label), reverse);
    emitter.instruction(&format!("b {}_build", label));                         // continue heapifying

    // -- phase 2: move the root behind the shrinking heap, then restore the heap --
    emitter.label(&format!("{}_extract_start", label));
    emitter.instruction("sub x3, x1, #1");                                      // x3 = last heap slot
    emitter.label(&format!("{}_extract", label));
    emitter.instruction(&format!("cbz x3, {}_done", label));                    // one element left: sorted
    emitter.instruction("ldr x8, [x2]");                                        // x8 = heap root (extreme element)
    emitter.instruction("ldr x9, [x2, x3, lsl #3]");                            // x9 = last heap element
    emitter.instruction("str x9, [x2]");                                        // move the last element to the root
    emitter.instruction("str x8, [x2, x3, lsl #3]");                            // park the extreme element in its final slot
    emitter.instruction("mov x4, x3");                                          // heap limit = slots still in the heap
    emitter.instruction("mov x5, #0");                                          // root = 0
    emit_sift_down_aarch64(emitter, &format!("{}_esift", label), reverse);
    emitter.instruction("sub x3, x3, #1");                                      // shrink the heap
    emitter.instruction(&format!("b {}_extract", label));                       // extract the next extreme element

    emitter.label(&format!("{}_done", label));
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits an inline AArch64 sift-down of `x5` (root) within the heap `x2[0..x4)`.
/// Clobbers `x5`–`x9`.
fn emit_sift_down_aarch64(emitter: &mut Emitter, prefix: &str, reverse: bool) {
    // Ascending sorts keep a max-heap, descending sorts a min-heap.
    let (pick_right_if, stop_if) = if reverse { ("lt", "ge") } else { ("gt", "le") };
    emitter.instruction("ldr x7, [x2, x5, lsl #3]");                            // v = value being sifted down
    emitter.label(&format!("{}_loop", prefix));
    emitter.instruction("lsl x6, x5, #1");                                      // 2 * root
    emitter.instruction("add x6, x6, #1");                                      // child = 2 * root + 1
    emitter.instruction("cmp x6, x4");                                          // is the child inside the heap?
    emitter.instruction(&format!("b.hs {}_end", prefix));                       // no — root is a leaf
    emitter.instruction("ldr x8, [x2, x6, lsl #3]");                            // x8 = left child value
    emitter.instruction("add x9, x6, #1");                                      // right child index
    emitter.instruction("cmp x9, x4");                                          // is there a right child?
    emitter.instruction(&format!("b.hs {}_one", prefix));                       // no — compare against the left child only
    emitter.instruction("ldr x9, [x2, x9, lsl #3]");                            // x9 = right child value
    emitter.instruction("cmp x9, x8");                                          // does the right child dominate the left one?
    emitter.instruction(&format!("b.{} {}_take_right", pick_right_if, prefix)); // yes — follow the right child
    emitter.instruction(&format!("b {}_one", prefix));                          // no — keep the left child
    emitter.label(&format!("{}_take_right", prefix));
    emitter.instruction("add x6, x6, #1");                                      // child = right child
    emitter.instruction("mov x8, x9");                                          // child value = right child value
    emitter.label(&format!("{}_one", prefix));
    emitter.instruction("cmp x8, x7");                                          // does the child dominate the sifted value?
    emitter.instruction(&format!("b.{} {}_end", stop_if, prefix));              // no — the sifted value belongs at root
    emitter.instruction("str x8, [x2, x5, lsl #3]");                            // move the child up into the hole
    emitter.instruction("mov x5, x6");                                          // continue from the child's slot
    emitter.instruction(&format!("b {}_loop", prefix));                         // keep sifting down
    emitter.label(&format!("{}_end", prefix));
    emitter.instruction("str x7, [x2, x5, lsl #3]");                            // drop the sifted value into the hole
}

/// x86_64 Linux implementation of the sort-int runtime helpers (in-place heapsort).
/// Uses System V AMD64 ABI: array pointer in `rdi`, indexed-array length in `[rdi]`;
/// clobbers only caller-saved registers.
fn emit_sort_int_linux_x86_64(emitter: &mut Emitter, reverse: bool) {
    let label = if reverse { "__rt_rsort_int" } else { "__rt_sort_int" };
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} (heapsort) ---", label));
    emitter.label_global(label);

    emitter.instruction("mov r8, QWORD PTR [rdi]");                             // r8 = indexed-array length
    emitter.instruction("lea r9, [rdi + 24]");                                  // r9 = payload base
    emitter.instruction("cmp r8, 2");                                           // fewer than two elements?
    emitter.instruction(&format!("jb {}_done", label));                         // already sorted

    // -- phase 1: heapify, sifting down every internal node from n/2 - 1 to 0 --
    emitter.instruction("mov r10, r8");                                         // copy the length
    emitter.instruction("shr r10, 1");                                          // r10 = n / 2 (one past the last internal node)
    emitter.instruction("mov rsi, r8");                                         // rsi = heap limit = n
    emitter.label(&format!("{}_build", label));
    emitter.instruction("test r10, r10");                                       // every internal node sifted?
    emitter.instruction(&format!("jz {}_extract_start", label));                // yes — start extracting
    emitter.instruction("sub r10, 1");                                          // next internal node
    emitter.instruction("mov rcx, r10");                                        // root = node
    emit_sift_down_x86_64(emitter, &format!("{}_bsift", label), reverse);
    emitter.instruction(&format!("jmp {}_build", label));                       // continue heapifying

    // -- phase 2: move the root behind the shrinking heap, then restore the heap --
    emitter.label(&format!("{}_extract_start", label));
    emitter.instruction("lea r10, [r8 - 1]");                                   // r10 = last heap slot
    emitter.label(&format!("{}_extract", label));
    emitter.instruction("test r10, r10");                                       // one element left?
    emitter.instruction(&format!("jz {}_done", label));                         // yes — sorted
    emitter.instruction("mov rax, QWORD PTR [r9]");                             // rax = heap root (extreme element)
    emitter.instruction("mov rdx, QWORD PTR [r9 + r10 * 8]");                   // rdx = last heap element
    emitter.instruction("mov QWORD PTR [r9], rdx");                             // move the last element to the root
    emitter.instruction("mov QWORD PTR [r9 + r10 * 8], rax");                   // park the extreme element in its final slot
    emitter.instruction("mov rsi, r10");                                        // heap limit = slots still in the heap
    emitter.instruction("xor ecx, ecx");                                        // root = 0
    emit_sift_down_x86_64(emitter, &format!("{}_esift", label), reverse);
    emitter.instruction("sub r10, 1");                                          // shrink the heap
    emitter.instruction(&format!("jmp {}_extract", label));                     // extract the next extreme element

    emitter.label(&format!("{}_done", label));
    emitter.instruction("ret");                                                 // return after sorting the payload in place
}

/// Emits an inline x86_64 sift-down of `rcx` (root) within the heap `r9[0..rsi)`.
/// Clobbers `rcx`, `rdx`, `rax`, `rdi`, `r11`.
fn emit_sift_down_x86_64(emitter: &mut Emitter, prefix: &str, reverse: bool) {
    // Ascending sorts keep a max-heap, descending sorts a min-heap.
    let (keep_left_if, stop_if) = if reverse { ("jge", "jge") } else { ("jle", "jle") };
    emitter.instruction("mov r11, QWORD PTR [r9 + rcx * 8]");                   // v = value being sifted down
    emitter.label(&format!("{}_loop", prefix));
    emitter.instruction("lea rdx, [rcx + rcx + 1]");                            // child = 2 * root + 1
    emitter.instruction("cmp rdx, rsi");                                        // is the child inside the heap?
    emitter.instruction(&format!("jae {}_end", prefix));                        // no — root is a leaf
    emitter.instruction("mov rax, QWORD PTR [r9 + rdx * 8]");                   // rax = left child value
    emitter.instruction("lea rdi, [rdx + 1]");                                  // right child index
    emitter.instruction("cmp rdi, rsi");                                        // is there a right child?
    emitter.instruction(&format!("jae {}_one", prefix));                        // no — compare against the left child only
    emitter.instruction("mov rdi, QWORD PTR [r9 + rdx * 8 + 8]");               // rdi = right child value
    emitter.instruction("cmp rdi, rax");                                        // does the right child dominate the left one?
    emitter.instruction(&format!("{} {}_one", keep_left_if, prefix));           // no — keep the left child
    emitter.instruction("add rdx, 1");                                          // child = right child
    emitter.instruction("mov rax, rdi");                                        // child value = right child value
    emitter.label(&format!("{}_one", prefix));
    emitter.instruction("cmp rax, r11");                                        // does the child dominate the sifted value?
    emitter.instruction(&format!("{} {}_end", stop_if, prefix));                // no — the sifted value belongs at root
    emitter.instruction("mov QWORD PTR [r9 + rcx * 8], rax");                   // move the child up into the hole
    emitter.instruction("mov rcx, rdx");                                        // continue from the child's slot
    emitter.instruction(&format!("jmp {}_loop", prefix));                       // keep sifting down
    emitter.label(&format!("{}_end", prefix));
    emitter.instruction("mov QWORD PTR [r9 + rcx * 8], r11");                   // drop the sifted value into the hole
}
