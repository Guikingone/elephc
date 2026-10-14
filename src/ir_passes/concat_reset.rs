//! Purpose:
//! Concat scratch-state optimization over EIR functions. Removes or coalesces
//! statement-boundary `concat_reset` instructions when CFG dataflow proves the
//! concat scratch cursor `_concat_off` is already at the frame base or will be
//! reset before any instruction can allocate, read, modify, or observe it.
//!
//! Called from:
//! - The fixed-point pass driver in `crate::ir_passes::driver`.
//!
//! Key details:
//! - Every function, method, closure, main entry, and web handler prologue
//!   captures `_concat_off` into its stack frame's `concat_base_offset`, and
//!   `Op::ConcatReset` restores `_concat_off` to that frame base. Lowering
//!   persists any value that outlives a statement (`Op::StrPersist` / `Op::Acquire`)
//!   before emitting the statement-boundary `Op::ConcatReset`.
//! - Phase 1 (forward clean-state elimination) computes a greatest-fixed-point
//!   `in_clean` / `out_clean` fact over reachable blocks: `_concat_off` equals
//!   `concat_base_offset` at function entry and immediately after `Op::ConcatReset`,
//!   and remains clean across instructions on a strict whitelist of concat-inert
//!   operations. Any `Op::ConcatReset` reached when the state is already clean on
//!   all incoming paths is neutralized to `nop`. This eliminates statement-boundary
//!   resets across scalar/arithmetic/container-only straight-line code and loop
//!   bodies while keeping any needed reset before the loop.
//! - Phase 2 (backward unobserved-reset coalescing) computes a greatest-fixed-point
//!   `in_will_reset` / `out_will_reset` fact: when every path from an earlier
//!   `Op::ConcatReset` reaches a later `Op::ConcatReset` before any instruction
//!   that can allocate, read, modify, or observe concat scratch (or throw/exit),
//!   the earlier reset is coalesced into the later one by neutralizing the earlier
//!   instruction to `nop`.
//! - Exception handler targets named by `Op::TryPushHandler` are seeded as dirty
//!   on entry because `__rt_throw_current` rewinds `_concat_off` and unwinding may
//!   leave an arbitrary scratch offset from inside the `try` region.

use std::collections::HashSet;

use crate::ir::{
    BlockId, DataPool, Effects, Function, Immediate, InstId, Instruction, IrType, LocalSlotId, Op,
    Terminator, ValueId,
};
use crate::types::PhpType;

use super::cfg::{predecessors, reverse_postorder, successors};
use super::driver::IrPass;
use super::rewrite::neutralize_to_nop;

/// Concat scratch-state optimization pass.
pub struct ConcatResetElim;

impl IrPass for ConcatResetElim {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "concat-reset"
    }

    /// Skips functions that contain no `concat_reset` instructions.
    fn is_applicable(&self, function: &Function) -> bool {
        function
            .instructions
            .iter()
            .any(|inst| inst.op == Op::ConcatReset)
    }

    /// Eliminates redundant clean-state `concat_reset` instructions and coalesces
    /// unobserved back-to-back resets across basic blocks.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        let mut changed = false;
        changed |= eliminate_clean_concat_resets(function);
        changed |= coalesce_postdominated_concat_resets(function);
        changed
    }
}

/// Collects all block identifiers targeted by `Op::TryPushHandler` tokens.
///
/// Handler blocks are reached via implicit runtime `longjmp` edges from throwing
/// instructions and cannot assume `_concat_off` equals the frame base on entry.
fn collect_handler_blocks(function: &Function) -> HashSet<BlockId> {
    let mut handlers = HashSet::new();
    for inst in &function.instructions {
        if inst.op == Op::TryPushHandler {
            if let Some(Immediate::I64(token)) = inst.immediate {
                if let Ok(raw) = u32::try_from(token) {
                    handlers.insert(BlockId::from_raw(raw));
                }
            }
        }
    }
    handlers
}

/// Computes the set of blocks reachable from `function.entry` or from any installed
/// exception handler block.
fn collect_reachable_blocks(
    function: &Function,
    handlers: &HashSet<BlockId>,
) -> HashSet<BlockId> {
    let mut reachable: HashSet<BlockId> = reverse_postorder(function).into_iter().collect();
    let mut stack: Vec<BlockId> = handlers.iter().copied().collect();
    while let Some(block_id) = stack.pop() {
        if !reachable.insert(block_id) {
            continue;
        }
        if let Some(block) = function.block(block_id) {
            if let Some(term) = &block.terminator {
                for succ in successors(term) {
                    if !reachable.contains(&succ) {
                        stack.push(succ);
                    }
                }
            }
        }
    }
    reachable
}

/// Neutralizes `Op::ConcatReset` instructions that are reached when `_concat_off`
/// is already guaranteed to equal the current frame's `concat_base_offset`.
fn eliminate_clean_concat_resets(function: &mut Function) -> bool {
    let num_blocks = function.blocks.len();
    if num_blocks == 0 {
        return false;
    }

    let handlers = collect_handler_blocks(function);
    let reachable = collect_reachable_blocks(function, &handlers);
    let preds = predecessors(function);

    let mut in_clean = vec![true; num_blocks];
    let mut out_clean = vec![true; num_blocks];
    for idx in 0..num_blocks {
        let block_id = BlockId::from_raw(idx as u32);
        if !reachable.contains(&block_id) || handlers.contains(&block_id) {
            in_clean[idx] = false;
            out_clean[idx] = false;
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for block in &function.blocks {
            let idx = block.id.as_raw() as usize;
            if !reachable.contains(&block.id) {
                continue;
            }

            let new_in = if handlers.contains(&block.id) {
                false
            } else {
                let mut clean = true;
                for &pred in &preds[idx] {
                    if reachable.contains(&pred) && !out_clean[pred.as_raw() as usize] {
                        clean = false;
                        break;
                    }
                }
                clean
            };

            let mut clean = new_in;
            for &inst_id in &block.instructions {
                let inst = function
                    .instruction(inst_id)
                    .expect("block references a valid instruction");
                clean = transfer_forward_clean(function, inst, clean);
            }
            if let Some(term) = &block.terminator {
                if !terminator_preserves_concat_state(term) {
                    clean = false;
                }
            }

            if new_in != in_clean[idx] || clean != out_clean[idx] {
                in_clean[idx] = new_in;
                out_clean[idx] = clean;
                changed = true;
            }
        }
    }

    let mut redundant: Vec<InstId> = Vec::new();
    for block in &function.blocks {
        let idx = block.id.as_raw() as usize;
        if !reachable.contains(&block.id) {
            continue;
        }
        let mut clean = in_clean[idx];
        for &inst_id in &block.instructions {
            let inst = function
                .instruction(inst_id)
                .expect("block references a valid instruction");
            if inst.op == Op::ConcatReset && clean {
                redundant.push(inst_id);
            }
            clean = transfer_forward_clean(function, inst, clean);
        }
    }

    if redundant.is_empty() {
        return false;
    }
    for inst_id in redundant {
        if let Some(inst) = function.instruction_mut(inst_id) {
            neutralize_to_nop(inst);
        }
    }
    true
}

/// Computes the forward clean-state transfer for a single instruction.
fn transfer_forward_clean(function: &Function, inst: &Instruction, clean: bool) -> bool {
    if inst.op == Op::ConcatReset {
        return true;
    }
    if !clean {
        return false;
    }
    instruction_is_concat_inert(function, inst)
}

/// Coalesces an earlier `Op::ConcatReset` into a later `Op::ConcatReset` when
/// every path from the earlier reset reaches a later reset before any instruction
/// can observe or modify `_concat_off`, throw to a handler, or exit the function.
fn coalesce_postdominated_concat_resets(function: &mut Function) -> bool {
    let num_blocks = function.blocks.len();
    if num_blocks == 0 {
        return false;
    }

    let handlers = collect_handler_blocks(function);
    let reachable = collect_reachable_blocks(function, &handlers);

    let mut in_will_reset = vec![false; num_blocks];
    let mut out_will_reset = vec![false; num_blocks];
    for idx in 0..num_blocks {
        let block_id = BlockId::from_raw(idx as u32);
        if reachable.contains(&block_id) {
            in_will_reset[idx] = true;
            out_will_reset[idx] = true;
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for block in function.blocks.iter().rev() {
            let idx = block.id.as_raw() as usize;
            if !reachable.contains(&block.id) {
                continue;
            }

            let new_out = block_out_will_reset(block, &in_will_reset, &reachable);
            let mut will_reset = new_out;
            for &inst_id in block.instructions.iter().rev() {
                let inst = function
                    .instruction(inst_id)
                    .expect("block references a valid instruction");
                will_reset = transfer_backward_will_reset(function, inst, will_reset);
            }

            if new_out != out_will_reset[idx] || will_reset != in_will_reset[idx] {
                out_will_reset[idx] = new_out;
                in_will_reset[idx] = will_reset;
                changed = true;
            }
        }
    }

    let mut redundant: Vec<InstId> = Vec::new();
    for block in &function.blocks {
        let idx = block.id.as_raw() as usize;
        if !reachable.contains(&block.id) {
            continue;
        }
        let mut will_reset = out_will_reset[idx];
        for &inst_id in block.instructions.iter().rev() {
            let inst = function
                .instruction(inst_id)
                .expect("block references a valid instruction");
            if inst.op == Op::ConcatReset && will_reset {
                redundant.push(inst_id);
                // The later reset on all outgoing paths remains active, so
                // `will_reset` stays true for any even earlier reset in the block.
                continue;
            }
            will_reset = transfer_backward_will_reset(function, inst, will_reset);
        }
    }

    if redundant.is_empty() {
        return false;
    }
    for inst_id in redundant {
        if let Some(inst) = function.instruction_mut(inst_id) {
            neutralize_to_nop(inst);
        }
    }
    true
}

/// Computes whether all outgoing edges from `block` are guaranteed to hit a
/// `ConcatReset` before any concat-observing operation or function exit.
fn block_out_will_reset(
    block: &crate::ir::BasicBlock,
    in_will_reset: &[bool],
    reachable: &HashSet<BlockId>,
) -> bool {
    let Some(term) = block.terminator.as_ref() else {
        return false;
    };
    if !terminator_preserves_concat_state(term) {
        return false;
    }
    let succs = successors(term);
    if succs.is_empty() {
        return false;
    }
    let mut saw_reachable_succ = false;
    for succ in succs {
        if reachable.contains(&succ) {
            saw_reachable_succ = true;
            if !in_will_reset[succ.as_raw() as usize] {
                return false;
            }
        }
    }
    saw_reachable_succ
}

/// Computes the backward `will_reset` transfer across a single instruction.
///
/// Any instruction that may throw (`Effects::MAY_THROW`) or fatal (`Effects::MAY_FATAL`)
/// is treated as a barrier even when the current function installs no `try` handler,
/// because unwinding or error reporting can exit the frame before the post-dominating
/// `ConcatReset` is reached.
fn transfer_backward_will_reset(
    function: &Function,
    inst: &Instruction,
    will_reset: bool,
) -> bool {
    if inst.op == Op::ConcatReset {
        return true;
    }
    if !will_reset {
        return false;
    }
    if inst
        .effects
        .intersects(Effects::MAY_THROW | Effects::MAY_FATAL)
    {
        return false;
    }
    instruction_is_concat_inert(function, inst)
}

/// Returns true when a terminator cannot observe or modify `_concat_off` on its
/// outgoing control-flow edges.
fn terminator_preserves_concat_state(term: &Terminator) -> bool {
    matches!(
        term,
        Terminator::Br { .. } | Terminator::CondBr { .. } | Terminator::Switch { .. }
    )
}

/// Returns true when `inst` is guaranteed not to allocate, read, modify, or
/// observe `_concat_off`, nor invoke any callee, destructor, output routine,
/// warning formatter, or `eval` path that can observe or reuse concat scratch.
fn instruction_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    // Handle opcodes whose default effects are conservatively broader than their
    // concrete operand/slot specialization before the generic effect filter.
    match inst.op {
        Op::ReleaseLocalSlot | Op::ReleaseLocalRefCell => {
            return release_slot_is_concat_inert(function, inst);
        }
        Op::Cast => return cast_is_concat_inert(function, inst),
        Op::IterStart
        | Op::IterNext
        | Op::IterCurrentKey
        | Op::IterCurrentValue
        | Op::IterCurrentValueRef
        | Op::IterEnd => return iter_op_is_concat_inert(function, inst),
        _ => {}
    }

    if inst.effects.contains(Effects::ALLOC_CONCAT) || inst.effects.contains(Effects::OUTPUT) {
        return false;
    }

    match inst.op {
        Op::Nop
        | Op::ConstI64
        | Op::ConstF64
        | Op::ConstStr
        | Op::ConstNull
        | Op::ConstBool
        | Op::ConstClassName
        | Op::ConstEnumCase
        | Op::LoadCalledClassId
        | Op::DataAddr
        | Op::IAdd
        | Op::ISub
        | Op::IMul
        | Op::ICheckedAdd
        | Op::ICheckedSub
        | Op::ICheckedMul
        | Op::ICheckedAddToInt
        | Op::ICheckedSubToInt
        | Op::ICheckedMulToInt
        | Op::ICheckedNumericChainToInt
        | Op::ICheckedPow
        | Op::IDiv
        | Op::ISDiv
        | Op::ISMod
        | Op::IPow
        | Op::INeg
        | Op::IBitAnd
        | Op::IBitOr
        | Op::IBitXor
        | Op::IBitNot
        | Op::IShl
        | Op::IShrA
        | Op::FAdd
        | Op::FSub
        | Op::FMul
        | Op::FDiv
        | Op::FPow
        | Op::FNeg
        | Op::ICmp
        | Op::FCmp
        | Op::IToF
        | Op::FToI
        | Op::StrToI
        | Op::StrToF
        | Op::StrLen
        | Op::StrCmp
        | Op::MixedNumericBinop
        | Op::MixedTagOf
        | Op::MixedBox
        | Op::MixedClone
        | Op::MixedUnbox
        | Op::MixedCastBool
        | Op::MixedCastInt
        | Op::MixedCastFloat
        | Op::IsNull
        | Op::TypePredicate
        | Op::InstanceOf
        | Op::Move
        | Op::Borrow
        | Op::StrPersist
        | Op::TryPushHandler
        | Op::TryPopHandler
        | Op::FinallyEnter
        | Op::FinallyExit
        | Op::ErrorSuppressBegin
        | Op::ErrorSuppressEnd
        | Op::PushCallOperandOwner
        | Op::PopCallOperandOwner
        | Op::PromoteLocalRefCell
        | Op::AliasLocalRefCell
        | Op::RetainLocalRefCell
        | Op::BindRefCellPtr
        | Op::AdoptRefCellPtr
        | Op::UnsetLocal
        | Op::ZeroLocalSlot
        | Op::StaticLocalUninitialized
        | Op::StaticPropInitialized
        | Op::ReflectionStaticPropertyInitialized
        | Op::PropInitialized
        | Op::PtrCast
        | Op::PtrOffset
        | Op::PtrCheckNonnull
        | Op::PtrRead
        | Op::PtrWrite
        | Op::BufferNew
        | Op::BufferLen
        | Op::BufferGet
        | Op::BufferSet
        | Op::BufferFree
        | Op::ArrayNew
        | Op::HashNew
        | Op::ArrayLen
        | Op::HashLen
        | Op::ArrayToMixed
        | Op::HashToMixed
        | Op::ArrayCloneShallow
        | Op::HashCloneShallow
        | Op::ArrayEnsureUnique
        | Op::HashEnsureUnique
        | Op::ArrayToHash
        | Op::ArrayIsset
        | Op::HashIsset
        | Op::ArrayGet
        | Op::ArrayGetSilent
        | Op::ArrayGetForWrite
        | Op::HashGet
        | Op::HashGetSilent
        | Op::HashGetForWrite
        | Op::ArrayElemAddr
        | Op::LoadArrayElemRefCell
        | Op::LoadArrayElemRefCellExisting
        | Op::ArrayKeyExists
        | Op::DescriptorArgKeyExists => true,

        Op::LoadLocal | Op::LoadRefCell => load_local_is_concat_inert(function, inst),
        Op::StoreLocal => store_local_is_concat_inert(function, inst),
        Op::Acquire => true,
        Op::Release | Op::ReleaseUnlessAliases => release_is_concat_inert(function, inst),
        Op::IsTruthy => is_truthy_is_concat_inert(function, inst),
        Op::StrictEq | Op::StrictNotEq => strict_eq_is_concat_inert(function, inst),
        Op::ArraySet
        | Op::ArraySetMixedKey
        | Op::ArrayPush
        | Op::MixedArrayAppend
        | Op::HashSet
        | Op::DescriptorArgSet
        | Op::HashAppend
        | Op::SlotDetach => container_write_is_concat_inert(function, inst),

        _ => false,
    }
}

/// Returns the PHP type of `value` in `function`, if present.
fn value_php_type(function: &Function, value: ValueId) -> Option<PhpType> {
    function.value(value).map(|v| v.php_type.codegen_repr())
}

/// Returns the PHP type of `slot` in `function`, if present.
fn slot_php_type(function: &Function, slot: LocalSlotId) -> Option<PhpType> {
    function
        .locals
        .get(slot.as_raw() as usize)
        .map(|local| local.php_type.codegen_repr())
}

/// Returns true when a `LoadLocal` or `LoadRefCell` cannot invoke `__rt_mixed_cast_string`.
///
/// Codegen calls `__rt_mixed_cast_string` only when coercing a `Mixed` local slot into a
/// `PhpType::Str` SSA result.
fn load_local_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    if inst.result_php_type.codegen_repr() != PhpType::Str {
        return true;
    }
    let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
        return false;
    };
    let Some(source_ty) = slot_php_type(function, slot) else {
        return false;
    };
    crate::ir::local_load_types_share_storage(&source_ty, &PhpType::Str)
}

/// Returns true when `Op::StoreLocal` cannot retire a previous ref-cell occupant or
/// invoke `reset_concat_to_frame_base` internally.
///
/// Codegen's `lower_store_local`:
/// 1. Calls `reset_concat_to_frame_base` itself when `inst.span` has `line > 0` and the
///    stored value is `Acquire(StrConcat(...))`. Treating such a store as dirty ensures
///    Phase 2 never coalesces a preceding `ConcatReset` across the `StrConcat` that feeds it.
/// 2. Retires the previous payload when the target slot has `RefCell` or `Dynamic`
///    representation (`store_value_to_ref_cell_local`), which can invoke a destructor if
///    the slot is not statically destructor-free.
fn store_local_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
        return false;
    };
    let Some(&value) = inst.operands.first() else {
        return false;
    };
    if value_is_acquire_of_str_concat(function, value) {
        return false;
    }
    if slot_is_never_ref_cell(function, slot) {
        return true;
    }
    slot_is_statically_destructor_free(
        function,
        slot,
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}

/// Returns true when `value` is `Acquire(StrConcat(...))`.
fn value_is_acquire_of_str_concat(function: &Function, value: ValueId) -> bool {
    let Some(acquire_inst) = super::rewrite::defining_instruction(function, value) else {
        return false;
    };
    if acquire_inst.op != Op::Acquire {
        return false;
    }
    let Some(&source) = acquire_inst.operands.first() else {
        return false;
    };
    super::rewrite::defining_instruction(function, source)
        .is_some_and(|src_inst| src_inst.op == Op::StrConcat)
}

/// Returns true when `slot` is never a by-reference parameter and never promoted or
/// bound to a reference cell anywhere in `function`.
fn slot_is_never_ref_cell(function: &Function, slot: LocalSlotId) -> bool {
    if function
        .params
        .get(slot.as_raw() as usize)
        .is_some_and(|param| param.by_ref)
    {
        return false;
    }
    let Some(local) = function.locals.get(slot.as_raw() as usize) else {
        return false;
    };
    if matches!(
        local.kind,
        crate::ir::LocalKind::RefCell | crate::ir::LocalKind::ReturnRefCell
    ) {
        return false;
    }
    for inst in &function.instructions {
        match (inst.op, inst.immediate.as_ref()) {
            (
                Op::PromoteLocalRefCell
                | Op::AliasLocalRefCell
                | Op::RetainLocalRefCell
                | Op::BindRefCellPtr
                | Op::AdoptRefCellPtr,
                Some(Immediate::LocalSlotPair { first, second }),
            ) if *first == slot || *second == slot => return false,
            (
                Op::BindRefCellPtr
                | Op::AdoptRefCellPtr
                | Op::IterCurrentValueRef
                | Op::StoreRefCell
                | Op::LoadRefCell
                | Op::ReleaseLocalRefCell,
                Some(Immediate::LocalSlot(s)),
            ) if *s == slot => return false,
            (Op::ClosureCapture, Some(Immediate::I64(1))) => {
                if inst.operands.first().is_some_and(|&op| {
                    super::rewrite::defining_instruction(function, op).is_some_and(|load| {
                        matches!(load.op, Op::LoadLocal | Op::LoadRefCell)
                            && load.immediate == Some(Immediate::LocalSlot(slot))
                    })
                }) {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

/// Returns true when `PhpType` is guaranteed never to hold an object with a user `__destruct`
/// or a closure/callable descriptor whose release can drop captured objects.
///
/// `PhpType::Callable` and `PhpType::Resource` are intentionally excluded: releasing a
/// callable descriptor (`__rt_callable_descriptor_release`) decrefs captured payloads or
/// bound `$this` objects (which can run `__destruct`), and `PhpType::Resource` in PHP 8
/// can represent object-backed handles or custom stream wrappers.
fn php_type_is_destructor_free(ty: &PhpType) -> bool {
    match ty {
        PhpType::Int
        | PhpType::Float
        | PhpType::Str
        | PhpType::Bool
        | PhpType::False
        | PhpType::Void
        | PhpType::Never
        | PhpType::TaggedScalar
        | PhpType::Pointer(_)
        | PhpType::Buffer(_)
        | PhpType::Packed(_) => true,
        PhpType::Array(elem) => php_type_is_destructor_free(elem),
        PhpType::AssocArray { key, value } => {
            php_type_is_destructor_free(key) && php_type_is_destructor_free(value)
        }
        PhpType::Union(members) => members.iter().all(php_type_is_destructor_free),
        PhpType::Callable
        | PhpType::Resource(_)
        | PhpType::Object(_)
        | PhpType::Iterable
        | PhpType::Mixed => false,
    }
}

/// Returns true when `Op::Release` or `Op::ReleaseUnlessAliases` cannot invoke a user destructor.
fn release_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    let Some(&value) = inst.operands.first() else {
        return false;
    };
    value_is_statically_destructor_free(
        function,
        value,
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}

/// Returns true when `Op::ReleaseLocalSlot` or `Op::ReleaseLocalRefCell` cannot invoke a user destructor.
fn release_slot_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
        return false;
    };
    slot_is_statically_destructor_free(
        function,
        slot,
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}

/// Returns true when a local slot's stored values are all statically destructor-free.
///
/// Cycles through `visiting_slots` fail closed (`false`) so self-referential local
/// copies (`$x = $x`) without an external non-cyclic base case cannot be proven
/// destructor-free vacuously.
fn slot_is_statically_destructor_free(
    function: &Function,
    slot: LocalSlotId,
    visiting_values: &mut HashSet<ValueId>,
    visiting_slots: &mut HashSet<LocalSlotId>,
) -> bool {
    if super::local_scope::has_dynamic_eval(function) {
        return false;
    }
    let Some(local) = function.locals.get(slot.as_raw() as usize) else {
        return false;
    };
    if php_type_is_destructor_free(&local.php_type) {
        return true;
    }
    // Parameter slots have an initial incoming value from the caller that is not
    // represented by an in-body `StoreLocal`.
    if (slot.as_raw() as usize) < function.params.len() {
        return false;
    }
    if !matches!(
        local.php_type.codegen_repr(),
        PhpType::Mixed | PhpType::Union(_) | PhpType::Array(_) | PhpType::AssocArray { .. }
    ) {
        return false;
    }
    if !visiting_slots.insert(slot) {
        return false;
    }
    let mut saw_store = false;
    let mut all_ok = true;
    for inst in &function.instructions {
        if inst.immediate == Some(Immediate::LocalSlot(slot))
            && !matches!(
                inst.op,
                Op::LoadLocal
                    | Op::StoreLocal
                    | Op::ReleaseLocalSlot
                    | Op::ZeroLocalSlot
                    | Op::PushCallOperandOwner
                    | Op::PopCallOperandOwner
            )
        {
            all_ok = false;
            break;
        }
        if matches!(
            inst.op,
            Op::ArrayPush
                | Op::MixedArrayAppend
                | Op::HashAppend
                | Op::ArraySet
                | Op::ArraySetMixedKey
                | Op::HashSet
        ) && inst.operands.first().is_some_and(|&container| {
            super::rewrite::defining_instruction(function, container).is_some_and(|def| {
                def.op == Op::LoadLocal && def.immediate == Some(Immediate::LocalSlot(slot))
            })
        }) {
            if !inst.operands.iter().skip(1).all(|&op| {
                value_is_statically_destructor_free(
                    function,
                    op,
                    visiting_values,
                    visiting_slots,
                )
            }) {
                all_ok = false;
                break;
            }
            continue;
        }
        if inst.op != Op::StoreLocal || inst.immediate != Some(Immediate::LocalSlot(slot)) {
            continue;
        }
        saw_store = true;
        let Some(&stored_val) = inst.operands.first() else {
            all_ok = false;
            break;
        };
        if !value_is_statically_destructor_free(
            function,
            stored_val,
            visiting_values,
            visiting_slots,
        ) {
            all_ok = false;
            break;
        }
    }
    visiting_slots.remove(&slot);
    all_ok && saw_store
}

/// Returns true when an SSA value is statically guaranteed never to hold an object with a destructor.
///
/// Any cycle in `visiting_values` (such as a loop block-parameter phi cycle) fails closed
/// (`false`) rather than assuming induction holds without a base case.
fn value_is_statically_destructor_free(
    function: &Function,
    value: ValueId,
    visiting_values: &mut HashSet<ValueId>,
    visiting_slots: &mut HashSet<LocalSlotId>,
) -> bool {
    let Some(meta) = function.value(value) else {
        return false;
    };
    if php_type_is_destructor_free(&meta.php_type) {
        return true;
    }
    if !matches!(
        meta.php_type.codegen_repr(),
        PhpType::Mixed | PhpType::Union(_) | PhpType::Array(_) | PhpType::AssocArray { .. }
    ) {
        return false;
    }
    if !visiting_values.insert(value) {
        return false;
    }
    let result = match meta.def {
        crate::ir::ValueDef::Instruction { inst: inst_id, .. } => {
            if let Some(inst) = function.instruction(inst_id) {
                match inst.op {
                    Op::ICheckedAdd
                    | Op::ICheckedSub
                    | Op::ICheckedMul
                    | Op::ICheckedPow
                    | Op::MixedNumericBinop
                    | Op::IterCurrentKey
                    | Op::ConstI64
                    | Op::ConstF64
                    | Op::ConstStr
                    | Op::ConstBool
                    | Op::ConstNull => true,
                    Op::ArrayNew | Op::HashNew => {
                        array_allocation_is_destructor_free(
                            function,
                            value,
                            visiting_values,
                            visiting_slots,
                        )
                    }
                    Op::IterCurrentValue => iter_op_is_concat_inert(function, inst),
                    Op::Acquire
                    | Op::Borrow
                    | Op::Move
                    | Op::Cast
                    | Op::MixedBox
                    | Op::MixedClone
                    | Op::ArrayCloneShallow
                    | Op::HashCloneShallow
                    | Op::ArrayEnsureUnique
                    | Op::HashEnsureUnique
                    | Op::ArrayToMixed
                    | Op::HashToMixed => inst.operands.first().is_some_and(|&src| {
                        value_is_statically_destructor_free(
                            function,
                            src,
                            visiting_values,
                            visiting_slots,
                        )
                    }),
                    Op::LoadLocal => {
                        if let Some(Immediate::LocalSlot(slot)) = inst.immediate {
                            slot_is_statically_destructor_free(
                                function,
                                slot,
                                visiting_values,
                                visiting_slots,
                            )
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        crate::ir::ValueDef::BlockParam { block, index } => {
            // Entry block parameters are function parameters; if their type is Mixed/Union,
            // the caller can pass an arbitrary object.
            if block == function.entry {
                false
            } else {
                let mut saw_incoming = false;
                let mut all_ok = true;
                for pred_block in &function.blocks {
                    let Some(term) = &pred_block.terminator else {
                        continue;
                    };
                    let mut check_edge = |target: BlockId, args: &[ValueId]| -> bool {
                        if target != block {
                            return true;
                        }
                        saw_incoming = true;
                        let Some(&arg) = args.get(index as usize) else {
                            return false;
                        };
                        value_is_statically_destructor_free(
                            function,
                            arg,
                            visiting_values,
                            visiting_slots,
                        )
                    };
                    match term {
                        Terminator::Br { target, args } => {
                            if !check_edge(*target, args) {
                                all_ok = false;
                                break;
                            }
                        }
                        Terminator::CondBr {
                            then_target,
                            then_args,
                            else_target,
                            else_args,
                            ..
                        } => {
                            if !check_edge(*then_target, then_args)
                                || !check_edge(*else_target, else_args)
                            {
                                all_ok = false;
                                break;
                            }
                        }
                        Terminator::Switch {
                            cases,
                            default,
                            default_args,
                            ..
                        } => {
                            for case in cases {
                                if !check_edge(case.target, &case.args) {
                                    all_ok = false;
                                    break;
                                }
                            }
                            if !all_ok || !check_edge(*default, default_args) {
                                all_ok = false;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                all_ok && saw_incoming
            }
        }
    };
    visiting_values.remove(&value);
    result
}

/// Returns true when every in-place write to a newly allocated `ArrayNew` or `HashNew`
/// container `alloc_val` stores a destructor-free value.
fn array_allocation_is_destructor_free(
    function: &Function,
    alloc_val: ValueId,
    visiting_values: &mut HashSet<ValueId>,
    visiting_slots: &mut HashSet<LocalSlotId>,
) -> bool {
    for inst in &function.instructions {
        if matches!(
            inst.op,
            Op::ArrayPush
                | Op::MixedArrayAppend
                | Op::HashAppend
                | Op::ArraySet
                | Op::ArraySetMixedKey
                | Op::HashSet
        ) && inst.operands.first() == Some(&alloc_val)
        {
            for &op in inst.operands.iter().skip(1) {
                if !value_is_statically_destructor_free(
                    function,
                    op,
                    visiting_values,
                    visiting_slots,
                ) {
                    return false;
                }
            }
        }
    }
    true
}

/// Returns true when `Op::IsTruthy` cannot emit a PHP 8.5 `NAN` coercion warning that formats into `_concat_buf`.
fn is_truthy_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    let Some(&value) = inst.operands.first() else {
        return false;
    };
    let Some(ty) = value_php_type(function, value) else {
        return false;
    };
    !matches!(ty, PhpType::Float | PhpType::Mixed | PhpType::Union(_))
}

/// Returns true when `Op::Cast` is an inline scalar cast that never touches `_concat_off`.
fn cast_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    if matches!(inst.immediate, Some(Immediate::StringOffsetCast)) {
        return false;
    }
    let Some(Immediate::CastTarget(target)) = inst.immediate else {
        return false;
    };
    if !matches!(target, IrType::I64 | IrType::F64) {
        return false;
    }
    if inst.result_php_type.codegen_repr() == PhpType::Bool {
        return is_truthy_is_concat_inert(function, inst);
    }
    let Some(&value) = inst.operands.first() else {
        return false;
    };
    let Some(raw_ty) = function.value(value).map(|v| &v.php_type) else {
        return false;
    };
    if matches!(raw_ty, PhpType::Resource(_)) {
        return false;
    }
    // Float-to-int cast (`emit_explicit_float_result_to_int`) can emit the PHP 8.5
    // non-representable float deprecation warning, which formats into `_concat_buf`.
    if target == IrType::I64 && matches!(raw_ty.codegen_repr(), PhpType::Float) {
        return false;
    }
    matches!(
        raw_ty.codegen_repr(),
        PhpType::Int
            | PhpType::Float
            | PhpType::Bool
            | PhpType::False
            | PhpType::Void
            | PhpType::Never
            | PhpType::TaggedScalar
            | PhpType::Str
            | PhpType::Array(_)
            | PhpType::AssocArray { .. }
    )
}

/// Returns true when `Op::StrictEq` / `Op::StrictNotEq` compares scalar, string, object,
/// pointer, or destructor-free `Mixed` operands without invoking a destructor.
///
/// `__rt_mixed_strict_eq` and `__rt_array_strict_eq` are read-only and never touch
/// `_concat_off`. When one operand is `Mixed` and the other is concrete, codegen boxes
/// the concrete operand into a temporary `Mixed` cell and releases that temporary box
/// after the comparison (`decref_mixed_temp_at`), so both operands must be statically
/// destructor-free to guarantee the post-comparison decref cannot run `__destruct`.
fn strict_eq_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    if inst.operands.len() != 2 {
        return false;
    }
    let lhs_val = inst.operands[0];
    let rhs_val = inst.operands[1];
    let (Some(lhs), Some(rhs)) = (
        value_php_type(function, lhs_val),
        value_php_type(function, rhs_val),
    ) else {
        return false;
    };
    let is_simple = |ty: &PhpType| {
        matches!(
            ty,
            PhpType::Int
                | PhpType::Float
                | PhpType::Bool
                | PhpType::False
                | PhpType::Void
                | PhpType::Never
                | PhpType::Str
                | PhpType::Object(_)
                | PhpType::Pointer(_)
        )
    };
    if is_simple(&lhs) && is_simple(&rhs) {
        return true;
    }
    value_is_statically_destructor_free(
        function,
        lhs_val,
        &mut HashSet::new(),
        &mut HashSet::new(),
    ) && value_is_statically_destructor_free(
        function,
        rhs_val,
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}

/// Returns true when an array or hash mutation cannot overwrite an entry holding a destructor-bearing object
/// or coerce a `Mixed` value to `Str`.
fn container_write_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    let Some(&container) = inst.operands.first() else {
        return false;
    };
    let Some(container_meta) = function.value(container) else {
        return false;
    };
    if matches!(inst.op, Op::ArrayPush | Op::MixedArrayAppend | Op::HashAppend) {
        if !matches!(
            &container_meta.php_type,
            PhpType::Array(_) | PhpType::AssocArray { .. }
        ) {
            return false;
        }
    } else if !value_is_statically_destructor_free(
        function,
        container,
        &mut HashSet::new(),
        &mut HashSet::new(),
    ) {
        return false;
    }
    // Check that no value operand is a `Mixed` being coerced into a `Str` container element.
    let container_holds_raw_str = matches!(
        &container_meta.php_type,
        PhpType::Array(elem) if **elem == PhpType::Str
    ) || matches!(
        &container_meta.php_type,
        PhpType::AssocArray { value, .. } if **value == PhpType::Str
    );
    for &operand in inst.operands.iter().skip(1) {
        let Some(op_ty) = value_php_type(function, operand) else {
            return false;
        };
        if container_holds_raw_str && matches!(op_ty, PhpType::Mixed | PhpType::Union(_)) {
            return false;
        }
        if !value_is_statically_destructor_free(
            function,
            operand,
            &mut HashSet::new(),
            &mut HashSet::new(),
        ) {
            return false;
        }
    }
    true
}

/// Returns true when an iterator operation iterates by value over a concrete indexed or
/// associative array with destructor-free elements rather than a user `Iterator` /
/// `IteratorAggregate` object, a dynamic `Mixed` / `Iterable` source, or a by-reference
/// `foreach` that can replace an existing container owner on `IterStart`.
fn iter_op_is_concat_inert(function: &Function, inst: &Instruction) -> bool {
    if inst.op == Op::IterEnd {
        return true;
    }
    let Some(&first_op) = inst.operands.first() else {
        return false;
    };
    let iter_start_inst = if inst.op == Op::IterStart {
        inst
    } else {
        let Some(def_inst) = super::rewrite::defining_instruction(function, first_op) else {
            return false;
        };
        if def_inst.op != Op::IterStart {
            return false;
        }
        def_inst
    };
    let Some(Immediate::IterStart(metadata)) = iter_start_inst.immediate.as_ref() else {
        return false;
    };
    if metadata.is_by_ref() || metadata.owner().is_some() || metadata.origin().is_some() {
        return false;
    }
    let Some(&source_val) = iter_start_inst.operands.first() else {
        return false;
    };
    let Some(source_meta) = function.value(source_val) else {
        return false;
    };
    if !matches!(
        &source_meta.php_type,
        PhpType::Array(_) | PhpType::AssocArray { .. }
    ) {
        return false;
    }
    value_is_statically_destructor_free(
        function,
        source_val,
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}
