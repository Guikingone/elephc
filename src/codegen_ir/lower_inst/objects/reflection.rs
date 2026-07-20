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
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;
use crate::codegen_ir::{CodegenIrError, Result};
use crate::ir::{Function, Immediate, Instruction, IrType, Module, Op, ValueDef, ValueId};
use crate::names::php_symbol_key;
use crate::parser::ast::Visibility;
use crate::types::{AttrArgEntry, AttrArgValue, AttrKey, PhpType};

use super::super::super::context::FunctionContext;
use super::super::super::frame;

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
///
/// `ReflectionClass` with a non-literal (runtime) reflected-name operand is routed to the shared
/// dynamic-name dispatcher instead (see `lower_reflection_class_new_dynamic`); every other case
/// keeps the compile-time metadata bake below unchanged.
pub(super) fn lower_reflection_owner_new(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_name: &str,
) -> Result<()> {
    if class_name == "ReflectionClass" {
        if let Some(&name_operand) = inst.operands.first() {
            if !is_const_string_or_class_value(ctx.function, name_operand) {
                return lower_reflection_class_new_dynamic(ctx, inst, name_operand);
            }
        }
    }
    if class_name == "ReflectionMethod" || class_name == "ReflectionProperty" {
        if let (Some(&class_operand), Some(&member_operand)) =
            (inst.operands.first(), inst.operands.get(1))
        {
            let dynamic = !is_const_string_or_class_value(ctx.function, class_operand)
                || !is_const_string_or_class_value(ctx.function, member_operand);
            if dynamic {
                return if class_name == "ReflectionMethod" {
                    super::reflection_members::lower_reflection_method_new_dynamic(
                        ctx,
                        inst,
                        class_operand,
                        member_operand,
                    )
                } else {
                    super::reflection_members::lower_reflection_property_new_dynamic(
                        ctx,
                        inst,
                        class_operand,
                        member_operand,
                    )
                };
            }
        }
    }
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
                emit_reflection_member_file(ctx, "ReflectionMethod", owner_class, method_key)?;
            }
        }
        "ReflectionProperty" => {
            if let (Some(owner_class), Some(property_name)) =
                (metadata.member_owner_class.as_deref(), metadata.member_name.as_deref())
            {
                emit_reflection_property_modifiers(ctx, owner_class, property_name)?;
                // NOTE: no `__file`/`getFileName()` baking here — `ReflectionProperty` has no
                // such method in real PHP (php -n verified: "Call to undefined method"); see
                // the checker shell in `crate::types::checker::builtin_types::reflection`.
            }
        }
        _ => {}
    }
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Lowers `new ReflectionFunction(...)` by populating its name and
/// parameter-count slots from the reflected function/closure's signature. The slot
/// layout is `__name` (8/16), `__short` (24/32), `__num_params` (40/48),
/// `__num_required` (56/64), `__unbacked_name` (see `reflection_function_construction_metadata`
/// offset lookup below).
pub(super) fn lower_reflection_function_new(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let (full_name, short_name, num_params, num_required, param_infos, source_file, unbacked_name) =
        reflection_function_construction_metadata(ctx, inst)?;
    let (
        class_id,
        property_count,
        uninitialized_marker_offsets,
        name_off,
        short_off,
        np_off,
        nr_off,
        file_off,
        unbacked_off,
        unbacked_file_off,
        unbacked_params_off,
        is_anonymous_off,
    ) = {
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
            slot("__file")?,
            slot("__unbacked_name")?,
            slot("__unbacked_file")?,
            slot("__unbacked_params")?,
            slot("__is_anonymous")?,
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
    emit_reflection_string_property(ctx, &source_file, file_off, file_off + 8);
    emit_reflection_int_property(ctx, unbacked_name as i64, unbacked_off, unbacked_off + 8);
    // M2 PART A: for this (static-operand) construction path, `__unbacked_file` and
    // `__is_anonymous` always mirror `__unbacked_name`'s own value — a closure LITERAL is both
    // name-unbacked AND always anonymous (`true`/`true`); a string-literal/first-class-callable
    // target is fully backed and never anonymous (`false`/`false`). `__unbacked_params` stays
    // `false`: this path always builds the real `__params` array below (compile-time-unrolled),
    // unlike the dynamic path (`reflection_function_dynamic`), which cannot cheaply build one for
    // a compile-time-unknown parameter count and gates `getParameters()` off instead.
    emit_reflection_int_property(ctx, unbacked_name as i64, unbacked_file_off, unbacked_file_off + 8);
    emit_reflection_int_property(ctx, unbacked_name as i64, is_anonymous_off, is_anonymous_off + 8);
    emit_reflection_int_property(ctx, 0, unbacked_params_off, unbacked_params_off + 8);

    // Build the `ReflectionParameter[]` array and store it into `__params`.
    let params_off = ctx
        .module
        .class_infos
        .get("ReflectionFunction")
        .and_then(|ci| ci.property_offsets.get("__params").copied())
        .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))?;
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

/// Resolves ALL metadata needed to construct a `ReflectionFunction` instance from its single
/// constructor operand. Three operand shapes reach here (checker-validated —
/// `crate::types::checker::inference::objects::constructors::
/// validate_reflection_function_constructor_arg` rejects anything else at compile time):
///
/// - A literal string constant, OR a first-class callable targeting a plain free function
///   (`target(...)`, resolved by `reflection_function_name_operand` — php -n VERIFIED PHP
///   treats an FCC-created closure's `ReflectionFunction` identically to the target's name:
///   `getName()` returns the real function name, not `"{closure}"`). Fully backed, exactly as
///   before this function existed: `full_name`/`short_name`/`source_file` are the reflected
///   function's real name/short-name/declaring-file, `unbacked_name=false`.
/// - A closure LITERAL (`closure_new_operand_name` detects the `Op::ClosureNew` operand and
///   resolves ITS OWN signature by synthetic name from `ctx.module.closures`, instead of
///   `ctx.module.functions`). The closure's identity is statically known (it's the exact value
///   this constructor call creates), so `num_params`/`num_required`/param metadata are just as
///   soundly derivable as for a named function — but `full_name`/`short_name`/`source_file`
///   stay EMPTY and `unbacked_name=true`, gating `getName`/`getShortName`/`getFileName` to
///   throw at runtime (see `crate::types::checker::builtin_types::reflection::
///   builtin_reflection_guarded_method`): PHP's real closure name embeds the declaring
///   file/function and line (php -n VERIFIED PHP 8.5 format: `"{closure:FILE:LINE}"` /
///   `"{closure:Class::method():LINE}"`), which elephc has no per-closure source-location
///   tracking to reproduce soundly — faking a value here would be silently wrong, not merely
///   incomplete.
fn reflection_function_construction_metadata(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<(String, String, i64, i64, Vec<ReflectionParamInfo>, String, bool)> {
    let Some(name_operand) = inst.operands.first().copied() else {
        return Ok((String::new(), String::new(), 0, 0, Vec::new(), String::new(), false));
    };
    if let Some(closure_name) = closure_new_operand_name(ctx, name_operand) {
        let signature = ctx
            .module
            .closures
            .iter()
            .find(|function| function.name == closure_name)
            .and_then(|function| function.signature.as_ref());
        let (num_params, num_required) = reflection_param_counts(signature);
        let param_infos = reflection_param_infos_from_signature(signature);
        return Ok((
            String::new(),
            String::new(),
            num_params,
            num_required,
            param_infos,
            String::new(),
            true,
        ));
    }
    let function_name = reflection_function_name_operand(ctx, name_operand)?;
    let key = php_symbol_key(function_name.trim_start_matches('\\'));
    let signature = ctx
        .module
        .functions
        .iter()
        .find(|function| php_symbol_key(function.name.trim_start_matches('\\')) == key)
        .and_then(|function| function.signature.as_ref());
    let (num_params, num_required) = reflection_param_counts(signature);
    let param_infos = reflection_param_infos_from_signature(signature);
    let short_name = function_name
        .trim_start_matches('\\')
        .rsplit('\\')
        .next()
        .unwrap_or(&function_name)
        .to_string();
    let source_file = ctx.module.function_source_files.get(&key).cloned().unwrap_or_default();
    Ok((
        function_name.clone(),
        short_name,
        num_params,
        num_required,
        param_infos,
        source_file,
        false,
    ))
}

/// Resolves the `new ReflectionFunction(...)` name operand to a function name string, accepting
/// either a literal string constant (the common case, `const_required_string_operand`) or a
/// first-class-callable operand targeting a free function. Closure-LITERAL operands are handled
/// by the caller (`reflection_function_construction_metadata`) before this is reached.
fn reflection_function_name_operand(ctx: &FunctionContext<'_>, value: ValueId) -> Result<String> {
    if let Some(name) = first_class_callable_operand_name(ctx, value) {
        return Ok(name);
    }
    const_required_string_operand(ctx, value, "ReflectionFunction")
}

/// Returns true when `new ReflectionFunction($operand)` can be resolved at COMPILE TIME: a
/// closure literal, a first-class callable targeting a plain free function, or a compile-time
/// constant string. `crate::codegen_ir::lower_inst::objects::lower_object_new()` routes anything
/// else (M2 PART A: a genuinely dynamic `Closure`/`callable`-typed value, or a `Mixed`/`Union`
/// value the checker accepted for the same reason — see
/// `crate::types::checker::inference::objects::constructors::
/// validate_reflection_function_constructor_arg`) to
/// `super::reflection_function_dynamic::lower_reflection_function_new_dynamic` instead. Mirrors
/// `is_const_string_or_class_value`'s role for `ReflectionClass`.
pub(super) fn is_reflection_function_static_operand(ctx: &FunctionContext<'_>, value: ValueId) -> bool {
    closure_new_operand_name(ctx, value).is_some()
        || first_class_callable_operand_name(ctx, value).is_some()
        || const_required_string_operand(ctx, value, "ReflectionFunction").is_ok()
}

/// Returns the target function's name when `value` is a `Op::FirstClassCallableNew` operand
/// (`target(...)`), or `None` for any other operand shape.
fn first_class_callable_operand_name(ctx: &FunctionContext<'_>, value: ValueId) -> Option<String> {
    reflection_data_string_for_op(ctx, value, Op::FirstClassCallableNew)
}

/// Returns the closure's own synthetic name (`ctx.module.closures`' key) when `value` is a
/// `Op::ClosureNew` operand (a closure LITERAL passed directly, e.g. `new
/// ReflectionFunction(function ($x) {...})`), or `None` for any other operand shape (including a
/// `Closure`-typed variable — that value's DEFINING instruction is whatever produced the
/// variable's value, not `Op::ClosureNew` itself, so this deliberately does not chase loads).
fn closure_new_operand_name(ctx: &FunctionContext<'_>, value: ValueId) -> Option<String> {
    reflection_data_string_for_op(ctx, value, Op::ClosureNew)
}

/// Shared lookup for `first_class_callable_operand_name`/`closure_new_operand_name`: resolves
/// `value`'s defining instruction, requires it to be exactly `expected_op`, and reads its
/// `Immediate::Data` string out of the module's data pool.
fn reflection_data_string_for_op(ctx: &FunctionContext<'_>, value: ValueId, expected_op: Op) -> Option<String> {
    let value_ref = ctx.function.value(value)?;
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return None;
    };
    let inst_ref = ctx.function.instruction(inst)?;
    if inst_ref.op != expected_op {
        return None;
    }
    let Some(Immediate::Data(data)) = inst_ref.immediate else {
        return None;
    };
    ctx.module.data.strings.get(data.as_raw() as usize).cloned()
}

/// Computes `(num_params, num_required)` from an already-resolved function/closure signature.
/// Shared core for the named-function and closure-literal `ReflectionFunction` construction
/// paths (`reflection_function_construction_metadata`).
fn reflection_param_counts(signature: Option<&crate::types::FunctionSig>) -> (i64, i64) {
    signature
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
        .unwrap_or((0, 0))
}

/// Stores an integer immediate into a Reflection object's property slot.
pub(super) fn emit_reflection_int_property(
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

/// Extracts per-parameter reflection metadata from an already-resolved function/closure
/// signature (`None` when the reflected name/closure could not be found — yields an empty
/// param list rather than erroring). A parameter is optional once a default or the variadic is
/// seen (matching PHP's `isOptional`). Shared core for the named-function and closure-literal
/// `ReflectionFunction` construction paths (`reflection_function_construction_metadata`).
fn reflection_param_infos_from_signature(
    signature: Option<&crate::types::FunctionSig>,
) -> Vec<ReflectionParamInfo> {
    let Some(signature) = signature else {
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
pub(super) fn emit_reflection_string_property(
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
pub(super) fn emit_reflection_attrs_property(
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
    "ReflectionException",
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
    /// K1 Part A: own + inherited method names, EXACT declared spelling, in real PHP
    /// `getMethods()` declaration order, with parent-private members already excluded — see
    /// `crate::codegen::runtime::data::reflect_member_registry::method_decl_order_and_names`.
    /// Bakes `ReflectionClass::__methods_ordered`; `getMethods()` loops over it.
    methods_ordered: Vec<String>,
    /// Property counterpart of `methods_ordered` — bakes `__properties_ordered`.
    properties_ordered: Vec<String>,
    const_names: Vec<String>,
    const_values: Vec<AttrArgValue>,
    /// The reflected class's declaring source file, or `None` when it is unknown (a builtin/
    /// internal class, or one `crate::pipeline::scan_reflection_source_files`'s snapshot could
    /// not attribute — see that function for why). Baked into `__file` as an empty-string
    /// sentinel when `None`; backs `getFileName()`.
    source_file: Option<String>,
    /// The reflected class's immediate parent class name, or `None` when it has no parent.
    /// Baked into `__parent_name` as an empty-string sentinel when `None`; backs
    /// `getParentClass()`.
    parent_name: Option<String>,
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

    let (_, methods_ordered) =
        crate::codegen::runtime::method_decl_order_and_names(ctx.module, class_name);
    let (_, properties_ordered) =
        crate::codegen::runtime::property_decl_order_and_names(ctx.module, class_name);

    let (const_names, const_values) = collect_reflection_class_constants(ctx, class_name);
    let source_file = ctx
        .module
        .class_source_files
        .get(&php_symbol_key(class_name.trim_start_matches('\\')))
        .cloned();
    let parent_name = info.parent.clone();

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
        methods_ordered,
        properties_ordered,
        const_names,
        const_values,
        source_file,
        parent_name,
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
            off("__methods_ordered")?,
            off("__properties_ordered")?,
            off("__const_names")?,
            off("__const_values")?,
            off("__file")?,
            off("__parent_name")?,
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
        methods_ordered_off,
        properties_ordered_off,
        const_names_off,
        const_values_off,
        file_off,
        parent_name_off,
    ) = offsets;

    emit_reflection_int_property(ctx, metadata.is_abstract as i64, is_abstract_off, is_abstract_off + 8);
    emit_reflection_int_property(ctx, metadata.is_final as i64, is_final_off, is_final_off + 8);
    emit_reflection_int_property(ctx, metadata.is_interface as i64, is_interface_off, is_interface_off + 8);
    emit_reflection_int_property(ctx, metadata.is_internal as i64, is_internal_off, is_internal_off + 8);
    emit_reflection_string_property(ctx, &metadata.short_name, short_off, short_off + 8);
    emit_reflection_string_property(
        ctx,
        metadata.source_file.as_deref().unwrap_or(""),
        file_off,
        file_off + 8,
    );
    emit_reflection_string_property(
        ctx,
        metadata.parent_name.as_deref().unwrap_or(""),
        parent_name_off,
        parent_name_off + 8,
    );

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
    emit_reflection_replace_array_property(ctx, methods_ordered_off, methods_ordered_off + 8, |ctx| {
        super::super::builtins::attributes::emit_string_array(ctx, &metadata.methods_ordered)
    })?;
    emit_reflection_replace_array_property(ctx, properties_ordered_off, properties_ordered_off + 8, |ctx| {
        super::super::builtins::attributes::emit_string_array(ctx, &metadata.properties_ordered)
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

/// Bakes `__file` for `ReflectionMethod`/`ReflectionProperty`: a member's declaring file is
/// always the file of the class that ACTUALLY DECLARES it, not necessarily the constructor's
/// `owner_class` argument (php -n verified: `(new ReflectionMethod('Dog', 'speak'))->getFileName()`
/// for a `speak()` inherited from `Animal` reports `Animal`'s file — same resolution
/// `find_method_decl` already performs for `__modifiers` via
/// `ClassInfo::method_declaring_classes`/`property_declaring_classes`). Empty string (PHP's
/// `false` sentinel — see
/// `crate::types::checker::builtin_types::reflection::empty_string_sentinel_expr`) when the
/// resolved declaring class has no known source file. Leaves the object pointer in the ABI
/// int-result register (matching every other `emit_reflection_*` baker's calling convention).
fn emit_reflection_member_file(
    ctx: &mut FunctionContext<'_>,
    class_name: &str,
    owner_class: &str,
    member_key: &str,
) -> Result<()> {
    let (file_off, declaring_class) = {
        let ci = ctx
            .module
            .class_infos
            .get(class_name)
            .ok_or_else(|| CodegenIrError::unsupported(format!("unknown class {}", class_name)))?;
        let file_off = ci
            .property_offsets
            .get("__file")
            .copied()
            .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))?;
        let owner_info = ctx.module.class_infos.get(owner_class);
        let declaring_classes_map = match class_name {
            "ReflectionMethod" => owner_info.map(|info| &info.method_declaring_classes),
            _ => owner_info.map(|info| &info.property_declaring_classes),
        };
        let declaring_class = declaring_classes_map
            .and_then(|map| map.get(member_key))
            .cloned()
            .unwrap_or_else(|| owner_class.to_string());
        (file_off, declaring_class)
    };
    let source_file = ctx
        .module
        .class_source_files
        .get(&php_symbol_key(declaring_class.trim_start_matches('\\')))
        .cloned()
        .unwrap_or_default();
    emit_reflection_string_property(ctx, &source_file, file_off, file_off + 8);
    Ok(())
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
    let (modifiers_off, has_type_off, name_off) = {
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
        (off("__modifiers")?, off("__has_declared_type")?, off("__name")?)
    };
    emit_reflection_int_property(ctx, bits, modifiers_off, modifiers_off + 8);
    emit_reflection_int_property(ctx, has_declared_type as i64, has_type_off, has_type_off + 8);
    // PHP: `getName()` returns the property's declared name — property names are
    // case-SENSITIVE, so the exact `property_name` the constructor was called with (which the
    // checker already validated exists, exact case, on `owner_class`) IS the declared spelling.
    // Pre-existing gap fixed here (found via the J4 dynamic-construction test suite):
    // `__name` was never baked for `ReflectionProperty` before this change (only
    // `ReflectionMethod`'s literal path fixed the equivalent gap earlier), so
    // `getName()` silently returned `''` for every literally-constructed `ReflectionProperty`.
    emit_reflection_string_property(ctx, property_name, name_off, name_off + 8);
    Ok(())
}

// ============================================================================================
// Dynamic-name `new ReflectionClass($runtimeName)` construction.
//
// The compile-time metadata bake above requires a literal/`Foo::class` reflected name. The
// checker (`crate::types::checker::inference::objects::constructors::reflection_class_literal_arg`)
// now also accepts a non-literal `string`-typed argument for `ReflectionClass` specifically; this
// section routes that case through ONE shared, program-wide dispatch function emitted once (not
// per call site, see `emit_reflection_class_dynamic_dispatch_if_needed`). It PHP-case-folds the
// runtime name (`php_symbol_key`-style) and strips one leading namespace-root backslash, then
// compares it against every closed-world class name; on a match it performs EXACTLY the same
// allocation/metadata bake as the literal path above (`emit_object_allocation` +
// `emit_reflection_string_property` + `emit_reflection_attrs_property` +
// `emit_reflection_class_extra_metadata`); on no match it throws a real, catchable
// `\ReflectionException` — mirroring how `__rt_constant` throws `\Error` on a registry miss
// (`crate::codegen::runtime::system::rt_constant`).
// ============================================================================================

/// Assembly label of the shared, program-wide dynamic `ReflectionClass(name)` dispatcher.
const DYNAMIC_CLASS_DISPATCH_LABEL: &str = "_elephc_reflect_class_new_dynamic";

/// Returns true when an EIR value is a compile-time-constant string or class-name literal
/// (an `Op::ConstStr` or `Op::ConstClassName` instruction) — the shape the literal metadata bake
/// above requires. Shared by the per-call-site dispatch decision and the module-wide scan that
/// decides whether the dynamic dispatcher needs to be emitted at all.
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

/// Lowers `new ReflectionClass($runtimeName)` for a non-literal reflected-name operand.
///
/// PHP's real constructor signature is `__construct(object|string $objectOrClass)` — php -n
/// verified `new ReflectionClass($obj)` is legal PHP, reflecting `$obj`'s own runtime class.
/// `crate::types::checker::inference::objects::constructors::reflection_class_literal_arg`
/// routes ANY non-literal argument here regardless of its static type, so this function performs
/// the actual runtime type determination:
/// - `Str`: materializes the pointer/length pair directly (existing behavior).
/// - `Object(_)`: the value is already an unboxed object pointer; resolve ITS concrete runtime
///   class name (not the static type name — PHP dispatches on the ACTUAL object, which may be a
///   subclass) via the same lookup `get_class()` uses.
/// - `Mixed`/`Union(_)`: unbox the runtime tag first, then take the `Str` or object path above
///   for tag 1 (string) or 6 (object); any OTHER tag is not `object|string` and throws a real,
///   catchable `\TypeError` (php -n verified: `new ReflectionClass(42)` throws `TypeError:
///   ReflectionClass::__construct(): Argument #1 ($objectOrClass) must be of type object|string,
///   int given` — this implementation's message omits the "int given" runtime-type-name suffix,
///   a scoped simplification; the class/method identification and catchability match PHP).
///
/// Every path that determines a class name funnels into `DYNAMIC_CLASS_DISPATCH_LABEL` via the
/// SAME shared dispatcher call convention (name pointer/length in the first two integer argument
/// registers, matching `crate::codegen::runtime::system::rt_constant`'s `x0/rdi`+`x1/rsi`
/// convention). That label never returns on a miss (it throws); on a match it leaves the
/// constructed object pointer in the ABI integer result register, exactly like the
/// literal-argument path's `ctx.store_result_value(result)` expects.
fn lower_reflection_class_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name_operand: ValueId,
) -> Result<()> {
    match ctx.value_php_type(name_operand)? {
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
            emit_dispatch_call_from_string_result_regs(ctx)?;
        }
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_result(name_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            emit_reflection_class_dynamic_dispatch_from_mixed_tag(ctx)?;
        }
        // A STATICALLY int/float/bool/void-typed argument (e.g. `$name = 42; new
        // ReflectionClass($name);`) is not boxed as Mixed — it is a raw scalar in the ABI
        // result register(s) — but PHP's runtime weak-coercion rule (see the doc comment above)
        // applies identically regardless of whether the checker knew the concrete type at
        // compile time or only a boxed Mixed tag at runtime, so these get the SAME `(string)`
        // cast + dispatch treatment inline, without any unboxing step.
        PhpType::Int => {
            ctx.load_value_to_result(name_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_itoa");
            emit_dispatch_call_from_string_result_regs(ctx)?;
        }
        PhpType::Float => {
            ctx.load_value_to_result(name_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_ftoa");
            emit_dispatch_call_from_string_result_regs(ctx)?;
        }
        PhpType::Bool => {
            ctx.load_value_to_result(name_operand)?;
            emit_reflection_class_loaded_bool_to_string(ctx);
            emit_dispatch_call_from_string_result_regs(ctx)?;
        }
        // A statically `void`/`never`-typed argument only arises from a degenerate expression
        // (e.g. the result of a `never`-returning call); treated the same as PHP's `null`
        // weak-coercion (→ "") for consistency with the Mixed-tag null case above.
        PhpType::Void | PhpType::Never => {
            let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
            abi::emit_load_int_immediate(ctx.emitter, ptr_reg, 0);
            abi::emit_load_int_immediate(ctx.emitter, len_reg, 0);
            emit_dispatch_call_from_string_result_regs(ctx)?;
        }
        // Any other statically-known type (array, resource, callable, …) is genuinely NOT
        // coercible to `object|string` in real PHP either (php -n verified for array/resource —
        // see the doc comment above) — throw the same catchable `\TypeError` rather than crash
        // the compiler on an "unsupported" internal error.
        _ => {
            ctx.load_value_to_result(name_operand)?;
            emit_reflection_class_argument_type_error_throw(ctx)?;
        }
    }
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Moves a (pointer, length) pair from the standard string-RESULT register convention
/// (`abi::string_result_regs`: `x1`/`x2` on AArch64, `rax`/`rdx` on x86_64 — what
/// `emit_dynamic_object_class_name` produces) into the dispatcher's ARGUMENT register
/// convention (`x0`/`x1` on AArch64, `rdi`/`rsi` on x86_64), then calls
/// `DYNAMIC_CLASS_DISPATCH_LABEL`.
fn emit_dispatch_call_from_string_result_regs(ctx: &mut FunctionContext<'_>) -> Result<()> {
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
    Ok(())
}

/// Weak-casts an ALREADY-LOADED (not boxed/unboxed — a raw scalar in the ABI int-result
/// register) bool value to `"1"` (true) or `""` (false), leaving the result in
/// `abi::string_result_regs`. Mirrors `crate::codegen_ir::lower_inst::strings::
/// lower_loaded_bool_to_string`'s exact logic (NOT re-exported across the `objects` module
/// boundary, so duplicated here rather than widening its visibility for one caller).
fn emit_reflection_class_loaded_bool_to_string(ctx: &mut FunctionContext<'_>) {
    let false_label = ctx.next_label("reflect_dyn_bool_str_false");
    let done_label = ctx.next_label("reflect_dyn_bool_str_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("cbz x0, {}", false_label));                 // false weak-casts to an empty string
            abi::emit_call_label(ctx.emitter, "__rt_itoa");                        // true weak-casts to "1" via decimal text
            ctx.emitter.instruction(&format!("b {}", done_label));              // skip the empty-string fallback after true conversion
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("mov x2, #0");                              // false has zero string length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // test whether the loaded bool payload is false
            ctx.emitter
                .instruction(&format!("je {}", false_label));                      // false weak-casts to an empty string
            abi::emit_call_label(ctx.emitter, "__rt_itoa");                        // true weak-casts to "1" via decimal text
            ctx.emitter.instruction(&format!("jmp {}", done_label));            // skip the empty-string fallback after true conversion
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("mov rdx, 0");                              // false has zero string length
        }
    }
    ctx.emitter.label(&done_label);
}

/// After `__rt_mixed_unbox` has run (tag in `x0`/`rax`, primary payload in `x1`/`rdi`, string
/// length in `x2`/`rdx` — see `crate::codegen_ir::block_emit`'s tagged-scalar unbox comment),
/// branches on the runtime tag: `1` (string) materializes the payload directly into the
/// dispatcher's argument registers; `6` (object) resolves the object's concrete runtime class
/// name first; any other tag throws a catchable `\TypeError` — `new ReflectionClass($x)` never
/// proceeds into the dispatcher with a non-`object|string` payload.
///
/// php -n VERIFIED runtime tag disposition (`new ReflectionClass($x)` for every scalar/compound
/// kind — this is PHP's real weak-typing union coercion for `object|string`, NOT a guess):
/// - tag 1 (string), tag 6 (object): accepted directly (existing behavior below).
/// - tag 0 (int), tag 2 (float), tag 3 (bool), tag 8 (null): PHP WEAK-COERCES the scalar to a
///   string (`new ReflectionClass(42)` → `ReflectionException: Class "42" does not exist`;
///   `new ReflectionClass(true)` → `Class "1" does not exist`; `new ReflectionClass(4.2)` →
///   `Class "4.2" does not exist`; `new ReflectionClass(null)` → `Class "" does not exist`, plus
///   a deprecation notice this implementation does not emit) — coerced the SAME way
///   `(string)$x` casts, then routed into the dispatcher exactly like a real string argument.
/// - tag 4 (array), tag 9 (resource), and any other/unknown tag: genuinely NOT coercible —
///   `new ReflectionClass([1,2])` / `new ReflectionClass($resource)` both throw a real
///   `TypeError` in PHP (verified), never a `ReflectionException`.
fn emit_reflection_class_dynamic_dispatch_from_mixed_tag(ctx: &mut FunctionContext<'_>) -> Result<()> {
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
            ctx.emitter.instruction(&format!("b.eq {}", null_label));           // PHP weak-coerces null to "" for this constructor (deprecated but accepted)
            ctx.emitter.instruction(&format!("b {}", type_error_label));        // array/resource/other: not coercible to `object|string`
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
            ctx.emitter.instruction(&format!("je {}", null_label));             // PHP weak-coerces null to "" for this constructor (deprecated but accepted)
            ctx.emitter.instruction(&format!("jmp {}", type_error_label));      // array/resource/other: not coercible to `object|string`
        }
    }

    // -- tag 1 (string): __rt_mixed_unbox already left (ptr, len) in x1/x2 (AArch64) or
    //    rdi/rdx (x86_64) — move into the dispatcher's own argument convention and call it. --
    ctx.emitter.label(&str_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // move the unboxed string pointer into the dispatcher's arg0
            ctx.emitter.instruction("mov x1, x2");                              // move the unboxed string length into the dispatcher's arg1
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rdx");                            // move the unboxed string length into the dispatcher's arg1 first (rdi already holds arg0)
        }
    }
    abi::emit_call_label(ctx.emitter, DYNAMIC_CLASS_DISPATCH_LABEL);
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 6 (object): __rt_mixed_unbox left the object pointer in x1 (AArch64) / rdi
    //    (x86_64) — resolve its concrete runtime class name, then dispatch on that name. --
    ctx.emitter.label(&object_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction("mov x0, x1"),                 // move the unboxed object pointer into the class-name lookup register
        Arch::X86_64 => ctx.emitter.instruction("mov rax, rdi"),                // move the unboxed object pointer into the class-name lookup register
    }
    super::super::builtins::types::emit_dynamic_object_class_name(ctx, "get_class");
    emit_dispatch_call_from_string_result_regs(ctx)?;
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 0 (int): weak-cast the unboxed int payload to decimal text via `__rt_itoa`
    //    (mirrors `__rt_mixed_cast_string`'s int branch), then dispatch on the cast string. --
    ctx.emitter.label(&int_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // move the unboxed int payload into __rt_itoa's argument register
            abi::emit_call_label(ctx.emitter, "__rt_itoa");                        // convert the int payload to decimal text → x1=ptr, x2=len
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, rdi");                            // move the unboxed int payload into __rt_itoa's argument register
            abi::emit_call_label(ctx.emitter, "__rt_itoa");                        // convert the int payload to decimal text → rax=ptr, rdx=len
        }
    }
    emit_dispatch_call_from_string_result_regs(ctx)?;
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 2 (float): weak-cast the unboxed float bit-pattern payload to decimal text via
    //    `__rt_ftoa` (mirrors `__rt_mixed_cast_string`'s float branch). --
    ctx.emitter.label(&float_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("fmov d0, x1");                             // move the unboxed float bit-pattern payload into the FP argument register
            abi::emit_call_label(ctx.emitter, "__rt_ftoa");                        // convert the float payload to decimal text → x1=ptr, x2=len
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("movq xmm0, rdi");                          // move the unboxed float bit-pattern payload into the FP argument register
            abi::emit_call_label(ctx.emitter, "__rt_ftoa");                        // convert the float payload to decimal text → rax=ptr, rdx=len
        }
    }
    emit_dispatch_call_from_string_result_regs(ctx)?;
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 3 (bool): weak-cast to "1" (true) or "" (false) — mirrors
    //    `__rt_mixed_cast_string`'s bool branch exactly (NOT itoa(0), which would wrongly
    //    produce "0" for false; PHP casts `false` to the empty string). --
    ctx.emitter.label(&bool_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("cbz x1, {}", bool_false_label));            // false skips straight to the empty-string result
            ctx.emitter.instruction("mov x0, x1");                              // move the true payload (1) into __rt_itoa's argument register
            abi::emit_call_label(ctx.emitter, "__rt_itoa");                        // convert true to the string "1" → x1=ptr, x2=len
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rdi, rdi");                           // false skips straight to the empty-string result
            ctx.emitter
                .instruction(&format!("je {}", bool_false_label));
            ctx.emitter.instruction("mov rax, rdi");                            // move the true payload (1) into __rt_itoa's argument register
            abi::emit_call_label(ctx.emitter, "__rt_itoa");                        // convert true to the string "1" → rax=ptr, rdx=len
        }
    }
    emit_dispatch_call_from_string_result_regs(ctx)?;
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
    emit_dispatch_call_from_string_result_regs(ctx)?;
    abi::emit_jump(ctx.emitter, &done_label);

    // -- tag 8 (null): weak-casts to the empty string (php -n verified, deprecation notice
    //    aside — see the doc comment above). --
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
    emit_dispatch_call_from_string_result_regs(ctx)?;
    abi::emit_jump(ctx.emitter, &done_label);

    // -- any other tag (array, resource, …): not coercible to `object|string` — throw, never
    //    returns. --
    ctx.emitter.label(&type_error_label);
    emit_reflection_class_argument_type_error_throw(ctx)?;

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Throws a catchable `\TypeError` for a dynamic `ReflectionClass($x)` construction whose
/// runtime argument is neither a string nor an object. Mirrors
/// `emit_reflection_class_not_found_throw`'s allocation/throw sequence exactly, but stamps
/// `_spl_type_error_class_id` (the shared, unconditionally-emitted `TypeError` runtime class id
/// — see `crate::codegen::runtime::data::user`) instead of `_reflection_exception_class_id`, and
/// a fixed message (php -n verified core wording; the concrete "X given" runtime-type-name
/// suffix real PHP appends is a scoped, documented simplification — see
/// `lower_reflection_class_new_dynamic`). Never returns.
fn emit_reflection_class_argument_type_error_throw(ctx: &mut FunctionContext<'_>) -> Result<()> {
    let (message_label, message_len) = ctx.data.add_string(
        b"ReflectionClass::__construct(): Argument #1 ($objectOrClass) must be of type object|string",
    );
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &message_label);       // message pointer
            ctx.emitter
                .instruction(&format!("mov x2, #{}", message_len));            // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");             // own a heap copy of the message bytes
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");                      // park the owned message across the allocation call
            ctx.emitter.instruction("mov x0, #32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");                  // allocate the TypeError object payload
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("str x9, [x0]");                            // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "x9", "x10");                      // reload the owned message pointer/length
            ctx.emitter.instruction("str x9, [x0, #8]");                        // store the exception message pointer
            ctx.emitter.instruction("str x10, [x0, #16]");                      // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0);     // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                     // enter the standard exception unwinder
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rdi", &message_label);          // message pointer
            ctx.emitter
                .instruction(&format!("mov rsi, {}", message_len));            // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");                 // own a heap copy of the message (ptr in rax, len carried in rdx)
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                    // park the owned message across the allocation call
            ctx.emitter.instruction("mov rax, 32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");                  // allocate the TypeError object payload
            ctx.emitter.instruction("mov r10, 0x4548504c00000006");             // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "r10", "r11");                     // reload the owned message pointer/length
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10");            // store the exception message pointer
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r11");           // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0);    // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                     // enter the standard exception unwinder
        }
    }
    Ok(())
}

/// Iterates every function-like body lowered into the EIR module.
///
/// Mirrors `crate::ir_lower::program::all_lowered_functions`; duplicated here (rather than
/// exported from there) because that helper is private to the lowering crate and this scan runs
/// once from the codegen backend after lowering has fully completed, not during it.
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

/// Returns the class name an `Op::ObjectNew` instruction constructs, or `None` for a malformed
/// instruction (never happens for a module that passed EIR validation).
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

/// Returns true when the module contains at least one `new ReflectionClass($runtimeName)` site
/// with a non-literal reflected-name operand — i.e. whether the shared dynamic dispatcher needs
/// to be emitted at all. Emitting it unconditionally would add roughly one construction branch
/// per closed-world class to every compiled program, regardless of whether it uses this feature.
fn module_needs_reflection_class_dynamic_dispatch(module: &Module) -> bool {
    reflection_dispatch_scan_functions(module).any(|function| {
        function.instructions.iter().any(|inst| {
            inst.op == Op::ObjectNew
                && object_new_class_name(module, inst) == Some("ReflectionClass")
                && inst
                    .operands
                    .first()
                    .is_some_and(|&value| !is_const_string_or_class_value(function, value))
        })
    })
}

/// Emits the shared, program-wide dynamic `ReflectionClass(name)` construction dispatcher, once,
/// if and only if the module actually contains a dynamic-name call site (see
/// `module_needs_reflection_class_dynamic_dispatch`). No-op otherwise.
///
/// Called once, after all per-function EIR lowering has completed, from
/// `crate::codegen_ir::generate_user_asm_from_ir_with_options`.
pub(crate) fn emit_reflection_class_dynamic_dispatch_if_needed(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
) -> Result<()> {
    if !module_needs_reflection_class_dynamic_dispatch(module) {
        return Ok(());
    }
    emit_reflection_class_dynamic_dispatch(module, emitter, data)
}

/// Builds and emits the dispatcher body.
///
/// Constructs a throwaway, valid-but-empty synthetic `ir::Function` purely so the existing
/// metadata bakers above (`emit_object_allocation`, `emit_reflection_string_property`,
/// `emit_reflection_attrs_property`, `emit_reflection_class_extra_metadata`, …) can be reused
/// unchanged: none of them read any per-real-function frame/value-placement state, only
/// `ctx.emitter`/`ctx.data`/`ctx.module`, so a trivial empty `Function` and its (equally trivial)
/// `FrameLayout` are a fully valid `FunctionContext` for this purpose. Every dispatch branch below
/// performs EXACTLY what `lower_reflection_owner_new`'s `"ReflectionClass"` literal-argument path
/// does for the same class.
fn emit_reflection_class_dynamic_dispatch(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
) -> Result<()> {
    let mut class_names: Vec<&String> = module.class_infos.keys().collect();
    class_names.sort();

    let target = emitter.target;
    let synthetic = Function::new(
        format!("{}_impl", DYNAMIC_CLASS_DISPATCH_LABEL),
        IrType::Void,
        PhpType::Void,
    );
    let layout = frame::layout_for_function(&synthetic, target, false);
    let mut ctx = FunctionContext::new(module, &synthetic, emitter, data, layout, false, false, false, None);

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
        // Drop the two parked query pairs (normalized + original, 16 bytes each) — a match no
        // longer needs them, and construction below assumes the same clean stack the literal
        // path starts from right after the prologue.
        abi::emit_release_temporary_stack(ctx.emitter, 32);
        emit_reflection_class_dynamic_construct(&mut ctx, name)?;
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&not_found_label);
    emit_reflection_class_not_found_throw(&mut ctx)?;

    ctx.emitter.label(&done_label);
    emit_dynamic_dispatch_epilogue(&mut ctx);
    Ok(())
}

/// Emits the leaf-function prologue for the shared dynamic dispatcher: a plain frame-pointer
/// save/establish, matching the hand-written runtime helpers this dispatcher is modeled after
/// (e.g. `crate::codegen::runtime::system::rt_class_exists`).
fn emit_dynamic_dispatch_prologue(ctx: &mut FunctionContext<'_>) {
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
/// integer result register by the matched dispatch branch (`emit_reflection_class_dynamic_construct`
/// leaves it there, mirroring every `emit_reflection_*` baker's calling convention).
fn emit_dynamic_dispatch_epilogue(ctx: &mut FunctionContext<'_>) {
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

/// Parks the caller's ORIGINAL `(name_ptr, name_len)` pair on the temporary stack (ends up at
/// offset 16/24 once the normalized copy below is parked on top of it) — kept byte-for-byte, in
/// case the query matches nothing and the exception message needs to echo it back exactly as PHP
/// does (php -n verified: `Class "NAME" does not exist`, where NAME is the UNMODIFIED argument the
/// caller passed, backslash and case included) — then computes a leading-backslash-stripped,
/// PHP-case-folded WORKING copy for the compare chain, parked at offset 0/8 (PHP class names are
/// case-insensitive; `super::emit_branch_if_dynamic_name_matches` reads its query from exactly
/// these two offsets).
fn emit_dynamic_dispatch_query_normalization(ctx: &mut FunctionContext<'_>) {
    let skip_label = ctx.next_label("reflect_dyn_skip_bs");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x0", "x1");                       // park the ORIGINAL query for the not-found exception message
            ctx.emitter
                .instruction(&format!("cbz x1, {}", skip_label));                   // an empty query cannot start with a leading backslash
            ctx.emitter.instruction("ldrb w9, [x0]");                           // peek at the query's first byte
            ctx.emitter.instruction("cmp w9, #0x5c");                           // is it a leading namespace-root backslash?
            ctx.emitter
                .instruction(&format!("b.ne {}", skip_label));                      // no backslash to strip
            ctx.emitter.instruction("add x0, x0, #1");                          // strip the leading backslash from the working pointer
            ctx.emitter.instruction("sub x1, x1, #1");                          // and from the working length
            ctx.emitter.label(&skip_label);
            ctx.emitter.instruction("mov x2, x1");                              // __rt_strtolower expects the length in x2
            ctx.emitter.instruction("mov x1, x0");                              // __rt_strtolower expects the pointer in x1
            abi::emit_call_label(ctx.emitter, "__rt_strtolower");
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");                       // park the case-folded working query for the compare chain
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rdi", "rsi");                     // park the ORIGINAL query for the not-found exception message
            ctx.emitter.instruction("test rsi, rsi");                           // an empty query cannot start with a leading backslash
            ctx.emitter
                .instruction(&format!("jz {}", skip_label));
            ctx.emitter.instruction("movzx r9d, BYTE PTR [rdi]");               // peek at the query's first byte
            ctx.emitter.instruction("cmp r9b, 0x5c");                           // is it a leading namespace-root backslash?
            ctx.emitter
                .instruction(&format!("jne {}", skip_label));                       // no backslash to strip
            ctx.emitter.instruction("add rdi, 1");                              // strip the leading backslash from the working pointer
            ctx.emitter.instruction("sub rsi, 1");                              // and from the working length
            ctx.emitter.label(&skip_label);
            ctx.emitter.instruction("mov rax, rdi");                            // __rt_strtolower expects the pointer in rax
            ctx.emitter.instruction("mov rdx, rsi");                            // and the length in rdx
            abi::emit_call_label(ctx.emitter, "__rt_strtolower");
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                     // park the case-folded working query for the compare chain
        }
    }
}

/// Bakes one class's `ReflectionClass` construction (allocation + `__name` + `__attrs` + the A1
/// closed-world metadata slots) into the object left in the ABI integer result register — the
/// exact same sequence `lower_reflection_owner_new`'s literal-argument `"ReflectionClass"` path
/// runs, just driven by a Rust-loop-known `reflected_class_name` instead of a resolved EIR
/// operand.
///
/// `reflected_class_name` is the CLASS BEING REFLECTED (e.g. `"ElephcDynDog"`), never the
/// allocated object's own class. The allocated object is always a `ReflectionClass` SHELL
/// instance — its class id/property count/marker offsets come from the `"ReflectionClass"` shell
/// itself, exactly like the literal path (`lower_reflection_owner_new`'s `class_name` parameter is
/// always `"ReflectionClass"` there too); `reflected_class_name` is only used to look up the
/// VALUES baked into that shell's slots (`__name`, `__attrs`, the A1 metadata fields).
fn emit_reflection_class_dynamic_construct(
    ctx: &mut FunctionContext<'_>,
    reflected_class_name: &str,
) -> Result<()> {
    let (class_id, property_count, uninitialized_marker_offsets) = {
        let class_info = ctx
            .module
            .class_infos
            .get("ReflectionClass")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionClass"))?;
        (
            class_info.class_id,
            class_info.properties.len(),
            super::uninitialized_property_marker_offsets(class_info),
        )
    };
    let (attr_names, attr_args) = {
        let reflected_info = ctx.module.class_infos.get(reflected_class_name).ok_or_else(|| {
            CodegenIrError::unsupported(format!("unknown class {}", reflected_class_name))
        })?;
        (
            reflected_info.attribute_names.clone(),
            reflected_info.attribute_args.clone(),
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
    emit_reflection_string_property(ctx, reflected_class_name, 8, 16);
    emit_reflection_attrs_property(ctx, "ReflectionClass", &attr_names, &attr_args)?;
    emit_reflection_class_extra_metadata(ctx, reflected_class_name)?;
    Ok(())
}

/// Throws a catchable `\ReflectionException` for a dynamic `ReflectionClass(name)` construction
/// whose case-folded query matched no closed-world class.
///
/// Builds PHP's exact message (php -n verified: `Class "NAME" does not exist`, NAME = the
/// ORIGINAL, unmodified query the caller passed — reloaded from the temporary stack slot
/// `emit_dynamic_dispatch_query_normalization` parked it at) via `__rt_concat`/`__rt_str_persist`,
/// then throws through the same mechanism `crate::codegen::runtime::system::rt_constant` uses for
/// a `constant()` registry miss: allocate the compact Throwable payload, stamp
/// `_reflection_exception_class_id`, publish it to `_exc_value`, and enter `__rt_throw_current`.
/// Never returns.
fn emit_reflection_class_not_found_throw(ctx: &mut FunctionContext<'_>) -> Result<()> {
    let (prefix_label, prefix_len) = ctx.data.add_string(b"Class \"");
    let (suffix_label, suffix_len) = ctx.data.add_string(b"\" does not exist");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &prefix_label);             // message prefix pointer
            ctx.emitter
                .instruction(&format!("mov x2, #{}", prefix_len));                  // message prefix byte length
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x3", 16);             // original query pointer (parked before normalization)
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x4", 24);             // original query byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");                       // prefix concatenated with the original query
            abi::emit_symbol_address(ctx.emitter, "x3", &suffix_label);             // message suffix pointer
            ctx.emitter
                .instruction(&format!("mov x4, #{}", suffix_len));                  // message suffix byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");                       // append the closing quote and suffix
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");                  // own a heap copy of the message bytes
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");                       // park the owned message across the allocation call
            ctx.emitter.instruction("mov x0, #32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");                   // allocate the ReflectionException object payload
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_reflection_exception_class_id", 0); // load ReflectionException's runtime class id
            ctx.emitter.instruction("str x9, [x0]");                            // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "x9", "x10");                       // reload the owned message pointer/length
            ctx.emitter.instruction("str x9, [x0, #8]");                        // store the exception message pointer
            ctx.emitter.instruction("str x10, [x0, #16]");                      // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0);      // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                      // enter the standard exception unwinder
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rax", &prefix_label);            // message prefix pointer
            ctx.emitter
                .instruction(&format!("mov rdx, {}", prefix_len));                  // message prefix byte length
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", 16);            // original query pointer (parked before normalization)
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", 24);            // original query byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");                       // prefix concatenated with the original query
            abi::emit_symbol_address(ctx.emitter, "rdi", &suffix_label);            // message suffix pointer
            ctx.emitter
                .instruction(&format!("mov rsi, {}", suffix_len));                  // message suffix byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");                       // append the closing quote and suffix
            ctx.emitter.instruction("mov rdi, rax");                            // move the message pointer into the persist argument
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");                  // own a heap copy of the message (ptr in rax, len carried in rdx)
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                     // park the owned message across the allocation call
            ctx.emitter.instruction("mov rax, 32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");                   // allocate the ReflectionException object payload
            ctx.emitter.instruction("mov r10, 0x4548504c00000006");             // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_reflection_exception_class_id", 0); // load ReflectionException's runtime class id
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "r10", "r11");                      // reload the owned message pointer/length
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10");            // store the exception message pointer
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r11");           // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0);     // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                      // enter the standard exception unwinder
        }
    }
    Ok(())
}
