//! Purpose:
//! Reflection owner detection and runtime-class dispatch.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::reflection`.
//!
//! Key details:
//! - Preserves compile-time metadata, target-aware object layout, and ownership.

use super::*;

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
    if reflection_owner_requires_runtime_metadata(ctx, inst)? {
        return crate::codegen::lower_inst::builtins::lower_eval_native_object_new(ctx, inst);
    } else {
        let metadata = reflection_owner_metadata(ctx, class_name, inst)?;
        emit_reflection_owner_object(ctx, class_name, &metadata)?;
    }
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Returns whether a constructor argument needs runtime Reflection metadata lookup.
pub(super) fn reflection_owner_requires_runtime_metadata(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<bool> {
    for operand in &inst.operands {
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
