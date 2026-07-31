//! Purpose:
//! Checks type compatibility for object types cases.
//! Supports the central assignability predicate used by declarations, calls, returns, and assignments.
//!
//! Called from:
//! - `crate::types::checker::type_compat`
//!
//! Key details:
//! - Rules here define accepted programs, so PHP covariance, inheritance, and extension-specific constraints must stay explicit.

use std::collections::HashSet;

use crate::errors::CompileError;
use crate::parser::ast::{Expr, Visibility};
use crate::types::{EnumInfo, PhpType, TypeEnv};

use super::super::Checker;

impl Checker {
    /// Checks whether the current class context can access a member with the given visibility
    /// declared in `declaring_class`. Public members are always accessible; protected members
    /// are accessible when the current and declaring classes share one inheritance chain in
    /// either direction; private members require the exact declaring class.
    pub(crate) fn can_access_member(
        &self,
        declaring_class: &str,
        visibility: &Visibility,
    ) -> bool {
        match visibility {
            Visibility::Public => true,
            Visibility::Protected => self.current_class.as_deref().is_some_and(|current| {
                current == declaring_class
                    || self.is_subclass_of(current, declaring_class)
                    || self.is_subclass_of(declaring_class, current)
            }),
            Visibility::Private => self.current_class.as_deref() == Some(declaring_class),
        }
    }

    /// Returns the string label ("public", "protected", "private") for a visibility level.
    pub(crate) fn visibility_label(visibility: &Visibility) -> &'static str {
        match visibility {
            Visibility::Public => "public",
            Visibility::Protected => "protected",
            Visibility::Private => "private",
        }
    }

    /// Returns true when no single runtime object can be an instance of both class `a` and class
    /// `b`. Only reasons about concrete/abstract classes (both present in `self.classes`): PHP is
    /// single-inheritance, so two distinct classes neither of which derives from the other can
    /// never share an instance. Interfaces are co-implementable and so are excluded (a miss in
    /// `self.classes` returns `false`, i.e. "cannot prove incompatible"). Used to recognize a
    /// statically-dead `$this instanceof B` guard inside a method of an unrelated class `a`.
    pub(crate) fn classes_are_instanceof_incompatible(&self, a: &str, b: &str) -> bool {
        let a = a.trim_start_matches('\\');
        let b = b.trim_start_matches('\\');
        if a == b {
            return false;
        }
        if !self.classes.contains_key(a) || !self.classes.contains_key(b) {
            return false;
        }
        !self.is_subclass_of(a, b) && !self.is_subclass_of(b, a)
    }

    /// Returns true if `class_name` is or inherits from `ancestor_name` (excluding self equality).
    /// Walks the parent chain via `class_info.parent`.
    pub(crate) fn is_subclass_of(&self, class_name: &str, ancestor_name: &str) -> bool {
        let mut current = self
            .classes
            .get(class_name)
            .and_then(|class| class.parent.clone());
        while let Some(parent_name) = current {
            if parent_name == ancestor_name {
                return true;
            }
            current = self
                .classes
                .get(&parent_name)
                .and_then(|class| class.parent.clone());
        }
        false
    }

    /// Returns true when a value statically typed as an object of `class_name` is usable in a
    /// PHP string context — the class or one of its ancestors declares a public `__toString()`
    /// method (satisfying the implicit `Stringable` contract), or the type implements
    /// `Stringable` explicitly (directly or through interface inheritance).
    ///
    /// The parent-chain walk is what makes an abstract-base static type (for example
    /// `Symfony\Component\String\AbstractString`, whose concrete children carry the real
    /// `__toString()`) count as Stringable: the abstract base itself declares the method, and a
    /// value typed as a subclass inherits it. This mirrors the codegen string-cast object arm
    /// (`lower_object_to_string`), which dispatches `__toString()` virtually on the runtime class.
    pub(crate) fn object_class_is_stringable(&self, class_name: &str) -> bool {
        let normalized = class_name.trim_start_matches('\\');
        if normalized.is_empty() {
            return false;
        }
        let tostring_key = crate::names::php_symbol_key("__toString");
        let stringable_key = crate::names::php_symbol_key("Stringable");
        let mut current = Some(normalized.to_string());
        let mut seen = HashSet::new();
        while let Some(name) = current {
            if !seen.insert(name.clone()) {
                break;
            }
            let Some(info) = self.classes.get(&name) else {
                break;
            };
            let tostring_is_public = info
                .method_visibilities
                .get(&tostring_key)
                .map(|visibility| *visibility == Visibility::Public)
                // PHP requires `__toString` to be public; a class that records the method
                // without a visibility entry (compiler-synthesized shells) is treated as public.
                .unwrap_or_else(|| info.methods.contains_key(&tostring_key));
            if tostring_is_public && info.methods.contains_key(&tostring_key) {
                return true;
            }
            if info
                .interfaces
                .iter()
                .any(|iface| crate::names::php_symbol_key(iface) == stringable_key)
            {
                return true;
            }
            current = info.parent.clone();
        }
        // Interface-typed targets (and any Stringable reachable only through interface
        // inheritance) are covered here.
        self.object_type_implements_interface(normalized, "Stringable")
    }

    /// Returns true when `class_name` is a CONCRETE (non-abstract) class with a resolvable
    /// `__toString`, so an eager `(string)$obj` cast at a plain-`string` value boundary resolves a
    /// direct call to a real `__toString` method body.
    ///
    /// Interfaces and unknown names (`self.classes` miss) and abstract classes are rejected: the
    /// eager cast would emit a call to a symbol with no body (a link failure). A concrete class
    /// that satisfies `Stringable` necessarily carries a concrete `__toString` (its own or an
    /// inherited concrete one) — otherwise it could not be concrete — so `object_class_is_stringable`
    /// on a concrete class is sufficient. Abstract-base and interface Stringable static types reach
    /// a `string` slot only through a boxed union, which dispatches `__toString` virtually at use.
    pub(crate) fn object_class_has_eager_tostring(&self, class_name: &str) -> bool {
        let normalized = class_name.trim_start_matches('\\');
        let Some(info) = self.classes.get(normalized) else {
            return false;
        };
        if info.is_abstract {
            return false;
        }
        self.object_class_is_stringable(normalized)
    }

    /// Returns true if `class_name` directly implements `interface_name` (not via inheritance).
    pub(crate) fn class_implements_interface(&self, class_name: &str, interface_name: &str) -> bool {
        self.classes.get(class_name).is_some_and(|class_info| {
            class_info
                .interfaces
                .iter()
                .any(|name| name == interface_name)
        })
    }

    /// Returns true if `type_name` (a class or interface) implements `interface_name`,
    /// checking direct implementation and interface inheritance chains.
    pub(crate) fn object_type_implements_interface(
        &self,
        type_name: &str,
        interface_name: &str,
    ) -> bool {
        if self.classes.contains_key(type_name) {
            return self.class_implements_interface(type_name, interface_name);
        }
        if self.interfaces.contains_key(type_name) {
            return type_name == interface_name
                || self.interface_extends_interface(type_name, interface_name);
        }
        false
    }

    /// Returns true if `interface_name` is or transitively extends `ancestor_name`.
    /// Uses a DFS with cycle detection to walk interface parent chains.
    pub(crate) fn interface_extends_interface(
        &self,
        interface_name: &str,
        ancestor_name: &str,
    ) -> bool {
        if interface_name == ancestor_name {
            return true;
        }
        let mut stack = vec![interface_name.to_string()];
        let mut seen = HashSet::new();
        while let Some(current_name) = stack.pop() {
            if !seen.insert(current_name.clone()) {
                continue;
            }
            let Some(interface_info) = self.interfaces.get(&current_name) else {
                continue;
            };
            for parent_name in &interface_info.parents {
                if parent_name == ancestor_name {
                    return true;
                }
                stack.push(parent_name.clone());
            }
        }
        false
    }

    /// Returns true if `type_name` (a class or interface) implements `Throwable`,
    /// checking both direct implementation and interface extension chains.
    pub(crate) fn object_type_implements_throwable(&self, type_name: &str) -> bool {
        if self.classes.contains_key(type_name) {
            return self.class_implements_interface(type_name, "Throwable");
        }
        if self.interfaces.contains_key(type_name) {
            return self.interface_extends_interface(type_name, "Throwable");
        }
        false
    }

    /// Returns true if `type_name` (a class or interface) implements `Iterator` or
    /// `IteratorAggregate`, which are the interfaces that make a type usable in `foreach`.
    pub(crate) fn object_type_implements_iterable(&self, type_name: &str) -> bool {
        if type_name == "Traversable" {
            return true;
        }
        if self.classes.contains_key(type_name) {
            return self.class_implements_interface(type_name, "Iterator")
                || self.class_implements_interface(type_name, "IteratorAggregate");
        }
        if self.interfaces.contains_key(type_name) {
            return self.interface_extends_interface(type_name, "Iterator")
                || self.interface_extends_interface(type_name, "IteratorAggregate");
        }
        false
    }

    /// Finds the most specific common ancestor object type shared by all type names in
    /// `type_names`. Falls back to "Throwable" if no common type is found or the list is empty.
    pub(crate) fn common_catch_type_name(&self, type_names: &[String]) -> String {
        let mut iter = type_names.iter();
        let Some(first_name) = iter.next() else {
            return "Throwable".to_string();
        };
        let mut common = first_name.clone();
        for type_name in iter {
            match self.common_object_type(&common, type_name) {
                Some(PhpType::Object(next_common)) => common = next_common,
                _ => return "Throwable".to_string(),
            }
        }
        common
    }

    /// Resolves a raw type name used in a `catch` clause to its concrete class name.
    /// Handles "self" (current class), "parent" (current class's parent), or a plain name.
    pub(crate) fn resolve_catch_type_name(
        &self,
        raw_name: &crate::names::Name,
        span: crate::span::Span,
    ) -> Result<String, CompileError> {
        match raw_name.as_str() {
            "self" => self.current_class.clone().ok_or_else(|| {
                CompileError::new(span, "Cannot use self in catch outside of a class context")
            }),
            "parent" => {
                let current_class = self.current_class.as_ref().ok_or_else(|| {
                    CompileError::new(
                        span,
                        "Cannot use parent in catch outside of a class context",
                    )
                })?;
                self.classes
                    .get(current_class)
                    .and_then(|class_info| class_info.parent.clone())
                    .ok_or_else(|| CompileError::new(span, "Class has no parent class"))
            }
            _ => Ok(raw_name.to_string()),
        }
    }

    /// Type-checks and resolves a static method call on an enum (`cases`, `from`, `tryfrom`).
    /// Returns the resolved `PhpType` for the call or an error if argument count or types mismatch.
    pub(crate) fn check_enum_static_call(
        &mut self,
        enum_info: &EnumInfo,
        class_name: &str,
        method: &str,
        args: &[Expr],
        env: &TypeEnv,
        span: crate::span::Span,
    ) -> Result<PhpType, CompileError> {
        match method {
            "cases" => {
                if !args.is_empty() {
                    return Err(CompileError::new(
                        span,
                        &format!(
                            "Static method '{}::cases' expects 0 arguments, got {}",
                            class_name,
                            args.len()
                        ),
                    ));
                }
                Ok(PhpType::Array(Box::new(PhpType::Object(
                    class_name.to_string(),
                ))))
            }
            "from" | "tryfrom" => {
                let Some(backing_ty) = enum_info.backing_type.as_ref() else {
                    return Err(CompileError::new(
                        span,
                        &format!("Undefined method: {}::{}", class_name, method),
                    ));
                };
                if args.len() != 1 {
                    return Err(CompileError::new(
                        span,
                        &format!(
                            "Static method '{}::{}' expects 1 argument, got {}",
                            class_name,
                            method,
                            args.len()
                        ),
                    ));
                }
                let arg_ty = self.infer_type(&args[0], env)?;
                // PHP's int-backed enum `from()`/`tryFrom()` accepts a numeric string and
                // coerces it to the integer backing value at runtime: a numeric string with
                // no matching case throws `ValueError`, a non-numeric string throws
                // `TypeError`. A `Mixed`/dynamic argument (e.g. a `foreach` value or untyped
                // parameter) coerces on its runtime tag. Accept these here and defer the
                // coercion and error to codegen instead of rejecting at compile time
                // (issues #349, #449).
                let accepts_runtime_coercion = matches!(backing_ty, PhpType::Int)
                    && matches!(arg_ty, PhpType::Str | PhpType::Mixed | PhpType::Union(_));
                if !accepts_runtime_coercion && !self.type_accepts(backing_ty, &arg_ty) {
                    return Err(CompileError::new(
                        span,
                        &format!(
                            "Type error: {}::{} expects {}, got {}",
                            class_name, method, backing_ty, arg_ty
                        ),
                    ));
                }
                if method == "from" {
                    Ok(PhpType::Object(class_name.to_string()))
                } else {
                    Ok(self.normalize_union_type(vec![
                        PhpType::Object(class_name.to_string()),
                        PhpType::Void,
                    ]))
                }
            }
            _ => {
                // User-declared static methods on the enum dispatch like class static methods.
                let key = crate::names::php_symbol_key(method);
                if let Some(sig) = self
                    .classes
                    .get(class_name)
                    .and_then(|class_info| class_info.static_methods.get(&key))
                    .cloned()
                {
                    return Ok(sig.return_type);
                }
                Err(CompileError::new(
                    span,
                    &format!("Undefined method: {}::{}", class_name, method),
                ))
            }
        }
    }

    /// Finds the most specific common object type between `left` and `right` class names.
    /// Returns `Some(PhpType::Object(...))` with the common ancestor, or `None` if they share no
    /// common type. Checks interface relationships, class inheritance, and ancestor chains.
    pub(crate) fn common_object_type(&self, left: &str, right: &str) -> Option<PhpType> {
        if left == right {
            return Some(PhpType::Object(left.to_string()));
        }
        if self.interfaces.contains_key(left) && self.interface_extends_interface(right, left) {
            return Some(PhpType::Object(left.to_string()));
        }
        if self.interfaces.contains_key(right) && self.interface_extends_interface(left, right) {
            return Some(PhpType::Object(right.to_string()));
        }
        if self.interfaces.contains_key(left) && self.class_implements_interface(right, left) {
            return Some(PhpType::Object(left.to_string()));
        }
        if self.interfaces.contains_key(right) && self.class_implements_interface(left, right) {
            return Some(PhpType::Object(right.to_string()));
        }
        if self.is_subclass_of(left, right) {
            return Some(PhpType::Object(right.to_string()));
        }
        if self.is_subclass_of(right, left) {
            return Some(PhpType::Object(left.to_string()));
        }

        let mut left_ancestors = HashSet::new();
        let mut current = Some(left.to_string());
        while let Some(class_name) = current {
            left_ancestors.insert(class_name.clone());
            current = self
                .classes
                .get(&class_name)
                .and_then(|class_info| class_info.parent.clone());
        }

        let mut current = Some(right.to_string());
        while let Some(class_name) = current {
            if left_ancestors.contains(&class_name) {
                return Some(PhpType::Object(class_name));
            }
            current = self
                .classes
                .get(&class_name)
                .and_then(|class_info| class_info.parent.clone());
        }

        None
    }

    /// Propagates a resolved constructor argument type into the corresponding constructor
    /// parameter and property slot for all classes that share an inherited property from
    /// `declaring_class`. Used to sharpen types across an inheritance hierarchy after
    /// constructor argument type inference.
    ///
    /// The constructor-parameter refinement is resolved per target class: for each class
    /// that shares the inherited property, the parameter index is looked up in that class's
    /// own `constructor_param_to_prop` (never the instantiated class's `param_index`) and the
    /// refinement is gated on that signature's own `declared_params`. A reordering subclass
    /// that does not promote the shared property in its own constructor is therefore skipped
    /// and never has an unrelated parameter overwritten.
    pub(crate) fn propagate_constructor_arg_type(
        &mut self,
        instantiated_class: &str,
        param_index: usize,
        arg_ty: &PhpType,
    ) {
        let Some((prop_name, declaring_class)) =
            self.classes.get(instantiated_class).and_then(|class_info| {
                class_info
                    .constructor_param_to_prop
                    .get(param_index)
                    .and_then(|mapped| mapped.as_ref())
                    .map(|prop_name| {
                        let declaring_class = class_info
                            .property_declaring_classes
                            .get(prop_name)
                            .cloned()
                            .unwrap_or_else(|| instantiated_class.to_string());
                        (prop_name.clone(), declaring_class)
                    })
            })
        else {
            return;
        };

        for class_info in self.classes.values_mut() {
            let shares_inherited_property = class_info
                .property_declaring_classes
                .get(&prop_name)
                .is_some_and(|owner| owner == &declaring_class);

            if !shares_inherited_property {
                continue;
            }

            let property_has_declared_type = class_info.visible_property_is_declared(&prop_name);
            if !property_has_declared_type {
                if let Some(slot) = class_info.visible_property_index(&prop_name) {
                    if let Some(prop) = class_info.properties.get_mut(slot) {
                        prop.1 = arg_ty.clone();
                    }
                }
            }

            // Resolve the constructor parameter index in THIS class's own promoted-property
            // map, not the instantiated class's `param_index`. A subclass may reorder or omit
            // the shared property in its own constructor, so the instantiated class's index is
            // only valid for that class. Compute the index before borrowing `methods` mutably
            // so the immutable borrow of `constructor_param_to_prop` is released first.
            let target_idx = class_info
                .constructor_param_to_prop
                .iter()
                .position(|mapped| mapped.as_deref() == Some(prop_name.as_str()));
            if let Some(idx) = target_idx {
                if let Some(sig) = class_info.methods.get_mut("__construct") {
                    let is_declared = sig.declared_params.get(idx).copied().unwrap_or(false);
                    if !is_declared {
                        if let Some((_, param_ty)) = sig.params.get_mut(idx) {
                            *param_ty = arg_ty.clone();
                        }
                    }
                }
            }
        }
    }
}

/// Returns whether `ty` is gradually acceptable where PHP requires an object value at
/// runtime (e.g. `throw`, `spl_object_id`, a nullsafe receiver).
///
/// Accepts a concrete `Object`, the `Mixed` top type, and any `Union` that contains an
/// `Object` or `Mixed` member (which covers `?Object`/`Object|null`, `Object|false`, and
/// a boxed value inferred as `Mixed`). Bare scalars (`Int`/`Str`/`Bool`/`Float`), arrays,
/// and other proven non-object types are rejected so genuine "not an object" bugs keep
/// surfacing. This is the object-family analogue of `array_arg_is_gradually_acceptable`;
/// gradual typing defers the real object/Throwable check to the runtime, matching PHP.
pub(crate) fn type_is_gradual_object_family(ty: &PhpType) -> bool {
    match ty {
        PhpType::Object(_) | PhpType::Mixed => true,
        PhpType::Union(members) => members
            .iter()
            .any(|member| matches!(member, PhpType::Object(_) | PhpType::Mixed)),
        _ => false,
    }
}

impl Checker {
        /// Returns true if the flow from `actual` into a slot declared `expected` is one a runtime
        /// checked-downcast guard can enforce — the base→derived relaxation PHP itself allows,
        /// deferring the real check to a runtime `instanceof`.
        ///
        /// This is the ONE source of truth for that acceptance decision, and it is consumed at
        /// every position that emits the guard: returns (`functions::returns`), call arguments
        /// (`functions::call_validation::call_arg_downcast_guardable`). A safety predicate with
        /// two implementations is two predicates whose permissiveness unions, so new positions
        /// must call this rather than re-deriving the hierarchy walk.
        ///
        /// Three conditions, all required:
        /// 1. `crate::types::checked_downcast::downcast_guard_shape` must find a representation
        ///    pair a guard could bridge — the SAME function `crate::ir_lower::checked_downcast`
        ///    consults before emitting, so acceptance cannot outrun emission.
        /// 2. `expected` must offer at least one NAMED class/interface arm. A bare `object`
        ///    (`Object("")`) target is not an `instanceof` target: no chain can be built for it.
        /// 3. `actual` must be an object the guard could plausibly narrow: a bare `object` source
        ///    (a raw object pointer of unknown class — every named arm is testable against it), or
        ///    a concrete `Object(B)` that is a proper ANCESTOR of some declared arm. `Mixed` and
        ///    `Union` sources deliberately return false here: `Mixed` already flows through
        ///    `gradual_boundary_accepts`, and union sources need the union-arm analysis that
        ///    belongs with their own emitter work.
        ///
        /// Only meant to be tried as a fallback AFTER normal covariant acceptance
        /// (`require_compatible_arg_type`) has already failed for this `(expected, actual)` pair —
        /// it does not special-case an `actual` that already satisfies `expected` normally.
        pub(crate) fn checked_downcast_guardable(&self, expected: &PhpType, actual: &PhpType) -> bool {
            if crate::types::checked_downcast::downcast_guard_shape(expected, actual).is_none() {
                return false;
            }
            let declared_arms = crate::types::checked_downcast::declared_instanceof_targets(expected);
            if declared_arms.is_empty() {
                return false;
            }
            let PhpType::Object(actual_name) = actual else {
                return false;
            };
            // A bare `object` source is a raw object pointer whose class is unknown statically;
            // every declared arm is a legitimate `instanceof` target for it, and the guard throws
            // PHP's own `TypeError` when none matches.
            if actual_name.is_empty() {
                return true;
            }
            declared_arms.iter().any(|declared_name| {
                declared_name != actual_name
                    && (self.is_subclass_of(declared_name, actual_name)
                        || self.object_type_implements_interface(declared_name, actual_name))
            })
        }
}

impl Checker {
        /// Checks whether `receiver` can access a member with the given visibility declared in
        /// `declaring_class`, additionally consulting an active `Closure::bind`/`bindTo` scope
        /// rebind (`Checker::bound_scope_context`) when the normal `can_access_member` check fails.
        ///
        /// The bound-scope override applies only when `receiver` is exactly
        /// `ExprKind::Variable(name)` naming either an eligible declared parameter or a variable
        /// local to the closure body. An untyped parameter becomes eligible only once ordinary
        /// inference reaches this property check with an object in the rebound scope. Captured
        /// variables remain excluded, as do computed expressions and `$this` (which the lexical
        /// gate that populates `bound_scope_context` already proved absent from the body).
        pub(crate) fn can_access_property(
            &self,
            receiver: &Expr,
            declaring_class: &str,
            visibility: &Visibility,
        ) -> bool {
            if self.can_access_member(declaring_class, visibility) {
                return true;
            }
            // Dead `$this instanceof Sibling` branch relaxation. A protected access on `$this`
            // narrowed to a class the current class can never be (a distinct class in a separate
            // single-inheritance chain) is statically unreachable: `$this` is always an instance
            // of the current class or a subclass, and single inheritance forbids any such value
            // from also being an instance of an unrelated class, so the guarded access never
            // executes. PHP checks visibility only when the fetch runs, so it never faults here
            // either. This is the Symfony `AddTrait` pattern (`$this instanceof RouteConfigurator
            // ? $this->parentConfigurator : ...` composed into the sibling `CollectionConfigurator`).
            // Private members stay strict (a private property is never reachable through a
            // narrowed-to-another-class receiver), and a non-`$this` receiver still faults, so a
            // genuinely external protected access keeps erroring.
            if matches!(visibility, Visibility::Protected)
                && matches!(receiver.kind, crate::parser::ast::ExprKind::This)
            {
                if let Some(current) = self.current_class.as_deref() {
                    if self.classes_are_instanceof_incompatible(current, declaring_class) {
                        return true;
                    }
                }
            }
            let Some(context) = self.bound_scope_context.as_ref() else {
                return false;
            };
            // A `Closure::bind(fn () => $this->prop, $newThis, Scope::class)` rebinds `$this` to
            // `$newThis` with `Scope` as the visibility scope. When the context authorizes the
            // `$this` receiver (only the single-`return $this->prop` shape codegen lowers directly),
            // the access is checked against `scope_class` exactly as PHP checks the rebound scope.
            if context.this_receiver_scope
                && matches!(receiver.kind, crate::parser::ast::ExprKind::This)
            {
                return match visibility {
                    Visibility::Public => true,
                    Visibility::Protected => {
                        context.scope_class == declaring_class
                            || self.is_subclass_of(&context.scope_class, declaring_class)
                            || self.is_subclass_of(declaring_class, &context.scope_class)
                    }
                    Visibility::Private => context.scope_class == declaring_class,
                };
            }
            let crate::parser::ast::ExprKind::Variable(name) = &receiver.kind else {
                return false;
            };
            let is_eligible_param = context.eligible_params.contains(name);
            let is_closure_local = !context.declared_params.contains(name)
                && !context.captured_variables.contains(name);
            if !is_eligible_param && !is_closure_local {
                return false;
            }
            match visibility {
                Visibility::Public => true,
                Visibility::Protected => {
                    context.scope_class == declaring_class
                        || self.is_subclass_of(&context.scope_class, declaring_class)
                }
                Visibility::Private => context.scope_class == declaring_class,
            }
        }
}
