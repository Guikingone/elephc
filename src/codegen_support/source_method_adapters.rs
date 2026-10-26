//! Purpose:
//! Adapts source-visible method calls to physical methods carrying generated `func_args` slots.
//! Owns planning, symbols, and target-aware wrapper assembly for these boundaries.
//!
//! Called from:
//! - `crate::codegen::finalize_user_asm()` and runtime method-table emission.
//!
//! Key details:
//! - Only one trailing generated collector can be synthesized because its actual count is known.
//! - A source variadic keeps its physical entry, and no entry may drop an injected hidden argc.
//! - The wrapper roots synthesized Mixed arguments and collectors before entering PHP.
//! - It preserves every return word while releasing temporary owners after the call.

use std::collections::{HashMap, HashSet};

use crate::codegen_support::emit::Emitter;
use crate::names::{join_php_symbol, method_symbol, static_method_symbol};
use crate::types::{ClassInfo, FunctionSig, PhpType};

use super::{abi, emit_array_value_type_stamp};

/// Whether the physical method receives an object or the late-static class id first.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum MethodKind {
    Instance,
    Static,
}

/// A validated source-to-physical method ABI relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MethodAbiPlan {
    Direct,
    AppendEmptyCollector,
    BoxArguments,
    BoxArgumentsAndAppendEmptyCollector,
    /// The physical method adds optional int/bool parameters with literal defaults and the
    /// generated collector (`RecursiveDirectoryIterator::hasChildren(bool $allowLinks = false)`
    /// implementing `RecursiveIterator::hasChildren()` while every frame carries the backtrace
    /// collector). The runtime's iterator machinery calls with the interface's arity, so the
    /// wrapper appends each default and an empty collector.
    AppendDefaultsAndEmptyCollector,
    /// Both sides carry the generated collector, and the physical method inserts optional
    /// parameters before it (`ObjectManager::find(string $className, $id)` implemented by
    /// Doctrine's `EntityManager::find($className, $id, LockMode|int|null $lockMode = null,
    /// ?int $lockVersion = null)`). The wrapper boxes the shared prefix where the representation
    /// differs, materializes each inserted default, and forwards the caller's own collector.
    BoxPrefixAppendDefaultsForwardCollector,
}

impl MethodAbiPlan {
    /// Returns whether the adapter must append an empty variadic collector argument.
    fn appends_collector(self) -> bool {
        matches!(
            self,
            Self::AppendEmptyCollector
                | Self::BoxArgumentsAndAppendEmptyCollector
                | Self::AppendDefaultsAndEmptyCollector
        )
    }
}

/// Plans the safe adaptation between one caller contract and its physical implementation.
pub(crate) fn plan_method_abi(
    caller: &FunctionSig,
    physical: &FunctionSig,
) -> Result<MethodAbiPlan, String> {
    if caller.by_ref_return != physical.by_ref_return {
        return Err("source method ABI adaptation cannot bridge a by-reference return mismatch".to_string());
    }
    if same_parameter_abi(caller, physical) {
        return Ok(MethodAbiPlan::Direct);
    }
    if crate::func_args::sig_has_hidden_argc_param(caller)
        || crate::func_args::sig_has_hidden_argc_param(physical)
    {
        return Err("source method ABI adaptation cannot synthesize the hidden actual-count parameter".to_string());
    }
    if caller.variadic.is_some() {
        if collector_bearing_defaulted_tail(caller, physical) {
            // `ContainerInterface::get($id)` implemented by `Container::get($id, int
            // $invalidBehavior = ...)` while every frame carries the backtrace collector: an
            // interface call site passes the implementation's physical arity
            // (`common_interface_implementation_signature`), so the table entry is the method
            // itself, exactly as for the collector-free shape below.
            return Ok(MethodAbiPlan::Direct);
        }
        // Both sides carry the collector at the same position and only a visible parameter's
        // representation differs (`TranslatorInterface::trans(string $id, ...)` implemented with
        // `?string $id`): the collector is an ordinary forwarded argument, and the boxing
        // wrapper below handles the rest.
        // Two REAL variadics in the same position align the same way: the caller packs the
        // list, and only its element representation may need the boxing the wrapper performs
        // (`CalculatorInterface::add(NumberInterface ...$addends)` implemented with the same
        // declaration whose list the body widened to `array<mixed>`).
        let collectors_align = crate::func_args::sig_collects_surplus_args(caller)
            == crate::func_args::sig_collects_surplus_args(physical)
            && physical.variadic.is_some()
            && caller.params.len() == physical.params.len()
            && caller.ref_params == physical.ref_params;
        if collectors_align {
            let mut boxes = false;
            let compatible = caller.params.iter().zip(&physical.params).enumerate().all(
                |(index, ((_, source_ty), (_, target_ty)))| {
                    if same_physical_param_abi(source_ty, target_ty) {
                        return true;
                    }
                    let boxable = !caller.ref_params[index] && can_box_param_as_mixed(source_ty, target_ty);
                    boxes |= boxable;
                    boxable
                },
            );
            if compatible {
                return Ok(if boxes { MethodAbiPlan::BoxArguments } else { MethodAbiPlan::Direct });
            }
        }
        if collector_forwarding_defaulted_tail(caller, physical) {
            return Ok(MethodAbiPlan::BoxPrefixAppendDefaultsForwardCollector);
        }
        return Err("source method ABI adaptation cannot forward a source variadic with an unknown actual count".to_string());
    }
    let append_collector = crate::func_args::sig_collects_surplus_args(physical);
    let projected = if append_collector {
        source_visible_signature(physical)?
    } else {
        physical.clone()
    };
    let compatible_shape = caller.ref_params == projected.ref_params
        && caller.variadic.is_some() == projected.variadic.is_some()
        && caller.params.len() == projected.params.len();
    let compatible_values = caller.params.iter().zip(&projected.params).enumerate().all(
        |(index, ((_, source_ty), (_, target_ty)))| {
            same_physical_param_abi(source_ty, target_ty)
                || (!caller.ref_params[index] && can_box_param_as_mixed(source_ty, target_ty))
        },
    );
    if !compatible_shape
        && !append_collector
        && physical_only_adds_defaulted_tail(caller, &projected)
    {
        // `RecursiveDirectoryIterator::hasChildren(bool $allowLinks = false)` implementing
        // `RecursiveIterator::hasChildren()`: php accepts the extra optional parameter, and the
        // interface call reaches the implementation directly, as it did before this planner.
        // The adapter cannot materialize the omitted default itself.
        return Ok(MethodAbiPlan::Direct);
    }
    if !compatible_shape
        && append_collector
        && physical_only_adds_defaulted_tail(caller, &projected)
        && (caller.params.len()..projected.params.len()).all(|index| {
            matches!(projected.params[index].1.codegen_repr(), PhpType::Int | PhpType::Bool)
                && projected.defaults[index]
                    .as_ref()
                    .and_then(literal_default_word)
                    .is_some()
        })
    {
        return Ok(MethodAbiPlan::AppendDefaultsAndEmptyCollector);
    }
    if !compatible_shape || !compatible_values {
        let reason = if append_collector {
            "source method ABI adaptation found a visible parameter mismatch"
        } else {
            "source method ABI adaptation found an unsupported physical signature difference"
        };
        return Err(reason.to_string());
    }
    let boxes_arguments = caller.params.iter().zip(&projected.params).enumerate().any(
        |(index, ((_, source_ty), (_, target_ty)))| {
            !caller.ref_params[index] && can_box_param_as_mixed(source_ty, target_ty)
        },
    );
    Ok(match (boxes_arguments, append_collector) {
        (false, false) => MethodAbiPlan::Direct,
        (false, true) => MethodAbiPlan::AppendEmptyCollector,
        (true, false) => MethodAbiPlan::BoxArguments,
        (true, true) => MethodAbiPlan::BoxArgumentsAndAppendEmptyCollector,
    })
}

/// Returns whether `caller` and `physical` both end in the generated collector and `physical`
/// only inserts optional by-value parameters before it.
fn collector_bearing_defaulted_tail(caller: &FunctionSig, physical: &FunctionSig) -> bool {
    if !crate::func_args::sig_collects_surplus_args(caller)
        || !crate::func_args::sig_collects_surplus_args(physical)
    {
        return false;
    }
    let (Ok(caller_visible), Ok(physical_visible)) =
        (source_visible_signature(caller), source_visible_signature(physical))
    else {
        return false;
    };
    physical_only_adds_defaulted_tail(&caller_visible, &physical_visible)
}

/// Returns whether both sides end in the generated collector and the physical method only
/// inserts optional by-value parameters before it, with a prefix the wrapper can forward or box
/// and defaults it can materialize.
fn collector_forwarding_defaulted_tail(caller: &FunctionSig, physical: &FunctionSig) -> bool {
    if !crate::func_args::sig_collects_surplus_args(caller)
        || !crate::func_args::sig_collects_surplus_args(physical)
    {
        return false;
    }
    let (Ok(caller_visible), Ok(physical_visible)) =
        (source_visible_signature(caller), source_visible_signature(physical))
    else {
        return false;
    };
    let prefix = caller_visible.params.len();
    caller_visible.variadic.is_none()
        && physical_visible.variadic.is_none()
        && prefix < physical_visible.params.len()
        && caller_visible.params.iter().zip(&physical_visible.params).enumerate().all(
            |(index, ((_, source_ty), (_, target_ty)))| {
                caller_visible.ref_params.get(index) == physical_visible.ref_params.get(index)
                    && (same_physical_param_abi(source_ty, target_ty)
                        || (!caller_visible.ref_params[index]
                            && !indexed_array_widens_to_mixed_elements(source_ty, target_ty)
                            && can_box_param_as_mixed(source_ty, target_ty)))
            },
        )
        && (prefix..physical_visible.params.len()).all(|index| {
            !physical_visible.ref_params.get(index).copied().unwrap_or(false)
                && materialized_default(
                    &physical_visible.params[index].1,
                    physical_visible.defaults.get(index).and_then(Option::as_ref),
                )
                .is_some()
        })
}

/// One inserted parameter's default as the wrapper writes it.
#[derive(Clone, Debug)]
enum MaterializedDefault {
    /// A raw int/bool payload word.
    Word(PhpType, i64),
    /// `null` for a `mixed`-represented parameter: a boxed null the wrapper owns.
    MixedNull,
    /// `null` for a tagged-scalar parameter (`?int`): payload 0, tag null.
    TaggedNull,
}

/// Returns how the wrapper materializes one inserted parameter's default, if it can.
fn materialized_default(
    ty: &PhpType,
    default: Option<&crate::parser::ast::Expr>,
) -> Option<MaterializedDefault> {
    let default = default?;
    let is_null = matches!(default.kind, crate::parser::ast::ExprKind::Null);
    match ty.codegen_repr() {
        PhpType::Int | PhpType::Bool => {
            literal_default_word(default).map(|word| MaterializedDefault::Word(ty.codegen_repr(), word))
        }
        PhpType::Mixed if is_null => Some(MaterializedDefault::MixedNull),
        PhpType::TaggedScalar if is_null => Some(MaterializedDefault::TaggedNull),
        _ => None,
    }
}

/// Emits the wrapper for [`MethodAbiPlan::BoxPrefixAppendDefaultsForwardCollector`].
///
/// Frame: one 16-byte slot per incoming argument (receiver first, the caller's collector last),
/// then one per boxed prefix argument, one per inserted default, and the return slot. Every box
/// the wrapper creates is published as a call-operand owner, so a throwing callee releases it on
/// the unwind path, and released here after a normal return.
fn emit_collector_forwarding_adapter(
    emitter: &mut Emitter,
    wrapper: &str,
    implementation: &str,
    kind: MethodKind,
    caller: &FunctionSig,
    physical: &FunctionSig,
    box_return_as_mixed: bool,
) -> Result<(), String> {
    let caller_visible = source_visible_signature(caller)?;
    let physical_visible = source_visible_signature(physical)?;
    let prefix = caller_visible.params.len();
    let implicit_ty = match kind {
        MethodKind::Instance => PhpType::Object("__source_method_receiver".to_string()),
        MethodKind::Static => PhpType::Int,
    };
    let mut incoming_types = vec![implicit_ty];
    incoming_types.extend(caller.params.iter().map(|(_, ty)| ty.codegen_repr()));
    let mut incoming_refs = vec![false];
    incoming_refs.extend(caller.ref_params.iter().copied());
    let collector_index = incoming_types.len() - 1;
    let boxed_params = (0..prefix)
        .filter(|&index| {
            !caller_visible.ref_params[index]
                && !same_physical_param_abi(&caller_visible.params[index].1, &physical_visible.params[index].1)
        })
        .map(|index| index + 1)
        .collect::<Vec<_>>();
    let defaults = (prefix..physical_visible.params.len())
        .map(|index| {
            materialized_default(
                &physical_visible.params[index].1,
                physical_visible.defaults.get(index).and_then(Option::as_ref),
            )
            .ok_or_else(|| "source method ABI adapter lost an inserted default".to_string())
        })
        .collect::<Result<Vec<_>, String>>()?;
    let boxed_offsets = (0..boxed_params.len())
        .map(|index| (incoming_types.len() + 1 + index) * 16)
        .collect::<Vec<_>>();
    let default_offsets = (0..defaults.len())
        .map(|index| (incoming_types.len() + boxed_params.len() + 1 + index) * 16)
        .collect::<Vec<_>>();
    let return_offset = (incoming_types.len() + boxed_params.len() + defaults.len() + 1) * 16;
    let frame_size = return_offset + 16;

    emitter.raw(".align 2");
    emitter.label_global(wrapper);
    emitter.comment("adapt an interface call to an implementation that inserts defaulted parameters");
    abi::emit_frame_prologue(emitter, frame_size);
    let mut cursor = abi::IncomingArgCursor::for_target(emitter.target, 0);
    for (index, (ty, by_ref)) in incoming_types.iter().zip(&incoming_refs).enumerate() {
        abi::emit_store_incoming_param(
            emitter,
            "source_method_arg",
            ty,
            (index + 1) * 16,
            *by_ref,
            &mut cursor,
        );
    }
    let mut owner_offsets = Vec::new();
    let mut outgoing: Vec<(PhpType, bool, usize)> = Vec::new();
    outgoing.push((incoming_types[0].clone(), false, 16));
    for param_index in 1..=prefix {
        let incoming_offset = (param_index + 1) * 16;
        if let Some(position) = boxed_params.iter().position(|&boxed| boxed == param_index) {
            let source_ty = incoming_types[param_index].clone();
            let offset = boxed_offsets[position];
            restore_return_value(emitter, &source_ty, incoming_offset);
            if source_ty == PhpType::Void {
                abi::emit_load_int_immediate(emitter, abi::int_result_reg(emitter), 0x7fff_ffff_ffff_fffe);
            }
            super::value_boxing::emit_box_current_value_as_mixed(emitter, &source_ty);
            abi::store_at_offset(emitter, abi::int_result_reg(emitter), offset);
            let owner_address = abi::tertiary_scratch_reg(emitter);
            abi::emit_frame_slot_address(emitter, owner_address, offset);
            abi::emit_push_call_operand_owner(emitter, owner_address, false);
            owner_offsets.push(offset);
            outgoing.push((PhpType::Mixed, false, offset));
        } else {
            outgoing.push((
                incoming_types[param_index].clone(),
                incoming_refs[param_index],
                incoming_offset,
            ));
        }
    }
    for (default, &offset) in defaults.iter().zip(&default_offsets) {
        match default {
            MaterializedDefault::Word(ty, word) => {
                abi::emit_load_int_immediate(emitter, abi::int_result_reg(emitter), *word);
                abi::store_at_offset(emitter, abi::int_result_reg(emitter), offset);
                outgoing.push((ty.clone(), false, offset));
            }
            MaterializedDefault::MixedNull => {
                abi::emit_load_int_immediate(emitter, abi::int_result_reg(emitter), 0x7fff_ffff_ffff_fffe);
                super::value_boxing::emit_box_current_value_as_mixed(emitter, &PhpType::Void);
                abi::store_at_offset(emitter, abi::int_result_reg(emitter), offset);
                let owner_address = abi::tertiary_scratch_reg(emitter);
                abi::emit_frame_slot_address(emitter, owner_address, offset);
                abi::emit_push_call_operand_owner(emitter, owner_address, false);
                owner_offsets.push(offset);
                outgoing.push((PhpType::Mixed, false, offset));
            }
            MaterializedDefault::TaggedNull => {
                abi::emit_load_int_immediate(emitter, abi::int_result_reg(emitter), 0);
                abi::store_at_offset(emitter, abi::int_result_reg(emitter), offset);
                abi::emit_load_int_immediate(
                    emitter,
                    abi::int_result_reg(emitter),
                    crate::codegen_support::sentinels::TAGGED_SCALAR_TAG_NULL,
                );
                abi::store_at_offset(emitter, abi::int_result_reg(emitter), offset - 8);
                outgoing.push((PhpType::TaggedScalar, false, offset));
            }
        }
    }
    outgoing.push((
        incoming_types[collector_index].clone(),
        incoming_refs[collector_index],
        (collector_index + 1) * 16,
    ));

    let abi_types = outgoing
        .iter()
        .map(|(ty, by_ref, _)| if *by_ref { PhpType::Int } else { ty.codegen_repr() })
        .collect::<Vec<_>>();
    for (ty, (_, _, offset)) in abi_types.iter().zip(&outgoing) {
        push_frame_value(emitter, ty, *offset);
    }
    let assignments = abi::build_outgoing_arg_assignments_for_target(emitter.target, &abi_types, 0);
    let overflow = abi::materialize_outgoing_args(emitter, &assignments);
    let pad = abi::outgoing_call_stack_pad_bytes(emitter.target, overflow);
    abi::emit_reserve_temporary_stack(emitter, pad);
    abi::emit_call_label(emitter, implementation);
    abi::emit_release_temporary_stack(emitter, pad);
    abi::emit_release_temporary_stack(emitter, overflow);

    let physical_return = if physical.by_ref_return {
        PhpType::Int
    } else {
        physical.return_type.codegen_repr()
    };
    preserve_return_value(emitter, &physical_return, return_offset);
    for &offset in owner_offsets.iter().rev() {
        abi::emit_pop_call_operand_owner(emitter);
        abi::load_at_offset(emitter, abi::int_result_reg(emitter), offset);
        abi::emit_store_zero_to_local_slot(emitter, offset);
        abi::emit_call_label(emitter, "__rt_decref_any");
    }
    restore_return_value(emitter, &physical_return, return_offset);
    if box_return_as_mixed {
        if physical_return == PhpType::Void {
            abi::emit_load_int_immediate(emitter, abi::int_result_reg(emitter), 0x7fff_ffff_ffff_fffe);
        }
        super::value_boxing::emit_box_current_value_as_mixed(emitter, &physical_return);
    }
    abi::emit_frame_restore(emitter, frame_size);
    abi::emit_return(emitter);
    Ok(())
}

/// Resolves a literal int/bool default to its argument word.
fn literal_default_word(default: &crate::parser::ast::Expr) -> Option<i64> {
    use crate::parser::ast::ExprKind;
    match &default.kind {
        ExprKind::IntLiteral(value) => Some(*value),
        ExprKind::BoolLiteral(value) => Some(i64::from(*value)),
        ExprKind::Negate(inner) => match inner.kind {
            ExprKind::IntLiteral(value) => value.checked_neg(),
            _ => None,
        },
        _ => None,
    }
}

/// Returns whether `physical` is `caller` plus trailing optional by-value parameters, with the
/// shared prefix already ABI-identical.
fn physical_only_adds_defaulted_tail(caller: &FunctionSig, physical: &FunctionSig) -> bool {
    caller.variadic.is_none()
        && physical.variadic.is_none()
        && caller.params.len() < physical.params.len()
        && caller.params.iter().zip(&physical.params).enumerate().all(
            |(index, ((_, source_ty), (_, target_ty)))| {
                same_physical_param_abi(source_ty, target_ty)
                    && caller.ref_params.get(index) == physical.ref_params.get(index)
            },
        )
        && (caller.params.len()..physical.params.len()).all(|index| {
            physical.defaults.get(index).is_some_and(Option::is_some)
                && !physical.ref_params.get(index).copied().unwrap_or(false)
        })
}

/// Selects a source-vtable entry, keeping every source variadic on its raw physical entry.
///
/// A source variadic reports its generated slots through `uses_physical_func_args_abi()`, so
/// every vtable caller already dispatches it through `_class_vtable_ptrs` or
/// `_class_static_vtable_ptrs`, and the operand path cannot reach it because
/// `source_visible_signature()` refuses to project a hidden actual count. The source slot is
/// therefore the same raw entry, with or without that hidden count. A fixed signature carrying
/// an injected hidden argc has no such caller and must never be published without the slot.
pub(crate) fn source_vtable_entry_symbol(
    class_name: &str,
    method_name: &str,
    physical: &FunctionSig,
    kind: MethodKind,
) -> Result<String, String> {
    if physical.variadic.is_some()
        && !crate::func_args::sig_collects_surplus_args(physical)
    {
        return Ok(physical_method_symbol(class_name, method_name, kind));
    }
    if crate::func_args::sig_has_hidden_argc_param(physical) {
        return Err(
            "source method vtable cannot omit a source variadic hidden actual-count parameter"
                .to_string(),
        );
    }
    source_method_entry_symbol(class_name, method_name, physical, kind)
}

/// Projects a collector-bearing physical signature back to the declaration visible in PHP.
pub(crate) fn source_visible_signature(physical: &FunctionSig) -> Result<FunctionSig, String> {
    if crate::func_args::sig_has_hidden_argc_param(physical) {
        return Err("source method ABI adaptation cannot project a source variadic hidden argc".to_string());
    }
    let mut source = physical.clone();
    if crate::func_args::sig_collects_surplus_args(physical) {
        let Some((name, _)) = source.params.last() else {
            return Err("generated method collector is missing its physical parameter".to_string());
        };
        if name != crate::func_args::HIDDEN_ARGS_PARAM {
            return Err("generated method collector is not the final physical parameter".to_string());
        }
        source.params.pop();
        source.param_type_exprs.pop();
        source.param_attributes.pop();
        source.defaults.pop();
        source.ref_params.pop();
        source.declared_params.pop();
        source.variadic = None;
    }
    if source.variadic.is_some() {
        return Err("source method ABI adaptation cannot project a source variadic".to_string());
    }
    Ok(source)
}

/// Returns true when a call signature already carries generated physical `func_args` slots.
pub(crate) fn uses_physical_func_args_abi(sig: &FunctionSig) -> bool {
    crate::func_args::sig_collects_surplus_args(sig)
        || crate::func_args::sig_has_hidden_argc_param(sig)
}

/// Returns the shared source-boundary symbol for one physical method implementation.
pub(crate) fn source_method_adapter_symbol(
    class_name: &str,
    method_name: &str,
    kind: MethodKind,
) -> String {
    let prefix = match kind {
        MethodKind::Instance => "_method_source_abi",
        MethodKind::Static => "_static_source_abi",
    };
    join_php_symbol(prefix, &[class_name, method_name])
}

/// Selects a raw method symbol or its source-boundary adapter after validating the ABI.
pub(crate) fn source_method_entry_symbol(
    class_name: &str,
    method_name: &str,
    physical: &FunctionSig,
    kind: MethodKind,
) -> Result<String, String> {
    let source = source_visible_signature(physical)?;
    match plan_method_abi(&source, physical)? {
        MethodAbiPlan::Direct => Ok(physical_method_symbol(class_name, method_name, kind)),
        MethodAbiPlan::AppendEmptyCollector
        | MethodAbiPlan::BoxArguments
        | MethodAbiPlan::BoxArgumentsAndAppendEmptyCollector
        | MethodAbiPlan::AppendDefaultsAndEmptyCollector
        | MethodAbiPlan::BoxPrefixAppendDefaultsForwardCollector => Ok(source_method_adapter_symbol(
            class_name,
            method_name,
            kind,
        )),
    }
}

/// Emits one deduplicated source adapter for every collector-bearing implementation in scope.
pub(crate) fn emit_source_method_adapters(
    emitter: &mut Emitter,
    classes: &HashMap<String, ClassInfo, impl std::hash::BuildHasher>,
    emitted_class_names: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut specs = Vec::new();
    for (class_name, class_info) in classes {
        if emitted_class_names.is_some_and(|names| !names.contains(class_name)) {
            continue;
        }
        for (method_name, sig) in &class_info.methods {
            if crate::func_args::sig_collects_surplus_args(sig)
                && class_info
                    .method_impl_classes
                    .get(method_name)
                    .is_some_and(|owner| owner == class_name)
            {
                specs.push((class_name.clone(), method_name.clone(), MethodKind::Instance));
            }
        }
        for (method_name, sig) in &class_info.static_methods {
            if crate::func_args::sig_collects_surplus_args(sig)
                && class_info
                    .static_method_impl_classes
                    .get(method_name)
                    .is_some_and(|owner| owner == class_name)
            {
                specs.push((class_name.clone(), method_name.clone(), MethodKind::Static));
            }
        }
    }
    specs.sort();
    specs.dedup();
    for (class_name, method_name, kind) in specs {
        let class_info = classes
            .get(&class_name)
            .ok_or_else(|| format!("missing class metadata for source adapter {class_name}"))?;
        let physical = match kind {
            MethodKind::Instance => class_info.methods.get(&method_name),
            MethodKind::Static => class_info.static_methods.get(&method_name),
        }
        .ok_or_else(|| format!("missing physical signature for {class_name}::{method_name}"))?;
        let source = source_visible_signature(physical)?;
        let symbol = source_method_adapter_symbol(&class_name, &method_name, kind);
        let implementation = physical_method_symbol(&class_name, &method_name, kind);
        emit_method_adapter(
            emitter,
            &symbol,
            &implementation,
            kind,
            &source,
            physical,
            false,
        )?;
    }
    Ok(())
}

/// Emits one wrapper, optionally boxing its concrete return for an interface slot.
pub(crate) fn emit_method_adapter(
    emitter: &mut Emitter,
    wrapper: &str,
    implementation: &str,
    kind: MethodKind,
    caller: &FunctionSig,
    physical: &FunctionSig,
    box_return_as_mixed: bool,
) -> Result<(), String> {
    let plan = plan_method_abi(caller, physical)?;
    if box_return_as_mixed && physical.by_ref_return {
        return Err("source method ABI adapter cannot value-box a by-reference return".to_string());
    }
    if plan == MethodAbiPlan::BoxPrefixAppendDefaultsForwardCollector {
        return emit_collector_forwarding_adapter(
            emitter,
            wrapper,
            implementation,
            kind,
            caller,
            physical,
            box_return_as_mixed,
        );
    }
    let implicit_ty = match kind {
        MethodKind::Instance => PhpType::Object("__source_method_receiver".to_string()),
        MethodKind::Static => PhpType::Int,
    };
    let mut incoming_types = vec![implicit_ty];
    incoming_types.extend(caller.params.iter().map(|(_, ty)| ty.codegen_repr()));
    let mut incoming_refs = vec![false];
    incoming_refs.extend(caller.ref_params.iter().copied());
    let boxed_params = caller.params.iter().zip(&physical.params).enumerate()
        .filter_map(|(index, ((_, source_ty), (_, target_ty)))| {
            (!caller.ref_params[index] && can_box_param_as_mixed(source_ty, target_ty))
                .then_some(index + 1)
        })
        .collect::<Vec<_>>();
    let boxed_targets = boxed_params
        .iter()
        .map(|&param_index| physical.params[param_index - 1].1.codegen_repr())
        .collect::<Vec<_>>();
    let mut outgoing_offsets = (1..=incoming_types.len()).map(|index| index * 16).collect::<Vec<_>>();
    let boxed_offsets = boxed_params.iter().enumerate().map(|(index, _)| {
        (incoming_types.len() + 1 + index) * 16
    }).collect::<Vec<_>>();
    // `AppendDefaultsAndEmptyCollector`: the physical parameters past the caller's, each with
    // its literal default word.
    let appended_defaults = if plan == MethodAbiPlan::AppendDefaultsAndEmptyCollector {
        let projected = source_visible_signature(physical)?;
        (caller.params.len()..projected.params.len())
            .map(|index| {
                let word = projected.defaults[index]
                    .as_ref()
                    .and_then(literal_default_word)
                    .ok_or_else(|| "source method ABI adapter lost a literal default".to_string())?;
                Ok((projected.params[index].1.codegen_repr(), word))
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        Vec::new()
    };
    let default_offsets = (0..appended_defaults.len())
        .map(|index| (incoming_types.len() + boxed_offsets.len() + 1 + index) * 16)
        .collect::<Vec<_>>();
    let owner_offset = (incoming_types.len() + boxed_offsets.len() + default_offsets.len() + 1) * 16;
    let return_offset = owner_offset + 16;
    let frame_size = return_offset + 16;

    emitter.raw(".align 2");
    emitter.label_global(wrapper);
    emitter.comment("adapt source-visible method ABI to physical method ABI");
    abi::emit_frame_prologue(emitter, frame_size);
    let mut cursor = abi::IncomingArgCursor::for_target(emitter.target, 0);
    for (index, (ty, by_ref)) in incoming_types.iter().zip(&incoming_refs).enumerate() {
        abi::emit_store_incoming_param(
            emitter,
            "source_method_arg",
            ty,
            (index + 1) * 16,
            *by_ref,
            &mut cursor,
        );
    }
    let mut outgoing_types = incoming_types.clone();
    for ((&param_index, &offset), target_ty) in
        boxed_params.iter().zip(&boxed_offsets).zip(&boxed_targets)
    {
        let source_ty = &incoming_types[param_index];
        restore_return_value(emitter, source_ty, (param_index + 1) * 16);
        if indexed_array_widens_to_mixed_elements(source_ty, target_ty) {
            // `__rt_array_to_mixed` consumes one reference and answers a list of boxed elements;
            // the retain keeps the caller's own list intact, and the result is this frame's owner.
            let PhpType::Array(source_elem) = source_ty.codegen_repr() else { unreachable!() };
            abi::emit_incref_if_refcounted(emitter, &source_ty.codegen_repr());
            let tag = super::runtime_value_tag(&source_elem.codegen_repr()) as i64;
            match emitter.target.arch {
                crate::codegen_support::platform::Arch::AArch64 => {
                    abi::emit_load_int_immediate(emitter, "x1", tag);
                }
                crate::codegen_support::platform::Arch::X86_64 => {
                    abi::emit_load_int_immediate(emitter, "rsi", tag);
                    emitter.instruction("mov rdi, rax");                        // pass the caller's list to the element-boxing helper
                }
            }
            abi::emit_call_label(emitter, "__rt_array_to_mixed");
        } else {
            if *source_ty == PhpType::Void {
                abi::emit_load_int_immediate(
                    emitter,
                    abi::int_result_reg(emitter),
                    0x7fff_ffff_ffff_fffe,
                );
            }
            super::value_boxing::emit_box_current_value_as_mixed(emitter, source_ty);
        }
        abi::store_at_offset(emitter, abi::int_result_reg(emitter), offset);
        let owner_address = abi::tertiary_scratch_reg(emitter);
        abi::emit_frame_slot_address(emitter, owner_address, offset);
        abi::emit_push_call_operand_owner(emitter, owner_address, false);
        outgoing_types[param_index] = target_ty.clone();
        outgoing_offsets[param_index] = offset;
    }
    for ((_, word), &offset) in appended_defaults.iter().zip(&default_offsets) {
        abi::emit_load_int_immediate(emitter, abi::int_result_reg(emitter), *word);
        abi::store_at_offset(emitter, abi::int_result_reg(emitter), offset);
    }
    if plan.appends_collector() {
        abi::emit_store_zero_to_local_slot(emitter, owner_offset);
        emit_empty_collector_owner(emitter, owner_offset);
    }

    let mut outgoing_refs = incoming_refs.clone();
    for ((ty, _), &offset) in appended_defaults.iter().zip(&default_offsets) {
        outgoing_types.push(ty.clone());
        outgoing_refs.push(false);
        outgoing_offsets.push(offset);
    }
    if plan.appends_collector() {
        outgoing_types.push(PhpType::Array(Box::new(PhpType::Mixed)));
        outgoing_refs.push(false);
        outgoing_offsets.push(owner_offset);
    }
    let abi_types = outgoing_types
        .iter()
        .zip(&outgoing_refs)
        .map(|(ty, by_ref)| if *by_ref { PhpType::Int } else { ty.codegen_repr() })
        .collect::<Vec<_>>();
    for (ty, offset) in abi_types.iter().zip(&outgoing_offsets) {
        push_frame_value(emitter, ty, *offset);
    }
    let assignments = abi::build_outgoing_arg_assignments_for_target(emitter.target, &abi_types, 0);
    let overflow = abi::materialize_outgoing_args(emitter, &assignments);
    let pad = abi::outgoing_call_stack_pad_bytes(emitter.target, overflow);
    abi::emit_reserve_temporary_stack(emitter, pad);
    abi::emit_call_label(emitter, implementation);
    abi::emit_release_temporary_stack(emitter, pad);
    abi::emit_release_temporary_stack(emitter, overflow);

    let physical_return = if physical.by_ref_return {
        PhpType::Int
    } else {
        physical.return_type.codegen_repr()
    };
    preserve_return_value(emitter, &physical_return, return_offset);
    if plan.appends_collector() {
        abi::emit_pop_call_operand_owner(emitter);
        abi::load_at_offset(emitter, abi::int_result_reg(emitter), owner_offset);
        abi::emit_store_zero_to_local_slot(emitter, owner_offset);
        abi::emit_call_label(emitter, "__rt_decref_any");
    }
    for &offset in boxed_offsets.iter().rev() {
        abi::emit_pop_call_operand_owner(emitter);
        abi::load_at_offset(emitter, abi::int_result_reg(emitter), offset);
        abi::emit_store_zero_to_local_slot(emitter, offset);
        abi::emit_call_label(emitter, "__rt_decref_any");
    }
    restore_return_value(emitter, &physical_return, return_offset);
    if box_return_as_mixed {
        if physical_return == PhpType::Void {
            abi::emit_load_int_immediate(
                emitter,
                abi::int_result_reg(emitter),
                0x7fff_ffff_ffff_fffe,
            );
        }
        super::value_boxing::emit_box_current_value_as_mixed(emitter, &physical_return);
    }
    abi::emit_frame_restore(emitter, frame_size);
    abi::emit_return(emitter);
    Ok(())
}

/// Returns true when two contracts pass the same argument words in the same order.
///
/// Parameter names can differ, and object class names share one pointer representation.
/// Arity, by-reference passing, and the value representation must match for direct forwarding.
fn same_parameter_abi(left: &FunctionSig, right: &FunctionSig) -> bool {
    left.ref_params == right.ref_params
        && left.variadic.is_some() == right.variadic.is_some()
        && left.params.len() == right.params.len()
        && left
            .params
            .iter()
            .zip(right.params.iter())
            .all(|((_, left_ty), (_, right_ty))| same_physical_param_abi(left_ty, right_ty))
}

/// Returns true when two parameter types have compatible physical value representations.
/// Register count alone does not distinguish boxed cells, pointers, scalars, or tagged values.
fn same_physical_param_abi(left: &PhpType, right: &PhpType) -> bool {
    left.reference_payload_compatible(right)
}

/// Returns whether a source parameter can be boxed for a physical Mixed parameter.
fn can_box_param_as_mixed(source: &PhpType, target: &PhpType) -> bool {
    if indexed_array_widens_to_mixed_elements(source, target) {
        return true;
    }
    target.codegen_repr() == PhpType::Mixed
        && source.codegen_repr() != PhpType::Mixed
        && !matches!(source.codegen_repr(), PhpType::Never)
}

/// Returns whether a caller's typed list must be rebuilt with boxed elements for the physical
/// method: a declared variadic (`NumberInterface ...$addends`) stays `array<NumberInterface>` on
/// a bodiless interface method while its implementation's collector is `array<mixed>`.
fn indexed_array_widens_to_mixed_elements(source: &PhpType, target: &PhpType) -> bool {
    matches!(
        (source.codegen_repr(), target.codegen_repr()),
        (PhpType::Array(source_elem), PhpType::Array(target_elem))
            if target_elem.codegen_repr() == PhpType::Mixed
                && !matches!(source_elem.codegen_repr(), PhpType::Mixed | PhpType::Never)
    )
}

fn physical_method_symbol(class_name: &str, method_name: &str, kind: MethodKind) -> String {
    match kind {
        MethodKind::Instance => method_symbol(class_name, method_name),
        MethodKind::Static => static_method_symbol(class_name, method_name),
    }
}

fn emit_empty_collector_owner(emitter: &mut Emitter, owner_offset: usize) {
    abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(emitter.target, 0), 0);
    abi::emit_load_int_immediate(
        emitter,
        abi::int_arg_reg_name(emitter.target, 1),
        PhpType::Mixed.stack_size() as i64,
    );
    abi::emit_call_label(emitter, "__rt_array_new");
    emit_array_value_type_stamp(emitter, abi::int_result_reg(emitter), &PhpType::Mixed);
    abi::store_at_offset(emitter, abi::int_result_reg(emitter), owner_offset);
    let owner_address = abi::tertiary_scratch_reg(emitter);
    abi::emit_frame_slot_address(emitter, owner_address, owner_offset);
    abi::emit_push_call_operand_owner(emitter, owner_address, false);
}

fn push_frame_value(emitter: &mut Emitter, ty: &PhpType, offset: usize) {
    match ty.codegen_repr() {
        PhpType::Float => {
            let reg = abi::float_result_reg(emitter);
            abi::load_at_offset(emitter, reg, offset);
            abi::emit_push_float_reg(emitter, reg);
        }
        PhpType::Str => {
            let (lo, hi) = abi::string_result_regs(emitter);
            abi::load_at_offset(emitter, lo, offset);
            abi::load_at_offset(emitter, hi, offset - 8);
            abi::emit_push_reg_pair(emitter, lo, hi);
        }
        PhpType::TaggedScalar => {
            let lo = abi::int_result_reg(emitter);
            let hi = crate::codegen_support::sentinels::tagged_scalar_tag_reg(emitter);
            abi::load_at_offset(emitter, lo, offset);
            abi::load_at_offset(emitter, hi, offset - 8);
            abi::emit_push_reg_pair(emitter, lo, hi);
        }
        PhpType::Void | PhpType::Never => {}
        _ => {
            let reg = abi::int_result_reg(emitter);
            abi::load_at_offset(emitter, reg, offset);
            abi::emit_push_reg(emitter, reg);
        }
    }
}

fn preserve_return_value(emitter: &mut Emitter, ty: &PhpType, offset: usize) {
    match ty.codegen_repr() {
        PhpType::Float => abi::store_at_offset(emitter, abi::float_result_reg(emitter), offset),
        PhpType::Str => {
            let (lo, hi) = abi::string_result_regs(emitter);
            abi::store_at_offset(emitter, lo, offset);
            abi::store_at_offset(emitter, hi, offset - 8);
        }
        PhpType::TaggedScalar => {
            let lo = abi::int_result_reg(emitter);
            let hi = crate::codegen_support::sentinels::tagged_scalar_tag_reg(emitter);
            abi::store_at_offset(emitter, lo, offset);
            abi::store_at_offset(emitter, hi, offset - 8);
        }
        PhpType::Void | PhpType::Never => {}
        _ => abi::store_at_offset(emitter, abi::int_result_reg(emitter), offset),
    }
}

fn restore_return_value(emitter: &mut Emitter, ty: &PhpType, offset: usize) {
    match ty.codegen_repr() {
        PhpType::Float => abi::load_at_offset(emitter, abi::float_result_reg(emitter), offset),
        PhpType::Str => {
            let (lo, hi) = abi::string_result_regs(emitter);
            abi::load_at_offset(emitter, lo, offset);
            abi::load_at_offset(emitter, hi, offset - 8);
        }
        PhpType::TaggedScalar => {
            let lo = abi::int_result_reg(emitter);
            let hi = crate::codegen_support::sentinels::tagged_scalar_tag_reg(emitter);
            abi::load_at_offset(emitter, lo, offset);
            abi::load_at_offset(emitter, hi, offset - 8);
        }
        PhpType::Void | PhpType::Never => {}
        _ => abi::load_at_offset(emitter, abi::int_result_reg(emitter), offset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(params: Vec<(String, PhpType)>, variadic: Option<String>) -> FunctionSig {
        let len = params.len();
        FunctionSig {
            params,
            param_type_exprs: vec![None; len],
            param_attributes: vec![Vec::new(); len],
            defaults: vec![None; len],
            return_type: PhpType::Str,
            declared_return: true,
            by_ref_return: false,
            ref_params: vec![false; len],
            declared_params: vec![true; len],
            variadic,
            deprecation: None,
            is_generator: false,
        }
    }

    #[test]
    fn planner_accepts_only_one_generated_collector_difference() {
        let source = signature(vec![("name".to_string(), PhpType::Str)], None);
        let mut physical = source.clone();
        physical.params.push((
            crate::func_args::HIDDEN_ARGS_PARAM.to_string(),
            PhpType::Array(Box::new(PhpType::Mixed)),
        ));
        physical.param_type_exprs.push(None);
        physical.param_attributes.push(Vec::new());
        physical.defaults.push(None);
        physical.ref_params.push(false);
        physical.declared_params.push(false);
        physical.variadic = Some(crate::func_args::HIDDEN_ARGS_PARAM.to_string());
        assert_eq!(
            plan_method_abi(&source, &physical).unwrap(),
            MethodAbiPlan::AppendEmptyCollector
        );
    }

    #[test]
    fn planner_rejects_source_variadic_actual_count_adaptation() {
        let source = signature(
            vec![("values".to_string(), PhpType::Mixed)],
            Some("values".to_string()),
        );
        let mut physical = source.clone();
        physical.params.insert(
            0,
            (crate::func_args::HIDDEN_ARGC_PARAM.to_string(), PhpType::Int),
        );
        physical.param_type_exprs.insert(0, None);
        physical.param_attributes.insert(0, Vec::new());
        physical.defaults.insert(0, None);
        physical.ref_params.insert(0, false);
        physical.declared_params.insert(0, false);
        assert!(plan_method_abi(&source, &physical)
            .unwrap_err()
            .contains("actual-count"));
    }

    #[test]
    fn source_vtable_keeps_source_variadics_raw_and_never_drops_an_injected_hidden_argc() {
        let mut physical = signature(
            vec![("values".to_string(), PhpType::Mixed)],
            Some("values".to_string()),
        );
        assert_eq!(
            source_vtable_entry_symbol("C", "sort", &physical, MethodKind::Static).unwrap(),
            static_method_symbol("C", "sort")
        );
        physical.params.insert(
            0,
            (crate::func_args::HIDDEN_ARGC_PARAM.to_string(), PhpType::Int),
        );
        physical.param_type_exprs.insert(0, None);
        physical.param_attributes.insert(0, Vec::new());
        physical.defaults.insert(0, None);
        physical.ref_params.insert(0, false);
        physical.declared_params.insert(0, false);
        // The hidden actual count rides the same physical entry every vtable caller already uses
        // for a source variadic, so the source slot stays the raw symbol.
        assert_eq!(
            source_vtable_entry_symbol("C", "sort", &physical, MethodKind::Static).unwrap(),
            static_method_symbol("C", "sort")
        );

        // A fixed signature carrying the injected count has no such caller and must be rejected.
        let mut fixed = physical.clone();
        fixed.variadic = None;
        assert!(source_vtable_entry_symbol("C", "sort", &fixed, MethodKind::Static)
            .unwrap_err()
            .contains("actual-count"));
    }

    /// Rejects boxed, pointer, and tagged arguments that merely share register counts.
    #[test]
    fn planner_rejects_incompatible_value_representations() {
        // An object pointer and a Mixed cell pointer occupy one word, but the callee
        // must not interpret the cell address as an object pointer.
        let caller = signature(
            vec![
                ("offset".to_string(), PhpType::Mixed),
                ("value".to_string(), PhpType::Mixed),
            ],
            None,
        );
        let mut physical = caller.clone();
        physical.params[1] = (
            "iterator".to_string(),
            PhpType::Object("Iterator".to_string()),
        );
        assert!(plan_method_abi(&caller, &physical)
            .unwrap_err()
            .contains("unsupported physical signature difference"));

        let mut raw_pointer = caller.clone();
        raw_pointer.params[1].1 = PhpType::Object("Iterator".to_string());
        let mut raw_integer = raw_pointer.clone();
        raw_integer.params[1].1 = PhpType::Int;
        assert!(plan_method_abi(&raw_pointer, &raw_integer)
            .unwrap_err()
            .contains("unsupported physical signature difference"));

        let mut string_pair = caller.clone();
        string_pair.params[1].1 = PhpType::Str;
        let mut tagged_pair = string_pair.clone();
        tagged_pair.params[1].1 = PhpType::TaggedScalar;
        assert!(plan_method_abi(&string_pair, &tagged_pair)
            .unwrap_err()
            .contains("unsupported physical signature difference"));

        physical.params[1] = ("value".to_string(), PhpType::Int);
        assert!(plan_method_abi(&caller, &physical)
            .unwrap_err()
            .contains("unsupported physical signature difference"));

        // A parameter that changes register class is still an unsupported difference.
        let mut float_physical = caller.clone();
        float_physical.params[1] = ("value".to_string(), PhpType::Float);
        assert!(plan_method_abi(&caller, &float_physical)
            .unwrap_err()
            .contains("unsupported physical signature difference"));

        let mut by_ref_caller = raw_pointer.clone();
        by_ref_caller.ref_params[1] = true;
        let mut by_ref_mixed = by_ref_caller.clone();
        by_ref_mixed.params[1].1 = PhpType::Mixed;
        assert!(plan_method_abi(&by_ref_caller, &by_ref_mixed)
            .unwrap_err()
            .contains("unsupported physical signature difference"));
    }

    #[test]
    fn by_reference_return_is_preserved_as_one_pointer_and_never_value_boxed() {
        let mut caller = signature(Vec::new(), None);
        caller.return_type = PhpType::Mixed;
        caller.by_ref_return = true;
        let mut physical = caller.clone();
        physical.return_type = PhpType::Str;
        physical.params.push((
            crate::func_args::HIDDEN_ARGS_PARAM.to_string(),
            PhpType::Array(Box::new(PhpType::Mixed)),
        ));
        physical.param_type_exprs.push(None);
        physical.param_attributes.push(Vec::new());
        physical.defaults.push(None);
        physical.ref_params.push(false);
        physical.declared_params.push(false);
        physical.variadic = Some(crate::func_args::HIDDEN_ARGS_PARAM.to_string());

        let mut mismatched_caller = caller.clone();
        mismatched_caller.by_ref_return = false;
        assert!(plan_method_abi(&mismatched_caller, &physical)
            .unwrap_err()
            .contains("by-reference return mismatch"));

        let mut emitter = Emitter::new(
            crate::codegen_support::platform::Target::parse("linux-x86_64").unwrap(),
        );
        emit_method_adapter(
            &mut emitter,
            "_test_by_ref_adapter",
            "_test_by_ref_physical",
            MethodKind::Instance,
            &caller,
            &physical,
            false,
        )
        .unwrap();
        assert!(!emitter.output().contains("__rt_mixed_from_value"));
        assert!(emit_method_adapter(
            &mut Emitter::new(
                crate::codegen_support::platform::Target::parse("linux-x86_64").unwrap(),
            ),
            "_test_invalid_by_ref_box",
            "_test_by_ref_physical",
            MethodKind::Instance,
            &caller,
            &physical,
            true,
        )
        .unwrap_err()
        .contains("by-reference"));
    }

    /// Ensures every target materializes a null payload before boxing an untyped void return.
    #[test]
    fn interface_wrapper_boxes_void_return_on_all_targets() {
        let mut caller = signature(Vec::new(), None);
        caller.return_type = PhpType::Mixed;
        let mut physical = caller.clone();
        physical.return_type = PhpType::Void;

        for target in [
            "macos-aarch64",
            "ios-arm64",
            "ios-sim-arm64",
            "linux-aarch64",
            "linux-x86_64",
        ] {
            let mut emitter = Emitter::new(
                crate::codegen_support::platform::Target::parse(target).unwrap(),
            );
            emit_method_adapter(
                &mut emitter,
                "_test_interface_wrapper",
                "_test_void_method",
                MethodKind::Instance,
                &caller,
                &physical,
                true,
            )
            .unwrap();
            let asm = emitter.output();
            assert!(asm.contains("__rt_mixed_from_value"), "{target}");
            let null_payload = if target == "linux-x86_64" {
                "mov rax, 9223372036854775806"
            } else {
                "movz x0, #0xfffe"
            };
            assert!(asm.contains(null_payload), "{target}");
        }
    }

    /// Typed arguments widened to mixed are boxed and released on every supported target.
    #[test]
    fn interface_wrapper_boxes_widened_parameters_on_all_targets() {
        let caller = signature(vec![
            ("user".to_string(), PhpType::Object("User".to_string())),
            ("label".to_string(), PhpType::Str),
        ], None);
        let mut physical = caller.clone();
        physical.params[0].1 = PhpType::Mixed;
        physical.params[1].1 = PhpType::Mixed;
        assert_eq!(plan_method_abi(&caller, &physical).unwrap(), MethodAbiPlan::BoxArguments);

        for target in [
            "macos-aarch64",
            "ios-arm64",
            "ios-sim-arm64",
            "linux-aarch64",
            "linux-x86_64",
        ] {
            let mut emitter = Emitter::new(
                crate::codegen_support::platform::Target::parse(target).unwrap(),
            );
            emit_method_adapter(
                &mut emitter,
                "_test_interface_wrapper",
                "_test_mixed_method",
                MethodKind::Instance,
                &caller,
                &physical,
                false,
            )
            .unwrap();
            let asm = emitter.output();
            assert_eq!(asm.matches("__rt_mixed_from_value").count(), 2, "{target}");
            assert_eq!(asm.matches("__rt_decref_any").count(), 2, "{target}");
            assert!(asm.find("param $source_method_arg").unwrap()
                < asm.find("__rt_mixed_from_value").unwrap(), "{target}");
            assert!(asm.find("_test_mixed_method").unwrap()
                < asm.find("__rt_decref_any").unwrap(), "{target}");
        }
    }

    #[test]
    fn emitter_spills_before_allocating_and_balances_collector_ownership_on_all_targets() {
        let source = signature(vec![("name".to_string(), PhpType::Str)], None);
        let mut physical = source.clone();
        physical.params.push((
            crate::func_args::HIDDEN_ARGS_PARAM.to_string(),
            PhpType::Array(Box::new(PhpType::Mixed)),
        ));
        physical.param_type_exprs.push(None);
        physical.param_attributes.push(Vec::new());
        physical.defaults.push(None);
        physical.ref_params.push(false);
        physical.declared_params.push(false);
        physical.variadic = Some(crate::func_args::HIDDEN_ARGS_PARAM.to_string());

        for target in [
            "macos-aarch64",
            "ios-arm64",
            "ios-sim-arm64",
            "linux-aarch64",
            "linux-x86_64",
        ] {
            let mut emitter = Emitter::new(
                crate::codegen_support::platform::Target::parse(target).unwrap(),
            );
            emit_method_adapter(
                &mut emitter,
                "_test_source_adapter",
                "_test_physical_method",
                MethodKind::Instance,
                &source,
                &physical,
                false,
            )
            .unwrap();
            let asm = emitter.output();
            let spill = asm.find("param $source_method_arg").unwrap();
            let allocation = asm.find("__rt_array_new").unwrap();
            let publish = asm.find("__rt_cleanup_call_operand_owner").unwrap();
            let call = asm.find("_test_physical_method").unwrap();
            let detach = asm.rfind("_exc_call_frame_top").unwrap();
            let release = asm.find("__rt_decref_any").unwrap();
            assert!(spill < allocation, "{target}: inputs must be spilled before allocation");
            assert!(allocation < publish, "{target}: allocation must precede publication");
            assert!(publish < call, "{target}: owner must be visible before entering PHP");
            assert!(call < detach, "{target}: normal return must detach the cleanup record");
            assert!(detach < release, "{target}: detach must precede normal decref");
            assert!(asm.contains("__rt_array_stamp_7"), "{target}: {asm}");

            let mut wide_source = signature(
                (0..10)
                    .map(|index| (format!("arg{index}"), PhpType::Int))
                    .collect(),
                None,
            );
            wide_source.return_type = PhpType::Int;
            let mut wide_physical = wide_source.clone();
            wide_physical.params.push((
                crate::func_args::HIDDEN_ARGS_PARAM.to_string(),
                PhpType::Array(Box::new(PhpType::Mixed)),
            ));
            wide_physical.param_type_exprs.push(None);
            wide_physical.param_attributes.push(Vec::new());
            wide_physical.defaults.push(None);
            wide_physical.ref_params.push(false);
            wide_physical.declared_params.push(false);
            wide_physical.variadic = Some(crate::func_args::HIDDEN_ARGS_PARAM.to_string());
            let mut wide_emitter = Emitter::new(
                crate::codegen_support::platform::Target::parse(target).unwrap(),
            );
            emit_method_adapter(
                &mut wide_emitter,
                "_test_static_source_adapter",
                "_test_static_physical",
                MethodKind::Static,
                &wide_source,
                &wide_physical,
                false,
            )
            .unwrap();
            let wide_asm = wide_emitter.output();
            assert!(wide_asm.contains("from caller stack"), "{target}: {wide_asm}");
            assert!(wide_asm.contains("_test_static_physical"), "{target}: {wide_asm}");
        }
    }
}
