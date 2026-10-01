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

/// Emits `ReflectionProperty(<class>, <property>)` for a property name the caller matched at
/// runtime, through the shared materializer the literal spelling of the same pair uses.
pub(super) fn emit_direct_reflection_property_object(
    ctx: &mut FunctionContext<'_>,
    class_operand: ReflectionLiteralOperand,
    property: &str,
    metadata: &ReflectionOwnerMetadata,
) -> Result<()> {
    emit_reflection_materializer(
        ctx,
        ReflectionMaterializerKey::Direct {
            owner: ReflectionOwnerKind::Property,
            operands: vec![class_operand, ReflectionLiteralOperand::Text(property.to_string())],
        },
        "ReflectionProperty",
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
        let Some(operand) = reflection_literal_operand(ctx, *value)? else {
            return Ok(None);
        };
        operands.push(operand);
    }
    Ok(Some(operands))
}

/// Extracts one constructor operand's constant payload, or `None` when it is not a literal.
pub(super) fn reflection_literal_operand(
    ctx: &FunctionContext<'_>,
    value: ValueId,
) -> Result<Option<ReflectionLiteralOperand>> {
    let value = reflection_literal_source(ctx, value)?;
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
    Ok(Some(operand))
}

/// Emits or reuses one zero-argument helper whose invocation allocates a fresh object graph.
fn emit_reflection_materializer(
    ctx: &mut FunctionContext<'_>,
    key: ReflectionMaterializerKey,
    class_name: &str,
    metadata: &ReflectionOwnerMetadata,
) -> Result<()> {
    let label = reflection_materializer_label(ctx, key, class_name, metadata)?;
    abi::emit_call_label_long_range(ctx.emitter, &label);
    Ok(())
}

/// Returns the label of the shared shallow `ReflectionClass` materializer for one class,
/// emitting the helper first when the module has none yet, without calling it.
///
/// A member reflector records this address instead of the object it builds, and
/// `__elephc_reflection_materialize()` calls it when `getDeclaringClass()` is asked for.
pub(super) fn shallow_reflection_class_materializer_label(
    ctx: &mut FunctionContext<'_>,
    reflected_name: &str,
) -> Result<String> {
    let metadata = reflection_shallow_class_metadata_for_name(ctx, reflected_name)?;
    reflection_materializer_label(
        ctx,
        ReflectionMaterializerKey::ShallowClass {
            owner: ReflectionOwnerKind::Class,
            name: reflected_name.to_string(),
        },
        "ReflectionClass",
        &metadata,
    )
}

/// Lowers `__elephc_reflection_materialize($address)`: calls the materializer whose address a
/// member reflector recorded and stores the fresh `ReflectionClass` it returns.
pub(in crate::codegen::lower_inst) fn lower_reflection_materialize(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let address = *inst
        .operands
        .first()
        .ok_or_else(|| CodegenIrError::invalid_module("reflection_materialize missing address"))?;
    ctx.load_value_to_result(address)?;
    let target = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_reg_move(ctx.emitter, target, abi::int_result_reg(ctx.emitter));
    abi::emit_call_reg(ctx.emitter, target);
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection_materialize missing result"))?;
    ctx.store_result_value(result)
}

/// Lowers `__elephc_reflection_fill_members($filler, $reflector)`: calls the member filler whose
/// address a class reflector recorded, with the reflector in the integer result register where
/// the filler's slot emitters expect their object.
pub(in crate::codegen::lower_inst) fn lower_reflection_fill_members(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let (filler, reflector) = match inst.operands.as_slice() {
        [filler, reflector] => (*filler, *reflector),
        _ => {
            return Err(CodegenIrError::invalid_module(
                "reflection_fill_members needs a filler and a reflector",
            ))
        }
    };
    ctx.load_value_to_result(filler)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    ctx.load_value_to_result(reflector)?;
    let target = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_pop_reg(ctx.emitter, target);
    abi::emit_call_reg(ctx.emitter, target);
    if let Some(result) = inst.result {
        abi::emit_load_int_immediate(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            crate::codegen_support::NULL_SENTINEL,
        );
        ctx.store_result_value(result)?;
    }
    Ok(())
}

/// Returns the label of the shared member filler for one fully reflected class-like, emitting
/// the helper first when the module has none yet, without calling it.
///
/// The filler takes the reflector in the integer result register, clears its
/// `__members_filler` slot, and builds the member reflectors the eager materializer used to:
/// methods, properties, constants, constructor, parent, interfaces, traits and enum cases.
fn reflection_members_filler_label(
    ctx: &mut FunctionContext<'_>,
    owner: ReflectionOwnerKind,
    class_name: &str,
    metadata: &ReflectionOwnerMetadata,
) -> Result<String> {
    let name = metadata.reflected_name.clone().unwrap_or_default();
    let key = ReflectionMaterializerKey::Members { owner, name };
    if let Some(label) = ctx.shared.reflection.materializer_label(&key) {
        return Ok(label);
    }
    let checkpoint = ctx.shared.reflection.materializer_checkpoint();
    let label = ctx.next_label("reflection_members_filler");
    let done_label = ctx.next_label("reflection_members_filler_done");
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
        &format!("--- shared {} member filler ---", class_name),
        |helper_ctx| {
            emit_reflection_owner_int_property(helper_ctx, class_name, "__members_filler", 0)?;
            emit_reflection_owner_members(helper_ctx, class_name, metadata)
        },
    );
    if let Err(error) = result {
        ctx.shared
            .reflection
            .rollback_materializers(checkpoint);
        return Err(error);
    }
    ctx.emitter.label(&done_label);
    Ok(label)
}

/// Returns the label of one shared materializer, emitting its helper body first when the module
/// has none yet. The body is emitted inline behind a jump, so the caller's code flows past it.
fn reflection_materializer_label(
    ctx: &mut FunctionContext<'_>,
    key: ReflectionMaterializerKey,
    class_name: &str,
    metadata: &ReflectionOwnerMetadata,
) -> Result<String> {
    if let Some(label) = ctx.shared.reflection.materializer_label(&key) {
        return Ok(label);
    }

    let checkpoint = ctx.shared.reflection.materializer_checkpoint();
    let label = ctx.next_label("reflection_materializer");
    let done_label = ctx.next_label("reflection_materializer_done");
    // A full class-like reflector has every slot its native bodies read, so compiled calls on it
    // skip the eval bridge (`emit_reflector_is_complete_probe`); shallow and source-only ones
    // do not, and neither does one the bridge builds. Its member reflectors are built on first
    // use by a shared filler (`reflection_members_filler_label`).
    let lazy_members_owner = match &key {
        ReflectionMaterializerKey::FullClass { owner, .. } => Some(owner.clone()),
        ReflectionMaterializerKey::Direct {
            owner:
                owner @ (ReflectionOwnerKind::Class
                | ReflectionOwnerKind::Object
                | ReflectionOwnerKind::Enum),
            ..
        } => Some(owner.clone()),
        _ => None,
    };
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
        |helper_ctx| {
            let Some(owner) = lazy_members_owner else {
                return emit_reflection_owner_object(helper_ctx, class_name, metadata);
            };
            let filler =
                reflection_members_filler_label(helper_ctx, owner, class_name, metadata)?;
            emit_reflection_owner_object_with_members(
                helper_ctx,
                class_name,
                metadata,
                Some(&filler),
            )?;
            emit_reflection_owner_bool_property(helper_ctx, class_name, "__complete", true)
        },
    );
    if let Err(error) = result {
        ctx.shared
            .reflection
            .rollback_materializers(checkpoint);
        return Err(error);
    }

    ctx.emitter.label(&done_label);
    Ok(label)
}
