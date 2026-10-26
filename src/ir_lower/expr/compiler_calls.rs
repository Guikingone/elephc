//! Purpose:
//! Lowers typed compiler-generated calls without PHP function or method declarations.
//!
//! Called from:
//! - Direct call lowering before any PHP name lookup or argument planning.
//!
//! Key details:
//! - Ordinary source spellings cannot select these operations.
//! - Snapshots borrow stable local storage; no boxing temporary may become the captured owner.

use super::*;
use crate::ir::{RuntimeCallSignature, RuntimeCallTarget};
use crate::names::CompilerCall;

/// Emits one trusted compiler operation with its fixed logical storage signature.
pub(super) fn lower(
    ctx: &mut LoweringContext<'_, '_>,
    operation: CompilerCall,
    args: &[Expr],
    expr: &Expr,
) -> LoweredValue {
    let target = match operation {
        CompilerCall::NativePropertyReplace => RuntimeCallTarget::NativePropertyReplace,
        CompilerCall::NativePropertyRestore => RuntimeCallTarget::NativePropertyRestore,
        CompilerCall::BorrowedValueSnapshot => RuntimeCallTarget::BorrowedValueSnapshot,
        CompilerCall::BorrowedValueRead => RuntimeCallTarget::BorrowedValueRead,
        CompilerCall::BorrowedValueFree => RuntimeCallTarget::BorrowedValueFree,
        CompilerCall::HeapScopeBegin => RuntimeCallTarget::HeapScopeBegin,
        CompilerCall::HeapScopeRead => RuntimeCallTarget::HeapScopeRead,
        CompilerCall::HeapScopeRootWritten => RuntimeCallTarget::HeapScopeRootWritten,
        CompilerCall::HeapScopeEnd => RuntimeCallTarget::HeapScopeEnd,
        CompilerCall::HeapScopeReleaseRoot => RuntimeCallTarget::HeapScopeReleaseRoot,
        CompilerCall::ExceptionChainMixed => RuntimeCallTarget::ExceptionChainMixed,
    };
    let Some(RuntimeCallSignature::Fixed { parameters, result }) = target.signature() else {
        unreachable!("compiler operation has no fixed storage signature");
    };
    assert_eq!(args.len(), parameters.len(), "invalid compiler call arity: {operation:?}");
    let mut operands = Vec::with_capacity(args.len());
    for (index, (argument, expected)) in args.iter().zip(parameters).enumerate() {
        assert!(!matches!(argument.kind, ExprKind::NamedArg { .. } | ExprKind::Spread(_)),
            "compiler call operands must already be normalized");
        let mut operand = lower_expr(ctx, argument);
        if operation == CompilerCall::ExceptionChainMixed && index == 0
            && operand.ir_type == IrType::Heap(crate::ir::IrHeapKind::Mixed)
        {
            // The generated property loop reaches this operation only behind instanceof Throwable.
            // PHP's object cast remains boxed; use an owned payload lease for this raw-object ABI.
            operand = super::merge_temps::coerce_value_for_temp(
                ctx, operand, &PhpType::Object("Throwable".into()), argument.span,
            );
        }
        if *expected == IrType::I64 && operand.ir_type != IrType::I64 {
            operand = instanceof_coercions::coerce_to_int_at_span(ctx, operand, Some(argument.span));
        }
        if *expected == IrType::Heap(crate::ir::IrHeapKind::Mixed) && operand.ir_type != *expected {
            operand = ctx.box_value_as_mixed(operand, PhpType::Mixed, Some(argument.span));
        }
        assert_eq!(operand.ir_type, *expected, "invalid storage for compiler call {operation:?} operand {index}: {:?}", argument.kind);
        operands.push(operand);
    }
    if operation == CompilerCall::BorrowedValueSnapshot {
        assert!(matches!(args[0].kind, ExprKind::Variable(_))
            && !ctx.value_is_owning_temporary(operands[0]),
            "a borrowed snapshot requires stable local storage for the complete generated scope");
    }
    if matches!(operation, CompilerCall::NativePropertyRestore | CompilerCall::NativePropertyReplace | CompilerCall::HeapScopeBegin | CompilerCall::HeapScopeRead | CompilerCall::HeapScopeRootWritten | CompilerCall::HeapScopeReleaseRoot) {
        let PhpType::Object(class) = ctx.builder.value_php_type(operands[0].value).codegen_repr() else {
            panic!("heap scope receiver is not an object");
        };
        let info = ctx.classes.get(&class).expect("heap scope receiver metadata");
        assert!(crate::types::spl_heap_layout::has_native_prefix(info), "invalid native heap scope receiver: {class}");
    }
    if operation == CompilerCall::NativePropertyReplace {
        let PhpType::Object(class) = ctx.builder.value_php_type(operands[0].value).codegen_repr() else { unreachable!() };
        let info = &ctx.classes[&class];
        let ExprKind::IntLiteral(offset) = args[1].kind else { panic!("native slot offset must be a compiler literal"); };
        let index = usize::try_from(offset).ok().and_then(|offset| offset.checked_sub(8))
            .filter(|offset| offset % 16 == 0).map(|offset| offset / 16).expect("native user slot offset");
        assert!(info.native_property_contracts.contains_key(&index), "native store cannot address backing or reference storage");
        assert_eq!(info.properties[index].1.codegen_repr(), PhpType::Mixed);
    }
    if matches!(operation, CompilerCall::HeapScopeBegin | CompilerCall::HeapScopeRead) {
        let count = if operation == CompilerCall::HeapScopeBegin { 2 } else { 1 };
        for operand in operands.iter().skip(1).take(count) {
            assert_eq!(ctx.builder.value_php_type(operand.value).codegen_repr(),
                PhpType::Array(Box::new(PhpType::Mixed)), "heap scopes require packed Mixed storage");
        }
    }
    let effects = Effects::from_bits_retain(Effects::READS_HEAP.bits() | Effects::WRITES_HEAP.bits()
        | Effects::READS_GLOBAL.bits() | Effects::WRITES_GLOBAL.bits()
        | Effects::ALLOC_HEAP.bits() | Effects::REFCOUNT_OP.bits() | Effects::MAY_FATAL.bits());
    // A helper that unwinds skips the post-call releases below, so its owning temporaries
    // (the serialized property-name string in particular) are published as call operand
    // owners first; the catch handler then releases them exactly once.
    let mut roots = Vec::new();
    if operation.may_unwind() {
        for operand in &mut operands {
            let (rooted, root) = super::call_operand_owners::root_owned_call_operand(ctx, *operand, expr.span);
            *operand = rooted;
            if let Some(slot) = root { roots.push((rooted.value, slot)); }
        }
    }
    let inputs = operands.iter().map(|value| value.value).collect();
    let value = if result == IrType::Void {
        ctx.builder.emit_with_effects(Op::RuntimeCall, inputs,
            Some(Immediate::RuntimeCall(target)), IrType::Void, PhpType::Void,
            Ownership::NonHeap, effects, Some(expr.span));
        None
    } else {
        let value = ctx.emit_value(Op::RuntimeCall, inputs,
            Some(Immediate::RuntimeCall(target)), operation.result_type(), effects, Some(expr.span));
        if matches!(operation, CompilerCall::BorrowedValueRead | CompilerCall::HeapScopeRead | CompilerCall::NativePropertyReplace | CompilerCall::NativePropertyRestore) {
            ctx.builder.set_value_ownership(value.value, Ownership::Owned);
        }
        Some(value)
    };
    // Each operation acquires its own result owner. In particular a scope owns no backing
    // array, so argument-read owners must retire before next clears the first payload slot.
    for &(_, slot) in roots.iter().rev() {
        super::call_operand_owners::retire_owned_call_operand(ctx, slot, expr.span);
    }
    for operand in operands.into_iter().rev() {
        if roots.iter().any(|(value, _)| *value == operand.value) { continue; }
        if ctx.value_is_owning_temporary(operand) {
            ctx.builder.emit_with_effects(Op::Release, vec![operand.value], None,
                IrType::Void, PhpType::Void, Ownership::NonHeap,
                Op::Release.default_effects(), Some(expr.span));
        }
    }
    value.unwrap_or_else(|| lower_expr(ctx, &Expr::new(ExprKind::Null, expr.span)))
}
