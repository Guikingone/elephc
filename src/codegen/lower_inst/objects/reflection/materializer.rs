//! Purpose:
//! Hoists compile-time Reflection object graphs into reusable module-wide helper bodies.
//! Preserves fresh runtime allocation while replacing repeated inline assembly with calls.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::reflection::owner_dispatch` for literal constructors.
//! - Nested Reflection member, parameter, parent, interface, trait, and enum emitters.
//!
//! Key details:
//! - Exact typed keys are reserved before body emission to terminate recursive codegen graphs.
//! - Failed helper subtrees roll back every cache reservation created with their assembly.
//! - AArch64 calls materialize the helper address and use `blr`, avoiding direct-call range limits.

use crate::codegen::shared_helper::emit_shared_object_helper;
use crate::codegen::shared_reflection::{
    ReflectionLiteralOperand, ReflectionMaterializerKey, ReflectionOwnerKind,
};

use super::*;

/// Emits a literal Reflection constructor through an exact shared materializer when eligible.
pub(super) fn emit_direct_reflection_owner_object(
    ctx: &mut FunctionContext<'_>,
    class_name: &str,
    inst: &Instruction,
    metadata: &ReflectionOwnerMetadata,
) -> Result<()> {
    let Some(owner) = ReflectionOwnerKind::from_class_name(class_name) else {
        return emit_reflection_owner_object(ctx, class_name, metadata);
    };
    let Some(operands) = reflection_literal_operands(ctx, inst)? else {
        return emit_reflection_owner_object(ctx, class_name, metadata);
    };
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::Direct { owner, operands },
        class_name,
        metadata,
    )
}

/// Emits a full nested `ReflectionClass` materializer for one resolved class-like name.
pub(super) fn emit_full_reflection_class_object(
    ctx: &mut FunctionContext<'_>,
    reflected_name: &str,
) -> Result<()> {
    let metadata = reflection_class_metadata_for_name(ctx, reflected_name)?;
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::FullClass {
            owner: ReflectionOwnerKind::Class,
            name: reflected_name.to_string(),
        },
        "ReflectionClass",
        &metadata,
    )
}

/// Emits a full `ReflectionObject` materializer for one runtime-selected object class.
pub(super) fn emit_full_reflection_object(
    ctx: &mut FunctionContext<'_>,
    reflected_name: &str,
) -> Result<()> {
    let metadata = reflection_class_metadata_for_name(ctx, reflected_name)?;
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::FullClass {
            owner: ReflectionOwnerKind::Object,
            name: reflected_name.to_string(),
        },
        "ReflectionObject",
        &metadata,
    )
}

/// Emits a ReflectionObject materializer containing only source-location metadata.
pub(super) fn emit_source_file_reflection_object(
    ctx: &mut FunctionContext<'_>,
    reflected_name: &str,
) -> Result<()> {
    let metadata = reflection_source_file_class_metadata_for_name(ctx, reflected_name)?;
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::SourceFileClass {
            owner: ReflectionOwnerKind::Object,
            name: reflected_name.to_string(),
        },
        "ReflectionObject",
        &metadata,
    )
}

/// Emits a shallow nested `ReflectionClass` materializer for one resolved class-like name.
pub(super) fn emit_shallow_reflection_class_object(
    ctx: &mut FunctionContext<'_>,
    reflected_name: &str,
) -> Result<()> {
    let metadata = reflection_shallow_class_metadata_for_name(ctx, reflected_name)?;
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::ShallowClass {
            owner: ReflectionOwnerKind::Class,
            name: reflected_name.to_string(),
        },
        "ReflectionClass",
        &metadata,
    )
}

/// Emits a shallow nested `ReflectionEnum` materializer for one resolved enum name.
pub(super) fn emit_shallow_reflection_enum_object(
    ctx: &mut FunctionContext<'_>,
    reflected_name: &str,
) -> Result<()> {
    let metadata = reflection_shallow_enum_metadata_for_name(ctx, reflected_name)?;
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::ShallowEnum {
            owner: ReflectionOwnerKind::Enum,
            name: reflected_name.to_string(),
        },
        "ReflectionEnum",
        &metadata,
    )
}

/// Emits a nested declaring-function materializer from already resolved shallow metadata.
pub(super) fn emit_declaring_function_object(
    ctx: &mut FunctionContext<'_>,
    function_name: &str,
    metadata: &ReflectionOwnerMetadata,
) -> Result<()> {
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::DeclaringFunction {
            owner: ReflectionOwnerKind::Function,
            name: function_name.to_string(),
        },
        "ReflectionFunction",
        metadata,
    )
}

/// Emits a nested declaring-method materializer from already resolved shallow metadata.
pub(super) fn emit_declaring_method_object(
    ctx: &mut FunctionContext<'_>,
    declaring_class: Option<&str>,
    method_name: &str,
    metadata: &ReflectionOwnerMetadata,
) -> Result<()> {
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::DeclaringMethod {
            owner: ReflectionOwnerKind::Method,
            declaring_class: declaring_class.map(str::to_string),
            name: method_name.to_string(),
        },
        "ReflectionMethod",
        metadata,
    )
}

/// Extracts the exact constant payload admitted by the compile-time Reflection resolvers.
fn reflection_literal_operands(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<Option<Vec<ReflectionLiteralOperand>>> {
    let mut operands = Vec::with_capacity(inst.operands.len());
    for value in &inst.operands {
        let value = reflection_literal_source(ctx, *value)?;
        let value_ref = ctx
            .function
            .value(value)
            .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
        let ValueDef::Instruction {
            inst: defining_inst,
            ..
        } = value_ref.def
        else {
            return Ok(None);
        };
        let defining = ctx
            .function
            .instruction(defining_inst)
            .ok_or_else(|| CodegenIrError::missing_entry("instruction", defining_inst.as_raw()))?;
        let operand = match (defining.op, defining.immediate.as_ref()) {
            (Op::ConstStr, Some(Immediate::Data(data))) => ctx
                .module
                .data
                .strings
                .get(data.as_raw() as usize)
                .cloned()
                .map(ReflectionLiteralOperand::Text),
            (Op::ConstClassName, Some(Immediate::Data(data))) => ctx
                .module
                .data
                .class_names
                .get(data.as_raw() as usize)
                .cloned()
                .map(ReflectionLiteralOperand::Text),
            (Op::ConstI64, Some(Immediate::I64(position))) => {
                Some(ReflectionLiteralOperand::Position(*position))
            }
            _ => return Ok(None),
        }
        .ok_or_else(|| CodegenIrError::missing_entry("reflection literal data", value.as_raw()))?;
        operands.push(operand);
    }
    Ok(Some(operands))
}

/// Emits or reuses one zero-argument helper whose invocation allocates a fresh object graph.
fn emit_reflection_materializer(
    ctx: &mut FunctionContext<'_>,
    key: ReflectionMaterializerKey,
    class_name: &str,
    metadata: &ReflectionOwnerMetadata,
) -> Result<()> {
    if let Some(label) = ctx.shared.reflection.materializer_label(&key) {
        abi::emit_call_label_long_range(ctx.emitter, &label);
        return Ok(());
    }

    let checkpoint = ctx.shared.reflection.materializer_checkpoint();
    let label = ctx.next_label("reflection_materializer");
    let done_label = ctx.next_label("reflection_materializer_done");
    ctx.shared
        .reflection
        .reserve_materializer(key, label.clone());
    abi::emit_jump(ctx.emitter, &done_label);

    let result = emit_shared_object_helper(
        ctx.module,
        ctx.emitter,
        ctx.data,
        ctx.shared,
        &label,
        class_name,
        &format!("--- shared {} materializer ---", class_name),
        |helper_ctx| emit_reflection_owner_object(helper_ctx, class_name, metadata),
    );
    if let Err(error) = result {
        ctx.shared
            .reflection
            .rollback_materializers(checkpoint);
        return Err(error);
    }

    ctx.emitter.label(&done_label);
    abi::emit_call_label_long_range(ctx.emitter, &label);
    Ok(())
}
