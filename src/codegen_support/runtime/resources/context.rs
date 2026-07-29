//! Purpose:
//! Emits typed lookup for resource-registry-backed stream contexts.
//! The helper hides registry offsets from context API lowering.
//!
//! Called from:
//! - `crate::codegen_support::runtime::resources::emit_resource_runtime()`.
//!
//! Key details:
//! - Only Live resources of kind Context resolve to state storage.
//! - Owned states defensively release retained options, reserved params, and
//!   notifier children before freeing the 32-byte state allocation.

use super::layout::{
    CONTEXT_NOTIFIER_OFFSET, CONTEXT_OPTIONS_OFFSET, CONTEXT_PARAMS_OFFSET,
    RESOURCE_KIND_CONTEXT, RESOURCE_STATUS_LIVE, SLOT_KIND_OFFSET, SLOT_STATE_PTR_OFFSET,
    SLOT_STATUS_OFFSET,
};
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the target-specific `__rt_context_state` typed lookup helper.
pub(super) fn emit_context_resources(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emit_context_state_aarch64(emitter);
            emit_context_destroy_state_aarch64(emitter);
        }
        Arch::X86_64 => {
            emit_context_state_x86_64(emitter);
            emit_context_destroy_state_x86_64(emitter);
        }
    }
}

/// Emits AArch64 Live Context state lookup.
fn emit_context_state_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: resolve an opaque stream-context state ---");
    emitter.label_global("__rt_context_state");
    emitter.instruction("sub sp, sp, #16");                                     // preserve the link register around generic lookup
    emitter.instruction("str x30, [sp, #8]");                                   // save the caller link register
    emitter.instruction("bl __rt_resource_lookup_any");                         // validate and resolve the opaque handle
    emitter.instruction("cbz x0, __rt_context_state_fail");                     // reject invalid or stale resources
    emitter.instruction(&format!(
        "ldr x9, [x0, #{}]", SLOT_KIND_OFFSET
    ));                                                                         // load the registry resource kind
    emitter.instruction(&format!(
        "cmp x9, #{}", RESOURCE_KIND_CONTEXT
    ));                                                                         // is the slot a stream context?
    emitter.instruction("b.ne __rt_context_state_fail");                        // reject streams, filters, and other resources
    emitter.instruction(&format!(
        "ldr x9, [x0, #{}]", SLOT_STATUS_OFFSET
    ));                                                                         // load the context lifecycle state
    emitter.instruction(&format!(
        "cmp x9, #{}", RESOURCE_STATUS_LIVE
    ));                                                                         // only Live contexts expose their state
    emitter.instruction("b.ne __rt_context_state_fail");                        // reject Closing and Closed contexts
    emitter.instruction(&format!(
        "ldr x0, [x0, #{}]", SLOT_STATE_PTR_OFFSET
    ));                                                                         // return the stable context-state pointer
    emitter.instruction("b __rt_context_state_done");                           // join the helper epilogue
    emitter.label("__rt_context_state_fail");
    emitter.instruction("mov x0, #0");                                          // return null for invalid context resources
    emitter.label("__rt_context_state_done");
    emitter.instruction("ldr x30, [sp, #8]");                                   // restore the caller link register
    emitter.instruction("add sp, sp, #16");                                     // release the aligned link-register save
    emitter.instruction("ret");                                                 // return the context-state pointer or null
}

/// Emits Linux x86_64 Live Context state lookup.
fn emit_context_state_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: resolve an opaque stream-context state ---");
    emitter.label_global("__rt_context_state");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable context lookup frame
    emitter.instruction("call __rt_resource_lookup_any");                       // validate and resolve the opaque handle
    emitter.instruction("test rax, rax");                                       // did lookup resolve a registry slot?
    emitter.instruction("jz __rt_context_state_fail");                          // reject invalid or stale resources
    emitter.instruction(&format!(
        "cmp QWORD PTR [rax + {}], {}",
        SLOT_KIND_OFFSET, RESOURCE_KIND_CONTEXT
    ));                                                                         // is the slot a stream context?
    emitter.instruction("jne __rt_context_state_fail");                         // reject streams, filters, and other resources
    emitter.instruction(&format!(
        "cmp QWORD PTR [rax + {}], {}",
        SLOT_STATUS_OFFSET, RESOURCE_STATUS_LIVE
    ));                                                                         // only Live contexts expose their state
    emitter.instruction("jne __rt_context_state_fail");                         // reject Closing and Closed contexts
    emitter.instruction(&format!(
        "mov rax, QWORD PTR [rax + {}]",
        SLOT_STATE_PTR_OFFSET
    ));                                                                         // return the stable context-state pointer
    emitter.instruction("jmp __rt_context_state_done");                         // join the helper epilogue
    emitter.label("__rt_context_state_fail");
    emitter.instruction("xor eax, eax");                                        // return null for invalid context resources
    emitter.label("__rt_context_state_done");
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the context-state pointer or null
}

/// Emits AArch64 owned ContextState teardown in child-before-parent order.
fn emit_context_destroy_state_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: destroy an owned stream-context state ---");
    emitter.label_global("__rt_context_destroy_state");
    emitter.instruction("cbz x0, __rt_context_destroy_state_done");             // null context states own no children or storage
    emitter.instruction("sub sp, sp, #32");                                     // reserve stable state storage and a saved frame
    emitter.instruction("stp x29, x30, [sp, #16]");                             // preserve the caller frame and link register
    emitter.instruction("add x29, sp, #16");                                    // establish a stable teardown frame
    emitter.instruction("str x0, [sp, #0]");                                    // preserve ContextState across nested releases
    emitter.instruction(&format!(
        "ldr x0, [x0, #{}]",
        CONTEXT_OPTIONS_OFFSET
    ));                                                                         // load the retained options hash
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload ContextState before clearing ownership
    emitter.instruction(&format!(
        "str xzr, [x9, #{}]",
        CONTEXT_OPTIONS_OFFSET
    ));                                                                         // detach options before potentially re-entrant release
    emitter.instruction("cbz x0, __rt_context_destroy_state_params");           // skip an absent options value
    emitter.instruction("bl __rt_decref_any");                                  // release the retained options hash
    emitter.label("__rt_context_destroy_state_params");
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload ContextState for reserved params teardown
    emitter.instruction(&format!(
        "ldr x0, [x9, #{}]",
        CONTEXT_PARAMS_OFFSET
    ));                                                                         // load the defensively retained params payload
    emitter.instruction(&format!(
        "str xzr, [x9, #{}]",
        CONTEXT_PARAMS_OFFSET
    ));                                                                         // detach params before its potentially re-entrant release
    emitter.instruction("cbz x0, __rt_context_destroy_state_notifier");         // skip the normally empty reserved params slot
    emitter.instruction("bl __rt_decref_any");                                  // release a future retained params payload exactly once
    emitter.label("__rt_context_destroy_state_notifier");
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload ContextState for notifier teardown
    emitter.instruction(&format!(
        "ldr x0, [x9, #{}]",
        CONTEXT_NOTIFIER_OFFSET
    ));                                                                         // load the retained notification descriptor
    emitter.instruction(&format!(
        "str xzr, [x9, #{}]",
        CONTEXT_NOTIFIER_OFFSET
    ));                                                                         // detach notifier before its potentially re-entrant release
    emitter.instruction("cbz x0, __rt_context_destroy_state_storage");          // skip an absent notification descriptor
    emitter.instruction("bl __rt_callable_descriptor_release");                 // release the retained callable descriptor
    emitter.label("__rt_context_destroy_state_storage");
    emitter.instruction("ldr x0, [sp, #0]");                                    // pass ContextState itself to the heap allocator
    emitter.instruction("bl __rt_heap_free");                                   // release the owned 32-byte state allocation
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore the caller frame and link register
    emitter.instruction("add sp, sp, #32");                                     // release teardown scratch storage
    emitter.label("__rt_context_destroy_state_done");
    emitter.instruction("ret");                                                 // return after exact-once child and state teardown
}

/// Emits Linux x86_64 owned ContextState teardown in child-before-parent order.
fn emit_context_destroy_state_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: destroy an owned stream-context state ---");
    emitter.label_global("__rt_context_destroy_state");
    emitter.instruction("test rax, rax");                                       // do null context states own any storage?
    emitter.instruction("jz __rt_context_destroy_state_done");                  // no, return without entering a teardown frame
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable teardown frame
    emitter.instruction("sub rsp, 16");                                         // reserve aligned ContextState storage
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve ContextState across nested releases
    emitter.instruction(&format!(
        "mov rax, QWORD PTR [rax + {}]",
        CONTEXT_OPTIONS_OFFSET
    ));                                                                         // load the retained options hash
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload ContextState before clearing ownership
    emitter.instruction(&format!(
        "mov QWORD PTR [r10 + {}], 0",
        CONTEXT_OPTIONS_OFFSET
    ));                                                                         // detach options before potentially re-entrant release
    emitter.instruction("test rax, rax");                                       // was an options hash retained?
    emitter.instruction("jz __rt_context_destroy_state_params");                // skip an absent options value
    emitter.instruction("call __rt_decref_any");                                // release the retained options hash
    emitter.label("__rt_context_destroy_state_params");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload ContextState for reserved params teardown
    emitter.instruction(&format!(
        "mov rax, QWORD PTR [r10 + {}]",
        CONTEXT_PARAMS_OFFSET
    ));                                                                         // load the defensively retained params payload
    emitter.instruction(&format!(
        "mov QWORD PTR [r10 + {}], 0",
        CONTEXT_PARAMS_OFFSET
    ));                                                                         // detach params before its potentially re-entrant release
    emitter.instruction("test rax, rax");                                       // does the reserved params slot own a payload?
    emitter.instruction("jz __rt_context_destroy_state_notifier");              // skip the normally empty reserved params slot
    emitter.instruction("call __rt_decref_any");                                // release a future retained params payload exactly once
    emitter.label("__rt_context_destroy_state_notifier");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload ContextState for notifier teardown
    emitter.instruction(&format!(
        "mov rax, QWORD PTR [r10 + {}]",
        CONTEXT_NOTIFIER_OFFSET
    ));                                                                         // load the retained notification descriptor
    emitter.instruction(&format!(
        "mov QWORD PTR [r10 + {}], 0",
        CONTEXT_NOTIFIER_OFFSET
    ));                                                                         // detach notifier before its potentially re-entrant release
    emitter.instruction("test rax, rax");                                       // was a notification descriptor retained?
    emitter.instruction("jz __rt_context_destroy_state_storage");               // skip an absent notification descriptor
    emitter.instruction("call __rt_callable_descriptor_release");               // release the retained callable descriptor
    emitter.label("__rt_context_destroy_state_storage");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // pass ContextState itself to the heap allocator
    emitter.instruction("call __rt_heap_free");                                 // release the owned 32-byte state allocation
    emitter.instruction("add rsp, 16");                                         // release teardown scratch storage
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.label("__rt_context_destroy_state_done");
    emitter.instruction("ret");                                                 // return after exact-once child and state teardown
}
