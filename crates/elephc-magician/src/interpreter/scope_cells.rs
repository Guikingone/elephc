//! Purpose:
//! Provides scope-cell read, write, unset, global-alias, and reference-alias helpers for eval execution.
//!
//! Called from:
//! - `crate::interpreter::statements` and `crate::interpreter::expressions`.
//!
//! Key details:
//! - Global aliases redirect through `ElephcEvalContext` while local aliases stay in the materialized eval scope.
//! - Replaced owned cells are released by callers through existing scope APIs.
//! - Php's request superglobals behave like an automatic `global $name;` in EVERY scope, on every
//!   mention -- unlike an explicit `global` statement, this is not a one-time bind: a name in
//!   `REQUEST_SUPERGLOBAL_NAMES` redirects to the context's global scope even where no `global`
//!   statement was ever written and even after a local `unset()` of that name in this scope.

use super::*;

/// The eight request superglobals php auto-globalizes in every scope without a `global`
/// statement. `$GLOBALS` is handled separately (`globals::is_globals_array`) because it is a
/// pseudo-array rather than an ordinary variable name.
pub(in crate::interpreter) const REQUEST_SUPERGLOBAL_NAMES: [&str; 8] = [
    "_SERVER", "_ENV", "_GET", "_POST", "_COOKIE", "_FILES", "_REQUEST", "_SESSION",
];

/// Returns true when `name` is one of php's automatically global request superglobals.
pub(in crate::interpreter) fn is_request_superglobal_name(name: &str) -> bool {
    REQUEST_SUPERGLOBAL_NAMES.contains(&name)
}

/// Returns the effective global-alias target for a name: an explicit `global` alias first, then
/// a request superglobal's own name, matching php's implicit per-mention auto-global fetch.
fn effective_global_alias_target<'a>(scope: &'a ElephcEvalScope, name: &'a str) -> Option<&'a str> {
    scope
        .global_alias_target(name)
        .or_else(|| is_request_superglobal_name(name).then_some(name))
}

/// Returns the eval-visible entry for a variable, following `global` aliases.
pub(in crate::interpreter) fn scope_entry(
    context: &ElephcEvalContext,
    scope: &ElephcEvalScope,
    name: &str,
) -> Option<ScopeEntry> {
    // A `static` reads from its slot every time, not from a copy taken when the activation
    // started: php keeps ONE slot per function, so a frame must see what a deeper frame wrote.
    if let Some(slot) = scope.static_alias_slot(name) {
        let slot = slot.to_string();
        // BORROWED, whatever the slot's own flags say. The slot OWNS the cell and keeps it after
        // the activation ends; the activation only borrows it. Handing the slot's entry back
        // verbatim reported `Owned`, so the activation's cleanup released a cell the slot still
        // held -- which the counting fixture catches as `reached count -1`.
        return unsafe { context.static_scope_ptr().as_ref() }
            .and_then(|statics| statics.visible_cell(&slot))
            .map(|cell| ScopeEntry::present(cell, ScopeCellOwnership::Borrowed, 0));
    }
    let Some(global_name) = effective_global_alias_target(scope, name) else {
        return scope.entry(name);
    };
    let Some(global_scope) = context.global_scope_ptr() else {
        return scope.entry(name);
    };
    let current_scope = scope as *const ElephcEvalScope as *mut ElephcEvalScope;
    if global_scope == current_scope {
        return scope.entry(global_name);
    }
    unsafe {
        global_scope
            .as_ref()
            .and_then(|scope| scope.entry(global_name))
    }
}

/// Returns the eval-visible cell for a variable, following `global` aliases.
pub(in crate::interpreter) fn visible_scope_cell(
    context: &ElephcEvalContext,
    scope: &ElephcEvalScope,
    name: &str,
) -> Option<RuntimeCellHandle> {
    scope_entry(context, scope, name)
        .filter(|entry| entry.flags().is_visible())
        .map(ScopeEntry::cell)
}

/// Stores a variable cell, redirecting `global` aliases to the global scope.
pub(in crate::interpreter) fn set_scope_cell(
    context: &ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    name: impl Into<String>,
    cell: RuntimeCellHandle,
    ownership: ScopeCellOwnership,
) -> Result<Vec<RuntimeCellHandle>, EvalStatus> {
    let name = name.into();
    // A write to a `static` lands in the slot, so the next read -- in this frame, in a recursive
    // one, or in a later call -- sees it. The displaced cell is handed back for release exactly
    // as the scope would have done.
    if let Some(slot) = scope.static_alias_slot(&name).map(str::to_string) {
        let Some(statics) = (unsafe { context.static_scope_ptr().as_mut() }) else {
            return Err(EvalStatus::RuntimeFatal);
        };
        return Ok(statics.set_respecting_references(slot, cell, ownership));
    }
    if let Some(global_name) = effective_global_alias_target(scope, &name).map(str::to_string) {
        let Some(global_scope) = context.global_scope_ptr() else {
            // An explicit `global $x;` already validated a global scope exists before it could
            // ever mark this alias (see `execute_global_stmt`), so reaching here with no scope
            // pointer is a genuine fatal for it. A superglobal name carries no such guarantee --
            // nothing declared it -- so with no established global scope at all it degrades to an
            // ordinary local write rather than failing a statement php would never refuse.
            return if scope.global_alias_target(&name).is_some() {
                Err(EvalStatus::RuntimeFatal)
            } else {
                Ok(scope.set(name, cell, ownership).into_iter().collect())
            };
        };
        let current_scope = scope as *mut ElephcEvalScope;
        if global_scope == current_scope {
            return Ok(scope.set_respecting_references(global_name, cell, ownership));
        }
        let Some(global_scope) = (unsafe { global_scope.as_mut() }) else {
            return Err(EvalStatus::RuntimeFatal);
        };
        return Ok(global_scope.set_respecting_references(global_name, cell, ownership));
    }
    Ok(scope.set_respecting_references(name, cell, ownership))
}

/// Creates a PHP reference alias between two eval-visible variable names.
pub(in crate::interpreter) fn set_reference_alias(
    context: &ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    target: &str,
    source: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<Vec<RuntimeCellHandle>, EvalStatus> {
    let source_reference_target = scope.reference_target(source).cloned();
    if let Some(global_name) = scope.global_alias_target(source).map(str::to_string) {
        scope.mark_global_alias_to(target.to_string(), global_name);
        return Ok(Vec::new());
    }
    let (cell, ownership) = scope_entry(context, scope, source)
        .filter(|entry| entry.flags().is_visible())
        .map_or_else(
            || values.null().map(|cell| (cell, ScopeCellOwnership::Owned)),
            |entry| Ok((entry.cell(), entry.flags().ownership)),
        )?;
    let replaced = scope.set_reference(target.to_string(), source.to_string(), cell, ownership);
    if let Some(source_reference_target) = source_reference_target {
        scope.set_reference_target(target.to_string(), source_reference_target);
    } else {
        scope.remove_reference_target(target);
    }
    Ok(replaced)
}

/// Unsets a variable, removing only the local alias when the name is global.
pub(in crate::interpreter) fn unset_scope_cell(
    scope: &mut ElephcEvalScope,
    name: impl Into<String>,
) -> Option<RuntimeCellHandle> {
    let name = name.into();
    if scope.is_global_alias(&name) {
        scope.clear_global_alias(&name);
    }
    // `unset($x)` on a static breaks the local binding, exactly as it does for a global; the slot
    // itself keeps its value for the next call, which is what php does.
    scope.clear_static_alias(&name);
    scope.unset_respecting_references(name)
}

/// Marks variables as aliases to the context global scope for later reads/writes.
pub(in crate::interpreter) fn execute_global_stmt(
    vars: &[String],
    context: &ElephcEvalContext,
    scope: &mut ElephcEvalScope,
) -> Result<(), EvalStatus> {
    if context.global_scope_ptr().is_none() {
        return Err(EvalStatus::RuntimeFatal);
    }
    for name in vars {
        scope.mark_global_alias(name.clone());
    }
    Ok(())
}
