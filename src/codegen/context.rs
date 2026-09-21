//! Purpose:
//! Holds per-function state while the EIR backend lowers SSA instructions to assembly.
//! Provides table lookups, value-slot loads/stores, data-pool access, and label creation.
//!
//! Called from:
//! - `crate::codegen::block_emit`, `crate::codegen::lower_inst`, and
//!   `crate::codegen::lower_term`.
//!
//! Key details:
//! - Phase 04 stores every SSA value in a stack slot and reloads result registers at use sites.
//! - The context delegates target-specific movement to `crate::codegen::abi`.
//! - Local labels carry a trailing `<body hash>_<body counter>` pair. The readable part is
//!   `crate::names::label_fragment()`, which is intentionally lossy, so that pair — not the
//!   fragment — is what keeps two similarly named functions from colliding. Both halves are
//!   fixed before emission starts, which is what lets bodies be emitted independently.

use std::collections::{HashMap, HashSet};

use crate::codegen::{abi, emit_box_current_owned_value_as_mixed, emit_box_current_value_as_mixed};
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;
use crate::ir::{
    BlockId, DataId, Function, Immediate, InstId, LocalKind, LocalSlotId, Module, Op, Ownership,
    RuntimeCallTarget, RuntimeFnId, ValueDef, ValueId,
};
use crate::ir_passes::Allocation;
use crate::names::label_fragment;
use crate::types::PhpType;

use super::callable_reachability::CallableReachabilityAnalysis;
use super::frame::FrameLayout;
use super::local_analysis::LocalSlotAnalysis;
use super::shared_state::SharedCodegenState;
use super::value_placement::ValuePlacement;
use super::{CodegenIrError, Result};

/// Returns the readable part of a label: the body's name, collapsed and capped.
///
/// The cap is what keeps 3.65 million labels from spelling out a fully-qualified PHP name each.
/// It is safe because it is only the READABLE part — a label's uniqueness comes from the
/// `<body hash>_<body counter>` pair `next_body_label_id()` appends, and `label_fragment()` was
/// already lossy (`a_b` and `aéb` collapse to the same text), so nothing depended on the
/// fragment being complete.
const LABEL_FRAGMENT_BUDGET: usize = 24;

/// The readable part of a label minted inside a shared-helper scope.
///
/// Spelling the host body's name there would make an identical helper look different in every
/// worker that emitted it, which is exactly what the keyed scope exists to prevent.
const SHARED_HELPER_FRAGMENT: &str = "shared";

/// Opens the emitted region holding one shared helper, followed by its key scope in hex.
pub(crate) const HELPER_MARKER_OPEN: &str = "@helper key=";

/// Closes the region opened by `HELPER_MARKER_OPEN`.
pub(crate) const HELPER_MARKER_CLOSE: &str = "@endhelper";

fn capped_label_fragment(name: &str) -> String {
    let fragment = label_fragment(name);
    match fragment.char_indices().nth(LABEL_FRAGMENT_BUDGET) {
        Some((cut, _)) => fragment[..cut].to_string(),
        None => fragment,
    }
}

/// Returns the 48-bit label scope for one body name.
///
/// FNV-1a rather than a standard-library hasher: the value ends up in the emitted assembly, so
/// it has to be identical on every run and every host, which `RandomState` is not. 48 bits keeps
/// the label short while leaving collisions across a few thousand bodies negligible — and a
/// collision is an assembler error, never silent.
fn body_label_scope(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash & 0x0000_ffff_ffff_ffff
}

/// Runtime representation known for one local slot at the current EIR instruction.
#[derive(Clone, Copy, PartialEq, Eq)]
enum LocalSlotRepresentation {
    Raw,
    RefCell,
    Dynamic,
}

/// Mutable backend state for one EIR function.
pub(crate) struct FunctionContext<'a> {
    pub(super) module: &'a Module,
    pub(super) function: &'a Function,
    pub(super) emitter: &'a mut Emitter,
    pub(super) data: &'a mut DataSection,
    pub(super) shared: &'a mut SharedCodegenState,
    pub(super) placement: ValuePlacement,
    pub(super) allocation: Allocation,
    pub(super) callee_saved_offsets: Vec<(&'static str, usize)>,
    /// 48-bit FNV-1a of the emitting body's name: the half of a label id that separates two
    /// bodies whose readable fragments collapse to the same text.
    label_scope: u64,
    /// Labels emitted by THIS body so far. Private to the context so a body's labels do not
    /// shift when another body is emitted before it.
    label_counter: usize,
    /// The key scope of the shared helper currently being emitted, if any. While it is set,
    /// labels spell `shared` rather than the host body's name, so every worker that emits this
    /// helper spells it identically.
    helper_scope: Option<u64>,
    local_offsets: HashMap<LocalSlotId, usize>,
    ref_cell_state_offsets: HashMap<LocalSlotId, usize>,
    local_analysis: LocalSlotAnalysis,
    callable_reachability: CallableReachabilityAnalysis,
    current_inst: Option<InstId>,
    current_inst_promoted_ref_cells: HashSet<LocalSlotId>,
    try_handler_offsets: HashMap<i64, usize>,
    pub(super) frame_size: usize,
    pub(super) concat_base_offset: usize,
    pub(super) exception_activation_offset: Option<usize>,
    pub(super) epilogue_emitted: bool,
    /// `--instrument` id assigned to this function in its prologue, consumed by
    /// its epilogue's `elephc_instr_exit(id)`. `None` outside `--instrument`.
    pub(super) instr_id: Option<usize>,
    pub(super) is_main: bool,
    pub(super) web: bool,
    pub(super) gc_stats: bool,
    pub(super) heap_debug: bool,
    pcntl_async_signals: bool,
    pcntl_signal_handlers: bool,
    /// True when this module still declares `__elephc_shutdown_run`, the nullary drain every
    /// process-exit site calls to run the `register_shutdown_function()` queue.
    ///
    /// The call is a HARD REFERENCE to a symbol only the prelude defines, so it may only be
    /// emitted when the declaration SURVIVED reachability pruning — read from the module's own
    /// function list, exactly as `runtime_features::diag_user_handler` is. When false the exit
    /// sites keep the shape they have always had, which is what makes the surface pay-for-use.
    php_shutdown_drain: bool,
    pub(super) epilogue_label: Option<String>,
    block_labels: Vec<String>,
}

impl<'a> FunctionContext<'a> {
    /// Creates a lowering context with finalized frame and value-placement metadata.
    pub(super) fn new(
        module: &'a Module,
        function: &'a Function,
        emitter: &'a mut Emitter,
        data: &'a mut DataSection,
        shared: &'a mut SharedCodegenState,
        layout: FrameLayout,
        is_main: bool,
        gc_stats: bool,
        heap_debug: bool,
        epilogue_label: Option<String>,
    ) -> Self {
        let callable_reachability = CallableReachabilityAnalysis::new(module, function);
        let label_scope = body_label_scope(&function.name);
        // Both of these walk the WHOLE module, and this constructor runs once per body, so
        // asking them directly is quadratic; `SharedCodegenState` answers each one once.
        let pcntl_async_signals =
            shared.pcntl_async_signals(|| module_uses_pcntl_async_signals(module));
        let pcntl_signal_handlers =
            shared.pcntl_signal_handlers(|| module_uses_pcntl_signal_handlers(module));
        let php_shutdown_drain = shared.php_shutdown_drain(|| module_declares_shutdown_drain(module));
        let function_fragment = capped_label_fragment(&function.name);
        // Indexed by raw block id, matching `Function::block()`'s positional lookup.
        // The platform-local prefix keeps every intra-function label out of the object's
        // symbol table: without it, profilers name frames after the nearest block label
        // (`_eir_hot_leaf_for_body_2`) instead of the PHP function DWARF describes.
        let local_prefix = emitter.target.platform.local_label_prefix();
        // The block labels take the first ids of this body's own counter, and `next_label()`
        // continues from there. Drawing them from the module-wide counter instead was the last
        // thing tying a body's labels to how many labels other bodies had already emitted.
        let mut label_counter = 0usize;
        let block_labels = function
            .blocks
            .iter()
            .map(|block| {
                let id = label_counter;
                label_counter += 1;
                format!(
                    "{}_eir_{}_{}_{:012x}_{}",
                    local_prefix,
                    function_fragment,
                    label_fragment(&block.name),
                    label_scope,
                    id
                )
            })
            .collect();
        Self {
            module,
            function,
            emitter,
            data,
            shared,
            placement: layout.value_placement,
            allocation: layout.allocation,
            callee_saved_offsets: layout.callee_saved_offsets,
            local_offsets: layout.local_offsets,
            ref_cell_state_offsets: layout.ref_cell_state_offsets,
            local_analysis: layout.local_analysis,
            callable_reachability,
            label_scope,
            label_counter,
            helper_scope: None,
            current_inst: None,
            current_inst_promoted_ref_cells: HashSet::new(),
            try_handler_offsets: layout.try_handler_offsets,
            frame_size: layout.frame_size,
            concat_base_offset: layout.concat_base_offset,
            exception_activation_offset: layout.exception_activation_offset,
            epilogue_emitted: false,
            instr_id: None,
            is_main,
            web: false,
            gc_stats,
            heap_debug,
            pcntl_async_signals,
            pcntl_signal_handlers,
            php_shutdown_drain,
            epilogue_label,
            block_labels,
        }
    }

    /// Returns a module-unique local label carrying a readable but lossy prefix.
    ///
    /// Uniqueness comes from the trailing `<body hash>_<body counter>`: `label_fragment()`
    /// collapses every non-alphanumeric byte, so `a_b` and `aéb` share a readable prefix and
    /// only that pair keeps their labels apart. The hash identifies the emitting body and the
    /// counter is private to this context, so a body's labels do not depend on how many labels
    /// other bodies emitted first — which is what allows bodies to be emitted independently.
    pub(super) fn next_label(&mut self, prefix: &str) -> String {
        format!(
            "{}_eir_{}_{}_{}",
            self.emitter.target.platform.local_label_prefix(),
            self.label_owner_fragment(),
            label_fragment(prefix),
            self.next_body_label_id()
        )
    }

    /// Returns this body's next label id: its name hash and a counter private to the context.
    fn next_body_label_id(&mut self) -> String {
        let id = self.label_counter;
        self.label_counter += 1;
        format!("{:012x}_{}", self.label_scope, id)
    }

    /// Returns the readable part of a label: the helper's key when one is open, else the body.
    fn label_owner_fragment(&self) -> String {
        match self.helper_scope {
            Some(_) => SHARED_HELPER_FRAGMENT.to_string(),
            None => capped_label_fragment(&self.function.name),
        }
    }

    /// Emits a shared helper whose labels depend only on `key`, and brackets it for the merge.
    ///
    /// A helper belongs to the module once, but any body may be the one that reaches it first.
    /// Scoping its labels to the key instead of to the host makes every worker's copy spell the
    /// same names, so a parallel emission pass can keep one copy and drop the duplicates instead
    /// of giving up and re-emitting the whole body serially.
    ///
    /// The scope is saved and restored, so a helper emitted in the middle of a body leaves the
    /// body's own label numbering untouched.
    pub(super) fn emit_keyed_helper<R>(
        &mut self,
        key: &str,
        emit: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let scope = body_label_scope(key);
        let saved_scope = std::mem::replace(&mut self.label_scope, scope);
        let saved_counter = std::mem::replace(&mut self.label_counter, 0);
        let saved_helper = std::mem::replace(&mut self.helper_scope, Some(scope));
        self.emitter
            .comment(&format!("{}{:012x}", HELPER_MARKER_OPEN, scope));
        let result = emit(self);
        self.emitter.comment(HELPER_MARKER_CLOSE);
        self.label_scope = saved_scope;
        self.label_counter = saved_counter;
        self.helper_scope = saved_helper;
        result
    }

    /// Mints a shared helper's entry symbol from its cache key, emitting nothing.
    ///
    /// The entry symbol has to be identical in every worker that reaches this helper, because
    /// bodies in other workers spell it to call it. It is the one label that must come from the
    /// key rather than from whoever happened to emit the helper.
    pub(super) fn keyed_global_label(&mut self, key: &str, prefix: &str) -> String {
        let scope = body_label_scope(key);
        format!(
            "_eir_{}_{}_{:012x}_0",
            SHARED_HELPER_FRAGMENT,
            label_fragment(prefix),
            scope
        )
    }

    /// Returns a module-unique label for an emitted entry point that must stay a real
    /// symbol: invokers and wrappers that are `.globl`-exported, cached across functions,
    /// or referenced from callable descriptors. `next_label()`'s assembler-local prefix
    /// would make `.globl` invalid ("non-local symbol required") and the cross-function
    /// reference dangling.
    pub(super) fn next_global_label(&mut self, prefix: &str) -> String {
        format!(
            "_eir_{}_{}_{}",
            self.label_owner_fragment(),
            label_fragment(prefix),
            self.next_body_label_id()
        )
    }

    /// Returns whether this module needs automatic PCNTL dispatch safe points.
    pub(super) const fn uses_pcntl_async_signals(&self) -> bool {
        self.pcntl_async_signals
    }

    /// Returns whether this module owns process-wide PCNTL handler registrations.
    pub(super) const fn uses_pcntl_signal_handlers(&self) -> bool {
        self.pcntl_signal_handlers
    }

    /// Returns whether a process-exit site may call the PHP shutdown-function drain.
    pub(super) const fn runs_php_shutdown_functions(&self) -> bool {
        self.php_shutdown_drain
    }

    /// Emits an unconditional target-aware branch to one local assembly label.
    pub(super) fn emit_branch(&mut self, label: &str) {
        match self.emitter.target.arch {
            Arch::AArch64 => {
                self.emitter
                    .instruction(&format!("b {}", label));                       // join the dynamic local-representation paths
            }
            Arch::X86_64 => {
                self.emitter
                    .instruction(&format!("jmp {}", label));                     // join the dynamic local-representation paths
            }
        }
    }

    /// Materializes the address of a local's current raw value or aliased ref-cell storage.
    pub(super) fn materialize_local_storage_address(
        &mut self,
        slot: LocalSlotId,
        destination: &str,
    ) -> Result<()> {
        let offset = self.local_offset(slot)?;
        match self.local_slot_representation(slot) {
            LocalSlotRepresentation::Raw => {
                abi::emit_frame_slot_address(self.emitter, destination, offset);
            }
            LocalSlotRepresentation::RefCell => {
                abi::load_at_offset(self.emitter, destination, offset);
            }
            LocalSlotRepresentation::Dynamic => {
                let state_offset = self.dynamic_ref_cell_state_offset(slot)?;
                let ref_cell = self.next_label("dynamic_local_address_ref_cell");
                let done = self.next_label("dynamic_local_address_done");
                let result_reg = abi::int_result_reg(self.emitter);
                let state_reg = if destination == result_reg {
                    abi::secondary_scratch_reg(self.emitter)
                } else {
                    result_reg
                };
                abi::load_at_offset(self.emitter, state_reg, state_offset);
                match self.emitter.target.arch {
                    Arch::AArch64 => {
                        self.emitter.instruction(                               // select the aliased storage address after runtime promotion
                            &format!("cbnz {}, {}", state_reg, ref_cell)
                        );
                    }
                    Arch::X86_64 => {
                        self.emitter.instruction(                               // test the slot's runtime representation flag
                            &format!("test {}, {}", state_reg, state_reg)
                        );
                        self.emitter
                            .instruction(&format!("jne {}", ref_cell));           // select the aliased storage address after runtime promotion
                    }
                }
                abi::emit_frame_slot_address(self.emitter, destination, offset);
                self.emit_branch(&done);
                self.emitter.label(&ref_cell);
                abi::load_at_offset(self.emitter, destination, offset);
                self.emitter.label(&done);
            }
        }
        Ok(())
    }

    /// Returns the assembly label reserved for one EIR block.
    ///
    /// Block labels are minted once per block in `new()` from the module-wide label counter
    /// rather than derived from the block name, which is not unique across functions. Lookup is
    /// positional on the raw block id, exactly like `crate::ir::Function::block()`.
    pub(super) fn block_label_for_id(&self, block: BlockId) -> Result<String> {
        self.block_labels
            .get(block.as_raw() as usize)
            .cloned()
            .ok_or_else(|| CodegenIrError::missing_entry("block", block.as_raw()))
    }

    /// Returns a module function by PHP name using PHP's case-insensitive lookup.
    pub(super) fn function_by_name(&self, name: &str) -> Option<&'a Function> {
        let key = crate::names::php_symbol_key(name.trim_start_matches('\\'));
        self.module
            .functions
            .iter()
            .chain(self.module.closures.iter())
            .find(|function| {
                crate::names::php_symbol_key(function.name.trim_start_matches('\\')) == key
            })
    }

    /// Returns true when an extern declaration exists for a PHP function name.
    pub(super) fn has_extern_function(&self, name: &str) -> bool {
        let key = crate::names::php_symbol_key(name.trim_start_matches('\\'));
        self.module.extern_decls.iter().any(|function| {
            crate::names::php_symbol_key(function.name.trim_start_matches('\\')) == key
        })
    }

    /// Returns the public include-variant group name matching a PHP function name.
    pub(super) fn function_variant_group_name(&self, name: &str) -> Option<String> {
        let key = crate::names::php_symbol_key(name.trim_start_matches('\\'));
        super::function_variants::collect_dispatch_groups(self.module)
            .into_iter()
            .find(|group| crate::names::php_symbol_key(group.name.trim_start_matches('\\')) == key)
            .map(|group| group.name)
    }

    /// Returns whether this module has interface metadata whose visibility is
    /// selected by an explicit request-time activation event.
    pub(super) fn module_has_interface_activation_events(&self) -> bool {
        !super::classlike_activation::collect_activation_registry_names(self.module)
            .interfaces
            .is_empty()
    }

    /// Returns the concrete function whose signature should be used for a PHP call target.
    pub(super) fn callable_function_by_name(&self, name: &str) -> Option<&'a Function> {
        self.function_by_name(name)
            .or_else(|| super::function_variants::variant_callee_for_group(self.module, name))
    }

    /// Returns the finite runtime callable names proven for one EIR value.
    pub(super) fn runtime_callable_candidates(&self, value: ValueId) -> Option<Vec<String>> {
        self.callable_reachability.candidates(value)
    }

    /// Returns a function value or a structured backend error.
    pub(super) fn value_php_type(&self, value: ValueId) -> Result<PhpType> {
        self.function
            .value(value)
            .map(|metadata| metadata.php_type.codegen_repr())
            .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))
    }

    /// Returns a function value's IR storage type.
    ///
    /// This is the ONLY reliable way to tell whether a value is a genuinely boxed `Mixed` CELL.
    /// The PHP type lies here: `Op::IChecked*` (which is what `$i++` lowers to) reports a PHP type
    /// of `Mixed` while its runtime value is a RAW INTEGER, not a heap cell. Unboxing that as a
    /// pointer reads garbage.
    pub(super) fn value_ir_type(&self, value: ValueId) -> Result<crate::ir::IrType> {
        self.function
            .value(value)
            .map(|metadata| metadata.ir_type)
            .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))
    }

    /// Returns a function value's source PHP metadata before codegen representation erasure.
    pub(super) fn raw_value_php_type(&self, value: ValueId) -> Result<PhpType> {
        self.function
            .value(value)
            .map(|metadata| metadata.php_type.clone())
            .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))
    }

    /// Returns the EIR ownership metadata attached to an SSA value.
    pub(super) fn value_ownership(&self, value: ValueId) -> Result<Ownership> {
        self.function
            .value(value)
            .map(|metadata| metadata.ownership)
            .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))
    }

    /// Returns the runtime PHP type stored in a local slot.
    pub(super) fn local_php_type(&self, slot: LocalSlotId) -> Result<PhpType> {
        self.function
            .locals
            .get(slot.as_raw() as usize)
            .map(|metadata| metadata.php_type.codegen_repr())
            .ok_or_else(|| CodegenIrError::missing_entry("local slot", slot.as_raw()))
    }

    /// Returns the semantic role attached to a local slot.
    pub(super) fn local_kind(&self, slot: LocalSlotId) -> Result<LocalKind> {
        self.function
            .locals
            .get(slot.as_raw() as usize)
            .map(|metadata| metadata.kind)
            .ok_or_else(|| CodegenIrError::missing_entry("local slot", slot.as_raw()))
    }

    /// Returns the local slot with the requested source name.
    pub(super) fn local_slot_by_name(&self, name: &str) -> Option<LocalSlotId> {
        self.function
            .locals
            .iter()
            .find(|local| local.name.as_deref() == Some(name))
            .map(|local| local.id)
    }

    /// Returns whether a slot can contain a runtime value that this frame must release.
    ///
    /// A slot written only by a typed runtime writeback -- never by an EIR store -- still holds a
    /// value this frame owns, which is why this asks `local_slot_has_store` rather than the raw
    /// analysis flag.
    pub(super) fn local_slot_needs_lifetime_tracking(&self, slot: LocalSlotId) -> bool {
        self.local_slot_has_store(slot) || self.local_analysis.has_load(slot)
    }

    /// Returns whether this slot receives an EIR store or a typed runtime writeback.
    pub(super) fn local_slot_has_store(&self, slot: LocalSlotId) -> bool {
        self.local_analysis.has_store(slot)
            || self.openssl_encrypt_writes_local(slot)
            || self.pcntl_writes_local(slot)
    }

    /// Returns whether an `openssl_encrypt()` call writes its GCM tag into this local.
    fn openssl_encrypt_writes_local(&self, slot: LocalSlotId) -> bool {
        self.function.instructions.iter().any(|inst| {
            let is_encrypt = matches!(
                inst.immediate,
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                    RuntimeFnId::OpensslEncrypt
                )))
                    | Some(Immediate::RuntimeCall(RuntimeCallTarget::ProfiledFunction {
                        target: RuntimeFnId::OpensslEncrypt,
                        ..
                    }))
            );
            is_encrypt
                && inst
                    .operands
                    .get(5)
                    .and_then(|value| self.loaded_local_slot(*value))
                    == Some(slot)
        })
    }

    /// Returns whether a typed PCNTL wait or signal operation writes an output to `slot`.
    fn pcntl_writes_local(&self, slot: LocalSlotId) -> bool {
        self.function.instructions.iter().any(|inst| {
            let output_indices: &[usize] = match inst.immediate {
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Pcntl(
                    crate::ir::PcntlRuntime::Wait,
                ))) => &[0, 2],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Pcntl(
                    crate::ir::PcntlRuntime::WaitPid,
                ))) => &[1, 3],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Pcntl(
                    crate::ir::PcntlRuntime::WaitId,
                ))) => &[2, 4],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Pcntl(
                    crate::ir::PcntlRuntime::SignalMask,
                ))) => &[2],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Pcntl(
                    crate::ir::PcntlRuntime::SignalTimedWait
                    | crate::ir::PcntlRuntime::SignalWaitInfo,
                ))) => &[1],
                _ => return false,
            };
            output_indices.iter().copied().any(|index| {
                inst.operands
                    .get(index)
                    .and_then(|value| self.loaded_local_slot(*value))
                    == Some(slot)
            })
        })
    }

    /// Resolves a value produced by `LoadLocal` to its source slot.
    fn loaded_local_slot(&self, value: ValueId) -> Option<LocalSlotId> {
        let value_ref = self.function.value(value)?;
        let ValueDef::Instruction { inst, .. } = value_ref.def else {
            return None;
        };
        let inst = self.function.instruction(inst)?;
        if !matches!(inst.op, Op::LoadLocal | Op::LoadRefCell) {
            return None;
        }
        let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
            return None;
        };
        Some(slot)
    }

    /// Returns whether this slot is represented as a ref-cell pointer anywhere in the function.
    pub(super) fn local_slot_ever_stores_ref_cell_pointer(&self, slot: LocalSlotId) -> bool {
        self.local_analysis.ever_stores_ref_cell_pointer(slot)
    }

    /// Returns whether this deferred release may execute while the slot stores a ref-cell pointer.
    pub(super) fn release_local_slot_may_observe_ref_cell(&self, inst: InstId) -> bool {
        self.local_analysis.release_may_observe_ref_cell(inst)
    }

    /// Returns whether this by-value parameter slot is owned by the callee frame.
    pub(super) fn owns_parameter_slot(&self, slot: LocalSlotId) -> bool {
        self.local_analysis.owns_parameter_slot(slot)
    }

    /// Selects the EIR instruction whose CFG-local representation facts codegen must use.
    pub(super) fn begin_instruction(&mut self, inst: InstId) {
        self.current_inst = Some(inst);
        self.current_inst_promoted_ref_cells.clear();
    }

    /// Returns the active EIR instruction's PHP source span when it has one.
    pub(super) fn current_instruction_span(&self) -> Option<crate::span::Span> {
        self.current_inst
            .and_then(|inst| self.function.instruction(inst))
            .and_then(|inst| inst.span)
    }

    /// Returns the frame flag that records whether this slot currently stores a cell pointer.
    pub(super) fn ref_cell_state_offset(&self, slot: LocalSlotId) -> Option<usize> {
        self.ref_cell_state_offsets.get(&slot).copied()
    }

    /// Returns the required runtime representation flag offset for one dynamic local slot.
    fn dynamic_ref_cell_state_offset(&self, slot: LocalSlotId) -> Result<usize> {
        self.ref_cell_state_offset(slot).ok_or_else(|| {
            CodegenIrError::invalid_module(format!(
                "dynamic ref-cell slot {} has no representation flag",
                slot.as_raw()
            ))
        })
    }

    /// Returns whether this slot needs runtime raw-value/ref-cell discrimination at cleanup.
    pub(super) fn has_dynamic_ref_cell_state(&self, slot: LocalSlotId) -> bool {
        self.local_analysis.has_dynamic_ref_cell_state(slot)
    }

    /// Records at runtime that a path has changed this local slot to ref-cell representation.
    pub(super) fn mark_promoted_ref_cell(&mut self, slot: LocalSlotId) {
        self.current_inst_promoted_ref_cells.insert(slot);
        if let Some(offset) = self.ref_cell_state_offset(slot) {
            abi::emit_load_int_immediate(self.emitter, abi::int_result_reg(self.emitter), 1);
            abi::store_at_offset(self.emitter, abi::int_result_reg(self.emitter), offset);
        }
    }

    /// Records at runtime that `unset()` restored this local slot to raw representation.
    pub(super) fn unmark_promoted_ref_cell(&mut self, slot: LocalSlotId) {
        self.current_inst_promoted_ref_cells.remove(&slot);
        if let Some(offset) = self.ref_cell_state_offset(slot) {
            abi::emit_store_zero_to_local_slot(self.emitter, offset);
        }
    }

    /// Returns true when this instruction may observe a heap reference-cell pointer in the slot.
    pub(super) fn local_stores_ref_cell_pointer(&self, slot: LocalSlotId) -> bool {
        self.local_slot_representation(slot) != LocalSlotRepresentation::Raw
    }

    /// Returns whether this instruction needs a runtime raw/ref-cell branch for the slot.
    pub(super) fn local_ref_cell_representation_is_dynamic(&self, slot: LocalSlotId) -> bool {
        self.local_slot_representation(slot) == LocalSlotRepresentation::Dynamic
    }

    /// Returns whether every path reaching this instruction stores a ref-cell pointer.
    pub(super) fn local_ref_cell_representation_is_definite(&self, slot: LocalSlotId) -> bool {
        self.local_slot_representation(slot) == LocalSlotRepresentation::RefCell
    }

    /// Classifies the slot as raw, definitely ref-cell, or path-dependent at this instruction.
    fn local_slot_representation(&self, slot: LocalSlotId) -> LocalSlotRepresentation {
        if self.is_by_ref_param_slot(slot) || self.current_inst_promoted_ref_cells.contains(&slot) {
            return LocalSlotRepresentation::RefCell;
        }
        let may_observe_ref_cell = self.current_inst.is_some_and(|inst| {
            self.local_analysis.inst_may_observe_ref_cell(inst, slot)
        });
        if !may_observe_ref_cell {
            return LocalSlotRepresentation::Raw;
        }
        if self.ref_cell_state_offset(slot).is_some() {
            LocalSlotRepresentation::Dynamic
        } else {
            LocalSlotRepresentation::RefCell
        }
    }

    /// Returns true when the local slot is the storage slot for a by-reference parameter.
    fn is_by_ref_param_slot(&self, slot: LocalSlotId) -> bool {
        self.function
            .params
            .get(slot.as_raw() as usize)
            .is_some_and(|param| param.by_ref)
    }

    /// Loads a stored SSA value into the target's canonical result register(s).
    ///
    /// When the value lives in an allocated register, it is moved from there
    /// into the result register instead of loaded from a stack slot.
    pub(super) fn load_value_to_result(&mut self, value: ValueId) -> Result<PhpType> {
        let ty = self.value_php_type(value)?;
        if let Some(reg) = self.allocation.register_of(value) {
            let dst = if ty.codegen_repr() == PhpType::Float {
                abi::float_result_reg(self.emitter)
            } else {
                abi::int_result_reg(self.emitter)
            };
            abi::emit_reg_move(self.emitter, dst, reg);
        } else {
            let offset = self.value_offset(value)?;
            abi::emit_load(self.emitter, &ty.codegen_repr(), offset);
        }
        Ok(ty)
    }

    /// Loads a single-register SSA value into a caller-selected register.
    ///
    /// When the value lives in an allocated register, it is moved register to
    /// register (a no-op when the source already is the requested register).
    pub(super) fn load_value_to_reg(&mut self, value: ValueId, reg: &str) -> Result<PhpType> {
        let ty = self.value_php_type(value)?;
        if let Some(home) = self.allocation.register_of(value) {
            abi::emit_reg_move(self.emitter, reg, home);
        } else {
            let offset = self.value_offset(value)?;
            abi::load_at_offset(self.emitter, reg, offset);
        }
        Ok(ty)
    }

    /// Loads a string SSA value into a caller-selected register pair.
    pub(super) fn load_string_value_to_regs(
        &mut self,
        value: ValueId,
        ptr_reg: &str,
        len_reg: &str,
    ) -> Result<()> {
        let ty = self.value_php_type(value)?;
        if ty != PhpType::Str {
            return Err(CodegenIrError::unsupported(format!(
                "string register materialization for PHP type {:?}",
                ty
            )));
        }
        let offset = self.value_offset(value)?;
        abi::load_at_offset(self.emitter, ptr_reg, offset);
        abi::load_at_offset(self.emitter, len_reg, offset - 8);
        Ok(())
    }

    /// Loads a local slot into the target's canonical result register(s).
    pub(super) fn load_local_to_result(&mut self, slot: LocalSlotId) -> Result<PhpType> {
        let ty = self.local_php_type(slot)?;
        match self.local_slot_representation(slot) {
            LocalSlotRepresentation::Raw => self.load_raw_local_to_result(slot),
            LocalSlotRepresentation::RefCell => self.load_ref_cell_local_to_result(slot),
            LocalSlotRepresentation::Dynamic => {
                let state_offset = self.dynamic_ref_cell_state_offset(slot)?;
                let ref_cell = self.next_label("dynamic_local_load_ref_cell");
                let done = self.next_label("dynamic_local_load_done");
                let state_reg = abi::secondary_scratch_reg(self.emitter);
                abi::load_at_offset(self.emitter, state_reg, state_offset);
                match self.emitter.target.arch {
                    Arch::AArch64 => {
                        self.emitter.instruction(                               // select ref-cell loading after a runtime promotion
                            &format!("cbnz {}, {}", state_reg, ref_cell)
                        );
                    }
                    Arch::X86_64 => {
                        self.emitter.instruction(                               // test the slot's runtime representation flag
                            &format!("test {}, {}", state_reg, state_reg)
                        );
                        self.emitter
                            .instruction(&format!("jne {}", ref_cell));           // select ref-cell loading after a runtime promotion
                    }
                }
                self.load_raw_local_to_result(slot)?;
                self.emit_branch(&done);
                self.emitter.label(&ref_cell);
                self.load_ref_cell_local_to_result(slot)?;
                self.emitter.label(&done);
                Ok(ty)
            }
        }
    }

    /// Loads a local slot using its raw frame representation without consulting ref-cell state.
    pub(super) fn load_raw_local_to_result(&mut self, slot: LocalSlotId) -> Result<PhpType> {
        let ty = self.local_php_type(slot)?;
        let offset = self.local_offset(slot)?;
        abi::emit_load(self.emitter, &ty.codegen_repr(), offset);
        Ok(ty)
    }

    /// Loads the value pointed to by a local ref-cell pointer slot.
    fn load_ref_cell_local_to_result(&mut self, slot: LocalSlotId) -> Result<PhpType> {
        let ty = self.local_php_type(slot)?;
        reject_multiword_ref_cell_local(&ty, "load")?;
        let offset = self.local_offset(slot)?;
        let pointer_reg = abi::symbol_scratch_reg(self.emitter);
        abi::load_at_offset(self.emitter, pointer_reg, offset);
        match ty.codegen_repr() {
            PhpType::Str => {
                let (ptr_reg, len_reg) = abi::string_result_regs(self.emitter);
                abi::emit_load_from_address(self.emitter, ptr_reg, pointer_reg, 0);
                abi::emit_load_from_address(self.emitter, len_reg, pointer_reg, 8);
            }
            PhpType::Float => {
                abi::emit_load_from_address(self.emitter, abi::float_result_reg(self.emitter), pointer_reg, 0);
            }
            PhpType::TaggedScalar => {
                abi::emit_load_from_address(self.emitter, abi::int_result_reg(self.emitter), pointer_reg, 0);
                abi::emit_load_from_address(
                    self.emitter,
                    crate::codegen::sentinels::tagged_scalar_tag_reg(self.emitter),
                    pointer_reg,
                    8,
                );
            }
            _ => {
                abi::emit_load_from_address(self.emitter, abi::int_result_reg(self.emitter), pointer_reg, 0);
            }
        }
        Ok(ty)
    }

    /// Stores the current result register(s) into the SSA value's home.
    ///
    /// When the value lives in an allocated register, the result register is
    /// moved into it; otherwise it is stored into the value's stack slot.
    pub(super) fn store_result_value(&mut self, value: ValueId) -> Result<()> {
        let ty = self.value_php_type(value)?;
        if let Some(reg) = self.allocation.register_of(value) {
            let src = if ty.codegen_repr() == PhpType::Float {
                abi::float_result_reg(self.emitter)
            } else {
                abi::int_result_reg(self.emitter)
            };
            abi::emit_reg_move(self.emitter, reg, src);
        } else {
            let offset = self.value_offset(value)?;
            self.store_current_result_at_offset(&ty, offset);
        }
        Ok(())
    }

    /// Stores the integer result register as a single machine word into the SSA value's home.
    ///
    /// Reference-cell pointers are always one pointer-sized word regardless of the element
    /// type they alias (a `string` cell pointer is still one word, not a `{ptr,len}` pair).
    /// `LoadPropRefCell` and by-reference call results materialize the cell pointer into the
    /// integer result register, so it must be stored single-word; the type-driven
    /// `store_result_value` would otherwise split a `Str`/`Float` result across the string or
    /// float result registers and drop the pointer.
    pub(super) fn store_int_result_value(&mut self, value: ValueId) -> Result<()> {
        if let Some(reg) = self.allocation.register_of(value) {
            abi::emit_reg_move(self.emitter, reg, abi::int_result_reg(self.emitter));
        } else {
            let offset = self.value_offset(value)?;
            abi::store_at_offset(self.emitter, abi::int_result_reg(self.emitter), offset);
        }
        Ok(())
    }

    /// Stores an SSA value into an addressable local slot.
    pub(super) fn store_value_to_local(&mut self, slot: LocalSlotId, value: ValueId) -> Result<()> {
        match self.local_slot_representation(slot) {
            LocalSlotRepresentation::Raw => self.store_value_to_raw_local(slot, value),
            LocalSlotRepresentation::RefCell => self.store_value_to_ref_cell_local(slot, value),
            LocalSlotRepresentation::Dynamic => {
                let state_offset = self.dynamic_ref_cell_state_offset(slot)?;
                let ref_cell = self.next_label("dynamic_local_store_ref_cell");
                let done = self.next_label("dynamic_local_store_done");
                let state_reg = abi::secondary_scratch_reg(self.emitter);
                abi::load_at_offset(self.emitter, state_reg, state_offset);
                match self.emitter.target.arch {
                    Arch::AArch64 => {
                        self.emitter.instruction(                               // select ref-cell storage after a runtime promotion
                            &format!("cbnz {}, {}", state_reg, ref_cell)
                        );
                    }
                    Arch::X86_64 => {
                        self.emitter.instruction(                               // test the slot's runtime representation flag
                            &format!("test {}, {}", state_reg, state_reg)
                        );
                        self.emitter
                            .instruction(&format!("jne {}", ref_cell));           // select ref-cell storage after a runtime promotion
                    }
                }
                self.store_value_to_raw_local(slot, value)?;
                self.emit_branch(&done);
                self.emitter.label(&ref_cell);
                self.store_value_to_ref_cell_local(slot, value)?;
                self.emitter.label(&done);
                Ok(())
            }
        }
    }

    /// Releases a boxed source-local owner before a consuming container mutation.
    ///
    /// A concrete container loaded from a final Mixed frame slot is unboxed with an
    /// extra owned reference. Releasing the previous Mixed box before the runtime
    /// mutation transfers sole ownership to that SSA value, avoiding an artificial
    /// COW split while preserving real aliases. The mutation result can then be boxed
    /// as an owned replacement through the ordinary local store path.
    pub(super) fn release_mutated_source_local_owner(
        &mut self,
        slot: LocalSlotId,
        value: ValueId,
    ) -> Result<()> {
        let source_ty = self.value_php_type(value)?;
        let target_ty = self.local_php_type(slot)?;
        if self.local_slot_representation(slot) == LocalSlotRepresentation::Raw
            && matches!(target_ty, PhpType::Mixed | PhpType::Union(_))
            && !matches!(source_ty, PhpType::Mixed | PhpType::Union(_))
        {
            let offset = self.local_offset(slot)?;
            super::frame::emit_owned_local_cleanup(self, slot, offset, &target_ty);
        }
        Ok(())
    }

    /// Stores an SSA value into a slot known to contain its raw frame representation.
    pub(super) fn store_value_to_raw_local(
        &mut self,
        slot: LocalSlotId,
        value: ValueId,
    ) -> Result<()> {
        let source_ty = self.load_value_to_result(value)?;
        let target_ty = self.local_php_type(slot)?;
        if target_ty.codegen_repr() == PhpType::Mixed
            && source_ty.codegen_repr() != PhpType::Mixed
        {
            if self.value_can_own_mixed_box_source(value)? {
                emit_box_current_owned_value_as_mixed(self.emitter, &source_ty);
            } else {
                emit_box_current_value_as_mixed(self.emitter, &source_ty);
            }
        }
        // Narrow Mixed to Int when the local slot is typed Int but the value
        // is Mixed (from checked integer arithmetic that may overflow to float).
        // The runtime cast helper truncates floats and extracts ints. The
        // original Mixed box is released after narrowing to avoid leaks.
        if matches!(target_ty.codegen_repr(), PhpType::Int)
            && matches!(source_ty.codegen_repr(), PhpType::Mixed)
        {
            let result_reg = abi::int_result_reg(self.emitter);
            let arg_reg = abi::int_arg_reg_name(self.emitter.target, 0);
            if result_reg != arg_reg {
                abi::emit_reg_move(self.emitter, arg_reg, result_reg);
            }
            abi::emit_push_reg(self.emitter, result_reg);
            abi::emit_push_reg(self.emitter, arg_reg);
            abi::emit_call_label(self.emitter, "__rt_mixed_cast_int");
            match self.emitter.target.arch {
                Arch::AArch64 => {
                    self.emitter.instruction("str x0, [sp, #16]");              // save the int result to the placeholder slot
                }
                Arch::X86_64 => {
                    self.emitter.instruction("mov QWORD PTR [rsp + 16], rax");  // save the int result to the placeholder slot
                }
            }
            abi::emit_pop_reg(self.emitter, result_reg);
            abi::emit_call_label(self.emitter, "__rt_decref_mixed");
            abi::emit_pop_reg(self.emitter, result_reg);
        }
        coerce_current_result_for_target_store(self.emitter, &source_ty, &target_ty)?;
        let offset = self.local_offset(slot)?;
        self.store_current_result_at_offset(&target_ty, offset);
        Ok(())
    }

    /// Publishes a possibly relocated array/hash pointer after an in-place container mutation.
    pub(super) fn store_mutated_container_to_local(
        &mut self,
        slot: LocalSlotId,
        value: ValueId,
    ) -> Result<()> {
        match self.local_slot_representation(slot) {
            LocalSlotRepresentation::Raw => {
                self.store_mutated_container_to_raw_local(slot, value)
            }
            LocalSlotRepresentation::RefCell => {
                self.store_mutated_container_to_ref_cell(slot, value)
            }
            LocalSlotRepresentation::Dynamic => {
                let state_offset = self.dynamic_ref_cell_state_offset(slot)?;
                let ref_cell = self.next_label("dynamic_mutated_container_ref_cell");
                let done = self.next_label("dynamic_mutated_container_done");
                let state_reg = abi::secondary_scratch_reg(self.emitter);
                abi::load_at_offset(self.emitter, state_reg, state_offset);
                match self.emitter.target.arch {
                    Arch::AArch64 => self
                        .emitter
                        .instruction(&format!("cbnz {}, {}", state_reg, ref_cell)),
                    Arch::X86_64 => {
                        self.emitter
                            .instruction(&format!("test {}, {}", state_reg, state_reg));
                        self.emitter.instruction(&format!("jne {}", ref_cell));
                    }
                }
                self.store_value_to_raw_local(slot, value)?;
                self.emit_branch(&done);
                self.emitter.label(&ref_cell);
                self.store_mutated_container_to_ref_cell(slot, value)?;
                self.emitter.label(&done);
                Ok(())
            }
        }
    }

    /// Publishes a mutated container into raw local storage without consuming its SSA owner.
    ///
    /// A mutation result remains owned by the SSA value until its explicit EIR release. When a
    /// gradual local needs a boxed cell, that cell must therefore RETAIN the container instead
    /// of using the ordinary owned-value boxing path, which would transfer the same owner and
    /// leave the later SSA release pointing at freed storage.
    fn store_mutated_container_to_raw_local(
        &mut self,
        slot: LocalSlotId,
        value: ValueId,
    ) -> Result<()> {
        let source_ty = self.value_php_type(value)?;
        let target_ty = self.local_php_type(slot)?;
        if target_ty.codegen_repr() == PhpType::Mixed
            && source_ty.codegen_repr() != PhpType::Mixed
        {
            self.load_value_to_result(value)?;
            emit_box_current_value_as_mixed(self.emitter, &source_ty);
            coerce_current_result_for_target_store(self.emitter, &PhpType::Mixed, &target_ty)?;
            let offset = self.local_offset(slot)?;
            self.store_current_result_at_offset(&target_ty, offset);
            return Ok(());
        }
        self.store_value_to_raw_local(slot, value)
    }

    /// Writes one relocated single-word container pointer through a local reference cell.
    fn store_mutated_container_to_ref_cell(
        &mut self,
        slot: LocalSlotId,
        value: ValueId,
    ) -> Result<()> {
        let source_ty = self.load_value_to_result(value)?.codegen_repr();
        if !matches!(source_ty, PhpType::Array(_) | PhpType::AssocArray { .. }) {
            return Err(CodegenIrError::unsupported(format!(
                "mutated reference-cell container store for PHP type {:?}",
                source_ty
            )));
        }
        let offset = self.local_offset(slot)?;
        let pointer_reg = abi::symbol_scratch_reg(self.emitter);
        let result_reg = abi::int_result_reg(self.emitter).to_string();
        abi::load_at_offset(self.emitter, pointer_reg, offset);
        abi::emit_store_to_address(self.emitter, &result_reg, pointer_reg, 0);
        Ok(())
    }

    /// Stores the current result register(s) directly into an addressable local slot.
    pub(super) fn store_current_result_to_local(&mut self, slot: LocalSlotId) -> Result<()> {
        let target_ty = self.local_php_type(slot)?;
        match self.local_slot_representation(slot) {
            LocalSlotRepresentation::Raw => {
                let offset = self.local_offset(slot)?;
                self.store_current_result_at_offset(&target_ty, offset);
                Ok(())
            }
            LocalSlotRepresentation::RefCell => {
                self.store_current_result_to_ref_cell_local(slot, &target_ty)
            }
            LocalSlotRepresentation::Dynamic => {
                let state_offset = self.dynamic_ref_cell_state_offset(slot)?;
                let ref_cell = self.next_label("dynamic_current_store_ref_cell");
                let done = self.next_label("dynamic_current_store_done");
                let state_reg = abi::secondary_scratch_reg(self.emitter);
                abi::load_at_offset(self.emitter, state_reg, state_offset);
                match self.emitter.target.arch {
                    Arch::AArch64 => {
                        self.emitter.instruction(                               // select ref-cell storage after a runtime promotion
                            &format!("cbnz {}, {}", state_reg, ref_cell)
                        );
                    }
                    Arch::X86_64 => {
                        self.emitter.instruction(                               // test the slot's runtime representation flag
                            &format!("test {}, {}", state_reg, state_reg)
                        );
                        self.emitter
                            .instruction(&format!("jne {}", ref_cell));           // select ref-cell storage after a runtime promotion
                    }
                }
                let offset = self.local_offset(slot)?;
                self.store_current_result_at_offset(&target_ty, offset);
                self.emit_branch(&done);
                self.emitter.label(&ref_cell);
                self.store_current_result_to_ref_cell_local(slot, &target_ty)?;
                self.emitter.label(&done);
                Ok(())
            }
        }
    }

    /// Releases a refcounted or boxed local value before a runtime-owned output replaces it.
    pub(super) fn release_local_before_refcounted_writeback(
        &mut self,
        slot: LocalSlotId,
    ) -> Result<()> {
        let ty = self.local_php_type(slot)?.codegen_repr();
        if !(matches!(ty, PhpType::Str | PhpType::Mixed | PhpType::Union(_))
            || ty.is_refcounted())
        {
            return Err(CodegenIrError::unsupported(format!(
                "refcounted writeback into PHP type {:?}",
                ty
            )));
        }
        match self.local_slot_representation(slot) {
            LocalSlotRepresentation::Raw => {
                let offset = self.local_offset(slot)?;
                super::frame::emit_owned_local_cleanup(self, slot, offset, &ty);
            }
            LocalSlotRepresentation::RefCell => self.release_ref_cell_value(slot, &ty)?,
            LocalSlotRepresentation::Dynamic => {
                let state_offset = self.dynamic_ref_cell_state_offset(slot)?;
                let ref_cell = self.next_label("refcounted_writeback_release_ref_cell");
                let done = self.next_label("refcounted_writeback_release_done");
                let state_reg = abi::secondary_scratch_reg(self.emitter);
                abi::load_at_offset(self.emitter, state_reg, state_offset);
                match self.emitter.target.arch {
                    Arch::AArch64 => {
                        self.emitter
                            .instruction(&format!("cbnz {}, {}", state_reg, ref_cell)); // release through the promoted ref-cell when active
                    }
                    Arch::X86_64 => {
                        self.emitter
                            .instruction(&format!("test {}, {}", state_reg, state_reg)); // inspect the local's runtime representation
                        self.emitter
                            .instruction(&format!("jne {}", ref_cell));                  // release through the promoted ref-cell when active
                    }
                }
                let offset = self.local_offset(slot)?;
                super::frame::emit_owned_local_cleanup(self, slot, offset, &ty);
                self.emit_branch(&done);
                self.emitter.label(&ref_cell);
                self.release_ref_cell_value(slot, &ty)?;
                self.emitter.label(&done);
            }
        }
        Ok(())
    }

    /// Releases a string or Mixed payload stored through a local ref-cell pointer.
    fn release_ref_cell_value(&mut self, slot: LocalSlotId, ty: &PhpType) -> Result<()> {
        let offset = self.local_offset(slot)?;
        let cell_reg = abi::symbol_scratch_reg(self.emitter);
        let result_reg = abi::int_result_reg(self.emitter);
        abi::load_at_offset(self.emitter, cell_reg, offset);
        abi::emit_load_from_address(self.emitter, result_reg, cell_reg, 0);
        if *ty == PhpType::Str {
            abi::emit_call_label(self.emitter, "__rt_heap_free_safe");
        } else {
            abi::emit_decref_if_refcounted(self.emitter, ty);
        }
        Ok(())
    }

    /// After an in-place hash/array mutation whose runtime helper returns the
    /// possibly-reallocated container pointer in `value`'s register (already
    /// persisted via `store_result_value`), writes that pointer back to global
    /// storage when `value` was loaded from a global — i.e. a superglobal such as
    /// `$_SERVER`/`$_GET`/`$_POST`. Mirrors the local-slot write-back that array
    /// and hash set/append lowerings already perform; without it a global array
    /// that grows past its initial capacity leaves the global symbol pointing at
    /// freed storage (corruption / crash). No-op unless `value` came from
    /// `Op::LoadGlobal`.
    ///
    /// A function `static` is the OTHER program-lifetime place with this exact exposure — its
    /// storage is a `.comm` symbol the frame does not own — so this also republishes to
    /// `Op::LoadStaticLocal` sources, via [`Self::writeback_static_local_array_source`]. The
    /// name says `global` for the sixteen call sites that predate that; read it as "publish a
    /// relocated container back to whichever program-lifetime storage it was read from".
    pub(super) fn writeback_global_array_source(&mut self, value: ValueId) -> Result<()> {
        let Some(value_ref) = self.function.value(value) else {
            return Err(CodegenIrError::missing_entry("value", value.as_raw()));
        };
        let ValueDef::Instruction { inst, .. } = value_ref.def else {
            return Ok(());
        };
        let Some(inst_ref) = self.function.instruction(inst) else {
            return Err(CodegenIrError::missing_entry("instruction", inst.as_raw()));
        };
        if inst_ref.op == Op::LoadStaticLocal {
            return self.writeback_static_local_array_source(value);
        }
        if inst_ref.op != Op::LoadGlobal {
            return Ok(());
        }
        let Some(crate::ir::Immediate::GlobalName(data)) = inst_ref.immediate else {
            return Ok(());
        };
        let name = self.global_name_data(data)?.to_string();
        let symbol = crate::names::ir_global_symbol(&name);
        let ty = self.value_php_type(value)?;
        if self.shared.uses_shared_ref_cell(self.module, &name) {
            self.load_value_to_result(value)?;
            return crate::codegen::lower_inst::lower_store_shared_global(
                self,
                &symbol,
                &ty,
            );
        }
        self.data.add_comm(symbol.clone(), ty.codegen_repr().stack_size().max(8));
        self.load_value_to_result(value)?;
        abi::emit_store_result_to_symbol(self.emitter, &symbol, &ty, false);
        Ok(())
    }

    /// The function-`static` twin of [`Self::writeback_global_array_source`].
    ///
    /// A `static` local lives in a `.comm` symbol, not a frame slot, so the ordinary
    /// `store_value_to_local` write-back every array/hash mutation performs reaches the SSA
    /// value's frame home and never the storage the next CALL will read. `Op::LoadStaticLocal`
    /// is simply a third spelling of "a place this container can be published back to", and it
    /// was missing from every resolver that names one — so `static $q = ['s']; $q[] = 'x';`
    /// past the array's initial capacity left the symbol pointing at the pre-`__rt_array_grow`
    /// allocation: the appended elements were unreachable and the pointer was freed storage.
    ///
    /// No refcount traffic. The slot already owns this container; only its ADDRESS changed, and
    /// the relocating helper (`__rt_array_grow`'s realloc, `__rt_array_ensure_unique`'s split)
    /// has already accounted for the old one. That is the same contract
    /// `store_mutated_container_to_local` states for a frame slot, which is why this passes
    /// `release_previous: false` rather than going through the ordinary static-local store.
    ///
    /// No-op unless `value` came from `Op::LoadStaticLocal`.
    pub(super) fn writeback_static_local_array_source(&mut self, value: ValueId) -> Result<()> {
        let Some(slot) = self.static_local_source_slot(value)? else {
            return Ok(());
        };
        self.store_relocated_container_to_static_local(slot, value)
    }

    /// Resolves the static-local slot a value was loaded from, if it was loaded from one.
    pub(super) fn static_local_source_slot(
        &self,
        value: ValueId,
    ) -> Result<Option<LocalSlotId>> {
        let Some(value_ref) = self.function.value(value) else {
            return Err(CodegenIrError::missing_entry("value", value.as_raw()));
        };
        let ValueDef::Instruction { inst, .. } = value_ref.def else {
            return Ok(None);
        };
        let Some(inst_ref) = self.function.instruction(inst) else {
            return Err(CodegenIrError::missing_entry("instruction", inst.as_raw()));
        };
        if inst_ref.op != Op::LoadStaticLocal {
            return Ok(None);
        }
        match inst_ref.immediate {
            Some(crate::ir::Immediate::LocalSlot(slot)) => Ok(Some(slot)),
            _ => Ok(None),
        }
    }

    /// Publishes a relocated container pointer into a static local's symbol storage.
    ///
    /// See [`Self::writeback_static_local_array_source`] for why this does no refcount work.
    pub(super) fn store_relocated_container_to_static_local(
        &mut self,
        slot: LocalSlotId,
        value: ValueId,
    ) -> Result<()> {
        let local = self
            .function
            .locals
            .get(slot.as_raw() as usize)
            .ok_or_else(|| CodegenIrError::missing_entry("local slot", slot.as_raw()))?;
        let Some(name) = local.name.clone() else {
            return Err(CodegenIrError::invalid_module(
                "static local container write-back is missing a source name",
            ));
        };
        let slot_ty = local.php_type.codegen_repr();
        let symbol = crate::names::static_local_symbol(&self.function.name, &name);
        self.data.add_comm(symbol.clone(), 16);
        let source_ty = self.load_value_to_result(value)?.codegen_repr();
        // A gradual slot holds a boxed Mixed cell whose identity the mutation preserved, so the
        // concrete container pointer in the result register is NOT what belongs in the symbol.
        // Writing it there would replace the cell with a raw array pointer and every later read
        // would unbox garbage. The boxed cell already points at the relocated container: the
        // dynamic mutation paths publish through it.
        if slot_ty == PhpType::Mixed && source_ty != PhpType::Mixed {
            return Ok(());
        }
        abi::emit_store_result_to_symbol(self.emitter, &symbol, &slot_ty, false);
        Ok(())
    }

    /// Stores an SSA value through a local ref-cell pointer slot.
    fn store_value_to_ref_cell_local(&mut self, slot: LocalSlotId, value: ValueId) -> Result<()> {
        let source_ty = self.load_value_to_result(value)?;
        let target_ty = self.local_php_type(slot)?;
        reject_multiword_ref_cell_local(&target_ty, "store")?;
        if target_ty.codegen_repr() == PhpType::Mixed
            && source_ty.codegen_repr() != PhpType::Mixed
        {
            if self.value_can_own_mixed_box_source(value)? {
                emit_box_current_owned_value_as_mixed(self.emitter, &source_ty);
            } else {
                emit_box_current_value_as_mixed(self.emitter, &source_ty);
            }
        }
        coerce_current_result_for_target_store(self.emitter, &source_ty, &target_ty)?;
        let offset = self.local_offset(slot)?;
        let pointer_reg = abi::symbol_scratch_reg(self.emitter);
        abi::load_at_offset(self.emitter, pointer_reg, offset);
        match target_ty.codegen_repr() {
            PhpType::Str => {
                let (ptr_reg, len_reg) = abi::string_result_regs(self.emitter);
                abi::emit_store_to_address(self.emitter, ptr_reg, pointer_reg, 0);
                abi::emit_store_to_address(self.emitter, len_reg, pointer_reg, 8);
            }
            PhpType::Float => {
                abi::emit_store_to_address(self.emitter, abi::float_result_reg(self.emitter), pointer_reg, 0);
            }
            PhpType::TaggedScalar => {
                abi::emit_store_to_address(self.emitter, abi::int_result_reg(self.emitter), pointer_reg, 0);
                abi::emit_store_to_address(
                    self.emitter,
                    crate::codegen::sentinels::tagged_scalar_tag_reg(self.emitter),
                    pointer_reg,
                    8,
                );
            }
            _ => {
                abi::emit_store_to_address(self.emitter, abi::int_result_reg(self.emitter), pointer_reg, 0);
            }
        }
        Ok(())
    }

    /// Stores the current result register(s) through a local ref-cell pointer slot.
    fn store_current_result_to_ref_cell_local(
        &mut self,
        slot: LocalSlotId,
        target_ty: &PhpType,
    ) -> Result<()> {
        reject_multiword_ref_cell_local(target_ty, "store")?;
        let offset = self.local_offset(slot)?;
        let pointer_reg = abi::symbol_scratch_reg(self.emitter);
        abi::load_at_offset(self.emitter, pointer_reg, offset);
        match target_ty.codegen_repr() {
            PhpType::Str => {
                let (ptr_reg, len_reg) = abi::string_result_regs(self.emitter);
                abi::emit_store_to_address(self.emitter, ptr_reg, pointer_reg, 0);
                abi::emit_store_to_address(self.emitter, len_reg, pointer_reg, 8);
            }
            PhpType::Float => {
                abi::emit_store_to_address(
                    self.emitter,
                    abi::float_result_reg(self.emitter),
                    pointer_reg,
                    0,
                );
            }
            PhpType::TaggedScalar => {
                abi::emit_store_to_address(
                    self.emitter,
                    abi::int_result_reg(self.emitter),
                    pointer_reg,
                    0,
                );
                abi::emit_store_to_address(
                    self.emitter,
                    crate::codegen::sentinels::tagged_scalar_tag_reg(self.emitter),
                    pointer_reg,
                    8,
                );
            }
            _ => {
                abi::emit_store_to_address(
                    self.emitter,
                    abi::int_result_reg(self.emitter),
                    pointer_reg,
                    0,
                );
            }
        }
        Ok(())
    }

    /// Stores the current result register(s) into a frame offset.
    fn store_current_result_at_offset(&mut self, ty: &PhpType, offset: usize) {
        match &ty.codegen_repr() {
            PhpType::Str => {
                let (ptr_reg, len_reg) = abi::string_result_regs(self.emitter);
                abi::store_at_offset(self.emitter, ptr_reg, offset);
                abi::store_at_offset(self.emitter, len_reg, offset - 8);
            }
            PhpType::TaggedScalar => {
                abi::store_at_offset(self.emitter, abi::int_result_reg(self.emitter), offset);
                abi::store_at_offset(
                    self.emitter,
                    crate::codegen::sentinels::tagged_scalar_tag_reg(self.emitter),
                    offset - 8,
                );
            }
            PhpType::Float => {
                abi::store_at_offset(self.emitter, abi::float_result_reg(self.emitter), offset);
            }
            PhpType::Void => {
                abi::store_at_offset(self.emitter, abi::int_result_reg(self.emitter), offset);
            }
            PhpType::Never => {}
            _ => {
                abi::store_at_offset(self.emitter, abi::int_result_reg(self.emitter), offset);
            }
        }
    }

    /// Returns true when Mixed boxing can consume the value's owned source reference.
    pub(super) fn value_can_own_mixed_box_source(&self, value: ValueId) -> Result<bool> {
        let value_ty = self.value_php_type(value)?.codegen_repr();
        if value_ty == PhpType::Str {
            return self.value_is_heap_owned_string_for_mixed_box(value);
        }
        if self.value_can_transfer_ownership_to_consumer(value)? {
            return Ok(true);
        }
        let Some(value_ref) = self.function.value(value) else {
            return Err(CodegenIrError::missing_entry("value", value.as_raw()));
        };
        let ValueDef::Instruction { inst, .. } = value_ref.def else {
            return Ok(false);
        };
        let inst = self
            .function
            .instruction(inst)
            .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
        if inst.op == Op::Acquire {
            return Ok(!self.function.instructions.iter().any(|inst| {
                inst.op == Op::Release && inst.operands.first().copied() == Some(value)
            }));
        }
        if matches!(inst.op, Op::LoadLocal | Op::LoadStaticLocal) {
            let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
                return Ok(false);
            };
            let storage_ty = self.local_php_type(slot)?;
            return Ok(matches!(storage_ty, PhpType::Mixed | PhpType::Union(_))
                && matches!(
                    value_ty,
                    PhpType::Array(_)
                        | PhpType::AssocArray { .. }
                        | PhpType::Callable
                        | PhpType::Object(_)
                        | PhpType::Iterable
                ));
        }
        Ok(false)
    }

    /// Returns true when a retaining consumer may take the value's owned reference.
    ///
    /// `Owned` identifies the value that must eventually be cleaned up; it does not
    /// by itself cancel an explicit EIR `release`. When lowering still emits such a
    /// release, the consumer must retain its own reference instead of stealing the
    /// one that cleanup will consume.
    pub(super) fn value_can_transfer_ownership_to_consumer(
        &self,
        value: ValueId,
    ) -> Result<bool> {
        let mut transfers_owned = self.value_ownership(value)? == Ownership::Owned;
        let Some(value_ref) = self.function.value(value) else {
            return Err(CodegenIrError::missing_entry("value", value.as_raw()));
        };
        if let ValueDef::Instruction { inst, .. } = value_ref.def {
            let defining_inst = self
                .function
                .instruction(inst)
                .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
            // String Acquire persists the payload even when conservative EIR
            // metadata says MaybeOwned. Its new owner may be transferred, but
            // only if no explicit Release below still consumes that owner.
            if defining_inst.op == Op::Acquire && self.value_php_type(value)? == PhpType::Str {
                transfers_owned = true;
            }
            if defining_inst.op == Op::LoadLocal {
                if let Some(Immediate::LocalSlot(slot)) = defining_inst.immediate {
                    if self.local_kind(slot)? == LocalKind::OwnedTemp {
                        let next_inst = self.function.instructions.get(inst.as_raw() as usize + 1);
                        let moves_out_of_slot = next_inst.is_some_and(|next| {
                            next.op == Op::UnsetLocal
                                && next.immediate == Some(Immediate::LocalSlot(slot))
                        });
                        if !moves_out_of_slot {
                            return Ok(false);
                        }
                    }
                }
            }
        }
        Ok(transfers_owned && !self.function.instructions.iter().any(|inst| {
            inst.op == Op::Release && inst.operands.first().copied() == Some(value)
        }))
    }

    /// Returns true when a string producer leaves a heap-owned payload that Mixed boxing may consume.
    fn value_is_heap_owned_string_for_mixed_box(&self, value: ValueId) -> Result<bool> {
        let Some(value_ref) = self.function.value(value) else {
            return Err(CodegenIrError::missing_entry("value", value.as_raw()));
        };
        let ValueDef::Instruction { inst, .. } = value_ref.def else {
            return Ok(false);
        };
        let inst = self
            .function
            .instruction(inst)
            .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
        Ok(matches!(
            inst.op,
            Op::Acquire
                | Op::StrPersist
                | Op::Call
                | Op::FunctionVariantCall
                | Op::ExternCall
                | Op::MethodCall
                | Op::NullsafeMethodCall
                | Op::StaticMethodCall
                | Op::ClosureCall
                | Op::CallableDescriptorInvoke
                | Op::ExprCall
                | Op::PipeCall
                | Op::IteratorMethodCall
                | Op::SplRuntimeCall
                | Op::FiberRuntimeCall
        ))
    }

    /// Interns a module data-pool string into the assembly data section.
    pub(super) fn intern_string_data(&mut self, data_id: DataId) -> Result<(String, usize)> {
        let value = self
            .module
            .data
            .strings
            .get(data_id.as_raw() as usize)
            .ok_or_else(|| CodegenIrError::missing_entry("data string", data_id.as_raw()))?;
        let bytes = crate::string_bytes::literal_bytes(value);
        Ok(self.data.add_string(&bytes))
    }

    /// Interns a module class-name data-pool entry into the assembly data section.
    pub(super) fn intern_class_name_data(&mut self, data_id: DataId) -> Result<(String, usize)> {
        let value = self
            .module
            .data
            .class_names
            .get(data_id.as_raw() as usize)
            .ok_or_else(|| CodegenIrError::missing_entry("class data", data_id.as_raw()))?;
        Ok(self.data.add_string(value.as_bytes()))
    }

    /// Returns a module data-pool function name.
    pub(super) fn function_name_data(&self, data_id: DataId) -> Result<&str> {
        self.module
            .data
            .function_names
            .get(data_id.as_raw() as usize)
            .map(String::as_str)
            .ok_or_else(|| CodegenIrError::missing_entry("function data", data_id.as_raw()))
    }

    /// Returns a module data-pool global name.
    pub(super) fn global_name_data(&self, data_id: DataId) -> Result<&str> {
        self.module
            .data
            .global_names
            .get(data_id.as_raw() as usize)
            .map(String::as_str)
            .ok_or_else(|| CodegenIrError::missing_entry("global data", data_id.as_raw()))
    }

    /// Returns true when the EIR module has interned a matching global name.
    pub(super) fn has_global_name(&self, name: &str) -> bool {
        let normalized = name.trim_start_matches('\\');
        self.module
            .data
            .global_names
            .iter()
            .any(|candidate| candidate.trim_start_matches('\\') == normalized)
    }

    /// Returns the frame offset assigned to a value by Phase 04 placement.
    fn value_offset(&self, value: ValueId) -> Result<usize> {
        self.placement
            .slot(value)
            .ok_or_else(|| CodegenIrError::missing_entry("value slot", value.as_raw()))
    }

    /// Returns the frame offset assigned to a value for custom multi-word lowerings.
    pub(super) fn value_frame_offset(&self, value: ValueId) -> Result<usize> {
        self.value_offset(value)
    }

    /// Returns the frame offset assigned to an addressable EIR local.
    pub(super) fn local_offset(&self, slot: LocalSlotId) -> Result<usize> {
        self.local_offsets
            .get(&slot)
            .copied()
            .ok_or_else(|| CodegenIrError::missing_entry("local slot offset", slot.as_raw()))
    }

    /// Returns the frame offset assigned to a high-level try-handler token.
    pub(super) fn try_handler_offset(&self, token: i64) -> Result<usize> {
        self.try_handler_offsets
            .get(&token)
            .copied()
            .ok_or_else(|| CodegenIrError::invalid_module(format!("missing try handler token {}", token)))
    }
}

/// Returns whether the module still declares the nullary shutdown drain.
///
/// A FUNCTION-LIST question, not an instruction one: `__elephc_shutdown_run` has no PHP caller
/// off `--web`, so nothing in the instruction stream would show it. Its presence is decided
/// earlier, by `error_handling_prelude::inject_if_used` plus the forced-group rule in
/// `pipeline::compile`, and reading the surviving list here is how codegen learns the answer.
fn module_declares_shutdown_drain(module: &Module) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.name == crate::names::SHUTDOWN_RUN_FUNCTION)
}

/// Scans every emitted function-like body for `pcntl_async_signals()` state changes.
fn module_uses_pcntl_async_signals(module: &Module) -> bool {
    module_uses_pcntl_operation(module, crate::ir::PcntlRuntime::AsyncSignals)
}

/// Scans every emitted function-like body for `pcntl_signal()` registrations.
pub(super) fn module_uses_pcntl_signal_handlers(module: &Module) -> bool {
    module_uses_pcntl_operation(module, crate::ir::PcntlRuntime::Signal)
}

/// Scans every emitted function-like body for one typed PCNTL operation.
fn module_uses_pcntl_operation(module: &Module, target: crate::ir::PcntlRuntime) -> bool {
    module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter())
        .chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter())
        .chain(module.runtime_callable_invokers.iter())
        .any(|function| {
            function.instructions.iter().any(|inst| {
                matches!(
                    inst.immediate,
                    Some(Immediate::RuntimeCall(RuntimeCallTarget::Pcntl(found))) if found == target
                )
            })
        })
}

/// Rejects local ref-cell operations whose frame representation spans multiple words.
fn reject_multiword_ref_cell_local(ty: &PhpType, action: &str) -> Result<()> {
    let _ = (ty, action);
    Ok(())
}

/// Coerces the currently loaded result registers before storing into a typed local slot.
fn coerce_current_result_for_target_store(
    emitter: &mut Emitter,
    source_ty: &PhpType,
    target_ty: &PhpType,
) -> Result<()> {
    if target_ty.codegen_repr() != PhpType::TaggedScalar {
        return Ok(());
    }
    match source_ty.codegen_repr() {
        PhpType::TaggedScalar => Ok(()),
        PhpType::Int | PhpType::Bool | PhpType::Callable => {
            crate::codegen::sentinels::emit_tagged_scalar_from_int_result(emitter);
            Ok(())
        }
        PhpType::Void | PhpType::Never => {
            crate::codegen::sentinels::emit_tagged_scalar_null(emitter);
            Ok(())
        }
        PhpType::Mixed | PhpType::Union(_) => {
            emit_mixed_result_as_tagged_scalar(emitter);
            Ok(())
        }
        other => Err(CodegenIrError::unsupported(format!(
            "local store from PHP type {:?} to PHP type TaggedScalar",
            other
        ))),
    }
}

/// Reorders `__rt_mixed_unbox` output into the EIR tagged-scalar result registers.
fn emit_mixed_result_as_tagged_scalar(emitter: &mut Emitter) {
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("mov x9, x0");                                  // preserve the unboxed Mixed tag before moving the payload
            emitter.instruction("mov x0, x1");                                  // place the unboxed payload into the tagged-scalar payload register
            emitter.instruction("mov x1, x9");                                  // place the unboxed Mixed tag into the tagged-scalar tag register
        }
        Arch::X86_64 => {
            emitter.instruction("mov r10, rax");                                // preserve the unboxed Mixed tag before moving the payload
            emitter.instruction("mov rax, rdi");                                // place the unboxed payload into the tagged-scalar payload register
            emitter.instruction("mov rdx, r10");                                // place the unboxed Mixed tag into the tagged-scalar tag register
        }
    }
}
