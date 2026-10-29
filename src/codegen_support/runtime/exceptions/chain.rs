//! Purpose:
//! Appends an owned previous exception without overwriting existing links or creating cycles.
//!
//! Called from:
//! - Protected cycle-collector destructor callbacks.
//!
//! Key details:
//! - The new exception is borrowed and rooted by the caller; the old exception owner is consumed.
//! - Both chains are traversed through the concrete-layout previous reader.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits exception chaining with raw-object and boxed-property ownership on every target.
pub fn emit_exception_chain(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Consumes the ARM64 old exception in x1, appending it to the borrowed exception in x0.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.label_global("__rt_exception_chain");
    // -- preserve chain cursors and the incoming owner across metadata lookups --
    emitter.instruction("sub sp, sp, #64");                                     // reserve cursors, a property slot, and frame linkage
    emitter.instruction("stp x29, x30, [sp, #48]");                             // preserve the caller across chain traversal
    emitter.instruction("add x29, sp, #48");                                    // establish the helper frame
    emitter.instruction("str x0, [sp]");                                        // start at the new exception whose caller roots the chain
    emitter.instruction("str x1, [sp, #8]");                                    // own the old exception until a link adopts it or it is released
    emitter.instruction("cbz x1, __rt_exception_chain_return");                 // there is no previous owner to attach
    emitter.instruction("ldr x9, [x1]");                                        // internal force-close markers are never PHP-visible previous exceptions
    emitter.instruction("cmn x9, #3");                                          // recognize the reserved forced-unwind class identity
    emitter.instruction("b.eq __rt_exception_chain_release");                   // consume the marker instead of attaching it to a real Throwable
    if emitter.phpext_output {
        emitter.instruction("cbnz x0, __rt_exception_chain_scan");              // native exceptions can adopt the older native owner directly
        emitter.instruction("ldr x0, [sp, #8]");                                // expose the native owner before the host escape consumes it
        emitter.bl_c("elephc_php_previous_exception");                           // copy the previous Throwable into malloc-owned transport
        emitter.instruction("ldr x0, [sp, #8]");                                // begin walking every previous link owned by that copied root
        emitter.label("__rt_exception_chain_copy_previous");
        emitter.instruction("bl __rt_throwable_previous");                      // borrow the next native previous link without retaining it
        emitter.instruction("cbz x0, __rt_exception_chain_release");            // finish once the full native chain has been copied
        emitter.instruction("str x0, [sp, #40]");                               // preserve the current node across the C callback
        emitter.bl_c("elephc_php_exception_previous");                           // append this node to the active malloc-owned copied chain
        emitter.instruction("ldr x0, [sp, #40]");                               // continue traversal from the node just copied
        emitter.instruction("b __rt_exception_chain_copy_previous");            // preserve every constructor and finally-chain link
    }
    emitter.label("__rt_exception_chain_scan");
    emitter.instruction("ldr x0, [sp, #8]");                                    // restart the old chain for this new-chain node
    emitter.label("__rt_exception_chain_ancestors");
    emitter.instruction("ldr x9, [sp]");                                        // borrow the current node of the new chain
    emitter.instruction("cmp x0, x9");                                          // does joining the chains repeat this object identity?
    emitter.instruction("b.eq __rt_exception_chain_release");                   // consume a redundant owner without creating a cycle
    emitter.instruction("bl __rt_throwable_previous");                          // follow the old chain without acquiring an extra owner
    emitter.instruction("cbnz x0, __rt_exception_chain_ancestors");             // inspect every old-chain ancestor for an overlap
    emitter.instruction("ldr x0, [sp]");                                        // recover the current new-chain node
    emitter.instruction("bl __rt_throwable_previous");                          // keep any previous link that was supplied by user code
    emitter.instruction("cbz x0, __rt_exception_chain_append");                 // append only at the existing chain's tail
    emitter.instruction("str x0, [sp]");                                        // advance through the borrowed new chain
    emitter.instruction("b __rt_exception_chain_scan");                         // check the next node against every old-chain ancestor

    // -- adopt the old owner using the actual previous-slot representation --
    emitter.label("__rt_exception_chain_append");
    emitter.instruction("ldr x0, [sp]");                                        // recover the tail whose previous link is empty
    emitter.instruction("bl __rt_throwable_previous_slot");                     // resolve raw, boxed, and reference property storage
    emitter.instruction("cbz x0, __rt_exception_chain_release");                // absent storage cannot adopt an owner
    emitter.instruction("cbz x1, __rt_exception_chain_raw");                    // raw slots adopt the existing object owner directly
    emitter.instruction("str x0, [sp, #24]");                                   // preserve the effective slot across Mixed allocation
    emitter.instruction("ldr x9, [x0]");                                        // borrow the existing null box before replacement
    emitter.instruction("str x9, [sp, #32]");                                   // release the old null box only after publishing the new one
    emitter.instruction("ldr x1, [sp, #8]");                                    // supply the owned old exception as the boxed payload
    emitter.instruction("mov x0, #6");                                          // runtime tag six denotes an object payload
    emitter.instruction("mov x2, #0");                                          // object values have no high payload word
    emitter.instruction("bl __rt_mixed_from_value");                            // create a Mixed box with its own retained object owner
    emitter.instruction("ldr x9, [sp, #24]");                                   // recover the effective previous-property slot
    emitter.instruction("str x0, [x9]");                                        // publish the box before releasing either displaced owner
    emitter.instruction("ldr x0, [sp, #32]");                                   // recover the displaced null box
    emitter.instruction("bl __rt_decref_any");                                  // release the box that no longer belongs to the property
    emitter.instruction("b __rt_exception_chain_release");                      // balance the raw owner now retained by the new box
    emitter.label("__rt_exception_chain_raw");
    emitter.instruction("ldr x9, [sp, #8]");                                    // recover the raw owner to transfer
    emitter.instruction("str x9, [x0]");                                        // the previous link adopts it without a redundant retain
    emitter.label("__rt_exception_chain_return");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore the caller after successful ownership transfer
    emitter.instruction("add sp, sp, #64");                                     // release traversal storage
    emitter.instruction("ret");                                                 // return with no owner left in this helper
    emitter.label("__rt_exception_chain_release");
    emitter.instruction("ldr x0, [sp, #8]");                                    // recover the redundant or reboxed raw owner
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore the caller before a possible last-owner destructor
    emitter.instruction("add sp, sp, #64");                                     // discard all borrowed traversal cursors
    emitter.instruction("b __rt_decref_any");                                   // consume the old owner under the caller's exception boundary
}

/// Consumes the System V old exception in rdi, appending it to the borrowed exception in rax.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.label_global("__rt_exception_chain");
    // -- preserve chain cursors and the incoming owner across metadata lookups --
    emitter.instruction("push rbp");                                            // preserve the caller and align outgoing helper calls
    emitter.instruction("mov rbp, rsp");                                        // establish stable traversal slots
    emitter.instruction("sub rsp, 48");                                         // reserve the current node, old owner, and replacement state
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // begin at the new exception rooted by the caller
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // keep the old owner until adoption or release
    emitter.instruction("test rdi, rdi");                                       // inspect the incoming previous owner
    emitter.instruction("jz __rt_exception_chain_return");                      // skip traversal when no previous exception exists
    emitter.instruction("cmp QWORD PTR [rdi], -3");                             // internal forced-unwind markers cannot escape through getPrevious
    emitter.instruction("je __rt_exception_chain_release");                     // release the marker while preserving the real pending Throwable
    if emitter.phpext_output {
        emitter.instruction("test rax, rax");                                   // distinguish a native replacement from a host callback escape
        emitter.instruction("jnz __rt_exception_chain_scan");                   // native replacements use the ordinary in-runtime chain
        emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                   // expose the native owner before the host escape consumes it
        emitter.bl_c("elephc_php_previous_exception");                           // copy the previous Throwable into malloc-owned transport
        emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                   // begin walking every previous link owned by that copied root
        emitter.label("__rt_exception_chain_copy_previous");
        emitter.instruction("call __rt_throwable_previous");                    // borrow the next native previous link without retaining it
        emitter.instruction("test rax, rax");                                   // inspect whether another copied node remains
        emitter.instruction("jz __rt_exception_chain_release");                 // release the root after the whole native chain is copied
        emitter.instruction("mov QWORD PTR [rbp - 24], rax");                   // preserve the current node across the SysV callback
        emitter.instruction("mov rdi, rax");                                    // pass the borrowed current node to the adapter
        emitter.bl_c("elephc_php_exception_previous");                           // append it to the active malloc-owned copied chain
        emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                   // continue traversal from the node just copied
        emitter.instruction("jmp __rt_exception_chain_copy_previous");          // preserve every constructor and finally-chain link
    }
    emitter.label("__rt_exception_chain_scan");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // restart the old chain at its root
    emitter.label("__rt_exception_chain_ancestors");
    emitter.instruction("cmp rax, QWORD PTR [rbp - 8]");                        // would this link repeat a new-chain object identity?
    emitter.instruction("je __rt_exception_chain_release");                     // consume a redundant owner without introducing a cycle
    emitter.instruction("call __rt_throwable_previous");                        // borrow the next old-chain ancestor
    emitter.instruction("test rax, rax");                                       // has the old chain ended?
    emitter.instruction("jnz __rt_exception_chain_ancestors");                  // compare every old-chain object with the current new-chain node
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // recover the current new-chain node
    emitter.instruction("call __rt_throwable_previous");                        // preserve links supplied before this collector exception
    emitter.instruction("test rax, rax");                                       // is this the tail of the new chain?
    emitter.instruction("jz __rt_exception_chain_append");                      // append only after the existing links
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // advance the borrowed new-chain cursor
    emitter.instruction("jmp __rt_exception_chain_scan");                       // validate the next node against the old chain

    // -- adopt the old owner using the actual previous-slot representation --
    emitter.label("__rt_exception_chain_append");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // recover the new-chain tail
    emitter.instruction("call __rt_throwable_previous_slot");                   // resolve its effective previous-property slot
    emitter.instruction("test rax, rax");                                       // check whether metadata supplied a writable slot
    emitter.instruction("jz __rt_exception_chain_release");                     // absent storage cannot adopt the previous owner
    emitter.instruction("test edx, edx");                                       // inspect the effective slot's boxed flag
    emitter.instruction("jz __rt_exception_chain_raw");                         // raw slots consume the old object owner directly
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // preserve the effective slot across allocation
    emitter.instruction("mov r10, QWORD PTR [rax]");                            // inspect the old null box being displaced
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // save that box for post-publication release
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // pass the old exception as the boxed object payload
    emitter.instruction("mov eax, 6");                                          // runtime tag six denotes an object
    emitter.instruction("xor esi, esi");                                        // object payloads have no second word
    emitter.instruction("call __rt_mixed_from_value");                          // retain the object under a fresh Mixed box owner
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // recover the effective previous slot
    emitter.instruction("mov QWORD PTR [r10], rax");                            // publish the new owner before releasing old storage
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // recover the displaced null box
    emitter.instruction("call __rt_decref_any");                                // release the box no longer owned by the property
    emitter.instruction("jmp __rt_exception_chain_release");                    // balance the old raw owner retained by the new box
    emitter.label("__rt_exception_chain_raw");
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                       // recover the old raw owner
    emitter.instruction("mov QWORD PTR [rax], r10");                            // transfer that owner directly into the raw previous link
    emitter.label("__rt_exception_chain_return");
    emitter.instruction("leave");                                               // restore the caller after ownership transfer
    emitter.instruction("ret");                                                 // leave no owner or cursor in this helper
    emitter.label("__rt_exception_chain_release");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // recover the redundant or reboxed owner
    emitter.instruction("leave");                                               // restore the caller before possible destructor execution
    emitter.instruction("jmp __rt_decref_any");                                 // consume the raw owner under the caller's exception boundary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::Target;

    /// Chaining uses layout-aware reads for both chains and publishes boxed links before releasing.
    #[test]
    fn exception_chain_uses_storage_metadata_on_every_target() {
        for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
            let mut emitter = Emitter::new(Target::parse(name).unwrap());
            emit_exception_chain(&mut emitter);
            let asm = emitter.output();
            assert_eq!(asm.matches("__rt_throwable_previous\n").count(), 2, "{name}");
            assert!(asm.contains("__rt_throwable_previous_slot"), "{name}");
            assert!(asm.find("__rt_mixed_from_value").unwrap() < asm.find("__rt_decref_any").unwrap(), "{name}");
            assert!(asm.contains("__rt_exception_chain_ancestors:") && asm.contains("__rt_exception_chain_raw:"), "{name}");
            assert!(!asm.contains("elephc_php_previous_exception"), "{name}");
        }
    }

    /// Hosted chains export an older native Throwable before a null host escape releases it.
    #[test]
    fn hosted_exception_chain_exports_native_previous_on_every_host_target() {
        for name in ["macos-aarch64", "linux-aarch64", "linux-x86_64"] {
            let mut emitter = Emitter::new(Target::parse(name).unwrap());
            emitter.phpext_output = true;
            emit_exception_chain(&mut emitter);
            let callback = emitter.target.extern_symbol("elephc_php_previous_exception");
            let continuation = emitter.target.extern_symbol("elephc_php_exception_previous");
            let asm = emitter.output();
            assert!(asm.contains(&callback), "{name}: {asm}");
            assert!(asm.contains(&continuation), "{name}: {asm}");
            assert_eq!(asm.matches("__rt_throwable_previous\n").count(), 3, "{name}: {asm}");
            assert!(asm.find(&callback).unwrap() < asm.find("__rt_exception_chain_release:").unwrap(),
                "{name}: {asm}");
        }
    }
}
