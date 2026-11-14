//! Purpose:
//! Builds linear-program-order live intervals for EIR values from per-block
//! liveness. Intervals are the input to the linear-scan register allocator.
//!
//! Called from:
//! - `crate::ir_passes::regalloc` (register allocation core).
//!
//! Key details:
//! - Blocks are numbered in reverse postorder. Each block reserves a position
//!   for its parameters (block entry), one per instruction, and one for its
//!   terminator. Intervals are contiguous `[start, end]` ranges (classic
//!   Poletto-Sarkar): a value is conservatively considered live across any hole
//!   between its definition and its last use.

use std::collections::HashMap;

use crate::ir::{BlockId, Function, InstId, IrType, ValueDef, ValueId};
use crate::ir_passes::cfg::reverse_postorder;
use crate::ir_passes::clobber::{inst_is_volatile_safe, ref_cell_slots, terminator_is_volatile_safe};
use crate::ir_passes::liveness::{terminator_uses, LivenessInfo};

/// A value's contiguous live range in linear program order: live from `start`
/// (its definition point) through `end` (its last use or last live-out point).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveInterval {
    /// The value this interval describes.
    pub value: ValueId,
    /// The value's IR type, used by the allocator to pick a register pool.
    pub ir_type: IrType,
    /// Linear position of the value's definition.
    pub start: u32,
    /// Linear position of the value's last use or last live-out edge.
    pub end: u32,
    /// Number of times the value is used (as an instruction operand or
    /// terminator use). Drives the spill heuristic: frequently-used values are
    /// cheaper to keep in registers and more expensive to spill.
    pub weight: u32,
    /// True when no instruction or terminator that could clobber a caller-saved
    /// register executes while the value is live (strictly after its definition,
    /// up to and including its last use). Such intervals may use caller-saved
    /// registers, which need no prologue save/restore.
    pub call_free: bool,
    /// Number of clobber points (instructions or terminators whose lowering may clobber a
    /// caller-saved register) inside `[start, end]`. A caller-saved register held across them
    /// is saved and reloaded around each one.
    pub clobber_count: u32,
}

/// Linear position numbering for a function: where each block's parameters,
/// instructions, and terminator sit on the single linear axis the allocator
/// scans.
struct LinearNumbering {
    /// Position assigned to a block's entry (where its parameters are defined).
    block_start: HashMap<BlockId, u32>,
    /// Position assigned to a block's terminator (its last live-out point).
    block_end: HashMap<BlockId, u32>,
    /// Position assigned to each instruction.
    inst_pos: HashMap<InstId, u32>,
    /// Blocks in reverse postorder, the order positions were assigned in.
    order: Vec<BlockId>,
}

/// Numbers blocks in reverse postorder. Each block reserves one position for
/// its entry/parameters, one per instruction, and one for its terminator, so
/// definitions precede their dominated uses on the linear axis.
fn number_positions(func: &Function) -> LinearNumbering {
    let order = reverse_postorder(func);
    let mut block_start = HashMap::new();
    let mut block_end = HashMap::new();
    let mut inst_pos = HashMap::new();
    let mut pos = 0u32;

    for &block_id in &order {
        let block = func.block(block_id).expect("ordered block exists");
        block_start.insert(block_id, pos);
        pos += 1;
        for inst_id in &block.instructions {
            inst_pos.insert(*inst_id, pos);
            pos += 1;
        }
        block_end.insert(block_id, pos);
        pos += 1;
    }

    LinearNumbering {
        block_start,
        block_end,
        inst_pos,
        order,
    }
}

/// Builds one contiguous live interval per value defined in a reachable block.
///
/// The start is the value's definition position. The end is the maximum of its
/// use positions (instruction operands and terminator uses) and the terminator
/// position of every block where the value is live-out, so values that stay
/// live across edges and loop back-edges span the intervening positions.
pub fn build_intervals(
    func: &Function,
    liveness: &LivenessInfo,
    extra_safe: &std::collections::HashSet<InstId>,
) -> Vec<LiveInterval> {
    let numbering = number_positions(func);

    let mut starts: HashMap<ValueId, u32> = HashMap::new();
    let mut ends: HashMap<ValueId, u32> = HashMap::new();

    // Seed each value with its definition position. Values defined in blocks
    // unreachable from entry have no position and are skipped entirely.
    for (index, value) in func.values.iter().enumerate() {
        let value_id = ValueId::from_raw(index as u32);
        let def_pos = match &value.def {
            ValueDef::BlockParam { block, .. } => numbering.block_start.get(block).copied(),
            ValueDef::Instruction { inst, .. } => numbering.inst_pos.get(inst).copied(),
        };
        if let Some(def_pos) = def_pos {
            starts.insert(value_id, def_pos);
            ends.insert(value_id, def_pos);
        }
    }

    let mut weights: HashMap<ValueId, u32> = HashMap::new();
    let mut clobbers: Vec<u32> = Vec::new();
    // Clobbers outside cold (`*.slow`) blocks: only these are paid on the common path, so they
    // alone weigh against keeping a value in a caller-saved register (`clobber_count`).
    let mut hot_clobbers: Vec<u32> = Vec::new();
    let ref_slots = ref_cell_slots(func);

    let extend = |value: ValueId, position: u32, ends: &mut HashMap<ValueId, u32>| {
        if let Some(end) = ends.get_mut(&value) {
            if position > *end {
                *end = position;
            }
        }
    };

    for &block_id in &numbering.order {
        let block = func.block(block_id).expect("ordered block exists");
        let cold = block.name.ends_with(".slow");

        for inst_id in &block.instructions {
            let position = numbering.inst_pos[inst_id];
            let inst = func.instruction(*inst_id).expect("valid instruction");
            for operand in &inst.operands {
                extend(*operand, position, &mut ends);
                *weights.entry(*operand).or_insert(0) += 1;
            }
            if !inst_is_volatile_safe(func, inst, &ref_slots) && !extra_safe.contains(inst_id) {
                clobbers.push(position);
                if !cold {
                    hot_clobbers.push(position);
                }
            }
        }

        let terminator_position = numbering.block_end[&block_id];
        if let Some(term) = &block.terminator {
            for value in terminator_uses(term) {
                extend(value, terminator_position, &mut ends);
                *weights.entry(value).or_insert(0) += 1;
            }
            if !terminator_is_volatile_safe(term) {
                clobbers.push(terminator_position);
                if !cold {
                    hot_clobbers.push(terminator_position);
                }
            }
        }
        for value in liveness.live_out_of(block_id) {
            extend(*value, terminator_position, &mut ends);
        }
    }

    clobbers.sort_unstable();
    hot_clobbers.sort_unstable();

    // Block parameters of one block are all written by the same parallel edge copy, so they
    // must never share a register: keep each one live past the block entry, even when it is
    // never read, so a sibling parameter allocated at the same position cannot take it.
    let block_params: std::collections::HashSet<ValueId> = func
        .blocks
        .iter()
        .flat_map(|block| block.params.iter().copied())
        .collect();

    let mut intervals: Vec<LiveInterval> = starts
        .into_iter()
        .map(|(value, start)| {
            let mut end = ends[&value].max(start);
            if block_params.contains(&value) {
                end = end.max(start + 1);
            }
            LiveInterval {
                value,
                ir_type: func.value(value).expect("value exists").ir_type,
                start,
                end,
                weight: weights.get(&value).copied().unwrap_or(0),
                call_free: is_call_free(&clobbers, start, end),
                clobber_count: clobbers_within(&hot_clobbers, start, end),
            }
        })
        .collect();
    intervals.sort_by_key(|iv| (iv.start, iv.value.as_raw()));
    intervals
}

/// Counts the clobber positions in the closed range `[start, end]`.
fn clobbers_within(clobbers: &[u32], start: u32, end: u32) -> u32 {
    let lo = clobbers.partition_point(|&pos| pos < start);
    let hi = clobbers.partition_point(|&pos| pos <= end);
    (hi - lo) as u32
}

/// Returns every clobbering instruction of `func` with its linear position, in position order.
///
/// Uses the same numbering as `build_intervals`, so a position can be compared against
/// interval bounds.
pub fn clobber_instructions(
    func: &Function,
    extra_safe: &std::collections::HashSet<InstId>,
) -> Vec<(u32, InstId)> {
    let numbering = number_positions(func);
    let ref_slots = ref_cell_slots(func);
    let mut out = Vec::new();
    for &block_id in &numbering.order {
        let block = func.block(block_id).expect("ordered block exists");
        for inst_id in &block.instructions {
            let inst = func.instruction(*inst_id).expect("valid instruction");
            if !inst_is_volatile_safe(func, inst, &ref_slots) && !extra_safe.contains(inst_id) {
                out.push((numbering.inst_pos[inst_id], *inst_id));
            }
        }
    }
    out
}

/// Returns true when no clobber position lies in the closed range
/// `[start, end]`.
///
/// Both endpoints are included. A clobber at the definition position (`start`)
/// means the defining instruction is itself a clobber: a call-emitting op such
/// as `Call` stores its result into the value's register and only then runs its
/// trailing argument-cleanup calls, which clobber caller-saved registers and so
/// would corrupt the just-stored result. A clobber at the last-use position
/// (`end`) means a multi-operand clobbering op can call a runtime helper while
/// materializing an earlier operand, clobbering a caller-saved register that
/// still holds this value before it is read. Either case disqualifies the
/// interval from a caller-saved register.
fn is_call_free(clobbers: &[u32], start: u32, end: u32) -> bool {
    // First clobber with position >= start.
    let lo = clobbers.partition_point(|&pos| pos < start);
    // First clobber with position > end.
    let hi = clobbers.partition_point(|&pos| pos <= end);
    lo >= hi
}

/// Precise (hole-aware) liveness on the same linear axis as `build_intervals`.
///
/// Used by the allocator's move coalescing: two values may share a register when one is
/// dead across the other's whole contiguous interval, which the contiguous intervals alone
/// cannot show (a loop-carried value used after the loop spans the whole loop body).
pub struct PreciseLiveness {
    /// Per value, the inclusive position ranges where it is live, one per block at most.
    ranges: HashMap<ValueId, Vec<(u32, u32)>>,
    /// Per block parameter, the predecessor terminator positions that write it and the
    /// argument value each one copies in.
    edge_writes: HashMap<ValueId, Vec<(u32, ValueId)>>,
    /// Block parameter -> the block that defines it.
    param_blocks: HashMap<ValueId, BlockId>,
}

impl PreciseLiveness {
    /// Maps every block parameter to its block.
    pub fn param_blocks(&self) -> &HashMap<ValueId, BlockId> {
        &self.param_blocks
    }

    /// Returns true when `value` is live at some position in the closed range `[lo, hi]`.
    pub fn live_within(&self, value: ValueId, lo: u32, hi: u32) -> bool {
        self.ranges
            .get(&value)
            .map(|ranges| ranges.iter().any(|&(a, b)| b >= lo && a <= hi))
            .unwrap_or(true)
    }

    /// Returns the edge copies that write block parameter `value`, as
    /// `(predecessor terminator position, argument)` pairs.
    pub fn edge_writes(&self, value: ValueId) -> &[(u32, ValueId)] {
        self.edge_writes.get(&value).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Builds hole-aware live ranges for every value, from per-block liveness.
pub fn precise_liveness(func: &Function, liveness: &LivenessInfo) -> PreciseLiveness {
    let numbering = number_positions(func);
    let mut ranges: HashMap<ValueId, Vec<(u32, u32)>> = HashMap::new();
    let mut edge_writes: HashMap<ValueId, Vec<(u32, ValueId)>> = HashMap::new();

    for &block_id in &numbering.order {
        let block = func.block(block_id).expect("ordered block exists");
        let block_start = numbering.block_start[&block_id];
        let block_end = numbering.block_end[&block_id];
        // value -> (first live position in this block, last live position in this block)
        let mut local: HashMap<ValueId, (u32, u32)> = HashMap::new();
        let touch = |local: &mut HashMap<ValueId, (u32, u32)>, value: ValueId, pos: u32| {
            let entry = local.entry(value).or_insert((pos, pos));
            entry.0 = entry.0.min(pos);
            entry.1 = entry.1.max(pos);
        };
        for value in liveness.live_in_of(block_id) {
            touch(&mut local, *value, block_start);
        }
        for param in &block.params {
            touch(&mut local, *param, block_start);
        }
        for inst_id in &block.instructions {
            let position = numbering.inst_pos[inst_id];
            let inst = func.instruction(*inst_id).expect("valid instruction");
            for operand in &inst.operands {
                touch(&mut local, *operand, position);
            }
            if let Some(result) = inst.result {
                touch(&mut local, result, position);
            }
        }
        if let Some(term) = &block.terminator {
            for value in terminator_uses(term) {
                touch(&mut local, value, block_end);
            }
            let mut record = |target: BlockId, args: &[ValueId]| {
                if let Some(target_block) = func.block(target) {
                    for (param, arg) in target_block.params.iter().zip(args) {
                        edge_writes.entry(*param).or_default().push((block_end, *arg));
                    }
                }
            };
            match term {
                crate::ir::Terminator::Br { target, args } => record(*target, args),
                crate::ir::Terminator::CondBr {
                    then_target,
                    then_args,
                    else_target,
                    else_args,
                    ..
                } => {
                    record(*then_target, then_args);
                    record(*else_target, else_args);
                }
                crate::ir::Terminator::Switch {
                    cases,
                    default,
                    default_args,
                    ..
                } => {
                    for case in cases {
                        record(case.target, &case.args);
                    }
                    record(*default, default_args);
                }
                crate::ir::Terminator::GeneratorSuspend {
                    resume, resume_args, ..
                } => record(*resume, resume_args),
                _ => {}
            }
        }
        for value in liveness.live_out_of(block_id) {
            touch(&mut local, *value, block_end);
        }
        for (value, range) in local {
            ranges.entry(value).or_default().push(range);
        }
    }

    let param_blocks = func
        .blocks
        .iter()
        .flat_map(|block| block.params.iter().map(move |param| (*param, block.id)))
        .collect();
    PreciseLiveness {
        ranges,
        edge_writes,
        param_blocks,
    }
}
