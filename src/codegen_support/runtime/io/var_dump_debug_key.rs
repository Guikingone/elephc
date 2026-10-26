//! Purpose:
//! Formats decoded PHP debug-property keys for var_dump and print_r.
//! Keeps each renderer's punctuation separate from the shared borrowed-key decoder.
//!
//! Called from:
//! - The var_dump debug hash and native print_r magic-view entry walkers.
//!
//! Key details:
//! - Source key bytes remain borrowed from a retained debug snapshot.
//! - Print_r notices run outside capture and defer handler exceptions to the render scope.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
use super::print_r_php_call::emit_guarded_php_call;
const FRAME:usize=112;
const KEY:usize=8;
const LENGTH:usize=16;
const INDENT:usize=24;
const STATUS:usize=32;
const PENDING:usize=40;
const PARTS:usize=96;

/// Emits the shared decoder and var_dump's key-output entry.
pub fn emit_var_dump_debug_key(e:&mut Emitter) {
    super::debug_key_parts::emit_debug_key_parts(e);
    emit_key(e,false);
}

/// Emits object-key spelling for print_r without changing ordinary array keys.
pub fn emit_print_r_debug_key(e:&mut Emitter) { emit_key(e,true); }

/// Formats one decoder result through the selected renderer's output ABI.
fn emit_key(e:&mut Emitter, print_r:bool) {
    let result=abi::int_result_reg(e);
    let prefix=if print_r {"__rt_pr_debug_key_"}else{"__rt_vd_debug_key_"};
    let label=|s:&str|format!("{prefix}{s}");
    let ptr=if print_r {abi::int_arg_reg_name(e.target,0)}else if e.target.arch==Arch::AArch64 {"x1"}else{"rdi"};
    let len=if print_r {abi::int_arg_reg_name(e.target,1)}else if e.target.arch==Arch::AArch64 {"x2"}else{"rsi"};
    e.label_global(if print_r {"__rt_print_r_debug_key"}else{"__rt_var_dump_emit_debug_key"});
    abi::emit_frame_prologue(e,FRAME);
    abi::store_at_offset(e,ptr,KEY);abi::store_at_offset(e,len,LENGTH);
    if print_r {abi::store_at_offset(e,abi::int_arg_reg_name(e.target,2),INDENT);}
    abi::emit_store_zero_to_local_slot(e,PENDING);
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),KEY);
    abi::load_at_offset(e,abi::int_arg_reg_name(e.target,1),LENGTH);
    abi::emit_frame_slot_address(e,abi::int_arg_reg_name(e.target,2),PARTS);
    abi::emit_call_label(e,"__rt_debug_key_parts");
    abi::store_at_offset(e,result,STATUS);
    for (status,target) in [(0,"public"),(3,"illegal"),(4,"corrupt")] { equal(e,status,&label(target)); }
    if print_r {
        abi::load_at_offset(e,abi::int_arg_reg_name(e.target,0),INDENT);
        abi::emit_call_label(e,"__rt_print_r_spaces");
    }else{abi::emit_call_label(e,"__rt_vd_pad");}
    literal(e,print_r,"open",if print_r {1}else{2});
    bytes(e,print_r,PARTS,PARTS-8);
    abi::load_at_offset(e,result,STATUS);equal(e,1,&label("protected"));
    literal(e,print_r,"private_mid",if print_r {1}else{3});
    bytes(e,print_r,PARTS-16,PARTS-24);
    literal(e,print_r,"private",if print_r {":private] => ".len()}else{"\":private]=>\n".len()});
    abi::emit_jump(e,&label("done"));
    e.label(&label("protected"));
    literal(e,print_r,"protected",if print_r {":protected] => ".len()}else{"\":protected]=>\n".len()});
    abi::emit_jump(e,&label("done"));
    for (suffix,message) in [("illegal","Notice: Illegal member variable name\n"),("corrupt","Notice: Corrupt member variable name\n")] {
        e.label(&label(suffix));
        if print_r {
            abi::emit_load_int_immediate(e,result,0);
            emit_guarded_php_call(e,&format!("__rt_pr_debug_key_notice_{suffix}"),result,PENDING);
            abi::load_at_offset(e,result,PENDING);
            abi::emit_store_zero_to_local_slot(e,PENDING);
            abi::emit_call_label(e,"__rt_print_r_record_exception");
        }else{notice(e,&format!("_vd_debug_key_{suffix}"),message.len());}
        abi::emit_jump(e,&label("public"));
    }
    e.label(&label("public"));
    abi::load_at_offset(e,ptr,KEY);abi::load_at_offset(e,len,LENGTH);
    if print_r {abi::load_at_offset(e,abi::int_arg_reg_name(e.target,2),INDENT);}
    abi::emit_frame_restore(e,FRAME);
    abi::emit_jump(e,if print_r {"__rt_print_r_str_key"}else{"__rt_var_dump_emit_string_key"});
    e.label(&label("done"));abi::emit_frame_restore(e,FRAME);abi::emit_return(e);
    if print_r {
        for (suffix,message) in [("illegal","Notice: Illegal member variable name\n"),("corrupt","Notice: Corrupt member variable name\n")] {
            e.label_global(&format!("__rt_pr_debug_key_notice_{suffix}"));
            abi::emit_frame_prologue(e,16);notice(e,&format!("_pr_debug_key_{suffix}"),message.len());
            abi::emit_frame_restore(e,16);abi::emit_return(e);
        }
    }
    e.raw(".data");
    let fields=if print_r {
        [("open","["),("protected",":protected] => "),("private_mid",":"),("private",":private] => "),("illegal","Notice: Illegal member variable name\n"),("corrupt","Notice: Corrupt member variable name\n")]
    }else{
        [("open","[\""),("protected","\":protected]=>\n"),("private_mid","\":\""),("private","\":private]=>\n"),("illegal","Notice: Illegal member variable name\n"),("corrupt","Notice: Corrupt member variable name\n")]
    };
    for (suffix,text) in fields {e.label(&format!("_{}_debug_key_{suffix}",if print_r {"pr"}else{"vd"}));e.raw(&format!("    .ascii {text:?}"));}
    e.raw(".text");
}

/// Branches on a decoder status without clobbering its saved slice pointers.
fn equal(e:&mut Emitter,status:i64,label:&str) {
    let r=abi::int_result_reg(e);
    match e.target.arch {
        Arch::AArch64=>{
                        e.instruction(&format!("cmp {r}, #{status}"));                      // select the matching property-key formatting status

                        e.instruction(&format!("b.eq {label}"));                            // select the matching property-key formatting status
        }
        Arch::X86_64=>{
                        e.instruction(&format!("cmp {r}, {status}"));                       // select the matching property-key formatting status

                        e.instruction(&format!("je {label}"));                              // select the matching property-key formatting status
        }
    }
}

/// Writes a renderer-specific punctuation fragment.
fn literal(e:&mut Emitter,pr:bool,suffix:&str,len:usize) {
    let (ptr,length)=output_registers(e);abi::emit_symbol_address(e,ptr,&format!("_{}_debug_key_{suffix}",if pr {"pr"}else{"vd"}));
    abi::emit_load_int_immediate(e,length,len as i64);abi::emit_call_label(e,if pr {"__rt_pr_write"}else{"__rt_vd_write"});
}

/// Writes one borrowed decoded name or class slice.
fn bytes(e:&mut Emitter,pr:bool,p:usize,n:usize) {
    let (ptr,len)=output_registers(e);abi::load_at_offset(e,ptr,p);abi::load_at_offset(e,len,n);
    abi::emit_call_label(e,if pr {"__rt_pr_write"}else{"__rt_vd_write"});
}

/// Delivers a malformed-key notice through the ordinary diagnostic dispatcher.
fn notice(e:&mut Emitter,symbol:&str,length:usize) {
    let (ptr,len)=if e.target.arch==Arch::AArch64 {("x1","x2")}else{("rdi","rsi")};
    abi::emit_symbol_address(e,ptr,symbol);abi::emit_load_int_immediate(e,len,length as i64);abi::emit_call_label(e,"__rt_diag_warning");
}

/// Returns the common byte-writer register pair for either renderer.
fn output_registers(e:&Emitter)->(&'static str,&'static str) {
    if e.target.arch==Arch::AArch64 {("x1","x2")}else{("rsi","rdx")}
}
