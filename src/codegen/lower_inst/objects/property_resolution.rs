//! Purpose:
//! Resolves declared property slots and receiver source types.
//!
//! Called from:
//! - The object lowering facade and sibling object support modules.
//!
//! Key details:
//! - Case-insensitive class lookup and packed/runtime storage overrides remain authoritative.
//! - A by-name dispatch asks `crate::types::resolve_property_name` first: the physical slot table
//!   still carries a strict ancestor's private slot under its plain name, which php resolves to a
//!   DYNAMIC property everywhere except inside the class that declared it.

use super::*;

use crate::types::PropertyNameResolution;

/// Allocates an object-owned ref cell before a physical initializer writes its default.
pub(super) fn initialize_owned_property_reference(
    ctx: &mut FunctionContext<'_>,
    slot: &PropertySlot,
    base_reg: &str,
) -> bool {
    let owns_cell = slot.is_reference && ctx.module.class_infos.get(&slot.class_name)
        .is_some_and(|class| class.owned_reference_properties.contains(&slot.property));
    if owns_cell {
        emit_owned_reference_property_cell(ctx, base_reg, slot.offset, &slot.php_type);
    }
    owns_cell
}

/// Resolves a physical initializer slot without applying name-based shadow selection.
pub(super) fn resolve_initializer_property_slot(
    ctx: &FunctionContext<'_>,
    object: ValueId,
    class_id: u32,
    index: u32,
    inst: &Instruction,
) -> Result<PropertySlot> {
    let PhpType::Object(class_name) = ctx.value_php_type(object)?.codegen_repr() else {
        return Err(CodegenIrError::invalid_module("property initializer needs a concrete object"));
    };
    let info = ctx.module.class_infos.get(class_name.trim_start_matches('\\'))
        .ok_or_else(|| CodegenIrError::unsupported(format!("unknown initializer class {class_name}")))?;
    if info.class_id != u64::from(class_id)
        || !ctx.function.flags.is_synthetic
        || ctx.function.name != format!("_class_propinit_{class_id}")
    {
        return Err(CodegenIrError::invalid_module("physical property reference outside its initializer"));
    }
    let index = index as usize;
    let (property, php_type) = info.properties.get(index)
        .ok_or_else(|| CodegenIrError::invalid_module("property initializer index is outside the class layout"))?;
    ensure_property_type_supported(php_type, inst)?;
    Ok(PropertySlot {
        class_name,
        property: property.clone(),
        php_type: php_type.clone(),
        offset: 8 + index * 16,
        is_declared: info.property_slot_is_declared(index, property),
        is_packed: false,
        is_reference: info.property_slot_is_reference(index, property),
    })
}

/// Resolves the property slot for a concrete object receiver and declared property name.
pub(super) fn resolve_property_slot(
    ctx: &FunctionContext<'_>,
    object: crate::ir::ValueId,
    property: &str,
    inst: &Instruction,
) -> Result<PropertySlot> {
    let object_ty = ctx.value_php_type(object)?;
    let PhpType::Object(class_name) = object_ty else {
        if let PhpType::Packed(class_name) = object_ty {
            return resolve_packed_field_slot(ctx, &class_name, property, inst);
        }
        return Err(CodegenIrError::unsupported(format!(
            "{} for receiver PHP type {:?}",
            inst.op.name(),
            object_ty
        )));
    };
    resolve_property_slot_for_class(ctx, &class_name, property, inst)
}

/// Returns the dynamic-property hash slot offset for an undeclared allow-dynamic property.
pub(super) fn dynamic_property_hash_offset_for_object(
    ctx: &FunctionContext<'_>,
    object: crate::ir::ValueId,
    property: &str,
) -> Result<Option<usize>> {
    let object_ty = ctx.value_php_type(object)?;
    let PhpType::Object(class_name) = object_ty else {
        return Ok(None);
    };
    dynamic_property_hash_offset_for_class(ctx, &class_name, property)
}

/// Resolves what php does with one property NAME on `class_name` from the body's LEXICAL scope.
///
/// `Function::lexical_class` is the php scope the frame executes in: the declaring class for a
/// method, the declaring class for a closure written inside one, the planned invocation scope for
/// a `clone()` override applicator (`crate::ir_lower::clone_overrides`), and `None` for global
/// scope. It is the same source `clone_hook_is_visible` uses for `__clone` visibility.
///
/// Every by-name ladder over `ClassInfo::properties` has to consult this before it matches a
/// runtime name against a slot: the physical layout still carries a strict ancestor's private
/// slot under its plain name, and php resolves that name to a DYNAMIC property everywhere except
/// inside the class that declared it. See `crate::types::resolve_property_name`.
pub(super) fn resolve_property_name_in_current_scope(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
) -> PropertyNameResolution {
    crate::types::resolve_property_name(
        &ctx.module.class_infos,
        class_name,
        property,
        ctx.function.lexical_class.as_deref(),
    )
}

/// One arm of a runtime-name by-name dispatch ladder.
pub(super) enum PropertyNameArm {
    /// The name addresses this slot.
    Slot(PropertySlot),
    /// php refuses the access from this scope: raise the catchable `Error` carrying this message.
    Refuse {
        /// Property name the runtime string must equal for this arm to run.
        property: String,
        /// php 8.5's verbatim wording, e.g. `Cannot access private property D::$n`.
        message: String,
    },
    /// php resolves the name to a DYNAMIC property here, so it answers from the instance hash.
    ///
    /// Only a READ builds this arm. A strict ancestor's private slot still occupies the physical
    /// layout under this plain name, so the arm exists to keep the name away from that slot and
    /// send it to the per-instance hash, or to php `null` when the class reserves no hash.
    ScopeDynamic {
        /// Property name the runtime string must equal for this arm to run.
        property: String,
    },
    /// php would answer this name from `__get` or `__isset`, which is not dispatchable yet.
    ///
    /// The class declares the accessor php consults BEFORE it reports anything, so neither the
    /// access `Error` nor the `Undefined property` warning may be emitted here: php reports
    /// neither. This compiler cannot call the accessor with a runtime name yet, so the arm answers
    /// php `null` and, above all, never reads the slot. The slot holds private storage this scope
    /// may not see, and handing it back would be a storage escape dressed up as a value.
    ///
    /// The php-correct value arrives with the dedicated runtime-name magic dispatch phase.
    MagicDeferred {
        /// Property name the runtime string must equal for this arm to run.
        property: String,
    },
}

impl PropertyNameArm {
    /// Returns the property name this arm answers for.
    pub(super) fn property(&self) -> &str {
        match self {
            Self::Slot(slot) => &slot.property,
            Self::Refuse { property, .. }
            | Self::ScopeDynamic { property }
            | Self::MagicDeferred { property } => property,
        }
    }
}

/// Returns php's fetch mode for one property-read instruction, defaulting to a value read.
///
/// A missing immediate means `Read`, the raising and warning variant, so an emitter that forgets
/// the immediate cannot silently downgrade a value read into a silent probe.
pub(super) fn property_fetch_mode(inst: &Instruction) -> PropertyFetchMode {
    match inst.immediate {
        Some(Immediate::PropertyFetchMode(mode)) => mode,
        _ => PropertyFetchMode::Read,
    }
}

/// Resolves the ladder arm one property name takes on a WRITE, or `None` when it is dynamic.
///
/// `Visible` keeps the receiver's own by-name answer. `ScopePrivate` addresses the ANCESTOR's
/// slot instead, because the receiver's by-name table resolves a redeclared private property to
/// the child's shadowing slot and writing that one would hit a different property. `Dynamic`
/// drops the arm so the name falls into the caller's hash miss path, which creates the dynamic
/// property with php's deprecation. `Inaccessible` becomes a refusal: php raises a catchable
/// `Error` and never touches the storage.
pub(super) fn resolve_property_write_arm(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
    inst: &Instruction,
) -> Result<Option<PropertyNameArm>> {
    let normalized = class_name.trim_start_matches('\\');
    match resolve_property_name_in_current_scope(ctx, normalized, property) {
        PropertyNameResolution::Visible => {
            resolve_property_slot_for_class(ctx, normalized, property, inst)
                .map(|slot| Some(PropertyNameArm::Slot(slot)))
        }
        PropertyNameResolution::ScopePrivate { scope, index } => {
            resolve_scope_private_property_slot(ctx, &scope, index, property, inst)
                .map(|slot| Some(PropertyNameArm::Slot(slot)))
        }
        PropertyNameResolution::Dynamic => Ok(None),
        PropertyNameResolution::Inaccessible(visibility) => Ok(Some(PropertyNameArm::Refuse {
            property: property.to_string(),
            message: property_access_error_message(&visibility, normalized, property),
        })),
    }
}

/// Resolves the ladder arm one property name takes on a READ, or `None` when it drops out.
///
/// The FETCH MODE is what separates php's two answers for a name this scope may not reach.
/// A value read raises the catchable `Error`; `isset()`, `empty()` and `??` answer `null` in
/// silence, so their arm is dropped and the name falls into the caller's miss path. That is the
/// whole reason `Op::DynamicPropGet` carries `PropertyFetchMode`: without it, refusing here would
/// have made `isset($o->{$k})` throw, and accepting here let an unrelated scope read private
/// storage.
pub(super) fn resolve_property_read_arm(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
    mode: PropertyFetchMode,
    inst: &Instruction,
) -> Result<Option<PropertyNameArm>> {
    let normalized = class_name.trim_start_matches('\\');
    let resolution = resolve_property_name_in_current_scope(ctx, normalized, property);
    // php consults the magic accessor BEFORE it reports either diagnostic: a private property
    // reached from an unrelated scope on a class that declares `__get` answers `__get`, not
    // `Cannot access private property`, and `isset()` there answers `__isset`. This compiler does
    // not dispatch magic for a runtime name yet, so neither the refusal nor the undefined-property
    // warning may be emitted for such a class: both would be diagnostics php never reports.
    if class_declares_property_magic(ctx, normalized, mode)
        && !matches!(resolution, PropertyNameResolution::Visible)
    {
        return Ok(Some(PropertyNameArm::MagicDeferred {
            property: property.to_string(),
        }));
    }
    match resolution {
        PropertyNameResolution::Visible => {
            resolve_property_slot_for_class(ctx, normalized, property, inst)
                .map(|slot| Some(PropertyNameArm::Slot(slot)))
        }
        PropertyNameResolution::ScopePrivate { scope, index } => {
            resolve_scope_private_property_slot(ctx, &scope, index, property, inst)
                .map(|slot| Some(PropertyNameArm::Slot(slot)))
        }
        PropertyNameResolution::Dynamic => Ok(Some(PropertyNameArm::ScopeDynamic {
            property: property.to_string(),
        })),
        PropertyNameResolution::Inaccessible(visibility) => Ok(mode.is_read().then(|| {
            PropertyNameArm::Refuse {
                property: property.to_string(),
                message: property_access_error_message(&visibility, normalized, property),
            }
        })),
    }
}

/// Returns whether the class declares the magic accessor php would consult in this fetch mode.
///
/// `__get` for a value read, `__isset` for a probe. `ClassInfo::methods` is already flattened over
/// the ancestry, so an inherited accessor counts, exactly as `magic_get_receiver_class` reads it.
fn class_declares_property_magic(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    mode: PropertyFetchMode,
) -> bool {
    let magic = if mode.is_read() { "__get" } else { "__isset" };
    ctx.module
        .class_infos
        .get(class_name)
        .is_some_and(|class_info| class_info.methods.contains_key(&php_symbol_key(magic)))
}

/// Resolves the slot a property name addresses for a REFERENCE binding, ignoring accessibility.
///
/// `$x = &$mixed->p` and a by-reference return are writes as much as reads, and php's refusal for
/// them is phase B2's subject. This keeps the pre-B1 answer verbatim so the reference paths do not
/// change behaviour in a phase that owns reads: `Dynamic` has no slot, everything else takes the
/// scope-selected one.
pub(super) fn resolve_property_reference_slot(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
    inst: &Instruction,
) -> Result<Option<PropertySlot>> {
    let normalized = class_name.trim_start_matches('\\');
    match resolve_property_name_in_current_scope(ctx, normalized, property) {
        PropertyNameResolution::ScopePrivate { scope, index } => {
            resolve_scope_private_property_slot(ctx, &scope, index, property, inst).map(Some)
        }
        PropertyNameResolution::Dynamic => Ok(None),
        PropertyNameResolution::Visible | PropertyNameResolution::Inaccessible(_) => {
            resolve_property_slot_for_class(ctx, normalized, property, inst).map(Some)
        }
    }
}

/// Formats php 8.5's verbatim member-access refusal for one property.
///
/// Measured against php 8.5.10: `Cannot access private property D::$n` and
/// `Cannot access protected property Prot::$p`, with no scope suffix on either.
fn property_access_error_message(
    visibility: &Visibility,
    class_name: &str,
    property: &str,
) -> String {
    let label = match visibility {
        Visibility::Public => "public",
        Visibility::Protected => "protected",
        Visibility::Private => "private",
    };
    format!(
        "Cannot access {} property {}::${}",
        label, class_name, property
    )
}

/// Builds the slot metadata for a private property the INVOCATION SCOPE declares.
///
/// The offset is computed from the scope class's own layout index, which is also the receiver's:
/// `ClassBuildState::inherit_properties` pushes a parent's slots first and in order, so a
/// subclass layout starts with an exact copy of its parent's prefix.
fn resolve_scope_private_property_slot(
    ctx: &FunctionContext<'_>,
    scope_class: &str,
    index: usize,
    property: &str,
    inst: &Instruction,
) -> Result<PropertySlot> {
    let class_info = ctx
        .module
        .class_infos
        .get(scope_class)
        .ok_or_else(|| CodegenIrError::unsupported(format!("unknown class {}", scope_class)))?;
    let (slot_property, php_type) = class_info.properties.get(index).ok_or_else(|| {
        CodegenIrError::invalid_module("scope-private property index is outside the class layout")
    })?;
    let php_type = runtime_property_type_override(ctx, scope_class, slot_property)
        .unwrap_or_else(|| php_type.clone());
    ensure_property_type_supported(&php_type, inst)?;
    Ok(PropertySlot {
        class_name: scope_class.to_string(),
        property: property.to_string(),
        php_type,
        offset: 8 + index * 16,
        is_declared: class_info.property_slot_is_declared(index, slot_property),
        is_packed: false,
        is_reference: class_info.property_slot_is_reference(index, slot_property),
    })
}

/// Returns whether the physical layout carries `property` but php resolves it to a DYNAMIC
/// property in this scope.
///
/// The physical-slot test is what keeps this narrow. `resolve_property_name` answers `Dynamic`
/// for every name a class does not declare, so without it an ordinary undeclared name on an
/// `#[\AllowDynamicProperties]` class would take the scope-dynamic route as well. Only a strict
/// ancestor's private slot is both present in the layout and invisible by name.
pub(super) fn property_name_is_scope_dynamic(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
) -> bool {
    let normalized = class_name.trim_start_matches('\\');
    ctx.module
        .class_infos
        .get(normalized)
        .is_some_and(|class_info| {
            class_info
                .properties
                .iter()
                .any(|(name, _)| name == property)
        })
        && resolve_property_name_in_current_scope(ctx, normalized, property)
            == PropertyNameResolution::Dynamic
}

/// Returns the receiver's class when `property` is a scope-dynamic name on it.
pub(super) fn scope_dynamic_property_class_for_object(
    ctx: &FunctionContext<'_>,
    object: ValueId,
    property: &str,
) -> Result<Option<String>> {
    let PhpType::Object(class_name) = ctx.value_php_type(object)? else {
        return Ok(None);
    };
    let normalized = class_name.trim_start_matches('\\').to_string();
    Ok(property_name_is_scope_dynamic(ctx, &normalized, property).then_some(normalized))
}

/// Returns the dynamic-property hash slot offset for a known class and property name.
pub(super) fn dynamic_property_hash_offset_for_class(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
) -> Result<Option<usize>> {
    let normalized = class_name.trim_start_matches('\\');
    if is_builtin_stdclass(normalized) {
        return Ok(Some(dynamic_property_hash_offset(0)));
    }
    let class_info = ctx
        .module
        .class_infos
        .get(normalized)
        .ok_or_else(|| CodegenIrError::unsupported(format!("unknown class {}", normalized)))?;
    // A slot this SCOPE does not resolve the name to is not a collision: php keeps a strict
    // ancestor's private property under a mangled key, so the plain name belongs to the hash here.
    if class_info
        .properties
        .iter()
        .any(|(name, _)| name == property)
        && resolve_property_name_in_current_scope(ctx, normalized, property)
            != PropertyNameResolution::Dynamic
    {
        return Ok(None);
    }
    if class_info.dynamic_property_hash_is_name_addressable() {
        return Ok(Some(dynamic_property_hash_offset(
            class_info.properties.len(),
        )));
    }
    Ok(None)
}

/// Returns true when a class name is the builtin `stdClass` dynamic-property container.
pub(super) fn is_builtin_stdclass(class_name: &str) -> bool {
    crate::types::checker::builtin_stdclass::is_stdclass(class_name.trim_start_matches('\\'))
}

/// Returns true when the SSA value is known to hold a stdClass object pointer.
pub(super) fn object_is_builtin_stdclass(ctx: &FunctionContext<'_>, object: ValueId) -> Result<bool> {
    Ok(matches!(
        ctx.value_php_type(object)?.codegen_repr(),
        PhpType::Object(class_name) if is_builtin_stdclass(&class_name)
    ))
}

/// Resolves a property slot for a known class name.
pub(super) fn resolve_property_slot_for_class(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
    inst: &Instruction,
) -> Result<PropertySlot> {
    let normalized = class_name.trim_start_matches('\\');
    let class_info = ctx
        .module
        .class_infos
        .get(normalized)
        .ok_or_else(|| CodegenIrError::unsupported(format!("unknown class {}", normalized)))?;
    let Some((index, (_, php_type))) = class_info.visible_property(property) else {
        return Err(CodegenIrError::unsupported(format!(
            "{} for dynamic or missing property {}::${}",
            inst.op.name(),
            normalized,
            property
        )));
    };
    let is_reference = class_info.property_slot_is_reference(index, property);
    let php_type = runtime_property_type_override(ctx, normalized, property)
        .unwrap_or_else(|| php_type.clone());
    ensure_property_type_supported(&php_type, inst)?;
    let offset = 8 + index * 16;
    Ok(PropertySlot {
        class_name: normalized.to_string(),
        property: property.to_string(),
        php_type,
        offset,
        is_declared: class_info.property_slot_is_declared(index, property),
        is_packed: false,
        is_reference,
    })
}

/// Returns precise runtime storage types for inherited SPL callback-filter internals.
pub(super) fn runtime_property_type_override(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
) -> Option<PhpType> {
    if !class_extends_class(ctx, class_name, "CallbackFilterIterator") {
        return None;
    }
    match property {
        "callback" => Some(PhpType::Callable),
        "callbackEnv" => Some(PhpType::Pointer(None)),
        _ => None,
    }
}

/// Returns the source PHP type for an SSA value before codegen representation erasure.
pub(in crate::codegen::lower_inst) fn raw_value_php_type(ctx: &FunctionContext<'_>, value: ValueId) -> Result<PhpType> {
    ctx.function
        .value(value)
        .map(|metadata| metadata.php_type.clone())
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))
}

/// Returns the literal string payload for a value produced by `ConstStr`, when statically known.
pub(super) fn const_string_operand<'a>(ctx: &FunctionContext<'a>, value: ValueId) -> Result<Option<&'a str>> {
    let metadata = ctx
        .function
        .value(value)
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = metadata.def else {
        return Ok(None);
    };
    let instruction = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    if instruction.op != Op::ConstStr {
        return Ok(None);
    }
    let Some(Immediate::Data(data)) = instruction.immediate else {
        return Err(CodegenIrError::invalid_module(
            "const_str missing data immediate",
        ));
    };
    ctx.module
        .data
        .strings
        .get(data.as_raw() as usize)
        .map(String::as_str)
        .map(Some)
        .ok_or_else(|| CodegenIrError::missing_entry("data string", data.as_raw()))
}

/// Resolves an object or object|null source type for a nullsafe receiver.
pub(in crate::codegen::lower_inst) fn nullable_object_receiver_class(
    ctx: &FunctionContext<'_>,
    object: ValueId,
) -> Result<Option<(String, bool)>> {
    match raw_value_php_type(ctx, object)? {
        PhpType::Object(class_name) => Ok(Some((class_name, false))),
        PhpType::Union(members) => {
            let mut class_name = None;
            let mut nullable = false;
            for member in members {
                match member {
                    PhpType::Void => nullable = true,
                    PhpType::Object(candidate) => {
                        if class_name
                            .as_ref()
                            .is_some_and(|existing: &String| existing != &candidate)
                        {
                            return Ok(None);
                        }
                        class_name = Some(candidate);
                    }
                    _ => return Ok(None),
                }
            }
            Ok(class_name.map(|name| (name, nullable)))
        }
        _ => Ok(None),
    }
}

/// Returns the unique object class carried by a boxed union, ignoring null and scalar arms.
pub(super) fn union_object_member_class(ctx: &FunctionContext<'_>, object: ValueId) -> Result<Option<String>> {
    let PhpType::Union(members) = raw_value_php_type(ctx, object)? else {
        return Ok(None);
    };
    let mut class_name = None;
    for member in members {
        let PhpType::Object(candidate) = member else {
            continue;
        };
        if class_name
            .as_ref()
            .is_some_and(|existing: &String| existing != &candidate)
        {
            return Ok(None);
        }
        class_name = Some(candidate);
    }
    Ok(class_name)
}

/// Unboxes a nullable object receiver and branches when it holds PHP null.
pub(in crate::codegen::lower_inst) fn emit_nullable_receiver_object_payload(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    null_label: &str,
    object_reg: &str,
) -> Result<()> {
    let ty = ctx.load_value_to_result(object)?;
    if ty != PhpType::Mixed {
        return Err(CodegenIrError::unsupported(format!(
            "nullsafe property receiver storage {:?}",
            ty
        )));
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #8");                              // check whether the nullable receiver holds PHP null
            ctx.emitter.instruction(&format!("b.eq {}", null_label));           // short-circuit property access for nullsafe null receivers
            ctx.emitter.instruction(&format!("mov {}, x1", object_reg));        // promote the unboxed object payload into the property base register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 8");                              // check whether the nullable receiver holds PHP null
            ctx.emitter.instruction(&format!("je {}", null_label));             // short-circuit property access for nullsafe null receivers
            ctx.emitter.instruction(&format!("mov {}, rdi", object_reg));       // promote the unboxed object payload into the property base register
        }
    }
    Ok(())
}

/// Boxes a PHP null sentinel as a runtime Mixed cell.
pub(in crate::codegen::lower_inst) fn emit_boxed_null(ctx: &mut FunctionContext<'_>) {
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        RUNTIME_NULL_SENTINEL,
    );
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Void);
}

/// Resolves a field slot on an embedded packed-class receiver.
pub(super) fn resolve_packed_field_slot(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    property: &str,
    inst: &Instruction,
) -> Result<PropertySlot> {
    let normalized = class_name.trim_start_matches('\\');
    let class_info = ctx
        .module
        .packed_class_infos
        .get(normalized)
        .ok_or_else(|| {
            CodegenIrError::unsupported(format!("unknown packed class {}", normalized))
        })?;
    let Some(field) = class_info
        .fields
        .iter()
        .find(|field| field.name == property)
    else {
        return Err(CodegenIrError::unsupported(format!(
            "{} for missing packed field {}::${}",
            inst.op.name(),
            normalized,
            property
        )));
    };
    ensure_property_type_supported(&field.php_type, inst)?;
    Ok(PropertySlot {
        class_name: normalized.to_string(),
        property: property.to_string(),
        php_type: field.php_type.clone(),
        offset: field.offset,
        is_declared: false,
        is_packed: true,
        is_reference: false,
    })
}
