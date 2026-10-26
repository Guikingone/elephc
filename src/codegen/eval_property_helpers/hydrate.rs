//! Purpose:
//! Emits the native serialized-property writer using shared typed slot replacement.
//!
//! Called from:
//! - `super::emit_unserialize_property_helper()` during user assembly generation.
//!
//! Key details:
//! - Mangled PHP keys distinguish inherited private physical slots.
//! - Inputs remain borrowed; ordinary typed writers retain before replacing storage.
//! - Reserved native slots cannot be restored through PHP property data.
//! - Strict decoded tags and numeric widening share serialization metadata.

use super::*;
use crate::ir::{Immediate, RuntimeCallTarget, RuntimeFnId};

/// Returns whether the parser needs its concrete user-class property writer.
pub(super) fn is_used(module: &Module) -> bool {
    module.required_runtime_features.eval_bridge || module.functions.iter()
        .chain(module.class_methods.iter()).chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter()).chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter())
        .chain(module.runtime_callable_invokers.iter())
        .any(|function| function.instructions.iter().any(|inst| matches!(inst.immediate,
            Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(RuntimeFnId::Unserialize))))))
}

/// Defines the cache-facing entry in every program, specializing slots when parsing is available.
pub(super) fn emit(module: &Module, emitter: &mut Emitter, data: &mut DataSection) {
    if !is_used(module) {
        emitter.label_global("__rt_obj_store_prop");
        abi::emit_load_int_immediate(emitter, abi::int_result_reg(emitter), 0);
        abi::emit_return(emitter);
        return;
    }
    let slots = collect_eval_property_slots(module).into_iter()
        .filter(|slot| !slot.property.starts_with('\0'))
        .map(|mut slot| {
            // Untyped fields without defaults carry logical Void but use a boxed Mixed slot.
            if slot.ty == PhpType::Void { slot.ty = PhpType::Mixed; }
            slot.property = match slot.visibility {
                Visibility::Private => format!("\0{}\0{}", slot.declaring_class, slot.property),
                Visibility::Protected => format!("\0*\0{}", slot.property),
                _ => slot.property,
            };
            // Mangling already identifies the exact slot; there is no user scope lookup.
            slot.visibility = Visibility::Public;
            slot
        }).collect::<Vec<_>>();
    if !module.required_runtime_features.eval_bridge {
        emit_property_hash_slot_helper(module, emitter);
    }
    emitter.label_global("__rt_obj_store_prop");
    let fail = "__rt_obj_store_prop_fail";
    let done = "__rt_obj_store_prop_done";
    match module.target.arch {
        Arch::AArch64 => {
            emitter.instruction("sub sp, sp, #96");                             // reserve the shared typed-writer frame layout
            emitter.instruction("stp x29, x30, [sp, #80]");                     // preserve the caller frame and link register
            emitter.instruction("add x29, sp, #80");                            // establish a stable frame pointer
            emitter.instruction("str x1, [sp]");                                // preserve the borrowed mangled key pointer
            emitter.instruction("str x2, [sp, #8]");                            // preserve the key byte length
            emitter.instruction("str x0, [sp, #16]");                           // preserve the raw receiver for slot replacement
            emitter.instruction("str x3, [sp, #24]");                           // preserve the borrowed boxed replacement
            emitter.instruction(&format!("cbz x0, {fail}"));                    // reject a missing receiver
            emitter.instruction("cmp x2, #0");                                  // inspect the serialized name length
            emitter.instruction(&format!("b.lt {fail}"));                       // refuse an unsupported negative name encoding
            for (class_id, class_slots) in grouped_slots(&slots) {
                let next = format!("__rt_hydrate_property_next_{class_id}");
                emitter.instruction("ldr x9, [sp, #16]");                       // reload after string comparison scratch use
                emitter.instruction("ldr x9, [x9]");                            // recover the runtime receiver class id
                abi::emit_load_int_immediate(emitter, "x10", class_id as i64);
                emitter.instruction("cmp x9, x10");                             // select one receiver class layout
                emitter.instruction(&format!("b.ne {next}"));                   // try the next class if ids differ
                for slot in class_slots {
                    emit_aarch64_property_name_compare(module, emitter, data, slot, "hydrate", fail);
                }
                emitter.label(&next);
            }
            emitter.instruction("ldr x9, [sp, #8]");                            // inspect the key length before reading a byte
            emitter.instruction("cbz x9, __rt_hydrate_property_dynamic");       // empty names are valid dynamic properties
            emitter.instruction("ldr x9, [sp]");                                // recover the unmatched key pointer
            emitter.instruction("ldrb w9, [x9]");                               // a leading NUL means a missing declared slot
            emitter.instruction(&format!("cbz x9, {fail}"));                    // never publish reserved or unknown mangled keys dynamically
            emitter.label("__rt_hydrate_property_dynamic");
            emit_dynamic_property_set_fallback(emitter, fail, done);
            for slot in &slots {
                emitter.label(&slot_body_label(module, slot, "hydrate"));
                let local_fail = format!("{}_reject", slot_body_label(module, slot, "hydrate"));
                emit_restore_slot(module, emitter, data, slot, &local_fail);
                emitter.instruction("mov x0, #1");                              // report a successful declared-property replacement
                emitter.instruction(&format!("b {done}"));                      // join the common writer epilogue
                emitter.label(&local_fail);
                abi::emit_jump(emitter, fail);
            }
            emitter.label(fail);
            emitter.instruction("mov x0, #0");                                  // report an unsupported property restoration
            emitter.label(done);
            emitter.instruction("ldp x29, x30, [sp, #80]");                     // restore the caller frame and link register
            emitter.instruction("add sp, sp, #96");                             // discard borrowed inputs and replacement temporaries
            emitter.instruction("ret");                                         // return the boolean restoration status
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");                                    // preserve the caller frame pointer
            emitter.instruction("mov rbp, rsp");                                // establish the shared typed-writer frame base
            emitter.instruction("sub rsp, 64");                                 // reserve aligned input and replacement temporaries
            emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                // preserve the borrowed mangled key pointer
            emitter.instruction("mov QWORD PTR [rbp - 16], rdx");               // preserve the key byte length
            emitter.instruction("mov QWORD PTR [rbp - 24], rdi");               // preserve the raw receiver
            emitter.instruction("mov QWORD PTR [rbp - 32], rcx");               // preserve the borrowed boxed replacement
            emitter.instruction("test rdi, rdi");                               // inspect the receiver before reading its header
            emitter.instruction(&format!("jz {fail}"));                         // reject a missing receiver
            emitter.instruction("test rdx, rdx");                               // inspect the serialized name length
            emitter.instruction(&format!("js {fail}"));                         // refuse an unsupported negative name encoding
            for (class_id, class_slots) in grouped_slots(&slots) {
                let next = format!("__rt_hydrate_property_next_{class_id}");
                emitter.instruction("mov r11, QWORD PTR [rbp - 24]");           // reload after string comparison scratch use
                emitter.instruction("mov r11, QWORD PTR [r11]");                // recover the runtime receiver class id
                abi::emit_load_int_immediate(emitter, "r10", class_id as i64);
                emitter.instruction("cmp r11, r10");                            // select one receiver class layout
                emitter.instruction(&format!("jne {next}"));                    // try the next class if ids differ
                for slot in class_slots {
                    emit_x86_64_property_name_compare(module, emitter, data, slot, "hydrate", fail);
                }
                emitter.label(&next);
            }
            emitter.instruction("cmp QWORD PTR [rbp - 16], 0");                 // check the unmatched property-name length
            emitter.instruction("je __rt_hydrate_property_dynamic");            // empty names are valid dynamic properties
            emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                // recover the unmatched key pointer
            emitter.instruction("cmp BYTE PTR [r11], 0");                       // a leading NUL means a missing declared slot
            emitter.instruction(&format!("je {fail}"));                         // never publish unknown mangled keys dynamically
            emitter.label("__rt_hydrate_property_dynamic");
            emit_dynamic_property_set_fallback(emitter, fail, done);
            for slot in &slots {
                emitter.label(&slot_body_label(module, slot, "hydrate"));
                let local_fail = format!("{}_reject", slot_body_label(module, slot, "hydrate"));
                emit_restore_slot(module, emitter, data, slot, &local_fail);
                emitter.instruction("mov rax, 1");                              // report a successful declared-property replacement
                emitter.instruction(&format!("jmp {done}"));                    // join the common writer epilogue
                emitter.label(&local_fail);
                abi::emit_jump(emitter, fail);
            }
            emitter.label(fail);
            emitter.instruction("xor eax, eax");                                // report an unsupported property restoration
            emitter.label(done);
            emitter.instruction("mov rsp, rbp");                                // discard borrowed inputs and replacement temporaries
            emitter.instruction("pop rbp");                                     // restore the caller frame pointer
            emitter.instruction("ret");                                         // return the boolean restoration status
        }
    }
}

/// Checks object type hints through native class/interface tables without linking eval.
fn emit_restore_slot(
    module: &Module, emitter: &mut Emitter, data: &mut DataSection,
    slot: &EvalPropertySlot, fail: &str,
) {
    emit_hydration_type_validation(module, emitter, data, slot);
    emit_hydration_int_widening(module, emitter, slot);
    let declared = slot.declared_type.as_ref().unwrap_or(&slot.ty);
    if let PhpType::Object(class_name) = declared.codegen_repr() {
        if !class_name.is_empty() {
            let target = module.class_infos.get(&class_name).map(|info| (info.class_id, 0))
                .or_else(|| module.interface_infos.get(&class_name).map(|info| (info.interface_id, 1)));
            let Some((id, kind)) = target else {
                abi::emit_jump(emitter, fail);
                return;
            };
            match module.target.arch {
                Arch::AArch64 => {
                    emitter.instruction("ldr x0, [sp, #24]");                   // borrow the assigned cell for native object type validation
                }
                Arch::X86_64 => {
                    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");       // borrow the assigned cell for native object type validation
                }
            }
            abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(module.target, 1), id as i64);
            abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(module.target, 2), kind);
            abi::emit_call_label(emitter, "__rt_mixed_instanceof");
            abi::emit_branch_if_int_result_zero(emitter, fail);
        }
        emit_mixed_overwrite_in_place(module, emitter, slot);
        match module.target.arch {
            Arch::AArch64 if slot.ty.codegen_repr() == PhpType::Mixed => {
                emit_aarch64_store_property_slot(module, emitter, data, slot, fail);
            }
            Arch::X86_64 if slot.ty.codegen_repr() == PhpType::Mixed => {
                emit_x86_64_store_property_slot(module, emitter, data, slot, fail);
            }
            Arch::AArch64 => emit_aarch64_store_heap_property_slot(emitter, slot, 6, fail),
            Arch::X86_64 => emit_x86_64_store_heap_property_slot(emitter, slot, 6, fail),
        }
    } else {
        emit_mixed_overwrite_in_place(module, emitter, slot);
        match module.target.arch {
            Arch::AArch64 => emit_aarch64_store_property_slot(module, emitter, data, slot, fail),
            Arch::X86_64 => emit_x86_64_store_property_slot(module, emitter, data, slot, fail),
        }
    }
}

/// Overwrites an initialized Mixed property's box in place, as php overwrites the zval.
///
/// The hydrated property shares its box with the parser's value slot table, so a later `R:`
/// back-reference to the first value must observe the replacement. An uninitialized, empty or
/// null-sentinel slot has no box to reuse and takes the ordinary retaining store, as does a
/// resource replacement the runtime helper declines.
fn emit_mixed_overwrite_in_place(module: &Module, emitter: &mut Emitter, slot: &EvalPropertySlot) {
    if slot.ty.codegen_repr() != PhpType::Mixed {
        return;
    }
    let skip = format!("{}_fresh_box", slot_body_label(module, slot, "hydrate"));
    let uninitialized = crate::codegen_support::sentinels::UNINITIALIZED_TYPED_PROPERTY_SENTINEL as i64;
    let null = crate::codegen_support::NULL_SENTINEL as i64;
    match module.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x9, [sp, #16]");                           // reload the receiver whose slot may already hold a box
            emitter.instruction(&format!("ldr x10, [x9, #{}]", slot.offset + 8)); // read the slot's initialization word
            abi::emit_load_int_immediate(emitter, "x11", uninitialized);
            emitter.instruction("cmp x10, x11");                                // is the property still uninitialized?
            emitter.instruction(&format!("b.eq {skip}"));                       // a fresh slot takes the ordinary store
            emitter.instruction(&format!("ldr x0, [x9, #{}]", slot.offset));    // the current Mixed box or null word
            emitter.instruction(&format!("cbz x0, {skip}"));                    // an empty slot has no box to reuse
            abi::emit_load_int_immediate(emitter, "x11", null);
            emitter.instruction("cmp x0, x11");                                 // is the slot holding the null sentinel?
            emitter.instruction(&format!("b.eq {skip}"));                       // sentinels have no box to reuse
            emitter.instruction("ldr w10, [x0, #-12]");                         // read the box refcount from the uniform heap header
            emitter.instruction("cmp w10, #1");                                 // is the box shared with the parser's value slots?
            emitter.instruction(&format!("b.ls {skip}"));                       // a private default box takes the ordinary store
            emitter.instruction("ldr x1, [sp, #24]");                           // borrow the parsed replacement box
            abi::emit_call_label(emitter, "__rt_unser_overwrite_box");
            emitter.instruction(&format!("cbz x0, {skip}"));                    // a shared resource cell takes the ordinary store
            emitter.instruction("mov x0, #1");                                  // report a successful declared-property replacement
            emitter.instruction("b __rt_obj_store_prop_done");                  // join the common writer epilogue
        }
        Arch::X86_64 => {
            emitter.instruction("mov r11, QWORD PTR [rbp - 24]");               // reload the receiver whose slot may already hold a box
            emitter.instruction(&format!("mov r10, QWORD PTR [r11 + {}]", slot.offset + 8)); // read the slot's initialization word
            abi::emit_load_int_immediate(emitter, "rax", uninitialized);
            emitter.instruction("cmp r10, rax");                                // is the property still uninitialized?
            emitter.instruction(&format!("je {skip}"));                         // a fresh slot takes the ordinary store
            emitter.instruction(&format!("mov rdi, QWORD PTR [r11 + {}]", slot.offset)); // the current Mixed box or null word
            emitter.instruction("test rdi, rdi");                               // is the slot empty?
            emitter.instruction(&format!("jz {skip}"));                         // an empty slot has no box to reuse
            abi::emit_load_int_immediate(emitter, "rax", null);
            emitter.instruction("cmp rdi, rax");                                // is the slot holding the null sentinel?
            emitter.instruction(&format!("je {skip}"));                         // sentinels have no box to reuse
            emitter.instruction("mov r10d, DWORD PTR [rdi - 12]");              // read the box refcount from the uniform heap header
            emitter.instruction("cmp r10d, 1");                                 // is the box shared with the parser's value slots?
            emitter.instruction(&format!("jbe {skip}"));                        // a private default box takes the ordinary store
            emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");               // borrow the parsed replacement box
            abi::emit_call_label(emitter, "__rt_unser_overwrite_box");
            emitter.instruction("test rax, rax");                               // did the helper overwrite the box?
            emitter.instruction(&format!("jz {skip}"));                         // a shared resource cell takes the ordinary store
            emitter.instruction("mov eax, 1");                                  // report a successful declared-property replacement
            emitter.instruction("jmp __rt_obj_store_prop_done");                // join the common writer epilogue
        }
    }
    emitter.label(&skip);
}

/// Preserves PHP's decoder-owned int-to-float widening before the retaining slot writer.
fn emit_hydration_int_widening(module: &Module, emitter: &mut Emitter, slot: &EvalPropertySlot) {
    if !slot.declared_type.as_ref().is_some_and(
        crate::codegen_support::runtime::data::hydration_widens_int_to_float,
    ) { return; }
    let done = slot_body_label(module, slot, "hydrate_widen_done");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x9, [sp, #24]");                           // recover the decoder-owned parsed value cell
            emitter.instruction("ldr x10, [x9]");                               // inspect the original parsed tag
            emitter.instruction(&format!("cbnz x10, {done}"));                  // only an integer requires widening
            emitter.instruction("ldr x10, [x9, #8]");                           // load the signed integer payload
            emitter.instruction("scvtf d0, x10");                               // convert the accepted integer to a PHP float
            emitter.instruction("fmov x10, d0");                                // recover the floating-point payload bits
            emitter.instruction("str x10, [x9, #8]");                           // publish the widened payload in the parsed cell
            emitter.instruction("mov x10, #2");                                 // select the PHP float runtime tag
            emitter.instruction("str x10, [x9]");                               // retag the decoder-owned cell before slot adaptation
        }
        Arch::X86_64 => {
            emitter.instruction("mov r11, QWORD PTR [rbp - 32]");               // recover the decoder-owned parsed value cell
            emitter.instruction("cmp QWORD PTR [r11], 0");                      // recognize an integer before changing its representation
            emitter.instruction(&format!("jne {done}"));                        // already floating-point and other accepted tags stay unchanged
            emitter.instruction("cvtsi2sd xmm0, QWORD PTR [r11 + 8]");          // convert the signed integer payload to a PHP float
            emitter.instruction("movq r10, xmm0");                              // recover the floating-point payload bits
            emitter.instruction("mov QWORD PTR [r11 + 8], r10");                // publish the widened payload in the parsed cell
            emitter.instruction("mov QWORD PTR [r11], 2");                      // retag the decoder-owned cell before slot adaptation
        }
    }
    emitter.label(&done);
}

/// Validates decoded tags against the same strict declared-type mask as serialization metadata.
///
/// Typed value writers retain borrowed inputs, but hydration must reject weak coercions first.
fn emit_hydration_type_validation(
    module: &Module, emitter: &mut Emitter, data: &mut DataSection, slot: &EvalPropertySlot,
) {
    let Some(declared) = slot.declared_type.as_ref() else { return; };
    let accepted = crate::codegen_support::runtime::data::hydration_accepted_mask(declared);
    if accepted == u64::MAX { return; }
    let stem = slot_body_label(module, slot, "hydrate_type");
    let good = format!("{stem}_valid");
    let rejected = format!("{stem}_rejected");
    match module.target.arch {
        Arch::AArch64 => emitter.instruction("ldr x0, [sp, #24]"),              // borrow the parsed property value
        Arch::X86_64 => emitter.instruction("mov rax, QWORD PTR [rbp - 32]"),   // borrow the parsed property value
    }
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    for tag in 0..=10 {
        if accepted & (1u64 << tag) == 0 { continue; }
        let next = format!("{stem}_after_valid_{tag}");
        emit_tag_branch(emitter, tag, &good, &next);
        emitter.label(&next);
    }
    if accepted & crate::codegen_support::sentinels::SERPROP_FALSE_ONLY_BIT != 0 {
        let boolean = format!("{stem}_false_member");
        emit_tag_branch(emitter, 3, &boolean, &rejected);
        emitter.label(&boolean);
        let result = abi::int_result_reg(emitter);
        let payload = crate::codegen_support::mixed_unbox_payload_reg(emitter.target);
        abi::emit_reg_move(emitter, result, payload);
        abi::emit_branch_if_int_result_zero(emitter, &good);
    }
    emitter.label(&rejected);
    emit_hydration_type_error(emitter, data, slot);
    emitter.label(&good);
}

/// Compares the canonical integer result without changing its payload registers.
fn emit_tag_branch(emitter:&mut Emitter, tag:i64, matched:&str, otherwise:&str) {
    let result=abi::int_result_reg(emitter);let scratch=abi::secondary_scratch_reg(emitter);
    abi::emit_load_int_immediate(emitter,scratch,tag);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cmp {result}, {scratch}"));           // compare the boxed runtime type or class id
            emitter.instruction(&format!("b.eq {matched}"));                    // select the matching property type path
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("cmp {result}, {scratch}"));           // compare the boxed runtime type or class id
            emitter.instruction(&format!("je {matched}"));                      // select the matching property type path
        }
    }
    abi::emit_jump(emitter,otherwise);
}

/// Throws through the parser boundary, which owns exception cleanup.
fn emit_hydration_type_error(emitter: &mut Emitter, data: &mut DataSection, slot: &EvalPropertySlot) {
    let property = slot.property.rsplit('\0').next().unwrap_or(&slot.property);
    let declared = slot.declared_type.as_ref()
        .map(crate::codegen_support::runtime::data::hydration_type_name)
        .unwrap_or_else(|| "mixed".to_string());
    let suffix = format!(" to property {}::${property} of type {declared}", slot.declaring_class);
    let (label, len) = data.add_string(suffix.as_bytes());
    let prefix_len = crate::codegen_support::runtime::data::UNSER_PROPERTY_ASSIGN_PREFIX.len();
    // The active parser owns the input, so the shared property entry must release no box
    // and leave context closure to the decoder's existing protected boundary.
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x0, [sp, #24]");                           // borrow the rejected parsed value
            abi::emit_call_label(emitter, "__rt_mixed_unbox");
            abi::emit_symbol_address(emitter, "x2", "_unser_property_assign_prefix");
            abi::emit_load_int_immediate(emitter, "x3", prefix_len as i64);
            abi::emit_symbol_address(emitter, "x4", &label);
            abi::emit_load_int_immediate(emitter, "x5", len as i64);
            abi::emit_load_int_immediate(emitter, "x6", 1);
            abi::emit_load_int_immediate(emitter, "x7", 0);
            emitter.instruction("ldp x29, x30, [sp, #80]");                     // restore the hydration caller frame
            emitter.instruction("add sp, sp, #96");                             // remove borrowed inputs before the nonreturning helper
        }
        Arch::X86_64 => {
            emitter.instruction("mov rax, QWORD PTR [rbp - 32]");               // borrow the rejected parsed value
            abi::emit_call_label(emitter, "__rt_mixed_unbox");
            abi::emit_symbol_address(emitter, "rsi", "_unser_property_assign_prefix");
            abi::emit_load_int_immediate(emitter, "rdx", prefix_len as i64);
            abi::emit_symbol_address(emitter, "r8", &label);
            abi::emit_load_int_immediate(emitter, "r9", len as i64);
            abi::emit_load_int_immediate(emitter, "r10", 1);
            abi::emit_load_int_immediate(emitter, "r11", 0);
            emitter.instruction("leave");                                       // remove the hydration frame before the shared helper
        }
    }
    abi::emit_jump(emitter, "__rt_unser_throw_property_type_error");
}
