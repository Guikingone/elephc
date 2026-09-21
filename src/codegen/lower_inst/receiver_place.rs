//! Purpose:
//! Resolves where a mutating array/hash builtin's by-reference receiver has to be written back,
//! and republishes a possibly-relocated container pointer into that place.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::arrays` (`array_pop`, `array_shift`, `array_unshift`,
//!   `array_splice`, the sort/shuffle family, `array_multisort`, the hash link sorters).
//! - `crate::codegen::lower_inst::hashes` (`hash_set`).
//!
//! Key details:
//! - Every mutating container builtin copy-on-write splits its receiver first, and a split
//!   RELOCATES the storage. So does any growth path that reaches `__rt_array_grow`. The new
//!   pointer has to reach the place the value was READ from, or the caller keeps pointing at
//!   storage that was already freed.
//! - A plain local is its own frame slot. A by-reference parameter is read with `load_ref_cell`
//!   and must be republished through that slot's ref-cell representation
//!   (`store_value_through_ref_cell_slot`), which is exactly what a bare `store_value_to_local`
//!   on a raw frame slot would skip. A direct declared-property receiver preserves the object SSA
//!   value through ownership and packed-to-hash transitions and republishes into its fixed slot.

use crate::codegen::context::FunctionContext;
use crate::codegen::{CodegenIrError, Result};
use crate::ir::{Immediate, LocalSlotId, Op, ValueDef, ValueId};
use crate::types::PhpType;

/// Where a mutating container builtin's receiver has to be written back.
#[derive(Clone)]
pub(super) enum ReceiverPlace {
    /// The receiver was not loaded from a slot this lowering can write back to.
    Opaque,
    /// A plain local frame slot.
    Local(LocalSlotId),
    /// A slot whose value is reached through its ref-cell representation.
    RefCell(LocalSlotId),
    /// A function `static`, whose storage is a `.comm` symbol rather than a frame slot.
    StaticLocal(LocalSlotId),
    /// A declared object property whose runtime container owner may be replaced by COW.
    Property {
        object: ValueId,
        slot: super::objects::PropertySlot,
    },
}

impl ReceiverPlace {
    /// Resolves the writable place a receiver value was loaded from, if any.
    ///
    /// Local loads resolve directly. A declared-property read also resolves through the explicit
    /// retain and optional packed-to-hash conversion emitted by the direct `krsort` write context.
    /// Calls, globals, dynamic properties, and arbitrary expressions remain opaque.
    pub(super) fn resolve(ctx: &FunctionContext<'_>, value: ValueId) -> Result<Self> {
        let Some(value_ref) = ctx.function.value(value) else {
            return Err(CodegenIrError::missing_entry("value", value.as_raw()));
        };
        let ValueDef::Instruction { inst, .. } = value_ref.def else {
            return Ok(Self::Opaque);
        };
        let Some(inst_ref) = ctx.function.instruction(inst) else {
            return Err(CodegenIrError::missing_entry("instruction", inst.as_raw()));
        };
        if let Some(Immediate::LocalSlot(slot)) = inst_ref.immediate {
            return match inst_ref.op {
                Op::LoadLocal => Ok(Self::Local(slot)),
                Op::LoadRefCell => Ok(Self::RefCell(slot)),
                // A function `static` is a writable place too, just not a frame one. Leaving it
                // `Opaque` made every relocating mutation on a `static` array publish its new
                // pointer nowhere — silently, because `Opaque`'s write-back is `Ok(())` and only
                // the growth paths call `require_writable`.
                Op::LoadStaticLocal => Ok(Self::StaticLocal(slot)),
                _ => Ok(Self::Opaque),
            };
        }
        match direct_property_receiver(ctx, value)? {
            Some((object, slot)) => Ok(Self::Property { object, slot }),
            None => Ok(Self::Opaque),
        }
    }

    /// Returns the resolved slot, whichever representation it is reached through.
    ///
    /// Used by the pre-mutation bookkeeping (`release_mutated_source_local_owner`) that only
    /// needs to name the slot; the representation choice belongs to the write-back.
    pub(super) fn slot(&self) -> Option<LocalSlotId> {
        match self {
            Self::Opaque => None,
            Self::Local(slot) | Self::RefCell(slot) => Some(*slot),
            // NOT a frame slot. The pre-mutation bookkeeping this feeds
            // (`release_mutated_source_local_owner`) releases the FRAME slot's occupant, and a
            // `static` local has none — its owner lives in the `.comm` symbol and is released
            // by `__rt_web_reset`, not per call.
            Self::StaticLocal(_) => None,
            Self::Property { .. } => None,
        }
    }

    /// Rejects a receiver this lowering could not resolve to a writable slot.
    ///
    /// Only calls that RELOCATE the receiver need this: a mutation that stays inside the existing
    /// payload is correct even for a receiver whose place is opaque. A growth reallocates, and a
    /// grown container lives somewhere else, so a receiver with nowhere to publish the new pointer
    /// must be refused instead of silently dropping the mutation.
    pub(super) fn require_writable(&self, what: &str) -> Result<()> {
        match self {
            Self::Opaque => Err(CodegenIrError::unsupported(format!(
                "{} for a by-reference receiver that is not a local variable slot",
                what
            ))),
            _ => Ok(()),
        }
    }

    /// Publishes the receiver's current pointer back into the place it was read from, through a
    /// ref cell TYPED at the value being published.
    ///
    /// `array_splice` is the only caller: its replacement insert can retype the receiver's
    /// elements, and `store_value_through_ref_cell_slot` is what refuses that against a
    /// by-reference parameter's declared cell instead of writing a representation the cell cannot
    /// hold (`test_array_splice_type_changing_replacement_on_by_ref_parameter_is_refused`).
    ///
    /// The LOCAL arm goes through the mutated-container path all the other mutating builtins use.
    /// It used to be the ordinary local store, whose boxing path TRANSFERS the SSA value's
    /// reference into the new Mixed cell — while that same value stays live for the splice call
    /// and is released again by the ordinary cleanup. A frame slot a LATER store had widened to
    /// boxed `Mixed` therefore held a cell whose payload had already been freed:
    /// `$c = ['bb', 'aa']; array_splice($c, 0, 1); var_dump($c); $c = null;` printed
    /// `array(0) {}` where php prints the one remaining element, and then looped forever inside
    /// the runtime. The slot only ends up boxed when a later store widens it, which is why the
    /// indexed path looked correct on its own.
    pub(super) fn store_back(
        &self,
        ctx: &mut FunctionContext<'_>,
        value: ValueId,
        value_php_type: &PhpType,
    ) -> Result<()> {
        match self {
            Self::Opaque => Ok(()),
            Self::Local(slot) => ctx.store_mutated_container_to_local(*slot, value),
            Self::StaticLocal(slot) => {
                ctx.store_relocated_container_to_static_local(*slot, value)
            }
            Self::RefCell(slot) => super::store_value_through_ref_cell_slot(
                ctx,
                *slot,
                value,
                value_php_type,
            ),
            Self::Property { object, slot } => {
                super::objects::store_mutated_container_property_owner(ctx, *object, slot, value)
            }
        }
    }

    /// Publishes a mutated container without consuming its SSA owner.
    ///
    /// A copy-on-write helper can return a relocated array/hash while the current SSA value still
    /// owns the reference that the call-cleanup path will release. Storing through the dedicated
    /// container path retains a Mixed local's replacement instead of transferring that owner and
    /// leaving the later cleanup with a dangling raw pointer.
    pub(super) fn store_back_value(
        &self,
        ctx: &mut FunctionContext<'_>,
        value: ValueId,
    ) -> Result<()> {
        match self {
            Self::Opaque => Ok(()),
            Self::Local(slot) | Self::RefCell(slot) => {
                ctx.store_mutated_container_to_local(*slot, value)
            }
            Self::StaticLocal(slot) => {
                ctx.store_relocated_container_to_static_local(*slot, value)
            }
            // The property path already publishes the owner without consuming the SSA value, so
            // it is the same call `store_back` makes — the distinction the two methods draw is
            // about LOCAL slots, where transferring the owner would strand the cleanup.
            Self::Property { object, slot } => {
                super::objects::store_mutated_container_property_owner(ctx, *object, slot, value)
            }
        }
    }
}

/// Finds a declared property behind the direct sort path's transparent value transitions.
///
/// `Acquire` owns the borrowed property payload during conversion, and `ArrayToHash` changes only
/// its physical representation. Neither changes the PHP lvalue that must receive the final COW
/// pointer, so both are peeled until the originating `PropGet` is reached.
fn direct_property_receiver(
    ctx: &FunctionContext<'_>,
    mut value: ValueId,
) -> Result<Option<(ValueId, super::objects::PropertySlot)>> {
    loop {
        let Some(value_ref) = ctx.function.value(value) else {
            return Err(CodegenIrError::missing_entry("value", value.as_raw()));
        };
        let ValueDef::Instruction { inst, .. } = value_ref.def else {
            return Ok(None);
        };
        let Some(inst_ref) = ctx.function.instruction(inst) else {
            return Err(CodegenIrError::missing_entry("instruction", inst.as_raw()));
        };
        match inst_ref.op {
            Op::Acquire | Op::ArrayToHash => {
                let Some(source) = inst_ref.operands.first().copied() else {
                    return Ok(None);
                };
                value = source;
            }
            Op::PropGet => {
                let Some(object) = inst_ref.operands.first().copied() else {
                    return Ok(None);
                };
                let Ok(slot) =
                    super::objects::resolve_mutated_container_property(ctx, object, inst_ref)
                else {
                    return Ok(None);
                };
                return Ok(Some((object, slot)));
            }
            _ => return Ok(None),
        }
    }
}
