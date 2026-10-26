//! Purpose:
//! Decodes public and mangled PHP debug keys into borrowed name and scope slices.
//! Shares the canonical Zend parsing rules between print_r and var_dump.
//!
//! Called from:
//! - Both native debug-key formatters.
//!
//! Key details:
//! - C arguments are key pointer, byte length and a four-word output record.
//! - Status is public0, protected1, private2, illegal3 or corrupt4; notices belong to callers.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
const FRAME: usize = 80;
const KEY: usize = 8;
const LENGTH: usize = 16;
const OUTPUT: usize = 24;
const CLASS: usize = 32;
const CLASS_LENGTH: usize = 40;
const PROPERTY: usize = 48;
const PROPERTY_LENGTH: usize = 56;

/// Selects a target instruction while retaining a single parser control flow.
fn ins(e: &mut Emitter, arm: &str, x86: &str) {
    e.instruction(if e.target.arch == Arch::AArch64 { arm } else { x86 }); // preserve the same borrowed-key parsing step on both architectures
}

/// Emits a nonallocating property-name decoder with no PHP callback side effects.
pub(super) fn emit_debug_key_parts(e: &mut Emitter) {
    let result = abi::int_result_reg(e);
    let base = abi::symbol_scratch_reg(e);
    let word = abi::secondary_scratch_reg(e);
    e.label_global("__rt_debug_key_parts");
    abi::emit_frame_prologue(e, FRAME);
    for (i, slot) in [(0,KEY),(1,LENGTH),(2,OUTPUT)] {
        abi::store_at_offset(e, abi::int_arg_reg_name(e.target,i), slot);
    }
    for (field, slot) in [(0,KEY),(8,LENGTH)] {
        abi::load_at_offset(e,result,slot); output(e,field);
    }
    abi::emit_load_int_immediate(e,result,0); output(e,16); output(e,24);
    abi::load_at_offset(e,result,LENGTH);
    abi::emit_branch_if_int_result_zero(e,"__rt_debug_key_parts_public");
    abi::load_at_offset(e,base,KEY);
    ins(e,"ldrb w0, [x9]","movzx eax, BYTE PTR [r11]");
    abi::emit_branch_if_int_result_nonzero(e,"__rt_debug_key_parts_public");
    abi::load_at_offset(e,result,LENGTH);
    ins(e,"cmp x0, #3","cmp rax, 3");
    ins(e,"b.lo __rt_debug_key_parts_illegal","jb __rt_debug_key_parts_illegal");
    ins(e,"ldrb w0, [x9, #1]","movzx eax, BYTE PTR [r11 + 1]");
    abi::emit_branch_if_int_result_zero(e,"__rt_debug_key_parts_illegal");
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),KEY);
    ins(e,"add x0, x0, #1","add rdi, 1");
    abi::store_at_offset(e,abi::int_arg_reg_name(e.target,0),CLASS);
    abi::emit_load_int_immediate(e,abi::int_arg_reg_name(e.target,1),0);
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,2),LENGTH);
    ins(e,"sub x2, x2, #2","sub rdx, 2");
    abi::emit_call_label(e,&e.target.extern_symbol("memchr"));
    abi::emit_branch_if_int_result_zero(e,"__rt_debug_key_parts_corrupt");
    ins(e,"add x0, x0, #1","add rax, 1");
    abi::store_at_offset(e,result,PROPERTY);
    abi::load_at_offset(e,word,CLASS);
    ins(e,"sub x0, x0, x10","sub rax, r10");
    ins(e,"sub x0, x0, #1","sub rax, 1");
    abi::store_at_offset(e,result,CLASS_LENGTH);
    property_length(e);
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),PROPERTY);
    abi::emit_load_int_immediate(e,abi::int_arg_reg_name(e.target,1),0);
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,2),PROPERTY_LENGTH);
    abi::emit_call_label(e,&e.target.extern_symbol("memchr"));
    abi::emit_branch_if_int_result_zero(e,"__rt_debug_key_parts_formatted");
    ins(e,"add x0, x0, #1","add rax, 1");
    abi::store_at_offset(e,result,PROPERTY);
    property_length(e);
    e.label("__rt_debug_key_parts_formatted");
    for (field,slot) in [(0,PROPERTY),(8,PROPERTY_LENGTH),(16,CLASS),(24,CLASS_LENGTH)] {
        abi::load_at_offset(e,result,slot);output(e,field);
    }
    abi::load_at_offset(e,base,CLASS);
    ins(e,"ldrb w0, [x9]","movzx eax, BYTE PTR [r11]");
    ins(e,"cmp x0, #42","cmp rax, 42");
    ins(e,"b.eq __rt_debug_key_parts_protected","je __rt_debug_key_parts_protected");
    abi::emit_load_int_immediate(e,result,2);
    abi::emit_jump(e,"__rt_debug_key_parts_done");
    e.label("__rt_debug_key_parts_protected");
    abi::emit_load_int_immediate(e,result,1);
    abi::emit_jump(e,"__rt_debug_key_parts_done");
    for (label,status) in [("public",0),("illegal",3),("corrupt",4)] {
        e.label(&format!("__rt_debug_key_parts_{label}"));
        abi::emit_load_int_immediate(e,result,status);
        abi::emit_jump(e,"__rt_debug_key_parts_done");
    }
    e.label("__rt_debug_key_parts_done");
    abi::emit_frame_restore(e,FRAME);abi::emit_return(e);
}

/// Stores one decoded slice component through the caller-owned output record.
fn output(e: &mut Emitter, field: usize) {
    let base=abi::secondary_scratch_reg(e);
    abi::load_at_offset(e,base,OUTPUT);
    abi::emit_store_to_address(e,abi::int_result_reg(e),base,field);
}

/// Recomputes the remaining property bytes after either class-name delimiter.
fn property_length(e: &mut Emitter) {
    let result=abi::int_result_reg(e);let word=abi::secondary_scratch_reg(e);
    abi::load_at_offset(e,result,KEY);abi::load_at_offset(e,word,LENGTH);
    ins(e,"add x0, x0, x10","add rax, r10");
    abi::load_at_offset(e,word,PROPERTY);
    ins(e,"sub x0, x0, x10","sub rax, r10");
    abi::store_at_offset(e,result,PROPERTY_LENGTH);
}
