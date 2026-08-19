//! Purpose:
//! Infers object methods expression types.
//! Validates class, method, constructor, property, and magic-access contracts against schema metadata.
//!
//! Called from:
//! - `crate::types::checker::inference::objects`
//!
//! Key details:
//! - Object inference depends on flattened class metadata, visibility, inheritance, and declared property types.

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{Expr, ExprKind, StaticReceiver, TypeExpr};
use crate::types::{FunctionSig, PhpType, TypeEnv};

use super::super::super::Checker;
use super::super::syntactic::wider_type_syntactic;

impl Checker {
    /// Infers the type of a method call expression (`$obj->method(...)`).
    ///
    /// Dispatches to `infer_method_call_on_class_type` for `Object` types,
    /// `infer_method_call_on_interface_type` for interface types, and
    /// handles nullable union receivers. A `Mixed` receiver dispatches at
    /// runtime over the classes that declare the method, so its result is the
    /// union of those candidates' return types (see
    /// `mixed_receiver_method_return_type`). Other unhandled receiver types fall
    /// back to `PhpType::Int`.
    pub(crate) fn infer_method_call_type(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let obj_ty = self.infer_type(object, env)?;
        if let PhpType::Object(class_name) = &obj_ty {
            if class_name.is_empty() {
                for arg in args {
                    self.infer_type(arg, env)?;
                }
                return Ok(self
                    .mixed_receiver_method_return_type(method, args.len())
                    .unwrap_or(PhpType::Mixed));
            }
            if self.interfaces.contains_key(class_name) {
                return self
                    .infer_method_call_on_interface_type(class_name, method, args, expr, env);
            }
            let return_ty = self.infer_method_call_on_class_type(class_name, method, args, expr, env)?;
            return Ok(self
                .tracked_reflection_class_method_return_type(object, method)
                .unwrap_or(return_ty));
        }
        // Method calls on a union object type are allowed when the union has a
        // single object class. `?Foo` / `Foo|null` faults on a null receiver as in
        // PHP; `Foo|false` (and other object-plus-scalar unions) dispatch on the
        // runtime class id and fault when the value is not an object. Either way
        // the checker surfaces the method's return type so callers can chain.
        if let PhpType::Union(_) = &obj_ty {
            let class_name = self.union_single_object_class(&obj_ty).or_else(|| {
                self.nullsafe_object_receiver(&obj_ty, expr, "method call")
                    .ok()
                    .flatten()
                    .map(|(name, _nullable)| name)
            });
            if let Some(class_name) = class_name {
                if self.interfaces.contains_key(&class_name) {
                    return self.infer_method_call_on_interface_type(
                        &class_name,
                        method,
                        args,
                        expr,
                        env,
                    );
                }
                let return_ty =
                    self.infer_method_call_on_class_type(&class_name, method, args, expr, env)?;
                return Ok(self
                    .tracked_reflection_class_method_return_type(object, method)
                    .unwrap_or(return_ty));
            }
            // Union of two or more distinct object classes (`A|B`, `A|B|false`):
            // codegen dispatches on the runtime class id. Validate every member
            // that can receive the call and merge their return types; a runtime
            // member without that method faults like PHP instead of making the
            // entire union a compile-time error.
            let object_classes = self.union_object_classes(&obj_ty);
            if object_classes.len() >= 2 {
                let mut return_types = Vec::with_capacity(object_classes.len());
                for class_name in &object_classes {
                    let supports_method = self
                        .classes
                        .get(class_name)
                        .is_some_and(|info| {
                            info.methods.contains_key(&php_symbol_key(method))
                                || info.methods.contains_key("__call")
                        })
                        || self
                            .interfaces
                            .get(class_name)
                            .is_some_and(|info| {
                                info.methods.contains_key(&php_symbol_key(method))
                            });
                    if !supports_method {
                        continue;
                    }
                    let return_ty = if self.interfaces.contains_key(class_name) {
                        self.infer_method_call_on_interface_type(class_name, method, args, expr, env)?
                    } else {
                        self.infer_method_call_on_class_type(class_name, method, args, expr, env)?
                    };
                    return_types.push(return_ty);
                }
                if !return_types.is_empty() {
                    return Ok(self.normalize_union_type(return_types));
                }
            }
            // No object class at all: re-run the strict check to surface its
            // diagnostic.
            self.nullsafe_object_receiver(&obj_ty, expr, "method call")?;
        }
        // Closure rebinding methods on a callable receiver. `bindTo` rebinds
        // `$this` and returns a new closure; `call` binds `$this` and invokes the
        // closure in one step, returning its result. `$scope` is accepted and
        // ignored (visibility is resolved at compile time).
        if matches!(obj_ty, PhpType::Callable) {
            match php_symbol_key(method).as_str() {
                "bindto" => {
                    for arg in args {
                        self.infer_type(arg, env)?;
                    }
                    return Ok(PhpType::Callable);
                }
                "call" => {
                    for arg in args {
                        self.infer_type(arg, env)?;
                    }
                    return Ok(PhpType::Mixed);
                }
                _ => {}
            }
        }
        // A method call on a `mixed` receiver dispatches on the runtime class id
        // over exactly the classes that declare the method (see
        // `mixed_method_candidates` / `lower_mixed_method_call` and the
        // Mixed-receiver method emission in `ir_lower::program`). The static
        // result is therefore the union of those candidates' return types. When
        // they all agree on one type, codegen stores the call result raw (no
        // boxing), so the precise type is correct; when they differ it is a
        // union, which codegen boxes like the two-class union dispatch. Returning
        // the historical `Int` fallback here instead made an *inferred* function
        // return type silently coerce a boxed result: an un-annotated
        // `function f($x) { return $x->name(); }` rendered the returned string as
        // `0`. With no declaring class the runtime would fatal, so `mixed` is the
        // safe static result.
        if matches!(obj_ty, PhpType::Mixed) {
            return Ok(self
                .mixed_receiver_method_return_type(method, args.len())
                .unwrap_or(PhpType::Mixed));
        }
        Ok(PhpType::Int)
    }

    /// Computes the static return type of a method call on a `mixed` receiver as
    /// the normalized union of the declared return types of every class that
    /// declares `method` with a matching arity. This mirrors the runtime
    /// candidate set used by `mixed_method_candidates` in codegen, so the
    /// inferred type stays consistent with how each candidate branch stores its
    /// result. Nominal object results degrade to `mixed`, because a runtime-only
    /// class may implement the same method with an unrelated object result.
    /// Falls back to the name-only candidate set when arity filtering finds
    /// nothing (e.g. methods with default parameters), and returns `None` when no
    /// class declares the method at all.
    fn mixed_receiver_method_return_type(&self, method: &str, arg_count: usize) -> Option<PhpType> {
        let method_key = php_symbol_key(method);
        let mut arity_matched: Vec<PhpType> = Vec::new();
        let mut any_matched: Vec<PhpType> = Vec::new();
        for class_info in self.classes.values() {
            let Some(sig) = class_info.methods.get(&method_key) else {
                continue;
            };
            let ty = sig.return_type.clone();
            if !any_matched.contains(&ty) {
                any_matched.push(ty.clone());
            }
            if sig.params.len() == arg_count && !arity_matched.contains(&ty) {
                arity_matched.push(ty);
            }
        }
        let candidates = if arity_matched.is_empty() {
            any_matched
        } else {
            arity_matched
        };
        if candidates.is_empty() {
            None
        } else {
            let normalized = self.normalize_union_type(candidates);
            match &normalized {
                PhpType::Object(_) => Some(PhpType::Mixed),
                PhpType::Union(members)
                    if members
                        .iter()
                        .any(|member| matches!(member, PhpType::Object(_))) =>
                {
                    Some(PhpType::Mixed)
                }
                _ => Some(normalized),
            }
        }
    }

    /// Returns a concrete reflected object type for tracked `ReflectionClass` construction helpers.
    fn tracked_reflection_class_method_return_type(
        &self,
        object: &Expr,
        method: &str,
    ) -> Option<PhpType> {
        let ExprKind::Variable(name) = &object.kind else {
            return None;
        };
        let reflected_class = self.reflection_class_targets.get(name)?;
        match php_symbol_key(method).as_str() {
            "newinstance" | "newinstanceargs" | "newinstancewithoutconstructor" => {
                Some(PhpType::Object(reflected_class.clone()))
            }
            _ => None,
        }
    }

    /// Infers a method return through the interface or concrete-class checker for a resolved
    /// receiver name.
    fn infer_method_return_on_class_or_interface(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        if self.interfaces.contains_key(class_name) {
            self.infer_method_call_on_interface_type(class_name, method, args, expr, env)
        } else {
            self.infer_method_call_on_class_type(class_name, method, args, expr, env)
        }
    }

    /// Infers the type of a nullsafe method call expression (`$obj?->method(...)`).
    ///
    /// A union with one object class keeps that class available for method validation even when
    /// gradual scalar members are also present. A nullable receiver adds `Void` to the result;
    /// non-object runtime members retain PHP's runtime failure behavior.
    pub(crate) fn infer_nullsafe_method_call_type(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let obj_ty = self.infer_type(object, env)?;
        if matches!(obj_ty, PhpType::Mixed)
            || matches!(&obj_ty, PhpType::Union(members) if members.iter().any(|member| *member == PhpType::Mixed))
        {
            return Ok(PhpType::Mixed);
        }
        if matches!(&obj_ty, PhpType::Union(_)) {
            if let Some(class_name) = self.union_single_object_class(&obj_ty) {
                let return_ty = self.infer_method_return_on_class_or_interface(
                    &class_name,
                    method,
                    args,
                    expr,
                    env,
                )?;
                let return_ty = self
                    .tracked_reflection_attribute_new_instance_return_type(object, method)
                    .unwrap_or(return_ty);
                let nullable = matches!(&obj_ty, PhpType::Union(members)
                    if members.iter().any(|member| *member == PhpType::Void));
                return if nullable {
                    Ok(self.normalize_union_type(vec![return_ty, PhpType::Void]))
                } else {
                    Ok(return_ty)
                };
            }
        }
        let Some((class_name, nullable)) =
            self.nullsafe_object_receiver(&obj_ty, expr, "method call")?
        else {
            return Ok(PhpType::Void);
        };
        let return_ty = self.infer_method_return_on_class_or_interface(
            &class_name,
            method,
            args,
            expr,
            env,
        )?;
        let return_ty = self
            .tracked_reflection_attribute_new_instance_return_type(object, method)
            .unwrap_or(return_ty);
        if nullable {
            Ok(self.normalize_union_type(vec![return_ty, PhpType::Void]))
        } else {
            Ok(return_ty)
        }
    }

    /// Recovers the concrete result of `ReflectionAttribute::newInstance()` from a filtered
    /// `getAttributes(ClassName::class)` receiver chain.
    ///
    /// The synthetic reflection signature is necessarily `mixed`, but a selected attribute whose
    /// filter name is statically known can only instantiate that class. This covers both direct
    /// array access and the common `($r->getAttributes(Foo::class)[0] ?? null)?->newInstance()`
    /// shape while leaving unfiltered or dynamic reflection attributes conservative.
    fn tracked_reflection_attribute_new_instance_return_type(
        &self,
        object: &Expr,
        method: &str,
    ) -> Option<PhpType> {
        if php_symbol_key(method) != "newinstance" {
            return None;
        }
        let selected = match &object.kind {
            ExprKind::NullCoalesce { value, .. } | ExprKind::ShortTernary { value, .. } => {
                value.as_ref()
            }
            _ => object,
        };
        let ExprKind::ArrayAccess { array, .. } = &selected.kind else {
            return None;
        };
        let ExprKind::MethodCall {
            method: owner_method,
            args,
            ..
        } = &array.kind
        else {
            return None;
        };
        if php_symbol_key(owner_method) != "getattributes" {
            return None;
        }
        let filter = args.first()?;
        let filter = match &filter.kind {
            ExprKind::NamedArg { value, .. } => value.as_ref(),
            _ => filter,
        };
        let class_name = match &filter.kind {
            ExprKind::StringLiteral(name) => name.trim_start_matches('\\').to_string(),
            ExprKind::ClassConstant { receiver } => match receiver {
                StaticReceiver::Named(name) => name.as_canonical(),
                StaticReceiver::Self_ | StaticReceiver::Static => self.current_class.clone()?,
                StaticReceiver::Parent => {
                    let current = self.current_class.as_ref()?;
                    self.classes.get(current)?.parent.clone()?
                }
            },
            _ => return None,
        };
        Some(PhpType::Object(class_name))
    }

    /// Infers `$obj?->$method(...)` when the method name is known only at runtime.
    ///
    /// The receiver must still be an object-or-null like other nullsafe member
    /// accesses. The dynamic target prevents method signature validation, so the
    /// return type is `Mixed`; method-name and argument expressions are inferred
    /// only when the receiver is not statically null.
    pub(crate) fn infer_nullsafe_dynamic_method_call_type(
        &mut self,
        object: &Expr,
        method: &Expr,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let obj_ty = self.infer_type(object, env)?;
        let Some((_class_name, _nullable)) =
            self.nullsafe_object_receiver(&obj_ty, expr, "method call")?
        else {
            return Ok(PhpType::Void);
        };
        self.infer_type(method, env)?;
        for arg in args {
            self.infer_type(arg, env)?;
        }
        Ok(PhpType::Mixed)
    }

    /// Infers the type of a method call on an interface type.
    ///
    /// Looks up the method in the interface schema, validates arguments via
    /// `normalize_named_call_args` and `check_known_callable_call`, and
    /// returns the declared return type.
    pub(crate) fn infer_method_call_on_interface_type(
        &mut self,
        interface_name: &str,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let method_key = php_symbol_key(method);
        let resolved_method = self
            .interfaces
            .get(interface_name)
            .and_then(|interface_info| {
                interface_info
                    .methods
                    .get(&method_key)
                    .map(|sig| (sig, false))
                    .or_else(|| {
                        interface_info
                            .static_methods
                            .get(&method_key)
                            .map(|sig| (sig, true))
                    })
            })
            .map(|(sig, is_static)| (sig.clone(), is_static));
        let Some((sig, is_static)) = resolved_method else {
            return self.infer_lenient_subtype_method_call(
                interface_name,
                method,
                &method_key,
                args,
                expr,
                env,
            );
        };
        let normalized_args = self.normalize_named_call_args(
            &sig,
            args,
            expr.span,
            &format!("Method {}::{}", interface_name, method),
            env,
        )?;
        self.check_user_declared_call(
            &sig,
            &normalized_args,
            expr.span,
            env,
            &format!("Method {}::{}", interface_name, method),
            interface_name,
        )?;
        let late_static_return = if is_static {
            self.static_method_late_static_return(interface_name, &method_key)
        } else {
            self.instance_method_late_static_return(interface_name, &method_key)
        };
        match late_static_return {
            Some(return_type) => self.resolve_late_static_return_type_hint(
                &return_type,
                interface_name,
                expr.span,
            ),
            None => Ok(sig.return_type),
        }
    }

    /// Infers the type of a method call on a class type.
    ///
    /// Looks up the method in the class schema, checks deprecation warnings,
    /// validates visibility, normalizes named arguments, validates the
    /// callable signature, and updates the method's parameter types from
    /// argument types (for local type inference). Handles `__call` magic
    /// methods and falls back to `PhpType::Int`.
    pub(crate) fn infer_method_call_on_class_type(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        self.infer_method_call_on_class_type_with_options(
            class_name, method, args, expr, env, false,
        )
    }

    /// Accepts a method absent from a nominal receiver when a compatible concrete subtype owns it.
    ///
    /// PHP dispatches on the runtime class. The closed-world backend mirrors that behavior with a
    /// class-id switch, so the checker derives the call result from the same compatible candidates.
    fn infer_lenient_subtype_method_call(
        &mut self,
        receiver_type: &str,
        method: &str,
        method_key: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let return_types = self.subtype_dispatch_return_types(receiver_type, method_key);
        if return_types.is_empty() {
            return Err(CompileError::new(
                expr.span,
                &format!("Undefined method: {}::{}", receiver_type, method),
            ));
        }
        for arg in args {
            self.infer_type(arg, env)?;
        }
        Ok(match return_types.as_slice() {
            [only] => only.clone(),
            _ => PhpType::Mixed,
        })
    }

    /// Collects distinct return types from compatible concrete classes declaring a method.
    fn subtype_dispatch_return_types(&self, receiver_type: &str, method_key: &str) -> Vec<PhpType> {
        let mut return_types = Vec::new();
        for (class_name, class_info) in &self.classes {
            let Some(sig) = class_info.methods.get(method_key) else {
                continue;
            };
            let is_compatible = class_name == receiver_type
                || self.is_subclass_of(class_name, receiver_type)
                || self.class_implements_interface(class_name, receiver_type);
            if is_compatible && !return_types.contains(&sig.return_type) {
                return_types.push(sig.return_type.clone());
            }
        }
        return_types
    }

    /// Infers a class method call for descriptor-backed callback paths that can
    /// preserve by-reference spread arguments through runtime invoker metadata.
    pub(crate) fn infer_method_call_on_class_type_allowing_by_ref_spread(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        self.infer_method_call_on_class_type_with_options(class_name, method, args, expr, env, true)
    }

    /// Shared implementation for class method call inference.
    fn infer_method_call_on_class_type_with_options(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
        allow_by_ref_spread: bool,
    ) -> Result<PhpType, CompileError> {
        let method_key = php_symbol_key(method);
        let late_static_return_type = self
            .instance_method_late_static_return(class_name, &method_key)
            .map(|return_type| {
                self.resolve_late_static_return_type_hint(&return_type, class_name, expr.span)
            })
            .transpose()?;
        let mut normalized_args = args.to_vec();
        let mut magic_return_ty = None;
        let mut magic_original_args = None;
        if let Some(class_info) = self.classes.get(class_name) {
            if let Some(sig) = class_info.methods.get(&method_key) {
                if let Some(reason) = sig.deprecation.clone() {
                    let message = if reason.is_empty() {
                        format!("Call to deprecated method: {}::{}()", class_name, method)
                    } else {
                        format!(
                            "Call to deprecated method: {}::{}() — {}",
                            class_name, method, reason
                        )
                    };
                    self.warnings
                        .push(crate::errors::CompileWarning::new(expr.span, &message));
                }
                if let Some(visibility) = class_info.method_visibilities.get(&method_key) {
                    let declaring_class = class_info
                        .method_declaring_classes
                        .get(&method_key)
                        .map(String::as_str)
                        .unwrap_or(class_name);
                    if !self.can_access_member(declaring_class, visibility)
                        && !self.can_access_pdo_prelude_internal_method(class_name, &method_key)
                    {
                        // PHP raises this as a catchable `Error` at runtime instead of a
                        // compile-time rejection. Record the throw site so EIR lowering
                        // emits the throw sequence, and continue with the declared return
                        // type so later passes stay type-consistent.
                        self.throw_access_sites.insert(
                            (self.current_loop_storage_scope.clone(), expr.span),
                            crate::types::ThrowAccessInfo {
                                span: expr.span,
                                kind: crate::types::ThrowAccessKind::PrivateMethod {
                                    visibility: match visibility {
                                        crate::parser::ast::Visibility::Private => {
                                            "private".to_string()
                                        }
                                        crate::parser::ast::Visibility::Protected => {
                                            "protected".to_string()
                                        }
                                        _ => "private".to_string(),
                                    },
                                    class_name: class_name.to_string(),
                                    method: method.to_string(),
                                },
                            },
                        );
                        return Ok(late_static_return_type
                            .clone()
                            .unwrap_or_else(|| sig.return_type.clone()));
                    }
                }
                let declared_flags =
                    Self::declared_method_param_flags(class_info, &method_key, false);
                let mut effective_sig =
                    Self::callable_sig_for_declared_params(sig, &declared_flags);
                if method_key == "__call" {
                    Self::relax_magic_call_validation_sig(&mut effective_sig);
                }
                normalized_args = self.normalize_named_call_args(
                    &effective_sig,
                    args,
                    expr.span,
                    &format!("Method {}::{}", class_name, method),
                    env,
                )?;
                if allow_by_ref_spread {
                    self.check_user_declared_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Method {}::{}", class_name, method),
                        class_name,
                    )?;
                } else {
                    self.check_user_declared_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Method {}::{}", class_name, method),
                        class_name,
                    )?;
                }
            } else if class_info.static_methods.contains_key(&method_key) {
                let receiver = StaticReceiver::Named(crate::names::Name::from(
                    class_name.to_string(),
                ));
                return self.infer_static_method_call_type_with_options(
                    &receiver,
                    method,
                    args,
                    expr,
                    env,
                    allow_by_ref_spread,
                );
            } else if let Some(sig) = class_info.methods.get("__call") {
                let magic_args = Self::magic_call_args(method, args, expr.span);
                let declared_flags = Self::declared_method_param_flags(class_info, "__call", false);
                let mut effective_sig =
                    Self::callable_sig_for_declared_params(sig, &declared_flags);
                Self::relax_magic_call_validation_sig(&mut effective_sig);
                normalized_args = self.normalize_named_call_args(
                    &effective_sig,
                    &magic_args,
                    expr.span,
                    &format!("Method {}::__call", class_name),
                    env,
                )?;
                if allow_by_ref_spread {
                    self.check_user_declared_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Method {}::__call", class_name),
                        class_name,
                    )?;
                } else {
                    self.check_user_declared_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Method {}::__call", class_name),
                        class_name,
                    )?;
                }
                magic_return_ty = Some(effective_sig.return_type.clone());
                magic_original_args = Some(args.to_vec());
            } else {
                return self.infer_lenient_subtype_method_call(
                    class_name,
                    method,
                    &method_key,
                    args,
                    expr,
                    env,
                );
            }
        }
        if let Some(return_ty) = magic_return_ty {
            if let Some(args) = magic_original_args {
                self.specialize_magic_call_signature(class_name, &args, env)?;
            }
            return Ok(return_ty);
        }
        let mut arg_types = Vec::new();
        for arg in &normalized_args {
            arg_types.push(self.infer_type(arg, env)?);
        }

        let impl_class_name = self
            .classes
            .get(class_name)
            .and_then(|class_info| class_info.method_impl_classes.get(&method_key))
            .cloned()
            .unwrap_or_else(|| class_name.to_string());
        let declared_flags = self
            .classes
            .get(&impl_class_name)
            .map(|class_info| Self::declared_method_param_flags(class_info, &method_key, false))
            .unwrap_or_default();
        if let Some(class_info) = self.classes.get_mut(&impl_class_name) {
            if let Some(sig) = class_info.methods.get_mut(&method_key) {
                let regular_param_count = if sig.variadic.is_some() {
                    sig.params.len().saturating_sub(1)
                } else {
                    sig.params.len()
                };
                for (i, arg_ty) in arg_types.iter().enumerate() {
                    if i < regular_param_count
                        && !declared_flags.get(i).copied().unwrap_or(false)
                        && !matches!(*arg_ty, PhpType::Void | PhpType::Never | PhpType::Callable)
                    {
                        let key = (format!("{}::{}", impl_class_name, method_key), i);
                        let seen = self.param_specialization_seen.contains(&key);
                        if sig.params[i].1 == PhpType::Int && !seen {
                            self.param_specialization_seen.insert(key);
                            sig.params[i].1 = arg_ty.clone();
                        } else {
                            sig.params[i].1 = Self::union_param_type(&sig.params[i].1, arg_ty);
                        }
                    }
                }
                let variadic_is_declared = declared_flags
                    .get(regular_param_count)
                    .copied()
                    .unwrap_or(false);
                if !variadic_is_declared
                    && method_variadic_tail_needs_iterable(
                        &normalized_args,
                        sig,
                        regular_param_count,
                        env,
                    )
                    && !method_variadic_param_is_by_ref(sig)
                {
                    if let Some((_, variadic_ty)) = sig.params.last_mut() {
                        *variadic_ty = PhpType::Iterable;
                    }
                } else if !variadic_is_declared
                    && sig.variadic.is_some()
                    && arg_types.len() > regular_param_count
                    && !method_variadic_param_is_by_ref(sig)
                    // A declared element type on the variadic (`mixed ...$xs`, `int ...$xs`) is
                    // the contract, exactly like a declared regular parameter above: call-site
                    // arguments are validated against it and must never narrow it. Without this
                    // guard `mixed ...$xs` was rewritten to the widened argument type, and a
                    // later checker pass then rejected the very call that produced it.
                    && !declared_flags.get(regular_param_count).copied().unwrap_or(false)
                {
                    let mut elem_ty = arg_types[regular_param_count].clone();
                    for arg_ty in arg_types.iter().skip(regular_param_count + 1) {
                        elem_ty = wider_type_syntactic(&elem_ty, arg_ty);
                    }
                    if let Some((_, PhpType::Array(existing_elem_ty))) = sig.params.last_mut() {
                        **existing_elem_ty =
                            wider_type_syntactic(existing_elem_ty.as_ref(), &elem_ty);
                    }
                }
                return Ok(late_static_return_type
                    .clone()
                    .unwrap_or_else(|| sig.return_type.clone()));
            }
        }
        // PHP permits unresolved nominal types in declarations. When such a receiver reaches a
        // method call, no declaration is available to determine its return representation, so a
        // boxed gradual result is the only sound fallback. The historical integer fallback could
        // corrupt a nullable chain into `int|null` and reject the following member access.
        Ok(PhpType::Mixed)
    }

    /// Returns preserved late-static return syntax for an instance method.
    fn instance_method_late_static_return(
        &self,
        receiver_type: &str,
        method_key: &str,
    ) -> Option<TypeExpr> {
        if let Some(class_info) = self.classes.get(receiver_type) {
            if let Some(return_type) = class_info.late_static_method_returns.get(method_key) {
                return Some(return_type.clone());
            }
        }
        self.interfaces
            .get(receiver_type)
            .and_then(|interface_info| interface_info.late_static_method_returns.get(method_key))
            .cloned()
    }

    /// Builds synthetic `__call` arguments: `[method_name, [args...]]`.
    ///
    /// Constructs a `StringLiteral` for the method name and an `ArrayLiteral`
    /// of the original arguments, used when forwarding to `__call`.
    fn magic_call_args(method: &str, args: &[Expr], span: crate::span::Span) -> Vec<Expr> {
        vec![
            Expr::new(ExprKind::StringLiteral(method.to_string()), span),
            Expr::new(ExprKind::ArrayLiteral(args.to_vec()), span),
        ]
    }

    /// Specializes `__call`'s second parameter (the args array) type based on
    /// the actual call arguments' inferred types.
    ///
    /// Merges all argument types into an element type, then updates the
    /// `__call` signature's params[1] (the array parameter) accordingly,
    /// respecting `declared_flags` and avoiding widening to `Mixed` when
    /// the declared type is already `Mixed`.
    fn specialize_magic_call_signature(
        &mut self,
        class_name: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Result<(), CompileError> {
        self.specialize_magic_dispatch_signature(class_name, "__call", false, args, env)
    }

    /// Refines a `__callStatic($name, $args)` signature's array parameter from
    /// the actual static-call arguments, the static counterpart of
    /// `specialize_magic_call_signature`.
    fn specialize_magic_static_call_signature(
        &mut self,
        class_name: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Result<(), CompileError> {
        self.specialize_magic_dispatch_signature(class_name, "__callstatic", true, args, env)
    }

    /// Shared body for `__call`/`__callStatic` argument-array specialization.
    ///
    /// Merges all argument types into an element type, then updates the magic
    /// method signature's params[1] (the array parameter) on its implementing
    /// class, selecting the instance or static method tables via `is_static`.
    fn specialize_magic_dispatch_signature(
        &mut self,
        class_name: &str,
        method_key: &str,
        is_static: bool,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Result<(), CompileError> {
        let mut elem_ty = PhpType::Never;
        for arg in args {
            let arg_ty = self.infer_type(arg, env)?;
            elem_ty = Self::merge_magic_call_arg_type(elem_ty, arg_ty);
        }
        let args_array_ty = PhpType::Array(Box::new(elem_ty.clone()));
        let impl_class_name = self
            .classes
            .get(class_name)
            .and_then(|class_info| {
                if is_static {
                    class_info.static_method_impl_classes.get(method_key)
                } else {
                    class_info.method_impl_classes.get(method_key)
                }
            })
            .cloned()
            .unwrap_or_else(|| class_name.to_string());
        let declared_flags = self
            .classes
            .get(&impl_class_name)
            .map(|class_info| Self::declared_method_param_flags(class_info, method_key, is_static))
            .unwrap_or_default();
        let sig_slot = self.classes.get_mut(&impl_class_name).and_then(|class_info| {
            if is_static {
                class_info.static_methods.get_mut(method_key)
            } else {
                class_info.methods.get_mut(method_key)
            }
        });
        if let Some(sig) = sig_slot {
            if !sig.params.is_empty() {
                sig.params[0].1 = PhpType::Str;
            }
            if sig.params.len() > 1 {
                let declared_array_param = declared_flags.get(1).copied().unwrap_or(false);
                sig.params[1].1 = match &sig.params[1].1 {
                    PhpType::Array(existing)
                        if declared_array_param
                            && matches!(existing.as_ref(), PhpType::Mixed)
                            && !matches!(elem_ty, PhpType::Mixed) =>
                    {
                        args_array_ty
                    }
                    PhpType::Array(existing) => PhpType::Array(Box::new(
                        Self::merge_magic_call_arg_type(*existing.clone(), elem_ty.clone()),
                    )),
                    PhpType::Int => args_array_ty,
                    _ => sig.params[1].1.clone(),
                };
            }
        }
        Ok(())
    }

    /// Merges two types for `__call` argument type inference.
    ///
    /// Returns `right` when `left` is `Never`, `left` when `right` is `Never`,
    /// `left` when equal, and `PhpType::Mixed` otherwise. Used to compute the
    /// element type of the synthetic args array.
    fn merge_magic_call_arg_type(left: PhpType, right: PhpType) -> PhpType {
        if left == right {
            return left;
        }
        if matches!(left, PhpType::Never) {
            return right;
        }
        if matches!(right, PhpType::Never) {
            return left;
        }
        PhpType::Mixed
    }

    /// Relaxes a `__call` signature for validation-only use.
    ///
    /// Sets the first parameter to `PhpType::Str` and the second to
    /// `PhpType::Array(PhpType::Mixed)`, bypassing strict type checking so
    /// arbitrary arguments can be forwarded without false validation errors.
    fn relax_magic_call_validation_sig(sig: &mut crate::types::FunctionSig) {
        if let Some(param) = sig.params.get_mut(0) {
            param.1 = PhpType::Str;
        }
        if let Some(param) = sig.params.get_mut(1) {
            param.1 = PhpType::Array(Box::new(PhpType::Mixed));
        }
    }

    /// Infers the type of a static method call expression (`Foo::method()`, `self::`, `parent::`, `static::`).
    ///
    /// Resolves the receiver to a class name, checks deprecation and visibility,
    /// validates arguments via `normalize_named_call_args` and `check_known_callable_call`,
    /// and updates parameter types from argument types for local type inference.
    /// Handles enum static calls, `parent::`/`self::` forwarding to instance methods,
    /// and falls back to `PhpType::Int`.
    pub(crate) fn infer_static_method_call_type(
        &mut self,
        receiver: &StaticReceiver,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        self.infer_static_method_call_type_with_options(receiver, method, args, expr, env, false)
    }

    /// Infers a static method call for descriptor-backed callback paths that can
    /// preserve by-reference spread arguments through runtime invoker metadata.
    pub(crate) fn infer_static_method_call_type_allowing_by_ref_spread(
        &mut self,
        receiver: &StaticReceiver,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        self.infer_static_method_call_type_with_options(receiver, method, args, expr, env, true)
    }

    /// Shared implementation for static method call inference.
    fn infer_static_method_call_type_with_options(
        &mut self,
        receiver: &StaticReceiver,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
        allow_by_ref_spread: bool,
    ) -> Result<PhpType, CompileError> {
        let parent_call = matches!(receiver, StaticReceiver::Parent);
        let self_call = matches!(receiver, StaticReceiver::Self_);
        let resolved_class_name = match receiver {
            StaticReceiver::Named(class_name) => class_name.as_str().to_string(),
            StaticReceiver::Self_ => self.current_class.as_ref().cloned().ok_or_else(|| {
                CompileError::new(expr.span, "Cannot use self:: outside class method scope")
            })?,
            StaticReceiver::Static => self.current_class.as_ref().cloned().ok_or_else(|| {
                CompileError::new(expr.span, "Cannot use static:: outside class method scope")
            })?,
            StaticReceiver::Parent => {
                let current_class = self.current_class.as_ref().ok_or_else(|| {
                    CompileError::new(expr.span, "Cannot use parent:: outside class method scope")
                })?;
                let current_info = self.classes.get(current_class).ok_or_else(|| {
                    CompileError::new(expr.span, &format!("Undefined class: {}", current_class))
                })?;
                current_info.parent.as_ref().cloned().ok_or_else(|| {
                    CompileError::new(
                        expr.span,
                        &format!("Class {} has no parent class", current_class),
                    )
                })?
            }
        };
        let class_name = resolved_class_name.as_str();
        let named_lexical_instance_call = matches!(receiver, StaticReceiver::Named(_))
            && !self.current_method_is_static
            && self.current_class.as_ref().is_some_and(|current| {
                current == class_name || self.is_subclass_of(current, class_name)
            });
        // `Closure::bind($closure, $newThis [, $scope])` is the static form of
        // `$closure->bindTo(...)`: it returns a new closure with `$this` rebound.
        if class_name.trim_start_matches('\\') == "Closure" && php_symbol_key(method) == "bind" {
            let rebound_this = closure_bind_property_receiver_type(self, args, env);
            let rebound_scope = closure_bind_visibility_scope(self, args, env);
            for (index, arg) in args.iter().enumerate() {
                if index == 0 {
                    let saved_class = self.current_class.clone();
                    if rebound_scope.is_some() {
                        self.current_class = rebound_scope.clone();
                    }
                    let mut closure_env = env.clone();
                    if let Some(rebound_this) = &rebound_this {
                        closure_env.insert(
                            Self::narrowed_this_env_key().to_string(),
                            rebound_this.clone(),
                        );
                    }
                    let result = self.infer_type(arg, &closure_env);
                    self.current_class = saved_class;
                    result?;
                } else {
                    self.infer_type(arg, env)?;
                }
            }
            return Ok(PhpType::Callable);
        }
        if let Some(enum_info) = self.enums.get(class_name).cloned() {
            return self
                .check_enum_static_call(&enum_info, class_name, method, args, env, expr.span);
        }
        let method_key = php_symbol_key(method);
        let late_static_receiver_type = if parent_call {
            self.current_class
                .clone()
                .unwrap_or_else(|| class_name.to_string())
        } else {
            class_name.to_string()
        };
        let late_static_static_return_type = self
            .static_method_late_static_return(class_name, &method_key)
            .map(|return_type| {
                self.resolve_late_static_return_type_hint(
                    &return_type,
                    &late_static_receiver_type,
                    expr.span,
                )
            })
            .transpose()?;
        let late_static_instance_return_type =
            if parent_call || self_call || named_lexical_instance_call {
            self.instance_method_late_static_return(class_name, &method_key)
                .map(|return_type| {
                    self.resolve_late_static_return_type_hint(
                        &return_type,
                        &late_static_receiver_type,
                        expr.span,
                    )
                })
                .transpose()?
            } else {
                None
            };
        let normalized_args: Vec<Expr>;
        let mut magic_return_ty = None;
        let mut magic_original_args = None;
        if let Some(class_info) = self.classes.get(class_name) {
            if let Some(sig) = class_info.static_methods.get(&method_key) {
                if let Some(reason) = sig.deprecation.clone() {
                    let message = if reason.is_empty() {
                        format!(
                            "Call to deprecated static method: {}::{}()",
                            class_name, method
                        )
                    } else {
                        format!(
                            "Call to deprecated static method: {}::{}() — {}",
                            class_name, method, reason
                        )
                    };
                    self.warnings
                        .push(crate::errors::CompileWarning::new(expr.span, &message));
                }
                if let Some(visibility) = class_info.static_method_visibilities.get(&method_key) {
                    let declaring_class = class_info
                        .static_method_declaring_classes
                        .get(&method_key)
                        .map(String::as_str)
                        .unwrap_or(class_name);
                    if !self.can_access_member(declaring_class, visibility)
                        && !self.can_access_pdo_exception_internal_factory(class_name, method)
                    {
                        return Err(CompileError::new(
                            expr.span,
                            &format!(
                                "Cannot access {} method: {}::{}",
                                Self::visibility_label(visibility),
                                class_name,
                                method
                            ),
                        ));
                    }
                }
                let declared_flags =
                    Self::declared_method_param_flags(class_info, &method_key, true);
                let mut effective_sig =
                    Self::callable_sig_for_declared_params(sig, &declared_flags);
                if method_key == "__callstatic" {
                    Self::relax_magic_call_validation_sig(&mut effective_sig);
                }
                normalized_args = self.normalize_named_call_args(
                    &effective_sig,
                    args,
                    expr.span,
                    &format!("Static method {}::{}", class_name, method),
                    env,
                )?;
                if allow_by_ref_spread {
                    self.check_user_declared_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Static method {}::{}", class_name, method),
                        class_name,
                    )?;
                } else {
                    self.check_user_declared_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Static method {}::{}", class_name, method),
                        class_name,
                    )?;
                }
            } else if parent_call || self_call || named_lexical_instance_call {
                if self.current_method_is_static {
                    return Err(CompileError::new(
                        expr.span,
                        if parent_call {
                            "Cannot call parent instance method from a static method"
                        } else {
                            "Cannot call self instance method from a static method"
                        },
                    ));
                }
                let sig = class_info.methods.get(&method_key).ok_or_else(|| {
                    CompileError::new(
                        expr.span,
                        &format!("Undefined method: {}::{}", class_name, method),
                    )
                })?;
                if let Some(visibility) = class_info.method_visibilities.get(&method_key) {
                    let declaring_class = class_info
                        .method_declaring_classes
                        .get(&method_key)
                        .map(String::as_str)
                        .unwrap_or(class_name);
                    if !self.can_access_member(declaring_class, visibility) {
                        return Err(CompileError::new(
                            expr.span,
                            &format!(
                                "Cannot access {} method: {}::{}",
                                Self::visibility_label(visibility),
                                class_name,
                                method
                            ),
                        ));
                    }
                }
                let declared_flags =
                    Self::declared_method_param_flags(class_info, &method_key, false);
                let effective_sig = Self::callable_sig_for_declared_params(sig, &declared_flags);
                normalized_args = self.normalize_named_call_args(
                    &effective_sig,
                    args,
                    expr.span,
                    &format!(
                        "{} method {}::{}",
                        if parent_call {
                            "Parent"
                        } else if self_call {
                            "Self"
                        } else {
                            "Ancestor"
                        },
                        class_name,
                        method
                    ),
                    env,
                )?;
                if allow_by_ref_spread {
                    self.check_user_declared_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!(
                            "{} method {}::{}",
                            if parent_call {
                                "Parent"
                            } else if self_call {
                                "Self"
                            } else {
                                "Ancestor"
                            },
                            class_name,
                            method
                        ),
                        class_name,
                    )?;
                } else {
                    self.check_user_declared_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!(
                            "{} method {}::{}",
                            if parent_call {
                                "Parent"
                            } else if self_call {
                                "Self"
                            } else {
                                "Ancestor"
                            },
                            class_name,
                            method
                        ),
                        class_name,
                    )?;
                }
            } else if class_info.methods.contains_key(&method_key) {
                return Err(CompileError::new(
                    expr.span,
                    &format!(
                        "Cannot call instance method statically: {}::{}",
                        class_name, method
                    ),
                ));
            } else if let Some(sig) = class_info.static_methods.get("__callstatic") {
                let magic_args = Self::magic_call_args(method, args, expr.span);
                let declared_flags =
                    Self::declared_method_param_flags(class_info, "__callstatic", true);
                let mut effective_sig =
                    Self::callable_sig_for_declared_params(sig, &declared_flags);
                Self::relax_magic_call_validation_sig(&mut effective_sig);
                normalized_args = self.normalize_named_call_args(
                    &effective_sig,
                    &magic_args,
                    expr.span,
                    &format!("Static method {}::__callStatic", class_name),
                    env,
                )?;
                if allow_by_ref_spread {
                    self.check_user_declared_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Static method {}::__callStatic", class_name),
                        class_name,
                    )?;
                } else {
                    self.check_user_declared_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Static method {}::__callStatic", class_name),
                        class_name,
                    )?;
                }
                magic_return_ty = Some(effective_sig.return_type.clone());
                magic_original_args = Some(args.to_vec());
            } else {
                return Err(CompileError::new(
                    expr.span,
                    &format!("Undefined method: {}::{}", class_name, method),
                ));
            }
        } else if (self.eval_barrier_active || self.allows_absent_runtime_class())
            && matches!(receiver, StaticReceiver::Named(_))
        {
            for arg in args {
                self.infer_type(arg, env)?;
            }
            return Ok(PhpType::Mixed);
        } else {
            return Err(CompileError::new(
                expr.span,
                &format!("Undefined class: {}", class_name),
            ));
        }
        if let Some(return_ty) = magic_return_ty {
            if let Some(args) = magic_original_args {
                self.specialize_magic_static_call_signature(class_name, &args, env)?;
            }
            return Ok(return_ty);
        }
        let mut arg_types = Vec::new();
        for arg in &normalized_args {
            arg_types.push(self.infer_type(arg, env)?);
        }

        let direct_impl_class_name = if parent_call || self_call {
            self.classes
                .get(class_name)
                .and_then(|class_info| class_info.method_impl_classes.get(&method_key))
                .cloned()
                .unwrap_or_else(|| class_name.to_string())
        } else {
            String::new()
        };
        let static_declared_flags = self
            .classes
            .get(class_name)
            .map(|class_info| Self::declared_method_param_flags(class_info, &method_key, true))
            .unwrap_or_default();
        if let Some(class_info) = self.classes.get_mut(class_name) {
            if let Some(sig) = class_info.static_methods.get_mut(&method_key) {
                let regular_param_count = if sig.variadic.is_some() {
                    sig.params.len().saturating_sub(1)
                } else {
                    sig.params.len()
                };
                for (i, arg_ty) in arg_types.iter().enumerate() {
                    if i < regular_param_count
                        && !static_declared_flags.get(i).copied().unwrap_or(false)
                        && !matches!(*arg_ty, PhpType::Void | PhpType::Never | PhpType::Callable)
                    {
                        let key = (format!("static:{}::{}", class_name, method), i);
                        let seen = self.param_specialization_seen.contains(&key);
                        if sig.params[i].1 == PhpType::Int && !seen {
                            self.param_specialization_seen.insert(key);
                            sig.params[i].1 = arg_ty.clone();
                        } else {
                            sig.params[i].1 = Self::union_param_type(&sig.params[i].1, arg_ty);
                        }
                    }
                }
                let variadic_is_declared = static_declared_flags
                    .get(regular_param_count)
                    .copied()
                    .unwrap_or(false);
                if !variadic_is_declared
                    && method_variadic_tail_needs_iterable(
                        &normalized_args,
                        sig,
                        regular_param_count,
                        env,
                    )
                    && !method_variadic_param_is_by_ref(sig)
                {
                    if let Some((_, variadic_ty)) = sig.params.last_mut() {
                        *variadic_ty = PhpType::Iterable;
                    }
                } else if !variadic_is_declared
                    && sig.variadic.is_some()
                    && arg_types.len() > regular_param_count
                    && !method_variadic_param_is_by_ref(sig)
                    // Same rule as the instance-method path: a declared variadic element type is
                    // a contract to validate against, not a slot to narrow from the call site.
                    && !static_declared_flags
                        .get(regular_param_count)
                        .copied()
                        .unwrap_or(false)
                {
                    let mut elem_ty = arg_types[regular_param_count].clone();
                    for arg_ty in arg_types.iter().skip(regular_param_count + 1) {
                        elem_ty = wider_type_syntactic(&elem_ty, arg_ty);
                    }
                    if let Some((_, PhpType::Array(existing_elem_ty))) = sig.params.last_mut() {
                        **existing_elem_ty =
                            wider_type_syntactic(existing_elem_ty.as_ref(), &elem_ty);
                    }
                }
                return Ok(late_static_static_return_type
                    .clone()
                    .unwrap_or_else(|| sig.return_type.clone()));
            }
        }
        if parent_call || self_call {
            let instance_declared_flags = self
                .classes
                .get(&direct_impl_class_name)
                .map(|class_info| Self::declared_method_param_flags(class_info, &method_key, false))
                .unwrap_or_default();
            if let Some(sig) = self
                .classes
                .get_mut(&direct_impl_class_name)
                .and_then(|class_info| class_info.methods.get_mut(&method_key))
            {
                let regular_param_count = if sig.variadic.is_some() {
                    sig.params.len().saturating_sub(1)
                } else {
                    sig.params.len()
                };
                for (i, arg_ty) in arg_types.iter().enumerate() {
                    if i < regular_param_count
                        && !instance_declared_flags.get(i).copied().unwrap_or(false)
                        && !matches!(*arg_ty, PhpType::Void | PhpType::Never | PhpType::Callable)
                    {
                        let key = (format!("{}::{}", direct_impl_class_name, method), i);
                        let seen = self.param_specialization_seen.contains(&key);
                        if sig.params[i].1 == PhpType::Int && !seen {
                            self.param_specialization_seen.insert(key);
                            sig.params[i].1 = arg_ty.clone();
                        } else {
                            sig.params[i].1 = Self::union_param_type(&sig.params[i].1, arg_ty);
                        }
                    }
                }
                let variadic_is_declared = instance_declared_flags
                    .get(regular_param_count)
                    .copied()
                    .unwrap_or(false);
                if !variadic_is_declared
                    && sig.variadic.is_some()
                    && arg_types.len() > regular_param_count
                    && !method_variadic_param_is_by_ref(sig)
                {
                    let mut elem_ty = arg_types[regular_param_count].clone();
                    for arg_ty in arg_types.iter().skip(regular_param_count + 1) {
                        elem_ty = wider_type_syntactic(&elem_ty, arg_ty);
                    }
                    if let Some((_, PhpType::Array(existing_elem_ty))) = sig.params.last_mut() {
                        **existing_elem_ty =
                            wider_type_syntactic(existing_elem_ty.as_ref(), &elem_ty);
                    }
                }
                return Ok(late_static_instance_return_type
                    .clone()
                    .unwrap_or_else(|| sig.return_type.clone()));
            }
        }
        Ok(PhpType::Int)
    }

    /// Returns preserved late-static return syntax for a static method.
    fn static_method_late_static_return(
        &self,
        receiver_type: &str,
        method_key: &str,
    ) -> Option<TypeExpr> {
        self.classes
            .get(receiver_type)
            .and_then(|class_info| {
                class_info
                    .late_static_static_method_returns
                    .get(method_key)
            })
            .cloned()
    }

    /// Allows tightly-scoped private helper calls between compiler-generated PDO prelude classes.
    fn can_access_pdo_prelude_internal_method(&self, class_name: &str, method_key: &str) -> bool {
        let lazy_row_refresh = class_name == "PDORow"
            && method_key == "__elephcrefresh"
            && self.current_class.as_deref() == Some("PDOStatement")
            && self.current_method.as_deref() == Some("fetch");
        let pgsql_notice_drain = matches!(class_name, "PDO" | "Pdo\\Pgsql")
            && method_key == "__elephcdrainpgsqlnotices"
            && matches!(
                (self.current_class.as_deref(), self.current_method.as_deref()),
                (Some("PDOStatement"), Some("execute"))
                    | (Some("Pdo\\Pgsql"), Some("exec" | "query"))
            );
        lazy_row_refresh || pgsql_notice_drain
    }

    /// Allows compiler-generated PDO methods to construct exceptions with private driver state.
    fn can_access_pdo_exception_internal_factory(&self, class_name: &str, method: &str) -> bool {
        class_name == "PDOException"
            && php_symbol_key(method) == "__elephcfromerrorinfo"
            && matches!(self.current_class.as_deref(), Some("PDO" | "PDOStatement"))
    }
}

/// Resolves the visibility scope supplied to `Closure::bind()`.
///
/// PHP accepts a class name or an object as the third argument. When omitted, the closure keeps
/// its lexical class scope. Dynamic values whose class is not statically known remain gradual and
/// do not grant additional private or protected access during closure-body checking.
fn closure_bind_visibility_scope(
    checker: &mut Checker,
    args: &[Expr],
    env: &TypeEnv,
) -> Option<String> {
    let Some(scope) = args.get(2) else {
        return checker.current_class.clone();
    };
    match &scope.kind {
        ExprKind::StringLiteral(name) => Some(name.trim_start_matches('\\').to_string()),
        ExprKind::ClassConstant { receiver } => match receiver {
            StaticReceiver::Named(name) => {
                Some(name.as_str().trim_start_matches('\\').to_string())
            }
            StaticReceiver::Self_ | StaticReceiver::Static => checker.current_class.clone(),
            StaticReceiver::Parent => checker
                .current_class
                .as_ref()
                .and_then(|class| checker.classes.get(class))
                .and_then(|class| class.parent.clone()),
        },
        _ => checker
            .infer_type(scope, env)
            .ok()
            .as_ref()
            .and_then(crate::types::checker::single_object_class_name),
    }
}

/// Resolves the rebound `$this` type for the directly lowered bound-property closure shape.
///
/// The specialized lowering supports a closure whose body is exactly `return $this->property`;
/// all other bodies keep their existing gradual/runtime path.
fn closure_bind_property_receiver_type(
    checker: &mut Checker,
    args: &[Expr],
    env: &TypeEnv,
) -> Option<PhpType> {
    let ExprKind::Closure { body, .. } = &args.first()?.kind else {
        return None;
    };
    let [stmt] = body.as_slice() else {
        return None;
    };
    let crate::parser::ast::StmtKind::Return(Some(value)) = &stmt.kind else {
        return None;
    };
    let ExprKind::PropertyAccess { object, .. } = &value.kind else {
        return None;
    };
    if !matches!(object.kind, ExprKind::This) {
        return None;
    }
    checker
        .infer_type(args.get(1)?, env)
        .ok()
        .as_ref()
        .and_then(crate::types::checker::single_object_class_name)
        .map(PhpType::Object)
}

/// Returns true when a method variadic parameter must keep runtime key information.
fn method_variadic_tail_needs_iterable(
    args: &[Expr],
    sig: &FunctionSig,
    regular_param_count: usize,
    env: &TypeEnv,
) -> bool {
    if sig.variadic.is_none() {
        return false;
    }

    if args.iter().any(|arg| {
        matches!(
            &arg.kind,
            ExprKind::Spread(inner) if spread_source_keeps_runtime_keys(inner, env)
        )
    }) {
        return true;
    }

    args.iter().any(|arg| {
        matches!(
            &arg.kind,
            ExprKind::NamedArg { name, .. }
                if !sig
                    .params
                    .iter()
                    .take(regular_param_count)
                    .any(|(param_name, _)| param_name == name)
        )
    })
}

/// Returns whether a method signature stores its variadic slot by reference.
fn method_variadic_param_is_by_ref(sig: &FunctionSig) -> bool {
    let Some(variadic_name) = sig.variadic.as_ref() else {
        return false;
    };
    sig.params
        .iter()
        .position(|(name, _)| name == variadic_name)
        .and_then(|index| sig.ref_params.get(index))
        .copied()
        .unwrap_or(false)
}

/// Returns true when a spread source can carry string keys into a variadic method tail.
fn spread_source_keeps_runtime_keys(expr: &Expr, env: &TypeEnv) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => matches!(
            env.get(name),
            Some(PhpType::AssocArray { .. } | PhpType::Iterable)
        ),
        ExprKind::ArrayLiteralAssoc(_) => true,
        _ => matches!(
            crate::types::checker::infer_expr_type_syntactic(expr),
            PhpType::AssocArray { .. } | PhpType::Iterable
        ),
    }
}
