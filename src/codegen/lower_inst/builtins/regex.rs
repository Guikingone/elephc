//! Purpose:
//! Lowers simple PCRE-style regex builtins for the EIR backend.
//! Bridges already-evaluated EIR operands to the shared target-aware regex runtime helpers.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::lower_builtin_call()`.
//!
//! Key details:
//! - `preg_match()` captures support both a direct local `$matches` variable and a
//!   by-reference parameter cell (`?array &$matches`), the latter written through the cell
//!   into the caller's storage exactly like a user-defined `&$param` writeback.
//! - `preg_replace_callback()` supports static string callbacks and descriptor-backed
//!   callable values through a regex-specific callback wrapper.
//! - `preg_split()` forces boxed Mixed element slots so dynamic flags cannot mismatch layout.

use crate::codegen::platform::Arch;
use crate::codegen::{abi, callable_descriptor};
use crate::codegen::{CodegenIrError, Result};
use crate::codegen_support::DeferredCallbackWrapper;
use crate::codegen_support::emit_box_current_value_as_mixed;
use crate::ir::{Immediate, Instruction, LocalSlotId, Op, ValueDef, ValueId};
use crate::names::function_symbol;
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::super::callables;

const PREG_SPLIT_FORCE_MIXED_RESULT: i64 = 1 << 30;

/// Lowers `preg_match(pattern, subject, &matches?, flags?, offset?)` via the regex runtime.
///
/// The optional `$matches` out-parameter is populated through
/// `__rt_preg_match_capture`. The `$flags` and `$offset` arguments are accepted
/// (so calls type-check and lower) but are not yet honored by the EIR capture
/// runtime; non-default flags/offset therefore behave as the defaults.
pub(crate) fn lower_preg_match(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_match", 2, 5)?;
    let pattern = super::expect_operand(inst, 0)?;
    let subject = super::expect_operand(inst, 1)?;
    let matches_target = inst
        .operands
        .get(2)
        .copied()
        .map(|value| matches_target(ctx, value))
        .transpose()?;
    load_pattern_and_subject(ctx, pattern, subject)?;
    if let Some(target) = &matches_target {
        // The capture helper builds an associative hash (so `$m['name']` reads work)
        // only when the destination is a boxed-Mixed cell and the pattern actually has
        // named groups. Signal that permission through the flag register so plain indexed
        // `$matches` locals keep their fast contiguous layout.
        let allow_hash = target_allows_named_hash(target);
        let flag_reg = match ctx.emitter.target.arch {
            Arch::AArch64 => "x5",
            Arch::X86_64 => "r8",
        };
        abi::emit_load_int_immediate(ctx.emitter, flag_reg, allow_hash as i64);
        abi::emit_call_label(ctx.emitter, "__rt_preg_match_capture");
        store_matches_array(ctx, target)?;
    } else {
        abi::emit_call_label(ctx.emitter, "__rt_preg_match");
    }
    super::store_if_result(ctx, inst)
}

/// Lowers `mb_ereg_match(pattern, subject, options = null)` as a start-anchored regex match.
///
/// The bare delimiter-less pattern and subject use the shared regex string loader. Optional
/// options are passed as a string pair when present, or as `(0, 0)` for `null`/omitted options.
pub(crate) fn lower_mb_ereg_match(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::ensure_arg_count_between(inst, "mb_ereg_match", 2, 3)?;
    let pattern = super::expect_operand(inst, 0)?;
    let subject = super::expect_operand(inst, 1)?;
    let options = inst.operands.get(2).copied();
    load_mb_ereg_match_args(ctx, pattern, subject, options)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_ereg_match");
    super::store_if_result(ctx, inst)
}

/// Lowers `preg_match_all(pattern, subject)` through the shared regex runtime helper.
/// Lowers `preg_match_all(pattern, subject, &matches?, flags?, offset?)` through the regex runtime.
///
/// The optional `$matches` out-parameter is populated through `__rt_preg_match_capture` (the same
/// helper `preg_match` uses), so the caller's variable is defined and readable after the call. The
/// capture helper records the first match and its capture groups; full `preg_match_all` semantics
/// (nested per-match arrays) require a dedicated runtime helper and are not yet implemented —
/// `$flags` and `$offset` are accepted so calls type-check and lower but behave as the defaults.
pub(crate) fn lower_preg_match_all(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_match_all", 2, 5)?;
    let pattern = super::expect_operand(inst, 0)?;
    let subject = super::expect_operand(inst, 1)?;
    let matches_target = inst
        .operands
        .get(2)
        .copied()
        .map(|value| matches_target(ctx, value))
        .transpose()?;
    load_pattern_and_subject(ctx, pattern, subject)?;
    if let Some(target) = &matches_target {
        let allow_hash = target_allows_named_hash(target);
        let flag_reg = match ctx.emitter.target.arch {
            Arch::AArch64 => "x5",
            Arch::X86_64 => "r8",
        };
        abi::emit_load_int_immediate(ctx.emitter, flag_reg, allow_hash as i64);
        abi::emit_call_label(ctx.emitter, "__rt_preg_match_capture");
        store_matches_array(ctx, target)?;
    } else {
        abi::emit_call_label(ctx.emitter, "__rt_preg_match_all");
    }
    super::store_if_result(ctx, inst)
}

/// Lowers `preg_replace(pattern, replacement, subject, limit?, &count?)`.
///
/// The optional `$count` out-parameter is populated with the number of
/// replacements performed, computed via `__rt_preg_match_all` over the same
/// pattern/subject before the replacement runs (the unlimited `limit = -1` case,
/// which matches every supported call). The optional `$limit` argument is
/// accepted but not yet enforced; replacement always processes every match.
pub(crate) fn lower_preg_replace(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_replace", 3, 5)?;
    let pattern = super::expect_operand(inst, 0)?;
    let replacement = super::expect_operand(inst, 1)?;
    let subject = super::expect_operand(inst, 2)?;
    if let Some(count_value) = inst.operands.get(4).copied() {
        // Populate `$count` before the replacement so the regex result registers
        // are not clobbered by the match-counting call.
        let count_target = matches_target(ctx, count_value)?;
        load_pattern_and_subject(ctx, pattern, subject)?;
        abi::emit_call_label(ctx.emitter, "__rt_preg_match_all");
        store_replacement_count(ctx, &count_target)?;
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_arg(ctx, pattern, "x1", "x2", "preg_replace pattern")?;
            load_string_arg(ctx, replacement, "x3", "x4", "preg_replace replacement")?;
            load_string_arg(ctx, subject, "x5", "x6", "preg_replace subject")?;
        }
        Arch::X86_64 => {
            load_string_arg(ctx, pattern, "rdi", "rsi", "preg_replace pattern")?;
            load_string_arg(ctx, replacement, "rdx", "rcx", "preg_replace replacement")?;
            load_string_arg(ctx, subject, "r8", "r9", "preg_replace subject")?;
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_preg_replace");
    super::store_if_result(ctx, inst)
}

/// Lowers `preg_replace_callback(pattern, callback, subject)` through supported direct callbacks.
pub(crate) fn lower_preg_replace_callback(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_replace_callback", 3, 6)?;
    let pattern = super::expect_operand(inst, 0)?;
    let callback = super::expect_operand(inst, 1)?;
    let subject = super::expect_operand(inst, 2)?;
    let callback_target = preg_replace_callback_target(ctx, callback)?;
    let env_bytes = callback_target.reserve_env(ctx)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_arg(ctx, pattern, "x1", "x2", "preg_replace_callback pattern")?;
            abi::emit_symbol_address(ctx.emitter, "x3", &callback_target.entry_label);
            load_static_callback_env_arg(ctx, "x4", env_bytes);
            load_string_arg(ctx, subject, "x5", "x6", "preg_replace_callback subject")?;
        }
        Arch::X86_64 => {
            load_string_arg(ctx, pattern, "rdi", "rsi", "preg_replace_callback pattern")?;
            abi::emit_symbol_address(ctx.emitter, "rdx", &callback_target.entry_label);
            load_static_callback_env_arg(ctx, "rcx", env_bytes);
            load_string_arg(ctx, subject, "r8", "r9", "preg_replace_callback subject")?;
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_preg_replace_callback");
    callback_target.release_env(ctx, env_bytes);
    super::store_if_result(ctx, inst)
}

/// Runtime callback target passed to `__rt_preg_replace_callback`.
struct PregReplaceCallbackTarget {
    entry_label: String,
    env: PregReplaceCallbackEnv,
}

impl PregReplaceCallbackTarget {
    /// Reserves any callback environment required by the regex callback runtime.
    fn reserve_env(&self, ctx: &mut FunctionContext<'_>) -> Result<usize> {
        self.env.reserve(ctx)
    }

    /// Releases any reserved callback environment while preserving the regex result.
    fn release_env(&self, ctx: &mut FunctionContext<'_>, env_bytes: usize) {
        self.env.release(ctx, env_bytes);
    }
}

/// Descriptor environment source used by the regex callback wrapper.
enum PregReplaceCallbackEnv {
    None,
    Descriptor(ValueId),
    RuntimeString(ValueId),
    CallableArray {
        callable: ValueId,
        instance_only: bool,
    },
}

impl PregReplaceCallbackEnv {
    /// Reserves the stack environment expected by the deferred regex callback wrapper.
    fn reserve(&self, ctx: &mut FunctionContext<'_>) -> Result<usize> {
        match self {
            Self::None => Ok(0),
            Self::Descriptor(callback) => reserve_descriptor_callback_env(ctx, *callback),
            Self::RuntimeString(callback) => {
                reserve_runtime_string_descriptor_callback_env(ctx, *callback)
            }
            Self::CallableArray {
                callable,
                instance_only,
                ..
            } => reserve_callable_array_descriptor_callback_env(ctx, *callable, *instance_only),
        }
    }

    /// Releases a descriptor environment only when this target owns the descriptor.
    fn release(&self, ctx: &mut FunctionContext<'_>, env_bytes: usize) {
        if env_bytes == 0 {
            return;
        }
        if self.releases_descriptor() {
            release_descriptor_callback_env_preserving_result(ctx, env_bytes);
        } else {
            abi::emit_release_temporary_stack(ctx.emitter, env_bytes);
        }
    }

    /// Returns true when the environment owns a descriptor pointer that must be released.
    fn releases_descriptor(&self) -> bool {
        matches!(self, Self::RuntimeString(_) | Self::CallableArray { .. })
    }
}

/// Resolves a regex replacement callback to a runtime callback entry and optional environment.
fn preg_replace_callback_target(
    ctx: &mut FunctionContext<'_>,
    callback: ValueId,
) -> Result<PregReplaceCallbackTarget> {
    if let Some(entry_label) = static_string_callback_entry(ctx, callback)? {
        return Ok(PregReplaceCallbackTarget {
            entry_label,
            env: PregReplaceCallbackEnv::None,
        });
    }
    let callback_ty = ctx.raw_value_php_type(callback)?;
    let callback_codegen_ty = callback_ty.codegen_repr();
    match callback_codegen_ty {
        PhpType::Str => {
            return Ok(PregReplaceCallbackTarget {
                entry_label: emit_descriptor_callback_wrapper(ctx),
                env: PregReplaceCallbackEnv::RuntimeString(callback),
            });
        }
        PhpType::Callable => {
            return Ok(PregReplaceCallbackTarget {
                entry_label: emit_descriptor_callback_wrapper(ctx),
                env: PregReplaceCallbackEnv::Descriptor(callback),
            });
        }
        PhpType::Array(elem) if elem.codegen_repr() == PhpType::Mixed => {
            return Ok(PregReplaceCallbackTarget {
                entry_label: emit_descriptor_callback_wrapper(ctx),
                env: PregReplaceCallbackEnv::CallableArray {
                    callable: callback,
                    instance_only: true,
                },
            });
        }
        PhpType::Array(elem) if elem.codegen_repr() == PhpType::Str => {
            return Ok(PregReplaceCallbackTarget {
                entry_label: emit_descriptor_callback_wrapper(ctx),
                env: PregReplaceCallbackEnv::CallableArray {
                    callable: callback,
                    instance_only: false,
                },
            });
        }
        _ => {}
    }
    let value_ref = ctx
        .function
        .value(callback)
        .ok_or_else(|| CodegenIrError::missing_entry("value", callback.as_raw()))?;
    let source_op = value_source_instruction(ctx, callback)?
        .map(|inst| format!("{:?}", inst.op))
        .unwrap_or_else(|| "non-instruction".to_string());
    Err(CodegenIrError::unsupported(format!(
        "preg_replace_callback callback with unsupported EIR type {:?} (raw {:?}, ir {:?}, source {})",
        ctx.value_php_type(callback)?,
        callback_ty,
        value_ref.ir_type,
        source_op
    )))
}

/// Resolves a literal string callback to a module-local function entry.
fn static_string_callback_entry(
    ctx: &FunctionContext<'_>,
    callback: ValueId,
) -> Result<Option<String>> {
    let Some(callback_name) = maybe_const_string_operand(ctx, callback)? else {
        return Ok(None);
    };
    let Some(function_name) = ctx
        .callable_function_by_name(&callback_name)
        .map(|function| function.name.to_string())
    else {
        return Ok(None);
    };
    Ok(Some(function_symbol(&function_name)))
}

/// Emits a descriptor callback wrapper that adapts regex matches to callable descriptors.
fn emit_descriptor_callback_wrapper(ctx: &mut FunctionContext<'_>) -> String {
    let wrapper_label = ctx.next_label("preg_replace_descriptor_callback_wrapper");
    let done_label = ctx.next_label("preg_replace_descriptor_callback_after_wrapper");
    let wrapper = DeferredCallbackWrapper {
        label: wrapper_label.clone(),
        visible_arg_types: vec![preg_matches_type()],
        target_visible_arg_types: None,
        capture_types: Vec::new(),
        descriptor_prefix_types: Vec::new(),
        descriptor_return_type: Some(PhpType::Str),
    };
    abi::emit_jump(ctx.emitter, &done_label);
    crate::codegen::emit_callback_wrapper(ctx.emitter, &wrapper);
    ctx.emitter.label(&done_label);
    wrapper_label
}

/// Reserves a one-slot callback environment containing the callable descriptor.
fn reserve_descriptor_callback_env(
    ctx: &mut FunctionContext<'_>,
    callback: ValueId,
) -> Result<usize> {
    abi::emit_reserve_temporary_stack(ctx.emitter, 16);
    let callback_ty = ctx.load_value_to_result(callback)?;
    if callback_ty.codegen_repr() != PhpType::Callable {
        return Err(CodegenIrError::invalid_module(format!(
            "preg_replace_callback descriptor operand has PHP type {:?}",
            callback_ty
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp]");                            // store the runtime callable descriptor for the regex callback wrapper
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");                // store the runtime callable descriptor for the regex callback wrapper
        }
    }
    Ok(16)
}

/// Reserves a one-slot callback environment containing a runtime string descriptor.
fn reserve_runtime_string_descriptor_callback_env(
    ctx: &mut FunctionContext<'_>,
    callable: ValueId,
) -> Result<usize> {
    abi::emit_reserve_temporary_stack(ctx.emitter, 16);
    let descriptor_reg = abi::int_result_reg(ctx.emitter).to_string();
    callables::emit_runtime_string_descriptor_value(
        ctx,
        callable,
        &descriptor_reg,
        "preg_replace_callback",
    )?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("str {descriptor_reg}, [sp]")); // store the runtime string descriptor for the regex callback wrapper
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("mov QWORD PTR [rsp], {descriptor_reg}"));
            // store the runtime string descriptor for the regex callback wrapper
        }
    }
    Ok(16)
}

/// Reserves a one-slot callback environment containing a callable-array descriptor.
fn reserve_callable_array_descriptor_callback_env(
    ctx: &mut FunctionContext<'_>,
    callable: ValueId,
    instance_only: bool,
) -> Result<usize> {
    abi::emit_reserve_temporary_stack(ctx.emitter, 16);
    if instance_only {
        callables::emit_runtime_mixed_instance_callable_array_descriptor_value(
            ctx,
            callable,
            "preg_replace_callback",
        )?;
    } else {
        callables::emit_runtime_callable_array_descriptor_value(
            ctx,
            callable,
            "preg_replace_callback",
        )?;
    }
    let descriptor_reg = abi::int_result_reg(ctx.emitter);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("str {descriptor_reg}, [sp]")); // store the callable-array descriptor for the regex callback wrapper
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("mov QWORD PTR [rsp], {descriptor_reg}"));
            // store the callable-array descriptor for the regex callback wrapper
        }
    }
    Ok(16)
}

/// Releases an owned descriptor env while preserving the regex replacement string result.
fn release_descriptor_callback_env_preserving_result(
    ctx: &mut FunctionContext<'_>,
    env_bytes: usize,
) {
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg);
    abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_result_reg(ctx.emitter), 16);
    callable_descriptor::emit_release_current_descriptor(ctx.emitter);
    abi::emit_pop_reg_pair(ctx.emitter, ptr_reg, len_reg);
    abi::emit_release_temporary_stack(ctx.emitter, env_bytes);
}

/// Loads the optional callback environment argument expected by the regex runtime.
fn load_static_callback_env_arg(ctx: &mut FunctionContext<'_>, env_reg: &str, env_bytes: usize) {
    if env_bytes == 0 {
        abi::emit_load_int_immediate(ctx.emitter, env_reg, 0);
    } else {
        abi::emit_temporary_stack_address(ctx.emitter, env_reg, 0);
    }
}

/// Returns the matches array type passed to preg replacement callbacks.
fn preg_matches_type() -> PhpType {
    PhpType::Array(Box::new(PhpType::Str))
}

/// Lowers `preg_split(pattern, subject, limit?, flags?)` through the regex split helper.
pub(crate) fn lower_preg_split(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_split", 2, 4)?;
    let pattern = super::expect_operand(inst, 0)?;
    let subject = super::expect_operand(inst, 1)?;
    let limit = inst.operands.get(2).copied();
    let flags = inst.operands.get(3).copied();
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_arg(ctx, pattern, "x1", "x2", "preg_split pattern")?;
            load_string_arg(ctx, subject, "x3", "x4", "preg_split subject")?;
            load_limit_arg(ctx, limit, "x5")?;
            load_flags_arg(ctx, flags, "x6")?;
            ctx.emitter
                .instruction(&format!("orr x6, x6, #{}", PREG_SPLIT_FORCE_MIXED_RESULT));
            // force boxed-Mixed split slots for EIR result layout
        }
        Arch::X86_64 => {
            load_string_arg(ctx, pattern, "rdi", "rsi", "preg_split pattern")?;
            load_string_arg(ctx, subject, "rdx", "rcx", "preg_split subject")?;
            load_limit_arg(ctx, limit, "r8")?;
            load_flags_arg(ctx, flags, "r9")?;
            ctx.emitter
                .instruction(&format!("or r9, {}", PREG_SPLIT_FORCE_MIXED_RESULT));
            // force boxed-Mixed split slots for EIR result layout
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_preg_split");
    super::store_if_result(ctx, inst)
}

/// Loads pattern and subject string operands into the regex runtime ABI registers.
fn load_pattern_and_subject(
    ctx: &mut FunctionContext<'_>,
    pattern: ValueId,
    subject: ValueId,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_arg(ctx, pattern, "x1", "x2", "preg pattern")?;
            load_string_arg(ctx, subject, "x3", "x4", "preg subject")
        }
        Arch::X86_64 => {
            load_string_arg(ctx, pattern, "rdi", "rsi", "preg pattern")?;
            load_string_arg(ctx, subject, "rdx", "rcx", "preg subject")
        }
    }
}

/// Loads `mb_ereg_match()` pattern, subject, and optional options into runtime ABI registers.
fn load_mb_ereg_match_args(
    ctx: &mut FunctionContext<'_>,
    pattern: ValueId,
    subject: ValueId,
    options: Option<ValueId>,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_arg(ctx, pattern, "x1", "x2", "mb_ereg_match pattern")?;
            load_string_arg(ctx, subject, "x3", "x4", "mb_ereg_match subject")?;
            load_optional_string_arg(ctx, options, "x5", "x6", "mb_ereg_match options")
        }
        Arch::X86_64 => {
            load_string_arg(ctx, pattern, "rdi", "rsi", "mb_ereg_match pattern")?;
            load_string_arg(ctx, subject, "rdx", "rcx", "mb_ereg_match subject")?;
            load_optional_string_arg(ctx, options, "r8", "r9", "mb_ereg_match options")
        }
    }
}

/// Storage destination for a `preg_*` out-parameter such as `$matches` or `$count`.
///
/// `Local` writes the captured value straight into a plain local frame slot (the historical
/// path). `RefCell` writes it through a by-reference parameter cell into the caller's storage,
/// mirroring the writeback that user-defined `&$param` arguments perform.
enum MatchesTarget {
    /// A plain local slot holding the captured value directly.
    Local(LocalSlotId),
    /// A by-reference parameter slot holding a pointer to the caller's storage, plus the
    /// declared type of the value the cell points to.
    RefCell { slot: LocalSlotId, cell_ty: PhpType },
}

/// Resolves a `preg_*` out-parameter operand (`$matches`/`$count`) to its storage destination.
///
/// Accepts a plain `load_local` (a direct local slot) or a `load_ref_cell` / promoted ref-cell
/// local (a by-reference parameter). For the by-reference case it records the cell's declared
/// type so the value can be coerced and written through the cell into the caller's storage.
fn matches_target(ctx: &FunctionContext<'_>, value: ValueId) -> Result<MatchesTarget> {
    let value_ref = ctx
        .function
        .value(value)
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Err(CodegenIrError::unsupported(
            "preg_match matches argument that is not a local load",
        ));
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    if !matches!(inst_ref.op, Op::LoadLocal | Op::LoadRefCell) {
        return Err(CodegenIrError::unsupported(
            "preg_match matches argument that is not a local variable",
        ));
    }
    let Some(Immediate::LocalSlot(slot)) = inst_ref.immediate else {
        return Err(CodegenIrError::invalid_module(
            "preg_match matches load missing local slot",
        ));
    };
    if inst_ref.op == Op::LoadRefCell || ctx.local_stores_ref_cell_pointer(slot) {
        let cell_ty = ctx.local_php_type(slot)?;
        Ok(MatchesTarget::RefCell { slot, cell_ty })
    } else {
        Ok(MatchesTarget::Local(slot))
    }
}

/// Returns whether a `$matches` destination may receive a named-group associative hash.
///
/// Only a by-reference cell whose declared type boxes to a Mixed cell (`Mixed`/`Union`, e.g.
/// PHP `?array`) can observe string-keyed entries through `$m['name']`, and only that path
/// boxes the runtime result kind-aware (indexed vs hash). A plain indexed local keeps its
/// contiguous layout so numeric `$m[0]` reads stay a direct indexed load.
fn target_allows_named_hash(target: &MatchesTarget) -> bool {
    match target {
        MatchesTarget::RefCell { cell_ty, .. } => {
            matches!(cell_ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
        }
        MatchesTarget::Local(_) => false,
    }
}

/// Stores the `preg_replace()` replacement count (in the int result register) into its destination.
fn store_replacement_count(ctx: &mut FunctionContext<'_>, target: &MatchesTarget) -> Result<()> {
    match target {
        MatchesTarget::Local(slot) => {
            let offset = ctx.local_offset(*slot)?;
            abi::store_at_offset(ctx.emitter, abi::int_result_reg(ctx.emitter), offset);
            Ok(())
        }
        MatchesTarget::RefCell { slot, cell_ty } => {
            store_count_through_ref_cell(ctx, *slot, cell_ty)
        }
    }
}

/// Stores the runtime-built matches array into its destination without clobbering the match flag.
fn store_matches_array(ctx: &mut FunctionContext<'_>, target: &MatchesTarget) -> Result<()> {
    match target {
        MatchesTarget::Local(slot) => store_matches_array_local(ctx, *slot),
        MatchesTarget::RefCell { slot, cell_ty } => {
            store_matches_array_through_ref_cell(ctx, *slot, cell_ty)
        }
    }
}

/// Stores the runtime-built matches array (ptr in x1/rdx) into a plain local slot.
fn store_matches_array_local(ctx: &mut FunctionContext<'_>, slot: LocalSlotId) -> Result<()> {
    let offset = ctx.local_offset(slot)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::store_at_offset(ctx.emitter, "x1", offset);
        }
        Arch::X86_64 => {
            abi::store_at_offset(ctx.emitter, "rdx", offset);
        }
    }
    Ok(())
}

/// Writes the runtime-built `$matches` array through a by-reference parameter cell.
///
/// Mirrors a user-defined `&$param` writeback: the previous value held in the caller's storage
/// is released first, then the freshly owned matches array (built by `__rt_preg_match_capture`,
/// ptr in x1/rdx) is written through the cell into the caller's variable. When the cell's
/// declared type is `Mixed`, the array is boxed into a mixed cell (which retains the array), so
/// the original array reference is released afterwards, leaving the cell the sole owner. The
/// match flag in the int result register is preserved across every helper call.
fn store_matches_array_through_ref_cell(
    ctx: &mut FunctionContext<'_>,
    slot: LocalSlotId,
    cell_ty: &PhpType,
) -> Result<()> {
    let cell_repr = cell_ty.codegen_repr();
    if !matches!(cell_repr, PhpType::Array(_) | PhpType::Mixed | PhpType::Union(_)) {
        return Err(CodegenIrError::unsupported(format!(
            "preg_match $matches by-reference parameter of PHP type {:?}",
            cell_repr
        )));
    }
    let matches_reg = match ctx.emitter.target.arch {
        Arch::AArch64 => "x1",
        Arch::X86_64 => "rdx",
    };
    let int_reg = abi::int_result_reg(ctx.emitter);
    let offset = ctx.local_offset(slot)?;
    let boxes_to_mixed = matches!(cell_repr, PhpType::Mixed | PhpType::Union(_));

    // -- preserve the match flag and the owned matches array across the writeback helpers --
    abi::emit_reserve_temporary_stack(ctx.emitter, 32);                         // scratch frame: flag, owned array ptr, store value
    abi::emit_store_to_sp(ctx.emitter, int_reg, 0);                             // save the preg_match flag for the final result store
    abi::emit_store_to_sp(ctx.emitter, matches_reg, 8);                         // save the freshly owned matches array pointer

    // -- release the previous value held in the caller's storage through the cell --
    let pointer_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::load_at_offset(ctx.emitter, pointer_reg, offset);                      // load the by-reference cell pointer into the caller's storage
    abi::emit_load_from_address(ctx.emitter, int_reg, pointer_reg, 0);          // load the previous value the caller's variable held
    abi::emit_decref_if_refcounted(ctx.emitter, &cell_repr);                    // release the previous value (the runtime helper ignores null/non-heap)

    // -- materialize the value to write through the cell from the owned matches array --
    abi::emit_load_temporary_stack_slot(ctx.emitter, int_reg, 8);              // reload the owned matches array pointer into the value register
    if boxes_to_mixed {
        abi::emit_call_label(ctx.emitter, "__rt_mixed_from_array_kind");       // box kind-aware (tag 4 indexed / 5 hash) so `$m['name']` reads work; retains the child
        abi::emit_store_to_sp(ctx.emitter, int_reg, 16);                       // save the boxed mixed-cell pointer to store through the cell
        abi::emit_load_temporary_stack_slot(ctx.emitter, int_reg, 8);          // reload the original owned array pointer for release
        abi::emit_call_label(ctx.emitter, "__rt_decref_any");                 // drop the original array/hash reference kind-aware; the mixed cell now owns it
        abi::emit_load_temporary_stack_slot(ctx.emitter, int_reg, 16);         // reload the boxed mixed-cell pointer to write through the cell
    }

    // -- write the new value through the cell into the caller's variable --
    let pointer_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::load_at_offset(ctx.emitter, pointer_reg, offset);                      // reload the by-reference cell pointer after the helper calls
    abi::emit_store_to_address(ctx.emitter, int_reg, pointer_reg, 0);           // write the matches value into the caller's storage

    // -- restore the match flag for the result store and release the scratch frame --
    abi::emit_load_temporary_stack_slot(ctx.emitter, int_reg, 0);             // restore the preg_match flag for `store_if_result`
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    Ok(())
}

/// Writes the `preg_replace()` replacement count through a by-reference parameter cell.
///
/// The count is a plain integer in the int result register. When the cell stores an integer
/// (or bool) it is written directly; when the cell's declared type is `Mixed` the count is
/// boxed and the previous value released, mirroring `store_matches_array_through_ref_cell`.
/// Unlike the matches store there is no second live result to preserve at this point.
fn store_count_through_ref_cell(
    ctx: &mut FunctionContext<'_>,
    slot: LocalSlotId,
    cell_ty: &PhpType,
) -> Result<()> {
    let cell_repr = cell_ty.codegen_repr();
    let int_reg = abi::int_result_reg(ctx.emitter);
    let offset = ctx.local_offset(slot)?;
    match cell_repr {
        PhpType::Int | PhpType::Bool => {
            let pointer_reg = abi::symbol_scratch_reg(ctx.emitter);
            abi::load_at_offset(ctx.emitter, pointer_reg, offset);             // load the by-reference cell pointer into the caller's storage
            abi::emit_store_to_address(ctx.emitter, int_reg, pointer_reg, 0);  // write the replacement count into the caller's variable
            Ok(())
        }
        PhpType::Mixed | PhpType::Union(_) => {
            abi::emit_reserve_temporary_stack(ctx.emitter, 16);                // scratch frame: boxed count value
            emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Int);      // box the integer count into a mixed cell
            abi::emit_store_to_sp(ctx.emitter, int_reg, 0);                   // save the boxed count to store after releasing the old value
            let pointer_reg = abi::symbol_scratch_reg(ctx.emitter);
            abi::load_at_offset(ctx.emitter, pointer_reg, offset);            // load the by-reference cell pointer into the caller's storage
            abi::emit_load_from_address(ctx.emitter, int_reg, pointer_reg, 0); // load the previous value the caller's variable held
            abi::emit_decref_if_refcounted(ctx.emitter, &cell_repr);          // release the previous value (the runtime helper ignores null/non-heap)
            abi::emit_load_temporary_stack_slot(ctx.emitter, int_reg, 0);     // reload the boxed count pointer to write through the cell
            let pointer_reg = abi::symbol_scratch_reg(ctx.emitter);
            abi::load_at_offset(ctx.emitter, pointer_reg, offset);            // reload the by-reference cell pointer after the helper calls
            abi::emit_store_to_address(ctx.emitter, int_reg, pointer_reg, 0); // write the boxed count into the caller's storage
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            Ok(())
        }
        other => Err(CodegenIrError::unsupported(format!(
            "preg_replace $count by-reference parameter of PHP type {:?}",
            other
        ))),
    }
}

/// Returns a string literal value when `value` is defined by a `ConstStr` instruction.
fn maybe_const_string_operand(ctx: &FunctionContext<'_>, value: ValueId) -> Result<Option<String>> {
    let Some(inst_ref) = value_source_instruction(ctx, value)? else {
        return Ok(None);
    };
    if inst_ref.op != Op::ConstStr {
        return Ok(None);
    }
    let Some(Immediate::Data(data)) = inst_ref.immediate else {
        return Err(CodegenIrError::invalid_module(
            "preg_replace_callback callback string literal has no data id",
        ));
    };
    ctx.module
        .data
        .strings
        .get(data.as_raw() as usize)
        .cloned()
        .map(Some)
        .ok_or_else(|| CodegenIrError::missing_entry("data string", data.as_raw()))
}

/// Returns the instruction that defines an SSA value, when it has one.
fn value_source_instruction<'a>(
    ctx: &'a FunctionContext<'_>,
    value: ValueId,
) -> Result<Option<&'a Instruction>> {
    let value_ref = ctx
        .function
        .value(value)
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Ok(None);
    };
    ctx.function
        .instruction(inst)
        .map(Some)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))
}

/// Loads a string operand into an explicit pointer/length register pair.
fn load_string_arg(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    ptr_reg: &str,
    len_reg: &str,
    context: &str,
) -> Result<()> {
    let ty = ctx.value_php_type(value)?;
    if ty != PhpType::Str {
        // PHP coerces a regex string operand to string, but the EIR regex bridge
        // does not yet coerce a non-string (e.g. Mixed) operand. Emit a runtime
        // fatal rather than miscompiling. Runtime-dead for the YAML probe: the
        // only Mixed operand is the block-scalar `$modifiers` subject on a path
        // the probe's simple mapping never reaches.
        let message = format!(
            "Fatal error: {} with PHP type {} is not yet supported by the elephc EIR backend\n",
            context, ty
        );
        super::super::emit_unsupported_feature_fatal(ctx, &message);
        return Ok(());
    }
    ctx.load_string_value_to_regs(value, ptr_reg, len_reg)
}

/// Loads an optional string operand, using a null pointer and zero length when absent or null.
fn load_optional_string_arg(
    ctx: &mut FunctionContext<'_>,
    value: Option<ValueId>,
    ptr_reg: &str,
    len_reg: &str,
    context: &str,
) -> Result<()> {
    let Some(value) = value else {
        abi::emit_load_int_immediate(ctx.emitter, ptr_reg, 0);
        abi::emit_load_int_immediate(ctx.emitter, len_reg, 0);
        return Ok(());
    };
    let ty = ctx.value_php_type(value)?;
    if matches!(ty, PhpType::Void | PhpType::Never) {
        abi::emit_load_int_immediate(ctx.emitter, ptr_reg, 0);
        abi::emit_load_int_immediate(ctx.emitter, len_reg, 0);
        return Ok(());
    }
    require_string(ty, context)?;
    ctx.load_string_value_to_regs(value, ptr_reg, len_reg)
}

/// Loads the optional `preg_split()` limit, using PHP's default `-1`.
fn load_limit_arg(ctx: &mut FunctionContext<'_>, limit: Option<ValueId>, reg: &str) -> Result<()> {
    let Some(limit) = limit else {
        abi::emit_load_int_immediate(ctx.emitter, reg, -1);
        return Ok(());
    };
    require_integer_like(ctx.load_value_to_reg(limit, reg)?, "preg_split limit")
}

/// Loads the optional `preg_split()` flags, using PHP's default `0`.
fn load_flags_arg(ctx: &mut FunctionContext<'_>, flags: Option<ValueId>, reg: &str) -> Result<()> {
    let Some(flags) = flags else {
        abi::emit_load_int_immediate(ctx.emitter, reg, 0);
        return Ok(());
    };
    require_integer_like(ctx.load_value_to_reg(flags, reg)?, "preg_split flags")
}

/// Verifies that a regex string operand is statically string-shaped.
fn require_string(ty: PhpType, context: &str) -> Result<()> {
    if ty == PhpType::Str {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "{} for PHP type {:?}",
        context, ty
    )))
}

/// Verifies that a regex integer option is statically integer-like.
fn require_integer_like(ty: PhpType, context: &str) -> Result<()> {
    if matches!(ty, PhpType::Int | PhpType::Bool) {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "{} for PHP type {:?}",
        context, ty
    )))
}
