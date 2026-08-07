//! Purpose:
//! Defines shared method-call targets, cleanup state, and runtime dispatch enums.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;

/// Resolved method metadata needed to issue a direct method call.
pub(super) struct MethodCallTarget {
    pub(super) impl_class: String,
    pub(super) method_key: String,
    pub(super) dynamic_slot: Option<usize>,
    pub(super) params: Vec<PhpType>,
    pub(super) ref_params: Vec<bool>,
    pub(super) return_ty: PhpType,
    pub(super) by_ref_return: bool,
}

/// Concrete runtime class branch available to a `Mixed` receiver method call.
pub(super) struct MixedMethodCandidate {
    pub(super) class_id: u64,
    pub(super) class_name: String,
    pub(super) target: MethodCallTarget,
}

/// Outgoing call argument state that must be cleaned up after the call returns.
pub(super) struct CallArgMaterialization {
    pub(super) overflow_bytes: usize,
    pub(super) ref_writebacks: Vec<RefArgWriteback>,
    pub(super) cleanup_slots: Vec<CallArgTempCleanup>,
    pub(super) cleanup_bytes: usize,
    pub(super) borrowed_stack_arg_bytes: usize,
}

/// Caller-owned temporary argument that must be released after the call returns.
pub(super) struct CallArgTempCleanup {
    pub(super) param_index: usize,
    pub(super) offset: usize,
    pub(super) ty: PhpType,
}

/// Caller-side stack Mixed cell borrowed by a read-only callee.
pub(super) struct BorrowedStackMixedArg {
    pub(super) param_index: usize,
    pub(super) offset: usize,
    pub(super) source_ty: PhpType,
}

/// A caller-side scalar local boxed into a temporary Mixed by-reference cell.
pub(super) struct RefArgWriteback {
    pub(super) param_index: usize,
    pub(super) source_value: ValueId,
    pub(super) source_slot: LocalSlotId,
    pub(super) source_ty: PhpType,
    pub(super) cell_offset: usize,
}

/// Runtime dispatch path for EIR `RuntimeCall` instructions that mean ArrayAccess indexing.
pub(super) enum ArrayAccessRuntimeDispatch {
    Concrete(String),
    Interface { boxed_receiver: bool },
}

/// Source for the hidden called-class id passed to static method bodies.
pub(super) enum CalledClassIdArg {
    Immediate(u64),
    Local(LocalSlotId),
    ThisObject(LocalSlotId),
}
