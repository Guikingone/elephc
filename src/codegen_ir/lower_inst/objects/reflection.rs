//! Purpose:
//! Lowers metadata-aware allocation for builtin Reflection owner objects in the
//! EIR backend.
//!
//! Called from:
//! - `crate::codegen_ir::lower_inst::objects::lower_object_new()`.
//!
//! Key details:
//! - `ReflectionClass`, `ReflectionMethod`, and `ReflectionProperty`
//!   constructors are compile-time metadata lookups that populate private
//!   `__name`/`__attrs` slots instead of running their public empty bodies.
//! - `ReflectionClass` construction also bakes a family of closed-world
//!   metadata slots (`__is_abstract`, `__ancestors_lower`, `__const_names`/
//!   `__const_values`, …) that back the shell's `isAbstract`/`isSubclassOf`/
//!   `hasMethod`/`getConstants`/… method bodies declared in
//!   `crate::types::checker::builtin_types::reflection`; `ReflectionMethod`/
//!   `ReflectionProperty` construction bakes a `__modifiers` bitmask (plus
//!   `ReflectionMethod::__name`, previously never populated, and
//!   `ReflectionProperty::__has_declared_type`) from the reflected member's
//!   real declaration.
//! - Every `emit_reflection_*` baker in this file shares one calling
//!   convention: the Reflection object pointer is held in the ABI int-result
//!   register on entry and must be left there on exit, so sequential bakers
//!   chain without re-parking the object between calls.

use std::collections::HashSet;

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen_ir::{CodegenIrError, Result};
use crate::ir::{Immediate, Instruction, Op, ValueDef, ValueId};
use crate::names::php_symbol_key;
use crate::parser::ast::Visibility;
use crate::types::{AttrArgEntry, AttrArgValue, AttrKey};

use super::super::super::context::FunctionContext;

/// Compile-time metadata used to populate one Reflection owner object.
struct ReflectionOwnerMetadata {
    reflected_name: Option<String>,
    attr_names: Vec<String>,
    attr_args: Vec<Option<Vec<AttrArgEntry>>>,
    /// The resolved reflected class name for `ReflectionMethod`/
    /// `ReflectionProperty` constructions (the `ReflectionClass` case already
    /// has this in `reflected_name`). Used to bake `__modifiers`/
    /// `__has_declared_type`.
    member_owner_class: Option<String>,
    /// The reflected method or property name, PHP-case-folded for method
    /// lookups (property lookups stay exact-case). Used alongside
    /// `member_owner_class` to bake `__modifiers`/`__has_declared_type`.
    member_name: Option<String>,
}

/// Returns true for reflection owner classes that need metadata-aware construction.
pub(super) fn is_reflection_owner_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "ReflectionClass" | "ReflectionMethod" | "ReflectionProperty"
    )
}

/// Lowers builtin Reflection owner allocation by populating compile-time metadata slots.
pub(super) fn lower_reflection_owner_new(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_name: &str,
) -> Result<()> {
    let metadata = reflection_owner_metadata(ctx, class_name, inst)?;
    let (class_id, property_count, uninitialized_marker_offsets) = {
        let class_info = ctx
            .module
            .class_infos
            .get(class_name)
            .ok_or_else(|| CodegenIrError::unsupported(format!("unknown class {}", class_name)))?;
        (
            class_info.class_id,
            class_info.properties.len(),
            super::uninitialized_property_marker_offsets(class_info),
        )
    };
    super::emit_object_allocation(
        ctx,
        class_id,
        property_count,
        false,
        &uninitialized_marker_offsets,
        &[],
    )?;
    if let Some(reflected_name) = metadata.reflected_name.as_deref() {
        emit_reflection_string_property(ctx, reflected_name, 8, 16);
    }
    emit_reflection_attrs_property(
        ctx,
        class_name,
        &metadata.attr_names,
        &metadata.attr_args,
    )?;
    match class_name {
        "ReflectionClass" => {
            if let Some(reflected_name) = metadata.reflected_name.as_deref() {
                emit_reflection_class_extra_metadata(ctx, reflected_name)?;
            }
        }
        "ReflectionMethod" => {
            if let (Some(owner_class), Some(method_key)) =
                (metadata.member_owner_class.as_deref(), metadata.member_name.as_deref())
            {
                emit_reflection_method_modifiers(ctx, owner_class, method_key)?;
            }
        }
        "ReflectionProperty" => {
            if let (Some(owner_class), Some(property_name)) =
                (metadata.member_owner_class.as_deref(), metadata.member_name.as_deref())
            {
                emit_reflection_property_modifiers(ctx, owner_class, property_name)?;
            }
        }
        _ => {}
    }
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Lowers `new ReflectionFunction("name")` by populating its name and
/// parameter-count slots from the reflected function's signature. The slot
/// layout is `__name` (8/16), `__short` (24/32), `__num_params` (40/48),
/// `__num_required` (56/64).
pub(super) fn lower_reflection_function_new(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let (full_name, short_name, num_params, num_required) = reflection_function_metadata(ctx, inst)?;
    let (class_id, property_count, uninitialized_marker_offsets, name_off, short_off, np_off, nr_off) = {
        let class_info = ctx
            .module
            .class_infos
            .get("ReflectionFunction")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionFunction"))?;
        let slot = |name: &str| -> Result<usize> {
            class_info
                .property_offsets
                .get(name)
                .copied()
                .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))
        };
        (
            class_info.class_id,
            class_info.properties.len(),
            super::uninitialized_property_marker_offsets(class_info),
            slot("__name")?,
            slot("__short")?,
            slot("__num_params")?,
            slot("__num_required")?,
        )
    };
    super::emit_object_allocation(
        ctx,
        class_id,
        property_count,
        false,
        &uninitialized_marker_offsets,
        &[],
    )?;
    emit_reflection_string_property(ctx, &full_name, name_off, name_off + 8);
    emit_reflection_string_property(ctx, &short_name, short_off, short_off + 8);
    emit_reflection_int_property(ctx, num_params, np_off, np_off + 8);
    emit_reflection_int_property(ctx, num_required, nr_off, nr_off + 8);

    // Build the `ReflectionParameter[]` array and store it into `__params`.
    let params_off = ctx
        .module
        .class_infos
        .get("ReflectionFunction")
        .and_then(|ci| ci.property_offsets.get("__params").copied())
        .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))?;
    let param_infos = reflection_function_param_infos(ctx, &full_name);
    let (rp_class_id, rp_prop_count, rp_markers, rp_name, rp_pos, rp_opt, rp_var, rp_type, rp_has_type) = {
        let ci = ctx
            .module
            .class_infos
            .get("ReflectionParameter")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionParameter"))?;
        let slot = |n: &str| -> Result<usize> {
            ci.property_offsets
                .get(n)
                .copied()
                .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))
        };
        (
            ci.class_id,
            ci.properties.len(),
            super::uninitialized_property_marker_offsets(ci),
            slot("__name")?,
            slot("__position")?,
            slot("__optional")?,
            slot("__variadic")?,
            slot("__type")?,
            slot("__has_type")?,
        )
    };
    let result_reg = abi::int_result_reg(ctx.emitter);
    let object_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_load_temporary_stack_slot(ctx.emitter, object_reg, 0);
    abi::emit_load_from_address(ctx.emitter, result_reg, object_reg, params_off);
    abi::emit_call_label(ctx.emitter, "__rt_decref_array");
    emit_reflection_parameter_array(
        ctx,
        &param_infos,
        rp_class_id,
        rp_prop_count,
        &rp_markers,
        rp_name,
        rp_pos,
        rp_opt,
        rp_var,
        rp_type,
        rp_has_type,
    )?;
    abi::emit_pop_reg(ctx.emitter, object_reg);
    abi::emit_store_to_address(ctx.emitter, result_reg, object_reg, params_off);
    abi::emit_load_int_immediate(ctx.emitter, abi::secondary_scratch_reg(ctx.emitter), 4);
    abi::emit_store_to_address(
        ctx.emitter,
        abi::secondary_scratch_reg(ctx.emitter),
        object_reg,
        params_off + 8,
    );
    abi::emit_push_reg(ctx.emitter, object_reg);
    abi::emit_pop_reg(ctx.emitter, result_reg);

    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Resolves `ReflectionFunction(name)` to its full name, short name, and
/// parameter counts from the reflected function's lowered signature.
fn reflection_function_metadata(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<(String, String, i64, i64)> {
    let Some(name_operand) = inst.operands.first().copied() else {
        return Ok((String::new(), String::new(), 0, 0));
    };
    let function_name = const_required_string_operand(ctx, name_operand, "ReflectionFunction")?;
    let key = php_symbol_key(function_name.trim_start_matches('\\'));
    let signature = ctx
        .module
        .functions
        .iter()
        .find(|function| php_symbol_key(function.name.trim_start_matches('\\')) == key)
        .and_then(|function| function.signature.as_ref());
    let (num_params, num_required) = signature
        .map(|sig| {
            let total = sig.params.len() as i64;
            let required = sig
                .params
                .iter()
                .zip(sig.defaults.iter().chain(std::iter::repeat(&None)))
                .filter(|((name, _), default)| {
                    default.is_none() && sig.variadic.as_deref() != Some(name.as_str())
                })
                .count() as i64;
            (total, required)
        })
        .unwrap_or((0, 0));
    let short_name = function_name
        .trim_start_matches('\\')
        .rsplit('\\')
        .next()
        .unwrap_or(&function_name)
        .to_string();
    Ok((function_name.clone(), short_name, num_params, num_required))
}

/// Stores an integer immediate into a Reflection object's property slot.
fn emit_reflection_int_property(
    ctx: &mut FunctionContext<'_>,
    value: i64,
    low_offset: usize,
    high_offset: usize,
) {
    let object_reg = abi::int_result_reg(ctx.emitter);
    let scratch = abi::secondary_scratch_reg(ctx.emitter);
    abi::emit_load_int_immediate(ctx.emitter, scratch, value);
    abi::emit_store_to_address(ctx.emitter, scratch, object_reg, low_offset);
    abi::emit_load_int_immediate(ctx.emitter, scratch, 0);
    abi::emit_store_to_address(ctx.emitter, scratch, object_reg, high_offset);
}

/// Per-parameter reflection metadata for one function parameter.
struct ReflectionParamInfo {
    name: String,
    optional: bool,
    variadic: bool,
    /// `Some((type_name, is_builtin, allows_null))` when the parameter declares a
    /// single named type; `None` for an untyped parameter (`getType()` is null).
    type_info: Option<(String, bool, bool)>,
}

/// Maps a declared parameter type to `ReflectionNamedType` metadata
/// `(name, is_builtin, allows_null)`, or `None` for an unsupported/union shape.
fn reflection_named_type_info(ty: &crate::types::PhpType) -> Option<(String, bool, bool)> {
    use crate::types::PhpType;
    match ty {
        PhpType::Int => Some(("int".to_string(), true, false)),
        PhpType::Str => Some(("string".to_string(), true, false)),
        PhpType::Float => Some(("float".to_string(), true, false)),
        PhpType::Bool => Some(("bool".to_string(), true, false)),
        PhpType::Array(_) | PhpType::AssocArray { .. } => Some(("array".to_string(), true, false)),
        PhpType::Callable => Some(("callable".to_string(), true, false)),
        PhpType::Iterable => Some(("iterable".to_string(), true, false)),
        // Bare `Mixed` is how an *untyped* parameter is represented in the EIR
        // signature (and `declared_params` is unreliable here — it is also set
        // for boxed-ABI params). PHP reports untyped parameters as having no
        // type, so map `Mixed` to no named type. An explicit `mixed` hint is
        // the only case this under-reports, which is an accepted edge case.
        PhpType::Object(class) => Some((class.trim_start_matches('\\').to_string(), false, false)),
        PhpType::Union(members) => {
            let has_null = members.iter().any(|m| matches!(m, PhpType::Void));
            let mut non_null = members.iter().filter(|m| !matches!(m, PhpType::Void));
            let single = non_null.next();
            // Only `T|null` (a single non-null member) maps to a named type.
            match (single, non_null.next()) {
                (Some(member), None) => reflection_named_type_info(member)
                    .map(|(name, builtin, _)| (name, builtin, has_null)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Extracts per-parameter reflection metadata from a function's lowered
/// signature. A parameter is optional once a default or the variadic is seen
/// (matching PHP's `isOptional`).
fn reflection_function_param_infos(
    ctx: &FunctionContext<'_>,
    function_name: &str,
) -> Vec<ReflectionParamInfo> {
    let key = php_symbol_key(function_name.trim_start_matches('\\'));
    let Some(signature) = ctx
        .module
        .functions
        .iter()
        .find(|function| php_symbol_key(function.name.trim_start_matches('\\')) == key)
        .and_then(|function| function.signature.as_ref())
    else {
        return Vec::new();
    };
    let mut seen_optional = false;
    signature
        .params
        .iter()
        .enumerate()
        .map(|(idx, (name, ty))| {
            let variadic = signature.variadic.as_deref() == Some(name.as_str());
            let has_default = signature.defaults.get(idx).map_or(false, Option::is_some);
            if has_default || variadic {
                seen_optional = true;
            }
            let declared = signature.declared_params.get(idx).copied().unwrap_or(false);
            let type_info = if declared {
                reflection_named_type_info(ty)
            } else {
                None
            };
            ReflectionParamInfo {
                name: name.clone(),
                optional: seen_optional,
                variadic,
                type_info,
            }
        })
        .collect()
}

/// Allocates a fresh indexed array sized for `count` object handles (8-byte stride).
fn emit_alloc_object_array(ctx: &mut FunctionContext<'_>, count: usize) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_int_immediate(ctx.emitter, "x0", count.max(1) as i64);
            abi::emit_load_int_immediate(ctx.emitter, "x1", 8);
        }
        Arch::X86_64 => {
            abi::emit_load_int_immediate(ctx.emitter, "rdi", count.max(1) as i64);
            abi::emit_load_int_immediate(ctx.emitter, "rsi", 8);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_array_new");
}

/// Pops a freshly built object and the result array off the stack and appends
/// the object handle to the array (leaving the array pointer in the result reg).
fn emit_append_object_to_array(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x1");
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            abi::emit_pop_reg(ctx.emitter, "rsi");
            abi::emit_pop_reg(ctx.emitter, "rdi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_array_push_int");
}

/// Builds an indexed array of `ReflectionParameter` objects (one per function
/// parameter), leaving the array pointer in the result register. Stack-balanced.
#[allow(clippy::too_many_arguments)]
fn emit_reflection_parameter_array(
    ctx: &mut FunctionContext<'_>,
    params: &[ReflectionParamInfo],
    class_id: u64,
    property_count: usize,
    markers: &[usize],
    name_off: usize,
    pos_off: usize,
    opt_off: usize,
    var_off: usize,
    type_off: usize,
    has_type_off: usize,
) -> Result<()> {
    // ReflectionNamedType layout for building per-parameter type objects.
    let named_type = ctx.module.class_infos.get("ReflectionNamedType").map(|ci| {
        let off = |n: &str| ci.property_offsets.get(n).copied().unwrap_or(0);
        (
            ci.class_id,
            ci.properties.len(),
            super::uninitialized_property_marker_offsets(ci),
            off("__name"),
            off("__allows_null"),
            off("__builtin"),
        )
    });
    emit_alloc_object_array(ctx, params.len());
    crate::codegen::emit_array_value_type_stamp(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        &crate::types::PhpType::Object("ReflectionParameter".to_string()),
    );
    for (position, param) in params.iter().enumerate() {
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
        super::emit_object_allocation(ctx, class_id, property_count, false, markers, &[])?;
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
        emit_reflection_string_property(ctx, &param.name, name_off, name_off + 8);
        emit_reflection_int_property(ctx, position as i64, pos_off, pos_off + 8);
        emit_reflection_int_property(ctx, param.optional as i64, opt_off, opt_off + 8);
        emit_reflection_int_property(ctx, param.variadic as i64, var_off, var_off + 8);
        if let (Some((type_name, builtin, allows_null)), Some((nt_id, nt_count, nt_markers, nt_name, nt_anull, nt_builtin))) =
            (&param.type_info, &named_type)
        {
            // Build a ReflectionNamedType (result reg); the parameter object is
            // safe on the stack at slot 0 across this balanced construction.
            super::emit_object_allocation(ctx, *nt_id, *nt_count, false, nt_markers, &[])?;
            emit_reflection_string_property(ctx, type_name, *nt_name, *nt_name + 8);
            emit_reflection_int_property(ctx, *builtin as i64, *nt_builtin, *nt_builtin + 8);
            emit_reflection_int_property(ctx, *allows_null as i64, *nt_anull, *nt_anull + 8);
            // `__type` is a `mixed` property, so its value must be a *boxed*
            // Mixed cell (the receiver later dispatches `getType()->...` through
            // the Mixed unbox path). Box the freshly built object pointer (still
            // in the result reg) into a cell, then store it as a Mixed slot:
            // boxed-cell pointer in the low word, 0 in the high word. The slot
            // was zero-initialized at allocation, so no decref of an old value
            // is required.
            crate::codegen::emit_box_current_value_as_mixed(
                ctx.emitter,
                &crate::types::PhpType::Object("ReflectionNamedType".to_string()),
            );
            let cell_reg = abi::int_result_reg(ctx.emitter);
            let param_reg = abi::symbol_scratch_reg(ctx.emitter);
            let flag_reg = abi::secondary_scratch_reg(ctx.emitter);
            abi::emit_load_temporary_stack_slot(ctx.emitter, param_reg, 0);
            abi::emit_store_to_address(ctx.emitter, cell_reg, param_reg, type_off);
            abi::emit_store_zero_to_address(ctx.emitter, param_reg, type_off + 8);
            abi::emit_load_int_immediate(ctx.emitter, flag_reg, 1);
            abi::emit_store_to_address(ctx.emitter, flag_reg, param_reg, has_type_off);
            abi::emit_store_zero_to_address(ctx.emitter, param_reg, has_type_off + 8);
        }
        emit_append_object_to_array(ctx);
    }
    Ok(())
}

/// Resolves Reflection constructor operands to captured class/member metadata.
fn reflection_owner_metadata(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    inst: &Instruction,
) -> Result<ReflectionOwnerMetadata> {
    match class_name {
        "ReflectionClass" => reflection_class_metadata(ctx, inst),
        "ReflectionMethod" => reflection_method_metadata(ctx, inst),
        "ReflectionProperty" => reflection_property_metadata(ctx, inst),
        _ => Ok(empty_reflection_metadata()),
    }
}

/// Resolves `ReflectionClass(class)` metadata.
fn reflection_class_metadata(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<ReflectionOwnerMetadata> {
    let Some(class_operand) = inst.operands.first().copied() else {
        return Ok(empty_reflection_metadata());
    };
    let reflected_class = const_string_or_class_operand(ctx, class_operand, "ReflectionClass")?;
    Ok(resolve_reflection_class(ctx, &reflected_class)
        .map(|(class_name, info)| ReflectionOwnerMetadata {
            reflected_name: Some(class_name.to_string()),
            attr_names: info.attribute_names.clone(),
            attr_args: info.attribute_args.clone(),
            member_owner_class: None,
            member_name: None,
        })
        .unwrap_or_else(empty_reflection_metadata))
}

/// Resolves `ReflectionMethod(class, method)` metadata.
fn reflection_method_metadata(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<ReflectionOwnerMetadata> {
    let Some(class_operand) = inst.operands.first().copied() else {
        return Ok(empty_reflection_metadata());
    };
    let Some(method_operand) = inst.operands.get(1).copied() else {
        return Ok(empty_reflection_metadata());
    };
    let reflected_class = const_string_or_class_operand(ctx, class_operand, "ReflectionMethod")?;
    let method_name = const_required_string_operand(ctx, method_operand, "ReflectionMethod")?;
    let method_key = php_symbol_key(&method_name);
    let Some((owner_class, _)) = resolve_reflection_class(ctx, &reflected_class) else {
        return Ok(empty_reflection_metadata());
    };
    let owner_class = owner_class.to_string();
    let mut metadata = resolve_reflection_class(ctx, &reflected_class)
        .and_then(|(_, info)| {
            Some(ReflectionOwnerMetadata {
                reflected_name: None,
                attr_names: info.method_attribute_names.get(&method_key)?.clone(),
                attr_args: info.method_attribute_args.get(&method_key)?.clone(),
                member_owner_class: None,
                member_name: None,
            })
        })
        .unwrap_or_else(empty_reflection_metadata);
    metadata.member_owner_class = Some(owner_class);
    metadata.member_name = Some(method_key);
    Ok(metadata)
}

/// Resolves `ReflectionProperty(class, property)` metadata.
fn reflection_property_metadata(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<ReflectionOwnerMetadata> {
    let Some(class_operand) = inst.operands.first().copied() else {
        return Ok(empty_reflection_metadata());
    };
    let Some(property_operand) = inst.operands.get(1).copied() else {
        return Ok(empty_reflection_metadata());
    };
    let reflected_class = const_string_or_class_operand(ctx, class_operand, "ReflectionProperty")?;
    let property_name = const_required_string_operand(ctx, property_operand, "ReflectionProperty")?;
    let Some((owner_class, _)) = resolve_reflection_class(ctx, &reflected_class) else {
        return Ok(empty_reflection_metadata());
    };
    let owner_class = owner_class.to_string();
    let mut metadata = resolve_reflection_class(ctx, &reflected_class)
        .and_then(|(_, info)| {
            Some(ReflectionOwnerMetadata {
                reflected_name: None,
                attr_names: info.property_attribute_names.get(&property_name)?.clone(),
                attr_args: info.property_attribute_args.get(&property_name)?.clone(),
                member_owner_class: None,
                member_name: None,
            })
        })
        .unwrap_or_else(empty_reflection_metadata);
    metadata.member_owner_class = Some(owner_class);
    // Property names are case-SENSITIVE in PHP; unlike the method-name key
    // above, the exact reflected spelling is kept (no `php_symbol_key` fold).
    metadata.member_name = Some(property_name);
    Ok(metadata)
}

/// Looks up class metadata by PHP-style case-insensitive name.
fn resolve_reflection_class<'a>(
    ctx: &'a FunctionContext<'_>,
    class_name: &str,
) -> Option<(&'a str, &'a crate::types::ClassInfo)> {
    let class_key = php_symbol_key(class_name.trim_start_matches('\\'));
    ctx.module
        .class_infos
        .iter()
        .find(|(candidate, _)| php_symbol_key(candidate.trim_start_matches('\\')) == class_key)
        .map(|(name, info)| (name.as_str(), info))
}

/// Returns empty Reflection metadata for unsupported dynamic constructor operands.
fn empty_reflection_metadata() -> ReflectionOwnerMetadata {
    ReflectionOwnerMetadata {
        reflected_name: None,
        attr_names: Vec::new(),
        attr_args: Vec::new(),
        member_owner_class: None,
        member_name: None,
    }
}

/// Extracts a constant string or class-name operand from an EIR value.
fn const_string_or_class_operand(
    ctx: &FunctionContext<'_>,
    value: ValueId,
    owner: &str,
) -> Result<String> {
    const_data_operand(ctx, value, owner, true)
}

/// Extracts a constant string operand from an EIR value.
fn const_required_string_operand(
    ctx: &FunctionContext<'_>,
    value: ValueId,
    owner: &str,
) -> Result<String> {
    const_data_operand(ctx, value, owner, false)
}

/// Reads a `ConstStr` or optional `ConstClassName` value from the module data pool.
fn const_data_operand(
    ctx: &FunctionContext<'_>,
    value: ValueId,
    owner: &str,
    allow_class_name: bool,
) -> Result<String> {
    let value_ref = ctx
        .function
        .value(value)
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Err(CodegenIrError::unsupported(format!(
            "{} constructor with non-literal reflection argument",
            owner
        )));
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    let Some(Immediate::Data(data)) = inst_ref.immediate else {
        return Err(CodegenIrError::invalid_module(format!(
            "{} reflection literal missing data id",
            owner
        )));
    };
    match inst_ref.op {
        Op::ConstStr => ctx
            .module
            .data
            .strings
            .get(data.as_raw() as usize)
            .cloned()
            .ok_or_else(|| CodegenIrError::missing_entry("data string", data.as_raw())),
        Op::ConstClassName if allow_class_name => ctx
            .module
            .data
            .class_names
            .get(data.as_raw() as usize)
            .cloned()
            .ok_or_else(|| CodegenIrError::missing_entry("class data", data.as_raw())),
        _ => Err(CodegenIrError::unsupported(format!(
            "{} constructor with non-literal reflection argument",
            owner
        ))),
    }
}

/// Writes a heap-persisted string into the current Reflection object result slot.
fn emit_reflection_string_property(
    ctx: &mut FunctionContext<'_>,
    value: &str,
    low_offset: usize,
    high_offset: usize,
) {
    let (label, len) = ctx.data.add_string(value.as_bytes());
    let object_reg = abi::symbol_scratch_reg(ctx.emitter);
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result_reg);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");
            abi::emit_pop_reg(ctx.emitter, object_reg);
            abi::emit_store_to_address(ctx.emitter, "x1", object_reg, low_offset);
            abi::emit_store_to_address(ctx.emitter, "x2", object_reg, high_offset);
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rax", &label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");
            abi::emit_pop_reg(ctx.emitter, object_reg);
            abi::emit_store_to_address(ctx.emitter, "rax", object_reg, low_offset);
            abi::emit_store_to_address(ctx.emitter, "rdx", object_reg, high_offset);
        }
    }
    abi::emit_push_reg(ctx.emitter, object_reg);
    abi::emit_pop_reg(ctx.emitter, result_reg);
}

/// Replaces the Reflection object's default `__attrs` array with populated metadata.
fn emit_reflection_attrs_property(
    ctx: &mut FunctionContext<'_>,
    class_name: &str,
    attr_names: &[String],
    attr_args: &[Option<Vec<AttrArgEntry>>],
) -> Result<()> {
    let (attrs_low_offset, attrs_high_offset) = reflection_attrs_offsets(class_name);
    let result_reg = abi::int_result_reg(ctx.emitter);
    let object_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_load_temporary_stack_slot(ctx.emitter, object_reg, 0);
    abi::emit_load_from_address(ctx.emitter, result_reg, object_reg, attrs_low_offset);
    abi::emit_call_label(ctx.emitter, "__rt_decref_array");
    super::super::builtins::attributes::emit_reflection_attribute_array(
        ctx, attr_names, attr_args,
    )?;
    abi::emit_pop_reg(ctx.emitter, object_reg);
    abi::emit_store_to_address(ctx.emitter, result_reg, object_reg, attrs_low_offset);
    abi::emit_load_int_immediate(ctx.emitter, abi::secondary_scratch_reg(ctx.emitter), 4);
    abi::emit_store_to_address(
        ctx.emitter,
        abi::secondary_scratch_reg(ctx.emitter),
        object_reg,
        attrs_high_offset,
    );
    abi::emit_push_reg(ctx.emitter, object_reg);
    abi::emit_pop_reg(ctx.emitter, result_reg);
    Ok(())
}

/// Returns the low/high object offsets for the private `__attrs` slot.
fn reflection_attrs_offsets(class_name: &str) -> (usize, usize) {
    if class_name == "ReflectionClass" {
        (24, 32)
    } else {
        (8, 16)
    }
}

/// A conservative, closed-world list of class names that are genuinely
/// real-PHP builtin/internal classes this compiler models (as opposed to
/// user-declared classes). Used only to back `ReflectionClass::isInternal()`.
/// Deliberately under-inclusive rather than over-inclusive: a name missing
/// from this list just makes `isInternal()` report `false` (matches PHP's
/// answer for user classes; never fabricates `true` for something that isn't
/// really a PHP internal). Matched case-insensitively via `php_symbol_key`.
const REAL_PHP_BUILTIN_CLASS_NAMES: &[&str] = &[
    "stdClass",
    "Exception",
    "Error",
    "TypeError",
    "ValueError",
    "ArgumentCountError",
    "ArithmeticError",
    "DivisionByZeroError",
    "UnhandledMatchError",
    "JsonException",
    "ErrorException",
    "FiberError",
    "Fiber",
    "Generator",
    "Closure",
    "WeakMap",
    "ArrayObject",
    "ArrayIterator",
    "SplStack",
    "SplQueue",
    "SplDoublyLinkedList",
    "SplObjectStorage",
    "SplFixedArray",
    "SplHeap",
    "SplMinHeap",
    "SplMaxHeap",
    "SplPriorityQueue",
    "DateTime",
    "DateTimeImmutable",
    "DateInterval",
    "DateTimeZone",
    "DatePeriod",
    "DOMDocument",
    "DOMElement",
    "DOMNode",
    "DOMNodeList",
    "DOMText",
    "ReflectionAttribute",
    "ReflectionClass",
    "ReflectionMethod",
    "ReflectionProperty",
    "ReflectionFunction",
    "ReflectionParameter",
    "ReflectionNamedType",
    "ReflectionType",
    "ReflectionUnionType",
    "LogicException",
    "BadFunctionCallException",
    "BadMethodCallException",
    "DomainException",
    "InvalidArgumentException",
    "LengthException",
    "OutOfRangeException",
    "RuntimeException",
    "OutOfBoundsException",
    "OverflowException",
    "RangeException",
    "UnderflowException",
    "UnexpectedValueException",
];

/// Returns `true` iff `class_name` matches a real-PHP builtin/internal class
/// this compiler models. See `REAL_PHP_BUILTIN_CLASS_NAMES`.
fn is_real_php_builtin_class(class_name: &str) -> bool {
    let key = php_symbol_key(class_name.trim_start_matches('\\'));
    REAL_PHP_BUILTIN_CLASS_NAMES
        .iter()
        .any(|name| php_symbol_key(name) == key)
}

/// Compile-time-computed `ReflectionClass` A1 metadata: closed-world facts
/// about the reflected class derivable entirely from `ClassInfo` at
/// construction time.
struct ReflectionClassExtraMetadata {
    is_abstract: bool,
    is_final: bool,
    /// Always `false`: elephc's `ReflectionClass` constructor only ever
    /// resolves to a real, closed-world CLASS (never an interface —
    /// reflecting an interface by name is rejected earlier, at the checker's
    /// `reflection_class_literal_arg` gate, as an "undefined class"). Kept as
    /// an explicit baked field (rather than a hardcoded `false` in the
    /// checker shell) so a future widening of that gate only needs to update
    /// this one computation.
    is_interface: bool,
    is_internal: bool,
    short_name: String,
    ancestors_lower: Vec<String>,
    interfaces: Vec<String>,
    interfaces_lower: Vec<String>,
    methods_lower: Vec<String>,
    properties: Vec<String>,
    const_names: Vec<String>,
    const_values: Vec<AttrArgValue>,
}

/// Computes `ReflectionClassExtraMetadata` for `class_name` from
/// `ctx.module.class_infos`. Returns `None` if the class is unknown (mirrors
/// the existing `resolve_reflection_class` fallback-to-empty convention).
fn reflection_class_extra_metadata(
    ctx: &FunctionContext<'_>,
    class_name: &str,
) -> Option<ReflectionClassExtraMetadata> {
    let info = ctx.module.class_infos.get(class_name)?;
    let short_name = class_name
        .trim_start_matches('\\')
        .rsplit('\\')
        .next()
        .unwrap_or(class_name)
        .to_string();

    // Parent classes, excluding self, plus every transitively implemented
    // interface (already flattened onto `info.interfaces`) — together these
    // are exactly the set PHP's `isSubclassOf()` accepts (php -n verified).
    let mut ancestors_lower = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(php_symbol_key(class_name.trim_start_matches('\\')));
    let mut current = info.parent.clone();
    while let Some(parent_name) = current {
        let key = php_symbol_key(parent_name.trim_start_matches('\\'));
        if !seen.insert(key.clone()) {
            break; // cycle guard against malformed metadata
        }
        ancestors_lower.push(key);
        current = ctx
            .module
            .class_infos
            .get(&parent_name)
            .and_then(|parent_info| parent_info.parent.clone());
    }
    for iface in &info.interfaces {
        ancestors_lower.push(php_symbol_key(iface.trim_start_matches('\\')));
    }

    let mut methods_lower: Vec<String> = info.methods.keys().cloned().collect();
    methods_lower.extend(info.static_methods.keys().cloned());
    methods_lower.sort();
    methods_lower.dedup();

    let mut properties: Vec<String> = info.properties.iter().map(|(name, _)| name.clone()).collect();
    properties.extend(info.static_properties.iter().map(|(name, _)| name.clone()));

    let (const_names, const_values) = collect_reflection_class_constants(ctx, class_name);

    Some(ReflectionClassExtraMetadata {
        is_abstract: info.is_abstract,
        is_final: info.is_final,
        is_interface: false,
        is_internal: is_real_php_builtin_class(class_name),
        short_name,
        ancestors_lower,
        interfaces: info.interfaces.clone(),
        interfaces_lower: info
            .interfaces
            .iter()
            .map(|name| php_symbol_key(name.trim_start_matches('\\')))
            .collect(),
        methods_lower,
        properties,
        const_names,
        const_values,
    })
}

/// Walks the reflected class's parent chain (own constants first, child wins
/// on a name collision) then its transitively-flattened interfaces,
/// collecting every class-constant name whose value expression folds to a
/// compile-time literal (see `fold_class_const_value`). Mirrors the lookup
/// order the type checker's `infer_class_constant_type_by_name` uses.
/// Returns parallel `(names, values)` vectors in insertion order.
fn collect_reflection_class_constants(
    ctx: &FunctionContext<'_>,
    class_name: &str,
) -> (Vec<String>, Vec<AttrArgValue>) {
    let mut names = Vec::new();
    let mut values = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let mut current = Some(class_name.to_string());
    let mut guard = 0usize;
    while let Some(current_class) = current {
        guard += 1;
        if guard > 64 {
            break; // cycle guard against malformed metadata
        }
        let Some(info) = ctx.module.class_infos.get(&current_class) else {
            break;
        };
        let mut names_here: Vec<&String> = info.constants.keys().collect();
        names_here.sort();
        for name in names_here {
            if !seen.insert(name.clone()) {
                continue; // a nearer class already shadows this constant name
            }
            if let Some(value) = fold_class_const_value(&info.constants[name]) {
                names.push(name.clone());
                values.push(value);
            }
        }
        current = info.parent.clone();
    }

    let reflected_interfaces = ctx
        .module
        .class_infos
        .get(class_name)
        .map(|info| info.interfaces.clone())
        .unwrap_or_default();
    for iface in &reflected_interfaces {
        let Some(iface_info) = ctx.module.interface_infos.get(iface) else {
            continue;
        };
        let mut names_here: Vec<&String> = iface_info.constants.keys().collect();
        names_here.sort();
        for name in names_here {
            if !seen.insert(name.clone()) {
                continue;
            }
            if let Some(value) = fold_class_const_value(&iface_info.constants[name]) {
                names.push(name.clone());
                values.push(value);
            }
        }
    }

    (names, values)
}

/// Folds a class-constant value expression into a compile-time
/// `AttrArgValue`, or `None` for a shape this reflection helper does not
/// evaluate at compile time (e.g. `const Y = self::X;`'s cross-referenced
/// `ScopedConstantAccess`, or a non-literal arithmetic expression). Such
/// constants are simply OMITTED from `getConstants()`/`getConstant()` — a
/// bounded, honest, documented limitation rather than a silently wrong
/// value. Handles the same literal shapes as PHP attribute-argument folding
/// (`crate::types::checker::schema::classes::state::fold_attr_value`), minus
/// symbolic constant references.
fn fold_class_const_value(expr: &crate::parser::ast::Expr) -> Option<AttrArgValue> {
    use crate::parser::ast::ExprKind;
    match &expr.kind {
        ExprKind::StringLiteral(value) => Some(AttrArgValue::Str(value.clone())),
        ExprKind::IntLiteral(value) => Some(AttrArgValue::Int(*value)),
        ExprKind::FloatLiteral(value) => Some(AttrArgValue::Float(value.to_bits())),
        ExprKind::BoolLiteral(value) => Some(AttrArgValue::Bool(*value)),
        ExprKind::Null => Some(AttrArgValue::Null),
        ExprKind::Negate(inner) => match &inner.kind {
            ExprKind::IntLiteral(n) => Some(AttrArgValue::Int(n.wrapping_neg())),
            ExprKind::FloatLiteral(n) => Some(AttrArgValue::Float((-n).to_bits())),
            _ => None,
        },
        ExprKind::ArrayLiteral(elements) => {
            let mut entries = Vec::with_capacity(elements.len());
            for element in elements {
                entries.push(AttrArgEntry {
                    key: None,
                    value: fold_class_const_value(element)?,
                });
            }
            Some(AttrArgValue::Array(entries))
        }
        ExprKind::ArrayLiteralAssoc(pairs) => {
            let mut entries = Vec::with_capacity(pairs.len());
            for (key_expr, value_expr) in pairs {
                let key = match &key_expr.kind {
                    ExprKind::IntLiteral(n) => AttrKey::Int(*n),
                    ExprKind::StringLiteral(s) => AttrKey::Str(s.clone()),
                    _ => return None,
                };
                entries.push(AttrArgEntry {
                    key: Some(key),
                    value: fold_class_const_value(value_expr)?,
                });
            }
            Some(AttrArgValue::Array(entries))
        }
        _ => None,
    }
}

/// Bakes all `ReflectionClass` A1 metadata slots (see
/// `ReflectionClassExtraMetadata`) into the object currently held in the ABI
/// int-result register, leaving the object pointer there on return
/// (matching `emit_reflection_string_property`'s calling convention).
fn emit_reflection_class_extra_metadata(
    ctx: &mut FunctionContext<'_>,
    reflected_class: &str,
) -> Result<()> {
    let Some(metadata) = reflection_class_extra_metadata(ctx, reflected_class) else {
        return Ok(());
    };
    let offsets = {
        let ci = ctx
            .module
            .class_infos
            .get("ReflectionClass")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionClass"))?;
        let off = |name: &str| -> Result<usize> {
            ci.property_offsets
                .get(name)
                .copied()
                .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))
        };
        (
            off("__is_abstract")?,
            off("__is_final")?,
            off("__is_interface")?,
            off("__is_internal")?,
            off("__short")?,
            off("__ancestors_lower")?,
            off("__interfaces")?,
            off("__interfaces_lower")?,
            off("__methods_lower")?,
            off("__properties")?,
            off("__const_names")?,
            off("__const_values")?,
        )
    };
    let (
        is_abstract_off,
        is_final_off,
        is_interface_off,
        is_internal_off,
        short_off,
        ancestors_off,
        interfaces_off,
        interfaces_lower_off,
        methods_off,
        properties_off,
        const_names_off,
        const_values_off,
    ) = offsets;

    emit_reflection_int_property(ctx, metadata.is_abstract as i64, is_abstract_off, is_abstract_off + 8);
    emit_reflection_int_property(ctx, metadata.is_final as i64, is_final_off, is_final_off + 8);
    emit_reflection_int_property(ctx, metadata.is_interface as i64, is_interface_off, is_interface_off + 8);
    emit_reflection_int_property(ctx, metadata.is_internal as i64, is_internal_off, is_internal_off + 8);
    emit_reflection_string_property(ctx, &metadata.short_name, short_off, short_off + 8);

    emit_reflection_replace_array_property(ctx, ancestors_off, ancestors_off + 8, |ctx| {
        super::super::builtins::attributes::emit_string_array(ctx, &metadata.ancestors_lower)
    })?;
    emit_reflection_replace_array_property(ctx, interfaces_off, interfaces_off + 8, |ctx| {
        super::super::builtins::attributes::emit_string_array(ctx, &metadata.interfaces)
    })?;
    emit_reflection_replace_array_property(ctx, interfaces_lower_off, interfaces_lower_off + 8, |ctx| {
        super::super::builtins::attributes::emit_string_array(ctx, &metadata.interfaces_lower)
    })?;
    emit_reflection_replace_array_property(ctx, methods_off, methods_off + 8, |ctx| {
        super::super::builtins::attributes::emit_string_array(ctx, &metadata.methods_lower)
    })?;
    emit_reflection_replace_array_property(ctx, properties_off, properties_off + 8, |ctx| {
        super::super::builtins::attributes::emit_string_array(ctx, &metadata.properties)
    })?;
    emit_reflection_replace_array_property(ctx, const_names_off, const_names_off + 8, |ctx| {
        super::super::builtins::attributes::emit_string_array(ctx, &metadata.const_names)
    })?;
    let const_value_entries: Vec<AttrArgEntry> = metadata
        .const_values
        .into_iter()
        .map(|value| AttrArgEntry { key: None, value })
        .collect();
    emit_reflection_replace_array_property(ctx, const_values_off, const_values_off + 8, |ctx| {
        super::super::builtins::attributes::emit_mixed_array(ctx, &const_value_entries)
    })?;
    Ok(())
}

/// Replaces an object property's default (empty-array) value with a freshly
/// built array. Assumes the target object pointer is held in the ABI
/// int-result register on entry (matching every other `emit_reflection_*`
/// baker in this file) and leaves it there on exit. `build` must leave the
/// freshly built array pointer in the ABI int-result register; runtime tag
/// `4` is the existing generic "array" marker for an object property slot
/// (matches `__attrs`/`__params`).
fn emit_reflection_replace_array_property(
    ctx: &mut FunctionContext<'_>,
    low_offset: usize,
    high_offset: usize,
    build: impl FnOnce(&mut FunctionContext<'_>) -> Result<()>,
) -> Result<()> {
    let result_reg = abi::int_result_reg(ctx.emitter);
    let object_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_load_temporary_stack_slot(ctx.emitter, object_reg, 0);
    abi::emit_load_from_address(ctx.emitter, result_reg, object_reg, low_offset);
    abi::emit_call_label(ctx.emitter, "__rt_decref_array");
    build(ctx)?;
    abi::emit_pop_reg(ctx.emitter, object_reg);
    abi::emit_store_to_address(ctx.emitter, result_reg, object_reg, low_offset);
    abi::emit_load_int_immediate(ctx.emitter, abi::secondary_scratch_reg(ctx.emitter), 4);
    abi::emit_store_to_address(
        ctx.emitter,
        abi::secondary_scratch_reg(ctx.emitter),
        object_reg,
        high_offset,
    );
    abi::emit_push_reg(ctx.emitter, object_reg);
    abi::emit_pop_reg(ctx.emitter, result_reg);
    Ok(())
}

/// Locates the `ClassMethod` AST node that declares `method_key` visible on
/// `class_name` (walking to the actual declaring class via
/// `method_declaring_classes` for inherited methods), used to bake
/// `ReflectionMethod::__modifiers` from the method's real visibility/
/// staticness/abstractness/finality.
fn find_method_decl<'a>(
    ctx: &'a FunctionContext<'_>,
    class_name: &str,
    method_key: &str,
) -> Option<&'a crate::parser::ast::ClassMethod> {
    let info = ctx.module.class_infos.get(class_name)?;
    let declaring_class = info
        .method_declaring_classes
        .get(method_key)
        .cloned()
        .unwrap_or_else(|| class_name.to_string());
    let declaring_info = ctx.module.class_infos.get(&declaring_class)?;
    declaring_info
        .method_decls
        .iter()
        .find(|decl| php_symbol_key(&decl.name) == method_key)
}

/// Returns the PHP `ReflectionMethod::IS_*` bitmask for a declared method
/// (php -n verified: `IS_STATIC=16, IS_PUBLIC=1, IS_PROTECTED=2,
/// IS_PRIVATE=4, IS_ABSTRACT=64, IS_FINAL=32`).
fn method_modifiers_bitmask(decl: &crate::parser::ast::ClassMethod) -> i64 {
    let mut bits = match decl.visibility {
        Visibility::Public => 1,
        Visibility::Protected => 2,
        Visibility::Private => 4,
    };
    if decl.is_static {
        bits |= 16;
    }
    if decl.is_abstract {
        bits |= 64;
    }
    if decl.is_final {
        bits |= 32;
    }
    bits
}

/// Bakes `ReflectionMethod::__modifiers` from the reflected method's real
/// declaration. Leaves the object pointer in the ABI int-result register
/// (matching `emit_reflection_int_property`'s calling convention); a no-op
/// (leaving the default `0` bitmask) if the method declaration cannot be
/// located, which never happens for a construction the checker accepted.
fn emit_reflection_method_modifiers(
    ctx: &mut FunctionContext<'_>,
    owner_class: &str,
    method_key: &str,
) -> Result<()> {
    let Some(decl) = find_method_decl(ctx, owner_class, method_key) else {
        return Ok(());
    };
    let bits = method_modifiers_bitmask(decl);
    let declared_name = decl.name.clone();
    let (modifiers_off, name_off) = {
        let ci = ctx
            .module
            .class_infos
            .get("ReflectionMethod")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionMethod"))?;
        let off = |name: &str| -> Result<usize> {
            ci.property_offsets
                .get(name)
                .copied()
                .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))
        };
        (off("__modifiers")?, off("__name")?)
    };
    emit_reflection_int_property(ctx, bits, modifiers_off, modifiers_off + 8);
    // PHP: `getName()` returns the method's declared name (its canonical
    // source spelling), not the (possibly differently-cased) string passed
    // to the `ReflectionMethod` constructor — `find_method_decl` resolves to
    // the actual declaring `ClassMethod`, so `decl.name` is that canonical
    // spelling. Pre-existing gap fixed here: `__name` was never baked for
    // `ReflectionMethod` before this change (only `ReflectionClass`'s
    // constructor populated the shared offset-8/16 slot), so `getName()` —
    // and this feature's `getShortName()`, which delegates to it — silently
    // returned `''` for every `ReflectionMethod` instance.
    emit_reflection_string_property(ctx, &declared_name, name_off, name_off + 8);
    Ok(())
}

/// Returns `(visibility/staticness/readonly bitmask, has_declared_type)` for
/// a property declared on `class_name`, checking both instance and static
/// property metadata (php -n verified: `IS_STATIC=16, IS_PUBLIC=1,
/// IS_PROTECTED=2, IS_PRIVATE=4, IS_READONLY=128`). `has_declared_type` comes
/// from `ClassInfo.declared_properties`/`declared_static_properties` — the
/// checker's own "does this property have an explicit source type hint" bit
/// — not from comparing the resolved `PhpType` (which an untyped property
/// still gets, inferred from its default value or `PhpType::Int` as the
/// no-default fallback).
fn property_modifiers_and_type(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property_name: &str,
) -> Option<(i64, bool)> {
    let info = ctx.module.class_infos.get(class_name)?;
    let is_static = info
        .static_properties
        .iter()
        .any(|(name, _)| name.as_str() == property_name);
    let visibility = if is_static {
        info.static_property_visibilities.get(property_name)
    } else {
        info.property_visibilities.get(property_name)
    }
    .cloned()
    .unwrap_or(Visibility::Public);
    let mut bits = match visibility {
        Visibility::Public => 1,
        Visibility::Protected => 2,
        Visibility::Private => 4,
    };
    if is_static {
        bits |= 16;
    }
    if info.readonly_properties.contains(property_name) {
        bits |= 128;
    }
    // PHP 8.4 added abstract property hooks and final properties, mirrored
    // by `ReflectionProperty::IS_ABSTRACT`/`IS_FINAL` (php -n verified: both
    // share the exact `ReflectionMethod` bit values, 64/32).
    if info.abstract_properties.contains(property_name) {
        bits |= 64;
    }
    if info.final_properties.contains(property_name)
        || info.final_static_properties.contains(property_name)
    {
        bits |= 32;
    }
    // `declared_properties`/`declared_static_properties` track exactly which
    // properties carry an EXPLICIT source type hint (see
    // `apply_static_property`/`apply_instance_property` in the checker's
    // property schema pass, which insert a name here only when
    // `resolve_property_declared_type` finds one) — an untyped property
    // still gets an INFERRED `PhpType` (from its default value, or
    // `PhpType::Int` as the no-default fallback), so comparing the resolved
    // `PhpType` against `Mixed` would wrongly report `hasType()==true` for
    // most untyped properties. This is the real signal PHP's `hasType()`
    // needs.
    let has_declared_type = if is_static {
        info.declared_static_properties.contains(property_name)
    } else {
        info.declared_properties.contains(property_name)
    };
    Some((bits, has_declared_type))
}

/// Bakes `ReflectionProperty::__modifiers`/`__has_declared_type` from the
/// reflected property's real declaration. No-op (leaving the defaults) if
/// the property cannot be located, which never happens for a construction
/// the checker accepted.
fn emit_reflection_property_modifiers(
    ctx: &mut FunctionContext<'_>,
    owner_class: &str,
    property_name: &str,
) -> Result<()> {
    let Some((bits, has_declared_type)) = property_modifiers_and_type(ctx, owner_class, property_name) else {
        return Ok(());
    };
    let (modifiers_off, has_type_off) = {
        let ci = ctx
            .module
            .class_infos
            .get("ReflectionProperty")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionProperty"))?;
        let off = |name: &str| -> Result<usize> {
            ci.property_offsets
                .get(name)
                .copied()
                .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))
        };
        (off("__modifiers")?, off("__has_declared_type")?)
    };
    emit_reflection_int_property(ctx, bits, modifiers_off, modifiers_off + 8);
    emit_reflection_int_property(ctx, has_declared_type as i64, has_type_off, has_type_off + 8);
    Ok(())
}
