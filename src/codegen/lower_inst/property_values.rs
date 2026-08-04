//! Purpose:
//! Shares target-aware value conversions used by instance and static property stores.
//! Keeps storage compatibility and refcount behavior identical across both property paths.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects` for declared instance-property writes.
//! - `crate::codegen::lower_inst::static_properties` for static-property writes.
//!
//! Key details:
//! - Mixed object payloads are retained before entering a concrete object slot.
//! - Non-object Mixed payloads normalize to the null object sentinel.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::types::PhpType;

use super::super::context::FunctionContext;

/// Returns true when a boxed Mixed value can be unboxed into an object property slot.
pub(super) fn can_unbox_mixed_to_object_property(
    value_ty: &PhpType,
    slot_ty: &PhpType,
) -> bool {
    matches!(value_ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
        && matches!(slot_ty.codegen_repr(), PhpType::Object(_))
}

/// Returns true when a boxed value is stored into a `Callable`-typed property slot.
///
/// The `Object` sibling below cannot serve this: its emitter tests the runtime OBJECT tag and
/// normalizes everything else to the null sentinel, so a boxed closure would silently become null.
pub(super) fn can_unbox_mixed_to_callable_property(
    value_ty: &PhpType,
    slot_ty: &PhpType,
) -> bool {
    matches!(value_ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
        && matches!(slot_ty.codegen_repr(), PhpType::Callable)
}

/// Unboxes a Mixed property value into an independently retained closure descriptor.
///
/// The mirror of `emit_mixed_object_for_property_store` for the CALLABLE tag. `self::$fn ??=
/// <closure>` types its value as the union of the slot's null and the closure, so the store sees a
/// boxed Mixed against a `Callable` slot; without this it fell into the dispatch's catch-all arm
/// and the box POINTER would have been stored raw as if it were a descriptor.
///
/// A non-callable payload becomes the null descriptor, matching the object path's treatment of a
/// mismatched tag. The retain goes through the descriptor helper, not `__rt_incref`: a callable is
/// not a plain refcounted heap value.
pub(super) fn emit_mixed_callable_for_property_store(ctx: &mut FunctionContext<'_>) {
    let callable_label = ctx.next_label("prop_store_mixed_value_callable");
    let done = ctx.next_label("prop_store_mixed_callable_done");
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #10");                            // require the runtime callable payload tag
            ctx.emitter.instruction(&format!("b.eq {}", callable_label));      // store descriptors through the concrete slot path
            ctx.emitter.instruction("mov x0, #0");                             // normalize non-callable payloads to the null descriptor
            ctx.emitter.instruction(&format!("b {}", done));                   // skip descriptor promotion for non-callable payloads
            ctx.emitter.label(&callable_label);
            ctx.emitter.instruction("mov x0, x1");                             // promote the unboxed descriptor into the result register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 10");                            // require the runtime callable payload tag
            ctx.emitter.instruction(&format!("je {}", callable_label));        // store descriptors through the concrete slot path
            ctx.emitter.instruction("xor eax, eax");                           // normalize non-callable payloads to the null descriptor
            ctx.emitter.instruction(&format!("jmp {}", done));                 // skip descriptor promotion for non-callable payloads
            ctx.emitter.label(&callable_label);
            ctx.emitter.instruction("mov rax, rdi");                           // promote the unboxed descriptor into the result register
        }
    }
    ctx.emitter.label(&done);
    abi::emit_incref_if_refcounted(
        ctx.emitter,
        &PhpType::Callable, // retain the descriptor independently for property storage
    );
}

/// Unboxes a Mixed property value into an independently retained object pointer.
///
/// Non-object payloads become the null object sentinel, matching the established
/// instance-property coercion. The caller remains responsible for releasing an
/// owning Mixed source after the retaining store has completed.
pub(super) fn emit_mixed_object_for_property_store(ctx: &mut FunctionContext<'_>) {
    let object_label = ctx.next_label("prop_store_mixed_value_object");
    let done = ctx.next_label("prop_store_mixed_value_done");
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                              // require the runtime object payload tag
            ctx.emitter.instruction(&format!("b.eq {}", object_label));         // store object payloads through the concrete slot path
            ctx.emitter.instruction("mov x0, #0");                              // normalize non-object payloads to the null sentinel
            ctx.emitter.instruction(&format!("b {}", done));                    // skip object-pointer promotion for non-object payloads
            ctx.emitter.label(&object_label);
            ctx.emitter.instruction("mov x0, x1");                              // promote the unboxed object pointer into the result register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                              // require the runtime object payload tag
            ctx.emitter.instruction(&format!("je {}", object_label));           // store object payloads through the concrete slot path
            ctx.emitter.instruction("xor eax, eax");                            // normalize non-object payloads to the null sentinel
            ctx.emitter.instruction(&format!("jmp {}", done));                  // skip object-pointer promotion for non-object payloads
            ctx.emitter.label(&object_label);
            ctx.emitter.instruction("mov rax, rdi");                            // promote the unboxed object pointer into the result register
        }
    }
    ctx.emitter.label(&done);
    abi::emit_incref_if_refcounted(
        ctx.emitter,
        &PhpType::Object(String::new()), // retain the object independently for property storage
    );
}
