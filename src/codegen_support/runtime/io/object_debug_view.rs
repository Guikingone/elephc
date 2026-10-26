//! Purpose:
//! Invokes a retained object's native debug hook through an ordinary boxed callable invoker.
//!
//! Called from:
//! - The pending runtime object-view owner boundary before rendering a debug snapshot.
//!
//! Key details:
//! - This helper borrows its receiver; its caller must keep the receiver alive across invocation.
//! - A per-call stack descriptor binds the one receiver capture without creating a PHP Closure.
//! - Zero means no native hook; a hook's owned Mixed result and native throws are transferred.
//! - The ABI's nested-call register is callee-saved and must survive this helper too.

use crate::codegen_support::{abi, callable_descriptor, emit::Emitter, platform::Arch};

/// Emits a borrowed-receiver hook invocation with a uniform owned Mixed result.
pub fn emit_object_debug_info(emitter: &mut Emitter) {
    const FRAME: usize = 128;
    const RECEIVER: usize = 8;
    const SAVED_CALL: usize = 16;
    const DESCRIPTOR: usize = FRAME - 16;
    let result = abi::int_result_reg(emitter);
    let argument = abi::int_arg_reg_name(emitter.target, 0);
    let descriptor = abi::symbol_scratch_reg(emitter);
    let word = abi::secondary_scratch_reg(emitter);
    let call = abi::nested_call_reg(emitter);
    emitter.blank();
    emitter.comment("--- runtime: object_debug_info ---");
    emitter.label_global("__rt_object_debug_info");
    abi::emit_frame_prologue(emitter, FRAME);
    abi::store_at_offset(emitter, call, SAVED_CALL);
    abi::store_at_offset(emitter, argument, RECEIVER);
    abi::emit_call_label(emitter, "__rt_object_debug_info_template");
    abi::emit_branch_if_int_result_zero(emitter, "__rt_object_debug_info_absent");
    abi::emit_frame_slot_address(emitter, descriptor, DESCRIPTOR);
    for offset in (0..callable_descriptor::CALLABLE_DESC_STATIC_SIZE).step_by(8) {
        abi::emit_load_from_address(emitter, word, result, offset);
        abi::emit_store_to_address(emitter, word, descriptor, offset);
    }
    abi::load_at_offset(emitter, result, RECEIVER);
    abi::emit_store_to_address(
        emitter, result, descriptor, callable_descriptor::CALLABLE_DESC_RUNTIME_CAPTURE_OFFSET,
    );
    abi::emit_load_int_immediate(emitter, word, 0);
    abi::emit_store_to_address(
        emitter, word, descriptor, callable_descriptor::CALLABLE_DESC_RUNTIME_CAPTURE_OFFSET + 8,
    );
    callable_descriptor::emit_load_invoker_from_descriptor(emitter, call, descriptor);
    abi::emit_frame_slot_address(emitter, argument, DESCRIPTOR);
    abi::emit_symbol_address(
        emitter, abi::int_arg_reg_name(emitter.target, 1), "_object_debug_info_empty_args",
    );
    abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(emitter.target, 2), -1);
    abi::emit_call_reg(emitter, call);
    abi::load_at_offset(emitter, call, SAVED_CALL);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
    emitter.label("__rt_object_debug_info_absent");
    abi::load_at_offset(emitter, call, SAVED_CALL);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_load_int_immediate(emitter, result, 0);
    abi::emit_return(emitter);
    emit_template_lookup(emitter);
}

/// Resolves a borrowed native debug template without running PHP or binding a receiver.
fn emit_template_lookup(e:&mut Emitter) {
    let result=abi::int_result_reg(e);let first=abi::int_arg_reg_name(e.target,0);
    let table=abi::secondary_scratch_reg(e);
    e.label_global("__rt_object_debug_info_template");
    abi::emit_reg_move(e,result,first);
    abi::emit_branch_if_int_result_zero(e,"__rt_object_debug_info_template_absent");
    abi::emit_load_from_address(e,result,result,0);
    abi::emit_cmp_reg_to_symbol(e,result,"_object_debug_info_template_count");
    match e.target.arch {
        Arch::AArch64=>{
                        e.instruction("b.hs __rt_object_debug_info_template_absent");       // resolve only an in-range immutable native debug template
        }
        Arch::X86_64=>{
                        e.instruction("jae __rt_object_debug_info_template_absent");        // resolve only an in-range immutable native debug template
        }
    }
    abi::emit_symbol_address(e,table,"_object_debug_info_templates");
    match e.target.arch {
        Arch::AArch64=>{
                        e.instruction("ldr x0, [x10, x0, lsl #3]");                         // resolve only an in-range immutable native debug template
        }
        Arch::X86_64=>{
                        e.instruction("mov rax, QWORD PTR [r10 + rax*8]");                  // resolve only an in-range immutable native debug template
        }
    }
    abi::emit_return(e);
    e.label("__rt_object_debug_info_template_absent");
    abi::emit_load_int_immediate(e,result,0);abi::emit_return(e);
}
