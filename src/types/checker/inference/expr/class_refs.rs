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
    /// Validates `new` expressions on late-bound static constructor targets by inferring
    /// the object type for every concrete class that descends from `base_class`.
    ///
    /// Used when `$obj::new(...)` or similar late-bound constructor syntax is used,
    /// to ensure each possible runtime-instantiable class variant is well-typed. Abstract bases
    /// and descendants cannot be the late-bound runtime class of a successful construction.
    pub(super) fn validate_late_bound_constructor_targets(
        &mut self,
        base_class: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<(), CompileError> {
        let mut class_names: Vec<String> = self
            .classes
            .iter()
            .filter(|(name, info)| {
                !info.is_abstract && self.class_is_same_or_descends_from(name, base_class)
            })
            .map(|(name, _)| name.clone())
            .collect();
        class_names.sort();

        for class_name in class_names {
            if class_name != base_class
                && self.late_bound_target_cannot_bind(&class_name, args, env)?
            {
                continue;
            }
            self.infer_new_object_type(&class_name, args, expr, env)?;
        }

        Ok(())
    }

    /// Returns whether this late-bound descendant is not a construction candidate for these
    /// arguments, and so has nothing to say about whether the program is well typed.
    ///
    /// `new static` binds to the class the call was made ON, so a descendant is a SEPARATE call
    /// site that PHP decides at run time. Two shapes matter, and Symfony's
    /// `HttpException::fromStatusCode()` — which ends in
    /// `new static($statusCode, $message, $previous, $headers, $code)` — hits both:
    ///
    /// - SURPLUS: `AccessDeniedHttpException` declares four parameters. PHP discards the surplus
    ///   and runs, so this is not an error at all; the lowering reproduces it by dropping the
    ///   extra arguments (`static_new_droppable_surplus_args`).
    /// - STORAGE CONFLICT: `MethodNotAllowedHttpException` takes `array $allow` where the factory
    ///   passes an int. PHP raises a TypeError, but only for a program that actually calls
    ///   `MethodNotAllowedHttpException::fromStatusCode()`; the lowering drops the class from its
    ///   candidate set (`static_new_args_match_param_storage`) and the branch raises if reached.
    ///
    /// Too FEW arguments is deliberately NOT here: that is an `ArgumentCountError` every time the
    /// branch runs, so reporting it at compile time still describes the program truthfully.
    fn late_bound_target_cannot_bind(
        &mut self,
        class_name: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Result<bool, CompileError> {
        let Some(sig) = self
            .classes
            .get(class_name)
            .and_then(|class_info| class_info.methods.get("__construct"))
            .cloned()
        else {
            return Ok(false);
        };
        if args.iter().any(|arg| {
            matches!(
                arg.kind,
                crate::parser::ast::ExprKind::Spread(_)
                    | crate::parser::ast::ExprKind::NamedArg { .. }
            )
        }) {
            return Ok(false);
        }
        if !crate::func_args::sig_collects_surplus_args(&sig) && args.len() > sig.params.len() {
            return Ok(true);
        }
        for (index, arg) in args.iter().enumerate() {
            let Some((_, param_ty)) = sig.params.get(index) else {
                break;
            };
            let arg_ty = self.infer_type(arg, env)?;
            if late_bound_param_storage_conflicts(
                &param_ty.codegen_repr(),
                &arg_ty.codegen_repr(),
            ) {
                return Ok(true);
            }
        }
        Ok(false)
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
        if !self.scoped_constant_receiver_is_known(&class_name) {
            // PHP resolves classes lazily when this expression executes. Keep an unknown
            // receiver gradual so optional integrations in unexecuted paths remain compilable;
            // the EIR backend emits the runtime class-not-found fatal for an executed read.
            return Ok(PhpType::Mixed);
        }
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
                if let Some(type_expr) = info.constant_types.get(name).cloned() {
                    return self.resolve_type_expr(&type_expr, expr.span);
                }
                if let Some(value_expr) = info.constants.get(name).cloned() {
                    if let Some(reason) = info.constant_deprecations.get(name).cloned() {
                        let message = if reason.is_empty() {
                            format!("Use of deprecated class constant: {}::{}", cn, name)
                        } else {
                            format!(
                                "Use of deprecated class constant: {}::{} — {}",
                                cn, name, reason
                            )
                        };
                        self.warnings
                            .push(crate::errors::CompileWarning::new(expr.span, &message));
                    }
                    return self.infer_type(&value_expr, &TypeEnv::default());
                }
            }
            current_class = self.classes.get(cn).and_then(|i| i.parent.clone());
        }
        // Fallback: search implemented interfaces (and parent interfaces).
        if let Some(class_info) = self.classes.get(&class_name).cloned() {
            for iface_name in &class_info.interfaces {
                if let Some((value, type_expr)) = self.lookup_interface_constant(iface_name, name) {
                    if let Some(type_expr) = type_expr {
                        return self.resolve_type_expr(&type_expr, expr.span);
                    }
                    return self.infer_type(&value, &TypeEnv::default());
                }
            }
        }
        // Direct interface receiver (`Limits::MAX`).
        if let Some((value, type_expr)) = self.lookup_interface_constant(&class_name, name) {
            if let Some(type_expr) = type_expr {
                return self.resolve_type_expr(&type_expr, expr.span);
            }
            return self.infer_type(&value, &TypeEnv::default());
        }
        // On an enum, a `::name` that is neither a declared case nor a constant is an undefined
        // case — report that rather than the generic class-constant message.
        if self.enums.contains_key(&class_name) {
            return Err(CompileError::new(
                expr.span,
                &format!("Undefined enum case: {}::{}", class_name, name),
            ));
        }
        Err(CompileError::new(
            expr.span,
            &format!("Undefined class constant: {}::{}", class_name, name),
        ))
    }

    /// Infers a class constant read whose receiver is an object-valued expression.
    pub(crate) fn infer_dynamic_scoped_constant_access(
        &mut self,
        receiver: &Expr,
        name: &str,
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let receiver_type = self.infer_type(receiver, env)?;
        match receiver_type.codegen_repr() {
            PhpType::Object(class_name) => self.infer_scoped_constant_access(
                &StaticReceiver::Named(crate::names::Name::from(class_name)),
                name,
                expr,
            ),
            // PHP permits `$className::CONSTANT` with a runtime class-string. The concrete
            // class and constant are selected by the closed-world registry during lowering;
            // until then its value must remain gradual.
            PhpType::Str | PhpType::Mixed | PhpType::Union(_) => Ok(PhpType::Mixed),
            other => Err(CompileError::new(
                receiver.span,
                &format!(
                    "Dynamic class constant receiver must be an object or class-string, got {}",
                    other
                ),
            )),
        }
    }

    /// Returns whether a scoped-constant receiver is known in static class-like metadata.
    fn scoped_constant_receiver_is_known(&self, class_name: &str) -> bool {
        self.classes.contains_key(class_name)
            || self.interfaces.contains_key(class_name)
            || self.declared_traits.contains(class_name)
            || self.enums.contains_key(class_name)
    }

    /// Looks up a constant by name on an interface, traversing parent interfaces breadth-first
    /// to find it. Returns its value expression and optional declared type.
    fn lookup_interface_constant(
        &self,
        interface_name: &str,
        const_name: &str,
    ) -> Option<(crate::parser::ast::Expr, Option<crate::parser::ast::TypeExpr>)> {
        let mut visited = std::collections::HashSet::new();
        let mut queue: Vec<String> = vec![interface_name.to_string()];
        while let Some(name) = queue.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }
            if let Some(iface) = self.interfaces.get(&name) {
                if let Some(value) = iface.constants.get(const_name) {
                    return Some((
                        value.clone(),
                        iface.constant_types.get(const_name).cloned(),
                    ));
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

/// Returns whether two concrete representations can never hold one another's values.
///
/// Mirrors `crate::ir_lower::expr::static_new_param_storage_conflicts`, so the checker and the
/// lowering agree on which late-bound descendants are construction candidates.
fn late_bound_param_storage_conflicts(param: &PhpType, arg: &PhpType) -> bool {
    let array_like = |ty: &PhpType| matches!(ty, PhpType::Array(_) | PhpType::AssocArray { .. });
    let scalar_like = |ty: &PhpType| {
        matches!(
            ty,
            PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::False | PhpType::Str
        )
    };
    (array_like(param) && scalar_like(arg)) || (scalar_like(param) && array_like(arg))
}
