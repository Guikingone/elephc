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
    /// are accessible if the current class is the declaring class or a subclass; private
    /// members are only accessible if the current class is exactly the declaring class.
    pub(crate) fn can_access_member(
        &self,
        declaring_class: &str,
        visibility: &Visibility,
    ) -> bool {
        match visibility {
            Visibility::Public => true,
            Visibility::Protected => self.current_class.as_deref().is_some_and(|current| {
                current == declaring_class || self.is_subclass_of(current, declaring_class)
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
    pub(crate) fn propagate_constructor_arg_type(
        &mut self,
        instantiated_class: &str,
        param_index: usize,
        arg_ty: &PhpType,
        param_has_declared_type: bool,
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

            if !param_has_declared_type {
                if let Some(sig) = class_info.methods.get_mut("__construct") {
                    if let Some((_, param_ty)) = sig.params.get_mut(param_index) {
                        *param_ty = arg_ty.clone();
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
        /// Returns true if the declared return `expected` (possibly nullable/union) contains at
        /// least one `Object(D)` arm where `actual` (a concrete `Object(B)`) is a proper ANCESTOR
        /// of `D` — i.e. `D` is a subclass of `B`, or `D` implements `B` as an interface. This is
        /// the "checked downcast on return" relaxation: PHP allows a function to declare a return
        /// type narrower than a value it's statically only known to be a supertype of, deferring
        /// the real check to a runtime `instanceof` guard. `crate::ir_lower::stmt::return_type_guard`
        /// mirrors this exact predicate over its own class-hierarchy metadata to decide whether to
        /// emit that guard, so the two MUST stay in lock-step: this only widens acceptance, it never
        /// substitutes for the runtime check.
        ///
        /// Only meant to be tried as a fallback AFTER normal covariant acceptance
        /// (`require_compatible_arg_type`) has already failed for this `(expected, actual)` pair —
        /// it does not special-case an `actual` that already satisfies `expected` normally.
        pub(crate) fn object_return_downcast_guardable(&self, expected: &PhpType, actual: &PhpType) -> bool {
            let PhpType::Object(actual_name) = actual else {
                return false;
            };
            flatten_type_arms(expected).into_iter().any(|arm| match arm {
                PhpType::Object(declared_name) => {
                    declared_name != *actual_name
                        && (self.is_subclass_of(&declared_name, actual_name)
                            || self.object_type_implements_interface(&declared_name, actual_name))
                }
                _ => false,
            })
        }
}

impl Checker {
        /// Checks whether `receiver` can access a member with the given visibility declared in
        /// `declaring_class`, additionally consulting an active `Closure::bind`/`bindTo` scope
        /// rebind (`Checker::bound_scope_context`) when the normal `can_access_member` check fails.
        ///
        /// The bound-scope override applies ONLY when `receiver` is exactly `ExprKind::Variable(name)`
        /// naming one of the active rebind's `eligible_params` (JURY ADDENDUM #2: "->prop writes/reads
        /// allowed only on PARAMETERS whose declared type equals (or is a subclass of) the rebound
        /// scope class") — never for a computed expression, a captured variable, or `$this` (which the
        /// lexical gate that populates `bound_scope_context` already proved absent from the body).
        pub(crate) fn can_access_property(
            &self,
            receiver: &Expr,
            declaring_class: &str,
            visibility: &Visibility,
        ) -> bool {
            if self.can_access_member(declaring_class, visibility) {
                return true;
            }
            let Some(context) = self.bound_scope_context.as_ref() else {
                return false;
            };
            let crate::parser::ast::ExprKind::Variable(name) = &receiver.kind else {
                return false;
            };
            if !context.eligible_params.contains(name) {
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

/// Flattens a possibly-nested `Union` into its member arms; a non-union type is a single arm.
/// Shared by `Checker::object_return_downcast_guardable` and its `ir_lower` mirror.
pub(crate) fn flatten_type_arms(ty: &PhpType) -> Vec<PhpType> {
    match ty {
        PhpType::Union(members) => members.iter().flat_map(flatten_type_arms).collect(),
        other => vec![other.clone()],
    }
}
