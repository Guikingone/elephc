//! Purpose:
//! Maps compiler-private SPL value snapshots to target-aware runtime helpers.
//!
//! Called from:
//! - The typed runtime-operation dispatcher.
//!
//! Key details:
//! - Snapshot handles are opaque integers, never PHP-visible pointers or owners.
//! - Snapshot input is borrowed; reads return an acquired Mixed owner.

use crate::codegen::{context::FunctionContext, CodegenIrError, Result};
use crate::codegen_support::abi;
use crate::ir::{Instruction, RuntimeCallTarget};

/// Emits one private unary helper using the platform's C argument register.
pub(super) fn lower(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    target: RuntimeCallTarget,
) -> Result<()> {
    let entry = match target {
        RuntimeCallTarget::BorrowedValueSnapshot => "__rt_borrowed_value_snapshot",
        RuntimeCallTarget::BorrowedValueRead => "__rt_borrowed_value_read",
        RuntimeCallTarget::BorrowedValueFree => "__rt_borrowed_value_free",
        _ => return Err(CodegenIrError::invalid_module("invalid borrowed-value target")),
    };
    let value = super::expect_operand(inst, 0)?;
    ctx.load_value_to_reg(value, abi::int_arg_reg_name(ctx.emitter.target, 0))?;
    abi::emit_call_label(ctx.emitter, entry);
    if let Some(result) = inst.result {
        ctx.store_result_value(result)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_user_asm_from_ir;
    use crate::codegen_support::platform::Target;
    use crate::ir::{Builder, Function, FunctionParam, Immediate, IrHeapKind, IrType, LocalKind,
        Module, Op, Ownership, RuntimeCallTarget, Terminator};
    use crate::types::PhpType;

    /// Validates and lowers every private typed target on the complete supported target matrix.
    #[test]
    fn test_borrowed_value_targets_lower_on_all_supported_targets() {
        for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
            let target = Target::parse(name).unwrap();
            for (id, input, output, entry) in [
                (RuntimeCallTarget::BorrowedValueSnapshot, PhpType::Mixed, PhpType::Int, "__rt_borrowed_value_snapshot"),
                (RuntimeCallTarget::BorrowedValueRead, PhpType::Int, PhpType::Mixed, "__rt_borrowed_value_read"),
                (RuntimeCallTarget::BorrowedValueFree, PhpType::Int, PhpType::Void, "__rt_borrowed_value_free"),
            ] {
                let ir_input = if input == PhpType::Int { IrType::I64 } else { IrType::Heap(IrHeapKind::Mixed) };
                let ir_output = match output { PhpType::Int => IrType::I64, PhpType::Void => IrType::Void, _ => IrType::Heap(IrHeapKind::Mixed) };
                let mut module = Module::new(target);
                let mut main = Function::new("{main}".to_string(), IrType::Void, PhpType::Void);
                main.flags.is_main = true;
                let mut builder = Builder::new(&mut main);
                let block = builder.create_block_with_params(Vec::new());
                builder.set_entry(block);
                builder.position_at_end(block);
                builder.terminate(Terminator::Return { value: None });
                module.add_function(main);
                let mut function = Function::new("borrowed_test".to_string(), ir_output, output.clone());
                function.params.push(FunctionParam { name: "input".to_string(), ir_type: ir_input,
                    php_type: input.clone(), by_ref: false, variadic: false });
                let slot = function.add_local(Some("input".to_string()), ir_input, input.clone(), LocalKind::BorrowedTemp);
                function.no_epilogue_cleanup_slots.insert(slot);
                let mut builder = Builder::new(&mut function);
                let block = builder.create_block_with_params(Vec::new());
                builder.set_entry(block);
                builder.position_at_end(block);
                let input_ownership = if input == PhpType::Int { Ownership::NonHeap } else { Ownership::Borrowed };
                let value = builder.emit(Op::LoadLocal, Vec::new(), Some(Immediate::LocalSlot(slot)),
                    ir_input, input, input_ownership).unwrap();
                let result = builder.emit_with_effects(Op::RuntimeCall, vec![value],
                    Some(Immediate::RuntimeCall(id)), ir_output, output,
                    if id == RuntimeCallTarget::BorrowedValueRead { Ownership::Owned } else { Ownership::NonHeap },
                    crate::ir::Effects::from_bits_retain(
                        crate::ir::Effects::READS_HEAP.bits() | crate::ir::Effects::WRITES_HEAP.bits()
                        | crate::ir::Effects::ALLOC_HEAP.bits() | crate::ir::Effects::REFCOUNT_OP.bits()
                        | crate::ir::Effects::MAY_FATAL.bits()), None);
                builder.terminate(Terminator::Return { value: result });
                module.add_function(function);
                crate::ir::validate_module(&module)
                    .unwrap_or_else(|error| panic!("{name}: {id:?}: {error:?}"));
                let assembly = generate_user_asm_from_ir(&module, false, false)
                    .unwrap_or_else(|error| panic!("{name}: {id:?}: {error}"));
                assert!(assembly.contains(entry), "{name}: {id:?} has no runtime implementation");
            }
        }
    }
}
