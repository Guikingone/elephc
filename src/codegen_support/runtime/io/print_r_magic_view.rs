//! Purpose:
//! Renders native print_r magic-debug views with retained snapshot ownership.
//! Runs PHP hooks and view destruction outside the active output capture.
//!
//! Called from:
//! - The object walker after recursion detection and before marking the receiver.
//!
//! Key details:
//! - The enclosing recursive value scope keeps the receiver alive through the view.
//! - Entries use the uniform logical iterator, preserving numeric keys and hash insertion order.
//! - Temporary-view exceptions stay pending until buffered output is complete.

use crate::codegen_support::{abi,emit::Emitter,platform::Arch};
use super::print_r_php_call::emit_guarded_php_call;
use super::var_dump_value_scope::{read_context,write_context,branch_tag};
const CONTEXT:usize=80;
const FRAME:usize=96;
const OBJECT:usize=0;
const BASE:usize=8;
const OWNER:usize=16;
const DATA:usize=24;
const KIND:usize=32;
const PENDING:usize=40;
const ROOT:usize=48;
const SEEN:usize=56;

/// Emits a handled/absent object-body adapter with one protected magic snapshot.
pub fn emit_print_r_magic_view(e:&mut Emitter) {
    let r=abi::int_result_reg(e);
    e.label_global("__rt_print_r_magic_view");abi::emit_frame_prologue(e,FRAME);
    for (i,field) in [(0,OBJECT),(1,BASE)] {abi::store_at_offset(e,abi::int_arg_reg_name(e.target,i),CONTEXT-field);}
    for field in [OWNER,DATA,PENDING] {abi::emit_store_zero_to_local_slot(e,CONTEXT-field);}
    abi::emit_load_symbol_to_reg(e,r,"_print_r_scope",0);abi::store_at_offset(e,r,CONTEXT-ROOT);
    abi::emit_load_symbol_to_reg(e,r,"_vd_seen_n",0);abi::store_at_offset(e,r,CONTEXT-SEEN);
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),CONTEXT-OBJECT);
    abi::emit_call_label(e,"__rt_object_debug_info_template");
    abi::emit_branch_if_int_result_zero(e,"__rt_print_r_magic_view_absent");
    abi::load_at_offset(e,r,CONTEXT-ROOT);
    abi::emit_branch_if_int_result_zero(e,"__rt_print_r_magic_view_fetch");
    abi::emit_load_from_address(e,r,r,96);
    abi::emit_branch_if_int_result_nonzero(e,"__rt_print_r_magic_view_fatal");
    e.label("__rt_print_r_magic_view_fetch");
    abi::emit_frame_slot_address(e,r,CONTEXT);
    emit_guarded_php_call(e,"__rt_print_r_magic_fetch",r,CONTEXT-PENDING);
    abi::emit_branch_if_int_result_nonzero(e,"__rt_print_r_magic_view_fatal");
    abi::load_at_offset(e,r,CONTEXT-OWNER);
    abi::emit_branch_if_int_result_zero(e,"__rt_print_r_magic_view_fatal");
    abi::emit_call_label(e,"__rt_mixed_unbox");
    branch_tag(e,4,"__rt_print_r_magic_view_array");branch_tag(e,5,"__rt_print_r_magic_view_array");
    branch_tag(e,8,"__rt_print_r_magic_view_null");
    abi::emit_jump(e,"__rt_print_r_magic_view_fatal");
    e.label("__rt_print_r_magic_view_array");
    abi::store_at_offset(e,r,CONTEXT-KIND);
    let low=if e.target.arch==Arch::AArch64 {"x1"}else{"rdi"};
    abi::store_at_offset(e,low,CONTEXT-DATA);abi::emit_reg_move(e,r,low);
    abi::emit_call_label(e,"__rt_incref");
    release_owner(e);
    abi::emit_jump(e,"__rt_print_r_magic_view_open");
    e.label("__rt_print_r_magic_view_null");
    abi::emit_load_int_immediate(e,r,8);abi::store_at_offset(e,r,CONTEXT-KIND);
    release_owner(e);
    abi::load_at_offset(e,r,CONTEXT-OBJECT);abi::emit_load_from_address(e,r,r,0);
    emit_guarded_php_call(e,"__rt_debug_info_warn_null",r,CONTEXT-PENDING);
    e.label("__rt_print_r_magic_view_open");
    abi::load_at_offset(e,r,CONTEXT-PENDING);abi::emit_store_zero_to_local_slot(e,CONTEXT-PENDING);
    abi::emit_call_label(e,"__rt_print_r_record_exception");
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),CONTEXT-OBJECT);
    abi::emit_call_label(e,"__rt_vd_seen_push");
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),CONTEXT-BASE);
    abi::emit_call_label(e,"__rt_print_r_open");
    abi::load_at_offset(e,r,CONTEXT-DATA);
    abi::emit_branch_if_int_result_zero(e,"__rt_print_r_magic_view_close");
    abi::emit_reg_move(e,abi::int_arg_reg_name(e.target,0),r);
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,1),CONTEXT-BASE);
    match e.target.arch {
        Arch::AArch64=>
                        e.instruction("add x1, x1, #4"),                                    // preserve the logical debug-view cursor and rendering order

        Arch::X86_64=>
                        e.instruction("add rsi, 4"),                                        // preserve the logical debug-view cursor and rendering order

    }
    abi::emit_call_label(e,"__rt_print_r_magic_entries");
    e.label("__rt_print_r_magic_view_close");
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),CONTEXT-BASE);
    abi::emit_call_label(e,"__rt_print_r_close");
    abi::load_at_offset(e,r,CONTEXT-SEEN);abi::emit_store_reg_to_symbol(e,r,"_vd_seen_n",0);
    abi::load_at_offset(e,r,CONTEXT-DATA);abi::emit_store_zero_to_local_slot(e,CONTEXT-DATA);
    emit_guarded_php_call(e,"__rt_decref_any",r,CONTEXT-PENDING);
    abi::load_at_offset(e,r,CONTEXT-PENDING);abi::emit_store_zero_to_local_slot(e,CONTEXT-PENDING);
    abi::emit_call_label(e,"__rt_print_r_record_exception");
    abi::emit_load_int_immediate(e,r,1);abi::emit_frame_restore(e,FRAME);abi::emit_return(e);
    e.label("__rt_print_r_magic_view_absent");
    abi::emit_load_int_immediate(e,r,0);abi::emit_frame_restore(e,FRAME);abi::emit_return(e);
    e.label("__rt_print_r_magic_view_fatal");
    abi::emit_store_zero_to_symbol(e,"_print_r_mode",0);
    abi::emit_jump(e,"__rt_debug_info_invalid_result");
    emit_fetch(e);emit_entries(e);
}

/// Retires only the result cell after transferring the array owner into this view.
fn release_owner(e:&mut Emitter) {
    let r=abi::int_result_reg(e);abi::load_at_offset(e,r,CONTEXT-OWNER);
    abi::emit_store_zero_to_local_slot(e,CONTEXT-OWNER);
    emit_guarded_php_call(e,"__rt_decref_any",r,CONTEXT-PENDING);
}

/// Invokes the ordinary debug descriptor and publishes its result before leaving the PHP boundary.
fn emit_fetch(e:&mut Emitter) {
    let r=abi::int_result_reg(e);
    e.label_global("__rt_print_r_magic_fetch");abi::emit_frame_prologue(e,32);
    abi::store_at_offset(e,r,8);
    read_context(e,abi::int_arg_reg_name(e.target,0),8,OBJECT);
    abi::emit_call_label(e,"__rt_object_debug_info");
    write_context(e,r,8,OWNER);
    abi::emit_frame_restore(e,32);abi::emit_return(e);
}

/// Walks either array layout through its logical cursor and preserves borrowed keys across recursion.
fn emit_entries(e:&mut Emitter) {
    let r=abi::int_result_reg(e);
    const ARRAY:usize=8;const INDENT:usize=16;const CURSOR:usize=24;
    const KEY:usize=32;const KEY_LENGTH:usize=40;const TAG:usize=48;const LOW:usize=56;const HIGH:usize=64;
    e.label_global("__rt_print_r_magic_entries");abi::emit_frame_prologue(e,96);
    abi::store_at_offset(e,abi::int_arg_reg_name(e.target,0),ARRAY);
    abi::store_at_offset(e,abi::int_arg_reg_name(e.target,1),INDENT);
    abi::emit_store_zero_to_local_slot(e,CURSOR);
    e.label("__rt_print_r_magic_entries_loop");
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),ARRAY);
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,1),CURSOR);
    abi::emit_call_label(e,"__rt_array_iter_next");
    match e.target.arch {
        Arch::AArch64=>{
                        e.instruction("cmn x0, #1");                                        // preserve the logical debug-view cursor and rendering order

                        e.instruction("b.eq __rt_print_r_magic_entries_done");              // preserve the logical debug-view cursor and rendering order
        }
        Arch::X86_64=>{
                        e.instruction("cmp rax, -1");                                       // preserve the logical debug-view cursor and rendering order

                        e.instruction("je __rt_print_r_magic_entries_done");                // preserve the logical debug-view cursor and rendering order
        }
    }
    abi::store_at_offset(e,r,CURSOR);
    let regs=if e.target.arch==Arch::AArch64 {["x1","x2","x3","x4","x5"]}else{["rcx","rdx","r8","r9","r10"]};
    for (reg,slot) in regs.into_iter().zip([KEY,KEY_LENGTH,TAG,LOW,HIGH]) {abi::store_at_offset(e,reg,slot);}
    abi::load_at_offset(e,r,KEY_LENGTH);
    match e.target.arch {
        Arch::AArch64=>{
                        e.instruction("cmn x0, #1");                                        // preserve the logical debug-view cursor and rendering order

                        e.instruction("b.eq __rt_print_r_magic_entries_integer");           // preserve the logical debug-view cursor and rendering order
        }
        Arch::X86_64=>{
                        e.instruction("cmp rax, -1");                                       // preserve the logical debug-view cursor and rendering order

                        e.instruction("je __rt_print_r_magic_entries_integer");             // preserve the logical debug-view cursor and rendering order
        }
    }
    for (i,slot) in [(0,KEY),(1,KEY_LENGTH),(2,INDENT)] {abi::load_at_offset(e,abi::int_arg_reg_name(e.target,i),slot);}
    abi::emit_call_label(e,"__rt_print_r_debug_key");
    abi::emit_jump(e,"__rt_print_r_magic_entries_value");
    e.label("__rt_print_r_magic_entries_integer");
    for (i,slot) in [(0,KEY),(1,INDENT)] {abi::load_at_offset(e,abi::int_arg_reg_name(e.target,i),slot);}
    abi::emit_call_label(e,"__rt_print_r_int_key");
    e.label("__rt_print_r_magic_entries_value");
    for (i,slot) in [(0,TAG),(1,LOW),(2,HIGH),(3,INDENT)] {abi::load_at_offset(e,abi::int_arg_reg_name(e.target,i),slot);}
    match e.target.arch {
        Arch::AArch64=>
                        e.instruction("add x3, x3, #4"),                                    // preserve the logical debug-view cursor and rendering order

        Arch::X86_64=>
                        e.instruction("add rcx, 4"),                                        // preserve the logical debug-view cursor and rendering order

    }
    abi::emit_call_label(e,"__rt_print_r_value");
    let (ptr,len)=if e.target.arch==Arch::AArch64 {("x1","x2")}else{("rsi","rdx")};
    abi::emit_symbol_address(e,ptr,"_pr_nl");abi::emit_load_int_immediate(e,len,1);
    abi::emit_call_label(e,"__rt_pr_write");abi::emit_jump(e,"__rt_print_r_magic_entries_loop");
    e.label("__rt_print_r_magic_entries_done");abi::emit_frame_restore(e,96);abi::emit_return(e);
}
