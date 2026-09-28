//! Purpose:
//! Removes offsets from boxed PHP arrays without renumbering surviving keys.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction` for `Op::OffsetUnset`.
//!
//! Key details:
//! - EIR publishes a detached cell before this lowering promotes and publishes its unique hash.
//! - Destructors can throw or replace the array; no receiver writeback follows value release.
//! - A non-array payload gets PHP's answer: null/false are a no-op, strings and scalars throw.

use crate::codegen::{abi, CodegenIrError, Result};
use crate::codegen::context::FunctionContext;
use crate::codegen::platform::Arch;
use crate::ir::Instruction;
use crate::types::PhpType;

use super::{expect_operand, hashes};

/// Promotes a rooted array cell to sparse storage, then removes the key from its installed hash.
///
/// A cell that holds no array gets PHP's answer instead: null and false are left alone, a string
/// raises `Error("Cannot unset string offsets")`, an object raises `Error("Cannot use object of
/// type C as array")` through `__rt_throw_object_not_array`, and any other payload raises
/// `Error("Cannot unset offset in a non-array variable")`. The object message is PHP's answer for
/// a non-`ArrayAccess` object only; an `ArrayAccess` object held in a boxed cell is not
/// dispatched to its `offsetUnset()` here.
pub(super) fn lower_offset_unset(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let cell = expect_operand(inst, 0)?;
    let key = expect_operand(inst, 1)?;
    if ctx.value_php_type(cell)?.codegen_repr() != PhpType::Mixed {
        return Err(CodegenIrError::invalid_module("offset_unset expects a boxed PHP array"));
    }
    ctx.load_value_to_reg(cell, abi::int_arg_reg_name(ctx.emitter.target, 0))?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_cell_promote_to_hash");
    let valid = ctx.next_label("offset_unset_array");
    let done = ctx.next_label("offset_unset_done");
    let string_offset = ctx.next_label("offset_unset_string");
    let non_array = ctx.next_label("offset_unset_non_array");
    let object = ctx.next_label("offset_unset_object");
    // PHP answers a receiver that is not an array by what it holds: null (a removed untyped
    // property is recreated as null by the write fetch) and false leave everything alone, a
    // string refuses its offsets, and every other scalar is not a container at all.
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbnz x0, {valid}"));              // a nonzero result is the cell's installed unique hash
            ctx.load_value_to_reg(cell, "x9")?;
            ctx.emitter.instruction(&format!("cbz x9, {done}"));                // a missing cell reads as null, which PHP leaves alone
            ctx.emitter.instruction("ldr x10, [x9]");                           // load the runtime tag of the non-array payload
            ctx.emitter.instruction("cmp x10, #8");                             // runtime tag 8 = null
            ctx.emitter.instruction(&format!("b.eq {done}"));                   // unset of an offset of null is a silent no-op
            ctx.emitter.instruction("cmp x10, #1");                             // runtime tag 1 = string
            ctx.emitter.instruction(&format!("b.eq {string_offset}"));          // strings refuse offset removal with their own Error
            ctx.emitter.instruction("cmp x10, #6");                             // runtime tag 6 = object
            ctx.emitter.instruction(&format!("b.eq {object}"));                 // an object refuses indexing with a message naming its class
            ctx.emitter.instruction("cmp x10, #3");                             // runtime tag 3 = bool
            ctx.emitter.instruction(&format!("b.ne {non_array}"));              // every other payload is not a container
            ctx.emitter.instruction("ldr x10, [x9, #8]");                       // load the bool payload
            ctx.emitter.instruction(&format!("cbz x10, {done}"));               // false is left alone like null
            ctx.emitter.instruction(&format!("b {non_array}"));                 // true is not a container
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // a nonzero result is the cell's installed unique hash
            ctx.emitter.instruction(&format!("jnz {valid}"));                   // take the removal path for a real array
            ctx.load_value_to_reg(cell, "r10")?;
            ctx.emitter.instruction("test r10, r10");                           // a missing cell reads as null, which PHP leaves alone
            ctx.emitter.instruction(&format!("jz {done}"));                     // skip the removal for a missing cell
            ctx.emitter.instruction("mov r11, QWORD PTR [r10]");                // load the runtime tag of the non-array payload
            ctx.emitter.instruction("cmp r11, 8");                              // runtime tag 8 = null
            ctx.emitter.instruction(&format!("je {done}"));                     // unset of an offset of null is a silent no-op
            ctx.emitter.instruction("cmp r11, 1");                              // runtime tag 1 = string
            ctx.emitter.instruction(&format!("je {string_offset}"));            // strings refuse offset removal with their own Error
            ctx.emitter.instruction("cmp r11, 6");                              // runtime tag 6 = object
            ctx.emitter.instruction(&format!("je {object}"));                   // an object refuses indexing with a message naming its class
            ctx.emitter.instruction("cmp r11, 3");                              // runtime tag 3 = bool
            ctx.emitter.instruction(&format!("jne {non_array}"));               // every other payload is not a container
            ctx.emitter.instruction("cmp QWORD PTR [r10 + 8], 0");              // is the bool payload false?
            ctx.emitter.instruction(&format!("je {done}"));                     // false is left alone like null
            ctx.emitter.instruction(&format!("jmp {non_array}"));               // true is not a container
        }
    }
    ctx.emitter.label(&object);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x0, [x9, #8]");                        // pass the object so the Error can name its class
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, QWORD PTR [r10 + 8]");            // pass the object so the Error can name its class
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_throw_object_not_array");
    ctx.emitter.label(&string_offset);
    super::exceptions::emit_error(ctx, "Cannot unset string offsets");
    ctx.emitter.label(&non_array);
    super::exceptions::emit_error(ctx, "Cannot unset offset in a non-array variable");
    ctx.emitter.label(&valid);
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            hashes::materialize_hash_key_aarch64(ctx, key)?;
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            hashes::materialize_hash_key_x86_64(ctx, key)?;
            abi::emit_pop_reg(ctx.emitter, "rdi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_hash_unset");
    ctx.emitter.label(&done);
    Ok(())
}
