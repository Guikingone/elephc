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

use crate::ir::{BlockId, Function, Immediate, IrType, LocalKind, Op, ValueDef, ValueId};
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
    let writers: HashSet<_> = function.instructions.iter().filter_map(|inst| {
        if inst.op != Op::StoreLocal { return None; }
        let Some(Immediate::LocalSlot(slot)) = inst.immediate else { return None; };
        inst.operands.first().is_some_and(|&value| numeric_writer(function, value)).then_some(slot)
    }).collect();
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
        if loads.is_empty() || (!exceptional && !cyclic(function, block.id)) {
            continue;
        }
        narrow.extend(loads);
    }
    let mut bindings: HashMap<String, HashSet<usize>> = HashMap::new();
    for local in &function.locals {
        if !narrow.contains(&local.id) || local.kind != LocalKind::PhpLocal
            || local.php_type.codegen_repr() != PhpType::Mixed
        { continue; }
        let name = ctx.local_slots.iter().find(|(_, slot)| **slot == local.id)
            .map(|(name, _)| name.as_str()).or_else(|| local.name.as_deref().map(|name| {
                name.strip_suffix("#cow").unwrap_or(name)
            }));
        let Some(name) = name else { continue; };
        if !ctx.is_ref_bound_local(name) {
            bindings.entry(name.to_owned()).or_default()
                .insert(ctx.local_binding_ordinal(name, Some(local.id)));
        }
    }
    bindings
}

/// Follows assignment ownership wrappers to arithmetic that can promote to a PHP float.
fn numeric_writer(function: &Function, mut value: ValueId) -> bool {
    for _ in 0..function.values.len() {
        let Some(ValueDef::Instruction { inst, .. }) = function.value(value).map(|value| value.def) else {
            return false;
        };
        let inst = &function.instructions[inst.as_raw() as usize];
        match inst.op {
            Op::ICheckedAdd | Op::ICheckedSub | Op::ICheckedMul | Op::ICheckedPow
                | Op::MixedNumericBinop | Op::StrIncDec => return true,
            Op::Acquire | Op::MixedClone | Op::Move | Op::Borrow | Op::EnsureOwned => {
                let Some(&source) = inst.operands.first() else { return false; };
                value = source;
            }
            _ => return false,
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
    ctx.restore(checkpoint);
    ctx.numeric_proof_sealed = true;
    if !suppress {
        ctx.numeric_closure_sites = sites;
    }
    for (name, bindings) in if suppress { scalar_force } else { force } {
        ctx.final_mixed_locals.entry(name).or_default().extend(bindings);
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

