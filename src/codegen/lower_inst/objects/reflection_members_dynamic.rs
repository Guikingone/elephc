//! Purpose:
//! Lowers dynamic `new ReflectionMethod($class, $method)` / `new ReflectionProperty($class,
//! $property)` / `new ReflectionClassConstant($class, $constant)` construction — either argument
//! a non-literal runtime value — through three shared, program-wide dispatcher labels
//! (`_elephc_reflect_method_new_dynamic` / `_elephc_reflect_property_new_dynamic` /
//! `_elephc_reflect_class_constant_new_dynamic`). The class-name argument accepts PHP's real
//! `object|string` surface (objects resolve `get_class`, scalars weak-coerce to a class-name
//! string, arrays/resources throw a catchable `\TypeError`); a class miss throws PHP's
//! `Class "NAME" does not exist` `\ReflectionException`; a member miss throws
//! `Method CLASS::NAME() does not exist` / `Property CLASS::$NAME does not exist` /
//! `Constant CLASS::NAME does not exist`.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::reflection::lower_reflection_owner_new()` routes a
//!   non-literal `ReflectionMethod`/`ReflectionProperty`/`ReflectionClassConstant` constructor
//!   operand pair here.
//! - `crate::codegen::block_emit::emit_module()` calls
//!   `emit_reflection_member_dynamic_dispatch_if_needed()` once, after per-function lowering.
//!
//! Key details:
//! - Class names and method names are PHP case-insensitive (both case-folded before the compare
//!   chains); property and class-constant names are case-sensitive and compared byte-for-byte.
//! - Gradual member-name operands are unboxed and runtime-checked as strings before dispatch;
//!   non-string values throw a catchable `TypeError`.
//! - Matched (class, member) arms reuse the same `emit_reflection_owner_object` metadata bake
//!   the literal construction path uses, so both routes produce identical objects.
//! - The candidate class set is the same pay-for-use, real-declaration-span set the dynamic
//!   `ReflectionClass(name)` dispatcher uses (`is_dynamic_reflection_candidate_class`).

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen::{CodegenIrError, Result};
use crate::codegen_support::data_section::DataSection;
use crate::codegen_support::emit::Emitter;
use crate::ir::{Function, Immediate, Instruction, IrType, Module, Op, ValueId};
use crate::names::php_symbol_key;
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::super::super::frame;
use super::super::super::shared_state::SharedCodegenState;
use super::reflection::{
    emit_reflection_owner_object, reflection_class_constant_metadata_for_names,
    reflection_class_constant_names, reflection_class_method_names,
    reflection_class_property_names, reflection_method_metadata_for_names,
    reflection_property_metadata_for_names, ReflectionOwnerMetadata,
};
use super::reflection_dynamic::{
    emit_dynamic_dispatch_epilogue, emit_dynamic_dispatch_prologue,
    emit_reflection_dynamic_not_found_throw, emit_reflection_dynamic_type_error_throw,
    is_const_string_or_class_value, is_dynamic_reflection_candidate_class,
};

/// Assembly label of the shared dynamic `ReflectionMethod(class, method)` dispatcher.
const METHOD_DISPATCH_LABEL: &str = "_elephc_reflect_method_new_dynamic";
/// Assembly label of the shared dynamic `ReflectionProperty(class, property)` dispatcher.
const PROPERTY_DISPATCH_LABEL: &str = "_elephc_reflect_property_new_dynamic";
/// Assembly label of the shared dynamic `ReflectionClassConstant(class, constant)` dispatcher.
const CLASS_CONSTANT_DISPATCH_LABEL: &str = "_elephc_reflect_class_constant_new_dynamic";

/// Which member reflector a dispatcher resolves; drives case-folding, metadata lookup, and
/// miss-message wording.
#[derive(Clone, Copy, PartialEq)]
enum MemberKind {
    Method,
    Property,
    ClassConstant,
}

impl MemberKind {
    /// Returns the shared dispatcher label for this member kind.
    fn dispatch_label(self) -> &'static str {
        match self {
            MemberKind::Method => METHOD_DISPATCH_LABEL,
            MemberKind::Property => PROPERTY_DISPATCH_LABEL,
            MemberKind::ClassConstant => CLASS_CONSTANT_DISPATCH_LABEL,
        }
    }

    /// Returns the constructed reflection owner class name for this member kind.
    fn owner_class(self) -> &'static str {
        match self {
            MemberKind::Method => "ReflectionMethod",
            MemberKind::Property => "ReflectionProperty",
            MemberKind::ClassConstant => "ReflectionClassConstant",
        }
    }
}

/// Lowers a dynamic-argument `new ReflectionMethod($class, $method)` call site.
pub(super) fn lower_reflection_method_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_operand: ValueId,
    member_operand: ValueId,
) -> Result<()> {
    lower_reflection_member_new_dynamic(ctx, inst, class_operand, member_operand, MemberKind::Method)
}

/// Lowers a dynamic-argument `new ReflectionProperty($class, $property)` call site.
pub(super) fn lower_reflection_property_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_operand: ValueId,
    member_operand: ValueId,
) -> Result<()> {
    lower_reflection_member_new_dynamic(
        ctx,
        inst,
        class_operand,
        member_operand,
        MemberKind::Property,
    )
}

/// Lowers a dynamic-argument `new ReflectionClassConstant($class, $constant)` call site.
pub(super) fn lower_reflection_class_constant_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_operand: ValueId,
    member_operand: ValueId,
) -> Result<()> {
    lower_reflection_member_new_dynamic(
        ctx,
        inst,
        class_operand,
        member_operand,
        MemberKind::ClassConstant,
    )
}

/// Materializes both dispatcher arguments and calls the shared member dispatcher: the resolved
/// class-name query in arg0/arg1 and the member-name query in arg2/arg3.
fn lower_reflection_member_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_operand: ValueId,
    member_operand: ValueId,
    kind: MemberKind,
) -> Result<()> {
    // Resolve the class query first (it may need runtime calls that clobber the argument
    // registers), park it, then load the member query, then restore the class pair.
    emit_member_class_name_query(ctx, class_operand, kind)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => abi::emit_push_reg_pair(ctx.emitter, "x1", "x2"),
        Arch::X86_64 => abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx"),
    }
    emit_member_name_query_to_arg_regs(ctx, member_operand, kind)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => abi::emit_pop_reg_pair(ctx.emitter, "x0", "x1"),
        Arch::X86_64 => abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi"),
    }
    abi::emit_call_label(ctx.emitter, kind.dispatch_label());
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Resolves the `object|string` class argument into a class-name string in the standard string
/// RESULT registers (`x1`/`x2` on AArch64, `rax`/`rdx` on x86_64), weak-coercing scalars and
/// throwing a catchable `\TypeError` for non-coercible runtime shapes, mirroring the dynamic
/// `ReflectionClass(name)` front end.
fn emit_member_class_name_query(
    ctx: &mut FunctionContext<'_>,
    class_operand: ValueId,
    kind: MemberKind,
) -> Result<()> {
    match ctx.value_php_type(class_operand)?.codegen_repr() {
        PhpType::Str => match ctx.emitter.target.arch {
            Arch::AArch64 => ctx.load_string_value_to_regs(class_operand, "x1", "x2")?,
            Arch::X86_64 => ctx.load_string_value_to_regs(class_operand, "rax", "rdx")?,
        },
        PhpType::Object(_) => {
            ctx.load_value_to_result(class_operand)?;
            super::super::builtins::types::emit_dynamic_object_class_name(ctx, "get_class");
        }
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_result(class_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            emit_member_class_query_from_mixed_tag(ctx, kind)?;
        }
        PhpType::Int => {
            ctx.load_value_to_result(class_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_itoa");
        }
        PhpType::Float => {
            ctx.load_value_to_result(class_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_ftoa");
        }
        PhpType::Bool => {
            ctx.load_value_to_result(class_operand)?;
            emit_member_loaded_bool_to_string(ctx);
        }
        PhpType::Void | PhpType::Never => {
            let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
            abi::emit_load_int_immediate(ctx.emitter, ptr_reg, 0);
            abi::emit_load_int_immediate(ctx.emitter, len_reg, 0);
        }
        _ => {
            ctx.load_value_to_result(class_operand)?;
            emit_member_class_argument_type_error_throw(ctx, kind);
        }
    }
    Ok(())
}

/// After `__rt_mixed_unbox` on the class argument, converts the runtime payload to a class-name
/// string in the string RESULT registers, weak-coercing scalar tags and throwing a catchable
/// `\TypeError` for array/resource/other tags.
fn emit_member_class_query_from_mixed_tag(
    ctx: &mut FunctionContext<'_>,
    kind: MemberKind,
) -> Result<()> {
    let str_label = ctx.next_label("reflect_member_mixed_str");
    let object_label = ctx.next_label("reflect_member_mixed_obj");
    let int_label = ctx.next_label("reflect_member_mixed_int");
    let float_label = ctx.next_label("reflect_member_mixed_float");
    let bool_label = ctx.next_label("reflect_member_mixed_bool");
    let bool_false_label = ctx.next_label("reflect_member_mixed_bool_false");
    let null_label = ctx.next_label("reflect_member_mixed_null");
    let type_error_label = ctx.next_label("reflect_member_mixed_type_error");
    let done_label = ctx.next_label("reflect_member_mixed_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #1");                              // runtime tag 1 means the boxed union holds a string payload
            ctx.emitter.instruction(&format!("b.eq {}", str_label));            // a string payload is already (ptr, len)
            ctx.emitter.instruction("cmp x0, #6");                              // runtime tag 6 means the boxed union holds an object payload
            ctx.emitter.instruction(&format!("b.eq {}", object_label));         // resolve the object's own runtime class name
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
            ctx.emitter.instruction(&format!("je {}", str_label));              // a string payload is already (ptr, len)
            ctx.emitter.instruction("cmp rax, 6");                              // runtime tag 6 means the boxed union holds an object payload
            ctx.emitter.instruction(&format!("je {}", object_label));           // resolve the object's own runtime class name
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

    // -- tag 1 (string): move the unboxed payload into the string result convention. --
    ctx.emitter.label(&str_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {}
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, rdi");                            // move the unboxed string pointer into the string result register
        }
    }
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 6 (object): resolve the object's concrete runtime class name. --
    ctx.emitter.label(&object_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction("mov x0, x1"),                 // move the unboxed object pointer into the class-name lookup register
        Arch::X86_64 => ctx.emitter.instruction("mov rax, rdi"),                // move the unboxed object pointer into the class-name lookup register
    }
    super::super::builtins::types::emit_dynamic_object_class_name(ctx, "get_class");
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 0 (int): weak-cast the unboxed int payload to decimal text. --
    ctx.emitter.label(&int_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction("mov x0, x1"),                 // move the unboxed int payload into __rt_itoa's argument register
        Arch::X86_64 => ctx.emitter.instruction("mov rax, rdi"),                // move the unboxed int payload into __rt_itoa's argument register
    }
    abi::emit_call_label(ctx.emitter, "__rt_itoa");
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
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 3 (bool): weak-cast to "1" (true) or "" (false). --
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
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 8 (null): weak-casts to the empty string. --
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
    abi::emit_jump(ctx.emitter, &done_label);

    // -- any other tag (array, resource, …): throw a catchable TypeError, never returns. --
    ctx.emitter.label(&type_error_label);
    emit_member_class_argument_type_error_throw(ctx, kind);

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Weak-casts an already-loaded raw bool value to `"1"`/`""` in the string result registers.
fn emit_member_loaded_bool_to_string(ctx: &mut FunctionContext<'_>) {
    let false_label = ctx.next_label("reflect_member_bool_str_false");
    let done_label = ctx.next_label("reflect_member_bool_str_done");
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

/// Throws a catchable `\TypeError` for a non-`object|string`-coercible class argument, with the
/// owner-specific PHP wording. Never returns.
fn emit_member_class_argument_type_error_throw(ctx: &mut FunctionContext<'_>, kind: MemberKind) {
    let message: &[u8] = match kind {
        MemberKind::Method => {
            b"ReflectionMethod::__construct(): Argument #1 ($objectOrMethod) must be of type object|string"
        }
        MemberKind::Property => {
            b"ReflectionProperty::__construct(): Argument #1 ($class) must be of type object|string"
        }
        MemberKind::ClassConstant => {
            b"ReflectionClassConstant::__construct(): Argument #1 ($class) must be of type object|string"
        }
    };
    emit_reflection_dynamic_type_error_throw(ctx, message);
}

/// Loads the member-name query into the dispatcher's arg2/arg3 registers (`x2`/`x3` on AArch64,
/// `rdx`/`rcx` on x86_64). Concrete strings load directly. Gradual `Mixed`/union values are
/// unboxed and must carry the runtime string tag; other tags throw a catchable `TypeError`.
fn emit_member_name_query_to_arg_regs(
    ctx: &mut FunctionContext<'_>,
    member_operand: ValueId,
    kind: MemberKind,
) -> Result<()> {
    match ctx.value_php_type(member_operand)?.codegen_repr() {
        PhpType::Str => match ctx.emitter.target.arch {
            Arch::AArch64 => ctx.load_string_value_to_regs(member_operand, "x2", "x3")?,
            Arch::X86_64 => ctx.load_string_value_to_regs(member_operand, "rdx", "rcx")?,
        },
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_result(member_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            let string_label = ctx.next_label("reflect_member_name_string");
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction("cmp x0, #1");                      // does the gradual member name carry the runtime string tag?
                    ctx.emitter.instruction(&format!("b.eq {}", string_label)); // continue only for a runtime string member name
                    emit_member_name_argument_type_error_throw(ctx, kind);
                    ctx.emitter.label(&string_label);
                    ctx.emitter.instruction("mov x3, x2");                      // move the unboxed member-name length into the dispatcher's arg3
                    ctx.emitter.instruction("mov x2, x1");                      // move the unboxed member-name pointer into the dispatcher's arg2
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction("cmp rax, 1");                      // does the gradual member name carry the runtime string tag?
                    ctx.emitter.instruction(&format!("je {}", string_label));   // continue only for a runtime string member name
                    emit_member_name_argument_type_error_throw(ctx, kind);
                    ctx.emitter.label(&string_label);
                    ctx.emitter.instruction("mov rcx, rdx");                    // move the unboxed member-name length into the dispatcher's arg3
                    ctx.emitter.instruction("mov rdx, rdi");                    // move the unboxed member-name pointer into the dispatcher's arg2
                }
            }
        }
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "{} constructor member-name operand of PHP type {:?}",
                kind.owner_class(),
                other
            )));
        }
    }
    Ok(())
}

/// Throws a catchable `\TypeError` when a gradual member-name operand is not a runtime string.
fn emit_member_name_argument_type_error_throw(
    ctx: &mut FunctionContext<'_>,
    kind: MemberKind,
) {
    let message: &[u8] = match kind {
        MemberKind::Method => {
            b"ReflectionMethod::__construct(): Argument #2 ($method) must be of type ?string"
        }
        MemberKind::Property => {
            b"ReflectionProperty::__construct(): Argument #2 ($property) must be of type string"
        }
        MemberKind::ClassConstant => {
            b"ReflectionClassConstant::__construct(): Argument #2 ($constant) must be of type string"
        }
    };
    emit_reflection_dynamic_type_error_throw(ctx, message);
}

/// Returns true when the module contains at least one dynamic-argument
/// `new ReflectionMethod(...)`/`new ReflectionProperty(...)`/`new ReflectionClassConstant(...)`
/// construction site.
fn module_needs_member_dynamic_dispatch(module: &Module, owner: &str) -> bool {
    scan_functions(module).any(|function| {
        function.instructions.iter().any(|inst| {
            inst.op == Op::ObjectNew
                && object_new_class_name(module, inst) == Some(owner)
                && inst.operands.len() >= 2
                && inst
                    .operands
                    .iter()
                    .take(2)
                    .any(|&value| !is_const_string_or_class_value(function, value))
        })
    })
}

/// Iterates every function-like body lowered into the EIR module.
fn scan_functions(module: &Module) -> impl Iterator<Item = &Function> {
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

/// Emits the shared dynamic member dispatchers, each once, if and only if the module actually
/// contains a dynamic-argument call site for that owner. No-op otherwise.
pub(crate) fn emit_reflection_member_dynamic_dispatch_if_needed(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
) -> Result<()> {
    for kind in [
        MemberKind::Method,
        MemberKind::Property,
        MemberKind::ClassConstant,
    ] {
        if module_needs_member_dynamic_dispatch(module, kind.owner_class()) {
            emit_member_dynamic_dispatch(module, emitter, data, shared, kind)?;
        }
    }
    Ok(())
}

/// Builds and emits one member dispatcher body over a throwaway synthetic function context
/// (mirroring the dynamic `ReflectionClass(name)` dispatcher's construction).
///
/// Temporary-stack protocol: the site passes the class query in arg0/arg1 and the member query
/// in arg2/arg3. The prologue parks `(class_orig, member_orig)` then the case-folded working
/// class copy, so the outer class chain reads offsets 0/8, the class-miss throw reads the
/// original class at 32/40, and each class arm drops the working copy before member matching.
fn emit_member_dynamic_dispatch(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
    kind: MemberKind,
) -> Result<()> {
    let mut class_names: Vec<&String> = module
        .class_infos
        .iter()
        .filter(|(name, info)| is_dynamic_reflection_candidate_class(name, info))
        .map(|(name, _)| name)
        .collect();
    class_names.sort();

    let label = kind.dispatch_label();
    let target = emitter.target;
    let synthetic = Function::new(format!("{}_impl", label), IrType::Void, PhpType::Void);
    let layout = frame::layout_for_function(&synthetic, target, false);
    let mut ctx = FunctionContext::new(
        module, &synthetic, emitter, data, shared, layout, false, false, false, None,
    );

    ctx.emitter.blank();
    ctx.emitter.comment(&format!(
        "--- reflection: dynamic {}(class, member) construction dispatch ---",
        kind.owner_class()
    ));
    ctx.emitter.label_global(label);
    emit_dynamic_dispatch_prologue(&mut ctx);

    // Park (class_orig at 16/24 after both pushes, member_orig at 0/8), then normalize the
    // class query on top: work copy at 0/8, member_orig at 16/24, class_orig at 32/40.
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x0", "x1");
            abi::emit_push_reg_pair(ctx.emitter, "x2", "x3");
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rdi", "rsi");
            abi::emit_push_reg_pair(ctx.emitter, "rdx", "rcx");
        }
    }
    emit_member_class_normalization(&mut ctx);

    let not_found_label = format!("{}_class_not_found", label);
    let done_label = format!("{}_done", label);
    let case_labels: Vec<String> = (0..class_names.len())
        .map(|index| format!("{}_class_case_{}", label, index))
        .collect();

    for (name, case_label) in class_names.iter().zip(case_labels.iter()) {
        let lowered = php_symbol_key(name.trim_start_matches('\\'));
        super::emit_branch_if_dynamic_name_matches(&mut ctx, &lowered, case_label);
    }
    abi::emit_jump(ctx.emitter, &not_found_label);

    for (index, (name, case_label)) in class_names.iter().zip(case_labels.iter()).enumerate() {
        ctx.emitter.label(case_label);
        // Drop the case-folded working class copy — member matching below re-parks what it
        // needs. Stack after this: member_orig at 0/8, class_orig at 16/24.
        abi::emit_release_temporary_stack(ctx.emitter, 16);
        emit_member_match_for_class(&mut ctx, kind, name, index, &done_label)?;
    }

    ctx.emitter.label(&not_found_label);
    let (prefix_label, prefix_len) = ctx.data.add_string(b"Class \"");
    let (suffix_label, suffix_len) = ctx.data.add_string(b"\" does not exist");
    emit_reflection_dynamic_not_found_throw(
        &mut ctx,
        &prefix_label,
        prefix_len,
        &suffix_label,
        suffix_len,
        32,
    );

    ctx.emitter.label(&done_label);
    emit_dynamic_dispatch_epilogue(&mut ctx);
    Ok(())
}

/// Case-folds the parked class query (strips one leading backslash first) and parks the working
/// copy on top of the temporary stack for the outer compare chain.
fn emit_member_class_normalization(ctx: &mut FunctionContext<'_>) {
    let skip_label = ctx.next_label("reflect_member_skip_bs");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", 16);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 24);
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
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", 16);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", 24);
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

/// Emits the per-class member compare chain, matched-member metadata bakes, and the member-miss
/// throw for one matched class arm. On entry, member_orig is parked at 0/8 and class_orig at
/// 16/24; every matched bake drops all parked pairs before constructing.
fn emit_member_match_for_class(
    ctx: &mut FunctionContext<'_>,
    kind: MemberKind,
    class_name: &str,
    class_index: usize,
    done_label: &str,
) -> Result<()> {
    let dispatch_label = kind.dispatch_label();
    let miss_label = format!("{}_miss_{}", dispatch_label, class_index);
    match kind {
        MemberKind::Method => {
            // Method names are case-insensitive: fold the parked member query and park the
            // working copy (work at 0/8, member_orig at 16/24, class_orig at 32/40).
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 0);
                    abi::emit_load_temporary_stack_slot(ctx.emitter, "x2", 8);
                    abi::emit_call_label(ctx.emitter, "__rt_strtolower");
                    abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
                }
                Arch::X86_64 => {
                    abi::emit_load_temporary_stack_slot(ctx.emitter, "rax", 0);
                    abi::emit_load_temporary_stack_slot(ctx.emitter, "rdx", 8);
                    abi::emit_call_label(ctx.emitter, "__rt_strtolower");
                    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
                }
            }
            let mut arms = Vec::new();
            for method_name in reflection_class_method_names(ctx, class_name) {
                let Some(metadata) =
                    reflection_method_metadata_for_names(ctx, class_name, &method_name)?
                else {
                    continue;
                };
                let arm_label = ctx.next_label("reflect_member_method_arm");
                super::emit_branch_if_dynamic_name_matches(
                    ctx,
                    &php_symbol_key(&method_name),
                    &arm_label,
                );
                arms.push((arm_label, metadata));
            }
            abi::emit_jump(ctx.emitter, &miss_label);
            for (arm_label, metadata) in arms {
                ctx.emitter.label(&arm_label);
                abi::emit_release_temporary_stack(ctx.emitter, 48);
                emit_reflection_owner_object(ctx, kind.owner_class(), &metadata)?;
                abi::emit_jump(ctx.emitter, done_label);
            }
            ctx.emitter.label(&miss_label);
            // php -n verified: `Method CLASS::NAME() does not exist`, NAME echoed as queried.
            let (prefix_label, prefix_len) = ctx
                .data
                .add_string(format!("Method {}::", class_name).as_bytes());
            let (suffix_label, suffix_len) = ctx.data.add_string(b"() does not exist");
            emit_reflection_dynamic_not_found_throw(
                ctx,
                &prefix_label,
                prefix_len,
                &suffix_label,
                suffix_len,
                16,
            );
        }
        MemberKind::Property => {
            // Property names are case-sensitive: compare the parked original member query
            // (already at 0/8) byte-for-byte against each declared property name.
            let class_info = ctx
                .module
                .class_infos
                .get(class_name)
                .ok_or_else(|| CodegenIrError::missing_entry("class", 0))?;
            let mut members = Vec::new();
            for property_name in reflection_class_property_names(ctx, class_name, class_info) {
                let Some(metadata) =
                    reflection_property_metadata_for_names(ctx, class_name, &property_name)
                else {
                    continue;
                };
                members.push((property_name, metadata));
            }
            // php -n verified: `Property CLASS::$NAME does not exist`, NAME echoed as queried.
            emit_case_sensitive_member_arms(
                ctx,
                kind,
                members,
                "reflect_member_property_arm",
                &miss_label,
                done_label,
                &format!("Property {}::$", class_name),
                b" does not exist",
            )?;
        }
        MemberKind::ClassConstant => {
            // Class constant names are case-sensitive exactly like properties (php -n verified:
            // `new ReflectionClassConstant('Holder', 'foo')` throws for a `FOO` constant), so the
            // parked original member query at 0/8 is compared byte-for-byte and no fold is pushed.
            let class_info = ctx
                .module
                .class_infos
                .get(class_name)
                .ok_or_else(|| CodegenIrError::missing_entry("class", 0))?;
            let mut members = Vec::new();
            for constant_name in reflection_class_constant_names(ctx, class_name, class_info) {
                let Some(metadata) =
                    reflection_class_constant_metadata_for_names(ctx, class_name, &constant_name)?
                else {
                    continue;
                };
                members.push((constant_name, metadata));
            }
            // php -n verified: `Constant CLASS::NAME does not exist`, NAME echoed as queried.
            emit_case_sensitive_member_arms(
                ctx,
                kind,
                members,
                "reflect_member_class_constant_arm",
                &miss_label,
                done_label,
                &format!("Constant {}::", class_name),
                b" does not exist",
            )?;
        }
    }
    Ok(())
}

/// Emits a case-sensitive member compare chain, one metadata bake per matched member, and the
/// member-miss throw.
///
/// Shared by the `ReflectionProperty` and `ReflectionClassConstant` arms, which are identical in
/// shape: neither case-folds the member query, so both compare the original parked query at 0/8
/// and both unwind the same two parked pairs (32 bytes) before constructing. Only the declared
/// member set and the miss-message wording differ.
fn emit_case_sensitive_member_arms(
    ctx: &mut FunctionContext<'_>,
    kind: MemberKind,
    members: Vec<(String, ReflectionOwnerMetadata)>,
    arm_label_stem: &str,
    miss_label: &str,
    done_label: &str,
    miss_prefix: &str,
    miss_suffix: &[u8],
) -> Result<()> {
    let mut arms = Vec::new();
    for (member_name, metadata) in members {
        let arm_label = ctx.next_label(arm_label_stem);
        super::emit_branch_if_dynamic_name_matches(ctx, &member_name, &arm_label);
        arms.push((arm_label, metadata));
    }
    abi::emit_jump(ctx.emitter, miss_label);
    for (arm_label, metadata) in arms {
        ctx.emitter.label(&arm_label);
        abi::emit_release_temporary_stack(ctx.emitter, 32);
        emit_reflection_owner_object(ctx, kind.owner_class(), &metadata)?;
        abi::emit_jump(ctx.emitter, done_label);
    }
    ctx.emitter.label(miss_label);
    let (prefix_label, prefix_len) = ctx.data.add_string(miss_prefix.as_bytes());
    let (suffix_label, suffix_len) = ctx.data.add_string(miss_suffix);
    emit_reflection_dynamic_not_found_throw(
        ctx,
        &prefix_label,
        prefix_len,
        &suffix_label,
        suffix_len,
        0,
    );
    Ok(())
}
