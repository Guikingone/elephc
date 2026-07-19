//! Purpose:
//! Lowers PHP output-buffering and header-control builtins for the EIR
//! backend: `ob_start()`/`ob_get_clean()`/`ob_end_flush()`/`ob_end_clean()`/
//! `ob_get_contents()`/`ob_get_level()`/`ob_get_status()`, `headers_sent()`,
//! `flush()`, and `header_remove()`.
//!
//! Called from:
//! - `crate::codegen_ir::lower_inst::builtins::lower_builtin_call()`.
//!
//! Key details:
//! - `ob_start($callback)`/`ob_start(..., $chunk_size)` are rejected before
//!   lowering ever runs: `ob_start` has a zero-parameter signature in
//!   `crate::types::signatures`, so passing extra arguments is a PHP-faithful
//!   arity `CompileError` (never accept-and-ignore a callback).
//! - `ob_get_status($full_status = true)` is a documented residual: only the
//!   omitted/literal-`false` form (current-level status) is lowered; a literal
//!   `true` or a non-literal argument is a loud `CodegenIrError::unsupported`.
//! - `headers_sent(&$file, &$line)`: elephc does not track the source
//!   location where output first left the buffer stack, so `$file`/`$line`
//!   are always written as `""`/`0` — php-verified that real PHP overwrites
//!   BOTH out-params even when returning `false` (never leaves them
//!   untouched), so writing them unconditionally matches that contract; only
//!   the exact non-empty `$file`/`$line` VALUES on the `true` branch are a
//!   disclosed approximation.
//! - `header_remove(?string $name)` only accepts a literal-absent call or a
//!   `Str`-typed argument (a `?string`/Mixed/Union argument is out of scope —
//!   loud, never silently coerced) and forwards to the `--web`-gated
//!   `__rt_header_remove` (a genuine no-op outside `--web`, mirroring
//!   `header()`'s own web-gating).

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen_ir::{CodegenIrError, Result};
use crate::ir::{Immediate, Instruction, Op, ValueDef, ValueId};
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::store_if_result;

/// Lowers `ob_start(): bool` — always succeeds (`true`) for elephc's one
/// supported (callback-free) buffer form.
pub(super) fn lower_ob_start(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count(inst, "ob_start", 0)?;
    abi::emit_call_label(ctx.emitter, "__rt_ob_start");
    store_if_result(ctx, inst)
}

/// Lowers `ob_get_contents(): string|false` — peeks the current top level's
/// bytes without popping it.
pub(super) fn lower_ob_get_contents(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count(inst, "ob_get_contents", 0)?;
    abi::emit_call_label(ctx.emitter, "__rt_ob_peek_contents");
    super::io::box_owned_string_or_false_result(ctx, "ob_get_contents");
    store_if_result(ctx, inst)
}

/// Lowers `ob_get_clean(): string|false` — peeks the current top level's
/// bytes, THEN pops it (discarding the level without flushing it through).
pub(super) fn lower_ob_get_clean(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count(inst, "ob_get_clean", 0)?;
    abi::emit_call_label(ctx.emitter, "__rt_ob_peek_contents");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");                   // preserve the peeked (ptr, len) across the pop call
            abi::emit_call_label(ctx.emitter, "__rt_ob_pop");
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                 // preserve the peeked (ptr, len) across the pop call
            abi::emit_call_label(ctx.emitter, "__rt_ob_pop");
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
        }
    }
    super::io::box_owned_string_or_false_result(ctx, "ob_get_clean");
    store_if_result(ctx, inst)
}

/// Lowers `ob_end_clean(): bool` — discards the current top level without
/// flushing it. `false` (plus a stderr notice) on an empty stack.
pub(super) fn lower_ob_end_clean(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count(inst, "ob_end_clean", 0)?;
    abi::emit_call_label(ctx.emitter, "__rt_ob_end_clean");
    store_if_result(ctx, inst)
}

/// Lowers `ob_end_flush(): bool` — writes the current top level's bytes
/// through to whatever is below it, then pops it. `false` (plus a stderr
/// notice) on an empty stack.
pub(super) fn lower_ob_end_flush(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count(inst, "ob_end_flush", 0)?;
    abi::emit_call_label(ctx.emitter, "__rt_ob_end_flush");
    store_if_result(ctx, inst)
}

/// Lowers `ob_get_level(): int`.
pub(super) fn lower_ob_get_level(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count(inst, "ob_get_level", 0)?;
    abi::emit_call_label(ctx.emitter, "__rt_ob_get_level");
    store_if_result(ctx, inst)
}

/// Lowers `ob_get_status($full_status = false): array`. Only the omitted or
/// literal-`false` form is supported; see the file-level residual note.
pub(super) fn lower_ob_get_status(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count_between(inst, "ob_get_status", 0, 1)?;
    if let Some(arg) = inst.operands.first().copied() {
        match optional_const_bool_operand(ctx, arg)? {
            Some(false) => {}
            Some(true) => {
                return Err(CodegenIrError::unsupported(
                    "ob_get_status(true): full output-buffer stack status is not supported by elephc (use ob_get_status() for the current buffer level)",
                ));
            }
            None => {
                return Err(CodegenIrError::unsupported(
                    "ob_get_status(): $full_status must be a compile-time literal bool",
                ));
            }
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_ob_get_status");
    store_if_result(ctx, inst)
}

/// Lowers `headers_sent(?string &$file = null, ?int &$line = null): bool`.
/// `$file`/`$line` are always written `""`/`0` (see the file-level doc
/// comment); the by-ref writes happen AFTER the runtime call, both preserving
/// the returned bool across them (via the shared `LocalSlotId` writer helpers).
pub(super) fn lower_headers_sent(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count_between(inst, "headers_sent", 0, 2)?;
    abi::emit_call_label(ctx.emitter, "__rt_headers_sent");
    let file_slot = match inst.operands.first().copied() {
        Some(v) => source_load_local_slot(ctx, v)?,
        None => None,
    };
    let line_slot = match inst.operands.get(1).copied() {
        Some(v) => source_load_local_slot(ctx, v)?,
        None => None,
    };
    if file_slot.is_none() && line_slot.is_none() {
        return store_if_result(ctx, inst);
    }
    let (empty_sym, _) = ctx.data.add_string(b"");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg(ctx.emitter, "x0");                              // preserve the returned bool across the out-param writes
            if let Some(slot) = file_slot {
                abi::emit_symbol_address(ctx.emitter, "x9", &empty_sym);
                store_string_output_to_local(ctx, slot, "x9", "xzr")?;
            }
            if let Some(slot) = line_slot {
                store_int_output_to_local(ctx, slot, "xzr")?;
            }
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            abi::emit_push_reg(ctx.emitter, "rax");                             // preserve the returned bool across the out-param writes
            if let Some(slot) = file_slot {
                abi::emit_symbol_address(ctx.emitter, "r9", &empty_sym);
                ctx.emitter.instruction("xor r10, r10");                        // zero-length empty string
                store_string_output_to_local(ctx, slot, "r9", "r10")?;
            }
            if let Some(slot) = line_slot {
                ctx.emitter.instruction("xor r10, r10");
                store_int_output_to_local(ctx, slot, "r10")?;
            }
            abi::emit_pop_reg(ctx.emitter, "rax");
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers `flush(): void` as a sound no-op: elephc's stdout writes are
/// already unbuffered syscalls (or `--web` per-request appends), so there is
/// nothing to flush at the syscall layer — a true no-op is PHP-faithful here
/// (php -n verified: `flush()` returns `NULL`/void, and CLI output is
/// observably identical with or without the call).
pub(super) fn lower_flush(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count(inst, "flush", 0)?;
    store_if_result(ctx, inst)
}

/// Lowers `header_remove(?string $name)` through the `--web`-gated
/// `__rt_header_remove` (a genuine no-op outside `--web`). No argument means
/// "remove every header" (`name_len = -1`, the in-band sentinel the bridge
/// expects — see `crate::codegen::runtime::io::http_response`); a `Str`
/// argument removes only the matching header. Any other argument type is
/// unsupported (loud), never silently coerced.
pub(super) fn lower_header_remove(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count_between(inst, "header_remove", 0, 1)?;
    match inst.operands.first().copied() {
        Some(name) => {
            let ty = ctx.load_value_to_result(name)?.codegen_repr();
            if ty != PhpType::Str {
                return Err(CodegenIrError::unsupported(format!(
                    "header_remove() argument must be a string in AOT mode (got {:?})",
                    ty
                )));
            }
            super::io::load_string_to_result(ctx, name, "header_remove name")?;
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction("mov x0, x1");                      // name pointer -> first argument register
                    ctx.emitter.instruction("mov x1, x2");                      // name length -> second argument register
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction("mov rdi, rax");                    // name pointer -> first argument register
                    ctx.emitter.instruction("mov rsi, rdx");                    // name length -> second argument register
                }
            }
        }
        None => match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction("mov x0, #0");                          // no name pointer needed
                ctx.emitter.instruction("mov x1, #-1");                         // "no argument" sentinel: remove every header
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("xor edi, edi");                        // no name pointer needed
                ctx.emitter.instruction("mov rsi, -1");                         // "no argument" sentinel: remove every header
            }
        },
    }
    abi::emit_call_label(ctx.emitter, "__rt_header_remove");
    store_if_result(ctx, inst)
}

/// Returns a literal bool operand when the value was produced by `ConstBool`.
fn optional_const_bool_operand(ctx: &FunctionContext<'_>, value: ValueId) -> Result<Option<bool>> {
    let value_ref = ctx
        .function
        .value(value)
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Ok(None);
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    if inst_ref.op != Op::ConstBool {
        return Ok(None);
    }
    match inst_ref.immediate {
        Some(Immediate::Bool(value)) => Ok(Some(value)),
        _ => Err(CodegenIrError::invalid_module(
            "bool literal operand has no bool immediate",
        )),
    }
}

/// Returns the local slot loaded by a `headers_sent()` by-ref operand when it
/// came from `load_local` (mirrors the same helper duplicated across several
/// `builtins/*.rs` files for their own by-ref out-params).
fn source_load_local_slot(
    ctx: &FunctionContext<'_>,
    value: ValueId,
) -> Result<Option<crate::ir::LocalSlotId>> {
    let Some(value_ref) = ctx.function.value(value) else {
        return Err(CodegenIrError::missing_entry("value", value.as_raw()));
    };
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Ok(None);
    };
    let Some(inst_ref) = ctx.function.instruction(inst) else {
        return Err(CodegenIrError::missing_entry("instruction", inst.as_raw()));
    };
    if inst_ref.op == Op::LoadLocal {
        if let Some(Immediate::LocalSlot(slot)) = inst_ref.immediate {
            return Ok(Some(slot));
        }
    }
    Ok(None)
}

/// Stores an integer output (`0`) into a local slot, boxing it when the slot
/// is `Mixed` (mirrors `builtins/io.rs`'s `store_int_output_to_local`).
fn store_int_output_to_local(
    ctx: &mut FunctionContext<'_>,
    slot: crate::ir::LocalSlotId,
    value_reg: &str,
) -> Result<()> {
    let offset = ctx.local_offset(slot)?;
    if ctx.local_php_type(slot)?.codegen_repr() == PhpType::Mixed {
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction(&format!("mov x0, {}", value_reg));     // move the int payload into the canonical integer result register
            }
            Arch::X86_64 => {
                ctx.emitter.instruction(&format!("mov rax, {}", value_reg));    // move the int payload into the canonical integer result register
            }
        }
        crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Int);
        abi::store_at_offset(ctx.emitter, abi::int_result_reg(ctx.emitter), offset);
        return Ok(());
    }
    abi::store_at_offset_scratch(ctx.emitter, value_reg, offset, "x13");
    Ok(())
}

/// Stores a string output (`""`) into a local slot, boxing it when the slot
/// is `Mixed` (mirrors `builtins/io.rs`'s `store_string_output_to_local`).
fn store_string_output_to_local(
    ctx: &mut FunctionContext<'_>,
    slot: crate::ir::LocalSlotId,
    ptr_reg: &str,
    len_reg: &str,
) -> Result<()> {
    let offset = ctx.local_offset(slot)?;
    if ctx.local_php_type(slot)?.codegen_repr() == PhpType::Mixed {
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction(&format!("mov x1, {}", ptr_reg));       // move the empty-string pointer into the canonical string result register
                ctx.emitter.instruction(&format!("mov x2, {}", len_reg));       // move the empty-string length into the canonical string result register
            }
            Arch::X86_64 => {
                ctx.emitter.instruction(&format!("mov rax, {}", ptr_reg));      // move the empty-string pointer into the canonical string result register
                ctx.emitter.instruction(&format!("mov rdx, {}", len_reg));      // move the empty-string length into the canonical string result register
            }
        }
        crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);
        abi::store_at_offset(ctx.emitter, abi::int_result_reg(ctx.emitter), offset);
        return Ok(());
    }
    abi::store_at_offset_scratch(ctx.emitter, ptr_reg, offset, "x13");
    abi::store_at_offset_scratch(ctx.emitter, len_reg, offset - 8, "x13");
    Ok(())
}
