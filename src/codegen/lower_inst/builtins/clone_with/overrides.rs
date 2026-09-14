//! Purpose:
//! Lowers the PHP 8.5 `clone($object, $withProperties)` override application: after the shallow
//! copy and `__clone()`, the clone and the runtime override array are handed to the generated
//! applicator selected by the clone's RUNTIME class and the INVOCATION-SITE scope.
//!
//! Called from:
//! - `super::lower_clone_with()`, inside the window where the clone is an unwind-visible owner.
//!
//! Key details:
//! - No property semantics live here. `crate::ir_lower::clone_overrides` generated one ordinary
//!   PHP function per `(runtime class, scope profile)`, so iteration order, key stringification,
//!   NUL rejection, typed weak coercion, set hooks, `__set`, dynamic-property storage, readonly
//!   reinitialization and every refusal message come from the ordinary lowering pipeline.
//! - Dispatch is two-level. The clone's dense class id selects the class arm; inside it the
//!   transported scope id selects the exact body, falling through to the group that also serves
//!   global scope. A call site whose scope is statically known skips the second level entirely
//!   and names one symbol.
//! - The receiver travels in the callee-saved nested-call register, which is the contract
//!   `materialize_method_call_args_with_receiver_reg_and_refs` enforces, so materializing the
//!   Mixed override argument cannot destroy it.
//! - A runtime class with NO applicator, a reflection or otherwise excluded class reached
//!   through a runtime callable, REPORTS. It runs the same non-empty-array guard the refusal
//!   slice used and raises a catchable `Error`, so an override is never silently dropped.

use std::collections::BTreeMap;

use crate::codegen::abi;
use crate::codegen::context::FunctionContext;
use crate::codegen::platform::Arch;
use crate::codegen::Result;
use crate::ir::{CloneOverrideApplicator, ValueId};
use crate::types::PhpType;

use super::super::super::{
    direct_call_stack_pad_bytes, emit_call_arg_temp_cleanups, emit_ref_arg_writebacks,
    materialize_method_call_args_with_receiver_reg_and_refs, RefArgCellLifetime,
};

/// Applies `$withProperties` to the clone parked in the caller's temporary stack slot.
///
/// `clone_box_offset` is the offset of that boxed clone inside the caller's owner window.
pub(super) fn emit_property_overrides(
    ctx: &mut FunctionContext<'_>,
    properties: ValueId,
    invocation_scope: Option<ValueId>,
    clone_box_offset: usize,
) -> Result<()> {
    if super::value_is_empty_array_literal(ctx, properties)? {
        return Ok(());
    }
    let applicators = applicators_by_class(ctx);
    let done = ctx.next_label("clone_overrides_done");
    let miss = ctx.next_label("clone_overrides_unsupported");
    if applicators.is_empty() {
        // Nothing in this program can take overrides, so every reachable clone reports.
        super::emit_property_override_guard(ctx, properties)?;
        return Ok(());
    }

    let receiver_reg = abi::nested_call_reg(ctx.emitter).to_string();
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        clone_box_offset,
    );
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let payload_reg = crate::codegen_support::mixed_unbox_payload_reg(ctx.emitter.target);
    abi::emit_reg_move(ctx.emitter, &receiver_reg, payload_reg);

    let class_labels = applicators
        .keys()
        .map(|class_id| (*class_id, ctx.next_label(&format!("clone_overrides_{class_id}"))))
        .collect::<Vec<_>>();
    emit_class_id_dispatch(ctx, &receiver_reg, &class_labels, &miss);

    for ((class_id, label), entries) in class_labels.iter().zip(applicators.values()) {
        let _ = class_id;
        ctx.emitter.label(label);
        emit_class_arm(ctx, &receiver_reg, properties, invocation_scope, entries)?;
        abi::emit_jump(ctx.emitter, &done);
    }

    ctx.emitter.label(&miss);
    super::emit_property_override_guard(ctx, properties)?;
    ctx.emitter.label(&done);
    Ok(())
}

/// Groups the module's applicators by the runtime class they serve, in class-id order.
fn applicators_by_class(
    ctx: &FunctionContext<'_>,
) -> BTreeMap<u64, Vec<CloneOverrideApplicator>> {
    let mut grouped: BTreeMap<u64, Vec<CloneOverrideApplicator>> = BTreeMap::new();
    for applicator in &ctx.module.clone_override_applicators {
        if !ctx
            .module
            .functions
            .iter()
            .any(|function| function.name == applicator.function_name)
        {
            continue;
        }
        grouped
            .entry(applicator.class_id)
            .or_default()
            .push(applicator.clone());
    }
    grouped
}

/// Branches to the arm whose compile-time class id matches the clone's runtime class id.
fn emit_class_id_dispatch(
    ctx: &mut FunctionContext<'_>,
    receiver_reg: &str,
    class_labels: &[(u64, String)],
    miss: &str,
) {
    let (class_id_reg, compare_reg) = match ctx.emitter.target.arch {
        Arch::AArch64 => ("x9", "x10"),
        Arch::X86_64 => ("r11", "r10"),
    };
    abi::emit_load_from_address(ctx.emitter, class_id_reg, receiver_reg, 0);
    for (class_id, label) in class_labels {
        abi::emit_load_int_immediate(ctx.emitter, compare_reg, *class_id as i64);
        ctx.emitter
            .instruction(&format!("cmp {class_id_reg}, {compare_reg}"));
        match ctx.emitter.target.arch {
            Arch::AArch64 => ctx.emitter.instruction(&format!("b.eq {label}")),
            Arch::X86_64 => ctx.emitter.instruction(&format!("je {label}")),
        }
    }
    abi::emit_jump(ctx.emitter, miss);
}

/// Emits one runtime class's arm: pick the scope's body, then call it.
fn emit_class_arm(
    ctx: &mut FunctionContext<'_>,
    receiver_reg: &str,
    properties: ValueId,
    invocation_scope: Option<ValueId>,
    entries: &[CloneOverrideApplicator],
) -> Result<()> {
    let Some(default) = entries
        .iter()
        .find(|entry| entry.is_default_scope)
        .or_else(|| entries.first())
    else {
        return Ok(());
    };
    let Some(invocation_scope) = invocation_scope else {
        // A direct call site knows its own lexical class, so one symbol is the whole answer.
        let scope_id = ctx
            .function
            .lexical_class
            .as_deref()
            .and_then(|name| ctx.module.class_infos.get(name))
            .map(|info| info.class_id);
        let selected = scope_id
            .and_then(|scope_id| {
                entries
                    .iter()
                    .find(|entry| entry.scope_class_ids.contains(&scope_id))
            })
            .unwrap_or(default);
        let selected = selected.clone();
        return emit_applicator_call(ctx, receiver_reg, properties, &selected);
    };
    let scoped = entries
        .iter()
        .filter(|entry| !entry.scope_class_ids.is_empty())
        .cloned()
        .collect::<Vec<_>>();
    let default = default.clone();
    if scoped.is_empty() {
        return emit_applicator_call(ctx, receiver_reg, properties, &default);
    }
    let arm_done = ctx.next_label("clone_overrides_scope_done");
    let default_label = ctx.next_label("clone_overrides_scope_default");
    let scope_labels = scoped
        .iter()
        .map(|_| ctx.next_label("clone_overrides_scope"))
        .collect::<Vec<_>>();
    ctx.load_value_to_result(invocation_scope)?;
    let scope_reg = abi::int_result_reg(ctx.emitter).to_string();
    for (entry, label) in scoped.iter().zip(scope_labels.iter()) {
        for scope_id in &entry.scope_class_ids {
            super::emit_branch_if_reg_equals_immediate(ctx, &scope_reg, *scope_id as i64, label);
        }
    }
    abi::emit_jump(ctx.emitter, &default_label);
    for (entry, label) in scoped.iter().zip(scope_labels.iter()) {
        ctx.emitter.label(label);
        emit_applicator_call(ctx, receiver_reg, properties, entry)?;
        abi::emit_jump(ctx.emitter, &arm_done);
    }
    ctx.emitter.label(&default_label);
    emit_applicator_call(ctx, receiver_reg, properties, &default)?;
    ctx.emitter.label(&arm_done);
    Ok(())
}

/// Calls one applicator with the clone as `$this` and the override array as the second argument.
fn emit_applicator_call(
    ctx: &mut FunctionContext<'_>,
    receiver_reg: &str,
    properties: ValueId,
    applicator: &CloneOverrideApplicator,
) -> Result<()> {
    let receiver_ty = PhpType::Object(applicator.class_name.clone());
    let params = [receiver_ty.clone(), PhpType::Mixed];
    let refs = [false, false];
    // The first operand is never read: the receiver comes from the register instead.
    let operands = [properties, properties];
    let call_args = materialize_method_call_args_with_receiver_reg_and_refs(
        ctx,
        receiver_reg,
        &receiver_ty,
        &operands,
        &params,
        &refs,
        RefArgCellLifetime::CallOnly,
    )?;
    let pad = direct_call_stack_pad_bytes(ctx, call_args.overflow_bytes);
    abi::emit_reserve_temporary_stack(ctx.emitter, pad);
    abi::emit_call_label(
        ctx.emitter,
        &crate::names::function_symbol(&applicator.function_name),
    );
    abi::emit_release_temporary_stack(ctx.emitter, pad);
    abi::emit_release_temporary_stack(ctx.emitter, call_args.overflow_bytes);
    emit_call_arg_temp_cleanups(ctx, &call_args, None)?;
    emit_ref_arg_writebacks(ctx, &call_args)
}
