//! Purpose:
//! Finds loop-carried numeric slots whose first lowered read would truncate a promoted float.
//!
//! Called from:
//! - `function::lower_body_into_function` before ownership finalization.
//!
//! Key details:
//! - A checked arithmetic store can carry a float around a backedge despite an integer entry seed.
//! - Discovery changes no instructions; replay emits ordinary Mixed reads and preserves PHP tags.

use std::collections::{HashMap, HashSet};

use crate::ir::{BlockId, Function, Immediate, InstId, IrType, LocalKind, LocalSlotId, Op,
    RuntimeCallTarget, RuntimeFnId, ValueDef, ValueId};
use crate::parser::ast::Stmt;
use crate::types::PhpType;
use super::context::LoweringContext;

/// Returns private numeric names with cyclic reads or unmodeled exceptional continuation.
fn loop_carried(ctx: &LoweringContext<'_, '_>) -> HashMap<String, HashSet<usize>> {
    let function = ctx.builder.function();
    // Explicit CFG edges omit throws into handlers. Widen conservatively in functions
    // with handlers, including a loop whose body always reaches its backedge via catch.
    let exceptional = function.instructions.iter().any(|inst| matches!(inst.op,
        Op::TryPushHandler | Op::CatchBind | Op::FinallyEnter));
    let cyclic_blocks: HashSet<_> = function.blocks.iter()
        .filter(|block| exceptional || cyclic(function, block.id)).map(|block| block.id).collect();
    let mut local_writers: HashMap<LocalSlotId, Vec<ValueId>> = HashMap::new();
    for inst in &function.instructions {
        if inst.op == Op::StoreLocal {
            if let (Some(Immediate::LocalSlot(slot)), Some(&source)) = (&inst.immediate, inst.operands.first()) {
                local_writers.entry(*slot).or_default().push(source);
            }
        }
    }
    let mut writers = HashSet::new();
    for block in &function.blocks {
        if !cyclic_blocks.contains(&block.id) { continue; }
        for &id in &block.instructions {
            let inst = &function.instructions[id.as_raw() as usize];
            if inst.op != Op::StoreLocal { continue; }
            let Some(Immediate::LocalSlot(slot)) = inst.immediate else { continue; };
            if inst.operands.first().is_some_and(|&value| numeric_writer(function, &local_writers, value)) {
                writers.insert(slot);
            }
        }
    }
    if writers.is_empty() { return HashMap::new(); }
    let mut narrow = HashSet::new();
    for block in &function.blocks {
        let loads: Vec<_> = block.instructions.iter().filter_map(|id| {
            let inst = &function.instructions[id.as_raw() as usize];
            if inst.op != Op::LoadLocal || !matches!(inst.result_type, IrType::I64 | IrType::F64) {
                return None;
            }
            let Some(Immediate::LocalSlot(slot)) = inst.immediate else { return None; };
            writers.contains(&slot).then_some(slot)
        }).collect();
        if loads.is_empty() || !cyclic_blocks.contains(&block.id) {
            continue;
        }
        narrow.extend(loads);
    }
    let mut bindings: HashMap<String, HashSet<usize>> = HashMap::new();
    for local in &function.locals {
        if !narrow.contains(&local.id) || local.kind != LocalKind::PhpLocal
            || local.php_type.codegen_repr() != PhpType::Mixed
        { continue; }
        let Some((name, ordinal)) = binding(ctx, local.id) else { continue; };
        if !ctx.is_ref_bound_local(&name) && !ctx.local_uses_global_storage(&name) {
            bindings.entry(name).or_default().insert(ordinal);
        }
    }
    bindings
}

/// Follows assignment ownership wrappers to arithmetic that can promote to a PHP float.
fn numeric_writer(function: &Function, writers: &HashMap<LocalSlotId, Vec<ValueId>>, value: ValueId) -> bool {
    let mut pending = vec![value];
    let mut seen = HashSet::new();
    while let Some(value) = pending.pop() {
        if !seen.insert(value) { continue; }
        let Some(ValueDef::Instruction { inst, .. }) = function.value(value).map(|value| value.def) else {
            continue;
        };
        let inst = &function.instructions[inst.as_raw() as usize];
        match inst.op {
            Op::ICheckedAdd | Op::ICheckedSub | Op::ICheckedMul | Op::ICheckedPow
                | Op::MixedNumericBinop | Op::StrIncDec => return true,
            Op::Acquire | Op::MixedClone | Op::Move | Op::Borrow | Op::EnsureOwned => {
                pending.extend(inst.operands.first().copied());
            }
            Op::LoadLocal => {
                let Some(Immediate::LocalSlot(slot)) = inst.immediate else { continue; };
                pending.extend(writers.get(&slot).into_iter().flatten().copied());
            }
            _ => {}
        }
    }
    false
}

/// Requires an explicit nonempty path back to a block, including nested loop backedges.
fn cyclic(function: &Function, start: BlockId) -> bool {
    let mut pending = successors(function, start);
    let mut seen = HashSet::new();
    while let Some(block) = pending.pop() {
        if block == start {
            return true;
        }
        if seen.insert(block) {
            pending.extend(successors(function, block));
        }
    }
    false
}

/// Reads explicit edges only; exceptional edges cannot invalidate a conservative storage widening.
fn successors(function: &Function, block: BlockId) -> Vec<BlockId> {
    function.blocks[block.as_raw() as usize].terminator.as_ref()
        .map(crate::ir_passes::successors).unwrap_or_default()
}

/// Replays numeric loop reads and admitted closure calls against their final Mixed storage.
///
/// The proof is sealed from the discovery pass, whose entry seeds are still the
/// proven `I64`/`F64` types. Replaying must not re-seed those locals from the Mixed
/// label it is about to force. Parameter and reference cases keep descriptor calls;
/// by-value parameters can separately acquire Mixed frame storage through their prologue.
pub(crate) fn lower_body(
    ctx: &mut LoweringContext<'_, '_>,
    body: &[Stmt],
) {
    let checkpoint = ctx.snapshot();
    lower_stmts(ctx, body);
    let scalar_force = loop_carried(ctx);
    let sites = ctx.numeric_closure_sites.clone();
    let tracked = ctx.numeric_tracked_locals.clone();
    let params = ctx
        .builder
        .function()
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    let mut force = scalar_force.clone();
    let mut suppress = false;
    for name in &tracked {
        if !narrow_load_of_mixed_slot(ctx, name) {
            continue;
        }
        if params.contains(name) || ctx.is_ref_bound_local(name) {
            suppress = true;
        } else {
            let binding = ctx.local_binding_ordinal(name, ctx.local_slots.get(name).copied());
            force.entry(name.clone()).or_default().insert(binding);
        }
    }
    if tracked.iter().any(|name| ctx.is_ref_bound_local(name)) {
        suppress = true;
    }
    if !suppress && force.is_empty() {
        return;
    }
    let mut final_force = if suppress { scalar_force } else { force };
    ctx.restore(checkpoint.clone());
    replay(ctx, body, &sites, suppress, &final_force);
    // Every trial sees all not-yet-proven recurrences widened. A promoted float bound
    // cannot masquerade as integer. Removing proven constraints can expose a copied
    // counter's integer seed or a surrounding loop, so repeat to a bounded fixed point.
    for _ in 0..16 {
        let calls = readonly_calls(ctx);
        let proven = crate::ir_passes::proven_integer_slots(ctx.builder.function(), ctx.data, &calls);
        let safe_bindings: Vec<_> = proven.into_iter().filter_map(|slot| binding(ctx, slot)).collect();
        let mut changed = false;
        for (name, ordinal) in safe_bindings {
            if let Some(bindings) = final_force.get_mut(&name) {
                changed |= bindings.remove(&ordinal);
            }
        }
        if !changed { break; }
        ctx.restore(checkpoint.clone());
        replay(ctx, body, &sites, suppress, &final_force);
    }
}

/// Replays one body with a binding-scoped storage contract and the original call-site proofs.
fn replay(
    ctx: &mut LoweringContext<'_, '_>, body: &[Stmt], sites: &HashSet<crate::span::Span>,
    suppress: bool, force: &HashMap<String, HashSet<usize>>,
) {
    ctx.numeric_proof_sealed = true;
    if !suppress {
        ctx.numeric_closure_sites = sites.clone();
    }
    for (name, bindings) in force {
        ctx.final_mixed_locals.entry(name.clone()).or_default().extend(bindings);
    }
    // Parameters already have slots at the checkpoint. Their prologue boxes an incoming
    // scalar when its final frame representation is Mixed, without changing the call ABI.
    let mut entry_names: Vec<_> = ctx.final_mixed_locals.keys().cloned().collect();
    entry_names.sort();
    for name in entry_names {
        if ctx.has_local_slot(&name) && ctx.is_final_mixed_local(&name) {
            ctx.set_local_type(&name, PhpType::Mixed);
        }
    }
    lower_stmts(ctx, body);
}

/// Resolves a physical storage epoch even after unset removed its active name binding.
fn binding(ctx: &LoweringContext<'_, '_>, slot: LocalSlotId) -> Option<(String, usize)> {
    let local = ctx.builder.function().locals.get(slot.as_raw() as usize)?;
    let name = ctx.local_slots.iter().find(|(_, candidate)| **candidate == slot)
        .map(|(name, _)| name.as_str()).or_else(|| local.name.as_deref().map(|name| {
            name.strip_suffix("#cow").unwrap_or(name)
        }))?;
    Some((name.to_owned(), ctx.local_binding_ordinal(name, Some(slot))))
}

/// Identifies call positions that copy scalar values rather than alias their originating slot.
fn readonly_calls(ctx: &LoweringContext<'_, '_>) -> HashMap<InstId, HashSet<usize>> {
    let function = ctx.builder.function();
    function.instructions.iter().enumerate().filter_map(|(index, inst)| {
        let parameters = match (inst.op, &inst.immediate) {
            (Op::Call, Some(Immediate::Data(target))) => {
                let name = ctx.data.function_names.get(target.as_raw() as usize)?;
                let signature = ctx.functions.get(name)?;
                if signature.variadic.is_some() || inst.operands.len() != signature.params.len() {
                    return None;
                }
                signature.ref_params.clone()
            }
            (Op::RuntimeCall, Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(target)))) => {
                let definition = crate::builtins::registry::lookup(target.as_eir())?;
                if !definition.spec.semantics.runtime_functions.contains(*target) { return None; }
                if definition.variadic.is_some() {
                    // array_push keeps individual by-value variadic operands in EIR;
                    // its first operand alone is a by-reference receiver address.
                    if *target != RuntimeFnId::ArrayPush
                        || definition.ref_params.len() != definition.params.len()
                    { return None; }
                    let regular = definition.params.len().checked_sub(1)?;
                    if inst.operands.len() < regular { return None; }
                    let mut parameters = definition.ref_params[..regular].to_vec();
                    parameters.resize(inst.operands.len(), *definition.ref_params.last()?);
                    parameters
                } else {
                    if inst.operands.len() != definition.params.len() { return None; }
                    definition.ref_params.clone()
                }
            }
            _ => return None,
        };
        let positions = parameters.iter().enumerate()
            .filter_map(|(index, by_ref)| (!by_ref).then_some(index)).collect();
        Some((InstId::from_raw(index as u32), positions))
    }).collect()
}

/// Lowers each statement in source order.
fn lower_stmts(ctx: &mut LoweringContext<'_, '_>, body: &[Stmt]) {
    for stmt in body {
        crate::ir_lower::stmt::lower_stmt(ctx, stmt);
    }
}

/// Reports a load whose result type is narrower than the slot's final Mixed storage.
fn narrow_load_of_mixed_slot(ctx: &LoweringContext<'_, '_>, name: &str) -> bool {
    let Some(slot) = ctx.local_slots.get(name).copied() else {
        return false;
    };
    if ctx.builder.local_php_type(slot).codegen_repr() != PhpType::Mixed {
        return false;
    }
    ctx.builder.function().instructions.iter().any(|inst| {
        inst.op == Op::LoadLocal
            && matches!(inst.immediate, Some(Immediate::LocalSlot(loaded)) if loaded == slot)
            && inst.result_php_type.codegen_repr() != PhpType::Mixed
    })
}
