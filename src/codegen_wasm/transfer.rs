//! Purpose:
//! Type-aware value-transfer layer for the wasm32-wasi backend.
//!
//! Moves EIR values and raw operand-stack values between WebAssembly
//! representations, applying PHP/EIR conversions such as concrete-to-Mixed boxing.
//! Compares concrete WASM types and payload shapes, not only component counts.
//!
//! Called from:
//! - `crate::codegen_wasm::function` for block-argument materialization and the
//!   main-function `$argc`/`$argv` prologue.
//! - `crate::codegen_wasm::inst` for call-result storage, local loads/stores, and
//!   value forwarding.
//!
//! Key details:
//! - Identical storage representations copy component-wise; concrete-to-Mixed
//!   destinations box via `__rt_mixed_from_value`.
//! - Mixed-to-concrete transfers use the runtime cast helpers, and heap-pointer
//!   unboxing validates the runtime tag before exposing the payload.
//! - Unsupported shape-mismatched transfers return a precise `WasmError`
//!   instead of emitting invalid WAT.
//! - All multi-source transfers (block arguments) load every source into temps
//!   before storing any destination, preserving parallel-move safety.

use super::context::{FnCtx, Result};
use super::values::WasmRepr;
use super::wat::ValType;
use super::WasmError;
use crate::ir::{IrHeapKind, IrType, LocalSlotId, ValueId};
use crate::types::PhpType;

/// Elephc's null sentinel value, loaded for the result of a void callee.
const VOID_SENTINEL: i64 = 0x7fff_ffff_ffff_fffe;

/// The exact conversion family shared by capability validation and emission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TransferKind {
    /// Source and destination have the same concrete WASM representation.
    Copy,
    /// A concrete source is boxed into a runtime Mixed cell.
    BoxMixed,
    /// A runtime Mixed cell is converted to a concrete destination.
    UnboxMixed,
}

/// Returns true when an EIR/PHP pair is realized as a runtime Mixed cell pointer.
///
/// Ordinary unions are normalized to `Mixed` before this boundary; nullable-int
/// unions use `TaggedScalar` and must not be treated as Mixed cells.
fn is_mixed_cell_storage(ir: IrType, php: &PhpType) -> bool {
    ir == IrType::Heap(IrHeapKind::Mixed) && php.codegen_repr() == PhpType::Mixed
}

/// Validates that one PHP type uses its canonical EIR storage representation.
///
/// Null values are the sole exception: EIR materializes their sentinel as
/// `I64/Void`, while function-level void results use `Void/Void`.
pub(super) fn validate_storage_pair(ir: IrType, php: &PhpType) -> Result<()> {
    let php = php.codegen_repr();
    let canonical_ir = IrType::from_php(&php);
    if ir == canonical_ir || (ir == IrType::I64 && php == PhpType::Void) {
        return Ok(());
    }
    Err(WasmError::Unsupported(format!(
        "invalid wasm storage pair {:?}/{:?}; canonical EIR storage is {:?}",
        ir, php, canonical_ir
    )))
}

/// Returns true when a source representation can be copied bit-wise into the
/// destination representation without any PHP-level conversion.
fn reprs_match_for_copy(
    source_repr: &WasmRepr,
    source_ir: IrType,
    source_php: &PhpType,
    dest_repr: &WasmRepr,
    dest_ir: IrType,
    dest_php: &PhpType,
) -> bool {
    match (source_repr, dest_repr) {
        (WasmRepr::I64(_), WasmRepr::I64(_))
        | (WasmRepr::F64(_), WasmRepr::F64(_))
        | (WasmRepr::Void, WasmRepr::Void)
        | (WasmRepr::Str { .. }, WasmRepr::Str { .. })
        | (WasmRepr::Tagged { .. }, WasmRepr::Tagged { .. }) => source_php == dest_php,
        (WasmRepr::Ptr(_), WasmRepr::Ptr(_)) => {
            // If either side is a Mixed cell, the transfer is not a plain copy:
            // concrete-to-Mixed must box, and Mixed-to-concrete must unbox.
            if is_mixed_cell_storage(source_ir, source_php)
                || is_mixed_cell_storage(dest_ir, dest_php)
            {
                return false;
            }
            // Concrete heap pointers are bit-compatible only when their EIR
            // storage kinds agree. Cross-kind pointer reinterpretation requires
            // an explicit lowering path instead of a count-only copy.
            source_ir == dest_ir && source_php == dest_php
        }
        _ => false,
    }
}

/// Classifies one source-to-destination transfer without emitting WAT.
///
/// This is the capability contract for call arguments and results. Emission
/// consumes the same classification, preventing the static gate from drifting
/// away from the conversion branches implemented below.
pub(super) fn classify_transfer(
    source_ir: IrType,
    source_php: PhpType,
    dest_ir: IrType,
    dest_php: PhpType,
) -> Result<TransferKind> {
    validate_storage_pair(source_ir, &source_php).map_err(|error| {
        WasmError::Unsupported(format!("invalid source for wasm value transfer: {error}"))
    })?;
    validate_storage_pair(dest_ir, &dest_php).map_err(|error| {
        WasmError::Unsupported(format!(
            "invalid destination for wasm value transfer: {error}"
        ))
    })?;
    let source_php = source_php.codegen_repr();
    let dest_php = dest_php.codegen_repr();
    let source_repr = repr_for_ir(source_ir);
    let dest_repr = repr_for_ir(dest_ir);
    if is_mixed_cell_storage(dest_ir, &dest_php) {
        if is_mixed_cell_storage(source_ir, &source_php) {
            return Ok(TransferKind::Copy);
        }
        mixed_box_tag(&source_php, source_ir)?;
        return Ok(TransferKind::BoxMixed);
    }
    if is_mixed_cell_storage(source_ir, &source_php) {
        let supported = match dest_repr {
            WasmRepr::I64(_) => matches!(dest_php, PhpType::Int | PhpType::Bool),
            WasmRepr::F64(_) => dest_php == PhpType::Float,
            WasmRepr::Str { .. } => dest_php == PhpType::Str,
            WasmRepr::Ptr(_) => matches!(
                dest_ir,
                IrType::Heap(
                    IrHeapKind::Array
                        | IrHeapKind::Hash
                        | IrHeapKind::Object
                        | IrHeapKind::Iterable
                )
            ),
            WasmRepr::Tagged { .. } | WasmRepr::Void => false,
        };
        if supported {
            return Ok(TransferKind::UnboxMixed);
        }
        return Err(WasmError::Unsupported(format!(
            "unboxing a Mixed cell to {:?} ({:?}) is not supported on wasm32-wasi",
            dest_ir, dest_php
        )));
    }
    if reprs_match_for_copy(
        &source_repr,
        source_ir,
        &source_php,
        &dest_repr,
        dest_ir,
        &dest_php,
    ) {
        return Ok(TransferKind::Copy);
    }
    Err(WasmError::Unsupported(format!(
        "unsupported wasm value transfer from {:?} ({:?}/{:?}) to {:?} ({:?}/{:?})",
        source_repr, source_php, source_ir, dest_repr, dest_php, dest_ir
    )))
}

/// Returns the Mixed-cell boxing tag for a concrete source value.
fn mixed_box_tag(source_php: &PhpType, source_ir: IrType) -> Result<i64> {
    let source_php = source_php.codegen_repr();
    match (source_ir, &source_php) {
        (IrType::I64, PhpType::Int) => Ok(0),
        (IrType::I64, PhpType::Bool) => Ok(3),
        (IrType::I64, PhpType::Callable) => Ok(10),
        (IrType::I64 | IrType::Void, PhpType::Void) => Ok(8),
        (IrType::F64, PhpType::Float) => Ok(2),
        (IrType::Str, PhpType::Str) => Ok(1),
        (IrType::Heap(IrHeapKind::Array), PhpType::Array(_)) => Ok(4),
        (IrType::Heap(IrHeapKind::Hash), PhpType::AssocArray { .. }) => Ok(5),
        (IrType::Heap(IrHeapKind::Object), PhpType::Object(_)) => Ok(6),
        (IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed) => Err(WasmError::Unsupported(
            "mixed_box_tag called on an already-Mixed value".to_string(),
        )),
        _ => Err(WasmError::Unsupported(format!(
            "boxing {:?}/{:?} into a Mixed cell is not supported on wasm32-wasi",
            source_ir, source_php
        ))),
    }
}

/// Builds a `WasmRepr` from a stack-captured temp name list and the EIR type
/// those temps represent.
fn repr_from_temps(ir: IrType, temps: &[String]) -> WasmRepr {
    match ir {
        IrType::I64 => WasmRepr::I64(temps[0].clone()),
        IrType::F64 => WasmRepr::F64(temps[0].clone()),
        IrType::Heap(_) => WasmRepr::Ptr(temps[0].clone()),
        IrType::Str => WasmRepr::Str {
            ptr: temps[0].clone(),
            len: temps[1].clone(),
        },
        IrType::TaggedScalar => WasmRepr::Tagged {
            payload: temps[0].clone(),
            tag: temps[1].clone(),
        },
        IrType::Void => WasmRepr::Void,
    }
}

/// Pops a value of the given WebAssembly component types from the operand stack
/// into fresh temp locals, returning the temp names in canonical order.
fn pop_stack_to_temps(ctx: &mut FnCtx, val_types: &[ValType]) -> Result<Vec<String>> {
    let mut temps = Vec::with_capacity(val_types.len());
    for ty in val_types {
        temps.push(ctx.fresh_temp(*ty));
    }
    for name in temps.iter().rev() {
        ctx.fb
            .ins(&format!("local.set {}", name), "capture stack component into temp");
    }
    Ok(temps)
}

/// Loads an EIR value into fresh temp locals and returns the temp names in
/// canonical order.
fn load_value_to_temps(ctx: &mut FnCtx, value: ValueId) -> Result<Vec<String>> {
    let repr = ctx.value_repr(value)?.clone();
    let val_types = repr.component_val_types();
    ctx.emit_load_value(value)?;
    pop_stack_to_temps(ctx, &val_types)
}

/// Loads a local slot into fresh temp locals and returns the temp names in
/// canonical order.
fn load_slot_to_temps(ctx: &mut FnCtx, slot: LocalSlotId) -> Result<Vec<String>> {
    let repr = ctx.slot_repr(slot)?.clone();
    let val_types = repr.component_val_types();
    ctx.emit_load_slot(slot)?;
    pop_stack_to_temps(ctx, &val_types)
}

/// Emits the unboxing of a Mixed cell pointer into a concrete destination.
///
/// Uses the existing `__rt_mixed_cast_*` helpers so the unboxed result is owned
/// when needed (strings are persisted; heap pointers are borrowed from the cell).
fn emit_unbox_mixed_to_concrete(
    ctx: &mut FnCtx,
    source_temps: &[String],
    dest_repr: &WasmRepr,
    dest_ir: IrType,
    dest_php: &PhpType,
) -> Result<()> {
    let ptr = &source_temps[0];
    ctx.fb
        .ins(&format!("local.get {}", ptr), "mixed cell to unbox");

    match dest_repr {
        WasmRepr::I64(_) => {
            let call = if *dest_php == PhpType::Bool {
                "__rt_mixed_cast_bool"
            } else {
                "__rt_mixed_cast_int"
            };
            ctx.fb.ins(
                &format!("call ${}", call),
                "unbox i64 payload from mixed cell",
            );
        }
        WasmRepr::F64(_) => {
            ctx.fb.ins(
                "call $__rt_mixed_cast_float",
                "unbox f64 payload from mixed cell",
            );
        }
        WasmRepr::Str { .. } => {
            ctx.fb.ins(
                "call $__rt_mixed_cast_string",
                "unbox string payload from mixed cell",
            );
            ctx.fb
                .ins("i64.extend_i32_u", "widen string length to i64");
        }
        WasmRepr::Ptr(_) => {
            let accepted_tag_test = match dest_ir {
                IrType::Heap(IrHeapKind::Array) => {
                    "(i64.eq (local.get {tag}) (i64.const 4))"
                }
                IrType::Heap(IrHeapKind::Hash) => {
                    "(i64.eq (local.get {tag}) (i64.const 5))"
                }
                IrType::Heap(IrHeapKind::Object) => {
                    "(i64.eq (local.get {tag}) (i64.const 6))"
                }
                IrType::Heap(IrHeapKind::Iterable) => concat!(
                    "(i32.or ",
                    "(i64.eq (local.get {tag}) (i64.const 4)) ",
                    "(i32.or ",
                    "(i64.eq (local.get {tag}) (i64.const 5)) ",
                    "(i64.eq (local.get {tag}) (i64.const 6))))"
                ),
                other => {
                    return Err(WasmError::Unsupported(format!(
                        "unboxing a Mixed cell to heap storage {:?} is not supported on wasm32-wasi",
                        other
                    )));
                }
            };
            let tag_tmp = ctx.fresh_temp(ValType::I64);
            let lo_tmp = ctx.fresh_temp(ValType::I64);
            let hi_tmp = ctx.fresh_temp(ValType::I64);
            // A ZERO cell pointer means the slot was never written, not that it holds PHP
            // `null` — a stored null is a real cell carrying tag 8. Slots start zeroed, and EIR
            // releases a local's previous value before overwriting it, so the very first write
            // to a Mixed-repr slot unboxes a slot that has never held a cell. Unboxing address
            // zero would read a tag out of the null page and fail the check below, turning
            // ordinary first assignment into a TypeError. Yielding the null pointer instead is
            // exactly what the consumer expects: `__rt_decref_any` treats it as a no-op.
            let cell_tmp = ctx.fresh_temp(ValType::I32);
            ctx.fb
                .ins(&format!("local.set {}", cell_tmp), "save mixed cell pointer");
            ctx.fb
                .ins(&format!("local.get {}", cell_tmp), "mixed cell pointer");
            ctx.fb.ins("i32.eqz", "slot never written?");
            ctx.fb.ins("if (result i32)", "unwritten slot yields the null pointer");
            ctx.fb.ins("i32.const 0", "null heap pointer for an unwritten slot");
            ctx.fb.ins("else", "the slot holds a real mixed cell");
            ctx.fb
                .ins(&format!("local.get {}", cell_tmp), "mixed cell to unbox");
            ctx.fb
                .ins("call $__rt_mixed_unbox", "unbox mixed cell -> (tag, lo, hi)");
            ctx.fb
                .ins(&format!("local.set {}", hi_tmp), "discard mixed high word");
            ctx.fb
                .ins(&format!("local.set {}", lo_tmp), "capture mixed low word");
            ctx.fb
                .ins(&format!("local.set {}", tag_tmp), "capture mixed tag");
            let accepted_tag_test = accepted_tag_test.replace("{tag}", &tag_tmp);
            ctx.fb
                .ins(&accepted_tag_test, "mixed tag belongs to destination kind");
            ctx.fb.ins("i32.eqz", "invert accepted-tag predicate");
            ctx.fb.ins("if", "mixed heap-kind mismatch?");
            if ctx
                .module
                .functions
                .iter()
                .any(|function| function.flags.is_main)
            {
                ctx.fb
                    .ins("i32.const 9", "Mixed heap-kind TypeError diagnostic");
                ctx.fb.ins(
                    "call $__rt_fail",
                    "raise deterministic PHP TypeError for the mismatch",
                );
                ctx.fb.ins(
                    "unreachable",
                    "elephc-trap:post-noreturn:mixed-heap-kind-mismatch runtime TypeError helper does not return",
                );
            } else {
                ctx.fb.ins(
                    "unreachable",
                    "elephc-trap:non-public:reactor-mixed-heap-mismatch import-free reactors are outside the public command surface",
                );
            }
            ctx.fb.ins("end", "end mixed heap-kind validation");
            ctx.fb
                .ins(&format!("local.get {}", lo_tmp), "load mixed heap pointer");
            ctx.fb
                .ins("i32.wrap_i64", "narrow payload pointer to i32");
            ctx.fb.ins("end", "end unwritten-slot guard");
        }
        WasmRepr::Tagged { .. } | WasmRepr::Void => {
            return Err(WasmError::Unsupported(format!(
                "unboxing a Mixed cell to {:?} is not supported on wasm32-wasi",
                dest_repr
            )));
        }
    }

    Ok(())
}

/// Emits the local.get sequence that pushes the destination representation of
/// `source_temps` onto the operand stack.
///
/// For identical representations this is a plain component-wise copy. For a
/// Mixed-cell destination this boxes (or moves) the value via
/// `__rt_mixed_from_value`.  Unsupported conversions return `WasmError`.
fn convert_temps_to_dest(
    ctx: &mut FnCtx,
    source_temps: &[String],
    source_repr: &WasmRepr,
    source_php: PhpType,
    source_ir: IrType,
    dest_repr: &WasmRepr,
    dest_ir: IrType,
    dest_php: PhpType,
) -> Result<()> {
    match classify_transfer(source_ir, source_php.clone(), dest_ir, dest_php.clone())? {
        TransferKind::Copy => {
            for name in source_temps {
                ctx.fb
                    .ins(&format!("local.get {}", name), "copy value component");
            }
            Ok(())
        }
        TransferKind::BoxMixed => emit_box_temps_into_mixed(
            ctx,
            source_temps,
            source_repr,
            source_php,
            source_ir,
        ),
        TransferKind::UnboxMixed => {
            emit_unbox_mixed_to_concrete(ctx, source_temps, dest_repr, dest_ir, &dest_php)
        }
    }
}

/// Boxes the value described by `source_temps` into a fresh owned Mixed cell via
/// `__rt_mixed_from_value`, leaving the i32 cell pointer on the operand stack.
fn emit_box_temps_into_mixed(
    ctx: &mut FnCtx,
    source_temps: &[String],
    source_repr: &WasmRepr,
    source_php: PhpType,
    source_ir: IrType,
) -> Result<()> {
    let tag = mixed_box_tag(&source_php, source_ir)?;
    ctx.fb
        .ins(&format!("i64.const {}", tag), "mixed boxing tag");

    match source_repr {
        WasmRepr::I64(_) => {
            ctx.fb.ins(
                &format!("local.get {}", source_temps[0]),
                "scalar payload lo",
            );
            ctx.fb.ins("i64.const 0", "scalar payload hi unused");
        }
        WasmRepr::F64(_) => {
            ctx.fb.ins(
                &format!("local.get {}", source_temps[0]),
                "float payload",
            );
            ctx.fb.ins("i64.reinterpret_f64", "float bits -> i64 lo");
            ctx.fb.ins("i64.const 0", "float payload hi unused");
        }
        WasmRepr::Str { .. } => {
            let ptr = &source_temps[0];
            let len = &source_temps[1];
            ctx.fb
                .ins(&format!("local.get {}", ptr), "string pointer lo");
            ctx.fb.ins("i64.extend_i32_u", "ptr -> i64 lo");
            ctx.fb
                .ins(&format!("local.get {}", len), "string length hi");
        }
        WasmRepr::Ptr(_) => {
            ctx.fb
                .ins(&format!("local.get {}", source_temps[0]), "heap pointer lo");
            ctx.fb.ins("i64.extend_i32_u", "ptr -> i64 lo");
            ctx.fb.ins("i64.const 0", "heap payload hi unused");
        }
        WasmRepr::Tagged { .. } => {
            return Err(WasmError::Unsupported(
                "boxing a tagged scalar into a Mixed cell is not supported on wasm32-wasi"
                    .to_string(),
            ));
        }
        WasmRepr::Void => {
            ctx.fb.ins(
                &format!("i64.const {}", VOID_SENTINEL),
                "void/null sentinel",
            );
            ctx.fb.ins("i64.const 0", "null hi unused");
        }
    }

    ctx.fb
        .ins("call $__rt_mixed_from_value", "box into owned mixed cell");
    Ok(())
}

/// Stores a stack-captured value into a destination EIR value, applying any
/// required representation conversion.
pub(super) fn emit_store_temps_into_value(
    ctx: &mut FnCtx,
    source_temps: &[String],
    source_repr: &WasmRepr,
    source_php: PhpType,
    source_ir: IrType,
    dest_value: ValueId,
) -> Result<()> {
    let dest_repr = ctx.value_repr(dest_value)?.clone();
    let (dest_php, dest_ir) = ctx
        .function
        .value(dest_value)
        .map(|v| (v.php_type.codegen_repr(), v.ir_type))
        .ok_or_else(|| {
            WasmError::Unsupported(format!("destination value {:?} missing", dest_value))
        })?;

    convert_temps_to_dest(
        ctx,
        source_temps,
        source_repr,
        source_php,
        source_ir,
        &dest_repr,
        dest_ir,
        dest_php,
    )?;

    for local_ref in dest_repr.local_refs().iter().rev() {
        ctx.fb
            .ins(&format!("local.set {}", local_ref), "store destination value component");
    }
    Ok(())
}

/// Stores a stack-captured value into a local slot, applying any required
/// representation conversion.
pub(super) fn emit_store_temps_into_slot(
    ctx: &mut FnCtx,
    source_temps: &[String],
    source_repr: &WasmRepr,
    source_php: PhpType,
    source_ir: IrType,
    dest_slot: LocalSlotId,
) -> Result<()> {
    let dest_repr = ctx.slot_repr(dest_slot)?.clone();
    let (dest_php, dest_ir) = ctx
        .function
        .locals
        .get(dest_slot.as_raw() as usize)
        .map(|local| (local.php_type.codegen_repr(), local.ir_type))
        .ok_or_else(|| {
            WasmError::Unsupported(format!("destination slot {:?} missing", dest_slot))
        })?;

    convert_temps_to_dest(
        ctx,
        source_temps,
        source_repr,
        source_php,
        source_ir,
        &dest_repr,
        dest_ir,
        dest_php,
    )?;

    for local_ref in dest_repr.local_refs().iter().rev() {
        ctx.fb
            .ins(&format!("local.set {}", local_ref), "store destination slot component");
    }
    Ok(())
}

/// Builds a placeholder `WasmRepr` for an EIR type.
///
/// Only the variant shape matters for conversion decisions; the local names are
/// never emitted.
fn repr_for_ir(ir: IrType) -> WasmRepr {
    match ir {
        IrType::I64 => WasmRepr::I64(String::new()),
        IrType::F64 => WasmRepr::F64(String::new()),
        IrType::Heap(_) => WasmRepr::Ptr(String::new()),
        IrType::Str => WasmRepr::Str {
            ptr: String::new(),
            len: String::new(),
        },
        IrType::TaggedScalar => WasmRepr::Tagged {
            payload: String::new(),
            tag: String::new(),
        },
        IrType::Void => WasmRepr::Void,
    }
}

/// Loads a call argument and pushes it in the callee parameter's representation.
///
/// Applies concrete-to-Mixed boxing or Mixed-to-concrete unboxing as needed so
/// the operand stack matches the callee's declared parameter signature.
pub(super) fn emit_push_call_argument(
    ctx: &mut FnCtx,
    arg: ValueId,
    param_ir: IrType,
    param_php: PhpType,
) -> Result<()> {
    let source_repr = ctx.value_repr(arg)?.clone();
    let (source_php, source_ir) = ctx
        .function
        .value(arg)
        .map(|v| (v.php_type.codegen_repr(), v.ir_type))
        .ok_or_else(|| WasmError::Unsupported(format!("call arg {:?} missing", arg)))?;
    let temps = load_value_to_temps(ctx, arg)?;
    let dest_repr = repr_for_ir(param_ir);
    convert_temps_to_dest(
        ctx,
        &temps,
        &source_repr,
        source_php,
        source_ir,
        &dest_repr,
        param_ir,
        param_php,
    )
}

/// Pops a value of the given EIR representation from the operand stack and
/// stores it into `dest_value`, applying any required conversion.
pub(super) fn emit_store_stack_value_into_value(
    ctx: &mut FnCtx,
    source_ir: IrType,
    source_php: PhpType,
    dest_value: ValueId,
) -> Result<()> {
    let val_types = WasmRepr::val_types(source_ir);
    let temps = pop_stack_to_temps(ctx, &val_types)?;
    let source_repr = repr_from_temps(source_ir, &temps);
    emit_store_temps_into_value(ctx, &temps, &source_repr, source_php, source_ir, dest_value)
}

/// Pops a value of the given EIR representation from the operand stack and
/// stores it into `dest_slot`, applying any required conversion.
pub(super) fn emit_store_stack_value_into_slot(
    ctx: &mut FnCtx,
    source_ir: IrType,
    source_php: PhpType,
    dest_slot: LocalSlotId,
) -> Result<()> {
    let val_types = WasmRepr::val_types(source_ir);
    let temps = pop_stack_to_temps(ctx, &val_types)?;
    let source_repr = repr_from_temps(source_ir, &temps);
    emit_store_temps_into_slot(ctx, &temps, &source_repr, source_php, source_ir, dest_slot)
}

/// Transfers `src_value` into `dest_value`, loading the source into temps first
/// so the move is safe even when the values share a local.
pub(super) fn emit_transfer_value(
    ctx: &mut FnCtx,
    src_value: ValueId,
    dest_value: ValueId,
) -> Result<()> {
    let source_repr = ctx.value_repr(src_value)?.clone();
    let (source_php, source_ir) = ctx
        .function
        .value(src_value)
        .map(|v| (v.php_type.codegen_repr(), v.ir_type))
        .ok_or_else(|| WasmError::Unsupported(format!("source value {:?} missing", src_value)))?;
    let temps = load_value_to_temps(ctx, src_value)?;
    emit_store_temps_into_value(ctx, &temps, &source_repr, source_php, source_ir, dest_value)
}

/// Transfers `src_value` into `dest_slot`, loading the source into temps first.
pub(super) fn emit_transfer_to_slot(
    ctx: &mut FnCtx,
    src_value: ValueId,
    dest_slot: LocalSlotId,
) -> Result<()> {
    let source_repr = ctx.value_repr(src_value)?.clone();
    let (source_php, source_ir) = ctx
        .function
        .value(src_value)
        .map(|v| (v.php_type.codegen_repr(), v.ir_type))
        .ok_or_else(|| WasmError::Unsupported(format!("source value {:?} missing", src_value)))?;
    let temps = load_value_to_temps(ctx, src_value)?;
    emit_store_temps_into_slot(ctx, &temps, &source_repr, source_php, source_ir, dest_slot)
}

/// Transfers the contents of `src_slot` into `dest_value`.
pub(super) fn emit_transfer_from_slot(
    ctx: &mut FnCtx,
    src_slot: LocalSlotId,
    dest_value: ValueId,
) -> Result<()> {
    let source_repr = ctx.slot_repr(src_slot)?.clone();
    let (source_php, source_ir) = ctx
        .function
        .locals
        .get(src_slot.as_raw() as usize)
        .map(|local| (local.php_type.codegen_repr(), local.ir_type))
        .ok_or_else(|| WasmError::Unsupported(format!("source slot {:?} missing", src_slot)))?;
    let temps = load_slot_to_temps(ctx, src_slot)?;
    emit_store_temps_into_value(ctx, &temps, &source_repr, source_php, source_ir, dest_value)
}

/// Materializes block arguments for a branch, loading every source into temps
/// before storing any destination so parallel moves remain safe.
pub(super) fn emit_transfer_block_args(
    ctx: &mut FnCtx,
    target: crate::ir::BlockId,
    args: &[ValueId],
) -> Result<()> {
    let target_block = ctx
        .function
        .block(target)
        .ok_or_else(|| WasmError::Unsupported(format!("target block {:?} not found", target)))?;
    let params = &target_block.params;

    if args.len() != params.len() {
        return Err(WasmError::Unsupported(format!(
            "branch arg count {} != param count {}",
            args.len(),
            params.len()
        )));
    }

    let mut arg_temps: Vec<Vec<String>> = Vec::with_capacity(args.len());
    let mut arg_infos: Vec<(WasmRepr, PhpType, IrType)> = Vec::with_capacity(args.len());
    for &arg in args {
        let repr = ctx.value_repr(arg)?.clone();
        let val = ctx
            .function
            .value(arg)
            .ok_or_else(|| WasmError::Unsupported(format!("branch arg {:?} missing", arg)))?;
        let source_php = val.php_type.codegen_repr();
        let source_ir = val.ir_type;
        let temps = load_value_to_temps(ctx, arg)?;
        arg_temps.push(temps);
        arg_infos.push((repr, source_php, source_ir));
    }

    for (i, &param) in params.iter().enumerate() {
        let temps = &arg_temps[i];
        let (source_repr, source_php, source_ir) = &arg_infos[i];
        emit_store_temps_into_value(
            ctx,
            temps,
            source_repr,
            source_php.clone(),
            *source_ir,
            param,
        )?;
    }

    Ok(())
}

/// Stores a function-call result into the instruction's result value.
///
/// For a void callee this materializes Elephc's null sentinel and boxes it when
/// the destination is a Mixed cell. Concrete return values are popped from the
/// operand stack and converted as needed.
pub(super) fn emit_store_call_result(
    ctx: &mut FnCtx,
    inst: &crate::ir::Instruction,
    return_type: IrType,
    return_php_type: PhpType,
) -> Result<()> {
    if let Some(result) = inst.result {
        if return_type == IrType::Void {
            let sentinel = ctx.fresh_temp(ValType::I64);
            ctx.fb.ins(
                &format!("i64.const {}", VOID_SENTINEL),
                "Elephc null sentinel for void callee result",
            );
            ctx.fb
                .ins(&format!("local.set {}", sentinel), "capture void sentinel");
            let source_repr = WasmRepr::I64(sentinel.clone());
            // The source is described as `I64`, not `Void`: nothing came back from the callee,
            // but what this transfer actually carries is the i64 sentinel just materialized.
            // Describing it as `Void` would classify a `WasmRepr::Void` (zero locals) against
            // the destination and reject the very store this branch exists to perform.
            emit_store_temps_into_value(
                ctx,
                &[sentinel],
                &source_repr,
                PhpType::Void,
                IrType::I64,
                result,
            )?;
        } else {
            emit_store_stack_value_into_value(ctx, return_type, return_php_type, result)?;
        }
    }
    Ok(())
}
