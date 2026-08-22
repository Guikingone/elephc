//! Purpose:
//! Carries the checker's span-keyed local-binding decisions (kill / retype sites) into the
//! post-typecheck AST passes, so those passes never break the one invariant the decisions rest on:
//! ONE decision span names ONE syntactic site.
//!
//! Called from:
//! - `crate::pipeline`, which installs the sets around post-typecheck optimization.
//! - `crate::optimize::control::dce`, whose tail-sinking is the pass that would otherwise
//!   duplicate a decision-carrying statement.
//!
//! Key details:
//! - The checker records `local_bind_kill_sites` / `local_retype_sites` BY SPAN, and EIR lowering
//!   consults them by span at every `unset` argument and every assignment. A pass that clones an
//!   AST node clones its span, so both copies then answer to the same decision — and abandoning a
//!   binding is not idempotent (it releases the old value and re-binds the name to a fresh slot),
//!   so the second copy is lowered against the FIRST copy's post-rebind state.
//! - Installed as a scoped thread-local, matching `with_callable_effect_analysis` and
//!   `with_by_ref_signatures`. Empty by default, which makes every query a cheap `is_empty()`.

use std::cell::RefCell;
use std::collections::HashSet;

use crate::span::Span;

thread_local! {
    /// Spans carrying a checker local-binding decision, for the duration of one optimizer run.
    static ACTIVE_BINDING_DECISION_SPANS: RefCell<HashSet<Span>> = RefCell::new(HashSet::new());
}

/// Installs `spans` as the active local-binding decision set for the duration of `f`.
pub(crate) fn with_local_binding_decision_spans<R>(spans: HashSet<Span>, f: impl FnOnce() -> R) -> R {
    ACTIVE_BINDING_DECISION_SPANS.with(|slot| {
        let previous = slot.replace(spans);
        let result = f();
        slot.replace(previous);
        result
    })
}

/// Returns whether any local-binding decision is in play at all.
///
/// Almost every program answers `false` here (a kill or retype is rare), which is what keeps the
/// scan below off the hot path entirely.
pub(crate) fn has_local_binding_decisions() -> bool {
    ACTIVE_BINDING_DECISION_SPANS.with(|slot| !slot.borrow().is_empty())
}

/// Returns whether `span` carries a checker local-binding decision.
pub(crate) fn span_carries_local_binding_decision(span: Span) -> bool {
    ACTIVE_BINDING_DECISION_SPANS.with(|slot| slot.borrow().contains(&span))
}
