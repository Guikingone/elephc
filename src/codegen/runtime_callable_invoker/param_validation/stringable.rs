//! Purpose:
//! Preserves PHP weak string parameter binding for runtime Stringable object arguments.
//!
//! Called from:
//! - The descriptor parameter preflight and Mixed-to-string coercion in the parent invoker.
//!
//! Key details:
//! - Preflight checks metadata without invoking user code; conversion calls __toString exactly once.
//! - The lookup uses the dense class-id table with bounds checks and leaves the target callable intact.

use super::*;

pub(super) fn accept_stringable_object(
    emitter: &mut Emitter,
    ctx: &mut InvokerEmitContext,
    valid: &str,
) {
    let result = abi::int_result_reg(emitter);
    let check = ctx.next_label("parameter_stringable_check");
    let next = ctx.next_label("parameter_stringable_next");
    abi::emit_load_temporary_stack_slot(emitter, result, 0);
    branch_equal(emitter, result, 6, &check);
    abi::emit_jump(emitter, &next);
    emitter.label(&check);
    abi::emit_load_temporary_stack_slot(emitter, result, 8);
    let entry = tostring_entry(emitter, ctx, &next);
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction(&format!("cbnz {entry}, {valid}")), // permit weak binding when an inherited or local __toString implementation exists
        Arch::X86_64 => {
            emitter.instruction(&format!("test {entry}, {entry}"));             // check the class's __toString entry without invoking user code
            emitter.instruction(&format!("jnz {valid}"));                       // permit weak binding only for Stringable objects
        }
    }
    emitter.label(&next);
}

/// Converts an already-validated borrowed Mixed value into a string result.
pub(in crate::codegen::runtime_callable_invoker) fn coerce_mixed_to_string(emitter: &mut Emitter, ctx: &mut InvokerEmitContext) {
    let result = abi::int_result_reg(emitter);
    let object = ctx.next_label("parameter_stringable_convert");
    let scalar = ctx.next_label("parameter_string_scalar_convert");
    let done = ctx.next_label("parameter_string_convert_done");
    abi::emit_push_reg(emitter, result);
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    branch_equal(emitter, result, 6, &object);
    emitter.label(&scalar);
    abi::emit_pop_reg(emitter, result);
    abi::emit_call_label(emitter, "__rt_mixed_cast_string");
    abi::emit_jump(emitter, &done);
    emitter.label(&object);
    let payload = if emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
    abi::emit_reg_move(emitter, result, payload);
    let missing = ctx.next_label("parameter_stringable_missing");
    let entry = tostring_entry(emitter, ctx, &missing);
    abi::emit_reg_move(emitter, abi::int_arg_reg_name(emitter.target, 0), result);
    abi::emit_call_reg(emitter, entry);
    abi::emit_release_temporary_stack(emitter, 16);
    abi::emit_jump(emitter, &done);
    // Only generic, untyped descriptor users can reach a missing __toString entry here.
    // Preserve their existing string-cast route rather than jumping through a null method.
    emitter.label(&missing);
    abi::emit_jump(emitter, &scalar);
    emitter.label(&done);
}

fn tostring_entry(emitter: &mut Emitter, ctx: &mut InvokerEmitContext, missing: &str) -> &'static str {
    let _ = ctx;
    let object = abi::int_result_reg(emitter);
    let (class, count, entry) = match emitter.target.arch {
        // x9 is scratched by symbol loads; keep the class id outside that register.
        Arch::AArch64 => ("x12", "x10", "x11"),
        Arch::X86_64 => ("r10", "r11", "r9"),
    };
    abi::emit_load_from_address(emitter, class, object, 0);
    abi::emit_load_symbol_to_reg(emitter, count, "_class_tostring_count", 0);
    emitter.instruction(&format!("cmp {class}, {count}"));                      // use an unsigned bound to reject negative and out-of-range class ids
    emitter.instruction(&format!("{} {missing}", if emitter.target.arch == Arch::AArch64 { "b.hs" } else { "jae" })); // never index outside the dense __toString table
    abi::emit_symbol_address(emitter, entry, "_class_tostring_ptrs");
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction(&format!("ldr {entry}, [{entry}, {class}, lsl #3]")), // resolve inherited __toString through the authoritative class metadata
        Arch::X86_64 => emitter.instruction(&format!("mov {entry}, QWORD PTR [{entry} + {class}*8]")), // resolve inherited __toString through the authoritative class metadata
    }
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction(&format!("cbz {entry}, {missing}")), // absent __toString is never an invocable conversion target
        Arch::X86_64 => {
            emitter.instruction(&format!("test {entry}, {entry}"));             // reject a class without a __toString implementation
            emitter.instruction(&format!("jz {missing}"));                      // avoid invoking an absent conversion target
        }
    }
    entry
}
