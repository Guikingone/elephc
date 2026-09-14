//! Purpose:
//! Emits the read-only C-ABI query that tells Magician whether one entry of a
//! natively built array still belongs to a live PHP reference set.
//!
//! Called from:
//! - `crate::codegen_support::runtime::eval_bridge::emit_eval_value_runtime()`, once per program.
//!
//! Key details:
//! - `clone($object, $withProperties)` on an eval-owned object is applied by Magician, whose
//!   own array-element alias table only knows arrays eval itself built. An override hash built
//!   in generated code carries its reference state in the hash entry instead, and this query is
//!   how Magician reads it. Without it a by-reference override entry would be applied silently.
//! - The predicate is the one the generated clone-override applicator already uses
//!   (`codegen/lower_inst/builtins/clone_with/overrides.rs`): value tag 7, the persistent
//!   `HASH_ENTRY_REFERENCE_FLAG`, and then either a non-zero direct-local alias count or a
//!   shared boxed cell whose owner count exceeds the caller's own single borrow.
//! - Every input is BORROWED. The query allocates nothing, retains nothing, releases nothing,
//!   writes no output slot, and cannot throw, so it needs no status/output split.
//! - Both supported architectures define the same symbol with the same argument order.

use super::*;
use crate::codegen_support::runtime::{
    HASH_ENTRY_REFERENCE_COUNT_MASK, HASH_ENTRY_REFERENCE_FLAG,
};

/// Runtime tag of a boxed Mixed payload, the only shape that can carry entry reference state.
const MIXED_VALUE_TAG: i64 = 7;

/// Runtime tag of an associative array payload inside a boxed Mixed cell.
const ASSOC_ARRAY_TAG: i64 = 5;

/// Emits `__elephc_eval_array_entry_is_shared_reference_v1` for the active target.
///
/// Arguments, in C order: the borrowed boxed override array, the entry key bytes and their
/// length, and the borrowed boxed entry value the caller already holds (or null). Returns one
/// when that entry still belongs to a live PHP reference set and zero otherwise.
pub(super) fn emit_array_entry_reference_query(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: eval array entry reference query ---");
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Emits the ARM64 query body.
fn emit_aarch64(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_array_entry_is_shared_reference_v1");
    emitter.instruction("sub sp, sp, #48");                                     // frame for the saved inputs and the hash pointer
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address before nested calls
    emitter.instruction("add x29, sp, #32");                                    // establish the query frame
    emitter.instruction("str x3, [sp, #0]");                                    // save the caller's borrowed boxed entry value
    emitter.instruction("str x1, [sp, #8]");                                    // save the entry key bytes pointer
    emitter.instruction("str x2, [sp, #16]");                                   // save the entry key byte length
    emitter.instruction("cbz x0, __elephc_eval_array_entry_ref_no");            // a null array box carries no entry state
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0 = payload tag, x1 = payload low word
    emitter.instruction(&format!("cmp x0, #{ASSOC_ARRAY_TAG}"));                // only associative arrays own hash entries
    emitter.instruction("b.ne __elephc_eval_array_entry_ref_no");               // every other shape is an ordinary by-value override
    emitter.instruction("str x1, [sp, #24]");                                   // save the hash table pointer across key normalization
    emitter.instruction("ldr x1, [sp, #8]");                                    // x1 = key bytes pointer
    emitter.instruction("ldr x2, [sp, #16]");                                   // x2 = key byte length
    emitter.instruction("bl __rt_hash_normalize_key");                          // x1/x2 = normalized key low/high words
    emitter.instruction("ldr x0, [sp, #24]");                                   // x0 = the hash table to probe
    emitter.instruction("bl __rt_hash_get");                                    // x4 = matching entry address, zero on a miss
    emitter.instruction("cbz x4, __elephc_eval_array_entry_ref_no");            // a key with no entry of its own carries no reference metadata
    emitter.instruction("mov x6, x4");                                          // hold the entry address across the payload loads
    emitter.instruction("ldr x3, [x6, #24]");                                   // x3 = value_lo, the entry's shared boxed Mixed cell
    emitter.instruction("ldr x4, [x6, #32]");                                   // x4 = the entry's persistent reference state
    emitter.instruction("ldr x5, [x6, #40]");                                   // x5 = the entry's value tag
    emitter.instruction(&format!("cmp x5, #{MIXED_VALUE_TAG}"));                // reference metadata is valid only beside a boxed Mixed cell
    emitter.instruction("b.ne __elephc_eval_array_entry_ref_no");               // concrete entry values cannot represent PHP references here
    abi::emit_load_int_immediate(emitter, "x9", HASH_ENTRY_REFERENCE_FLAG);
    emitter.instruction("tst x4, x9");                                          // is this entry part of a persistent PHP reference set?
    emitter.instruction("b.eq __elephc_eval_array_entry_ref_no");               // unmarked Mixed values are ordinary by-value overrides
    abi::emit_load_int_immediate(emitter, "x10", HASH_ENTRY_REFERENCE_COUNT_MASK);
    emitter.instruction("and x10, x4, x10");                                    // isolate the live direct-local alias count
    emitter.instruction("cbnz x10, __elephc_eval_array_entry_ref_yes");         // a live alias makes the override assignment by reference
    emitter.instruction("ldr w10, [x3, #-12]");                                 // load the shared boxed Mixed cell's owner count
    emitter.instruction("ldr x11, [sp, #0]");                                   // x11 = the caller's own borrowed entry value
    emitter.instruction("cmp x11, x3");                                         // does the caller's value borrow this very cell?
    emitter.instruction("b.ne __elephc_eval_array_entry_ref_owned");            // an unrelated value leaves the owner count as it stands
    emitter.instruction("sub w10, w10, #1");                                    // discount the caller's own by-value element borrow
    emitter.label("__elephc_eval_array_entry_ref_owned");
    emitter.instruction("cmp w10, #1");                                         // do multiple hash entries still share this reference cell?
    emitter.instruction("b.hi __elephc_eval_array_entry_ref_yes");              // shared entry ownership preserves PHP reference identity
    emitter.label("__elephc_eval_array_entry_ref_no");
    emitter.instruction("mov x0, xzr");                                         // report an ordinary by-value override entry
    emitter.instruction("b __elephc_eval_array_entry_ref_ret");                 // fall into the shared epilogue
    emitter.label("__elephc_eval_array_entry_ref_yes");
    emitter.instruction("mov x0, #1");                                          // report a live PHP reference set
    emitter.label("__elephc_eval_array_entry_ref_ret");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the query frame
    emitter.instruction("ret");                                                 // return the borrowed-only answer to Rust
}

/// Emits the x86_64 query body.
fn emit_x86_64(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_array_entry_is_shared_reference_v1");
    emitter.instruction("push rbp");                                            // save the caller frame pointer and realign the stack
    emitter.instruction("mov rbp, rsp");                                        // establish the query frame
    emitter.instruction("sub rsp, 48");                                         // reserve spill slots for the saved inputs
    emitter.instruction("mov QWORD PTR [rbp - 8], rcx");                        // save the caller's borrowed boxed entry value
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the entry key bytes pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save the entry key byte length
    emitter.instruction("test rdi, rdi");                                       // a null array box carries no entry state
    emitter.instruction("jz __elephc_eval_array_entry_ref_no_x86");             // report an ordinary override for a missing container
    emitter.instruction("mov rax, rdi");                                        // the unbox helper takes its boxed input in rax
    emitter.instruction("call __rt_mixed_unbox");                               // rax = payload tag, rdi = payload low word
    emitter.instruction(&format!("cmp rax, {ASSOC_ARRAY_TAG}"));                // only associative arrays own hash entries
    emitter.instruction("jne __elephc_eval_array_entry_ref_no_x86");            // every other shape is an ordinary by-value override
    emitter.instruction("mov QWORD PTR [rbp - 32], rdi");                       // save the hash table pointer across key normalization
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // rax = key bytes pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // rdx = key byte length
    emitter.instruction("call __rt_hash_normalize_key");                        // rax/rdx = normalized key low/high words
    emitter.instruction("mov rsi, rax");                                        // move the normalized key low word into the lookup ABI register
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // rdi = the hash table to probe
    emitter.instruction("call __rt_hash_get");                                  // r8 = matching entry address, zero on a miss
    emitter.instruction("test r8, r8");                                         // a key with no entry of its own carries no reference metadata
    emitter.instruction("jz __elephc_eval_array_entry_ref_no_x86");             // absent keys are ordinary by-value overrides
    emitter.instruction("mov r10, r8");                                         // hold the entry address across the payload loads
    emitter.instruction("mov rcx, QWORD PTR [r10 + 24]");                       // rcx = value_lo, the entry's shared boxed Mixed cell
    emitter.instruction("mov r8, QWORD PTR [r10 + 32]");                        // r8 = the entry's persistent reference state
    emitter.instruction("mov r9, QWORD PTR [r10 + 40]");                        // r9 = the entry's value tag
    emitter.instruction(&format!("cmp r9, {MIXED_VALUE_TAG}"));                 // reference metadata is valid only beside a boxed Mixed cell
    emitter.instruction("jne __elephc_eval_array_entry_ref_no_x86");            // concrete entry values cannot represent PHP references here
    abi::emit_load_int_immediate(emitter, "r11", HASH_ENTRY_REFERENCE_FLAG);
    emitter.instruction("test r8, r11");                                        // is this entry part of a persistent PHP reference set?
    emitter.instruction("jz __elephc_eval_array_entry_ref_no_x86");             // unmarked Mixed values are ordinary by-value overrides
    abi::emit_load_int_immediate(emitter, "r11", HASH_ENTRY_REFERENCE_COUNT_MASK);
    emitter.instruction("mov r10, r8");                                         // copy the reference state before masking its flag
    emitter.instruction("and r10, r11");                                        // isolate the live direct-local alias count
    emitter.instruction("jnz __elephc_eval_array_entry_ref_yes_x86");           // a live alias makes the override assignment by reference
    emitter.instruction("mov r10d, DWORD PTR [rcx - 12]");                      // load the shared boxed Mixed cell's owner count
    emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                        // r11 = the caller's own borrowed entry value
    emitter.instruction("cmp r11, rcx");                                        // does the caller's value borrow this very cell?
    emitter.instruction("jne __elephc_eval_array_entry_ref_owned_x86");         // an unrelated value leaves the owner count as it stands
    emitter.instruction("sub r10d, 1");                                         // discount the caller's own by-value element borrow
    emitter.label("__elephc_eval_array_entry_ref_owned_x86");
    emitter.instruction("cmp r10d, 1");                                         // do multiple hash entries still share this reference cell?
    emitter.instruction("ja __elephc_eval_array_entry_ref_yes_x86");            // shared entry ownership preserves PHP reference identity
    emitter.label("__elephc_eval_array_entry_ref_no_x86");
    emitter.instruction("xor eax, eax");                                        // report an ordinary by-value override entry
    emitter.instruction("jmp __elephc_eval_array_entry_ref_ret_x86");           // fall into the shared epilogue
    emitter.label("__elephc_eval_array_entry_ref_yes_x86");
    emitter.instruction("mov eax, 1");                                          // report a live PHP reference set
    emitter.label("__elephc_eval_array_entry_ref_ret_x86");
    emitter.instruction("mov rsp, rbp");                                        // discard the spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the borrowed-only answer to Rust
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The query keeps one C symbol, both refusal sources, and its hash probe on every target.
    #[test]
    fn eval_array_entry_reference_query_is_symmetric_on_every_target() {
        for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
            let target = crate::codegen_support::platform::Target::parse(name).unwrap();
            let mut emitter = Emitter::new(target);
            emit_array_entry_reference_query(&mut emitter);
            let output = emitter.output();
            let symbol = target.extern_symbol("__elephc_eval_array_entry_is_shared_reference_v1");
            assert_eq!(output.matches(&format!("{symbol}:")).count(), 1, "{name}");
            // The query must reach the real hash entry rather than guess from the container.
            assert!(output.contains("__rt_mixed_unbox"), "{name}");
            assert!(output.contains("__rt_hash_normalize_key"), "{name}");
            assert!(output.contains("__rt_hash_get"), "{name}");
            // Both halves of the refusal predicate have to survive on both architectures.
            let (alias_count_branch, shared_owner_branch) = match target.arch {
                Arch::AArch64 => (
                    "cbnz x10, __elephc_eval_array_entry_ref_yes",
                    "b.hi __elephc_eval_array_entry_ref_yes",
                ),
                Arch::X86_64 => (
                    "jnz __elephc_eval_array_entry_ref_yes_x86",
                    "ja __elephc_eval_array_entry_ref_yes_x86",
                ),
            };
            assert!(output.contains(alias_count_branch), "{name}");
            assert!(output.contains(shared_owner_branch), "{name}");
            // A pure read must not allocate, retain, release, or throw.
            for forbidden in [
                "__rt_heap_alloc",
                "__rt_incref",
                "__rt_mixed_release",
                "__rt_throw_current",
            ] {
                assert!(!output.contains(forbidden), "{name}: {forbidden}");
            }
        }
    }
}
