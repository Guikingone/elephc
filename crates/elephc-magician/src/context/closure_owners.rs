//! Purpose:
//! Tracks the runtime objects that keep eval closure capture metadata usable.
//!
//! Called from:
//! - Closure/Reflection materialization and final object metadata retirement.
//!
//! Key details:
//! - Runtime object edges own cells; metadata only borrows those cells.
//! - Bound copies share a definition, including across leased eval contexts.
//! - The final owner removes the definition before its cell pointers can be reused.

use super::*;

impl ElephcEvalContext {
    /// Collects capture edges from the context that owns the callable definition.
    pub(crate) fn closure_target_captures(&self, target: &EvalClosureObjectTarget) -> Vec<RuntimeCellHandle> {
        match target {
            EvalClosureObjectTarget::ForeignContext { target, owner } => {
                // The lease keeps the pointed-to context alive for this metadata read.
                unsafe { owner.context_ptr().as_ref() }
                    .map(|context| context.closure_target_captures(target)).unwrap_or_default()
            }
            EvalClosureObjectTarget::Named(name) | EvalClosureObjectTarget::BoundNamed { name, .. } => {
                self.closures.get(name).map(|closure| {
                    closure.captures().iter().map(EvalClosureCaptureBinding::value).collect()
                }).unwrap_or_default()
            }
            _ => Vec::new(),
        }
    }

    /// Records one successfully attached runtime object without acquiring extra GC roots.
    pub(crate) fn retain_closure_target_metadata(&mut self, target: &EvalClosureObjectTarget) {
        match target {
            EvalClosureObjectTarget::ForeignContext { target, owner } => {
                // Foreign wrappers keep their owner lease until object retirement finishes.
                if let Some(context) = unsafe { owner.context_ptr().as_mut() } {
                    context.retain_closure_target_metadata(target);
                }
            }
            EvalClosureObjectTarget::Named(name) | EvalClosureObjectTarget::BoundNamed { name, .. } => {
                if let Some(closure) = self.closures.get_mut(name) { closure.runtime_owners += 1; }
            }
            _ => {}
        }
    }

    /// Retires a definition only when all Closure and Reflection runtime owners are gone.
    pub(crate) fn release_closure_target_metadata(&mut self, target: &EvalClosureObjectTarget) {
        match target {
            EvalClosureObjectTarget::ForeignContext { target, owner } => {
                if let Some(context) = unsafe { owner.context_ptr().as_mut() } {
                    context.release_closure_target_metadata(target);
                }
            }
            EvalClosureObjectTarget::Named(name) | EvalClosureObjectTarget::BoundNamed { name, .. } => {
                if let Some(closure) = self.closures.get_mut(name) {
                    if closure.runtime_owners == 0 { return; }
                    closure.runtime_owners -= 1;
                    if closure.runtime_owners == 0 { self.closures.remove(name); }
                }
            }
            _ => {}
        }
    }

    /// Detaches callable metadata before child destructors can reenter eval; leases remain guarded.
    pub(crate) fn take_runtime_closure_metadata(&mut self, identity: u64) -> Vec<EvalClosureObjectTarget> {
        let targets = self.closure_objects.remove(&identity).into_iter()
            .chain(self.eval_reflection_function_closure_targets.remove(&identity).map(|metadata| metadata.target))
            .collect::<Vec<_>>();
        for target in &targets { self.release_closure_target_metadata(target); }
        if !targets.is_empty() { self.eval_reflection_functions.remove(&identity); }
        targets
    }

    /// Removes a literal definition whose runtime object could not be constructed.
    pub(crate) fn discard_unowned_closure(&mut self, name: &str) {
        if self.closures.get(name).is_some_and(|closure| closure.runtime_owners == 0) {
            self.closures.remove(name);
        }
    }
}
