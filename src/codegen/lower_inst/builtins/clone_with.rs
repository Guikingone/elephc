//! Purpose:
//! Lowers the typed PHP 8.5 `clone()` runtime operation.
//!
//! Called from:
//! - `crate::codegen::lower_inst::runtime_functions::group_02` for
//!   `RuntimeFnId::CloneWith`.
//!
//! Key details:
//! - Runtime-class cloning reuses the boxed shallow-copy adapter shared with Magician,
//!   which is emitted once per program by the managed runtime on every supported target.
//! - The adapter is a C-ABI wrapper, so the boxed input travels in the first ARGUMENT
//!   register and the boxed clone comes back in the result register.
//! - A non-object input is rejected by inspecting the boxed runtime tag BEFORE the clone,
//!   so a bad argument raises `TypeError` while a class the adapter cannot copy raises
//!   `Error`. Both are ordinary catchable throwables.
//! - The fresh clone is published as a temporary unwind owner before `__clone()` runs, so a
//!   throwing hook releases it instead of leaking it.
//! - Hook selection uses the runtime class id while visibility follows the INVOCATION SITE's
//!   lexical class, exactly as php-src does. A direct call takes that scope statically from the
//!   function being lowered. A body that a callable dispatch shared across call sites entered
//!   cannot: it reads the trailing hidden ABI operand and compares it against the statically
//!   computed set of scopes the hook is visible from, so one escaped `clone(...)` reports
//!   `global scope` and `scope X` at the two places it is invoked from.
//! - This slice REFUSES a non-empty `$withProperties`, at compile time when the operand is a
//!   literal `[]` and at run time otherwise. Overrides are never silently dropped. An omitted
//!   second operand needs no guard at all: it is the empty array by definition.

use crate::codegen::abi;
use crate::codegen::context::FunctionContext;
use crate::codegen::platform::Arch;
use crate::codegen::{emit_box_current_value_as_mixed, CodegenIrError, Result};
use crate::ir::{Immediate, Instruction, Op, ValueDef, ValueId};
use crate::names::php_symbol_key;
use crate::parser::ast::Visibility;
use crate::types::PhpType;

use super::super::{
    direct_call_stack_pad_bytes, emit_call_arg_temp_cleanups, emit_ref_arg_writebacks,
    emit_resolved_method_call, materialize_method_call_args_with_receiver_reg_and_refs,
    resolve_method_call_target, MethodCallTarget, RefArgCellLifetime,
};

const CLONE_OWNER_BYTES: usize = 16 + abi::CALL_OPERAND_OWNER_RECORD_BYTES;

/// Runtime class branch and the exact inherited clone-hook declaration it selects.
struct CloneHookCandidate {
    class_id: u64,
    class_name: String,
    declaring_class: String,
    visibility: Visibility,
    target: MethodCallTarget,
}

/// The message PHP-facing code sees when this slice meets property overrides it cannot apply.
const PROPERTY_OVERRIDE_MESSAGE: &str =
    "clone(): Argument #2 ($withProperties) property overrides are not supported yet";

/// Clones a runtime object and invokes its visible `__clone()` hook.
pub(crate) fn lower_clone_with(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if !(1..=3).contains(&inst.operands.len()) {
        return Err(CodegenIrError::invalid_module(format!(
            "clone expected 1 to 2 PHP args and an optional hidden invocation scope, got {} operands",
            inst.operands.len()
        )));
    }
    let object = super::expect_operand(inst, 0)?;
    let object_ty = ctx.value_php_type(object)?.codegen_repr();
    if !matches!(
        object_ty,
        PhpType::Object(_) | PhpType::Mixed | PhpType::Union(_)
    ) {
        // A statically non-object argument is a run-time `TypeError` in PHP, not a compile
        // failure, and a runtime callable can reach this lowering without the checker's
        // argument rule ever running. Throw the catchable error instead of refusing the build.
        super::super::exceptions::emit_type_error(
            ctx,
            &format!(
                "clone(): Argument #1 ($object) must be of type object, {} given",
                static_type_name(&object_ty)
            ),
        );
        return super::store_if_result(ctx, inst);
    }

    // An omitted second argument IS the empty override array, so there is nothing to refuse.
    if let Some(properties) = inst.operands.get(1).copied() {
        emit_property_override_guard(ctx, properties)?;
    }
    emit_boxed_shallow_clone(ctx, object)?;
    emit_uncloneable_guard(ctx);

    let clone_box = abi::int_result_reg(ctx.emitter).to_string();
    abi::emit_reserve_temporary_stack(ctx.emitter, CLONE_OWNER_BYTES);
    abi::emit_store_to_sp(ctx.emitter, &clone_box, 0);
    let owner_addr = abi::symbol_scratch_reg(ctx.emitter).to_string();
    abi::emit_temporary_stack_address(ctx.emitter, &owner_addr, 0);
    abi::emit_link_call_operand_owner_at_stack(ctx.emitter, &owner_addr, false, 16);

    emit_clone_hook(ctx, object, inst.operands.get(2).copied())?;

    abi::emit_unlink_call_operand_owner_at_stack(ctx.emitter, 16);
    abi::emit_load_temporary_stack_slot(ctx.emitter, &clone_box, 0);
    abi::emit_release_temporary_stack(ctx.emitter, CLONE_OWNER_BYTES);
    store_clone_result(ctx, inst)
}

/// Refuses property overrides this slice cannot apply, as early as the operand allows.
fn emit_property_override_guard(
    ctx: &mut FunctionContext<'_>,
    properties: ValueId,
) -> Result<()> {
    if value_is_empty_array_literal(ctx, properties)? {
        return Ok(());
    }
    let ty = ctx.value_php_type(properties)?.codegen_repr();
    let empty_label = ctx.next_label("clone_properties_empty");
    let result_reg = abi::int_result_reg(ctx.emitter).to_string();
    match ty {
        PhpType::Array(_) | PhpType::AssocArray { .. } => {
            ctx.load_value_to_result(properties)?;
            let scratch_reg = abi::secondary_scratch_reg(ctx.emitter).to_string();
            // A missed read forwarded as the null-container sentinel carries no overrides.
            crate::codegen::sentinels::emit_branch_if_null_container(
                ctx.emitter,
                &result_reg,
                &scratch_reg,
                &empty_label,
            );
            abi::emit_load_from_address(ctx.emitter, &result_reg, &result_reg, 0);
        }
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_result(properties)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_count");
        }
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "clone() property override operand of type {other:?}",
            )))
        }
    }
    emit_branch_if_zero(ctx, &result_reg, &empty_label);
    super::super::exceptions::emit_error(ctx, PROPERTY_OVERRIDE_MESSAGE);
    ctx.emitter.label(&empty_label);
    Ok(())
}

/// Produces a boxed clone while retiring a temporary box made for a concrete object input.
fn emit_boxed_shallow_clone(ctx: &mut FunctionContext<'_>, object: ValueId) -> Result<()> {
    let object_ty = ctx.value_php_type(object)?.codegen_repr();
    match object_ty {
        PhpType::Object(_) => {
            // A concrete object is statically known to satisfy the argument type, so the
            // only work here is handing the adapter a Mixed box and retiring it afterwards.
            ctx.load_value_to_result(object)?;
            emit_box_current_value_as_mixed(ctx.emitter, &object_ty);
            let input_box = abi::int_result_reg(ctx.emitter).to_string();
            let saved_clone = abi::nested_call_reg(ctx.emitter).to_string();
            abi::emit_reserve_temporary_stack(ctx.emitter, 16);
            abi::emit_store_to_sp(ctx.emitter, &input_box, 0);
            emit_clone_adapter_call(ctx);
            abi::emit_reg_move(ctx.emitter, &saved_clone, &input_box);
            abi::emit_load_temporary_stack_slot(ctx.emitter, &input_box, 0);
            abi::emit_decref_if_refcounted(ctx.emitter, &PhpType::Mixed);
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            abi::emit_reg_move(ctx.emitter, &input_box, &saved_clone);
            Ok(())
        }
        PhpType::Mixed | PhpType::Union(_) => {
            emit_mixed_object_type_guard(ctx, object)?;
            emit_clone_adapter_call(ctx);
            Ok(())
        }
        other => Err(CodegenIrError::invalid_module(format!(
            "clone() received non-object EIR operand {other:?}",
        ))),
    }
}

/// Names a statically known argument type the way PHP's `TypeError` wording does.
fn static_type_name(ty: &PhpType) -> &str {
    match ty {
        PhpType::Int => "int",
        PhpType::Float => "float",
        PhpType::Str => "string",
        PhpType::Bool | PhpType::False => "bool",
        PhpType::Void | PhpType::Never => "null",
        PhpType::Array(_) | PhpType::AssocArray { .. } => "array",
        PhpType::Object(name) | PhpType::Packed(name) => name,
        PhpType::Resource(_) | PhpType::Buffer(_) | PhpType::Pointer(_) => "resource",
        PhpType::Callable => "Closure",
        PhpType::Iterable => "object",
        PhpType::Mixed | PhpType::Union(_) | PhpType::TaggedScalar => "mixed",
    }
}

/// Rejects a runtime-shaped argument whose boxed tag is not an object, leaving the box loaded.
fn emit_mixed_object_type_guard(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
) -> Result<()> {
    let result_reg = abi::int_result_reg(ctx.emitter).to_string();
    let object_label = ctx.next_label("clone_argument_object");
    ctx.load_value_to_result(object)?;
    abi::emit_reserve_temporary_stack(ctx.emitter, 16);
    abi::emit_store_to_sp(ctx.emitter, &result_reg, 0);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    emit_branch_if_tag_is_object(ctx, &result_reg, &object_label);
    super::super::exceptions::emit_type_error(
        ctx,
        "clone(): Argument #1 ($object) must be of type object",
    );
    ctx.emitter.label(&object_label);
    abi::emit_load_temporary_stack_slot(ctx.emitter, &result_reg, 0);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    Ok(())
}

/// Calls the shared boxed shallow-clone adapter through its C argument register.
fn emit_clone_adapter_call(ctx: &mut FunctionContext<'_>) {
    let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    let result_reg = abi::int_result_reg(ctx.emitter).to_string();
    abi::emit_reg_move(ctx.emitter, arg_reg, &result_reg);
    let helper = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_value_object_clone_shallow");
    abi::emit_call_label(ctx.emitter, &helper);
}

/// Turns the adapter's null sentinel into PHP's catchable uncloneable-object `Error`.
fn emit_uncloneable_guard(ctx: &mut FunctionContext<'_>) {
    let clone_box = abi::int_result_reg(ctx.emitter).to_string();
    let cloned_label = ctx.next_label("clone_object_cloned");
    emit_branch_if_nonzero(ctx, &clone_box, &cloned_label);
    super::super::exceptions::emit_error(ctx, "Trying to clone an uncloneable object");
    ctx.emitter.label(&cloned_label);
}

/// Dispatches `__clone()` by runtime class id, or does nothing when the class has no hook.
fn emit_clone_hook(
    ctx: &mut FunctionContext<'_>,
    object_operand: ValueId,
    invocation_scope: Option<ValueId>,
) -> Result<()> {
    let candidates = clone_hook_candidates(ctx)?;
    if candidates.is_empty() {
        return Ok(());
    }
    let receiver_reg = abi::nested_call_reg(ctx.emitter).to_string();
    let no_hook = ctx.next_label("clone_hook_absent");
    let done = ctx.next_label("clone_hook_done");
    let labels = candidates
        .iter()
        .map(|candidate| ctx.next_label(&format!("clone_hook_{}", candidate.class_id)))
        .collect::<Vec<_>>();

    abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    // The unboxed payload low word is the clone's object pointer; park it in the
    // callee-saved nested-call register the receiver-register contract requires.
    let payload_reg = crate::codegen_support::mixed_unbox_payload_reg(ctx.emitter.target);
    abi::emit_reg_move(ctx.emitter, &receiver_reg, payload_reg);
    emit_clone_hook_class_dispatch(ctx, &receiver_reg, &candidates, &labels, &no_hook);
    abi::emit_jump(ctx.emitter, &no_hook);

    for (candidate, label) in candidates.iter().zip(labels.iter()) {
        ctx.emitter.label(label);
        if let Some(invocation_scope) = invocation_scope {
            emit_runtime_clone_hook_visibility_guard(
                ctx,
                candidate,
                invocation_scope,
            )?;
        } else if !clone_hook_is_visible(ctx, candidate) {
            emit_static_clone_hook_visibility_error(ctx, candidate);
            continue;
        }
        let receiver_ty = PhpType::Object(candidate.class_name.clone());
        let params = [receiver_ty.clone()];
        let refs = [false];
        let operands = [object_operand];
        let call_args = materialize_method_call_args_with_receiver_reg_and_refs(
            ctx,
            &receiver_reg,
            &receiver_ty,
            &operands,
            &params,
            &refs,
            RefArgCellLifetime::CallOnly,
        )?;
        let pad = direct_call_stack_pad_bytes(ctx, call_args.overflow_bytes);
        abi::emit_reserve_temporary_stack(ctx.emitter, pad);
        emit_resolved_method_call(ctx, &candidate.target)?;
        abi::emit_release_temporary_stack(ctx.emitter, pad);
        abi::emit_release_temporary_stack(ctx.emitter, call_args.overflow_bytes);
        emit_call_arg_temp_cleanups(ctx, &call_args, None)?;
        emit_ref_arg_writebacks(ctx, &call_args)?;
        abi::emit_jump(ctx.emitter, &done);
    }
    ctx.emitter.label(&no_hook);
    ctx.emitter.label(&done);
    Ok(())
}

/// Collects clone hooks for every runtime class, including an inherited private hook.
///
/// Normal method maps intentionally omit inaccessible parent-private methods. Cloning differs:
/// PHP still selects that hook and then performs the invocation-site visibility check, producing
/// `Call to private method ...` instead of silently skipping it on a child object.
fn clone_hook_candidates(ctx: &FunctionContext<'_>) -> Result<Vec<CloneHookCandidate>> {
    let method_key = php_symbol_key("__clone");
    let mut candidates = Vec::new();
    for (runtime_class, runtime_info) in &ctx.module.class_infos {
        let mut owner = Some(runtime_class.as_str());
        while let Some(class_name) = owner {
            let Some(info) = ctx.module.class_infos.get(class_name) else {
                break;
            };
            if info.methods.contains_key(&method_key) {
                let target = resolve_method_call_target(ctx, class_name, "__clone", 1)?;
                candidates.push(CloneHookCandidate {
                    class_id: runtime_info.class_id,
                    class_name: runtime_class.clone(),
                    declaring_class: info
                        .method_declaring_classes
                        .get(&method_key)
                        .cloned()
                        .unwrap_or_else(|| class_name.to_string()),
                    visibility: info
                        .method_visibilities
                        .get(&method_key)
                        .cloned()
                        .unwrap_or(Visibility::Public),
                    target,
                });
                break;
            }
            owner = info.parent.as_deref();
        }
    }
    candidates.sort_by_key(|candidate| candidate.class_id);
    Ok(candidates)
}

/// Dispatches the cloned receiver's dense class id to its resolved hook branch.
fn emit_clone_hook_class_dispatch(
    ctx: &mut FunctionContext<'_>,
    receiver_reg: &str,
    candidates: &[CloneHookCandidate],
    labels: &[String],
    no_hook: &str,
) {
    let (class_id_reg, compare_reg) = match ctx.emitter.target.arch {
        Arch::AArch64 => ("x9", "x10"),
        Arch::X86_64 => ("r11", "r10"),
    };
    abi::emit_load_from_address(ctx.emitter, class_id_reg, receiver_reg, 0);
    for (candidate, label) in candidates.iter().zip(labels.iter()) {
        abi::emit_load_int_immediate(ctx.emitter, compare_reg, candidate.class_id as i64);
        ctx.emitter
            .instruction(&format!("cmp {class_id_reg}, {compare_reg}"));
        match ctx.emitter.target.arch {
            Arch::AArch64 => ctx.emitter.instruction(&format!("b.eq {label}")),
            Arch::X86_64 => ctx.emitter.instruction(&format!("je {label}")),
        }
    }
    abi::emit_jump(ctx.emitter, no_hook);
}

/// Checks a callable wrapper's invocation scope against the selected hook declaration.
fn emit_runtime_clone_hook_visibility_guard(
    ctx: &mut FunctionContext<'_>,
    candidate: &CloneHookCandidate,
    invocation_scope: ValueId,
) -> Result<()> {
    let visibility = candidate.visibility.clone();
    if visibility == Visibility::Public {
        return Ok(());
    }
    let declaring = candidate.declaring_class.clone();
    let declaring_id = ctx
        .module
        .class_infos
        .get(&declaring)
        .map(|class| class.class_id as i64);
    let visible = ctx.next_label("clone_hook_scope_visible");
    ctx.load_value_to_result(invocation_scope)?;
    let scope_reg = abi::int_result_reg(ctx.emitter).to_string();
    let mut allowed = ctx
        .module
        .class_infos
        .iter()
        .filter_map(|(name, info)| {
            let permitted = match visibility {
                Visibility::Public => true,
                Visibility::Private => Some(info.class_id as i64) == declaring_id,
                Visibility::Protected => {
                    name == &declaring
                        || class_is_subclass_of(ctx, name, &declaring)
                        || class_is_subclass_of(ctx, &declaring, name)
                }
            };
            permitted.then_some(info.class_id as i64)
        })
        .collect::<Vec<_>>();
    allowed.sort_unstable();
    allowed.dedup();
    for class_id in allowed {
        emit_branch_if_reg_equals_immediate(ctx, &scope_reg, class_id, &visible);
    }
    emit_runtime_clone_hook_visibility_error(ctx, &visibility, &declaring, invocation_scope)?;
    ctx.emitter.label(&visible);
    Ok(())
}

/// Emits the statically known direct-call visibility error.
fn emit_static_clone_hook_visibility_error(
    ctx: &mut FunctionContext<'_>,
    candidate: &CloneHookCandidate,
) {
    super::super::exceptions::emit_error(
        ctx,
        &format!(
            "Call to {} method {}::__clone() from {}",
            visibility_name(&candidate.visibility),
            candidate.declaring_class,
            invocation_scope_phrase(ctx),
        ),
    );
}

/// Emits the exact denial phrase for the transported runtime scope id.
fn emit_runtime_clone_hook_visibility_error(
    ctx: &mut FunctionContext<'_>,
    visibility: &Visibility,
    declaring: &str,
    invocation_scope: ValueId,
) -> Result<()> {
    let mut scopes = ctx
        .module
        .class_infos
        .iter()
        .map(|(name, info)| (info.class_id as i64, name.clone()))
        .collect::<Vec<_>>();
    scopes.sort_by_key(|(class_id, _)| *class_id);
    let labels = scopes
        .iter()
        .map(|(class_id, _)| (*class_id, ctx.next_label("clone_hook_denied_scope")))
        .collect::<Vec<_>>();
    ctx.load_value_to_result(invocation_scope)?;
    let scope_reg = abi::int_result_reg(ctx.emitter).to_string();
    for (class_id, label) in &labels {
        emit_branch_if_reg_equals_immediate(ctx, &scope_reg, *class_id, label);
    }
    super::super::exceptions::emit_error(
        ctx,
        &format!(
            "Call to {} method {}::__clone() from global scope",
            visibility_name(visibility),
            declaring,
        ),
    );
    for ((_, class_name), (_, label)) in scopes.iter().zip(labels.iter()) {
        ctx.emitter.label(label);
        super::super::exceptions::emit_error(
            ctx,
            &format!(
                "Call to {} method {}::__clone() from scope {}",
                visibility_name(visibility),
                declaring,
                class_name,
            ),
        );
    }
    Ok(())
}

/// Branches when a transported dense class id equals one compile-time id.
fn emit_branch_if_reg_equals_immediate(
    ctx: &mut FunctionContext<'_>,
    reg: &str,
    value: i64,
    label: &str,
) {
    let scratch = abi::secondary_scratch_reg(ctx.emitter).to_string();
    abi::emit_load_int_immediate(ctx.emitter, &scratch, value);
    ctx.emitter.instruction(&format!("cmp {reg}, {scratch}"));
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("b.eq {label}")),
        Arch::X86_64 => ctx.emitter.instruction(&format!("je {label}")),
    }
}

/// Transfers the boxed clone to either a boxed or concrete object result slot.
fn store_clone_result(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let Some(result) = inst.result else {
        abi::emit_decref_if_refcounted(ctx.emitter, &PhpType::Mixed);
        return Ok(());
    };
    let result_ty = ctx.value_php_type(result)?.codegen_repr();
    if matches!(result_ty, PhpType::Mixed | PhpType::Union(_)) {
        return ctx.store_result_value(result);
    }
    let PhpType::Object(_) = result_ty else {
        return Err(CodegenIrError::invalid_module(format!(
            "clone() produced incompatible EIR result {result_ty:?}",
        )));
    };
    // A concrete result slot owns the object itself, so retain the payload, then retire the
    // Mixed box that carried it out of the adapter.
    let box_reg = abi::int_result_reg(ctx.emitter).to_string();
    let object_reg = abi::nested_call_reg(ctx.emitter).to_string();
    abi::emit_reserve_temporary_stack(ctx.emitter, 16);
    abi::emit_store_to_sp(ctx.emitter, &box_reg, 0);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let payload_reg = crate::codegen_support::mixed_unbox_payload_reg(ctx.emitter.target);
    abi::emit_reg_move(ctx.emitter, &object_reg, payload_reg);
    abi::emit_reg_move(ctx.emitter, &box_reg, &object_reg);
    abi::emit_incref_if_refcounted(ctx.emitter, &result_ty);
    abi::emit_load_temporary_stack_slot(ctx.emitter, &box_reg, 0);
    abi::emit_decref_if_refcounted(ctx.emitter, &PhpType::Mixed);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    abi::emit_reg_move(ctx.emitter, &box_reg, &object_reg);
    ctx.store_result_value(result)
}

/// Returns whether the operand is a literal empty array the frontend materialized inline.
fn value_is_empty_array_literal(ctx: &FunctionContext<'_>, value: ValueId) -> Result<bool> {
    let mut value = value;
    loop {
        let Some(value_ref) = ctx.function.value(value) else {
            return Ok(false);
        };
        let ValueDef::Instruction { inst, .. } = value_ref.def else {
            return Ok(false);
        };
        let Some(source) = ctx.function.instruction(inst) else {
            return Ok(false);
        };
        if source.op == Op::Acquire {
            let Some(operand) = source.operands.first().copied() else {
                return Ok(false);
            };
            value = operand;
            continue;
        }
        return Ok(source.op == Op::ArrayNew
            && matches!(source.immediate, Some(Immediate::Capacity(0))));
    }
}

/// Branches to `label` when the register holds zero.
fn emit_branch_if_zero(ctx: &mut FunctionContext<'_>, reg: &str, label: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("cbz {reg}, {label}")),
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("test {reg}, {reg}"));
            ctx.emitter.instruction(&format!("jz {label}"));
        }
    }
}

/// Branches to `label` when the register holds a non-zero value.
fn emit_branch_if_nonzero(ctx: &mut FunctionContext<'_>, reg: &str, label: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("cbnz {reg}, {label}")),
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("test {reg}, {reg}"));
            ctx.emitter.instruction(&format!("jnz {label}"));
        }
    }
}

/// Branches to `label` when an unboxed runtime tag register holds the object tag.
fn emit_branch_if_tag_is_object(ctx: &mut FunctionContext<'_>, tag_reg: &str, label: &str) {
    ctx.emitter.instruction(&format!("cmp {tag_reg}, 6"));
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("b.eq {label}")),
        Arch::X86_64 => ctx.emitter.instruction(&format!("je {label}")),
    }
}

/// Returns whether the active lexical class may invoke this runtime class's clone hook.
fn clone_hook_is_visible(ctx: &FunctionContext<'_>, candidate: &CloneHookCandidate) -> bool {
    let declaring = candidate.declaring_class.as_str();
    match candidate.visibility {
        Visibility::Public => true,
        Visibility::Private => ctx.function.lexical_class.as_deref() == Some(declaring),
        Visibility::Protected => ctx.function.lexical_class.as_deref().is_some_and(|current| {
            current == declaring
                || class_is_subclass_of(ctx, current, declaring)
                || class_is_subclass_of(ctx, declaring, current)
        }),
    }
}

/// Walks the emitted parent chain for protected hook access checks.
fn class_is_subclass_of(ctx: &FunctionContext<'_>, class_name: &str, ancestor: &str) -> bool {
    let mut current = ctx
        .module
        .class_infos
        .get(class_name)
        .and_then(|info| info.parent.as_deref());
    while let Some(name) = current {
        if name == ancestor {
            return true;
        }
        current = ctx
            .module
            .class_infos
            .get(name)
            .and_then(|info| info.parent.as_deref());
    }
    false
}

/// Names the invocation scope the way PHP's member-access diagnostics do.
///
/// PHP writes `from global scope` at the top level and `from scope Other` inside a class,
/// so the phrase carries its own trailing word and callers must not append one.
fn invocation_scope_phrase(ctx: &FunctionContext<'_>) -> String {
    ctx.function
        .lexical_class
        .as_deref()
        .map_or_else(|| "global scope".to_string(), |class| format!("scope {class}"))
}

/// Formats PHP visibility in runtime diagnostics.
fn visibility_name(visibility: &Visibility) -> &'static str {
    match visibility {
        Visibility::Public => "public",
        Visibility::Protected => "protected",
        Visibility::Private => "private",
    }
}
