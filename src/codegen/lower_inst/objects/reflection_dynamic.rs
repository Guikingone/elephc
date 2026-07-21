//! Purpose:
//! Lowers dynamic `new ReflectionClass($runtimeName)` construction — a non-literal
//! string/Mixed/scalar reflected-name operand — through one shared, program-wide dispatcher
//! label (`_elephc_reflect_class_new_dynamic`) that case-folds the runtime query, compares it
//! against every closed-world class, and materializes the matched class's full compile-time
//! metadata via the same `emit_reflection_owner_object` bake the literal path uses. A miss
//! throws a catchable `\ReflectionException` with PHP's exact `Class "NAME" does not exist`
//! message; a non-`object|string`-coercible runtime tag throws a catchable `\TypeError`.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::reflection::lower_reflection_owner_new()` routes a
//!   non-literal `ReflectionClass` constructor operand here instead of the compile-time bake.
//! - `crate::codegen::block_emit::emit_module()` calls
//!   `emit_reflection_class_dynamic_dispatch_if_needed()` once, after per-function lowering.
//!
//! Key details:
//! - PHP weak-coerces int/float/bool/null first arguments to a class-name string (php -n
//!   verified: `new ReflectionClass(42)` → `ReflectionException: Class "42" does not exist`);
//!   array/resource/other tags throw a real `\TypeError`, never a `ReflectionException`.
//! - The dispatcher is a leaf-style helper (fp/lr prologue only); every value that must
//!   survive an intervening runtime call is parked on the temporary stack at a fixed offset.
//! - Object operands resolve their concrete runtime class name via `get_class` first, then
//!   dispatch on that name exactly like a string argument.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen::{CodegenIrError, Result};
use crate::codegen_support::data_section::DataSection;
use crate::codegen_support::emit::Emitter;
use crate::ir::{Function, Immediate, Instruction, IrType, Module, Op, ValueDef, ValueId};
use crate::names::php_symbol_key;
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::super::super::frame;
use super::super::super::shared_state::SharedCodegenState;
use super::reflection::{
    emit_reflection_owner_object, reflection_class_like_is_internal,
    reflection_class_metadata_for_name,
};

/// Assembly label of the shared, program-wide dynamic `ReflectionClass(name)` construction
/// dispatcher.
const DYNAMIC_CLASS_DISPATCH_LABEL: &str = "_elephc_reflect_class_new_dynamic";

/// Returns true when an EIR value is a compile-time `ConstStr`/`ConstClassName` literal — the
/// shape the literal reflection-construction path can resolve without any runtime dispatch.
pub(super) fn is_const_string_or_class_value(function: &Function, value: ValueId) -> bool {
    let Some(value_ref) = function.value(value) else {
        return false;
    };
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return false;
    };
    let Some(inst_ref) = function.instruction(inst) else {
        return false;
    };
    matches!(inst_ref.op, Op::ConstStr | Op::ConstClassName)
}

/// Lowers one dynamic-argument `new ReflectionClass($x)` call site: materializes the runtime
/// class-name query into the shared dispatcher's argument registers (weak-coercing scalar
/// operands to their `(string)` cast exactly like PHP) and calls
/// `DYNAMIC_CLASS_DISPATCH_LABEL`. Non-coercible statically-known types throw a catchable
/// `\TypeError` instead of erroring the compile.
pub(super) fn lower_reflection_class_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name_operand: ValueId,
) -> Result<()> {
    match ctx.value_php_type(name_operand)?.codegen_repr() {
        PhpType::Str => {
            match ctx.emitter.target.arch {
                Arch::AArch64 => ctx.load_string_value_to_regs(name_operand, "x0", "x1")?,
                Arch::X86_64 => ctx.load_string_value_to_regs(name_operand, "rdi", "rsi")?,
            };
            abi::emit_call_label(ctx.emitter, DYNAMIC_CLASS_DISPATCH_LABEL);
        }
        PhpType::Object(_) => {
            ctx.load_value_to_result(name_operand)?;
            super::super::builtins::types::emit_dynamic_object_class_name(ctx, "get_class");
            emit_dispatch_call_from_string_result_regs(ctx);
        }
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_result(name_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            emit_reflection_class_dynamic_dispatch_from_mixed_tag(ctx)?;
        }
        // A statically int/float/bool-typed argument is a raw scalar in the ABI result
        // register(s); PHP's runtime weak-coercion rule applies identically, so these get the
        // same `(string)` cast + dispatch treatment inline, without any unboxing step.
        PhpType::Int => {
            ctx.load_value_to_result(name_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_itoa");
            emit_dispatch_call_from_string_result_regs(ctx);
        }
        PhpType::Float => {
            ctx.load_value_to_result(name_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_ftoa");
            emit_dispatch_call_from_string_result_regs(ctx);
        }
        PhpType::Bool => {
            ctx.load_value_to_result(name_operand)?;
            emit_reflection_loaded_bool_to_string(ctx);
            emit_dispatch_call_from_string_result_regs(ctx);
        }
        // A `void`/`never`-typed argument only arises from a degenerate expression; treat it
        // like PHP's `null` weak-coercion (→ "").
        PhpType::Void | PhpType::Never => {
            let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
            abi::emit_load_int_immediate(ctx.emitter, ptr_reg, 0);
            abi::emit_load_int_immediate(ctx.emitter, len_reg, 0);
            emit_dispatch_call_from_string_result_regs(ctx);
        }
        // Any other statically-known type (array, resource, callable, …) is genuinely not
        // coercible to `object|string` in real PHP either — throw the same catchable
        // `\TypeError` rather than crash the compile on an unsupported internal error.
        _ => {
            ctx.load_value_to_result(name_operand)?;
            emit_reflection_class_argument_type_error_throw(ctx);
        }
    }
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Moves a (pointer, length) pair from the standard string-result register convention
/// (`abi::string_result_regs`: `x1`/`x2` on AArch64, `rax`/`rdx` on x86_64) into the
/// dispatcher's argument register convention (`x0`/`x1` / `rdi`/`rsi`), then calls
/// `DYNAMIC_CLASS_DISPATCH_LABEL`.
fn emit_dispatch_call_from_string_result_regs(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // move the resolved class-name pointer into the dispatcher's arg0
            ctx.emitter.instruction("mov x1, x2");                              // move the resolved class-name length into the dispatcher's arg1
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // move the resolved class-name pointer into the dispatcher's arg0
            ctx.emitter.instruction("mov rsi, rdx");                            // move the resolved class-name length into the dispatcher's arg1
        }
    }
    abi::emit_call_label(ctx.emitter, DYNAMIC_CLASS_DISPATCH_LABEL);
}

/// Weak-casts an already-loaded raw bool value (in the ABI int-result register) to `"1"`
/// (true) or `""` (false), leaving the result in `abi::string_result_regs`.
fn emit_reflection_loaded_bool_to_string(ctx: &mut FunctionContext<'_>) {
    let false_label = ctx.next_label("reflect_dyn_bool_str_false");
    let done_label = ctx.next_label("reflect_dyn_bool_str_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x0, {}", false_label));       // false weak-casts to an empty string
            abi::emit_call_label(ctx.emitter, "__rt_itoa");
            ctx.emitter.instruction(&format!("b {}", done_label));              // skip the empty-string fallback after true conversion
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("mov x2, #0");                              // false has zero string length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // test whether the loaded bool payload is false
            ctx.emitter.instruction(&format!("je {}", false_label));            // false weak-casts to an empty string
            abi::emit_call_label(ctx.emitter, "__rt_itoa");
            ctx.emitter.instruction(&format!("jmp {}", done_label));            // skip the empty-string fallback after true conversion
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("mov rdx, 0");                              // false has zero string length
        }
    }
    ctx.emitter.label(&done_label);
}

/// After `__rt_mixed_unbox` (tag in `x0`/`rax`, primary payload in `x1`/`rdi`, string length in
/// `x2`/`rdx`), branches on the runtime tag: string payloads dispatch directly; object payloads
/// resolve their concrete runtime class name first; int/float/bool/null weak-coerce to their
/// `(string)` cast (php -n verified: `new ReflectionClass(42)` → `ReflectionException:
/// Class "42" does not exist`, `null` → `Class "" does not exist`); any other tag (array,
/// resource, …) throws a catchable `\TypeError`.
fn emit_reflection_class_dynamic_dispatch_from_mixed_tag(
    ctx: &mut FunctionContext<'_>,
) -> Result<()> {
    let str_label = ctx.next_label("reflect_dyn_mixed_str");
    let object_label = ctx.next_label("reflect_dyn_mixed_obj");
    let int_label = ctx.next_label("reflect_dyn_mixed_int");
    let float_label = ctx.next_label("reflect_dyn_mixed_float");
    let bool_label = ctx.next_label("reflect_dyn_mixed_bool");
    let bool_false_label = ctx.next_label("reflect_dyn_mixed_bool_false");
    let null_label = ctx.next_label("reflect_dyn_mixed_null");
    let type_error_label = ctx.next_label("reflect_dyn_mixed_type_error");
    let done_label = ctx.next_label("reflect_dyn_mixed_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #1");                              // runtime tag 1 means the boxed union holds a string payload
            ctx.emitter.instruction(&format!("b.eq {}", str_label));            // a string payload is already (ptr, len) — dispatch directly
            ctx.emitter.instruction("cmp x0, #6");                              // runtime tag 6 means the boxed union holds an object payload
            ctx.emitter.instruction(&format!("b.eq {}", object_label));         // resolve the object's own runtime class name first
            ctx.emitter.instruction("cmp x0, #0");                              // runtime tag 0 means the boxed union holds an int payload
            ctx.emitter.instruction(&format!("b.eq {}", int_label));            // PHP weak-coerces int to string for this constructor
            ctx.emitter.instruction("cmp x0, #2");                              // runtime tag 2 means the boxed union holds a float payload
            ctx.emitter.instruction(&format!("b.eq {}", float_label));          // PHP weak-coerces float to string for this constructor
            ctx.emitter.instruction("cmp x0, #3");                              // runtime tag 3 means the boxed union holds a bool payload
            ctx.emitter.instruction(&format!("b.eq {}", bool_label));           // PHP weak-coerces bool to string for this constructor
            ctx.emitter.instruction("cmp x0, #8");                              // runtime tag 8 means the boxed union holds null
            ctx.emitter.instruction(&format!("b.eq {}", null_label));           // PHP weak-coerces null to "" for this constructor
            ctx.emitter.instruction(&format!("b {}", type_error_label));        // array/resource/other: not coercible to object|string
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 1");                              // runtime tag 1 means the boxed union holds a string payload
            ctx.emitter.instruction(&format!("je {}", str_label));              // a string payload is already (ptr, len) — dispatch directly
            ctx.emitter.instruction("cmp rax, 6");                              // runtime tag 6 means the boxed union holds an object payload
            ctx.emitter.instruction(&format!("je {}", object_label));           // resolve the object's own runtime class name first
            ctx.emitter.instruction("cmp rax, 0");                              // runtime tag 0 means the boxed union holds an int payload
            ctx.emitter.instruction(&format!("je {}", int_label));              // PHP weak-coerces int to string for this constructor
            ctx.emitter.instruction("cmp rax, 2");                              // runtime tag 2 means the boxed union holds a float payload
            ctx.emitter.instruction(&format!("je {}", float_label));            // PHP weak-coerces float to string for this constructor
            ctx.emitter.instruction("cmp rax, 3");                              // runtime tag 3 means the boxed union holds a bool payload
            ctx.emitter.instruction(&format!("je {}", bool_label));             // PHP weak-coerces bool to string for this constructor
            ctx.emitter.instruction("cmp rax, 8");                              // runtime tag 8 means the boxed union holds null
            ctx.emitter.instruction(&format!("je {}", null_label));             // PHP weak-coerces null to "" for this constructor
            ctx.emitter.instruction(&format!("jmp {}", type_error_label));      // array/resource/other: not coercible to object|string
        }
    }

    // -- tag 1 (string): __rt_mixed_unbox left (ptr, len) in x1/x2 (AArch64) or rdi/rdx
    //    (x86_64) — move into the dispatcher's argument convention and call it. --
    ctx.emitter.label(&str_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // move the unboxed string pointer into the dispatcher's arg0
            ctx.emitter.instruction("mov x1, x2");                              // move the unboxed string length into the dispatcher's arg1
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rdx");                            // move the unboxed string length into the dispatcher's arg1 (rdi already holds arg0)
        }
    }
    abi::emit_call_label(ctx.emitter, DYNAMIC_CLASS_DISPATCH_LABEL);
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 6 (object): resolve the object's concrete runtime class name, then dispatch. --
    ctx.emitter.label(&object_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction("mov x0, x1"),                 // move the unboxed object pointer into the class-name lookup register
        Arch::X86_64 => ctx.emitter.instruction("mov rax, rdi"),                // move the unboxed object pointer into the class-name lookup register
    }
    super::super::builtins::types::emit_dynamic_object_class_name(ctx, "get_class");
    emit_dispatch_call_from_string_result_regs(ctx);
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 0 (int): weak-cast the unboxed int payload to decimal text via __rt_itoa. --
    ctx.emitter.label(&int_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // move the unboxed int payload into __rt_itoa's argument register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, rdi");                            // move the unboxed int payload into __rt_itoa's argument register
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_itoa");
    emit_dispatch_call_from_string_result_regs(ctx);
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 2 (float): weak-cast the unboxed float bit-pattern payload to decimal text. --
    ctx.emitter.label(&float_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("fmov d0, x1");                             // move the unboxed float bit-pattern payload into the FP argument register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("movq xmm0, rdi");                          // move the unboxed float bit-pattern payload into the FP argument register
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_ftoa");
    emit_dispatch_call_from_string_result_regs(ctx);
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 3 (bool): weak-cast to "1" (true) or "" (false) — PHP casts false to "". --
    ctx.emitter.label(&bool_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x1, {}", bool_false_label));  // false skips straight to the empty-string result
            ctx.emitter.instruction("mov x0, x1");                              // move the true payload (1) into __rt_itoa's argument register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rdi, rdi");                           // false skips straight to the empty-string result
            ctx.emitter.instruction(&format!("je {}", bool_false_label));       // branch to the empty-string false arm
            ctx.emitter.instruction("mov rax, rdi");                            // move the true payload (1) into __rt_itoa's argument register
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_itoa");
    emit_dispatch_call_from_string_result_regs(ctx);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&bool_false_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, xzr");                             // false weak-casts to an empty string pointer
            ctx.emitter.instruction("mov x2, xzr");                             // false weak-casts to zero string length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("xor rax, rax");                            // false weak-casts to an empty string pointer
            ctx.emitter.instruction("xor rdx, rdx");                            // false weak-casts to zero string length
        }
    }
    emit_dispatch_call_from_string_result_regs(ctx);
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 8 (null): weak-casts to the empty string (php -n verified). --
    ctx.emitter.label(&null_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, xzr");                             // null weak-casts to an empty string pointer
            ctx.emitter.instruction("mov x2, xzr");                             // null weak-casts to zero string length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("xor rax, rax");                            // null weak-casts to an empty string pointer
            ctx.emitter.instruction("xor rdx, rdx");                            // null weak-casts to zero string length
        }
    }
    emit_dispatch_call_from_string_result_regs(ctx);
    abi::emit_jump(ctx.emitter, &done_label);

    // -- any other tag (array, resource, …): throw a catchable TypeError, never returns. --
    ctx.emitter.label(&type_error_label);
    emit_reflection_class_argument_type_error_throw(ctx);

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Throws a catchable `\TypeError` for a dynamic `ReflectionClass($x)` construction whose
/// runtime argument is neither a string nor an object nor a weak-coercible scalar. Stamps
/// `_spl_type_error_class_id` with PHP's core wording (the concrete "X given" runtime-type-name
/// suffix real PHP appends is a scoped, documented simplification). Never returns.
fn emit_reflection_class_argument_type_error_throw(ctx: &mut FunctionContext<'_>) {
    emit_reflection_dynamic_type_error_throw(
        ctx,
        b"ReflectionClass::__construct(): Argument #1 ($objectOrClass) must be of type object|string",
    );
}

/// Throws a catchable `\TypeError` carrying `message` through the standard exception
/// machinery: persist the message, allocate the compact Throwable payload, stamp
/// `_spl_type_error_class_id`, publish `_exc_value`, and enter `__rt_throw_current`.
/// Never returns.
pub(super) fn emit_reflection_dynamic_type_error_throw(
    ctx: &mut FunctionContext<'_>,
    message: &[u8],
) {
    let (message_label, message_len) = ctx.data.add_string(message);
    emit_reflection_dynamic_throw_from_message_symbol(
        ctx,
        &message_label,
        message_len,
        "_spl_type_error_class_id",
    );
}

/// Throws a catchable Throwable of the runtime class id stored at `class_id_symbol`, with the
/// static message at `message_label`. Never returns.
fn emit_reflection_dynamic_throw_from_message_symbol(
    ctx: &mut FunctionContext<'_>,
    message_label: &str,
    message_len: usize,
    class_id_symbol: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", message_label);
            ctx.emitter.instruction(&format!("mov x2, #{}", message_len));      // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");
            emit_reflection_dynamic_throw_from_persisted_message(ctx, class_id_symbol);
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rdi", message_label);
            ctx.emitter.instruction(&format!("mov rsi, {}", message_len));      // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");
            emit_reflection_dynamic_throw_from_persisted_message(ctx, class_id_symbol);
        }
    }
}

/// Allocates and throws a Throwable whose owned message (ptr, len) is currently in the string
/// persist result registers (`x1`/`x2` on AArch64, `rax`/`rdx` on x86_64). Never returns.
pub(super) fn emit_reflection_dynamic_throw_from_persisted_message(
    ctx: &mut FunctionContext<'_>,
    class_id_symbol: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            ctx.emitter.instruction("mov x0, #32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", class_id_symbol, 0);
            ctx.emitter.instruction("str x9, [x0]");                            // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "x9", "x10");
            ctx.emitter.instruction("str x9, [x0, #8]");                        // store the exception message pointer
            ctx.emitter.instruction("str x10, [x0, #16]");                      // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0);
            abi::emit_jump(ctx.emitter, "__rt_throw_current");
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            ctx.emitter.instruction("mov rax, 32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
            ctx.emitter.instruction("mov r10, 0x4548504c00000006");             // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", class_id_symbol, 0);
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "r10", "r11");
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10");            // store the exception message pointer
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r11");           // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0);
            abi::emit_jump(ctx.emitter, "__rt_throw_current");
        }
    }
}

/// Returns the class name an `Op::ObjectNew` instruction constructs, or `None`.
fn object_new_class_name<'a>(module: &'a Module, inst: &Instruction) -> Option<&'a str> {
    let Some(Immediate::Data(data)) = inst.immediate else {
        return None;
    };
    module
        .data
        .class_names
        .get(data.as_raw() as usize)
        .map(String::as_str)
}

/// Iterates every function-like body lowered into the EIR module.
fn reflection_dispatch_scan_functions(module: &Module) -> impl Iterator<Item = &Function> {
    module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter())
        .chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter())
        .chain(module.runtime_callable_invokers.iter())
}

/// Returns true when the module contains at least one `new ReflectionClass($runtimeName)` site
/// with a non-literal, non-statically-Object reflected-name operand — i.e. whether the shared
/// dynamic dispatcher needs to be emitted at all.
fn module_needs_reflection_class_dynamic_dispatch(module: &Module) -> bool {
    reflection_dispatch_scan_functions(module).any(|function| {
        function.instructions.iter().any(|inst| {
            inst.op == Op::ObjectNew
                && object_new_class_name(module, inst) == Some("ReflectionClass")
                && inst.operands.first().is_some_and(|&value| {
                    !is_const_string_or_class_value(function, value)
                        && !matches!(
                            function
                                .value(value)
                                .map(|v| v.php_type.codegen_repr()),
                            Some(PhpType::Object(_))
                        )
                })
        })
    })
}

/// Returns true when a class participates in the dynamic-name dispatch candidate set.
///
/// Pay-for-use: only classes actually declared in PHP source (a real declaration span —
/// user code, includes, autoloaded libraries, and compiler PHP preludes) get a dispatch arm.
/// Compiler-injected builtin shells (Reflection*, Exception hierarchy, Spl*, DOM, Date*,
/// iterators, ...) are all built with dummy spans and carry enormous synthesized method
/// surfaces — baking full metadata for each measured in the gigabytes of assembly for a
/// hello-world program. A dynamic query naming one of them misses and throws the same loud,
/// catchable `\ReflectionException` an unknown name does (documented divergence from PHP,
/// which can reflect internal classes).
pub(super) fn is_dynamic_reflection_candidate_class(
    name: &str,
    info: &crate::types::ClassInfo,
) -> bool {
    info.declaration_span != crate::span::Span::dummy() && !reflection_class_like_is_internal(name)
}

/// Emits the shared, program-wide dynamic `ReflectionClass(name)` construction dispatcher,
/// once, if and only if the module actually contains a dynamic-name call site. No-op otherwise.
pub(crate) fn emit_reflection_class_dynamic_dispatch_if_needed(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
) -> Result<()> {
    if !module_needs_reflection_class_dynamic_dispatch(module) {
        return Ok(());
    }
    emit_reflection_class_dynamic_dispatch(module, emitter, data, shared)
}

/// Builds and emits the dispatcher body.
///
/// Constructs a throwaway, valid-but-empty synthetic `ir::Function` purely so the existing
/// metadata bakers (`emit_reflection_owner_object` and friends) can be reused unchanged: none
/// of them read per-real-function frame/value-placement state, only
/// `ctx.emitter`/`ctx.data`/`ctx.module`. Every dispatch branch performs exactly what
/// `lower_reflection_owner_new`'s literal-argument `ReflectionClass` path does for the same
/// class.
fn emit_reflection_class_dynamic_dispatch(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
) -> Result<()> {
    let mut class_names: Vec<&String> = module
        .class_infos
        .iter()
        .filter(|(name, info)| is_dynamic_reflection_candidate_class(name, info))
        .map(|(name, _)| name)
        .collect();
    class_names.sort();

    let target = emitter.target;
    let synthetic = Function::new(
        format!("{}_impl", DYNAMIC_CLASS_DISPATCH_LABEL),
        IrType::Void,
        PhpType::Void,
    );
    let layout = frame::layout_for_function(&synthetic, target, false);
    let mut ctx = FunctionContext::new(
        module, &synthetic, emitter, data, shared, layout, false, false, false, None,
    );

    ctx.emitter.blank();
    ctx.emitter
        .comment("--- reflection: dynamic ReflectionClass(name) construction dispatch ---");
    ctx.emitter.label_global(DYNAMIC_CLASS_DISPATCH_LABEL);
    emit_dynamic_dispatch_prologue(&mut ctx);
    emit_dynamic_dispatch_query_normalization(&mut ctx);

    let not_found_label = format!("{}_not_found", DYNAMIC_CLASS_DISPATCH_LABEL);
    let done_label = format!("{}_done", DYNAMIC_CLASS_DISPATCH_LABEL);
    let case_labels: Vec<String> = (0..class_names.len())
        .map(|index| format!("{}_case_{}", DYNAMIC_CLASS_DISPATCH_LABEL, index))
        .collect();

    for (name, label) in class_names.iter().zip(case_labels.iter()) {
        let lowered = php_symbol_key(name.trim_start_matches('\\'));
        super::emit_branch_if_dynamic_name_matches(&mut ctx, &lowered, label);
    }
    abi::emit_jump(ctx.emitter, &not_found_label);

    for (name, label) in class_names.iter().zip(case_labels.iter()) {
        ctx.emitter.label(label);
        // Drop the two parked query pairs (normalized + original, 16 bytes each) — a match
        // no longer needs them, and construction below assumes the same clean stack the
        // literal path starts from right after the prologue.
        abi::emit_release_temporary_stack(ctx.emitter, 32);
        let metadata = reflection_class_metadata_for_name(&ctx, name)?;
        emit_reflection_owner_object(&mut ctx, "ReflectionClass", &metadata)?;
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&not_found_label);
    emit_reflection_class_not_found_throw(&mut ctx);

    ctx.emitter.label(&done_label);
    emit_dynamic_dispatch_epilogue(&mut ctx);
    Ok(())
}

/// Emits the leaf-function prologue for the shared dynamic dispatcher: a plain frame-pointer
/// save/establish, matching the hand-written runtime helpers this dispatcher is modeled after.
pub(super) fn emit_dynamic_dispatch_prologue(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("stp x29, x30, [sp, #-16]!");               // save frame pointer and return address
            ctx.emitter.instruction("mov x29, sp");                             // establish the new frame pointer
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("push rbp");                                // preserve the caller frame pointer
            ctx.emitter.instruction("mov rbp, rsp");                            // establish an aligned helper frame
        }
    }
}

/// Emits the matching epilogue. The constructed object pointer is already parked in the ABI
/// integer result register by the matched dispatch branch.
pub(super) fn emit_dynamic_dispatch_epilogue(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldp x29, x30, [sp], #16");                 // restore frame pointer and return address
            ctx.emitter.instruction("ret");                                     // return the constructed object pointer in x0
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("pop rbp");                                 // restore the caller frame pointer
            ctx.emitter.instruction("ret");                                     // return the constructed object pointer in rax
        }
    }
}

/// Parks the caller's original `(name_ptr, name_len)` pair on the temporary stack (offsets
/// 16/24 once the normalized copy is parked on top of it) — kept byte-for-byte for the
/// not-found exception message, which echoes the query exactly as PHP does — then computes a
/// leading-backslash-stripped, PHP-case-folded working copy for the compare chain, parked at
/// offsets 0/8 (`emit_branch_if_dynamic_name_matches` reads its query from those offsets).
pub(super) fn emit_dynamic_dispatch_query_normalization(ctx: &mut FunctionContext<'_>) {
    let skip_label = ctx.next_label("reflect_dyn_skip_bs");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x0", "x1");
            ctx.emitter.instruction(&format!("cbz x1, {}", skip_label));        // an empty query cannot start with a leading backslash
            ctx.emitter.instruction("ldrb w9, [x0]");                           // peek at the query's first byte
            ctx.emitter.instruction("cmp w9, #0x5c");                           // is it a leading namespace-root backslash?
            ctx.emitter.instruction(&format!("b.ne {}", skip_label));           // no backslash to strip
            ctx.emitter.instruction("add x0, x0, #1");                          // strip the leading backslash from the working pointer
            ctx.emitter.instruction("sub x1, x1, #1");                          // and from the working length
            ctx.emitter.label(&skip_label);
            ctx.emitter.instruction("mov x2, x1");                              // __rt_strtolower expects the length in x2
            ctx.emitter.instruction("mov x1, x0");                              // __rt_strtolower expects the pointer in x1
            abi::emit_call_label(ctx.emitter, "__rt_strtolower");
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rdi", "rsi");
            ctx.emitter.instruction("test rsi, rsi");                           // an empty query cannot start with a leading backslash
            ctx.emitter.instruction(&format!("jz {}", skip_label));             // skip stripping for an empty query
            ctx.emitter.instruction("movzx r9d, BYTE PTR [rdi]");               // peek at the query's first byte
            ctx.emitter.instruction("cmp r9b, 0x5c");                           // is it a leading namespace-root backslash?
            ctx.emitter.instruction(&format!("jne {}", skip_label));            // no backslash to strip
            ctx.emitter.instruction("add rdi, 1");                              // strip the leading backslash from the working pointer
            ctx.emitter.instruction("sub rsi, 1");                              // and from the working length
            ctx.emitter.label(&skip_label);
            ctx.emitter.instruction("mov rax, rdi");                            // __rt_strtolower expects the pointer in rax
            ctx.emitter.instruction("mov rdx, rsi");                            // and the length in rdx
            abi::emit_call_label(ctx.emitter, "__rt_strtolower");
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
        }
    }
}

/// Throws a catchable `\ReflectionException` for a dynamic `ReflectionClass(name)` construction
/// whose case-folded query matched no closed-world class. Builds PHP's exact message
/// (`Class "NAME" does not exist`, NAME = the original, unmodified query reloaded from the
/// temporary stack slot the normalization parked it at). Never returns.
fn emit_reflection_class_not_found_throw(ctx: &mut FunctionContext<'_>) {
    let (prefix_label, prefix_len) = ctx.data.add_string(b"Class \"");
    let (suffix_label, suffix_len) = ctx.data.add_string(b"\" does not exist");
    emit_reflection_dynamic_not_found_throw(
        ctx,
        &prefix_label,
        prefix_len,
        &suffix_label,
        suffix_len,
        16,
    );
}

/// Concatenates `prefix + original-query + suffix` (the query reloaded from temporary stack
/// offsets `query_slot_offset`/`+8`), persists it, and throws a catchable
/// `\ReflectionException` with that message. Never returns.
pub(super) fn emit_reflection_dynamic_not_found_throw(
    ctx: &mut FunctionContext<'_>,
    prefix_label: &str,
    prefix_len: usize,
    suffix_label: &str,
    suffix_len: usize,
    query_slot_offset: usize,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", prefix_label);
            ctx.emitter.instruction(&format!("mov x2, #{}", prefix_len));       // message prefix byte length
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x3", query_slot_offset);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x4", query_slot_offset + 8);
            abi::emit_call_label(ctx.emitter, "__rt_concat");
            abi::emit_symbol_address(ctx.emitter, "x3", suffix_label);
            ctx.emitter.instruction(&format!("mov x4, #{}", suffix_len));       // message suffix byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");
            emit_reflection_dynamic_throw_from_persisted_message(
                ctx,
                "_reflection_exception_class_id",
            );
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rax", prefix_label);
            ctx.emitter.instruction(&format!("mov rdx, {}", prefix_len));       // message prefix byte length
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", query_slot_offset);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", query_slot_offset + 8);
            abi::emit_call_label(ctx.emitter, "__rt_concat");
            abi::emit_symbol_address(ctx.emitter, "rdi", suffix_label);
            ctx.emitter.instruction(&format!("mov rsi, {}", suffix_len));       // message suffix byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");
            ctx.emitter.instruction("mov rdi, rax");                            // move the message pointer into the persist argument
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");
            emit_reflection_dynamic_throw_from_persisted_message(
                ctx,
                "_reflection_exception_class_id",
            );
        }
    }
}
