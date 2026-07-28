//! Purpose:
//! Flow-sensitive type narrowing for `if`/`else` branches guarded by type predicates or truthiness.
//! Narrows a union- or mixed-typed variable to the guarded type in the matching branch.
//!
//! Called from:
//! - `crate::types::checker::stmt_check::control_flow` when checking `StmtKind::If` and
//!   `switch (true)` case bodies.
//!
//! Key details:
//! - Recognizes `is_int`/`is_float`/`is_string`/`is_bool`/`is_array`/`is_object`/`is_numeric`/
//!   `is_countable($var)` (and aliases), `$var instanceof Class`, and strict false/null guards
//!   around a simple local assignment, optionally negated with a leading `!`. A local or simple
//!   local assignment truthiness guard removes only representable `null`/`false` union arms on
//!   its true edge, leaving scalar zero/empty-value possibilities conservative on its false edge.
//!   An assignment's inferred value type replaces the prior binding on both edges. `is_object`
//!   narrows to generic `object`; `is_numeric` preserves numeric strings through an
//!   `int|float|string` arithmetic-safe union; `is_countable` narrows to
//!   `array<mixed>|Countable`.
//!   Narrowing is applied to each clause in an
//!   if/elseif*/else chain (each subsequent clause, and the else, see the accumulated complement
//!   from previous guards). For a chain with no else where *every* clause body cannot fall through
//!   to the following statement — via `src/termination.rs`'s structural analysis
//!   (return/throw/break/continue/exit/die, statically infinite loops, nested if/switch/try whose
//!   branches all terminate, or a terminal statement before unreachable code), extended
//!   recursively with checker-known `never` calls — the accumulated complement is applied to the
//!   statements after the entire if construct.
//! - `and_chain_then_narrowings` collects guard-true narrowings and assignment facts for a
//!   single condition or pure `&&` chain, folding repeated guards cumulatively and replacing
//!   earlier facts on a later assignment. It powers `if` and `switch (true)` bodies.
//! - Conservative: a concrete (non-union, non-mixed) type is left unchanged, and an empty narrowing
//!   result falls back to the original type, so valid code is never narrowed away to `Never`.

use crate::errors::CompileError;
use crate::names::{php_symbol_key, property_hook_get_method};
use crate::parser::ast::{BinOp, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind};
use crate::termination::{block_terminal_effect_with_divergence, TerminalEffect};
use crate::types::{PhpType, TypeEnv};

use super::super::Checker;

/// A detected type-guard narrowing: the guarded binding's env key and the types it takes in the
/// then-branch (guard true) and else-branch (guard false).
pub(crate) struct GuardNarrowing {
    /// `TypeEnv` key of the guarded binding: a variable name (without the leading `$`) or the
    /// synthetic property key from `narrowed_property_env_key`.
    pub var: String,
    /// Type the binding has where the guard is true.
    pub then_ty: PhpType,
    /// Type the binding has where the guard is false.
    pub else_ty: PhpType,
    /// Extra `TypeEnv` keys that hold the SAME value as `var` on branch entry and must receive the
    /// identical then/else narrowing. Populated for an assignment receiver `$f = $src` whose source
    /// is a plain variable: right after `$f = $src` the two locals alias one value, so a guard on
    /// `$f` (`!is_array($f = $src)`) narrows `$src` in each branch too. This is a branch-entry fact;
    /// a later reassignment of either local overwrites it normally. Empty for every other receiver
    /// shape, so ordinary guards are unaffected.
    pub aliases: Vec<String>,
}

impl Checker {
    /// Detects a type-predicate guard in an `if`/ternary condition and computes the then/else
    /// narrowing for the guarded binding against the current environment. Handles the scalar
    /// `is_*` predicates, `is_null`, `instanceof Class`, and `=== false` / `=== null`, each with an
    /// optional leading `!` that swaps the branches. A bare local condition also narrows away
    /// representable `null` and literal `false` arms on its truthy edge. The guarded receiver may be
    /// a variable, a
    /// simple local assignment such as `false === $parts = parse_url($dsn)`, or a simple property
    /// access `$var->prop` / `$this->prop` (narrowed under a synthetic key that
    /// `infer_property_access_type` consults). Returns `Ok(None)` when the condition is not a
    /// recognized guard or the receiver's current type is unknown.
    pub(crate) fn guard_narrowing(
        &mut self,
        condition: &Expr,
        env: &TypeEnv,
    ) -> Result<Option<GuardNarrowing>, CompileError> {
        let (cond, negated) = match &condition.kind {
            ExprKind::Not(inner) => (inner.as_ref(), true),
            _ => (condition, false),
        };
        if let Some(narrowing) = self.truthy_binding_guard_narrowing(cond, negated, env)? {
            return Ok(Some(narrowing));
        }
        let Some((receiver, target, guard_negated)) = guard_receiver_and_type(cond) else {
            return Ok(None);
        };
        let negated = negated ^ guard_negated;
        // Resolve a relative `instanceof self`/`parent`/`static` target to the concrete enclosing
        // class before narrowing, mirroring `type_guard_narrowing`. Without this the receiver was
        // narrowed to the literal `Object("self")`, and a later member access reported
        // "Undefined class: self". `guard_receiver_and_type` deliberately keeps the raw name so this
        // resolution stays in one place.
        let target = self.resolve_relative_instanceof_target(target);
        let Some(key) = Self::guard_env_key(receiver) else {
            return Ok(None);
        };
        if self.property_guard_receiver_is_unstable(receiver, env)? {
            return Ok(None);
        }
        // For a comparison-wrapped assignment, narrow the value just assigned rather than the
        // local's storage-wide join with its previous values. In
        // `false === $parts = parse_url($dsn)`, the assignment result is `array|false` even when
        // `$parts` previously held the input string.
        let current = match &receiver.kind {
            ExprKind::Assignment {
                value,
                result_target: None,
                prelude,
                conditional_value_temp: None,
                ..
            } if prelude.is_empty() => self.infer_type(value, env)?,
            // A prior narrowing (or a variable binding) wins; otherwise a property or `$this`
            // receiver falls back to its declared type (the enclosing class for `$this`). An
            // unbound plain variable stays un-narrowed.
            _ => match env.get(&key) {
                Some(ty) => ty.clone(),
                None if matches!(
                    receiver.kind,
                    ExprKind::PropertyAccess { .. } | ExprKind::This
                ) =>
                {
                    self.infer_type(receiver, env)?
                }
                None => return Ok(None),
            },
        };
        let matched = self.narrow_to(&current, &target);
        let complement = self.narrow_complement(&current, &target);
        let (then_ty, else_ty) = if negated {
            (complement, matched)
        } else {
            (matched, complement)
        };
        // When the receiver is a simple `$f = $src` assignment whose source is a plain variable,
        // `$f` and `$src` alias the same value on branch entry, so the narrowing applies to both.
        let aliases = match &receiver.kind {
            ExprKind::Assignment {
                value,
                result_target: None,
                prelude,
                conditional_value_temp: None,
                ..
            } if prelude.is_empty() => match &value.kind {
                ExprKind::Variable(src) => vec![src.clone()],
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        Ok(Some(GuardNarrowing { var: key, then_ty, else_ty, aliases }))
    }

    /// Narrows a local binding on its truthy edge by removing only union arms whose complete value
    /// space is falsey (`Void` for null and the literal `False` subtype). The binding may be read
    /// directly or be the target of a simple assignment in the condition. An assignment replaces
    /// the prior local type on both edges because PHP executes it before testing truthiness; this
    /// remains useful even when an empty array/string subset cannot be represented more narrowly.
    /// Integer zero, empty strings/arrays, and the false half of `Bool` stay conservative. A leading
    /// logical `!` swaps the truthy and falsey branch facts.
    fn truthy_binding_guard_narrowing(
        &self,
        condition: &Expr,
        negated: bool,
        env: &TypeEnv,
    ) -> Result<Option<GuardNarrowing>, CompileError> {
        let (var, current, overwrites) = match &condition.kind {
            ExprKind::Variable(var) => {
                let Some(current) = env.get(var) else {
                    return Ok(None);
                };
                (var.clone(), current.clone(), false)
            }
            ExprKind::Assignment {
                target,
                result_target: None,
                prelude,
                conditional_value_temp: None,
                ..
            } if prelude.is_empty() => {
                let ExprKind::Variable(var) = &target.kind else {
                    return Ok(None);
                };
                let Some(current) = env.get(var) else {
                    return Ok(None);
                };
                (var.clone(), current.clone(), true)
            }
            _ => return Ok(None),
        };
        let truthy = match &current {
            PhpType::Union(members) => {
                let kept: Vec<PhpType> = members
                    .iter()
                    .filter(|member| !matches!(member, PhpType::Void | PhpType::False))
                    .cloned()
                    .collect();
                if kept.is_empty() || kept.len() == members.len() {
                    current.clone()
                } else {
                    self.normalize_union_type(kept)
                }
            }
            _ => current.clone(),
        };
        if !overwrites && truthy == current {
            return Ok(None);
        }
        let (then_ty, else_ty) = if negated {
            (current, truthy)
        } else {
            (truthy, current)
        };
        Ok(Some(GuardNarrowing {
            var,
            then_ty,
            else_ty,
            aliases: Vec::new(),
        }))
    }

    /// Synthetic `TypeEnv` key for a narrowed simple property access `$var->prop` (`None` for a
    /// more complex receiver). The `\x01` sigil bytes cannot appear in a real variable name, so
    /// this key never collides with a variable binding — a normal property read only picks it up
    /// when a narrowing has explicitly inserted it.
    pub(crate) fn narrowed_property_env_key(object: &Expr, property: &str) -> Option<String> {
        match &object.kind {
            ExprKind::Variable(var) => Some(format!("\u{1}prop\u{1}{var}->{property}")),
            ExprKind::This => Some(format!("\u{1}prop\u{1}$this->{property}")),
            _ => None,
        }
    }

    /// Synthetic `TypeEnv` key under which an `instanceof` narrowing of the bare `$this` receiver
    /// is recorded (`if ($this instanceof I) { $this->onlyOnI(); }`). Mirrors
    /// `narrowed_property_env_key`: the leading `\x01` sigil keeps it out of the variable
    /// namespace and lets `purge_property_narrowings` drop it after a potential mutation, while its
    /// distinct `this` segment keeps `purge_property_narrowings_for_root` (which targets
    /// `\x01prop\x01<root>->`) from touching it — an object's class cannot change under a local
    /// rebind. `infer_this_type` consults it so a narrowed `$this` sees the proven subtype.
    pub(crate) fn narrowed_this_env_key() -> &'static str {
        "\u{1}this\u{1}$this"
    }

    /// `TypeEnv` key for a guard receiver: a variable's name, an assignment's local target, or the
    /// synthetic property key for a simple property access. Conditional/compound assignment forms
    /// are safe here because expression-effect inference has already recorded the value stored in
    /// the local before guard detection. `None` for receivers narrowing cannot key safely.
    fn guard_env_key(receiver: &Expr) -> Option<String> {
        match &receiver.kind {
            ExprKind::Variable(var) => Some(var.clone()),
            ExprKind::Assignment { target, .. } => match &target.kind {
                ExprKind::Variable(var) => Some(var.clone()),
                _ => None,
            },
            ExprKind::PropertyAccess { object, property } => {
                Self::narrowed_property_env_key(object, property)
            }
            ExprKind::This => Some(Self::narrowed_this_env_key().to_string()),
            _ => None,
        }
    }

    /// Drops every synthetic property narrowing from the environment. Called after effects that
    /// may write a property (property assignments, any call — a callee can mutate the object),
    /// and at loop-body entry (a later iteration may observe an earlier iteration's write), so a
    /// stale narrowing never survives a potential mutation. Variable narrowings are unaffected —
    /// visible assignments already update those bindings directly.
    pub(crate) fn purge_property_narrowings(env: &mut TypeEnv) {
        env.retain(|key, _| !key.starts_with('\u{1}'));
    }

    /// Drops synthetic property narrowings rooted at one local variable after that local is
    /// rebound. Other receivers remain valid and keep their precision.
    pub(crate) fn purge_property_narrowings_for_root(env: &mut TypeEnv, root: &str) {
        let prefix = format!("\u{1}prop\u{1}{root}->");
        env.retain(|key, _| !key.starts_with(&prefix));
    }

    /// Invalidates the property narrowings a *direct* property write (`$obj->prop = …`) can affect.
    ///
    /// A direct assignment rebinds one object's own `prop` slot; it cannot change what a *different*
    /// syntactic receiver's property refers to, so only the fact keyed on this exact `<root>->prop`
    /// is dropped. Sibling facts such as `$this->pool` therefore keep their precision when the write
    /// targets `$clone->pool` — extending the syntactic scoping `purge_property_narrowings_for_root`
    /// already applies to local rebinds. A target that is not a simple keyable receiver (an array
    /// element or a deeper expression that may alias and mutate a shared value through) falls back to
    /// purging every property fact.
    pub(crate) fn purge_property_narrowings_for_property_write(
        env: &mut TypeEnv,
        object: &Expr,
        property: &str,
    ) {
        match Self::narrowed_property_env_key(object, property) {
            Some(key) => {
                env.remove(&key);
            }
            None => Self::purge_property_narrowings(env),
        }
    }

    /// Returns whether a property guard can invoke user code on either read. Hooked or magic
    /// properties are not stable flow bindings because two reads may produce different values.
    fn property_guard_receiver_is_unstable(
        &mut self,
        receiver: &Expr,
        env: &TypeEnv,
    ) -> Result<bool, CompileError> {
        let ExprKind::PropertyAccess { object, property } = &receiver.kind else {
            return Ok(false);
        };
        let object_ty = self.infer_type(object, env)?;
        let classes = match object_ty {
            PhpType::Object(class) => vec![class],
            PhpType::Union(_) => self.union_object_classes(&object_ty),
            _ => return Ok(false),
        };
        let get_hook = php_symbol_key(&property_hook_get_method(property));
        Ok(classes.iter().any(|class| {
            self.classes.get(class).is_some_and(|info| {
                info.methods.contains_key(&get_hook)
                    || (!info.properties.iter().any(|(name, _)| name == property)
                        && info.methods.contains_key("__get"))
            })
        }))
    }


    /// Collects the guard-true (then) narrowings for a condition that is a single recognized guard
    /// or a pure `&&` chain of guards (`$a instanceof X && is_int($b) && ...`). Recurses only through
    /// `&&`; returns one `(var, then_type)` per distinct guarded variable, folding repeated guards on
    /// the same variable cumulatively (so `$x instanceof A && $x instanceof B` intersects via
    /// `narrow_to`). Returns an empty vector for `||`/mixed/`!`-top-level/non-guard conditions
    /// (conservative — no narrowing rather than an unsound one). The else/complement side is
    /// intentionally not computed: callers narrowing only a guard-true region (a `switch (true)`
    /// case body) do not need it, and the `&&`-chain complement is a union this single-guard helper
    /// must not approximate.
    pub(crate) fn and_chain_then_narrowings(
        &mut self,
        cond: &Expr,
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        self.and_chain_then_facts(cond, env)
            .into_iter()
            .map(|(var, ty, _)| (var, ty))
            .collect()
    }

    /// Collects ordered guard and assignment facts that hold when a pure `&&` condition is true.
    fn and_chain_then_facts(
        &mut self,
        cond: &Expr,
        env: &TypeEnv,
    ) -> Vec<(String, PhpType, bool)> {
        if let ExprKind::BinaryOp {
            left,
            op: BinOp::And,
            right,
        } = &cond.kind
        {
            // Process operands left-to-right, threading the accumulated narrowings. Each leaf's
            // `then_ty` is already narrowed against the declared type in `env`; when two operands
            // guard the same variable, intersect the accumulated type with the new one via
            // `narrow_to` so repeated guards refine cumulatively.
            let mut narrowings = self.and_chain_then_facts(left, env);
            for (var, then_ty, overwrites) in self.and_chain_then_facts(right, env) {
                match narrowings.iter().position(|(v, _, _)| *v == var) {
                    Some(idx) => {
                        if overwrites {
                            narrowings[idx].1 = then_ty;
                            narrowings[idx].2 = true;
                        } else {
                            let existing = narrowings[idx].1.clone();
                            narrowings[idx].1 = self.narrow_to(&existing, &then_ty);
                        }
                    }
                    None => narrowings.push((var, then_ty, overwrites)),
                }
            }
            narrowings
        } else {
            if let ExprKind::Assignment {
                target,
                value,
                result_target: None,
                prelude,
                conditional_value_temp: None,
                ..
            } = &cond.kind
            {
                if prelude.is_empty() {
                    if let ExprKind::Variable(var) = &target.kind {
                        if let Ok(ty) = self.infer_type(value, env) {
                            return vec![(var.clone(), ty, true)];
                        }
                    }
                }
            }
            match self.guard_narrowing(cond, env) {
                Ok(Some(g)) => vec![(g.var, g.then_ty, false)],
                Ok(None) | Err(_) => vec![],
            }
        }
    }

    /// Collects the receiver narrowings that hold on the fall-through path of a *diverging*
    /// `if (A || B || …) { <cannot fall through> }`. Reaching the statement after such an `if`
    /// proves the whole `||` condition was false, so De Morgan gives `!A && !B && …`: every
    /// disjunct is false. For each disjunct this takes its guard-FALSE fact (the guard's `else_ty`)
    /// and intersects facts that name the same binding via `narrow_to` (mirroring how a `&&` chain
    /// refines repeated guards). Returns an empty vector when the condition is not a top-level `||`
    /// chain, so a caller may invoke it unconditionally; the caller is responsible for confirming
    /// the guarded body cannot fall through before persisting these facts.
    pub(crate) fn or_chain_complement_narrowings(
        &mut self,
        condition: &Expr,
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        if !matches!(&condition.kind, ExprKind::BinaryOp { op: BinOp::Or, .. }) {
            return Vec::new();
        }
        let mut disjuncts: Vec<&Expr> = Vec::new();
        collect_or_operands(condition, &mut disjuncts);
        let mut narrowings: Vec<(String, PhpType)> = Vec::new();
        for disjunct in disjuncts {
            let Ok(Some(guard)) = self.guard_narrowing(disjunct, env) else {
                continue;
            };
            match narrowings.iter().position(|(v, _)| *v == guard.var) {
                Some(idx) => {
                    let existing = narrowings[idx].1.clone();
                    narrowings[idx].1 = self.narrow_to(&existing, &guard.else_ty);
                }
                None => narrowings.push((guard.var, guard.else_ty)),
            }
        }
        narrowings
    }

    /// Resolves a relative class name (`self`/`static`/`parent`, case-insensitive) inside an
    /// `instanceof` narrowing target to the concrete enclosing class. `self`/`static` map to
    /// `current_class`; `parent` maps to the current class's parent. A target that is not a
    /// relative-name `Object`, or a relative name that cannot be resolved (no class context, or
    /// `parent` on a class with no parent), is returned unchanged so the existing unknown-class
    /// diagnostics still fire downstream. Non-`Object` targets pass through untouched.
    fn resolve_relative_instanceof_target(&self, target: PhpType) -> PhpType {
        let PhpType::Object(class_name) = &target else {
            return target;
        };
        let concrete = match class_name.to_ascii_lowercase().as_str() {
            "self" | "static" => self.current_class.clone(),
            "parent" => self
                .current_class
                .as_ref()
                .and_then(|c| self.classes.get(c))
                .and_then(|ci| ci.parent.clone()),
            _ => return target,
        };
        match concrete {
            Some(name) => PhpType::Object(name),
            None => target,
        }
    }

    /// Narrows `current` to the guard-true type. Inside the branch the guard guarantees the target,
    /// so `Mixed` and any incompatible concrete type become `target`; a `Union` keeps only its
    /// matching members (falling back to `target` if none match); a concrete type already matching
    /// the guard is kept as-is (preserving a more specific class for `instanceof`).
    fn narrow_to(&self, current: &PhpType, target: &PhpType) -> PhpType {
        match current {
            PhpType::Union(members) => {
                let kept: Vec<PhpType> =
                    members.iter().filter(|m| guard_matches(m, target)).cloned().collect();
                if kept.is_empty() {
                    target.clone()
                } else if kept.len() > 1 && matches!(target, PhpType::Array(_)) {
                    // Several array arms can differ only in their inferred element type
                    // (`array<mixed>|array<never>` after `$x ??= []`). The `is_array` proof
                    // guarantees the common array family; collapse to its gradual element type
                    // instead of retaining a union that array builtins reject as non-concrete.
                    target.clone()
                } else {
                    self.normalize_union_type(kept)
                }
            }
            _ if guard_matches(current, target) => current.clone(),
            _ => target.clone(),
        }
    }

    /// Narrows `current` to the subset incompatible with `target` (the guard-false type): a `Union`
    /// drops its matching members, while `Mixed` and concrete types are returned unchanged (the
    /// complement of `Mixed` is not representable). An empty result falls back to `current`.
    fn narrow_complement(&self, current: &PhpType, target: &PhpType) -> PhpType {
        match current {
            PhpType::Union(members) => {
                let kept: Vec<PhpType> =
                    members.iter().filter(|m| !guard_matches(m, target)).cloned().collect();
                if kept.is_empty() {
                    current.clone()
                } else {
                    self.normalize_union_type(kept)
                }
            }
            _ => current.clone(),
        }
    }

    /// Returns true when a statement body cannot fall through to the statement that textually
    /// follows it.
    ///
    /// The structural control-flow analysis in [`crate::termination`] recognizes
    /// `return`/`throw`/`break`/`continue`/`exit`/`die`, statically infinite loops, and nested
    /// `if`/`switch`/`try` forms whose branches all terminate, including a terminal statement placed
    /// before unreachable trailing code. `break`/`continue` count as non-fallthrough here — they
    /// prevent reaching the following statement even though they do not exit the function, which
    /// is exactly the distinction post-guard narrowing needs.
    ///
    /// User functions declared `never` need the checker's function table. The checker supplies that
    /// one semantic predicate to the shared traversal, so it applies at every nested structural
    /// level rather than as a separate shallow scan.
    ///
    /// Used by type narrowing so that an `if (guard) { ... non-fallthrough ... }` (with no else)
    /// allows the statements *after* the if to keep the complement type.
    pub(crate) fn body_cannot_fall_through(&self, body: &[Stmt]) -> bool {
        block_terminal_effect_with_divergence(body, &|expr| {
            self.expr_is_declared_never_call(expr)
        })
            != TerminalEffect::FallsThrough
    }

    /// Returns true when the body's control cannot reach past its last statement.
    ///
    /// A body is considered diverging if its last statement is:
    /// - `return` or `throw`
    /// - a call to `exit()` or `die()`
    /// - a call to a user function whose declared return type is `never`
    ///
    /// This is used by type narrowing so that an `if (guard) { ... diverging ... }` (with no else)
    /// allows the statements *after* the if to be narrowed to the complement type.
    pub(crate) fn body_always_diverges(&self, body: &[Stmt]) -> bool {
        let Some(last) = body.last() else {
            return false;
        };

        match &last.kind {
            StmtKind::Return(_) | StmtKind::Throw(_) => true,
            StmtKind::ExprStmt(expr) => self.expr_always_diverges(expr),
            _ => false,
        }
    }

    /// Returns true when a `switch` case body cannot fall through to the next case: its last
    /// statement is `break`/`continue`/`return`/`throw`, or the body always diverges
    /// (`exit`/`die`/never-returning call). Used to gate `switch (true)` case-body narrowing: a case
    /// is only sound to narrow by its own guard when control cannot reach it by falling through from
    /// a previous, non-terminating case (where the guard may be false at runtime).
    pub(crate) fn case_body_terminates(&self, body: &[Stmt]) -> bool {
        matches!(
            body.last().map(|s| &s.kind),
            Some(
                StmtKind::Break(_)
                    | StmtKind::Continue(_)
                    | StmtKind::Return(_)
                    | StmtKind::Throw(_)
            )
        ) || self.body_always_diverges(body)
    }

    /// Returns true if the expression is known to never return normally: a call to `exit()` or
    /// `die()` (recognized by name), or a call to a user function whose declared return type is
    /// `never`. The function name is resolved case-insensitively against the checker's function
    /// table, matching PHP's call semantics.
    fn expr_always_diverges(&self, expr: &Expr) -> bool {
        let ExprKind::FunctionCall { name, .. } = &expr.kind else {
            return false;
        };
        let lowered = name.to_ascii_lowercase();
        if lowered == "exit" || lowered == "die" {
            return true;
        }
        self.canonical_function_name_folded(name)
            .and_then(|canonical| self.functions.get(&canonical))
            .map(|sig| sig.return_type == PhpType::Never)
            .unwrap_or(false)
    }

    /// Returns true if the expression calls a user function whose declared return type is `never`.
    /// The function name is resolved case-insensitively against the checker's function table,
    /// matching PHP's call semantics. Error suppression preserves the call's divergence.
    pub(in crate::types::checker) fn expr_is_declared_never_call(&self, expr: &Expr) -> bool {
        if let ExprKind::ErrorSuppress(inner) = &expr.kind {
            return self.expr_is_declared_never_call(inner);
        }
        let ExprKind::FunctionCall { name, .. } = &expr.kind else {
            return false;
        };
        self.canonical_function_name_folded(name)
            .and_then(|canonical| self.functions.get(&canonical))
            .map(|sig| sig.return_type == PhpType::Never)
            .unwrap_or(false)
    }
}

/// Flattens a left-associative `||` chain into its disjunct operands in source order. A non-`||`
/// expression is a single disjunct. Used to distribute De Morgan's law over an `if (A || B) {…}`
/// early-exit guard.
fn collect_or_operands<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
    if let ExprKind::BinaryOp {
        left,
        op: BinOp::Or,
        right,
    } = &expr.kind
    {
        collect_or_operands(left, out);
        collect_or_operands(right, out);
    } else {
        out.push(expr);
    }
}

/// Extracts the guarded receiver expression and the target type from a (non-negated) guard
/// expression. Recognizes the scalar `is_*` predicates, `is_null`, `instanceof <Name>`, and
/// `=== false` / `!== false` / `=== null` / `!== null`. The receiver may be any expression here
/// — `guard_env_key` decides which receivers narrowing can actually key (variables, assignment
/// results, and simple property accesses). The boolean marks a comparison whose truth edge is
/// the complement of the literal type.
fn guard_receiver_and_type(cond: &Expr) -> Option<(&Expr, PhpType, bool)> {
    match &cond.kind {
        ExprKind::FunctionCall { name, args } if args.len() == 1 => {
            let target = match name.as_str().to_ascii_lowercase().as_str() {
                "is_int" | "is_integer" | "is_long" => PhpType::Int,
                "is_float" | "is_double" | "is_real" => PhpType::Float,
                "is_string" => PhpType::Str,
                "is_bool" => PhpType::Bool,
                // `is_array($x)` proves the value is *some* array. The checker narrows to the
                // gradual `array<mixed>` family: it is accepted by every array operation and array
                // builtin (a union of `array|assoc-array` would be rejected by the concrete-only
                // builtins such as `array_sum`/`array_unique`), and it does not over-refine the key
                // or element type. The runtime indexed/associative distinction is handled where it
                // matters — by the EIR lowering, which keeps the guarded local in its boxed Mixed
                // representation so `foreach`/index/`count` dispatch on the runtime tag (see
                // `ir_lower::stmt::is_array_narrowed_type`); an assoc payload no longer fatals.
                "is_array" => PhpType::Array(Box::new(PhpType::Mixed)),
                // `is_numeric($x)` accepts ints, floats, and numeric strings without changing
                // the runtime value. Preserve all three possibilities so arithmetic selects
                // mixed numeric dispatch for strings instead of pretending the value was cast.
                "is_numeric" => PhpType::Union(vec![
                    PhpType::Int,
                    PhpType::Float,
                    PhpType::Str,
                ]),
                // `is_object($x)` proves the boxed value carries an object pointer, but it does
                // not identify a concrete class. Keep that distinction through generic `object`
                // so guarded `$x::class` and other class-agnostic object operations type-check.
                "is_object" => PhpType::Object(String::new()),
                // `is_null($x)`: same narrowing as `$x === null` — elephc models a `?T` value's
                // null as Void, so the complement strips it (`if (is_null($x)) { throw; }` leaves
                // ?int as int on the fall-through path).
                "is_null" => PhpType::Void,
                // `is_countable($x)` guarantees the value is an `array` or a `Countable`
                // object — exactly the two things `count()` accepts. Narrowing to this
                // union lets guarded `count($x)` type-check even when `$x` is declared
                // `iterable` (a non-Countable `Traversable` is dropped by the guard, so
                // unguarded `count(iterable)` still errors).
                "is_countable" => PhpType::Union(vec![
                    PhpType::Array(Box::new(PhpType::Mixed)),
                    PhpType::Object("Countable".to_string()),
                ]),
                _ => return None,
            };
            Some((&args[0], target, false))
        }
        ExprKind::InstanceOf { value, target } => {
            let InstanceOfTarget::Name(class) = target else {
                return None;
            };
            Some((value, PhpType::Object(class.as_str().to_string()), false))
        }
        // `$var === false` / `$var !== false` (and reversed operands): equality narrows the
        // then-branch to False, while inequality narrows it to the complement (e.g.
        // int|false → int). A full `bool` member remains representable and is not stripped.
        ExprKind::BinaryOp {
            left,
            op: op @ (BinOp::StrictEq | BinOp::StrictNotEq),
            right,
        } => {
            let (receiver, lit) = if let Some(receiver) =
                strict_comparison_guard_receiver(left)
            {
                (receiver, &right.kind)
            } else if let Some(receiver) = strict_comparison_guard_receiver(right) {
                (receiver, &left.kind)
            } else {
                return None;
            };
            match lit {
                ExprKind::BoolLiteral(false) => {
                    Some((receiver, PhpType::False, *op == BinOp::StrictNotEq))
                }
                // `$x === null`: strip the null-ish member (elephc models a `?T` value's null as
                // Void), e.g. `?self` / self|null → self after `if ($x === null) { throw; }`.
                ExprKind::Null => Some((receiver, PhpType::Void, *op == BinOp::StrictNotEq)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Returns a receiver whose binding can be narrowed by a strict false/null comparison.
///
/// Besides variables and stable property reads, this accepts an ordinary expression-position
/// assignment to a local. Its assigned value is the current runtime value even when the local's
/// storage type also includes values written before the assignment.
fn strict_comparison_guard_receiver(expr: &Expr) -> Option<&Expr> {
    match &expr.kind {
        ExprKind::Variable(_) | ExprKind::PropertyAccess { .. } => Some(expr),
        ExprKind::Assignment {
            target,
            result_target: None,
            prelude,
            conditional_value_temp: None,
            ..
        } if prelude.is_empty() && matches!(target.kind, ExprKind::Variable(_)) => Some(expr),
        _ => None,
    }
}


/// Returns true when a union member is compatible with a guard target, used to keep (then) or drop
/// (else) members. Scalar targets require an exact variant match; an `Object` target matches an
/// object member with the same class name (inheritance-aware narrowing is left for the future).
/// Generic `object` matches every concrete object member so `is_object()` preserves any known
/// classes already present in a union. The compiler's `Callable` representation is the same
/// closure descriptor used for anonymous and first-class callables, so it matches nominal
/// `Closure` guards as well.
fn guard_matches(member: &PhpType, target: &PhpType) -> bool {
    match (member, target) {
        (PhpType::Callable, PhpType::Object(target_class))
            if target_class
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("Closure") =>
        {
            true
        }
        (PhpType::Object(member_class), PhpType::Object(target_class)) => {
            target_class.is_empty() || member_class == target_class
        }
        (
            PhpType::Array(_) | PhpType::AssocArray { .. },
            PhpType::Array(_),
        ) => true,
        (PhpType::False, PhpType::Bool) => true,
        _ => member == target,
    }
}
