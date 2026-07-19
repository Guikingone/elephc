//! Purpose:
//! Infers expression class refs forms for the checker.
//! Handles type facts and diagnostics for expression shapes that need more than scalar/operator inference.
//!
//! Called from:
//! - `crate::types::checker::inference::expr`
//!
//! Key details:
//! - Expression inference shares environments with statement checking, so variable and effect updates must stay synchronized.

use crate::errors::CompileError;
use crate::parser::ast::{Expr, StaticReceiver};
use crate::span::Span;
use crate::types::{PhpType, TypeEnv};

use super::super::super::Checker;

impl Checker {
    /// Validates `new static(...)` late-bound constructor targets by inferring the object type
    /// for every CONCRETE class that is `base_class` or descends from it.
    ///
    /// `static` is late static binding: it resolves at runtime to the concrete class the method
    /// was *called* on, which can never be abstract. Abstract classes in the hierarchy are skipped
    /// so a valid `new static()` written inside an abstract class is not flagged with a false
    /// "cannot instantiate abstract class". When no concrete target exists (an abstract class with
    /// no concrete subclass in the closed world), the argument expressions are still inferred once
    /// so genuine errors inside them are not masked.
    pub(super) fn validate_late_bound_constructor_targets(
        &mut self,
        base_class: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<(), CompileError> {
        // `static` late-binds to the concrete class the method is *called* on, which can never be
        // abstract. Validate constructor args against every CONCRETE class in the hierarchy and skip
        // abstract ones — flagging an abstract base/descendant here is a false "cannot instantiate
        // abstract class" for the valid `new static()` pattern inside an abstract class.
        let mut class_names: Vec<String> = self
            .classes
            .keys()
            .filter(|name| {
                self.class_is_same_or_descends_from(name, base_class)
                    && !self
                        .classes
                        .get(name.as_str())
                        .map(|info| info.is_abstract)
                        .unwrap_or(false)
            })
            .cloned()
            .collect();
        class_names.sort();

        if class_names.is_empty() {
            // No concrete runtime target exists yet (e.g. an abstract class with no concrete
            // subclass in the closed world). Still infer the argument expressions once so errors
            // inside them are not masked; there is no concrete constructor to validate against.
            for arg in args {
                self.infer_type(arg, env)?;
            }
            return Ok(());
        }

        for class_name in class_names {
            self.infer_new_object_type(&class_name, args, expr, env)?;
        }

        Ok(())
    }

    /// Checks whether `class_name` is either `base_class` itself or a descendant of it
    /// by walking the parent chain.
    fn class_is_same_or_descends_from(&self, class_name: &str, base_class: &str) -> bool {
        let mut current = Some(class_name);
        while let Some(name) = current {
            if name == base_class {
                return true;
            }
            current = self
                .classes
                .get(name)
                .and_then(|info| info.parent.as_deref());
        }
        false
    }

    /// Infers the type of a class constant or enum case accessed via scope resolution
    /// (e.g., `MyClass::CONSTANT` or `Color::Red`).
    ///
    /// Searches the class/interface hierarchy for the named constant, preferring enum cases
    /// when the receiver is an enum. Falls back to interface constants and finally returns
    /// an error if the constant is not found.
    pub(crate) fn infer_scoped_constant_access(
        &mut self,
        receiver: &StaticReceiver,
        name: &str,
        expr: &Expr,
    ) -> Result<PhpType, CompileError> {
        let class_name = self.resolve_static_receiver_class(receiver, expr.span)?;
        self.infer_class_constant_type_by_name(&class_name, name, expr)
    }

    /// Infers the type of a class constant `$obj::CONST` accessed through an object or
    /// variable whose class is only known by type (closed world). The object expression is
    /// still evaluated for its side effects; the constant value itself is compile-time.
    ///
    /// Resolves a single concrete class from `object`'s inferred type (a lone `Object(T)`,
    /// or a union — such as a nullable object — that names exactly one class), then reuses
    /// the same class-constant lookup as `MyClass::CONST`. A value whose class is not
    /// statically a unique class (`Mixed`, a scalar, or a multi-class union) is a clear error.
    pub(crate) fn infer_dynamic_class_constant_access(
        &mut self,
        object: &Expr,
        name: &str,
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let object_type = self.infer_type(object, env)?;
        let class_name = unique_object_class_name(&object_type).ok_or_else(|| {
            CompileError::new(
                expr.span,
                &format!(
                    "Cannot resolve class constant `{}` on a value of type `{}`; \
                     the class must be statically known (a single object type)",
                    name, object_type
                ),
            )
        })?;
        self.infer_class_constant_type_by_name(&class_name, name, expr)
    }

    /// Shared class-constant / enum-case lookup by resolved class name, used by both
    /// `MyClass::CONST` (static receiver) and `$obj::CONST` (dynamic receiver). Prefers enum
    /// cases, then walks the class parent chain, then implemented/parent interfaces, and
    /// degrades an entirely-unknown class to `Mixed` with a warning (absent optional
    /// dependency). A missing constant on a known class is a hard error.
    fn infer_class_constant_type_by_name(
        &mut self,
        class_name: &str,
        name: &str,
        expr: &Expr,
    ) -> Result<PhpType, CompileError> {
        let class_name = class_name.to_string();
        // First: enum case access (`Color::Red`). Enums shadow classes for
        // this syntax in PHP since 8.1. A name that is not a declared case is an enum *constant*
        // (`Scale::FACTOR`), which is resolved through the class-constant table below.
        if let Some(enum_info) = self.enums.get(&class_name) {
            if enum_info.cases.iter().any(|case| case.name == name) {
                return self.infer_enum_case_type(&class_name, name, expr);
            }
        }
        // Walk parent chain to find a class constant.
        let mut current_class = Some(class_name.clone());
        while let Some(cn) = current_class.as_deref() {
            if let Some(info) = self.classes.get(cn) {
                if let Some(value_expr) = info.constants.get(name).cloned() {
                    return self.infer_const_value_type(&value_expr);
                }
            }
            current_class = self.classes.get(cn).and_then(|i| i.parent.clone());
        }
        // Fallback: search implemented interfaces (and parent interfaces).
        if let Some(class_info) = self.classes.get(&class_name).cloned() {
            for iface_name in &class_info.interfaces {
                if let Some(value) = self.lookup_interface_constant(iface_name, name) {
                    return self.infer_const_value_type(&value);
                }
            }
        }
        // Direct interface receiver (`Limits::MAX`).
        if let Some(value) = self.lookup_interface_constant(&class_name, name) {
            return self.infer_const_value_type(&value);
        }
        // On an enum, a `::name` that is neither a declared case nor a constant is an undefined
        // case — report that rather than the generic class-constant message.
        if self.enums.contains_key(&class_name) {
            return Err(CompileError::new(
                expr.span,
                &format!("Undefined enum case: {}::{}", class_name, name),
            ));
        }
        // Static constant access on a class that is unknown everywhere in the closed world is an
        // absent optional dependency (e.g. `Process::ERR`): degrade to `Mixed` with a warning
        // instead of erroring. A missing constant on a *known* class stays a hard error below.
        if !self.class_like_exists(&class_name) {
            self.warn_absent_class(expr.span, &class_name);
            return Ok(PhpType::Mixed);
        }
        Err(CompileError::new(
            expr.span,
            &format!("Undefined class constant: {}::{}", class_name, name),
        ))
    }

    /// Infers a class/interface constant's value type with `compile_time_const_depth`
    /// incremented: this is a genuinely compile-time-evaluated context (PHP itself rejects any
    /// function call in a class-constant initializer — "Constant expression contains invalid
    /// operations"), so the curated late-bound undefined-function carve-out
    /// (`functions::late_bound`) must not apply while inferring it, matching top-level `const`.
    fn infer_const_value_type(&mut self, value_expr: &Expr) -> Result<PhpType, CompileError> {
        self.compile_time_const_depth += 1;
        let result = self.infer_type(value_expr, &TypeEnv::default());
        self.compile_time_const_depth -= 1;
        result
    }

    /// Looks up a constant by name on an interface, traversing parent interfaces breadth-first
    /// to find it. Returns the constant's value expression if found.
    fn lookup_interface_constant(
        &self,
        interface_name: &str,
        const_name: &str,
    ) -> Option<crate::parser::ast::Expr> {
        let mut visited = std::collections::HashSet::new();
        let mut queue: Vec<String> = vec![interface_name.to_string()];
        while let Some(name) = queue.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }
            if let Some(iface) = self.interfaces.get(&name) {
                if let Some(value) = iface.constants.get(const_name) {
                    return Some(value.clone());
                }
                queue.extend(iface.parents.iter().cloned());
            }
        }
        None
    }

    /// Resolves a `StaticReceiver` to its canonical class name string.
    ///
    /// - `Named` returns the class name directly.
    /// - `Self_` / `Static` return the current class, or error if not inside a class.
    /// - `Parent` returns the parent of the current class, or error if there is no parent.
    fn resolve_static_receiver_class(
        &self,
        receiver: &StaticReceiver,
        span: Span,
    ) -> Result<String, CompileError> {
        match receiver {
            StaticReceiver::Named(name) => Ok(name.as_canonical()),
            StaticReceiver::Self_ | StaticReceiver::Static => self
                .current_class
                .clone()
                .ok_or_else(|| CompileError::new(span, "Cannot use self:: outside a class context")),
            StaticReceiver::Parent => {
                let current = self.current_class.as_ref().ok_or_else(|| {
                    CompileError::new(span, "Cannot use parent:: outside a class context")
                })?;
                self.classes
                    .get(current)
                    .and_then(|info| info.parent.clone())
                    .ok_or_else(|| {
                        CompileError::new(
                            span,
                            &format!("Class '{}' has no parent class", current),
                        )
                    })
            }
        }
    }

    /// Validates that `self::class`, `static::class`, or `parent::class` is used in an
    /// appropriate class context. Returns an error for invalid scope (e.g., outside a class
    /// or on a class with no parent for `parent::class`).
    pub(super) fn validate_class_constant_receiver(
        &self,
        receiver: &StaticReceiver,
        span: Span,
    ) -> Result<(), CompileError> {
        match receiver {
            StaticReceiver::Named(_) => Ok(()),
            StaticReceiver::Self_ | StaticReceiver::Static => {
                if self.current_class.is_some() {
                    Ok(())
                } else {
                    Err(CompileError::new(
                        span,
                        "Cannot use self::class or static::class outside a class context",
                    ))
                }
            }
            StaticReceiver::Parent => {
                let current = self.current_class.as_ref().ok_or_else(|| {
                    CompileError::new(
                        span,
                        "Cannot use parent::class outside a class context",
                    )
                })?;
                if self
                    .classes
                    .get(current)
                    .and_then(|info| info.parent.as_ref())
                    .is_some()
                {
                    Ok(())
                } else {
                    Err(CompileError::new(
                        span,
                        &format!("Class '{}' has no parent class", current),
                    ))
                }
            }
        }
    }
}

/// Returns the single concrete class name named by an object-typed value, or `None` when the
/// type does not resolve to exactly one class. `Object(T)`/`Packed(T)` yield `T`; a `Union`
/// (e.g. a nullable object `T|null`) yields the class only when it names exactly one distinct
/// class across its members. Scalars, `Mixed`, and multi-class unions return `None`, which the
/// caller turns into a clear "class must be statically known" error.
fn unique_object_class_name(ty: &PhpType) -> Option<String> {
    let mut names = std::collections::BTreeSet::new();
    collect_object_class_names(ty, &mut names);
    if names.len() == 1 {
        names.into_iter().next()
    } else {
        None
    }
}

/// Accumulates every distinct object/packed class name reachable in `ty`, descending into
/// `Union` members. Non-object members contribute nothing so a nullable object still resolves.
fn collect_object_class_names(ty: &PhpType, names: &mut std::collections::BTreeSet<String>) {
    match ty {
        // An empty class name is an unknown/untyped object (`Object("")`), which is not a
        // statically-known class; skip it so the receiver resolves to "unresolvable".
        PhpType::Object(name) | PhpType::Packed(name) if !name.is_empty() => {
            names.insert(name.clone());
        }
        PhpType::Union(members) => {
            for member in members {
                collect_object_class_names(member, names);
            }
        }
        _ => {}
    }
}
