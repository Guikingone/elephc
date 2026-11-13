//! Purpose:
//! Linear-scan register allocator (Poletto-Sarkar) over EIR functions. Assigns
//! registers to non-overlapping value live intervals, spilling to the stack
//! under register pressure, with separate integer and float pools.
//!
//! Called from:
//! - `crate::codegen` per function, before instruction lowering.
//!
//! Key details:
//! - Two register classes are allocated. Values whose live range never crosses a
//!   clobber point (call-free, per `crate::ir_passes::clobber`) prefer
//!   caller-saved registers, which need no prologue save/restore. Values that
//!   live across a clobber point use callee-saved registers, which survive
//!   calls. Both classes are disjoint from the scratch/result registers the
//!   instruction emitters use.
//! - The spill heuristic is use-weighted: under pressure the rarely-used,
//!   furthest-reaching interval is evicted first, keeping hot values in
//!   registers.
//! - First cut: only single-word `NonHeap` scalars (`I64`, `F64`) are
//!   register-eligible, and never block parameters or branch arguments, which
//!   stay in stack slots so the existing block-parameter moves are unchanged.
//!   Generators and functions with exception handlers fall back to all-spilled.

use std::collections::{HashMap, HashSet};

use crate::codegen::platform::{Arch, Target};
use crate::ir::{Function, IrType, Op, Ownership, Terminator, ValueId};
use crate::ir_passes::allocation::{Allocation, CallSave};
use crate::ir_passes::intervals::{
    build_intervals, clobber_instructions, precise_liveness, LiveInterval, PreciseLiveness,
};
use crate::ir_passes::liveness::compute_liveness;

/// Computes a register allocation for `func` on `target`.
///
/// Runs liveness and interval analysis, then a linear scan that assigns
/// callee-saved registers to eligible intervals and spills the longest-lived
/// interval when a pool is exhausted. Generators and functions containing
/// exception handlers conservatively fall back to all-spilled.
pub fn allocate_registers(func: &Function, target: Target) -> Allocation {
    allocate_registers_with_safe(func, target, &HashSet::new())
}

/// Like `allocate_registers`, with extra instructions the backend vouches are volatile-safe
/// (their lowering, known only with module context, touches no allocatable caller-saved
/// register on any path that returns).
pub fn allocate_registers_with_safe(
    func: &Function,
    target: Target,
    extra_safe: &HashSet<crate::ir::InstId>,
) -> Allocation {
    if func.flags.is_generator || has_exception_handlers(func) {
        return Allocation::all_spilled();
    }

    let liveness = compute_liveness(func);
    let intervals = build_intervals(func, &liveness, extra_safe);
    let ineligible = ineligible_values(func);

    let eligible: Vec<LiveInterval> = intervals
        .into_iter()
        .filter(|iv| is_eligible(func, iv, &ineligible))
        .collect();

    let hints = coalesce_partners(func);
    let precise = precise_liveness(func, &liveness);
    let mut allocation = scan(&eligible, target, &hints, &precise);
    allocation.set_call_saves(plan_call_saves(
        func,
        &eligible,
        &allocation,
        target,
        extra_safe,
        &precise,
    ));
    allocation
}

/// Lists, for every clobbering instruction, the caller-saved register values live across it.
fn plan_call_saves(
    func: &Function,
    intervals: &[LiveInterval],
    allocation: &Allocation,
    target: Target,
    extra_safe: &HashSet<crate::ir::InstId>,
    precise: &PreciseLiveness,
) -> HashMap<crate::ir::InstId, Vec<CallSave>> {
    let mut saves: HashMap<crate::ir::InstId, Vec<CallSave>> = HashMap::new();
    let caller_saved: HashSet<&'static str> = caller_int_pool(target)
        .iter()
        .chain(caller_float_pool(target))
        .copied()
        .collect();
    let clobbers = clobber_instructions(func, extra_safe);
    if clobbers.is_empty() {
        return saves;
    }
    for iv in intervals {
        if iv.call_free {
            continue;
        }
        let Some(reg) = allocation.register_of(iv.value) else {
            continue;
        };
        if !caller_saved.contains(reg) {
            continue;
        }
        let lo = clobbers.partition_point(|(pos, _)| *pos < iv.start);
        for (pos, inst) in clobbers[lo..].iter().take_while(|(pos, _)| *pos <= iv.end) {
            saves.entry(*inst).or_default().push(CallSave {
                value: iv.value,
                reg,
                store_before: iv.start < *pos,
                // Only reload a value that is really live after the clobber: a register shared
                // with a coalescing partner may hold the partner's (newer) value by then.
                reload_after: iv.end > *pos && precise.live_within(iv.value, *pos + 1, *pos + 1),
            });
        }
    }
    saves
}

/// Returns true when the function contains any exception handler. Such
/// functions are skipped because a thrown exception may clobber registers
/// before reaching a handler in this first cut.
fn has_exception_handlers(func: &Function) -> bool {
    func.instructions
        .iter()
        .any(|inst| inst.op == Op::TryPushHandler)
}

/// Collects values that must stay in stack slots regardless of their type.
///
/// Constants the backend never emits (rematerialized immediates) need no register. In
/// generators, block parameters and branch arguments also stay in slots: their resume edges
/// are implicit, so the parallel edge moves cannot cover them.
fn ineligible_values(func: &Function) -> HashSet<ValueId> {
    // Constants the backend never emits (read as immediates) need no register.
    let skipped = super::remat::rematerialized_consts(func).skipped;
    let mut ineligible: HashSet<ValueId> = skipped
        .iter()
        .filter_map(|inst| func.instruction(*inst).and_then(|inst| inst.result))
        .collect();
    let has_suspend = func
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, Some(Terminator::GeneratorSuspend { .. })));
    if !func.flags.is_generator && !has_suspend {
        // Edge copies read every argument before writing any parameter, and any value
        // live across an edge is live at the target's entry (the parameters' definition
        // point), so parameters and arguments can share the ordinary register pools.
        return ineligible;
    }
    for block in &func.blocks {
        for param in &block.params {
            ineligible.insert(*param);
        }
        if let Some(term) = &block.terminator {
            for arg in terminator_branch_args(term) {
                ineligible.insert(arg);
            }
        }
    }
    ineligible
}

/// Returns the values a terminator passes as block-parameter arguments. These
/// are distinct from condition/scrutinee/return uses, which are ordinary uses.
fn terminator_branch_args(term: &Terminator) -> Vec<ValueId> {
    match term {
        Terminator::Br { args, .. } => args.clone(),
        Terminator::CondBr {
            then_args,
            else_args,
            ..
        } => then_args.iter().chain(else_args).copied().collect(),
        Terminator::Switch {
            cases,
            default_args,
            ..
        } => cases
            .iter()
            .flat_map(|case| case.args.iter().copied())
            .chain(default_args.iter().copied())
            .collect(),
        Terminator::GeneratorSuspend { resume_args, .. } => resume_args.clone(),
        Terminator::Return { .. }
        | Terminator::Throw { .. }
        | Terminator::Fatal { .. }
        | Terminator::Unreachable => Vec::new(),
    }
}

/// Returns true when an interval's value can live in a register: a single-word
/// non-heap scalar that `ineligible_values` does not exclude.
fn is_eligible(func: &Function, iv: &LiveInterval, ineligible: &HashSet<ValueId>) -> bool {
    if ineligible.contains(&iv.value) {
        return false;
    }
    if !matches!(iv.ir_type, IrType::I64 | IrType::F64) {
        return false;
    }
    func.value(iv.value)
        .map(|value| value.ownership == Ownership::NonHeap)
        .unwrap_or(false)
}

/// Which of the four register pools an active interval drew its register from.
/// Determines the free list a register returns to on expiry, and whether the
/// function must save/restore the register (only callee-saved pools).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PoolKind {
    /// Caller-saved integer pool (`x12`–`x15` / `rsi`,`rdi`,`r8`,`r9`).
    CallerInt,
    /// Callee-saved integer pool (`x21`–`x28` / `rbx`).
    CalleeInt,
    /// Caller-saved float pool (`d16`–`d23` / `xmm2`–`xmm7`).
    CallerFloat,
    /// Callee-saved float pool (`d8`–`d14`; empty on x86_64).
    CalleeFloat,
}

impl PoolKind {
    /// Returns true when registers from this pool are callee-saved and so must
    /// be preserved across the function via prologue/epilogue save/restore.
    fn is_callee_saved(self) -> bool {
        matches!(self, PoolKind::CalleeInt | PoolKind::CalleeFloat)
    }
}

/// An interval currently holding a register during the scan.
struct ActiveInterval {
    /// Linear position where the interval ends.
    end: u32,
    /// Register the interval occupies.
    reg: &'static str,
    /// Values sharing the register over this (merged) interval: one value, or coalesced
    /// edge partners that are never live at the same time.
    members: Vec<ValueId>,
    /// Use count, used to choose the cheapest interval to spill.
    weight: u32,
    /// Pool the register came from, so it returns to the right free list.
    pool: PoolKind,
}

/// Mutable free lists for the four register pools during a scan.
struct FreePools {
    caller_int: Vec<&'static str>,
    callee_int: Vec<&'static str>,
    caller_float: Vec<&'static str>,
    callee_float: Vec<&'static str>,
}

impl FreePools {
    /// Builds the four free lists for `target`, ordered so `pop` hands out the
    /// lowest-numbered register first.
    fn new(target: Target) -> Self {
        let rev = |regs: &[&'static str]| -> Vec<&'static str> { regs.iter().rev().copied().collect() };
        Self {
            caller_int: rev(caller_int_pool(target)),
            callee_int: rev(callee_int_pool(target)),
            caller_float: rev(caller_float_pool(target)),
            callee_float: rev(callee_float_pool(target)),
        }
    }

    /// Returns the free list for a pool kind.
    fn list_mut(&mut self, pool: PoolKind) -> &mut Vec<&'static str> {
        match pool {
            PoolKind::CallerInt => &mut self.caller_int,
            PoolKind::CalleeInt => &mut self.callee_int,
            PoolKind::CallerFloat => &mut self.caller_float,
            PoolKind::CalleeFloat => &mut self.callee_float,
        }
    }
}

/// Returns the pools an interval may draw from, most-preferred first.
///
/// Call-free intervals prefer caller-saved registers, which need no save or
/// restore, and fall back to callee-saved under pressure. Intervals that live
/// across a clobber point may only use callee-saved registers, which survive
/// calls and any caller-saved register the lowering touches.
fn candidate_pools(iv: &LiveInterval) -> &'static [PoolKind] {
    let is_float = iv.ir_type == IrType::F64;
    // A caller-saved register held across clobber points costs a store and a reload at each
    // one; take that only when the value is used at least as often as it is preserved.
    let preserve_ok = iv.clobber_count <= iv.weight;
    match (is_float, iv.call_free, preserve_ok) {
        (false, true, _) => &[PoolKind::CallerInt, PoolKind::CalleeInt],
        (false, false, true) => &[PoolKind::CalleeInt, PoolKind::CallerInt],
        (false, false, false) => &[PoolKind::CalleeInt],
        (true, true, _) => &[PoolKind::CallerFloat, PoolKind::CalleeFloat],
        (true, false, true) => &[PoolKind::CalleeFloat, PoolKind::CallerFloat],
        (true, false, false) => &[PoolKind::CalleeFloat],
    }
}

/// Runs the linear scan over already-eligible, start-sorted intervals.
///
/// Maintains four per-pool free lists (caller/callee × int/float) and an active
/// set. Each interval is assigned from its preferred pool; on exhaustion it
/// spills using a use-weighted heuristic that keeps frequently-used values in
/// registers.
fn scan(
    eligible: &[LiveInterval],
    target: Target,
    hints: &HashMap<ValueId, Vec<ValueId>>,
    precise: &PreciseLiveness,
) -> Allocation {
    let param_blocks: HashMap<ValueId, crate::ir::BlockId> = precise.param_blocks().clone();
    let mut free = FreePools::new(target);
    let mut active: Vec<ActiveInterval> = Vec::new();
    let mut assignments: HashMap<ValueId, &'static str> = HashMap::new();
    let mut used: Vec<&'static str> = Vec::new();

    for iv in eligible {
        expire_old_intervals(&mut active, iv.start, &mut free);

        let pools = candidate_pools(iv);
        if let Some(index) = shareable_partner(iv, pools, hints, &active, precise, &param_blocks) {
            let entry = &mut active[index];
            entry.end = entry.end.max(iv.end);
            entry.weight = entry.weight.max(iv.weight);
            entry.members.push(iv.value);
            assignments.insert(iv.value, entry.reg);
            continue;
        }
        let hinted = take_hinted_register(iv.value, pools, hints, &assignments, &mut free);
        let chosen = match hinted {
            Some(choice) => Some(choice),
            None => pools
                .iter()
                .copied()
                .find(|&p| !free.list_mut(p).is_empty())
                .map(|pool| (pool, free.list_mut(pool).pop().expect("pool checked non-empty"))),
        };
        if let Some((pool, reg)) = chosen {
            if pool.is_callee_saved() {
                used.push(reg);
            }
            assignments.insert(iv.value, reg);
            active.push(ActiveInterval {
                end: iv.end,
                reg,
                members: vec![iv.value],
                weight: iv.weight,
                pool,
            });
        } else {
            spill_at_interval(iv, pools, &mut active, &mut assignments, &mut used);
        }
    }

    Allocation::from_assignments(assignments, used)
}

/// Maps each block parameter to the values passed for it on incoming edges, and each such
/// argument back to the parameter. Giving both ends the same register turns the edge copy
/// into a no-op (classic move coalescing on SSA block arguments).
fn coalesce_partners(func: &Function) -> HashMap<ValueId, Vec<ValueId>> {
    let mut partners: HashMap<ValueId, Vec<ValueId>> = HashMap::new();
    let mut link = |param: ValueId, arg: ValueId| {
        if param == arg {
            return;
        }
        partners.entry(param).or_default().push(arg);
        partners.entry(arg).or_default().push(param);
    };
    // Two-address integer ops compute into a copy of their left operand on x86_64; giving the
    // result the operand's register (when the operand dies there) drops that copy.
    for inst in &func.instructions {
        if matches!(
            inst.op,
            Op::IAdd | Op::ISub | Op::IMul | Op::IBitAnd | Op::IBitOr | Op::IBitXor
        ) {
            if let (Some(result), Some(lhs)) = (inst.result, inst.operands.first()) {
                link(result, *lhs);
            }
        }
    }
    for block in &func.blocks {
        let Some(term) = &block.terminator else {
            continue;
        };
        let mut edge = |target: crate::ir::BlockId, args: &[ValueId]| {
            if let Some(target_block) = func.block(target) {
                for (param, arg) in target_block.params.iter().zip(args) {
                    link(*param, *arg);
                }
            }
        };
        match term {
            Terminator::Br { target, args } => edge(*target, args),
            Terminator::CondBr {
                then_target,
                then_args,
                else_target,
                else_args,
                ..
            } => {
                edge(*then_target, then_args);
                edge(*else_target, else_args);
            }
            Terminator::Switch {
                cases,
                default,
                default_args,
                ..
            } => {
                for case in cases {
                    edge(case.target, &case.args);
                }
                edge(*default, default_args);
            }
            _ => {}
        }
    }
    partners
}

/// Finds an active register whose holders include a coalescing partner of `iv` and are all
/// dead across `iv`'s whole interval, so `iv` can share that register.
///
/// Sharing is sound when, at every position where `iv` may be live (its contiguous interval,
/// from just after its definition), no other holder is live, and no edge copy into a holder
/// (a block-parameter write) lands inside the interval with a different value. The defining
/// instruction may still read a holder: operands are read before the result is written.
fn shareable_partner(
    iv: &LiveInterval,
    pools: &[PoolKind],
    hints: &HashMap<ValueId, Vec<ValueId>>,
    active: &[ActiveInterval],
    precise: &PreciseLiveness,
    param_blocks: &HashMap<ValueId, crate::ir::BlockId>,
) -> Option<usize> {
    let partners = partner_closure(iv.value, hints);
    if partners.is_empty() {
        return None;
    }
    let own_block = param_blocks.get(&iv.value);
    let lo = iv.start + 1;
    let hi = iv.end.max(lo);
    active.iter().position(|entry| {
        pools.contains(&entry.pool)
            && entry.members.iter().any(|member| partners.contains(member))
            // Two parameters of one block are written by one parallel copy: never share.
            && !entry
                .members
                .iter()
                .any(|member| own_block.is_some() && param_blocks.get(member) == own_block)
            && entry.members.iter().all(|member| {
                !precise.live_within(*member, lo, hi)
                    && precise
                        .edge_writes(*member)
                        .iter()
                        .all(|&(pos, arg)| arg == iv.value || pos < iv.start || pos > hi)
            })
    })
}

/// Collects the values reachable from `value` through coalescing links (a few hops), nearest
/// first, so a chain such as `t = a & 7; s = t + b; p = s & m` -> block parameter can steer
/// every link toward the parameter's register.
fn partner_closure(value: ValueId, hints: &HashMap<ValueId, Vec<ValueId>>) -> Vec<ValueId> {
    const MAX_HOPS: usize = 4;
    let mut seen: Vec<ValueId> = vec![value];
    let mut frontier: Vec<ValueId> = vec![value];
    for _ in 0..MAX_HOPS {
        let mut next = Vec::new();
        for current in &frontier {
            for partner in hints.get(current).map(Vec::as_slice).unwrap_or(&[]) {
                if !seen.contains(partner) {
                    seen.push(*partner);
                    next.push(*partner);
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    seen.remove(0);
    seen
}

/// Takes the register already assigned to one of `value`'s coalescing partners when it is
/// free in one of the interval's allowed pools, so the edge copy between them vanishes.
fn take_hinted_register(
    value: ValueId,
    pools: &[PoolKind],
    hints: &HashMap<ValueId, Vec<ValueId>>,
    assignments: &HashMap<ValueId, &'static str>,
    free: &mut FreePools,
) -> Option<(PoolKind, &'static str)> {
    for partner in &partner_closure(value, hints) {
        let Some(&reg) = assignments.get(partner) else {
            continue;
        };
        for &pool in pools {
            let list = free.list_mut(pool);
            if let Some(index) = list.iter().position(|r| *r == reg) {
                list.remove(index);
                return Some((pool, reg));
            }
        }
    }
    None
}

/// Frees registers held by intervals that end at or before `position`, so the
/// register can be reused by the interval starting there.
fn expire_old_intervals(active: &mut Vec<ActiveInterval>, position: u32, free: &mut FreePools) {
    active.retain(|a| {
        if a.end <= position {
            free.list_mut(a.pool).push(a.reg);
            false
        } else {
            true
        }
    });
}

/// Resolves register pressure for `current` when all its candidate pools are
/// exhausted.
///
/// Among `current` and every active interval that holds a register in one of
/// `current`'s candidate pools, the cheapest interval to spill is chosen: the
/// one with the lowest use weight, breaking ties toward the furthest end so the
/// freed register stays available longest (the classic furthest-use rule). If
/// `current` is the cheapest, it stays spilled; otherwise the victim is evicted
/// and `current` takes its register.
fn spill_at_interval(
    current: &LiveInterval,
    pools: &[PoolKind],
    active: &mut Vec<ActiveInterval>,
    assignments: &mut HashMap<ValueId, &'static str>,
    used: &mut Vec<&'static str>,
) {
    // The interior victim with the best (lowest) spill cost among matching-pool
    // actives, if any beats `current`.
    let victim = active
        .iter()
        .enumerate()
        .filter(|(_, a)| pools.contains(&a.pool))
        .min_by(|(_, a), (_, b)| spill_cost(a.weight, a.end).cmp(&spill_cost(b.weight, b.end)));

    let Some((index, a)) = victim else {
        // No matching-pool active to steal from; `current` stays spilled.
        return;
    };

    // Keep `current` spilled when it is itself the cheapest to spill.
    if spill_cost(current.weight, current.end) <= spill_cost(a.weight, a.end) {
        return;
    }

    let reg = a.reg;
    let pool = a.pool;
    for member in &a.members {
        assignments.remove(member);
    }
    active.remove(index);
    if pool.is_callee_saved() {
        // The register was already clobbered when first assigned; ensure it is
        // recorded so the prologue/epilogue preserves it for `current` too.
        used.push(reg);
    }
    assignments.insert(current.value, reg);
    active.push(ActiveInterval {
        end: current.end,
        reg,
        members: vec![current.value],
        weight: current.weight,
        pool,
    });
}

/// Returns a spill-cost key for an interval: lower is cheaper to spill.
///
/// Ordered by use weight ascending (rarely-used values spill first), then by
/// furthest end (`Reverse`), so among equally-used intervals the one that would
/// otherwise block a register the longest is the cheaper victim. Spilling the
/// furthest-reaching interval frees the register for the longest stretch and
/// minimizes future pressure — the classic furthest-use rule applied as the
/// tie-breaker under the use-weighted primary criterion.
fn spill_cost(weight: u32, end: u32) -> (u32, std::cmp::Reverse<u32>) {
    (weight, std::cmp::Reverse(end))
}

/// Returns the integer callee-saved register pool for `target`, excluding the
/// frame pointer, scratch registers, and registers the emitters use inline.
fn callee_int_pool(target: Target) -> &'static [&'static str] {
    match target.arch {
        Arch::AArch64 => &["x21", "x22", "x23", "x24", "x25", "x26", "x27", "x28"],
        // Only rbx is reliably preserved across the hand-written x86_64 runtime
        // routines and shared heap-marker codegen; r14/r15 are used there as
        // scratch without ABI-compliant save/restore, so they are not allocated.
        Arch::X86_64 => &["rbx"],
    }
}

/// Returns the float callee-saved register pool for `target`. SysV x86_64 has
/// no callee-saved XMM registers, so float values that live across a clobber
/// point are never register-allocated there and must stay spilled.
fn callee_float_pool(target: Target) -> &'static [&'static str] {
    match target.arch {
        Arch::AArch64 => &["d8", "d9", "d10", "d11", "d12", "d13", "d14"],
        Arch::X86_64 => &[],
    }
}

/// Returns the integer caller-saved register pool for `target`.
///
/// These registers are clobbered by any call, so they may only hold values
/// whose live range never crosses a clobber point. The audited volatile-safe
/// op lowerings (see `crate::ir_passes::clobber`) never touch them, so a
/// call-free value keeps its register intact with no save/restore.
fn caller_int_pool(target: Target) -> &'static [&'static str] {
    match target.arch {
        // x12–x15 are caller-saved temporaries the volatile-safe lowerings never use.
        Arch::AArch64 => &["x12", "x13", "x14", "x15"],
        // rsi/rdi/r8/r9 are caller-saved argument registers, untouched by the
        // volatile-safe lowerings (which use rax/rdx/rcx/r10/r11 only).
        Arch::X86_64 => &["rsi", "rdi", "r8", "r9"],
    }
}

/// Returns the float caller-saved register pool for `target`.
///
/// On x86_64, where there are no callee-saved XMM registers, this is the only
/// way float values are register-allocated at all, enabled for call-free
/// intervals. The volatile-safe float lowerings use only d0/d1 and xmm0/xmm1.
fn caller_float_pool(target: Target) -> &'static [&'static str] {
    match target.arch {
        // d16–d30 are caller-saved vector registers never used by the backend
        // as arguments, results, or scratch.
        Arch::AArch64 => &[
            "d16", "d17", "d18", "d19", "d20", "d21", "d22", "d23", "d24", "d25", "d26", "d27",
            "d28", "d29", "d30",
        ],
        Arch::X86_64 => &[
            "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "xmm8", "xmm9", "xmm10", "xmm11",
            "xmm12", "xmm13", "xmm14",
        ],
    }
}
