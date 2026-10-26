//! Purpose:
//! Leases recursive print_r values across arbitrary debug callbacks and child cleanup.
//! Keeps source payloads stable while the enclosing call owns its final buffered result.
//!
//! Called from:
//! - The common print_r capture body and recursive indexed/hash/object entry walkers.
//!
//! Key details:
//! - The borrowed tag/payload/indent tuple is preserved independently from mutable PHP cells.
//! - Child destructor errors are deferred to the enclosing print_r call after formatting.

use crate::codegen_support::{abi,callable_descriptor,emit::Emitter};
use crate::codegen_support::runtime::exceptions;
use super::print_r_php_call::emit_guarded_php_call;
use super::var_dump_value_scope::{read_context,context_literal,branch_tag};
const CONTEXT:usize=80;
const FRAME:usize=96;
const TAG:usize=0;
const LOW:usize=8;
const HIGH:usize=16;
const INDENT:usize=24;
const OWNED:usize=32;
const PENDING:usize=40;
const SEEN:usize=48;

/// Emits the shared borrowed-value lease boundary.
pub fn emit_print_r_value_scope(e:&mut Emitter) {
    let r=abi::int_result_reg(e);
    e.label_global("__rt_print_r_value");abi::emit_frame_prologue(e,FRAME);
    for (i,field) in [(0,TAG),(1,LOW),(2,HIGH),(3,INDENT)] {
        abi::store_at_offset(e,abi::int_arg_reg_name(e.target,i),CONTEXT-field);
    }
    for field in [OWNED,PENDING] {abi::emit_store_zero_to_local_slot(e,CONTEXT-field);}
    abi::emit_load_symbol_to_reg(e,r,"_vd_seen_n",0);abi::store_at_offset(e,r,CONTEXT-SEEN);
    abi::emit_frame_slot_address(e,r,CONTEXT);
    exceptions::emit_guarded_cleanup_call(e,"__rt_print_r_value_scope_body",r,CONTEXT-PENDING);
    restore_seen(e);
    abi::load_at_offset(e,r,CONTEXT-OWNED);
    abi::emit_branch_if_int_result_zero(e,"__rt_print_r_value_scope_cleaned");
    abi::emit_store_zero_to_local_slot(e,CONTEXT-OWNED);
    abi::load_at_offset(e,r,CONTEXT-TAG);branch_tag(e,10,"__rt_print_r_value_scope_callable");
    abi::load_at_offset(e,r,CONTEXT-LOW);
    emit_guarded_php_call(e,"__rt_decref_any",r,CONTEXT-PENDING);
    abi::emit_jump(e,"__rt_print_r_value_scope_cleaned");
    e.label("__rt_print_r_value_scope_callable");
    abi::load_at_offset(e,r,CONTEXT-LOW);
    emit_guarded_php_call(e,"__rt_callable_descriptor_release",r,CONTEXT-PENDING);
    e.label("__rt_print_r_value_scope_cleaned");restore_seen(e);
    abi::load_at_offset(e,r,CONTEXT-PENDING);
    abi::emit_store_zero_to_local_slot(e,CONTEXT-PENDING);
    abi::emit_call_label(e,"__rt_print_r_record_exception");
    abi::emit_frame_restore(e,FRAME);abi::emit_return(e);
    emit_body(e);
}

/// Restores the enclosing recursion membership after normal or exceptional traversal.
fn restore_seen(e:&mut Emitter) {
    let r=abi::int_result_reg(e);abi::load_at_offset(e,r,CONTEXT-SEEN);
    abi::emit_store_reg_to_symbol(e,r,"_vd_seen_n",0);
}

/// Acquires the logical payload lease before entering the recursive renderer.
fn emit_body(e:&mut Emitter) {
    let r=abi::int_result_reg(e);
    e.label_global("__rt_print_r_value_scope_body");abi::emit_frame_prologue(e,32);
    abi::store_at_offset(e,r,8);read_context(e,r,8,TAG);
    for tag in [4,5,6] {branch_tag(e,tag,"__rt_print_r_value_scope_container");}
    abi::emit_jump(e,"__rt_print_r_value_scope_type");
    e.label("__rt_print_r_value_scope_container");read_context(e,r,8,LOW);
    crate::codegen_support::sentinels::emit_branch_if_null_container(
        e,r,abi::secondary_scratch_reg(e),"__rt_print_r_value_scope_null");
    e.label("__rt_print_r_value_scope_type");read_context(e,r,8,TAG);
    for tag in [1,4,5,6,7,11] {branch_tag(e,tag,"__rt_print_r_value_scope_acquire");}
    branch_tag(e,10,"__rt_print_r_value_scope_acquire_callable");
    abi::emit_jump(e,"__rt_print_r_value_scope_invoke");
    e.label("__rt_print_r_value_scope_acquire");read_context(e,r,8,LOW);
    abi::emit_call_label(e,"__rt_incref");abi::emit_jump(e,"__rt_print_r_value_scope_acquired");
    e.label("__rt_print_r_value_scope_acquire_callable");read_context(e,r,8,LOW);
    callable_descriptor::emit_retain_current_descriptor(e);
    e.label("__rt_print_r_value_scope_acquired");context_literal(e,8,OWNED,1);
    abi::emit_jump(e,"__rt_print_r_value_scope_invoke");
    e.label("__rt_print_r_value_scope_null");context_literal(e,8,TAG,8);
    context_literal(e,8,LOW,0);context_literal(e,8,HIGH,0);
    e.label("__rt_print_r_value_scope_invoke");
    for (i,field) in [(0,TAG),(1,LOW),(2,HIGH),(3,INDENT)] {
        read_context(e,abi::int_arg_reg_name(e.target,i),8,field);
    }
    abi::emit_call_label(e,"__rt_print_r_value_impl");
    abi::emit_frame_restore(e,32);abi::emit_return(e);
}
