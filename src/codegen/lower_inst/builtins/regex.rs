//! Purpose:
//! Lowers simple PCRE-style regex builtins for the EIR backend.
//! Bridges already-evaluated EIR operands to the shared target-aware regex runtime helpers.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::lower_language_construct_call()`.
//!
//! Key details:
//! - Regex output variables are written through the local's active raw/ref-cell representation.
//! - `preg_replace_callback()` supports static string callbacks and descriptor-backed
//!   callable values through a regex-specific callback wrapper.
//! - `preg_split()` forces boxed Mixed element slots so dynamic flags cannot mismatch layout.

use crate::codegen::platform::Arch;
use crate::codegen::{abi, callable_descriptor};
use crate::codegen::{CodegenIrError, Result};
use crate::codegen_support::DeferredCallbackWrapper;
use crate::ir::{Immediate, Instruction, LocalSlotId, Op, ValueDef, ValueId};
use crate::names::function_symbol;
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::super::callables;
use super::super::load_value_to_first_int_arg;

const PREG_SPLIT_FORCE_MIXED_RESULT: i64 = 1 << 30;

/// Lowers `preg_match(pattern, subject, matches, flags, offset)` through the regex runtime.
pub(crate) fn lower_preg_match(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_match", 2, 5)?;
    let pattern = super::expect_operand(inst, 0)?;
    let subject = super::expect_operand(inst, 1)?;
    let matches_slot = inst
        .operands
        .get(2)
        .copied()
        .map(|value| optional_local_slot_operand(ctx, value, "preg_match matches"))
        .transpose()?
        .flatten();
    load_match_args(
        ctx,
        pattern,
        subject,
        inst.operands.get(3).copied(),
        inst.operands.get(4).copied(),
    )?;
    if let Some(slot) = matches_slot {
        abi::emit_call_label(ctx.emitter, "__rt_preg_match_capture");
        store_matches_array(ctx, slot)?;
    } else {
        abi::emit_call_label(ctx.emitter, "__rt_preg_match");
    }
    super::store_if_result(ctx, inst)
}

/// Lowers `preg_grep(pattern, array, flags = 0)` through the key-preserving mixed filter runtime.
///
/// The synthetic predicate casts each boxed array value to PHP string form, invokes the shared
/// PCRE matcher, and reverses the predicate when `PREG_GREP_INVERT` is set. Both indexed and
/// associative inputs deliberately use the mixed filter so the returned array retains its
/// original keys and insertion order instead of being compacted like `array_filter()`'s packed
/// fast path.
pub(crate) fn lower_preg_grep(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_grep", 2, 3)?;
    let pattern = super::expect_operand(inst, 0)?;
    let source = super::expect_operand(inst, 1)?;
    let source_ty = ctx.value_php_type(source)?.codegen_repr();
    let (runtime_label, return_raw_hash) = match source_ty {
        PhpType::Array(_) | PhpType::AssocArray { .. } => {
            ("__rt_array_filter_mixed_raw", true)
        }
        PhpType::Mixed | PhpType::Union(_) => {
            emit_preg_grep_mixed_source_guard(ctx, source)?;
            ("__rt_array_filter_mixed", false)
        }
        ref other => {
            return Err(CodegenIrError::invalid_module(format!(
                "preg_grep array operand has non-array EIR type {:?}",
                other
            )));
        }
    };

    let predicate_label = emit_preg_grep_predicate_wrapper(ctx);
    abi::emit_reserve_temporary_stack(ctx.emitter, 32);
    let flags_reg = match ctx.emitter.target.arch {
        Arch::AArch64 => "x0",
        Arch::X86_64 => "rax",
    };
    load_integer_args(
        ctx,
        &[(inst.operands.get(2).copied(), 0, "preg_grep flags")],
        &[flags_reg],
        &[],
        &[],
    )?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("and x0, x0, #1");                          // retain only PHP's PREG_GREP_INVERT bit
            ctx.emitter.instruction("str x0, [sp, #16]");                       // store normalized inversion flag in predicate environment
            load_string_arg(ctx, pattern, "x1", "x2", "preg_grep pattern")?;
            ctx.emitter.instruction("stp x1, x2, [sp]");                        // store pattern pointer and length in predicate environment
            abi::emit_symbol_address(ctx.emitter, "x0", &predicate_label);
            ctx.load_value_to_reg(source, "x1")?;
            abi::emit_temporary_stack_address(ctx.emitter, "x2", 0);
            abi::emit_load_int_immediate(ctx.emitter, "x3", 0);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("and rax, 1");                              // retain only PHP's PREG_GREP_INVERT bit
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], rax");           // store normalized inversion flag in predicate environment
            load_string_arg(ctx, pattern, "rax", "rdx", "preg_grep pattern")?;
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");                // store pattern pointer in predicate environment
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rdx");            // store pattern length in predicate environment
            abi::emit_symbol_address(ctx.emitter, "rdi", &predicate_label);
            ctx.load_value_to_reg(source, "rsi")?;
            abi::emit_temporary_stack_address(ctx.emitter, "rdx", 0);
            abi::emit_load_int_immediate(ctx.emitter, "rcx", 0);
        }
    }
    abi::emit_call_label(ctx.emitter, runtime_label);
    if return_raw_hash {
        emit_take_preg_grep_raw_hash_result(ctx);
    }
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    super::store_if_result(ctx, inst)
}

/// Transfers the filtered hash out of the owned Mixed wrapper returned for a concrete source.
fn emit_take_preg_grep_raw_hash_result(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp]");                            // preserve the owned result box in the retired predicate environment
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            ctx.emitter.instruction("str x1, [sp, #8]");                        // preserve the borrowed filtered-hash payload
            ctx.emitter.instruction("mov x0, x1");                              // retain a raw owner before releasing the Mixed wrapper
            abi::emit_call_label(ctx.emitter, "__rt_incref");
            ctx.emitter.instruction("ldr x0, [sp]");                            // release the result box and its child ownership
            abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            ctx.emitter.instruction("ldr x0, [sp, #8]");                        // return the transferred raw hash owner
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");                // preserve the owned result box in the retired predicate environment
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rdi");            // preserve the borrowed filtered-hash payload
            ctx.emitter.instruction("mov rax, rdi");                            // retain a raw owner before releasing the Mixed wrapper
            abi::emit_call_label(ctx.emitter, "__rt_incref");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp]");                // release the result box and its child ownership
            abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 8]");            // return the transferred raw hash owner
        }
    }
}

/// Validates that a boxed gradual `preg_grep()` source currently holds an array.
fn emit_preg_grep_mixed_source_guard(
    ctx: &mut FunctionContext<'_>,
    source: ValueId,
) -> Result<()> {
    ctx.load_value_to_result(source)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let ok_label = ctx.next_label("preg_grep_mixed_array");
    let wrong_label = ctx.next_label("preg_grep_mixed_wrong_type");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #4");                              // runtime tag 4 = indexed array
            ctx.emitter.instruction(&format!("b.eq {}", ok_label));
            ctx.emitter.instruction("cmp x0, #5");                              // runtime tag 5 = associative array
            ctx.emitter.instruction(&format!("b.eq {}", ok_label));
            ctx.emitter.instruction(&format!("b {}", wrong_label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 4");                              // runtime tag 4 = indexed array
            ctx.emitter.instruction(&format!("je {}", ok_label));
            ctx.emitter.instruction("cmp rax, 5");                              // runtime tag 5 = associative array
            ctx.emitter.instruction(&format!("je {}", ok_label));
            ctx.emitter.instruction(&format!("jmp {}", wrong_label));
        }
    }
    super::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &wrong_label,
        &|given| {
            format!(
                "preg_grep(): Argument #2 ($array) must be of type array, {} given",
                given
            )
        },
    );
    ctx.emitter.label(&ok_label);
    Ok(())
}

/// Emits the target-specific mixed-value predicate used by `preg_grep()`.
///
/// The filter runtime owns and later releases the boxed value argument. This wrapper owns only
/// the temporary string allocated by `__rt_mixed_cast_string`; `__rt_heap_free` is intentionally
/// safe for the shared formatting scratch returned by non-string scalar casts.
fn emit_preg_grep_predicate_wrapper(ctx: &mut FunctionContext<'_>) -> String {
    let predicate_label = ctx.next_label("preg_grep_predicate");
    let continuation_label = ctx.next_label("preg_grep_after_predicate");
    abi::emit_jump(ctx.emitter, &continuation_label);
    ctx.emitter.label(&predicate_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("sub sp, sp, #64");                         // reserve aligned predicate spill slots
            ctx.emitter.instruction("stp x29, x30, [sp, #48]");                 // preserve caller frame state
            ctx.emitter.instruction("add x29, sp, #48");                        // establish predicate frame pointer
            ctx.emitter.instruction("str x1, [sp]");                            // preserve pattern environment across runtime calls
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_string");
            ctx.emitter.instruction("stp x1, x2, [sp, #16]");                   // preserve temporary subject pointer and length
            ctx.emitter.instruction("mov x3, x1");                              // pass subject pointer to regex runtime
            ctx.emitter.instruction("mov x4, x2");                              // pass subject length to regex runtime
            ctx.emitter.instruction("ldr x9, [sp]");                            // reload pattern environment
            ctx.emitter.instruction("ldp x1, x2, [x9]");                        // load pattern pointer and length
            abi::emit_call_label(ctx.emitter, "__rt_preg_match");
            ctx.emitter.instruction("ldr x9, [sp]");                            // reload predicate environment after regex call
            ctx.emitter.instruction("ldr x9, [x9, #16]");                       // load normalized inversion flag
            ctx.emitter.instruction("eor x0, x0, x9");                          // invert match result when requested
            ctx.emitter.instruction("str x0, [sp, #32]");                       // preserve predicate result across temporary release
            ctx.emitter.instruction("ldr x0, [sp, #16]");                       // load temporary cast-string allocation
            abi::emit_call_label(ctx.emitter, "__rt_heap_free");
            ctx.emitter.instruction("ldr x0, [sp, #32]");                       // restore predicate result
            ctx.emitter.instruction("ldp x29, x30, [sp, #48]");                 // restore caller frame state
            ctx.emitter.instruction("add sp, sp, #64");                         // release predicate frame
            ctx.emitter.instruction("ret");                                     // return boolean predicate in x0
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("push rbp");                                // preserve caller frame pointer
            ctx.emitter.instruction("mov rbp, rsp");                            // establish predicate frame pointer
            ctx.emitter.instruction("sub rsp, 48");                             // reserve aligned predicate spill slots
            ctx.emitter.instruction("mov QWORD PTR [rbp - 8], rsi");            // preserve pattern environment across runtime calls
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_string");
            ctx.emitter.instruction("mov QWORD PTR [rbp - 16], rax");           // preserve temporary subject pointer
            ctx.emitter.instruction("mov QWORD PTR [rbp - 24], rdx");           // preserve subject length
            ctx.emitter.instruction("mov r10, QWORD PTR [rbp - 8]");            // reload pattern environment
            ctx.emitter.instruction("mov rdi, QWORD PTR [r10]");                // pass pattern pointer to regex runtime
            ctx.emitter.instruction("mov rsi, QWORD PTR [r10 + 8]");            // pass pattern length to regex runtime
            ctx.emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");           // pass subject pointer to regex runtime
            ctx.emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");           // pass subject length to regex runtime
            abi::emit_call_label(ctx.emitter, "__rt_preg_match");
            ctx.emitter.instruction("mov r10, QWORD PTR [rbp - 8]");            // reload predicate environment after regex call
            ctx.emitter.instruction("xor rax, QWORD PTR [r10 + 16]");           // invert match result when requested
            ctx.emitter.instruction("mov QWORD PTR [rbp - 32], rax");           // preserve predicate result across temporary release
            ctx.emitter.instruction("mov rax, QWORD PTR [rbp - 16]");           // load temporary cast-string allocation
            abi::emit_call_label(ctx.emitter, "__rt_heap_free");
            ctx.emitter.instruction("mov rax, QWORD PTR [rbp - 32]");           // restore predicate result
            ctx.emitter.instruction("add rsp, 48");                             // release predicate spill slots
            ctx.emitter.instruction("pop rbp");                                 // restore caller frame pointer
            ctx.emitter.instruction("ret");                                     // return boolean predicate in rax
        }
    }
    ctx.emitter.label(&continuation_label);
    predicate_label
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

/// Lowers `preg_match_all(pattern, subject, matches, flags, offset)` through the regex runtime.
pub(crate) fn lower_preg_match_all(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_match_all", 2, 5)?;
    let pattern = super::expect_operand(inst, 0)?;
    let subject = super::expect_operand(inst, 1)?;
    let matches_slot = inst
        .operands
        .get(2)
        .copied()
        .map(|value| optional_local_slot_operand(ctx, value, "preg_match_all matches"))
        .transpose()?
        .flatten();
    load_match_args(
        ctx,
        pattern,
        subject,
        inst.operands.get(3).copied(),
        inst.operands.get(4).copied(),
    )?;
    if matches_slot.is_some() {
        abi::emit_call_label(ctx.emitter, "__rt_preg_match_all_capture");
    } else {
        abi::emit_call_label(ctx.emitter, "__rt_preg_match_all");
    }
    if let Some(slot) = matches_slot {
        store_matches_array(ctx, slot)?;
    }
    super::store_if_result(ctx, inst)
}

/// Lowers `preg_replace(pattern, replacement, subject, limit, count)` through the regex helper.
pub(crate) fn lower_preg_replace(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_replace", 3, 5)?;
    let pattern = super::expect_operand(inst, 0)?;
    let replacement = super::expect_operand(inst, 1)?;
    let subject = super::expect_operand(inst, 2)?;
    let limit = inst.operands.get(3).copied();
    let count_slot = inst
        .operands
        .get(4)
        .copied()
        .map(|value| optional_local_slot_operand(ctx, value, "preg_replace count"))
        .transpose()?
        .flatten();
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_args(
                ctx,
                &[
                    (pattern, "preg_replace pattern"),
                    (replacement, "preg_replace replacement"),
                    (subject, "preg_replace subject"),
                ],
                &[("x1", "x2"), ("x3", "x4"), ("x5", "x6")],
            )?;
            load_integer_args(
                ctx,
                &[(limit, -1, "preg_replace limit")],
                &["x7"],
                &[("x1", "x2"), ("x3", "x4"), ("x5", "x6")],
                &[],
            )?;
        }
        Arch::X86_64 => {
            load_string_args(
                ctx,
                &[
                    (pattern, "preg_replace pattern"),
                    (replacement, "preg_replace replacement"),
                    (subject, "preg_replace subject"),
                ],
                &[("rdi", "rsi"), ("rdx", "rcx"), ("r8", "r9")],
            )?;
            load_integer_args(
                ctx,
                &[(limit, -1, "preg_replace limit")],
                &["r10"],
                &[("rdi", "rsi"), ("rdx", "rcx"), ("r8", "r9")],
                &[],
            )?;
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_preg_replace");
    if let Some(slot) = count_slot {
        store_preg_replace_count(ctx, slot)?;
    }
    super::store_if_result(ctx, inst)
}

/// Lowers callback replacement with its optional limit and by-reference counter.
pub(crate) fn lower_preg_replace_callback(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::ensure_arg_count_between(inst, "preg_replace_callback", 3, 5)?;
    let pattern = super::expect_operand(inst, 0)?;
    let callback = super::expect_operand(inst, 1)?;
    let subject = super::expect_operand(inst, 2)?;
    let limit = inst.operands.get(3).copied();
    let count_slot = inst
        .operands
        .get(4)
        .copied()
        .map(|value| optional_local_slot_operand(ctx, value, "preg_replace_callback count"))
        .transpose()?
        .flatten();
    let callback_target = preg_replace_callback_target(ctx, callback)?;
    let env_bytes = callback_target.reserve_env(
        ctx,
        super::super::instruction_strict_php_profile(inst),
    )?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_args(
                ctx,
                &[
                    (pattern, "preg_replace_callback pattern"),
                    (subject, "preg_replace_callback subject"),
                ],
                &[("x1", "x2"), ("x5", "x6")],
            )?;
            abi::emit_symbol_address(ctx.emitter, "x3", &callback_target.entry_label);
            load_static_callback_env_arg(ctx, "x4", env_bytes);
            load_integer_args(
                ctx,
                &[(limit, -1, "preg_replace_callback limit")],
                &["x7"],
                &[("x1", "x2"), ("x5", "x6")],
                &["x3", "x4"],
            )?;
        }
        Arch::X86_64 => {
            load_string_args(
                ctx,
                &[
                    (pattern, "preg_replace_callback pattern"),
                    (subject, "preg_replace_callback subject"),
                ],
                &[("rdi", "rsi"), ("r8", "r9")],
            )?;
            abi::emit_symbol_address(ctx.emitter, "rdx", &callback_target.entry_label);
            load_static_callback_env_arg(ctx, "rcx", env_bytes);
            load_integer_args(
                ctx,
                &[(limit, -1, "preg_replace_callback limit")],
                &["r10"],
                &[("rdi", "rsi"), ("r8", "r9")],
                &["rdx", "rcx"],
            )?;
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_preg_replace_callback");
    if let Some(slot) = count_slot {
        store_preg_replace_count(ctx, slot)?;
    }
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
    fn reserve_env(
        &self,
        ctx: &mut FunctionContext<'_>,
        strict_php: bool,
    ) -> Result<usize> {
        self.env.reserve(ctx, strict_php)
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
    fn reserve(&self, ctx: &mut FunctionContext<'_>, strict_php: bool) -> Result<usize> {
        match self {
            Self::None => Ok(0),
            Self::Descriptor(callback) => reserve_descriptor_callback_env(ctx, *callback),
            Self::RuntimeString(callback) => {
                reserve_runtime_string_descriptor_callback_env(ctx, *callback, strict_php)
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
    let wrapper_label = ctx.next_global_label("preg_replace_descriptor_callback_wrapper");
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
    strict_php: bool,
) -> Result<usize> {
    abi::emit_reserve_temporary_stack(ctx.emitter, 16);
    let descriptor_reg = abi::int_result_reg(ctx.emitter).to_string();
    callables::emit_runtime_string_descriptor_value(
        ctx,
        callable,
        &descriptor_reg,
        "preg_replace_callback",
        strict_php,
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
            load_string_args(
                ctx,
                &[
                    (pattern, "preg_split pattern"),
                    (subject, "preg_split subject"),
                ],
                &[("x1", "x2"), ("x3", "x4")],
            )?;
            load_integer_args(
                ctx,
                &[
                    (limit, -1, "preg_split limit"),
                    (flags, 0, "preg_split flags"),
                ],
                &["x5", "x6"],
                &[("x1", "x2"), ("x3", "x4")],
                &[],
            )?;
            ctx.emitter
                .instruction(&format!("orr x6, x6, #{}", PREG_SPLIT_FORCE_MIXED_RESULT));
            // force boxed-Mixed split slots for EIR result layout
        }
        Arch::X86_64 => {
            load_string_args(
                ctx,
                &[
                    (pattern, "preg_split pattern"),
                    (subject, "preg_split subject"),
                ],
                &[("rdi", "rsi"), ("rdx", "rcx")],
            )?;
            load_integer_args(
                ctx,
                &[
                    (limit, -1, "preg_split limit"),
                    (flags, 0, "preg_split flags"),
                ],
                &["r8", "r9"],
                &[("rdi", "rsi"), ("rdx", "rcx")],
                &[],
            )?;
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
        Arch::AArch64 => load_string_args(
            ctx,
            &[(pattern, "preg pattern"), (subject, "preg subject")],
            &[("x1", "x2"), ("x3", "x4")],
        ),
        Arch::X86_64 => load_string_args(
            ctx,
            &[(pattern, "preg pattern"), (subject, "preg subject")],
            &[("rdi", "rsi"), ("rdx", "rcx")],
        ),
    }
}

/// Loads the common regex inputs plus PHP flags and starting offset.
fn load_match_args(
    ctx: &mut FunctionContext<'_>,
    pattern: ValueId,
    subject: ValueId,
    flags: Option<ValueId>,
    offset: Option<ValueId>,
) -> Result<()> {
    load_pattern_and_subject(ctx, pattern, subject)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => load_integer_args(
            ctx,
            &[(flags, 0, "preg flags"), (offset, 0, "preg offset")],
            &["x5", "x6"],
            &[("x1", "x2"), ("x3", "x4")],
            &[],
        ),
        Arch::X86_64 => load_integer_args(
            ctx,
            &[(flags, 0, "preg flags"), (offset, 0, "preg offset")],
            &["r8", "r9"],
            &[("rdi", "rsi"), ("rdx", "rcx")],
            &[],
        ),
    }
}

/// Loads `mb_ereg_match()` pattern, subject, and optional options into runtime ABI registers.
fn load_mb_ereg_match_args(
    ctx: &mut FunctionContext<'_>,
    pattern: ValueId,
    subject: ValueId,
    options: Option<ValueId>,
) -> Result<()> {
    let mut args = vec![
        (pattern, "mb_ereg_match pattern"),
        (subject, "mb_ereg_match subject"),
    ];
    if let Some(options) = options {
        args.push((options, "mb_ereg_match options"));
    }
    match (ctx.emitter.target.arch, options.is_some()) {
        (Arch::AArch64, true) => load_string_args(
            ctx,
            &args,
            &[("x1", "x2"), ("x3", "x4"), ("x5", "x6")],
        ),
        (Arch::AArch64, false) => {
            load_string_args(ctx, &args, &[("x1", "x2"), ("x3", "x4")])?;
            abi::emit_load_int_immediate(ctx.emitter, "x5", 0);
            abi::emit_load_int_immediate(ctx.emitter, "x6", 0);
            Ok(())
        }
        (Arch::X86_64, true) => load_string_args(
            ctx,
            &args,
            &[("rdi", "rsi"), ("rdx", "rcx"), ("r8", "r9")],
        ),
        (Arch::X86_64, false) => {
            load_string_args(ctx, &args, &[("rdi", "rsi"), ("rdx", "rcx")])?;
            abi::emit_load_int_immediate(ctx.emitter, "r8", 0);
            abi::emit_load_int_immediate(ctx.emitter, "r9", 0);
            Ok(())
        }
    }
}

/// Returns an optional writable local slot, treating an omitted nullable default as no destination.
fn optional_local_slot_operand(
    ctx: &FunctionContext<'_>,
    value: ValueId,
    context: &str,
) -> Result<Option<LocalSlotId>> {
    if matches!(ctx.value_php_type(value)?, PhpType::Void | PhpType::Never) {
        return Ok(None);
    }
    let Some(inst_ref) = value_source_instruction(ctx, value)? else {
        return Err(CodegenIrError::unsupported(format!(
            "{} argument that is not a local load",
            context
        )));
    };
    if !matches!(inst_ref.op, Op::LoadLocal | Op::LoadRefCell) {
        return Err(CodegenIrError::unsupported(format!(
            "{} argument that is not a local variable",
            context
        )));
    }
    let Some(Immediate::LocalSlot(slot)) = inst_ref.immediate else {
        return Err(CodegenIrError::invalid_module(format!(
            "{} load missing local slot",
            context
        )));
    };
    Ok(Some(slot))
}

/// Stores the runtime-built string matches array without clobbering the match flag.
fn store_matches_array(ctx: &mut FunctionContext<'_>, slot: LocalSlotId) -> Result<()> {
    let target_ty = ctx.local_php_type(slot)?.codegen_repr();
    let box_result = match &target_ty {
        PhpType::Array(element_ty)
            if matches!(element_ty.codegen_repr(), PhpType::Str | PhpType::Mixed) =>
        {
            false
        }
        // A pattern that declares a capture NAME makes the runtime build hash storage, and the
        // checker types the destination for it. The pointer the runtime returns is already that
        // hash, so it is stored as-is -- the classification below exists for destinations whose
        // storage only the runtime knows.
        PhpType::AssocArray { .. } => false,
        PhpType::Mixed | PhpType::Union(_) => true,
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "preg_match matches destination PHP type {:?}",
                other
            )));
        }
    };
    let result_reg = abi::int_result_reg(ctx.emitter);
    let matches_reg = match ctx.emitter.target.arch {
        Arch::AArch64 => "x1",
        Arch::X86_64 => "rdx",
    };
    abi::emit_reserve_temporary_stack(ctx.emitter, 16);
    abi::emit_store_to_sp(ctx.emitter, result_reg, 0);
    abi::emit_store_to_sp(ctx.emitter, matches_reg, 8);
    ctx.release_local_before_refcounted_writeback(slot)?;
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, 8);
    if box_result {
        // A pattern that declares a capture name makes the runtime return hash storage rather
        // than a list, and only the runtime knows which it built. Stamping the box with the
        // indexed tag reads that hash back as a dense list: the values survive, the string keys
        // do not, and every read silently renumbers. Classify the storage instead.
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction("str x0, [sp, #-16]!");                 // hold the matches array across the storage classification
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("sub rsp, 16");                         // reserve one temporary slot for the matches array pointer
                ctx.emitter.instruction("mov QWORD PTR [rsp], rax");            // hold the matches array across the storage classification
            }
        }
        abi::emit_call_label(ctx.emitter, "__rt_heap_kind");
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction("cmp x0, #3");                          // is the matches array hash storage?
                ctx.emitter.instruction("mov x0, #4");                          // runtime value tag 4 = indexed array
                ctx.emitter.instruction("mov x9, #5");                          // runtime value tag 5 = associative array
                ctx.emitter.instruction("csel x0, x9, x0, eq");                 // box a hash under its own tag, a list under the indexed one
                ctx.emitter.instruction("ldr x1, [sp], #16");                   // reload the matches array as the Mixed payload
                ctx.emitter.instruction("mov x2, xzr");                         // an array payload has no high word
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("cmp rax, 3");                          // is the matches array hash storage?
                ctx.emitter.instruction("mov eax, 4");                          // runtime value tag 4 = indexed array
                ctx.emitter.instruction("mov r8d, 5");                          // runtime value tag 5 = associative array
                ctx.emitter.instruction("cmove rax, r8");                       // box a hash under its own tag, a list under the indexed one
                ctx.emitter.instruction("mov rdi, QWORD PTR [rsp]");            // reload the matches array as the Mixed payload
                ctx.emitter.instruction("add rsp, 16");                         // release the classification temporary slot
                ctx.emitter.instruction("xor esi, esi");                        // an array payload has no high word
            }
        }
        abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
    }
    ctx.store_current_result_to_local(slot)?;
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, 0);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    Ok(())
}

/// Stores the replacement count returned beside a string result into its caller local.
fn store_preg_replace_count(ctx: &mut FunctionContext<'_>, slot: LocalSlotId) -> Result<()> {
    let target_ty = ctx.local_php_type(slot)?.codegen_repr();
    let (result_ptr_reg, result_len_reg) = abi::string_result_regs(ctx.emitter);
    let int_result_reg = abi::int_result_reg(ctx.emitter);
    let count_reg = match ctx.emitter.target.arch {
        Arch::AArch64 => "x3",
        Arch::X86_64 => "rcx",
    };

    abi::emit_reserve_temporary_stack(ctx.emitter, 32);
    abi::emit_store_to_sp(ctx.emitter, result_ptr_reg, 0);
    abi::emit_store_to_sp(ctx.emitter, result_len_reg, 8);
    abi::emit_store_to_sp(ctx.emitter, count_reg, 16);
    abi::emit_load_temporary_stack_slot(ctx.emitter, int_result_reg, 16);

    match target_ty {
        PhpType::Mixed => {
            abi::emit_push_reg(ctx.emitter, int_result_reg);
            ctx.release_local_before_refcounted_writeback(slot)?;
            abi::emit_pop_reg(ctx.emitter, int_result_reg);
            crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Int);
        }
        PhpType::TaggedScalar => {
            crate::codegen::sentinels::emit_tagged_scalar_from_int_result(ctx.emitter);
        }
        PhpType::Int => {}
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "regex replacement count destination with PHP type {:?}",
                other
            )));
        }
    }
    ctx.store_current_result_to_local(slot)?;

    abi::emit_load_temporary_stack_slot(ctx.emitter, result_ptr_reg, 0);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_len_reg, 8);
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    Ok(())
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
    super::strings::load_value_as_string_to_regs(ctx, value, context, ptr_reg, len_reg)
}

/// Materializes string-coercible operands without allowing later conversions to clobber earlier pairs.
fn load_string_args(
    ctx: &mut FunctionContext<'_>,
    args: &[(ValueId, &str)],
    destination_regs: &[(&str, &str)],
) -> Result<()> {
    if args.len() != destination_regs.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "regex string argument count {} does not match destination count {}",
            args.len(),
            destination_regs.len()
        )));
    }
    let spill_bytes = args.len() * 16;
    abi::emit_reserve_temporary_stack(ctx.emitter, spill_bytes);
    let (result_ptr, result_len) = abi::string_result_regs(ctx.emitter);
    for (index, (value, context)) in args.iter().enumerate() {
        super::strings::load_value_as_string_to_regs(
            ctx,
            *value,
            context,
            result_ptr,
            result_len,
        )?;
        abi::emit_store_to_sp(ctx.emitter, result_ptr, index * 16);
        abi::emit_store_to_sp(ctx.emitter, result_len, index * 16 + 8);
    }
    for (index, (ptr_reg, len_reg)) in destination_regs.iter().enumerate() {
        abi::emit_load_temporary_stack_slot(ctx.emitter, ptr_reg, index * 16);
        abi::emit_load_temporary_stack_slot(ctx.emitter, len_reg, index * 16 + 8);
    }
    abi::emit_release_temporary_stack(ctx.emitter, spill_bytes);
    Ok(())
}

/// Materializes integer-coercible operands while preserving previously staged string arguments.
fn load_integer_args(
    ctx: &mut FunctionContext<'_>,
    args: &[(Option<ValueId>, i64, &str)],
    destination_regs: &[&str],
    preserved_string_regs: &[(&str, &str)],
    preserved_scalar_regs: &[&str],
) -> Result<()> {
    if args.len() != destination_regs.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "regex integer argument count {} does not match destination count {}",
            args.len(),
            destination_regs.len()
        )));
    }
    let scalar_base = preserved_string_regs.len() * 16;
    let integer_base = scalar_base + preserved_scalar_regs.len() * 16;
    let spill_bytes = integer_base + args.len() * 16;
    abi::emit_reserve_temporary_stack(ctx.emitter, spill_bytes);
    for (index, (ptr_reg, len_reg)) in preserved_string_regs.iter().enumerate() {
        abi::emit_store_to_sp(ctx.emitter, ptr_reg, index * 16);
        abi::emit_store_to_sp(ctx.emitter, len_reg, index * 16 + 8);
    }
    for (index, reg) in preserved_scalar_regs.iter().enumerate() {
        abi::emit_store_to_sp(ctx.emitter, reg, scalar_base + index * 16);
    }
    for (index, (value, default, context)) in args.iter().enumerate() {
        if let Some(value) = value {
            load_integer_arg_to_result(ctx, *value, context)?;
        } else {
            let result_reg = abi::int_result_reg(ctx.emitter);
            abi::emit_load_int_immediate(ctx.emitter, result_reg, *default);
        }
        let result_reg = abi::int_result_reg(ctx.emitter);
        abi::emit_store_to_sp(
            ctx.emitter,
            result_reg,
            integer_base + index * 16,
        );
    }
    for (index, (ptr_reg, len_reg)) in preserved_string_regs.iter().enumerate() {
        abi::emit_load_temporary_stack_slot(ctx.emitter, ptr_reg, index * 16);
        abi::emit_load_temporary_stack_slot(ctx.emitter, len_reg, index * 16 + 8);
    }
    for (index, reg) in preserved_scalar_regs.iter().enumerate() {
        abi::emit_load_temporary_stack_slot(ctx.emitter, reg, scalar_base + index * 16);
    }
    for (index, destination_reg) in destination_regs.iter().enumerate() {
        abi::emit_load_temporary_stack_slot(
            ctx.emitter,
            destination_reg,
            integer_base + index * 16,
        );
    }
    abi::emit_release_temporary_stack(ctx.emitter, spill_bytes);
    Ok(())
}

/// Resolves a regex integer operand into the canonical integer result register.
fn load_integer_arg_to_result(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    context: &str,
) -> Result<()> {
    match ctx.value_php_type(value)?.codegen_repr() {
        PhpType::Int | PhpType::Bool => {
            ctx.load_value_to_result(value)?;
        }
        PhpType::TaggedScalar => {
            ctx.load_value_to_result(value)?;
            crate::codegen::sentinels::emit_tagged_scalar_to_int_null_as_zero(ctx.emitter);
        }
        PhpType::Mixed | PhpType::Union(_) => {
            load_value_to_first_int_arg(ctx, value)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int");
        }
        PhpType::Void | PhpType::Never => {
            let result_reg = abi::int_result_reg(ctx.emitter);
            abi::emit_load_int_immediate(ctx.emitter, result_reg, 0);
        }
        PhpType::Float => {
            ctx.load_value_to_result(value)?;
            abi::emit_float_result_to_int_result(ctx.emitter);
        }
        PhpType::Str => {
            ctx.load_value_to_result(value)?;
            abi::emit_call_label(ctx.emitter, "__rt_str_to_int");
        }
        ty => {
            return Err(CodegenIrError::unsupported(format!(
                "{} for PHP type {:?}",
                context, ty
            )));
        }
    }
    Ok(())
}
