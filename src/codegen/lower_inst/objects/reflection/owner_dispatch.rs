//! Purpose:
//! Reflection owner detection and runtime-class dispatch.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::reflection`.
//!
//! Key details:
//! - Preserves compile-time metadata, target-aware object layout, and ownership.

use super::*;
use std::collections::HashSet;

use crate::codegen::lower_inst::current_method_class;
use crate::ir::Terminator;
use crate::names::label_fragment;

/// Returns true for reflection owner classes that need metadata-aware construction.
pub(in crate::codegen::lower_inst::objects) fn is_reflection_owner_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "ReflectionClass"
            | "ReflectionObject"
            | "ReflectionFunction"
            | "ReflectionMethod"
            | "ReflectionProperty"
            | "ReflectionParameter"
            | "ReflectionClassConstant"
            | "ReflectionEnum"
            | "ReflectionEnumUnitCase"
            | "ReflectionEnumBackedCase"
    )
}

/// Lowers builtin Reflection owner allocation by populating compile-time metadata slots.
pub(in crate::codegen::lower_inst::objects) fn lower_reflection_owner_new(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_name: &str,
) -> Result<()> {
    if class_name == "ReflectionObject"
        && lower_reflection_object_by_runtime_class(ctx, inst)?
    {
        // Runtime-class dispatch leaves the fresh ReflectionObject in the result register.
    } else if crate::codegen::lower_inst::builtins::has_eval_context(ctx)
        || reflection_class_literal_requires_runtime_metadata(ctx, class_name, inst)?
        || reflection_owner_requires_runtime_metadata(ctx, inst)?
    {
        return crate::codegen::lower_inst::builtins::lower_eval_native_object_new(ctx, inst);
    } else {
        let metadata = reflection_owner_metadata(ctx, class_name, inst)?;
        emit_direct_reflection_owner_object(ctx, class_name, inst, &metadata)?;
    }
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Returns whether a literal `ReflectionClass` target needs runtime declaration metadata.
///
/// A class string can be lexically constant while its declaration is intentionally supplied by a
/// dynamic include. A missing AOT doc comment is ambiguous: it can mean either an undocumented
/// static class or a class loaded at runtime. The bridge resolves that ambiguity after loading,
/// returning the actual comment or PHP's `false` result.
fn reflection_class_literal_requires_runtime_metadata(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    inst: &Instruction,
) -> Result<bool> {
    if class_name != "ReflectionClass" {
        return Ok(false);
    }
    let Some(operand) = inst.operands.first().copied() else {
        return Ok(false);
    };
    let literal_source = reflection_literal_source(ctx, operand)?;
    let Some(value) = ctx.function.value(literal_source) else {
        return Err(CodegenIrError::missing_entry("value", literal_source.as_raw()));
    };
    let ValueDef::Instruction {
        inst: source_inst, ..
    } = value.def
    else {
        return Ok(false);
    };
    let Some(source_inst) = ctx.function.instruction(source_inst) else {
        return Err(CodegenIrError::missing_entry("instruction", source_inst.as_raw()));
    };
    if !matches!(source_inst.op, Op::ConstStr | Op::ConstClassName) {
        return Ok(false);
    }
    let reflected_class = const_string_or_class_operand(ctx, operand, "ReflectionClass")?;
    if let Some((_, class_info)) = resolve_reflection_class(ctx, &reflected_class) {
        return Ok(class_info.doc_comment.is_none());
    }
    Ok(resolve_reflection_interface(ctx, &reflected_class).is_none()
        && resolve_reflection_trait(ctx, &reflected_class).is_none()
        && !is_reflection_enum(ctx, &reflected_class))
}

/// Materializes `ReflectionObject` metadata from the concrete runtime class of an object operand.
///
/// The eval bridge exposes one script-level source file and therefore cannot represent per-class
/// files from includes or autoloading. Closed object types dispatch directly to cached AOT
/// materializers, preserving exact file, line, parent, interface, and member metadata. Very broad
/// object universes stay on the existing dynamic bridge to avoid emitting thousands of graphs.
fn lower_reflection_object_by_runtime_class(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<bool> {
    let Some(source) = inst.operands.first().copied() else {
        return Ok(false);
    };
    let (mut static_type, boxed_source) = match ctx.value_php_type(source)?.codegen_repr() {
        PhpType::Object(static_type) => (static_type, false),
        PhpType::Mixed | PhpType::Union(_) => {
            let Some(lexical_class) = current_method_class(ctx).ok() else {
                return Ok(false);
            };
            (lexical_class.to_string(), true)
        }
        _ => return Ok(false),
    };
    if static_type.is_empty() || static_type.eq_ignore_ascii_case("object") {
        let Some(lexical_class) = current_method_class(ctx).ok() else {
            return Ok(false);
        };
        if !ctx.module.class_infos.contains_key(lexical_class) {
            return Ok(false);
        }
        static_type = lexical_class.to_string();
    }
    let mut candidates = ctx
        .module
        .class_infos
        .iter()
        .filter(|(class_name, _)| {
            reflection_class_matches_object_type(ctx, class_name, &static_type)
        })
        .map(|(class_name, class_info)| (class_name.clone(), class_info.class_id))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(_, class_id)| *class_id);
    if candidates.is_empty() || candidates.len() > 64 {
        return Ok(false);
    }

    let result_reg = abi::int_result_reg(ctx.emitter).to_string();
    let done_label = ctx.next_label("reflection_object_runtime_done");
    let miss_label = ctx.next_label("reflection_object_runtime_miss");
    let match_labels = candidates
        .iter()
        .map(|(class_name, _)| {
            ctx.next_label(&format!(
                "reflection_object_runtime_{}",
                label_fragment(class_name)
            ))
        })
        .collect::<Vec<_>>();
    ctx.load_value_to_reg(source, &result_reg)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            if boxed_source {
                abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
                ctx.emitter.instruction("cmp x0, #6");                          // ReflectionObject requires a boxed object payload
                ctx.emitter.instruction(&format!("b.ne {}", miss_label));
                ctx.emitter.instruction("ldr x9, [x1]");                        // load the reflected object's concrete class id
            } else {
                ctx.emitter.instruction("ldr x9, [x0]");                        // load the reflected object's concrete class id
            }
            for ((_, class_id), label) in candidates.iter().zip(match_labels.iter()) {
                abi::emit_load_int_immediate(ctx.emitter, "x10", *class_id as i64);
                ctx.emitter.instruction("cmp x9, x10");                         // compare this reflected-class candidate id
                ctx.emitter.instruction(&format!("b.eq {}", label));            // materialize metadata for the matching object class
            }
        }
        Arch::X86_64 => {
            if boxed_source {
                abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
                ctx.emitter.instruction("cmp rax, 6");                          // ReflectionObject requires a boxed object payload
                ctx.emitter.instruction(&format!("jne {}", miss_label));
                ctx.emitter.instruction("mov r11, QWORD PTR [rdi]");            // load the reflected object's concrete class id
            } else {
                ctx.emitter.instruction("mov r11, QWORD PTR [rax]");            // load the reflected object's concrete class id
            }
            for ((_, class_id), label) in candidates.iter().zip(match_labels.iter()) {
                abi::emit_load_int_immediate(ctx.emitter, "r10", *class_id as i64);
                ctx.emitter.instruction("cmp r11, r10");                        // compare this reflected-class candidate id
                ctx.emitter.instruction(&format!("je {}", label));              // materialize metadata for the matching object class
            }
        }
    }
    abi::emit_jump(ctx.emitter, &miss_label);

    let source_metadata_only = reflection_object_uses_only_source_metadata(ctx, inst);
    for ((class_name, _), label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        if source_metadata_only {
            emit_source_file_reflection_object(ctx, class_name)?;
        } else {
            emit_full_reflection_object(ctx, class_name)?;
        }
        abi::emit_jump(ctx.emitter, &done_label);
    }
    ctx.emitter.label(&miss_label);
    let message = b"Fatal error: ReflectionObject runtime class is not available\n";
    super::super::emit_fatal_message(ctx, message);
    ctx.emitter.label(&done_label);
    Ok(true)
}

/// Returns whether a reflection object only flows into source-location and name queries.
///
/// The source metadata materializer preserves every observable field consumed by
/// `getFileName()` and `name` while avoiding eager construction of method, property, and constant
/// reflection graphs. Any escape, control-flow transfer, unrecognized alias, or other member
/// access conservatively selects the full materializer instead.
fn reflection_object_uses_only_source_metadata(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> bool {
    let Some(result) = inst.result else {
        return false;
    };
    let mut object_aliases = HashSet::from([result]);
    collect_transparent_aliases(ctx, &mut object_aliases);

    let mut slots = HashSet::new();
    let mut saw_direct_source_metadata = false;
    for instruction in &ctx.function.instructions {
        if !instruction
            .operands
            .iter()
            .any(|operand| object_aliases.contains(operand))
        {
            continue;
        }
        match instruction.op {
            Op::StoreLocal => match instruction.immediate {
                Some(Immediate::LocalSlot(slot)) => {
                    slots.insert(slot);
                }
                _ => return false,
            },
            Op::MethodCall
                if instruction
                    .operands
                    .first()
                    .is_some_and(|receiver| object_aliases.contains(receiver))
                    && reflection_member_name(ctx, instruction)
                        .is_some_and(|method| method.eq_ignore_ascii_case("getFileName")) =>
            {
                saw_direct_source_metadata = true;
            }
            Op::PropGet
                if instruction
                    .operands
                    .first()
                    .is_some_and(|receiver| object_aliases.contains(receiver))
                    && reflection_member_name(ctx, instruction)
                        .is_some_and(|property| property.eq_ignore_ascii_case("name")) =>
            {
                saw_direct_source_metadata = true;
            }
            Op::Acquire | Op::Borrow | Op::Move | Op::Release | Op::ReleaseUnlessAliases => {}
            _ => return false,
        }
    }
    if saw_direct_source_metadata {
        return !ctx
            .function
            .blocks
            .iter()
            .filter_map(|block| block.terminator.as_ref())
            .any(|terminator| terminator_uses_alias(terminator, &object_aliases));
    }
    let Some(slot) = (slots.len() == 1).then(|| *slots.iter().next().unwrap()) else {
        return false;
    };

    let mut local_aliases = ctx
        .function
        .instructions
        .iter()
        .filter_map(|instruction| {
            (instruction.op == Op::LoadLocal
                && instruction.immediate == Some(Immediate::LocalSlot(slot)))
            .then_some(instruction.result)
            .flatten()
        })
        .collect::<HashSet<_>>();
    if local_aliases.is_empty() {
        return false;
    }
    collect_transparent_aliases(ctx, &mut local_aliases);

    let mut saw_source_metadata = false;
    for instruction in &ctx.function.instructions {
        if !instruction
            .operands
            .iter()
            .any(|operand| local_aliases.contains(operand))
        {
            continue;
        }
        if instruction.op == Op::MethodCall
            && instruction
                .operands
                .first()
                .is_some_and(|receiver| local_aliases.contains(receiver))
            && reflection_member_name(ctx, instruction)
                .is_some_and(|method| method.eq_ignore_ascii_case("getFileName"))
        {
            saw_source_metadata = true;
            continue;
        }
        if instruction.op == Op::PropGet
            && instruction
                .operands
                .first()
                .is_some_and(|receiver| local_aliases.contains(receiver))
            && reflection_member_name(ctx, instruction)
                .is_some_and(|property| property.eq_ignore_ascii_case("name"))
        {
            saw_source_metadata = true;
            continue;
        }
        if matches!(instruction.op, Op::Acquire | Op::Borrow | Op::Move | Op::Release) {
            continue;
        }
        return false;
    }

    saw_source_metadata
        && !ctx
            .function
            .blocks
            .iter()
            .filter_map(|block| block.terminator.as_ref())
            .any(|terminator| terminator_uses_alias(terminator, &local_aliases))
}

/// Extends `aliases` through SSA ownership-preserving forwarding instructions.
fn collect_transparent_aliases(ctx: &FunctionContext<'_>, aliases: &mut HashSet<ValueId>) {
    loop {
        let mut changed = false;
        for instruction in &ctx.function.instructions {
            if !matches!(instruction.op, Op::Acquire | Op::Borrow | Op::Move)
                || instruction.operands.len() != 1
                || !aliases.contains(&instruction.operands[0])
            {
                continue;
            }
            if let Some(result) = instruction.result {
                changed |= aliases.insert(result);
            }
        }
        if !changed {
            return;
        }
    }
}

/// Returns the source-level property or method name attached to an EIR member instruction.
fn reflection_member_name<'a>(
    ctx: &'a FunctionContext<'_>,
    instruction: &Instruction,
) -> Option<&'a str> {
    let Some(Immediate::Data(data)) = instruction.immediate else {
        return None;
    };
    ctx.module
        .data
        .strings
        .get(data.as_raw() as usize)
        .map(String::as_str)
}

/// Returns whether a control-flow terminator consumes any alias from `aliases`.
fn terminator_uses_alias(terminator: &Terminator, aliases: &HashSet<ValueId>) -> bool {
    let contains = |value: &ValueId| aliases.contains(value);
    match terminator {
        Terminator::Br { args, .. } => args.iter().any(contains),
        Terminator::CondBr {
            cond,
            then_args,
            else_args,
            ..
        } => contains(cond) || then_args.iter().any(contains) || else_args.iter().any(contains),
        Terminator::Switch {
            scrutinee,
            cases,
            default_args,
            ..
        } => {
            contains(scrutinee)
                || cases.iter().any(|case| case.args.iter().any(contains))
                || default_args.iter().any(contains)
        }
        Terminator::Return { value } => value.as_ref().is_some_and(contains),
        Terminator::Throw { value } => contains(value),
        Terminator::GeneratorSuspend {
            key,
            value,
            resume_args,
            ..
        } => {
            key.as_ref().is_some_and(contains)
                || value.as_ref().is_some_and(contains)
                || resume_args.iter().any(contains)
        }
        Terminator::Fatal { .. } | Terminator::Unreachable => false,
    }
}

/// Returns whether a constructor argument needs runtime Reflection metadata lookup.
pub(super) fn reflection_owner_requires_runtime_metadata(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<bool> {
    for operand in &inst.operands {
        let literal_source = reflection_literal_source(ctx, *operand)?;
        if literal_source != *operand {
            let source_value = ctx.function.value(literal_source).ok_or_else(|| {
                CodegenIrError::missing_entry("value", literal_source.as_raw())
            })?;
            if let ValueDef::Instruction { inst: source_inst, .. } = source_value.def {
                let source_inst = ctx.function.instruction(source_inst).ok_or_else(|| {
                    CodegenIrError::missing_entry("instruction", source_inst.as_raw())
                })?;
                if matches!(
                    source_inst.op,
                    Op::ConstStr | Op::ConstClassName | Op::ConstI64
                ) {
                    continue;
                }
            }
        }
        let ty = ctx.value_php_type(*operand)?.codegen_repr();
        if matches!(ty, PhpType::Callable | PhpType::Object(_)) {
            return Ok(true);
        }
        if matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
            return Ok(true);
        }
        if ty == PhpType::Str {
            let Some(value) = ctx.function.value(*operand) else {
                return Err(CodegenIrError::missing_entry("value", operand.as_raw()));
            };
            let ValueDef::Instruction { inst: defining_inst, .. } = value.def else {
                return Ok(true);
            };
            let Some(defining_inst) = ctx.function.instruction(defining_inst) else {
                return Err(CodegenIrError::missing_entry(
                    "instruction",
                    defining_inst.as_raw(),
                ));
            };
            if !matches!(defining_inst.op, Op::ConstStr | Op::ConstClassName) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Returns true when a runtime candidate class can inhabit an object's static type.
pub(in crate::codegen::lower_inst::objects) fn reflection_class_matches_object_type(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    static_type: &str,
) -> bool {
    if reflection_same_php_type_name(class_name, static_type) {
        return true;
    }
    if resolve_reflection_interface(ctx, static_type).is_some() {
        return reflection_class_implements_interface(ctx, class_name, static_type);
    }
    reflection_class_extends_class(ctx, class_name, static_type)
}

/// Returns true when two PHP type names compare case-insensitively after namespace trimming.
fn reflection_same_php_type_name(left: &str, right: &str) -> bool {
    php_symbol_key(left.trim_start_matches('\\')) == php_symbol_key(right.trim_start_matches('\\'))
}

/// Returns true when a runtime candidate class is or extends `target_class`.
fn reflection_class_extends_class(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    target_class: &str,
) -> bool {
    let mut current = Some(class_name.to_string());
    while let Some(name) = current {
        if reflection_same_php_type_name(&name, target_class) {
            return true;
        }
        current = resolve_reflection_class(ctx, &name)
            .and_then(|(_, class_info)| class_info.parent.clone());
    }
    false
}

/// Returns true when a runtime candidate class implements the requested interface.
fn reflection_class_implements_interface(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    target_interface: &str,
) -> bool {
    let mut current = Some(class_name.to_string());
    while let Some(name) = current {
        let Some((_, class_info)) = resolve_reflection_class(ctx, &name) else {
            return false;
        };
        if class_info.interfaces.iter().any(|interface_name| {
            reflection_interface_extends_interface(ctx, interface_name, target_interface)
        }) {
            return true;
        }
        current = class_info.parent.clone();
    }
    false
}

/// Returns true when an interface is or extends the requested interface target.
fn reflection_interface_extends_interface(
    ctx: &FunctionContext<'_>,
    interface_name: &str,
    target_interface: &str,
) -> bool {
    if reflection_same_php_type_name(interface_name, target_interface) {
        return true;
    }
    let Some(interface_name) = resolve_reflection_interface(ctx, interface_name) else {
        return false;
    };
    let Some(interface) = ctx.module.interface_infos.get(interface_name) else {
        return false;
    };
    interface
        .parents
        .iter()
        .any(|parent| reflection_interface_extends_interface(ctx, parent, target_interface))
}
