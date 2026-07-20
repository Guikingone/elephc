//! Purpose:
//! Coerces literal string-callable call arguments (e.g. `'someFunc'`) at `callable`-typed
//! regular-parameter positions into their first-class-callable AST equivalent
//! (`ExprKind::FirstClassCallable`), so a call like `register_shutdown_function('someFunc')`
//! reaches EIR lowering as the SAME node shape as writing `someFunc(...)` explicitly, instead of
//! materializing a raw string where the ABI expects a callable descriptor.
//!
//! Called from:
//! - `crate::pipeline::compile()` via `crate::optimize::coerce_callable_string_args` (after type
//!   checking, alongside `fold_class_existence`/`fold_function_existence`, before
//!   `prune_constant_control_flow`), plus `coerce_callable_string_args_in_method_bodies` for
//!   class/enum method bodies.
//! - `crate::optimize::fold::expr::fold_expr` calls `try_coerce_callable_string_args` on each
//!   `FunctionCall`.
//!
//! Key details:
//! - The type checker
//!   (`crate::types::checker::functions::call_validation::Checker::coerce_callable_string_args`)
//!   ACCEPTS the same coercion at type-check time using an ephemeral, checker-local copy of the
//!   call's normalized arguments — enough to validate the call — but the checker never mutates
//!   the real program AST (`crate::types::check` takes `&Program`, not `&mut Program`). THIS pass
//!   performs the equivalent rewrite on the REAL AST afterward, informed by the completed
//!   `CheckResult`, so `crate::ir_lower` (which re-walks the real AST, not the checker's copy)
//!   sees the coerced node too. Both sides must stay in lockstep on what they accept/rewrite —
//!   see the scope list below, which both implementations share.
//! - Scope is intentionally narrow (JURY ADDENDUM #2 on the N1 checker-bundle spec — explicit
//!   decisions, documented, not silent):
//!   - Only a LITERAL string naming a KNOWN function (user-declared, `extern`, or a builtin with
//!     a `first_class_callable_builtin_sig`) is coerced. A non-literal string (e.g. a variable
//!     holding a name) is left alone: dynamic name resolution is out of scope.
//!   - Only PLAIN POSITIONAL calls (no named arguments, no spread) are coerced. Matching the
//!     `plan_call_args` planner's positional-index assumption for a named/spread call would
//!     require reconstructing the call's args from `CallArgPlan::normalized_args()` this early in
//!     the pipeline, which risks changing call shape in ways not yet proven safe; left loud
//!     (unchanged — the ordinary "expects Callable, got Str" diagnostic still fires) instead.
//!   - `'Class::method'` static-method strings and `[$obj, 'method']` array-form callables are
//!     NOT coerced this cycle: static-method-string visibility depends on the CALLING scope
//!     (which this seam does not have), and array-form resolution needs the receiver's runtime
//!     type — neither drops out trivially from this seam. Left loud (unchanged).
//!   - Only free-FUNCTION calls (`ExprKind::FunctionCall`) are covered — direct METHOD calls with
//!     a `callable`-typed parameter are NOT coerced (would need receiver-type-aware method
//!     signature resolution, which this fold-level pass does not have). Left loud (unchanged).

use std::cell::RefCell;
use std::collections::HashMap;

use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{CallableTarget, Expr, ExprKind, Program};
use crate::types::{CheckResult, PhpType};

thread_local! {
    /// Callable-typed-parameter snapshot installed for the duration of
    /// `coerce_callable_string_args`. `None` outside that pass, so unrelated folds never touch
    /// these calls.
    static ACTIVE_CALLABLE_COERCION: RefCell<Option<CallableCoercionSet>> = const { RefCell::new(None) };
}

/// Closed-world snapshot built once from a completed `CheckResult`, used to resolve both sides of
/// the coercion: which CALLEE parameter positions are `callable`-typed, and which literal TARGET
/// names are known functions elephc's first-class-callable machinery can dispatch.
#[derive(Clone)]
pub struct CallableCoercionSet {
    /// Callee name (case-insensitive PHP symbol key) -> regular-parameter indices typed
    /// `callable` (or a union containing it).
    callable_param_positions: HashMap<String, Vec<usize>>,
    /// Case-insensitive PHP symbol key -> canonical declared name, for every user-declared
    /// function and `extern` declaration. Builtins are resolved separately (a static catalog,
    /// not tied to a particular `CheckResult`).
    known_target_names: HashMap<String, String>,
}

impl CallableCoercionSet {
    /// Builds the snapshot from every user-declared function in `check.functions` (methods are
    /// intentionally excluded — see module docs) plus `check.extern_functions`.
    pub fn from_check_result(check: &CheckResult) -> Self {
        let mut callable_param_positions = HashMap::new();
        for (name, sig) in &check.functions {
            let regular_param_count = crate::types::call_args::regular_param_count(sig);
            let positions: Vec<usize> = sig.params[..regular_param_count]
                .iter()
                .enumerate()
                .filter(|(_, (_, ty))| type_accepts_callable(ty))
                .map(|(idx, _)| idx)
                .collect();
            if !positions.is_empty() {
                callable_param_positions.insert(php_symbol_key(name), positions);
            }
        }
        let mut known_target_names = HashMap::new();
        for name in check.functions.keys().chain(check.extern_functions.keys()) {
            known_target_names.insert(php_symbol_key(name), name.clone());
        }
        Self {
            callable_param_positions,
            known_target_names,
        }
    }

    /// Resolves `name` (case-insensitively) to a function elephc's first-class-callable
    /// machinery already knows how to target: a user-declared function, an `extern`
    /// declaration, or a builtin with a `first_class_callable_builtin_sig`. Returns the
    /// canonical name to embed in the coerced `FirstClassCallable` node, or `None` if `name`
    /// does not resolve to anything usable.
    fn coercible_callable_string_target(&self, name: &str) -> Option<String> {
        if let Some(canonical) = self.known_target_names.get(&php_symbol_key(name)) {
            return Some(canonical.clone());
        }
        if crate::name_resolver::is_builtin_function(name) {
            let canonical = crate::types::checker::builtins::canonical_builtin_function_name(name)?;
            if crate::types::first_class_callable_builtin_sig(&canonical).is_some() {
                return Some(canonical);
            }
        }
        None
    }
}

/// Returns `true` if `ty` is `PhpType::Callable` or a `Union` containing `Callable` as a member
/// (e.g. a nullable `?callable` parameter). Mirrors
/// `crate::types::checker::functions::call_validation`'s identically-named helper (kept separate
/// rather than shared across the checker/optimizer module boundary, matching this file's other
/// deliberately-duplicated resolution logic — see module docs).
fn type_accepts_callable(ty: &PhpType) -> bool {
    match ty {
        PhpType::Callable => true,
        PhpType::Union(members) => members.iter().any(type_accepts_callable),
        _ => false,
    }
}

/// Coerces string-callable arguments across `program` and returns the rewritten AST. Installs
/// `set` in the thread-local slot for the duration of the pass, then runs the shared constant-
/// folding driver so every plain-positional `FunctionCall` with a literal string at a
/// `callable`-typed parameter position is rewritten, before restoring the previous slot. Covers
/// top-level statements and function bodies; class/enum method bodies are covered separately by
/// `coerce_callable_string_args_in_method_bodies` (mirrors `fold_class_existence`'s split).
pub fn coerce_callable_string_args(program: Program, set: &CallableCoercionSet) -> Program {
    ACTIVE_CALLABLE_COERCION.with(|slot| {
        let previous = slot.replace(Some(set.clone()));
        let result = super::control::fold_block(program);
        slot.replace(previous);
        result
    })
}

/// Coerces string-callable arguments in every class/enum method body stored in `check`. EIR
/// lowering re-sources method bodies from `check.classes[..].method_decls`, not the optimized
/// program AST, so a free-function call made FROM inside a method body (e.g.
/// `register_shutdown_function(...)` called from a constructor) would otherwise reach codegen
/// unfolded. Mirrors `fold_class_existence_in_method_bodies`. Method CALL SITES themselves stay
/// out of scope for this feature — see module docs.
pub fn coerce_callable_string_args_in_method_bodies(check: &mut CheckResult, set: &CallableCoercionSet) {
    ACTIVE_CALLABLE_COERCION.with(|slot| {
        let previous = slot.replace(Some(set.clone()));
        for class_info in check.classes.values_mut() {
            for method in class_info.method_decls.iter_mut() {
                let body = std::mem::take(&mut method.body);
                method.body = super::control::fold_block(body);
            }
        }
        slot.replace(previous);
    });
}

/// Attempts to coerce a `FunctionCall`'s string-callable arguments. Returns `None` (leaving the
/// call intact) when the coercion set is not installed, the callee has no known `callable`-typed
/// parameter, the call uses named/spread arguments (see module docs), or no argument at a
/// `callable`-typed position is a literal string naming a known function.
pub(in crate::optimize) fn try_coerce_callable_string_args(name: &Name, args: &[Expr]) -> Option<ExprKind> {
    ACTIVE_CALLABLE_COERCION.with(|slot| {
        let borrowed = slot.borrow();
        let set = borrowed.as_ref()?;
        let key = php_symbol_key(name.as_str().trim_start_matches('\\'));
        let positions = set.callable_param_positions.get(&key)?;
        if args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. } | ExprKind::Spread(_)))
        {
            return None;
        }
        let mut new_args = args.to_vec();
        let mut changed = false;
        for &idx in positions {
            let Some(arg) = new_args.get_mut(idx) else {
                continue;
            };
            let ExprKind::StringLiteral(target_name) = &arg.kind else {
                continue;
            };
            let Some(canonical) = set.coercible_callable_string_target(target_name) else {
                continue;
            };
            arg.kind = ExprKind::FirstClassCallable(CallableTarget::Function(Name::unqualified(
                canonical,
            )));
            changed = true;
        }
        changed.then(|| ExprKind::FunctionCall {
            name: name.clone(),
            args: new_args,
        })
    })
}
