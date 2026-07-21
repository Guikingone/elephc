//! Purpose:
//! Emits the target-aware assembly for `Op::ThrowCheckedReturnTypeError` — the runtime half of
//! the checked-downcast-on-return guard (`crate::ir_lower::stmt::return_type_guard` emits the
//! `Op::InstanceOf` chain that falls through to this op only when every declared return-type
//! arm mismatches).
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::lower_throw_checked_return_type_error()`.
//!
//! Key details:
//! - Builds `"<prefix><actual runtime class name> returned"` at runtime (the prefix — function
//!   name + declared type — is a compile-time string baked by `ir_lower`; the class name is
//!   looked up dynamically from the mismatched object's header, per the jury addendum requiring
//!   the ACTUAL runtime class, never a static approximation) via two `__rt_concat` calls plus
//!   `__rt_str_persist`, mirrors `objects::reflection::emit_reflection_class_argument_type_error_throw`'s
//!   allocation/publish/unwind tail exactly (stamps `_spl_type_error_class_id`), and additionally
//!   releases the mismatched object (it is never returned to the caller, so nothing else owns it)
//!   before publishing the exception. Never returns.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use crate::codegen::Result;

/// Returns a register pair guaranteed NOT to collide with `abi::int_result_reg` — used to
/// reload the parked message across the object allocation, once `int_result_reg` holds the
/// freshly allocated exception object pointer. Mirrors the scratch pair
/// `objects::reflection::emit_reflection_class_argument_type_error_throw` reloads into.
fn message_scratch_regs(ctx: &FunctionContext<'_>) -> (&'static str, &'static str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ("x9", "x10"),
        Arch::X86_64 => ("r10", "r11"),
    }
}

/// Returns the AArch64/x86_64 `__rt_concat` left-operand (also its output) register pair.
fn concat_lhs_regs(ctx: &FunctionContext<'_>) -> (&'static str, &'static str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ("x1", "x2"),
        Arch::X86_64 => ("rax", "rdx"),
    }
}

/// Returns the AArch64/x86_64 `__rt_concat` right-operand register pair.
fn concat_rhs_regs(ctx: &FunctionContext<'_>) -> (&'static str, &'static str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ("x3", "x4"),
        Arch::X86_64 => ("rdi", "rsi"),
    }
}

/// Loads a compile-time interned string's (ptr, len) into the given register pair.
fn load_static_string_into(ctx: &mut FunctionContext<'_>, label: &str, len: usize, ptr_reg: &str, len_reg: &str) {
    abi::emit_symbol_address(ctx.emitter, ptr_reg, label);
    abi::emit_load_int_immediate(ctx.emitter, len_reg, len as i64);
}

/// Builds `"<prefix><actual class name> returned"` and throws it as a catchable `\TypeError`.
///
/// On entry, the mismatched object's pointer must already be loaded in the target's integer
/// result register (the caller — `lower_throw_checked_return_type_error` — does this via
/// `ctx.load_value_to_result`). `prefix_label`/`prefix_len` name the compile-time message prefix
/// (function name + declared type, already formatted by `ir_lower`). Never returns.
pub(super) fn emit_throw_checked_return_type_error(
    ctx: &mut FunctionContext<'_>,
    prefix_label: &str,
    prefix_len: usize,
) -> Result<()> {
    let object_reg = abi::int_result_reg(ctx.emitter);
    let (suffix_label, suffix_len) = ctx.data.add_string(b" returned");

    // -- park the mismatched object pointer across the message-building/allocation calls, so
    //    it can be released once the exception object no longer needs it --
    abi::emit_push_reg(ctx.emitter, object_reg);

    // -- resolve the mismatched value's ACTUAL runtime class name (never a static
    //    approximation), then move it out of the concat lhs/output slot before it gets
    //    overwritten by the compile-time prefix --
    super::super::builtins::types::emit_dynamic_object_class_name(ctx, "get_class");
    let (lhs_ptr, lhs_len) = concat_lhs_regs(ctx);
    let (rhs_ptr, rhs_len) = concat_rhs_regs(ctx);
    move_reg_pair(ctx, rhs_ptr, rhs_len, lhs_ptr, lhs_len);

    // -- prefix ("F(): Return value must be of type D, ") + actual class name --
    load_static_string_into(ctx, prefix_label, prefix_len, lhs_ptr, lhs_len);
    abi::emit_call_label(ctx.emitter, "__rt_concat");

    // -- append the fixed " returned" suffix --
    let (rhs_ptr, rhs_len) = concat_rhs_regs(ctx);
    load_static_string_into(ctx, &suffix_label, suffix_len, rhs_ptr, rhs_len);
    abi::emit_call_label(ctx.emitter, "__rt_concat");

    // -- own a permanent heap copy of the assembled message --
    emit_persist_from_concat_result(ctx);
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg);

    // -- allocate and stamp the TypeError object (mirrors
    //    `objects::reflection::emit_reflection_class_argument_type_error_throw`) --
    emit_alloc_type_error_header(ctx);
    // Reload the parked message into DEDICATED scratch registers, never `string_result_regs`:
    // on x86_64 that pair IS `(rax, rdx)`, and `rax` already holds the freshly allocated
    // exception object pointer at this point (`int_result_reg` == `rax` on x86_64, unlike
    // AArch64 where the object pointer sits in `x0`, distinct from `(x1, x2)`) — reusing it
    // here would silently clobber the object pointer before its fields are stored.
    let (ptr_reg, len_reg) = message_scratch_regs(ctx);
    abi::emit_pop_reg_pair(ctx.emitter, ptr_reg, len_reg);
    emit_store_message_fields(ctx, ptr_reg, len_reg);

    // -- release the mismatched object: it is never returned to the caller, so this guard
    //    is its only owner. Park the freshly built exception object across the release call. --
    let result_reg = abi::int_result_reg(ctx.emitter);
    let scratch = abi::temp_int_reg(ctx.emitter.target);
    abi::emit_pop_reg(ctx.emitter, scratch); // reload the parked mismatched object pointer
    abi::emit_push_reg(ctx.emitter, result_reg); // park the exception object
    move_reg(ctx, result_reg, scratch);
    abi::emit_decref_if_refcounted(ctx.emitter, &PhpType::Object(String::new()));
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_pop_reg(ctx.emitter, result_reg); // restore the exception object

    abi::emit_store_reg_to_symbol(ctx.emitter, result_reg, "_exc_value", 0); // publish the active exception object
    abi::emit_jump(ctx.emitter, "__rt_throw_current"); // enter the standard exception unwinder
    Ok(())
}

/// Moves a two-register pair `(src_a, src_b)` into `(dst_a, dst_b)`, skipping identity moves.
fn move_reg_pair(ctx: &mut FunctionContext<'_>, dst_a: &str, dst_b: &str, src_a: &str, src_b: &str) {
    move_reg(ctx, dst_a, src_a);
    move_reg(ctx, dst_b, src_b);
}

/// Moves one register into another, skipping identity moves. Both targets share `mov dst, src`
/// operand order in this codebase's emitted syntax (AArch64 native, x86_64 Intel-style).
fn move_reg(ctx: &mut FunctionContext<'_>, dst: &str, src: &str) {
    if dst == src {
        return;
    }
    ctx.emitter.instruction(&format!("mov {}, {}", dst, src));                  // shuffle a scratch register ahead of the next runtime call
}

/// Persists the current `__rt_concat` result (in the lhs/output slot) as an owned heap string.
fn emit_persist_from_concat_result(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own a heap copy of the assembled message (x1/x2 in, x1/x2 out)
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // move the message pointer into the persist argument (length already sits in rdx)
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own a heap copy of the assembled message
        }
    }
}

/// Allocates the 32-byte `Throwable` payload and stamps its object header and class id.
fn emit_alloc_type_error_header(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, #32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");               // allocate the TypeError object payload
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("str x9, [x0]");                            // store the class id at the object header
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, 32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");               // allocate the TypeError object payload
            ctx.emitter.instruction("mov r10, 0x4548504c00000006");             // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the class id at the object header
        }
    }
}

/// Stores the persisted message pointer/length and a zero exception code into the freshly
/// allocated `TypeError` object (whose pointer sits in the target's integer result register).
fn emit_store_message_fields(ctx: &mut FunctionContext<'_>, ptr_reg: &str, len_reg: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("str {}, [x0, #8]", ptr_reg));     // store the exception message pointer
            ctx.emitter.instruction(&format!("str {}, [x0, #16]", len_reg));    // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov QWORD PTR [rax + 8], {}", ptr_reg)); // store the exception message pointer
            ctx.emitter.instruction(&format!("mov QWORD PTR [rax + 16], {}", len_reg)); // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
        }
    }
}
