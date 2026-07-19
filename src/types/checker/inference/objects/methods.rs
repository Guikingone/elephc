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
use crate::parser::ast::{Expr, ExprKind, StaticReceiver};
use crate::types::{FunctionSig, PhpType, TypeEnv};

use super::super::super::Checker;
use super::super::syntactic::wider_type_syntactic;

impl Checker {
    /// Infers the type of a method call expression (`$obj->method(...)`).
    ///
    /// Dispatches to `infer_method_call_on_class_type` for `Object` types,
    /// `infer_method_call_on_interface_type` for interface types, and
    /// handles nullable union receivers. Returns `PhpType::Int` as a fallback
    /// for unhandled types (e.g. `Mixed` without specific handler).
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
            if self.interfaces.contains_key(class_name) {
                return self.infer_method_call_on_interface_type(
                    class_name, method, args, expr, env,
                );
            }
            return self.infer_method_call_on_class_type(class_name, method, args, expr, env);
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
                return self.infer_method_call_on_class_type(&class_name, method, args, expr, env);
            }
            // Union of two or more distinct object classes (`A|B`, `A|B|false`): PHP dispatches
            // on the runtime class, so the method only needs to exist on at least one member
            // (see `infer_method_call_on_object_union`). Codegen already dispatches on the
            // runtime class id (`lower_mixed_method_call`) and faults cleanly when the actual
            // value's class has no matching candidate.
            let object_classes = self.union_object_classes(&obj_ty);
            if object_classes.len() >= 2 {
                return self.infer_method_call_on_object_union(&object_classes, method, args, expr, env);
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
        // A method call on a gradual `Mixed` receiver has an unknown runtime class, so its return
        // type is genuinely unknown — `Mixed`, not a default scalar. Returning `Mixed` keeps a call
        // such as `$value->format(...)` on an un-narrowed `mixed` value compatible with any declared
        // return type (gradual typing), which matches PHP where the method exists at runtime. This
        // is what makes a `switch (true) { case $v instanceof DateTimeInterface: return $v->format(...) }`
        // body (where the case arm does not narrow `$v`) type-check against a `: string` signature.
        //
        // The argument expressions are intentionally not re-inferred here: the receiver class is
        // unknown so there is no signature to check them against, and the caller already inferred
        // them (the `infer_type_with_assignment_effects` method-call path threads assignment effects
        // through the arguments before this runs). Re-inferring them with plain `infer_type` would
        // drop that flow state and, for an argument such as `match (true) { !$x = f() => ..., $x => ... }`,
        // falsely report the in-condition assignment target as undefined.
        if matches!(obj_ty, PhpType::Mixed) {
            return Ok(PhpType::Mixed);
        }
        Ok(PhpType::Int)
    }

    /// Infers a method's return type given a resolved object/interface class name,
    /// dispatching to the interface- or class-method path. Shared by the plain and
    /// nullsafe method-call receiver logic.
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

    /// Resolves a method call against a multi-class object union (`A|B`, `A|B|false`) using
    /// PHP-faithful lenient dispatch instead of requiring the method on every member: PHP
    /// dispatches `$u->m()` on the runtime class, so a union type-checks as long as at least
    /// one member declares `m`.
    ///
    /// - A member whose class/interface only reaches the call through `__call`/`__callStatic`
    ///   magic forwarding does NOT count as "resolving" here (JURY ADDENDUM #3) — codegen's
    ///   Mixed-receiver dispatch (`mixed_method_candidates` in `codegen_ir/lower_inst.rs`)
    ///   still forwards to `__call` for such a class at runtime independently of this checker
    ///   decision, which is a documented, sound divergence (the checker under-approximates;
    ///   the runtime path is strictly more permissive, never less).
    /// - Exactly one resolving member: the call is validated/typed against that member alone
    ///   (JURY ADDENDUM #1's dominant case), with no cross-member argument requirement.
    /// - Two or more resolving members: the call's arguments are validated against EVERY
    ///   resolving member's signature and must be accepted by ALL of them (codegen materializes
    ///   the call arguments once for whichever branch runs, so a per-branch ABI mismatch would
    ///   silently pass garbage); if any resolving member rejects the arguments, the whole call
    ///   stays loud with that member's diagnostic. When all accept, the result type is the
    ///   union of each member's return type.
    /// - No member resolves: reports "Undefined method" against the full union type, naming
    ///   every object member (JURY ADDENDUM #5), matching today's diagnostic style.
    fn infer_method_call_on_object_union(
        &mut self,
        object_classes: &[String],
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let method_key = php_symbol_key(method);
        let resolving: Vec<String> = object_classes
            .iter()
            .filter(|class_name| self.union_member_declares_method(class_name, &method_key))
            .cloned()
            .collect();
        if resolving.is_empty() {
            let union_ty = PhpType::Union(
                object_classes
                    .iter()
                    .map(|class_name| PhpType::Object(class_name.clone()))
                    .collect(),
            );
            return Err(CompileError::new(
                expr.span,
                &format!("Undefined method: {}::{}", union_ty, method),
            ));
        }
        if resolving.len() == 1 {
            return self.infer_method_return_on_class_or_interface(
                &resolving[0],
                method,
                args,
                expr,
                env,
            );
        }
        let mut return_types = Vec::with_capacity(resolving.len());
        let mut first_err: Option<CompileError> = None;
        for class_name in &resolving {
            match self.infer_method_return_on_class_or_interface(class_name, method, args, expr, env) {
                Ok(return_ty) => return_types.push(return_ty),
                Err(err) => {
                    if first_err.is_none() {
                        first_err = Some(err);
                    }
                }
            }
        }
        if let Some(err) = first_err {
            return Err(err);
        }
        Ok(self.normalize_union_type(return_types))
    }

    /// Returns true when `class_name` (a class or interface) literally declares `method_key` —
    /// i.e. the receiver's runtime class dispatches straight to it rather than only through
    /// `__call` magic forwarding. Used by `infer_method_call_on_object_union` to decide whether
    /// a union member "resolves" a call (JURY ADDENDUM #3 excludes `__call`-only members).
    fn union_member_declares_method(&self, class_name: &str, method_key: &str) -> bool {
        if let Some(interface_info) = self.interfaces.get(class_name) {
            return interface_info.methods.contains_key(method_key);
        }
        self.classes
            .get(class_name)
            .map(|class_info| class_info.methods.contains_key(method_key))
            .unwrap_or(false)
    }

    /// Infers the type of a nullsafe method call expression (`$obj?->method(...)`).
    ///
    /// Mirrors the gradual receiver handling of the plain `->` path: a `Mixed` receiver
    /// (or a union whose only non-object members include `Mixed`) has an unknown runtime
    /// class, so the result is `Mixed`; a `?Object`/`Object|null` receiver yields the
    /// method's return type unioned with `Void`; a gradual object-plus-scalar union
    /// (`Foo|false`) or multi-class union (`A|B`) dispatches on the runtime class id. A
    /// proven-null receiver short-circuits to `Void`, and a proven non-object stays loud.
    pub(crate) fn infer_nullsafe_method_call_type(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let obj_ty = self.infer_type(object, env)?;
        // Gradual `Mixed` receiver: unknown runtime class → unknown return type. The `?->`
        // null branch is subsumed by `Mixed`. Args were already inferred by the
        // assignment-effects caller (same contract as the plain `->` Mixed path).
        if matches!(obj_ty, PhpType::Mixed) {
            return Ok(PhpType::Mixed);
        }
        match self.nullsafe_object_receiver(&obj_ty, expr, "method call") {
            Ok(Some((class_name, nullable))) => {
                let return_ty = self
                    .infer_method_return_on_class_or_interface(&class_name, method, args, expr, env)?;
                if nullable {
                    Ok(self.normalize_union_type(vec![return_ty, PhpType::Void]))
                } else {
                    Ok(return_ty)
                }
            }
            Ok(None) => Ok(PhpType::Void),
            Err(strict_err) => {
                // The strict single-class resolver rejects gradual unions that the plain
                // `->` path still accepts (`Foo|false`, `A|B`, or a union carrying a
                // `Mixed` member). A `?->` receiver may be non-object at runtime, so the
                // result always admits `Void`.
                if let Some(class_name) = self.union_single_object_class(&obj_ty) {
                    let return_ty = self.infer_method_return_on_class_or_interface(
                        &class_name,
                        method,
                        args,
                        expr,
                        env,
                    )?;
                    return Ok(self.normalize_union_type(vec![return_ty, PhpType::Void]));
                }
                let object_classes = self.union_object_classes(&obj_ty);
                if object_classes.len() >= 2 {
                    let return_ty =
                        self.infer_method_call_on_object_union(&object_classes, method, args, expr, env)?;
                    return Ok(self.normalize_union_type(vec![return_ty, PhpType::Void]));
                }
                if matches!(&obj_ty, PhpType::Union(members)
                    if members.iter().any(|member| matches!(member, PhpType::Mixed)))
                {
                    return Ok(PhpType::Mixed);
                }
                // A `Callable` receiver (bare or nullable, e.g. `?Closure`/`?callable`) may
                // be a `Closure` object at runtime, so `?->` on it is gradual — mirror the
                // plain `->` path, which accepts callable receivers. Result is `Mixed`.
                if type_contains_callable(&obj_ty) {
                    return Ok(PhpType::Mixed);
                }
                Err(strict_err)
            }
        }
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
        let sig = self
            .interfaces
            .get(interface_name)
            .and_then(|interface_info| interface_info.methods.get(&method_key))
            .cloned()
            .ok_or_else(|| {
                CompileError::new(
                    expr.span,
                    &format!("Undefined method: {}::{}", interface_name, method),
                )
            })?;
        let normalized_args = self.normalize_named_call_args(
            &sig,
            args,
            expr.span,
            &format!("Method {}::{}", interface_name, method),
            env,
        )?;
        self.check_known_callable_call(
            &sig,
            &normalized_args,
            expr.span,
            env,
            &format!("Method {}::{}", interface_name, method),
        )?;
        Ok(sig.return_type)
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
            class_name,
            method,
            args,
            expr,
            env,
            false,
        )
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
        self.infer_method_call_on_class_type_with_options(
            class_name,
            method,
            args,
            expr,
            env,
            true,
        )
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
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Method {}::{}", class_name, method),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Method {}::{}", class_name, method),
                    )?;
                }
            } else if let Some(sig) = class_info.methods.get("__call") {
                let magic_args = Self::magic_call_args(method, args, expr.span);
                let declared_flags =
                    Self::declared_method_param_flags(class_info, "__call", false);
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
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Method {}::__call", class_name),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Method {}::__call", class_name),
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
        // Resolve the closure's own signature for each Callable-typed argument BEFORE
        // taking a mutable borrow of `self.classes` below (`resolve_expr_callable_sig`
        // needs `&mut self`). Feeds the class-qualified `callable_param_sigs` write so a
        // method's OWN callable-typed parameter gets the same cross-call specialization
        // free functions already get (see `functions::resolution::mod::check_function_call`'s
        // analogous write at the free-function call site).
        let mut normalized_arg_callable_sigs: Vec<Option<FunctionSig>> =
            Vec::with_capacity(normalized_args.len());
        for (i, arg) in normalized_args.iter().enumerate() {
            if arg_types.get(i) == Some(&PhpType::Callable) {
                normalized_arg_callable_sigs.push(self.resolve_expr_callable_sig(arg, env)?);
            } else {
                normalized_arg_callable_sigs.push(None);
            }
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
        let mut resolved_return: Option<PhpType> = None;
        if let Some(class_info) = self.classes.get_mut(&impl_class_name) {
            if let Some(sig) = class_info.methods.get_mut(&method_key) {
                let regular_param_count = if sig.variadic.is_some() {
                    sig.params.len().saturating_sub(1)
                } else {
                    sig.params.len()
                };
                for (i, arg_ty) in arg_types.iter().enumerate() {
                    if i < regular_param_count
                        && declared_flags.get(i).copied().unwrap_or(false)
                        && Self::is_generic_array_hint(&sig.params[i].1)
                        && matches!(arg_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
                    {
                        // Sharpen a declared generic `array` parameter to the call-site array
                        // shape so method `array` params keep their associative shape, matching
                        // how free-function `array` parameters are specialized (issue #406).
                        sig.params[i].1 = Self::specialize_generic_array_hint(&sig.params[i].1, arg_ty);
                    }
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
                    if i < regular_param_count && *arg_ty == PhpType::Callable {
                        if let Some(closure_sig) = normalized_arg_callable_sigs[i].clone() {
                            // Keyed by the DECLARING/flattened-owner class (`impl_class_name`,
                            // the same class whose `methods` table is mutated above) so
                            // unrelated classes' same-named methods/params never share an
                            // entry — matches the key scheme `ir_lower::context::Context::
                            // callable_param_signature` already reads (`owner_name =
                            // "{class}::{method}"`, `owner_name` built from
                            // `format!("{}::{}", class_name, method_name)` in
                            // `ir_lower::function::lower_class_method`).
                            self.callable_param_sigs.insert(
                                (format!("{}::{}", impl_class_name, method_key), sig.params[i].0.clone()),
                                closure_sig,
                            );
                        }
                    }
                }
                if method_variadic_tail_needs_iterable(
                    &normalized_args,
                    sig,
                    regular_param_count,
                    env,
                ) {
                    if let Some((_, variadic_ty)) = sig.params.last_mut() {
                        *variadic_ty = PhpType::Iterable;
                    }
                } else if sig.variadic.is_some() && arg_types.len() > regular_param_count {
                    let mut elem_ty = arg_types[regular_param_count].clone();
                    for arg_ty in arg_types.iter().skip(regular_param_count + 1) {
                        elem_ty = wider_type_syntactic(&elem_ty, arg_ty);
                    }
                    if let Some((_, PhpType::Array(existing_elem_ty))) = sig.params.last_mut() {
                        // Never narrow a declared `mixed ...$args` variadic. `wider_type_syntactic`
                        // lets `Str` swallow `Mixed`, so specializing a `mixed` variadic from the
                        // first call arg would wrongly retype it to that arg's type and then reject
                        // a later differently-typed arg. A `mixed` variadic accepts any element in
                        // PHP, so it stays `Mixed`. Genuinely-typed variadics still specialize.
                        if !matches!(existing_elem_ty.as_ref(), PhpType::Mixed) {
                            **existing_elem_ty =
                                wider_type_syntactic(existing_elem_ty.as_ref(), &elem_ty);
                        }
                    }
                }
                resolved_return = Some(sig.return_type.clone());
            }
        }
        if let Some(ret) = resolved_return {
            // Late-bind a `: static` return to the receiver class (PHP late static binding); a
            // genuine `: DeclaringClass`/`: self` return is not in the side-table and passes through.
            return Ok(self.resolve_static_return(&ret, class_name, method));
        }
        Ok(PhpType::Int)
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
    fn specialize_magic_callstatic_signature(
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
        // `Closure::bind($closure, $newThis [, $scope])` is the static form of
        // `$closure->bindTo(...)`: it returns a new closure with `$this` rebound. A literal
        // `$scope` (`X::class`) additionally rebinds the closure's VISIBILITY scope to `X` when
        // `check_closure_bind_call_args`'s JURY-mandated lexical gate proves it sound (see
        // `crate::types::checker::inference::expr::static_closure`'s module doc comment); an
        // omitted/non-literal/gate-failing `$scope` checks the closure body normally (no
        // rebind), matching the original (unrelaxed) behavior exactly.
        if class_name.trim_start_matches('\\') == "Closure" && php_symbol_key(method) == "bind" {
            if let Some(closure_arg) = args.first() {
                let rest: Vec<&Expr> = args.get(1..2).unwrap_or(&[]).iter().collect();
                let scope_arg = args.get(2);
                super::super::check_closure_bind_call_args(self, closure_arg, &rest, scope_arg, env)?;
            }
            return Ok(PhpType::Callable);
        }
        if let Some(enum_info) = self.enums.get(class_name).cloned() {
            return self
                .check_enum_static_call(&enum_info, class_name, method, args, env, expr.span);
        }
        let normalized_args: Vec<Expr>;
        if let Some(class_info) = self.classes.get(class_name) {
            if let Some(sig) = class_info.static_methods.get(method) {
                if let Some(reason) = sig.deprecation.clone() {
                    let message = if reason.is_empty() {
                        format!("Call to deprecated static method: {}::{}()", class_name, method)
                    } else {
                        format!(
                            "Call to deprecated static method: {}::{}() — {}",
                            class_name, method, reason
                        )
                    };
                    self.warnings
                        .push(crate::errors::CompileWarning::new(expr.span, &message));
                }
                if let Some(visibility) = class_info.static_method_visibilities.get(method) {
                    let declaring_class = class_info
                        .static_method_declaring_classes
                        .get(method)
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
                let declared_flags = Self::declared_method_param_flags(class_info, method, true);
                let effective_sig = Self::callable_sig_for_declared_params(sig, &declared_flags);
                normalized_args = self.normalize_named_call_args(
                    &effective_sig,
                    args,
                    expr.span,
                    &format!("Static method {}::{}", class_name, method),
                    env,
                )?;
                if allow_by_ref_spread {
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Static method {}::{}", class_name, method),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!("Static method {}::{}", class_name, method),
                    )?;
                }
            } else if parent_call || self_call {
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
                let sig = class_info.methods.get(method).ok_or_else(|| {
                    CompileError::new(
                        expr.span,
                        &format!("Undefined method: {}::{}", class_name, method),
                    )
                })?;
                if let Some(visibility) = class_info.method_visibilities.get(method) {
                    let declaring_class = class_info
                        .method_declaring_classes
                        .get(method)
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
                let declared_flags = Self::declared_method_param_flags(class_info, method, false);
                let effective_sig = Self::callable_sig_for_declared_params(sig, &declared_flags);
                normalized_args = self.normalize_named_call_args(
                    &effective_sig,
                    args,
                    expr.span,
                    &format!(
                        "{} method {}::{}",
                        if parent_call { "Parent" } else { "Self" },
                        class_name,
                        method
                    ),
                    env,
                )?;
                if allow_by_ref_spread {
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!(
                            "{} method {}::{}",
                            if parent_call { "Parent" } else { "Self" },
                            class_name,
                            method
                        ),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        &normalized_args,
                        expr.span,
                        env,
                        &format!(
                            "{} method {}::{}",
                            if parent_call { "Parent" } else { "Self" },
                            class_name,
                            method
                        ),
                    )?;
                }
            } else if let Some(callstatic_sig) =
                class_info.static_methods.get("__callstatic").cloned()
            {
                // Forward `Foo::missing(...)` to `Foo::__callStatic("missing", [...])`.
                let magic_args = Self::magic_call_args(method, args, expr.span);
                let mut validation_sig = callstatic_sig.clone();
                Self::relax_magic_call_validation_sig(&mut validation_sig);
                self.check_known_callable_call(
                    &validation_sig,
                    &magic_args,
                    expr.span,
                    env,
                    &format!("Static method {}::__callStatic", class_name),
                )?;
                self.specialize_magic_callstatic_signature(class_name, args, env)?;
                return Ok(callstatic_sig.return_type.clone());
            } else if class_info.methods.contains_key(method) {
                return Err(CompileError::new(
                    expr.span,
                    &format!(
                        "Cannot call instance method statically: {}::{}",
                        class_name, method
                    ),
                ));
            } else {
                return Err(CompileError::new(
                    expr.span,
                    &format!("Undefined method: {}::{}", class_name, method),
                ));
            }
        } else if self.interfaces.contains_key(class_name) {
            // Every interface method (static or instance) is abstract — an interface never
            // has a runtime object to dispatch on, so `I::method()` is unconditionally invalid.
            // PHP defers this to a runtime `Error` ("Cannot call abstract method I::f()",
            // `php -n` verified), but the receiver is a literal class-like name here, so
            // elephc's closed world can detect it at compile time instead of leaving it to
            // fail at runtime — reported for both static and instance interface methods, since
            // PHP's fatal wording does not distinguish between the two. Dynamic
            // `$class::method()` dispatch through an interface-typed class-string is a
            // different code path (`StaticReceiver::Named` never triggers this branch for it)
            // and is intentionally left untouched here.
            return Err(CompileError::new(
                expr.span,
                &format!("Cannot call abstract method {}::{}()", class_name, method),
            ));
        } else {
            // A static call on a class that is unknown everywhere in the closed world is an
            // absent optional dependency (e.g. `Process::fromShellCommandline(...)`): the call
            // yields an opaque `PhpType::Mixed` value with a warning instead of erroring. Argument
            // expressions are still inferred so genuine errors inside them keep surfacing.
            if !self.class_like_exists(class_name) {
                self.warn_absent_class(expr.span, class_name);
                for arg in args {
                    self.infer_type(arg, env)?;
                }
                return Ok(PhpType::Mixed);
            }
            return Err(CompileError::new(
                expr.span,
                &format!("Undefined class: {}", class_name),
            ));
        }
        let mut arg_types = Vec::new();
        for arg in &normalized_args {
            arg_types.push(self.infer_type(arg, env)?);
        }
        // Resolve the closure's own signature for each Callable-typed argument BEFORE
        // taking a mutable borrow of `self.classes` below — mirrors the instance-method
        // call path above.
        let mut normalized_arg_callable_sigs: Vec<Option<FunctionSig>> =
            Vec::with_capacity(normalized_args.len());
        for (i, arg) in normalized_args.iter().enumerate() {
            if arg_types.get(i) == Some(&PhpType::Callable) {
                normalized_arg_callable_sigs.push(self.resolve_expr_callable_sig(arg, env)?);
            } else {
                normalized_arg_callable_sigs.push(None);
            }
        }

        let direct_impl_class_name = if parent_call || self_call {
            self.classes
                .get(class_name)
                .and_then(|class_info| class_info.method_impl_classes.get(method))
                .cloned()
                .unwrap_or_else(|| class_name.to_string())
        } else {
            String::new()
        };
        let static_declared_flags = self
            .classes
            .get(class_name)
            .map(|class_info| Self::declared_method_param_flags(class_info, method, true))
            .unwrap_or_default();
        if let Some(class_info) = self.classes.get_mut(class_name) {
            if let Some(sig) = class_info.static_methods.get_mut(method) {
                let regular_param_count = if sig.variadic.is_some() {
                    sig.params.len().saturating_sub(1)
                } else {
                    sig.params.len()
                };
                for (i, arg_ty) in arg_types.iter().enumerate() {
                    if i < regular_param_count
                        && static_declared_flags.get(i).copied().unwrap_or(false)
                        && Self::is_generic_array_hint(&sig.params[i].1)
                        && matches!(arg_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
                    {
                        // Sharpen a declared generic `array` parameter to the call-site array
                        // shape so static-method `array` params keep their associative shape,
                        // matching free-function specialization (issue #406).
                        sig.params[i].1 = Self::specialize_generic_array_hint(&sig.params[i].1, arg_ty);
                    }
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
                    if i < regular_param_count && *arg_ty == PhpType::Callable {
                        if let Some(closure_sig) = normalized_arg_callable_sigs[i].clone() {
                            // NOT "static:"-prefixed (unlike `param_specialization_seen` above):
                            // this key must match the UNPREFIXED `"{class}::{method}"` scheme the
                            // active EIR lowering already reads for both static and instance
                            // methods (`ir_lower::function::lower_class_method`'s `owner_name`),
                            // and PHP forbids a class from declaring both a static and an
                            // instance method with the same name, so the plain key stays
                            // unambiguous.
                            self.callable_param_sigs.insert(
                                (format!("{}::{}", class_name, method), sig.params[i].0.clone()),
                                closure_sig,
                            );
                        }
                    }
                }
                if method_variadic_tail_needs_iterable(
                    &normalized_args,
                    sig,
                    regular_param_count,
                    env,
                ) {
                    if let Some((_, variadic_ty)) = sig.params.last_mut() {
                        *variadic_ty = PhpType::Iterable;
                    }
                } else if sig.variadic.is_some() && arg_types.len() > regular_param_count {
                    let mut elem_ty = arg_types[regular_param_count].clone();
                    for arg_ty in arg_types.iter().skip(regular_param_count + 1) {
                        elem_ty = wider_type_syntactic(&elem_ty, arg_ty);
                    }
                    if let Some((_, PhpType::Array(existing_elem_ty))) = sig.params.last_mut() {
                        // Never narrow a declared `mixed ...$args` variadic. `wider_type_syntactic`
                        // lets `Str` swallow `Mixed`, so specializing a `mixed` variadic from the
                        // first call arg would wrongly retype it to that arg's type and then reject
                        // a later differently-typed arg. A `mixed` variadic accepts any element in
                        // PHP, so it stays `Mixed`. Genuinely-typed variadics still specialize.
                        if !matches!(existing_elem_ty.as_ref(), PhpType::Mixed) {
                            **existing_elem_ty =
                                wider_type_syntactic(existing_elem_ty.as_ref(), &elem_ty);
                        }
                    }
                }
                return Ok(sig.return_type.clone());
            }
        }
        if parent_call || self_call {
            let instance_declared_flags = self
                .classes
                .get(&direct_impl_class_name)
                .map(|class_info| Self::declared_method_param_flags(class_info, method, false))
                .unwrap_or_default();
            let mut resolved_return: Option<PhpType> = None;
            if let Some(sig) = self
                .classes
                .get_mut(&direct_impl_class_name)
                .and_then(|class_info| class_info.methods.get_mut(method))
            {
                let regular_param_count = if sig.variadic.is_some() {
                    sig.params.len().saturating_sub(1)
                } else {
                    sig.params.len()
                };
                for (i, arg_ty) in arg_types.iter().enumerate() {
                    if i < regular_param_count
                        && instance_declared_flags.get(i).copied().unwrap_or(false)
                        && Self::is_generic_array_hint(&sig.params[i].1)
                        && matches!(arg_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
                    {
                        // Sharpen a declared generic `array` parameter to the call-site array
                        // shape on `parent::`/`self::` instance dispatch, matching free-function
                        // specialization (issue #406).
                        sig.params[i].1 = Self::specialize_generic_array_hint(&sig.params[i].1, arg_ty);
                    }
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
                if sig.variadic.is_some() && arg_types.len() > regular_param_count {
                    let mut elem_ty = arg_types[regular_param_count].clone();
                    for arg_ty in arg_types.iter().skip(regular_param_count + 1) {
                        elem_ty = wider_type_syntactic(&elem_ty, arg_ty);
                    }
                    if let Some((_, PhpType::Array(existing_elem_ty))) = sig.params.last_mut() {
                        // Never narrow a declared `mixed ...$args` variadic. `wider_type_syntactic`
                        // lets `Str` swallow `Mixed`, so specializing a `mixed` variadic from the
                        // first call arg would wrongly retype it to that arg's type and then reject
                        // a later differently-typed arg. A `mixed` variadic accepts any element in
                        // PHP, so it stays `Mixed`. Genuinely-typed variadics still specialize.
                        if !matches!(existing_elem_ty.as_ref(), PhpType::Mixed) {
                            **existing_elem_ty =
                                wider_type_syntactic(existing_elem_ty.as_ref(), &elem_ty);
                        }
                    }
                }
                resolved_return = Some(sig.return_type.clone());
            }
            if let Some(ret) = resolved_return {
                // `parent::`/`self::` are forwarding calls: a `: static` return late-binds to the
                // current class (the runtime `$this` class), not the resolved parent/self class.
                // The declaring class is looked up on the impl class, but the substitution target
                // is `current_class`, so `parent::trimPrefix()` inside `UnicodeString` yields
                // `UnicodeString`, not the parent's collapsed `AbstractUnicodeString`.
                let bind = self
                    .current_class
                    .clone()
                    .unwrap_or_else(|| class_name.to_string());
                return Ok(self.resolve_static_return_bound(
                    &ret,
                    &direct_impl_class_name,
                    &bind,
                    method,
                ));
            }
        }
        Ok(PhpType::Int)
    }

    /// Late-binds a `: static` instance-method return type to the receiver class (PHP late static
    /// binding). Both the declaring-class lookup and the substitution target are the receiver, so
    /// `$mid->append()` — where `append(): static` is declared in `Base` — returns `Mid`.
    fn resolve_static_return(&self, ret: &PhpType, receiver_class: &str, method: &str) -> PhpType {
        self.resolve_static_return_bound(ret, receiver_class, receiver_class, method)
    }

    /// Core late-static-binding substitution. Resolves the declaring class of `method` on
    /// `declaring_lookup_class` (via `method_declaring_classes`, falling back to that class for a
    /// directly declared method) and consults `static_return_methods`. When `(declaring, method)` is
    /// a recorded `: static` return, the declaring-class `Object` — which the flatten pass collapsed
    /// `static` into — is rewritten to `substitute_class`, preserving nullable/union shape.
    /// Otherwise `ret` is returned unchanged, so a genuine `: DeclaringClass` / `: self` return stays
    /// bound to its declaring class. `declaring_lookup_class` and `substitute_class` coincide for
    /// direct instance dispatch but differ for `parent::`/`self::` forwarding calls, where `static`
    /// binds to the current class rather than the resolved parent/self class.
    fn resolve_static_return_bound(
        &self,
        ret: &PhpType,
        declaring_lookup_class: &str,
        substitute_class: &str,
        method: &str,
    ) -> PhpType {
        let method_key = php_symbol_key(method);
        let declaring = self
            .classes
            .get(declaring_lookup_class)
            .and_then(|ci| ci.method_declaring_classes.get(&method_key))
            .map(String::as_str)
            .unwrap_or(declaring_lookup_class);
        if !self
            .static_return_methods
            .contains(&(declaring.to_string(), method_key))
        {
            return ret.clone();
        }
        substitute_object_class(ret, substitute_class)
    }
}

/// Replaces a bare `Object(_)` — or the `Object` members of a union — with `Object(receiver)`, used
/// to late-bind a `static` return type to the receiver class. A `?static` return collapses to a
/// `Union([Object(_), Void])`, so only the `Object` member is rewritten and the `Void` (null) arm is
/// preserved. Non-object leaves pass through unchanged.
fn substitute_object_class(ty: &PhpType, receiver: &str) -> PhpType {
    match ty {
        PhpType::Object(_) => PhpType::Object(receiver.to_string()),
        PhpType::Union(members) => PhpType::Union(
            members
                .iter()
                .map(|m| substitute_object_class(m, receiver))
                .collect(),
        ),
        other => other.clone(),
    }
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

/// Returns whether `ty` is `Callable` or a `Union` that contains a `Callable` member.
///
/// A callable value may be a `Closure` object at runtime, so a nullsafe method call on a
/// callable (bare or nullable, e.g. `?Closure`/`?callable`) is accepted gradually — the
/// same permissiveness the plain `->` path applies to callable receivers.
fn type_contains_callable(ty: &PhpType) -> bool {
    match ty {
        PhpType::Callable => true,
        PhpType::Union(members) => members
            .iter()
            .any(|member| matches!(member, PhpType::Callable)),
        _ => false,
    }
}
