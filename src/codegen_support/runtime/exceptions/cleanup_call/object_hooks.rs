//! Purpose:
//! Proves that one per-object release hook will return without running PHP code.
//!
//! Called from:
//! - `super::empty_array` after its typed-array callback authentication rejects an entry.
//!
//! Key details:
//! - Every object reclamation passes three hooks through `__rt_cleanup_invoke`: the destructor
//!   dispatcher, the eval child release and the WeakMap observer notification. For ordinary
//!   objects all three return immediately, yet each paid a fresh jump buffer.
//! - The gate repeats exactly the early-out tests those hooks perform themselves (no destructor
//!   for the class and no eval dispatcher, no eval release hook, no weak observer for the key).
//! - An admitted hook still runs through the shared direct stub, so every side effect it has on
//!   the way out (such as the destructor-invoked marker) is unchanged.
//! - A native release dispatcher whose validated payload still has another owner (refcount of
//!   at least two, no destruction guard bit) only decrements and notes a collector candidate;
//!   no storage is freed, so no PHP cleanup can run and it also takes the direct stub.
//! - Anything else, including a class with `__destruct`, keeps the protected boundary.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

pub(super) const CANDIDATE: &str = "__rt_cleanup_invoke_object_hook_candidate";
/// Entry for a validated managed payload whose release may be a shared owner's plain decrement.
pub(super) const SHARED: &str = "__rt_cleanup_invoke_shared_owner";

/// Emits the object-hook proofs, falling back to the protected path on any doubt.
pub(super) fn emit_candidate(emitter: &mut Emitter, protected: &str, direct: &str) {
    emitter.label(CANDIDATE);
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter, protected, direct),
        Arch::X86_64 => emit_x86(emitter, protected, direct),
    }
}

/// Preserves the AAPCS64 inputs (x0 entry, x1 payload, x2 pending slot) while proving a hook inert.
fn emit_aarch64(emitter: &mut Emitter, protected: &str, direct: &str) {
    let release = "__rt_cleanup_invoke_object_hook_release";
    let weak = "__rt_cleanup_invoke_object_hook_weak";
    // -- destructor dispatcher: no eval dispatcher and no class destructor --
    abi::emit_symbol_address(emitter, "x10", "__rt_call_object_destructor");
    emitter.instruction("cmp x0, x10");                                         // is this the per-object destructor hook?
    emitter.instruction(&format!("b.ne {release}"));                            // other hooks carry their own proof
    emitter.instruction(&format!("cbz x1, {direct}"));                          // a null receiver returns before any PHP call
    abi::emit_load_symbol_to_reg(emitter, "x10", "_elephc_eval_dynamic_object_destruct_fn", 0);
    emitter.instruction(&format!("cbnz x10, {protected}"));                     // eval-owned objects may run dynamic destructors
    emitter.instruction("ldr x11, [x1]");                                       // the runtime class id leads the object payload
    abi::emit_load_symbol_to_reg(emitter, "x10", "_class_destruct_count", 0);
    emitter.instruction("cmp x11, x10");                                        // ids past the table have no destructor
    emitter.instruction(&format!("b.hs {direct}"));                             // the hook returns without user code
    abi::emit_symbol_address(emitter, "x10", "_class_destruct_ptrs");
    emitter.instruction("ldr x10, [x10, x11, lsl #3]");                         // load this class's destructor entry (or zero)
    emitter.instruction(&format!("cbz x10, {direct}"));                         // no __destruct: nothing in the hook can throw
    emitter.instruction(&format!("b {protected}"));                             // a real destructor keeps the exception boundary

    // -- eval child release: only an installed eval hook can run PHP --
    emitter.label(release);
    abi::emit_symbol_address(emitter, "x10", "__rt_eval_object_release_children");
    emitter.instruction("cmp x0, x10");                                         // is this the eval child-release hook?
    emitter.instruction(&format!("b.ne {weak}"));                               // try the WeakMap observer hook next
    abi::emit_load_symbol_to_reg(emitter, "x10", "_elephc_eval_object_release_fn", 0);
    emitter.instruction(&format!("cbz x10, {direct}"));                         // programs without eval release no external children
    emitter.instruction(&format!("b {protected}"));                             // eval-owned children may run PHP cleanup

    // -- WeakMap notification: only an observed key reaches a PHP callback --
    emitter.label(weak);
    abi::emit_symbol_address(emitter, "x10", "__rt_weak_map_notify");
    emitter.instruction("cmp x0, x10");                                         // is this the weak observer hook?
    emitter.instruction("b.ne __rt_cleanup_invoke_object_hook_decref");         // native release dispatchers may still qualify as shared
    abi::emit_symbol_address(emitter, "x10", "_heap_buf");
    emitter.instruction("cmp x1, x10");                                         // the observer index covers managed storage only
    emitter.instruction(&format!("b.lo {protected}"));                          // foreign payloads keep the original handling
    emitter.instruction("sub x11, x1, x10");                                    // derive the key's heap-relative offset
    emitter.instruction("lsr x11, x11, #4");                                    // select its weak observer granule, as the hook does
    abi::emit_symbol_address(emitter, "x10", "_obj_weak_reference_index");
    emitter.instruction("ldr x10, [x10, x11, lsl #3]");                         // load the observer entry for this key
    emitter.instruction(&format!("cbz x10, {direct}"));                         // unobserved keys notify nobody
    emitter.instruction(&format!("b {protected}"));                             // observed keys may call WeakMap PHP code

    // -- typed release dispatchers: only a shared owner's decrement is admitted --
    emitter.label("__rt_cleanup_invoke_object_hook_decref");
    for (index, entry) in ["__rt_decref_object", "__rt_decref_hash", "__rt_decref_mixed"].iter().enumerate() {
        abi::emit_symbol_address(emitter, "x10", entry);
        emitter.instruction("cmp x0, x10");                                     // is this a native release dispatcher?
        emitter.instruction(&format!("b.eq __rt_cleanup_invoke_object_hook_range_{index}")); // validate its payload before reading the owner count
    }
    emitter.instruction(&format!("b {protected}"));                             // arbitrary callbacks keep the protected path
    for index in 0..3 {
        emitter.label(&format!("__rt_cleanup_invoke_object_hook_range_{index}"));
    }
    abi::emit_symbol_address(emitter, "x9", "_heap_buf");
    emitter.instruction("add x10, x9, #16");                                    // the first payload follows the uniform heap header
    emitter.instruction("cmp x1, x10");                                         // reject null, literals and foreign payloads
    emitter.instruction(&format!("b.lo {protected}"));                          // do not read a header below managed storage
    abi::emit_symbol_address(emitter, "x10", "_heap_off");
    emitter.instruction("ldr x10, [x10]");                                      // load the bump offset while keeping the base in x9
    emitter.instruction("add x10, x9, x10");                                    // derive the allocated heap end
    emitter.instruction("cmp x1, x10");                                         // require a payload inside allocated storage
    emitter.instruction(&format!("b.hs {protected}"));                          // out-of-range values retain existing behavior

    // -- validated payload: two or more real owners means the release only decrements --
    emitter.label(SHARED);
    emitter.instruction("ldr w10, [x1, #-12]");                                 // borrow the owner count without changing it
    emitter.instruction("cmp w10, #2");                                         // the releasing owner must not be the last one
    emitter.instruction(&format!("b.lo {protected}"));                          // a final release may run destructors
    emitter.instruction(&format!("tbnz w10, #31, {protected}"));                // a destruction in progress keeps its own protocol
    emitter.instruction(&format!("b {direct}"));                                // a plain decrement cannot run PHP code
}

/// Preserves the System V inputs (rdi entry, rsi payload, rdx pending slot) while proving a hook inert.
fn emit_x86(emitter: &mut Emitter, protected: &str, direct: &str) {
    let release = "__rt_cleanup_invoke_object_hook_release";
    let weak = "__rt_cleanup_invoke_object_hook_weak";
    // -- destructor dispatcher: no eval dispatcher and no class destructor --
    abi::emit_symbol_address(emitter, "r9", "__rt_call_object_destructor");
    emitter.instruction("cmp rdi, r9");                                         // is this the per-object destructor hook?
    emitter.instruction(&format!("jne {release}"));                             // other hooks carry their own proof
    emitter.instruction("test rsi, rsi");                                       // a null receiver returns before any PHP call
    emitter.instruction(&format!("jz {direct}"));                               // nothing to destruct
    abi::emit_load_symbol_to_reg(emitter, "r10", "_elephc_eval_dynamic_object_destruct_fn", 0);
    emitter.instruction("test r10, r10");                                       // is an eval dynamic destructor dispatcher installed?
    emitter.instruction(&format!("jnz {protected}"));                           // eval-owned objects may run dynamic destructors
    emitter.instruction("mov r10, QWORD PTR [rsi]");                            // the runtime class id leads the object payload
    abi::emit_cmp_reg_to_symbol(emitter, "r10", "_class_destruct_count");       // ids past the table have no destructor
    emitter.instruction(&format!("jae {direct}"));                              // the hook returns without user code
    abi::emit_symbol_address(emitter, "r11", "_class_destruct_ptrs");
    emitter.instruction("cmp QWORD PTR [r11 + r10 * 8], 0");                    // does this class define __destruct?
    emitter.instruction(&format!("je {direct}"));                               // no destructor: nothing in the hook can throw
    emitter.instruction(&format!("jmp {protected}"));                           // a real destructor keeps the exception boundary

    // -- eval child release: only an installed eval hook can run PHP --
    emitter.label(release);
    abi::emit_symbol_address(emitter, "r9", "__rt_eval_object_release_children");
    emitter.instruction("cmp rdi, r9");                                         // is this the eval child-release hook?
    emitter.instruction(&format!("jne {weak}"));                                // try the WeakMap observer hook next
    abi::emit_load_symbol_to_reg(emitter, "r10", "_elephc_eval_object_release_fn", 0);
    emitter.instruction("test r10, r10");                                       // is the eval release hook installed?
    emitter.instruction(&format!("jz {direct}"));                               // programs without eval release no external children
    emitter.instruction(&format!("jmp {protected}"));                           // eval-owned children may run PHP cleanup

    // -- WeakMap notification: only an observed key reaches a PHP callback --
    emitter.label(weak);
    abi::emit_symbol_address(emitter, "r9", "__rt_weak_map_notify");
    emitter.instruction("cmp rdi, r9");                                         // is this the weak observer hook?
    emitter.instruction("jne __rt_cleanup_invoke_object_hook_decref");          // native release dispatchers may still qualify as shared
    abi::emit_symbol_address(emitter, "r9", "_heap_buf");
    emitter.instruction("cmp rsi, r9");                                         // the observer index covers managed storage only
    emitter.instruction(&format!("jb {protected}"));                            // foreign payloads keep the original handling
    emitter.instruction("mov r10, rsi");                                        // copy the key before deriving its granule
    emitter.instruction("sub r10, r9");                                         // derive the key's heap-relative offset
    emitter.instruction("shr r10, 4");                                          // select its weak observer granule, as the hook does
    abi::emit_symbol_address(emitter, "r11", "_obj_weak_reference_index");
    emitter.instruction("cmp QWORD PTR [r11 + r10 * 8], 0");                    // is any WeakMap observing this key?
    emitter.instruction(&format!("je {direct}"));                               // unobserved keys notify nobody
    emitter.instruction(&format!("jmp {protected}"));                           // observed keys may call WeakMap PHP code

    // -- typed release dispatchers: only a shared owner's decrement is admitted --
    emitter.label("__rt_cleanup_invoke_object_hook_decref");
    for entry in ["__rt_decref_object", "__rt_decref_hash", "__rt_decref_mixed"] {
        abi::emit_symbol_address(emitter, "r9", entry);
        emitter.instruction("cmp rdi, r9");                                     // is this a native release dispatcher?
        emitter.instruction("je __rt_cleanup_invoke_object_hook_range");        // validate its payload before reading the owner count
    }
    emitter.instruction(&format!("jmp {protected}"));                           // arbitrary callbacks keep the protected path
    emitter.label("__rt_cleanup_invoke_object_hook_range");
    abi::emit_symbol_address(emitter, "r9", "_heap_buf");
    emitter.instruction("lea r10, [r9 + 16]");                                  // locate the first valid managed payload
    emitter.instruction("cmp rsi, r10");                                        // reject null, static and sentinel payloads
    emitter.instruction(&format!("jb {protected}"));                            // leave foreign values on the protected path
    abi::emit_load_symbol_to_reg(emitter, "r10", "_heap_off", 0);
    emitter.instruction("add r10, r9");                                         // derive the current heap end from its bump offset
    emitter.instruction("cmp rsi, r10");                                        // require a payload inside allocated storage
    emitter.instruction(&format!("jae {protected}"));                           // out-of-range values cannot qualify
    emitter.instruction("mov r10, QWORD PTR [rsi - 8]");                        // load the stamped kind word to authenticate the marker
    emitter.instruction("shr r10, 32");                                         // isolate the x86 managed-heap marker
    emitter.instruction(&format!("cmp r10, {}", crate::codegen_support::sentinels::X86_64_HEAP_MAGIC_HI32)); // reject foreign headers inside the managed range
    emitter.instruction(&format!("jne {protected}"));                           // only stamped native allocations can qualify

    // -- validated payload: two or more real owners means the release only decrements --
    emitter.label(SHARED);
    emitter.instruction("mov r10d, DWORD PTR [rsi - 12]");                      // borrow the owner count without changing it
    emitter.instruction("cmp r10d, 2");                                         // the releasing owner must not be the last one
    emitter.instruction(&format!("jb {protected}"));                            // a final release may run destructors
    emitter.instruction("test r10d, 0x80000000");                               // is a destruction in progress on this payload?
    emitter.instruction(&format!("jnz {protected}"));                           // keep that protocol on the protected path
    emitter.instruction(&format!("jmp {direct}"));                              // a plain decrement cannot run PHP code
}
