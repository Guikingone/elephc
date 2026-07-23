//! Purpose:
//! Flow-sensitive type narrowing for `if`/`else` branches guarded by type predicates.
//! Narrows a union- or mixed-typed variable to the guarded type in the matching branch.
//!
//! Called from:
//! - `crate::types::checker::stmt_check::control_flow` when checking `StmtKind::If` and
//!   `switch (true)` case bodies.
//!
//! Key details:
//! - Recognizes `is_int`/`is_float`/`is_string`/`is_bool`/`is_countable($var)` (and aliases) and
//!   `$var instanceof Class` guards, optionally negated with a leading `!`. `is_countable` narrows
//!   to `array<mixed>|Countable`. Narrowing is applied to each clause in an
//!   if/elseif*/else chain (each subsequent clause, and the else, see the accumulated complement
//!   from previous guards). For a chain with no else where *every* clause body always diverges
//!   (return/throw/exit/die/never-function), the accumulated complement is applied to the statements
//!   after the entire if construct.
//! - `and_chain_then_narrowings` collects the guard-true narrowings for a single guard or a pure
//!   `&&` chain of guards, folding repeated guards on one variable cumulatively; it powers
//!   `switch (true)` case-body narrowing. `case_body_terminates` gates that narrowing under
//!   PHP fall-through (only a case that cannot be fallen into is narrowed by its own guard).
//! - Conservative: a concrete (non-union, non-mixed) type is left unchanged, and an empty narrowing
//!   result falls back to the original type, so valid code is never narrowed away to `Never`.

use crate::errors::CompileError;
use crate::names::{php_symbol_key, property_hook_get_method};
use crate::parser::ast::{BinOp, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind};
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
}

impl Checker {
    /// Detects a type-predicate guard in an `if`/ternary condition and computes the then/else
    /// narrowing for the guarded binding against the current environment. Handles the scalar
    /// `is_*` predicates, `is_null`, `instanceof Class`, and `=== false` / `=== null`, each with an
    /// optional leading `!` that swaps the branches. The guarded receiver may be a variable
    /// (narrowed under its name) or a simple property access `$var->prop` / `$this->prop`
    /// (narrowed under a synthetic key that `infer_property_access_type` consults). Returns
    /// `Ok(None)` when the condition is not a recognized guard or the receiver's current type is
    /// unknown.
    pub(crate) fn guard_narrowing(
        &mut self,
        condition: &Expr,
        env: &TypeEnv,
    ) -> Result<Option<GuardNarrowing>, CompileError> {
        let (cond, negated) = match &condition.kind {
            ExprKind::Not(inner) => (inner.as_ref(), true),
            _ => (condition, false),
        };
        let Some((receiver, target)) = guard_receiver_and_type(cond) else {
            return Ok(None);
        };
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
        // A prior narrowing (or a variable binding) wins; otherwise a property receiver falls back
        // to its declared field type. An unbound plain variable stays un-narrowed.
        let current = match env.get(&key) {
            Some(ty) => ty.clone(),
            None if matches!(receiver.kind, ExprKind::PropertyAccess { .. }) => {
                self.infer_type(receiver, env)?
            }
            None => return Ok(None),
        };
        let matched = self.narrow_to(&current, &target);
        let complement = self.narrow_complement(&current, &target);
        let (then_ty, else_ty) = if negated {
            (complement, matched)
        } else {
            (matched, complement)
        };
        Ok(Some(GuardNarrowing { var: key, then_ty, else_ty }))
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

    /// `TypeEnv` key for a guard receiver: a variable's name, or the synthetic property key for a
    /// simple property access. `None` for receivers narrowing can't key (complex chains).
    fn guard_env_key(receiver: &Expr) -> Option<String> {
        match &receiver.kind {
            ExprKind::Variable(var) => Some(var.clone()),
            ExprKind::PropertyAccess { object, property } => {
                Self::narrowed_property_env_key(object, property)
            }
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
        &self,
        cond: &Expr,
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
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
            let mut narrowings = self.and_chain_then_narrowings(left, env);
            for (var, then_ty) in self.and_chain_then_narrowings(right, env) {
                match narrowings.iter().position(|(v, _)| *v == var) {
                    Some(idx) => {
                        let existing = narrowings[idx].1.clone();
                        narrowings[idx].1 = self.narrow_to(&existing, &then_ty);
                    }
                    None => narrowings.push((var, then_ty)),
                }
            }
            narrowings
        } else {
            match self.type_guard_narrowing(cond, env) {
                Some(g) => vec![(g.var, g.then_ty)],
                None => vec![],
            }
        }
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

    /// Returns true when a statement body always diverges.
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
}

/// Extracts the guarded receiver expression and the target type from a (non-negated) guard
/// expression. Recognizes the scalar `is_*` predicates, `is_null`, `instanceof <Name>`, and
/// `=== false` / `=== null`. The receiver may be any expression here — `guard_env_key` decides
/// which receivers narrowing can actually key (variables and simple property accesses).
fn guard_receiver_and_type(cond: &Expr) -> Option<(&Expr, PhpType)> {
    match &cond.kind {
        ExprKind::FunctionCall { name, args } if args.len() == 1 => {
            let target = match name.as_str().to_ascii_lowercase().as_str() {
                "is_int" | "is_integer" | "is_long" => PhpType::Int,
                "is_float" | "is_double" | "is_real" => PhpType::Float,
                "is_string" => PhpType::Str,
                "is_bool" => PhpType::Bool,
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
            Some((&args[0], target))
        }
        ExprKind::InstanceOf { value, target } => {
            let InstanceOfTarget::Name(class) = target else {
                return None;
            };
            Some((value, PhpType::Object(class.as_str().to_string())))
        }
        // `$var === false` / `false === $var`: narrow to the literal False subtype in the
        // then-branch; the else-branch strips only that member (e.g. int|false → int) while a full
        // `bool` member remains. Enables the common
        // `if ($x === false) { throw; } return $x;` guard (ward-http StreamGuards::requireInt etc.).
        ExprKind::BinaryOp { left, op: BinOp::StrictEq, right } => {
            let (receiver, lit) = match (&left.kind, &right.kind) {
                (ExprKind::Variable(_) | ExprKind::PropertyAccess { .. }, _) => {
                    (left.as_ref(), &right.kind)
                }
                (_, ExprKind::Variable(_) | ExprKind::PropertyAccess { .. }) => {
                    (right.as_ref(), &left.kind)
                }
                _ => return None,
            };
            match lit {
                ExprKind::BoolLiteral(false) => Some((receiver, PhpType::False)),
                // `$x === null`: strip the null-ish member (elephc models a `?T` value's null as
                // Void), e.g. `?self` / self|null → self after `if ($x === null) { throw; }`.
                ExprKind::Null => Some((receiver, PhpType::Void)),
                _ => None,
            }
        }
        _ => None,
    }
}


/// Returns true when a union member is compatible with a guard target, used to keep (then) or drop
/// (else) members. Scalar targets require an exact variant match; an `Object` target matches an
/// object member with the same class name (inheritance-aware narrowing is left for the future).
fn guard_matches(member: &PhpType, target: &PhpType) -> bool {
    match (member, target) {
        (PhpType::Object(member_class), PhpType::Object(target_class)) => member_class == target_class,
        (PhpType::False, PhpType::Bool) => true,
        _ => member == target,
    }
}

impl Checker {
        /// Detects a type-predicate guard in an `if` condition and computes the then/else narrowing
        /// for the guarded variable against the current environment. Handles the scalar `is_*`
        /// predicates and `$var instanceof Class`, with an optional leading `!` that swaps the
        /// branches. Returns `None` when the condition is not a recognized single-variable guard or the
        /// variable has no known type in `env`.
        pub(crate) fn type_guard_narrowing(
            &self,
            condition: &Expr,
            env: &TypeEnv,
        ) -> Option<GuardNarrowing> {
            let (cond, negated) = match &condition.kind {
                ExprKind::Not(inner) => (inner.as_ref(), true),
                _ => (condition, false),
            };
            let (var, target) = guard_var_and_type(cond)?;
            let target = self.resolve_relative_instanceof_target(target);
            let current = env.get(&var)?.clone();
            let matched = self.narrow_to(&current, &target);
            let complement = self.narrow_complement(&current, &target);
            let (then_ty, else_ty) = if negated {
                (complement, matched)
            } else {
                (matched, complement)
            };
            Some(GuardNarrowing { var, then_ty, else_ty })
        }
}

/// Extracts the guarded variable name and the target type from a (non-negated) guard expression.
/// Recognizes the scalar `is_*` predicates and `instanceof <Name>`; returns `None` for anything
/// else (including guards on non-variable operands).
fn guard_var_and_type(cond: &Expr) -> Option<(String, PhpType)> {
    match &cond.kind {
        ExprKind::FunctionCall { name, args } if args.len() == 1 => {
            let ExprKind::Variable(var) = &args[0].kind else {
                return None;
            };
            let target = match name.as_str().to_ascii_lowercase().as_str() {
                "is_int" | "is_integer" | "is_long" => PhpType::Int,
                "is_float" | "is_double" => PhpType::Float,
                "is_string" => PhpType::Str,
                "is_bool" => PhpType::Bool,
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
            Some((var.clone(), target))
        }
        ExprKind::InstanceOf { value, target } => {
            let ExprKind::Variable(var) = &value.kind else {
                return None;
            };
            let InstanceOfTarget::Name(class) = target else {
                return None;
            };
            Some((var.clone(), PhpType::Object(class.as_str().to_string())))
        }
        _ => None,
    }
}
