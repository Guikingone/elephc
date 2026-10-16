//! Purpose:
//! Proves and publishes the local owner transaction for consuming string append.
//!
//! Called from:
//! - `stmt::string_append::try_lower` for eligible plain string assignments.
//!
//! Key details:
//! - Parameters, captures, globals, reference cells and eval-visible slots cannot be consumed.
//! - Publication moves the runtime result directly, without another acquire or old-owner release.

use super::*;

impl LoweringContext<'_, '_> {
    /// Proves an initialized local can hand its string owner to a consuming append.
    pub(crate) fn consuming_string_slot(&self, name: &str, span: Span) -> Option<LocalSlotId> {
        let slot = *self.local_slots.get(name)?;
        let local = self.builder.function().locals.get(slot.as_raw() as usize)?;
        (local.kind == LocalKind::PhpLocal && local.ir_type == IrType::Str
            && local.php_type == PhpType::Str && self.local_type(name) == PhpType::Str
            && !self.builder.function().params.iter().any(|param| param.name == name)
            && self.slot_is_initialized(slot) && !self.local_uses_global_storage(name)
            && !self.extern_globals.contains_key(name) && !self.is_ref_bound_local(name)
            && !self.has_eval_barrier() && !self.eval_executed()
            && !self.should_store_to_eval_scope(name)
            && !self.is_recorded_retype_site(span, name)
            && !self.is_recorded_mixed_storage_site(span, name)).then_some(slot)
    }

    /// Installs an append's replacement owner without acquiring or retiring the consumed owner.
    pub(crate) fn store_consumed_string(
        &mut self, name: &str, slot: LocalSlotId, value: LoweredValue, span: Span,
    ) {
        self.clear_static_callable_local(name);
        self.clear_reflection_class_local(name);
        self.clear_reflection_function_local(name);
        self.clear_reflection_property_local(name);
        self.clear_reflection_method_local(name);
        self.clear_reflection_arg_array_local(name);
        self.clear_fiber_start_sig(name);
        self.store_slot_with_op(slot, value, Op::StoreLocal, Some(span));
        self.set_local_type(name, PhpType::Str);
    }

}
