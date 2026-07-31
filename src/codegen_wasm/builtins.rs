//! Purpose:
//! Lowers PHP builtin functions that compile to a handful of WebAssembly instructions with no
//! runtime helper and no allocation, plus the audit contract each of them is admitted under.
//!
//! Called from:
//! - `crate::codegen_wasm::inst::lower_runtime_call` for emission.
//! - `crate::codegen_wasm::capability::runtime_function_shape_issue` for the static audit.
//!
//! Key details:
//! - Everything here is an EXACT identity: the WebAssembly instruction and the PHP builtin agree
//!   on every input including NaN, both infinities and negative zero, so there is no diagnostic
//!   to emit and no profile to branch on. A builtin that needs a table, an allocation or a
//!   warning does not belong in this module.
//! - The audit and the emitter read the same operand contract, so a shape the emitter cannot
//!   lower is refused before planning rather than producing an invalid module.

use super::context::{FnCtx, Result};
use super::inst::{operand, store_result};
use super::WasmError;
use crate::ir::{Function, Instruction, IrHeapKind, IrType, RuntimeFnId};
use crate::types::PhpType;

/// The storage one direct builtin accepts and produces.
///
/// Both the audit and the emitter derive from this single description, which is what keeps a
/// newly admitted builtin from being auditable but unlowerable, or the reverse.
struct DirectSignature {
    /// EIR type every operand must carry.
    operand_ir: IrType,
    /// PHP type every operand must carry, after `codegen_repr`.
    operand_php: PhpType,
    /// EIR type the result must carry.
    result_ir: IrType,
    /// PHP type the result must carry, after `codegen_repr`.
    result_php: PhpType,
}

/// Returns the signature and WebAssembly instruction for a builtin lowered inline, or `None`
/// when the builtin needs a runtime helper.
///
/// `count` is absent from the instruction column because it is a memory load rather than an
/// arithmetic operation; it is handled separately by [`lower_count`].
fn direct_builtin(target: RuntimeFnId, operand_php: &PhpType) -> Option<(DirectSignature, &'static str)> {
    let float = |instruction| {
        Some((
            DirectSignature {
                operand_ir: IrType::F64,
                operand_php: PhpType::Float,
                result_ir: IrType::F64,
                result_php: PhpType::Float,
            },
            instruction,
        ))
    };
    match target {
        // `abs` is the one entry whose storage depends on its argument: PHP keeps an integer
        // argument integral and a float one floating.
        RuntimeFnId::Abs => match operand_php {
            PhpType::Int => Some((
                DirectSignature {
                    operand_ir: IrType::I64,
                    operand_php: PhpType::Int,
                    result_ir: IrType::I64,
                    result_php: PhpType::Int,
                },
                // WebAssembly has no i64 absolute value; the branchless form is
                // `(x ^ (x >> 63)) - (x >> 63)`, emitted by `lower_int_abs`.
                "",
            )),
            PhpType::Float => float("f64.abs"),
            _ => None,
        },
        RuntimeFnId::Floor => float("f64.floor"),
        RuntimeFnId::Ceil => float("f64.ceil"),
        RuntimeFnId::Sqrt => float("f64.sqrt"),
        _ => None,
    }
}

/// Returns whether `target` is lowered inline by this module.
pub(super) fn is_direct_builtin(target: RuntimeFnId) -> bool {
    matches!(
        target,
        RuntimeFnId::Abs
            | RuntimeFnId::Floor
            | RuntimeFnId::Ceil
            | RuntimeFnId::Sqrt
            | RuntimeFnId::Count
    )
}

/// Validates one direct builtin's operand and result storage before planning.
pub(super) fn direct_builtin_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    if target == RuntimeFnId::Count {
        return count_shape_issue(function, call);
    }
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("operand is missing from the value table".to_string());
    };
    let operand_php = value.php_type.codegen_repr();
    let Some((signature, _)) = direct_builtin(target, &operand_php) else {
        return Some(format!(
            "no inline lowering for a {operand_php:?} argument"
        ));
    };
    if value.ir_type != signature.operand_ir || operand_php != signature.operand_php {
        return Some(format!(
            "operand {:?}/{operand_php:?} is not the expected {:?}/{:?}",
            value.ir_type, signature.operand_ir, signature.operand_php
        ));
    }
    if call.result.is_none()
        || call.result_type != signature.result_ir
        || call.result_php_type.codegen_repr() != signature.result_php
    {
        return Some(format!(
            "result {:?}/{:?} is not the expected {:?}/{:?}",
            call.result_type,
            call.result_php_type.codegen_repr(),
            signature.result_ir,
            signature.result_php
        ));
    }
    None
}

/// Validates `count($array)` against the one shape its load can serve.
///
/// The length is read straight from the container header, so the operand has to be a container
/// this backend allocated. PHP's `count()` of a non-countable value is a `TypeError`, which a
/// header load cannot raise, so any other operand type is refused rather than answering nonsense.
fn count_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one container operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("container operand is missing from the value table".to_string());
    };
    if !matches!(
        value.ir_type,
        IrType::Heap(IrHeapKind::Array | IrHeapKind::Hash)
    ) || !matches!(
        value.php_type.codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. }
    ) {
        return Some(format!(
            "expected a statically typed array or hash, got {:?}/{:?}",
            value.ir_type,
            value.php_type.codegen_repr()
        ));
    }
    if call.result.is_none()
        || call.result_type != IrType::I64
        || call.result_php_type.codegen_repr() != PhpType::Int
    {
        return Some(format!(
            "result {:?}/{:?} is not the expected I64/Int",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Lowers one direct builtin.
pub(super) fn lower_direct_builtin(
    ctx: &mut FnCtx,
    inst: &Instruction,
    target: RuntimeFnId,
) -> Result<()> {
    if target == RuntimeFnId::Count {
        return lower_count(ctx, inst);
    }
    let argument = operand(inst, 0)?;
    let operand_php = ctx.value_php_type(argument)?.codegen_repr();
    let Some((_, instruction)) = direct_builtin(target, &operand_php) else {
        return Err(WasmError::Unsupported(format!(
            "builtin {:?} over a {operand_php:?} argument",
            target
        )));
    };
    if target == RuntimeFnId::Abs && operand_php == PhpType::Int {
        return lower_int_abs(ctx, inst, argument);
    }
    ctx.emit_load_value(argument)?;
    ctx.fb.ins(instruction, "PHP builtin lowered inline");
    store_result(ctx, inst)
}

/// Lowers `abs($int)` branchlessly as `(x ^ (x >> 63)) - (x >> 63)`.
///
/// KNOWN DIVERGENCE, shared with the native backend and rooted in EIR rather than in either
/// emitter: PHP promotes `abs(PHP_INT_MIN)` to the float `9.2233720368548E+18`, because its
/// magnitude has no integer representation. EIR types this call `I64`/`int`, so there is no slot
/// a float could be returned in, and both backends therefore answer `PHP_INT_MIN` unchanged.
/// Every other input is exact.
fn lower_int_abs(ctx: &mut FnCtx, inst: &Instruction, argument: crate::ir::ValueId) -> Result<()> {
    let mask = ctx.fresh_temp(super::wat::ValType::I64);
    ctx.emit_load_value(argument)?;
    ctx.fb.ins("i64.const 63", "sign-bit shift distance");
    ctx.fb
        .ins("i64.shr_s", "all ones for a negative argument, zero otherwise");
    ctx.fb.ins(&format!("local.tee {}", mask), "keep the sign mask");
    ctx.emit_load_value(argument)?;
    ctx.fb.ins("i64.xor", "conditionally invert the argument");
    ctx.fb.ins(&format!("local.get {}", mask), "the sign mask again");
    ctx.fb.ins("i64.sub", "add one back for a negative argument");
    store_result(ctx, inst)
}

/// Lowers `count($array)` to the container header's element count at `[ptr + 0]`.
fn lower_count(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb.ins("i64.load", "container element count @ +0");
    store_result(ctx, inst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Builder, Immediate, Op, Ownership, RuntimeCallTarget};

    /// Builds a one-instruction function calling `target` with one operand of the given storage.
    fn call_with(
        target: RuntimeFnId,
        operand_ir: IrType,
        operand_php: PhpType,
        result_ir: IrType,
        result_php: PhpType,
    ) -> Function {
        let mut function = Function::new("probe".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let slot = builder.add_local(
                Some("v".to_string()),
                operand_ir,
                operand_php.clone(),
                crate::ir::LocalKind::PhpLocal,
            );
            let argument = builder.emit_load_local(slot, operand_ir, operand_php);
            builder.emit(
                Op::RuntimeCall,
                vec![argument],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(target))),
                result_ir,
                result_php,
                Ownership::NonHeap,
            );
            builder.terminate(crate::ir::Terminator::Return { value: None });
        }
        function
    }

    /// Returns the audit verdict for the last instruction of `function`.
    fn verdict(function: &Function, target: RuntimeFnId) -> Option<String> {
        let call = function
            .instructions
            .last()
            .expect("the probe emitted a call");
        direct_builtin_shape_issue(function, call, target)
    }

    /// Verifies each inline builtin admits exactly the storage its lowering can emit.
    ///
    /// `RuntimeFnId::Floor`, `RuntimeFnId::Ceil` and `RuntimeFnId::Sqrt` are float-only;
    /// `RuntimeFnId::Abs` accepts both widths and must keep an integral argument integral; and
    /// `RuntimeFnId::Count` reads a container header, so a scalar operand has to be refused
    /// rather than loading whatever lies at that address.
    #[test]
    fn direct_builtins_admit_only_the_storage_they_lower() {
        for target in [RuntimeFnId::Floor, RuntimeFnId::Ceil, RuntimeFnId::Sqrt] {
            let ok = call_with(target, IrType::F64, PhpType::Float, IrType::F64, PhpType::Float);
            assert_eq!(verdict(&ok, target), None, "{target:?} over a float");

            let bad = call_with(target, IrType::I64, PhpType::Int, IrType::I64, PhpType::Int);
            assert!(
                verdict(&bad, target).is_some(),
                "{target:?} has no integral lowering"
            );
        }

        let int_abs = call_with(
            RuntimeFnId::Abs,
            IrType::I64,
            PhpType::Int,
            IrType::I64,
            PhpType::Int,
        );
        assert_eq!(verdict(&int_abs, RuntimeFnId::Abs), None);
        let float_abs = call_with(
            RuntimeFnId::Abs,
            IrType::F64,
            PhpType::Float,
            IrType::F64,
            PhpType::Float,
        );
        assert_eq!(verdict(&float_abs, RuntimeFnId::Abs), None);
        let widened_abs = call_with(
            RuntimeFnId::Abs,
            IrType::I64,
            PhpType::Int,
            IrType::F64,
            PhpType::Float,
        );
        assert!(
            verdict(&widened_abs, RuntimeFnId::Abs).is_some(),
            "an integral argument must not claim a float result"
        );

        let counted = call_with(
            RuntimeFnId::Count,
            IrType::Heap(IrHeapKind::Array),
            PhpType::Array(Box::new(PhpType::Int)),
            IrType::I64,
            PhpType::Int,
        );
        assert_eq!(verdict(&counted, RuntimeFnId::Count), None);
        let scalar_count = call_with(
            RuntimeFnId::Count,
            IrType::I64,
            PhpType::Int,
            IrType::I64,
            PhpType::Int,
        );
        assert!(
            verdict(&scalar_count, RuntimeFnId::Count).is_some(),
            "count() of a scalar is a PHP TypeError a header load cannot raise"
        );
    }
}
