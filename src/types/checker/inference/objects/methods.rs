//! Purpose:
//! Infers object methods expression types.
//! Validates class, method, constructor, property, and magic-access contracts against schema metadata.
//!
//! Called from:
//! - `crate::types::checker::inference::objects`
//!
//! Key details:
//! - Object inference depends on flattened class metadata, visibility, inheritance, and declared property types.
//! - Mixed-receiver calls retain a concrete return type only when every runtime candidate agrees.
//! - Declared variadic element types, including `mixed`, remain authoritative across call-site
//!   specialization; only untyped variadics are widened from observed arguments.
//! - Calls on an `instanceof`-narrowed class absent from the closed world stay gradual `Mixed`,
//!   matching other optional-dependency reference positions instead of inventing `Int`.

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
            // PHP-faithful lenient dispatch — the call type-checks as long as at least one
            // member declares the method; codegen dispatches on the runtime class id
            // (see `infer_method_call_on_object_union`). A non-object runtime value
            // faults like PHP.
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
    /// the shared declared return type of every class that declares `method`
    /// with a matching arity. Multiple distinct candidate return types collapse
    /// to `Mixed`: the receiver may itself come from an absent optional class
    /// hint degraded to `Mixed`, so selecting unrelated same-named methods would
    /// invent a false nominal union and reject otherwise unreachable guarded
    /// code. Codegen boxes each concrete branch into the `Mixed` result slot.
    /// Falls back to the name-only candidate set when arity filtering finds
    /// nothing (e.g. methods with default parameters), and returns `None` when
    /// no class declares the method at all.
    ///
    /// A SINGLE arbitrary class happening to declare `method` does NOT mean the
    /// `mixed` receiver IS that class — a common source of `mixed` here is an
    /// absent optional-dependency class hint (`private readonly \Vendor\Absent
    /// $x`) degraded to `Mixed`. Returning that lone candidate's concrete NOMINAL
    /// object return type would make a later chained call resolve strictly against
    /// the unrelated class and mis-report "Undefined method": e.g. an absent
    /// `Stopwatch` property whose `start()` mis-binds to `Cache\Adapter\
    /// TraceableAdapter::start(): TraceableAdapterEvent`, so `->stop()` then fails.
    /// Such an object-typed single candidate is therefore degraded to gradual
    /// `Mixed`. Scalar single candidates (e.g. `$x->name(): string`) stay precise:
    /// they render correctly and never chain into a strict object method lookup.
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
        match candidates.as_slice() {
            [] => None,
            [only] if Self::type_mentions_nominal_object(only) => Some(PhpType::Mixed),
            [only] => Some(only.clone()),
            _ => Some(PhpType::Mixed),
        }
    }

    /// Returns whether `ty` is (or, for a union, contains) a nominal `Object` class type.
    ///
    /// Used to decide whether a lone `mixed`-receiver method candidate is safe to surface as a
    /// concrete static type: a nominal object result would make chained calls resolve strictly
    /// against a possibly-unrelated class, so it is degraded to gradual `Mixed` instead.
    fn type_mentions_nominal_object(ty: &PhpType) -> bool {
        match ty {
            PhpType::Object(_) => true,
            PhpType::Union(members) => members.iter().any(Self::type_mentions_nominal_object),
            _ => false,
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

    /// Infers the type of a nullsafe method call expression (`$obj?->method(...)`).
    ///
    /// Returns `PhpType::Void` for invalid receivers. For valid nullable object
    /// unions, returns a union of the method's return type with `void`.
    /// Infers a method call's return type against one class-or-interface receiver name,
    /// routing interfaces through the interface-call path and classes through the class-call
    /// path. Shared by the multi-member union dispatch below.
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
    ///   magic forwarding does NOT count as "resolving" here — codegen's Mixed-receiver
    ///   dispatch still forwards to `__call` for such a class at runtime independently of this
    ///   checker decision, which is a documented, sound divergence (the checker
    ///   under-approximates; the runtime path is strictly more permissive, never less).
    /// - Exactly one resolving member: the call is validated/typed against that member alone,
    ///   with no cross-member argument requirement.
    /// - Two or more resolving members: the call's arguments are validated against EVERY
    ///   resolving member's signature and must be accepted by ALL of them (codegen materializes
    ///   the call arguments once for whichever branch runs, so a per-branch ABI mismatch would
    ///   silently pass garbage); if any resolving member rejects the arguments, the whole call
    ///   stays loud with that member's diagnostic. When all accept, the result type is the
    ///   union of each member's return type.
    /// - No member resolves: reports "Undefined method" against the full union type, naming
    ///   every object member, matching today's diagnostic style.
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
    /// a union member "resolves" a call (`__call`-only members are excluded).
    fn union_member_declares_method(&self, class_name: &str, method_key: &str) -> bool {
        if let Some(interface_info) = self.interfaces.get(class_name) {
            return interface_info.methods.contains_key(method_key);
        }
        self.classes
            .get(class_name)
            .map(|class_info| class_info.methods.contains_key(method_key))
            .unwrap_or(false)
    }

    /// Accepts a method call on a SINGLE interface- or base-class-typed receiver whose static type
    /// does NOT declare `method` itself, matching PHP's runtime-dispatch semantics: PHP performs no
    /// compile-time method-existence check on such a receiver — it dispatches on the runtime class,
    /// faulting cleanly (a PHP `Error`) only if the actual object's class lacks the method.
    ///
    /// The call type-checks when at least one CONCRETE class that IS-A `receiver_type` (the class
    /// itself, a subclass, or — for an interface receiver — an implementor) declares `method`: that
    /// is exactly the set of runtime classes the by-class-id dynamic dispatch can land on (the same
    /// path union receivers and narrowed-interface receivers use in codegen, see
    /// `lower_narrowed_interface_method_call`). At runtime the receiver's real class id selects its
    /// own implementation; a class id with no declaration falls through to the clean member-call
    /// fatal. With no such concrete class the call stays LOUD with the original "Undefined method"
    /// diagnostic — a genuinely undefined method (no class could satisfy it), or one living only on
    /// a sub-interface with no concrete implementor for the dispatch to reach.
    ///
    /// Argument expressions are inferred so nested errors still surface, but they are deliberately
    /// NOT validated against any one candidate signature: the concrete runtime class is unknown
    /// here and codegen materializes the arguments per matched candidate branch, so PHP's
    /// no-compile-time-argument-check is the faithful behavior. The result type is the candidates'
    /// single shared declared return type, or `Mixed` when they disagree (the receiver may resolve
    /// to any of them at runtime).
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

    /// Returns the distinct declared return types of `method_key` across every CONCRETE class that
    /// IS-A `receiver_type` and declares it (see `infer_lenient_subtype_method_call`). Interfaces
    /// are never counted — they have no runtime instances and the dispatch is on concrete class ids
    /// only — so a method declared solely on a sub-interface with no implementor yields an empty
    /// result, keeping the call loud. An empty result means no concrete runtime class in the closed
    /// world could dispatch the method.
    fn subtype_dispatch_return_types(&self, receiver_type: &str, method_key: &str) -> Vec<PhpType> {
        let mut return_types: Vec<PhpType> = Vec::new();
        for (class_name, class_info) in &self.classes {
            let Some(sig) = class_info.methods.get(method_key) else {
                continue;
            };
            let is_a = class_name.as_str() == receiver_type
                || self.is_subclass_of(class_name, receiver_type)
                || self.class_implements_interface(class_name, receiver_type);
            if !is_a {
                continue;
            }
            if !return_types.contains(&sig.return_type) {
                return_types.push(sig.return_type.clone());
            }
        }
        return_types
    }

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
        let Some(sig) = self
            .interfaces
            .get(interface_name)
            .and_then(|interface_info| interface_info.methods.get(&method_key))
            .cloned()
        else {
            // The interface itself does not declare the method. PHP performs no compile-time
            // method-existence check on an interface-typed receiver — it dispatches on the runtime
            // class — so accept the call whenever a concrete implementor declares the method (the
            // by-runtime-class-id dispatch path lowers it), and keep it loud otherwise.
            return self.infer_lenient_subtype_method_call(
                interface_name,
                method,
                &method_key,
                args,
                expr,
                env,
            );
        };
        self.check_known_callable_call(
            &sig,
            args,
            expr.span,
            env,
            &format!("Method {}::{}", interface_name, method),
        )?;
        let late_static_return = self.instance_method_late_static_return(interface_name, &method_key);
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
    /// argument types (for local type inference). PHP also permits a declared
    /// static method to be invoked through an object; that fallback is resolved
    /// before `__call` magic dispatch.
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
        // `instanceof MissingExtensionClass` narrows a gradual receiver to a nominal Object even
        // though that class is absent from the closed world. Its guarded method calls must follow
        // the same optional-dependency contract as absent type hints, constructors, and static
        // calls: infer argument effects, warn, and keep the opaque result Mixed. Falling through
        // to the legacy Int sentinel makes valid guarded iteration (`$redis->_hosts()`) fail.
        if !self.class_like_exists(class_name) {
            self.warn_absent_class(expr.span, class_name);
            for arg in args {
                self.infer_type(arg, env)?;
            }
            return Ok(PhpType::Mixed);
        }
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
                    if !self.can_access_member(declaring_class, visibility) {
                        // PHP raises this as a catchable `Error` at runtime instead of a
                        // compile-time rejection. Record the throw site so EIR lowering
                        // emits the throw sequence, and continue with the declared return
                        // type so later passes stay type-consistent.
                        self.throw_access_sites.insert(
                            expr.span,
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
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        args,
                        expr.span,
                        env,
                        &format!("Method {}::{}", class_name, method),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        args,
                        expr.span,
                        env,
                        &format!("Method {}::{}", class_name, method),
                    )?;
                }
            } else if class_info.static_methods.contains_key(&method_key) {
                return self.infer_static_method_call_type_with_options(
                    &StaticReceiver::Named(class_name.to_string().into()),
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
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        &magic_args,
                        expr.span,
                        env,
                        &format!("Method {}::__call", class_name),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        &magic_args,
                        expr.span,
                        env,
                        &format!("Method {}::__call", class_name),
                    )?;
                }
                magic_return_ty = Some(effective_sig.return_type.clone());
                magic_original_args = Some(args.to_vec());
            } else {
                // The class exists but declares neither this instance method, a same-named static
                // method, nor `__call`. PHP still performs no compile-time method-existence check
                // on a base-class-typed receiver — it dispatches on the runtime class — so accept
                // the call whenever a concrete subclass declares the method (the by-runtime-class-id
                // dispatch path lowers it), and keep it loud otherwise.
                return self.infer_lenient_subtype_method_call(
                    class_name, method, &method_key, args, expr, env,
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
                        && declared_flags.get(i).copied().unwrap_or(false)
                        && !Self::method_array_param_keeps_generic_shape(
                            &impl_class_name,
                            &method_key,
                        )
                        && Self::is_generic_array_hint(&sig.params[i].1)
                        && matches!(arg_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
                    {
                        // Sharpen a declared generic `array` parameter to the call-site array
                        // shape so method `array` params keep their associative shape, matching
                        // how free-function `array` parameters are specialized (issue #406).
                        sig.params[i].1 =
                            Self::specialize_generic_array_param_hint(&sig.params[i].1, arg_ty);
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
                }
                if method_variadic_tail_needs_iterable(
                    &normalized_args,
                    sig,
                    regular_param_count,
                    env,
                ) && !method_variadic_param_is_by_ref(sig)
                    && !method_variadic_param_is_declared(
                        &declared_flags,
                        regular_param_count,
                    )
                {
                    if let Some((_, variadic_ty)) = sig.params.last_mut() {
                        *variadic_ty = PhpType::Iterable;
                    }
                } else if sig.variadic.is_some()
                    && arg_types.len() > regular_param_count
                    && !method_variadic_param_is_by_ref(sig)
                    && !method_variadic_param_is_declared(
                        &declared_flags,
                        regular_param_count,
                    )
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
        Ok(PhpType::Int)
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

    /// Returns true for builtin method array params whose accepted shape must remain broad.
    fn method_array_param_keeps_generic_shape(class_name: &str, method_key: &str) -> bool {
        matches!(class_name, "ReflectionFunction" | "ReflectionMethod")
            && method_key == php_symbol_key("invokeArgs")
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
        // PHP also accepts `Ancestor::instanceMethod()` from a compatible non-static descendant
        // scope. It is a lexical call to that ancestor implementation with the current `$this`,
        // not a genuinely static invocation. Unrelated named classes and static caller scopes
        // remain invalid.
        let named_lexical_instance_call =
            matches!(receiver, StaticReceiver::Named(_))
                && self.current_class.as_deref().is_some_and(|current_class| {
                    current_class == class_name
                        || self.is_subclass_of(current_class, class_name)
                });
        let lexical_instance_call =
            parent_call || self_call || named_lexical_instance_call;
        let lexical_call_label = if parent_call {
            "Parent"
        } else if self_call {
            "Self"
        } else {
            "Ancestor"
        };
        // `Closure::bind($closure, $newThis [, $scope])` is the static form of
        // `$closure->bindTo(...)`: it returns a new closure with `$this` rebound.
        // `$scope` is accepted and ignored (closed-world visibility).
        if class_name.trim_start_matches('\\') == "Closure" && php_symbol_key(method) == "bind" {
            // `Closure::bind($closure, $newThis, $scope)`: when the closure is a literal, `$scope`
            // is a literal class, and the body is free of lexically-resolved scope references, the
            // closure body's protected/private access on parameters typed as the rebound scope is
            // authorized against that scope (see `bound_scope_context`). The context is active only
            // while inferring the closure literal argument.
            let scope_ctx = self.closure_bind_scope_context(args, env);
            for (index, arg) in args.iter().enumerate() {
                if index == 0 && scope_ctx.is_some() {
                    let saved = self.enter_bound_scope_context(scope_ctx.clone());
                    let result = self.infer_type(arg, env);
                    self.exit_bound_scope_context(saved);
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
        let late_static_receiver_type = if lexical_instance_call {
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
        let late_static_instance_return_type = if lexical_instance_call {
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
                    Self::declared_method_param_flags(class_info, &method_key, true);
                let mut effective_sig =
                    Self::callable_sig_for_declared_params(sig, &declared_flags);
                Self::relax_datetime_create_from_format_validation_sig(
                    class_name,
                    &method_key,
                    &mut effective_sig,
                );
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
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        args,
                        expr.span,
                        env,
                        &format!("Static method {}::{}", class_name, method),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        args,
                        expr.span,
                        env,
                        &format!("Static method {}::{}", class_name, method),
                    )?;
                }
            } else if lexical_instance_call {
                if self.current_method_is_static {
                    return Err(CompileError::new(
                        expr.span,
                        if parent_call {
                            "Cannot call parent instance method from a static method"
                        } else if self_call {
                            "Cannot call self instance method from a static method"
                        } else {
                            "Cannot call named ancestor instance method from a static method"
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
                        lexical_call_label,
                        class_name,
                        method
                    ),
                    env,
                )?;
                if allow_by_ref_spread {
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        args,
                        expr.span,
                        env,
                        &format!(
                            "{} method {}::{}",
                            lexical_call_label,
                            class_name,
                            method
                        ),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        args,
                        expr.span,
                        env,
                        &format!(
                            "{} method {}::{}",
                            lexical_call_label,
                            class_name,
                            method
                        ),
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
                    self.check_known_callable_call_allowing_by_ref_spread(
                        &effective_sig,
                        &magic_args,
                        expr.span,
                        env,
                        &format!("Static method {}::__callStatic", class_name),
                    )?;
                } else {
                    self.check_known_callable_call(
                        &effective_sig,
                        &magic_args,
                        expr.span,
                        env,
                        &format!("Static method {}::__callStatic", class_name),
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
        } else if self.eval_barrier_active && matches!(receiver, StaticReceiver::Named(_)) {
            for arg in args {
                self.infer_type(arg, env)?;
            }
            return Ok(PhpType::Mixed);
        } else {
            // Every interface method (static or instance) is abstract — an interface never
            // has a runtime object to dispatch on, so `I::method()` is unconditionally invalid.
            // PHP defers this to a runtime `Error` ("Cannot call abstract method I::f()",
            // `php -n` verified), but the receiver is a literal class-like name here, so
            // elephc's closed world can detect it at compile time instead of leaving it to
            // fail at runtime — reported for both static and instance interface methods, since
            // PHP's fatal wording does not distinguish between the two. Dynamic
            // `$class::method()` dispatch through an interface-typed class-string is a
            // different code path and is intentionally left untouched here.
            if self.interfaces.contains_key(class_name) {
                return Err(CompileError::new(
                    expr.span,
                    &format!("Cannot call abstract method {}::{}()", class_name, method),
                ));
            }
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

        let direct_impl_class_name = if lexical_instance_call {
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
                        && static_declared_flags.get(i).copied().unwrap_or(false)
                        && Self::is_generic_array_hint(&sig.params[i].1)
                        && matches!(arg_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
                    {
                        // Sharpen a declared generic `array` parameter to the call-site array
                        // shape so static-method `array` params keep their associative shape,
                        // matching free-function specialization (issue #406).
                        sig.params[i].1 =
                            Self::specialize_generic_array_param_hint(&sig.params[i].1, arg_ty);
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
                }
                if method_variadic_tail_needs_iterable(
                    &normalized_args,
                    sig,
                    regular_param_count,
                    env,
                ) && !method_variadic_param_is_by_ref(sig)
                    && !method_variadic_param_is_declared(
                        &static_declared_flags,
                        regular_param_count,
                    )
                {
                    if let Some((_, variadic_ty)) = sig.params.last_mut() {
                        *variadic_ty = PhpType::Iterable;
                    }
                } else if sig.variadic.is_some()
                    && arg_types.len() > regular_param_count
                    && !method_variadic_param_is_by_ref(sig)
                    && !method_variadic_param_is_declared(
                        &static_declared_flags,
                        regular_param_count,
                    )
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
        if lexical_instance_call {
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
                        && instance_declared_flags.get(i).copied().unwrap_or(false)
                        && Self::is_generic_array_hint(&sig.params[i].1)
                        && matches!(arg_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
                    {
                        // Sharpen a declared generic `array` parameter to the call-site array
                        // shape on `parent::`/`self::` instance dispatch, matching free-function
                        // specialization (issue #406).
                        sig.params[i].1 =
                            Self::specialize_generic_array_param_hint(&sig.params[i].1, arg_ty);
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
                if sig.variadic.is_some()
                    && arg_types.len() > regular_param_count
                    && !method_variadic_param_is_by_ref(sig)
                    && !method_variadic_param_is_declared(
                        &instance_declared_flags,
                        regular_param_count,
                    )
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

    /// Lets the DateTime format parser accept PHP's weak integer-to-string subject coercion.
    ///
    /// The stored class signature remains `string`, so the synthetic method body and ABI keep
    /// string storage. This validation-only copy admits the integer timestamps Symfony passes
    /// for format `U`; EIR inserts the corresponding conversion at the call boundary.
    fn relax_datetime_create_from_format_validation_sig(
        class_name: &str,
        method_key: &str,
        sig: &mut FunctionSig,
    ) {
        if method_key != "createfromformat"
            || !matches!(
                class_name.trim_start_matches('\\'),
                "DateTime" | "DateTimeImmutable"
            )
        {
            return;
        }
        if let Some((_, datetime_ty)) = sig.params.get_mut(1) {
            *datetime_ty = PhpType::Union(vec![PhpType::Str, PhpType::Int]);
        }
    }

    /// Returns whether a static call is `Closure::bind(...)` (the static form that can carry a
    /// scope-rebind argument), by receiver name and method, independent of argument count.
    pub(crate) fn is_closure_bind_static_call(
        &self,
        receiver: &StaticReceiver,
        method: &str,
    ) -> bool {
        php_symbol_key(method) == "bind"
            && matches!(
                receiver,
                StaticReceiver::Named(name)
                    if name.as_str().trim_start_matches('\\') == "Closure"
            )
    }

    /// Builds the `BoundScopeContext` for a `Closure::bind($closure, $newThis [, $scope])` call,
    /// or `None` when the rebind does not qualify for a relaxed receiver or visibility.
    ///
    /// Two shapes qualify, both requiring a closure literal as the first argument.
    ///
    /// The `$this`-receiver shape (`fn () => $this->prop`, the single form
    /// `crate::ir_lower::closure_bind_property_return_type` lowers) resolves the rebound receiver
    /// from `$newThis` — PHP's second argument — and the visibility scope from `$scope`, falling
    /// back to the lexically enclosing class when `$scope` is omitted. A receiver whose class is
    /// not statically known yields `None`, so an unresolved rebind keeps its existing diagnostics
    /// rather than being mistyped.
    ///
    /// The parameter shape requires a literal `$scope` and a body provably free of
    /// `$this`/`self::`/`static::`/`parent::`. Parameters declared with a type equal to or a
    /// subclass of the scope become `eligible_params`, as do untyped parameters once inference
    /// narrows them to an object in that scope. Variables created inside the closure are also
    /// eligible, while captured variables remain excluded.
    pub(crate) fn closure_bind_scope_context(
        &mut self,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Option<crate::types::checker::BoundScopeContext> {
        let closure = args.first()?;
        let ExprKind::Closure {
            params,
            body,
            variadic,
            captures,
            capture_refs,
            ..
        } = &closure.kind
        else {
            return None;
        };
        // `Closure::bind(fn () => …$this…, $newThis, Scope::class)`: the body's `$this` is
        // `$newThis`. `crate::ir_lower` rebinds it for any body — the single-`return $this->prop`
        // form through `closure_bind_property_return_type` (which additionally carries a
        // by-reference return), every other form through the generic bind, which installs
        // `$newThis` as the closure's receiver so members dispatch against it at runtime.
        // `closure_body_rebinds_this_only` keeps `self::`/`static::`/`parent::` bodies out: those
        // codegen resolves against the LEXICAL class, so the checker must not authorize them
        // against the rebound scope.
        if params.is_empty()
            && crate::types::checker::inference::expr::static_closure::closure_body_rebinds_this_only(
                body,
            )
        {
            // PHP takes the rebound `$this` from `$newThis` (argument two). `$scope` only widens
            // the visibility the body is checked under, so resolve the two independently instead
            // of reading the property off the scope. `infer_type` already carries `instanceof`
            // narrowing for the receiver local, so a guarded parameter resolves here too.
            let this_class = args
                .get(1)
                .and_then(|new_this| self.infer_type(new_this, env).ok())
                .as_ref()
                .and_then(crate::types::checker::single_object_class_name)?;
            // With no explicit `$scope`, PHP keeps the closure's current scope — the lexically
            // enclosing class. Outside a class the closure is scope-less, so decline the context
            // rather than invent a scope that would authorize non-public members.
            let scope_class = match args.get(2) {
                Some(scope_arg) => self.closure_bind_scope_class(scope_arg, env)?,
                None => self.current_class.clone()?,
            };
            return Some(crate::types::checker::BoundScopeContext {
                scope_class,
                this_class: Some(this_class),
                eligible_params: std::collections::HashSet::new(),
                declared_params: std::collections::HashSet::new(),
                captured_variables: std::collections::HashSet::new(),
                this_receiver_scope: true,
                rebinds_relative_static: false,
            });
        }
        // `Closure::bind(static fn () => self::$x, $newThis, Scope::class)`: the body's relative
        // static receivers resolve to the BIND SCOPE, not the lexical class. `crate::ir_lower`
        // lowers such a body with `current_class` swapped to the scope, so the checker swaps the
        // same way (`rebinds_relative_static`) rather than authorizing a resolution codegen would
        // not reproduce. The scope must be one of the two literal spellings ir_lower resolves
        // identically, so an unresolvable scope keeps today's loud diagnostics.
        if params.is_empty()
            && crate::types::checker::closure_body_rebinds_scope_only(body)
        {
            let scope_class = self.closure_bind_lockstep_scope_class(args.get(2)?)?;
            return Some(crate::types::checker::BoundScopeContext {
                scope_class,
                this_class: None,
                eligible_params: std::collections::HashSet::new(),
                declared_params: std::collections::HashSet::new(),
                captured_variables: std::collections::HashSet::new(),
                this_receiver_scope: false,
                rebinds_relative_static: true,
            });
        }
        // The parameter-based relaxation below reasons purely about `$scope`, so it keeps
        // requiring an explicit literal third argument.
        let scope_arg = args.get(2)?;
        let scope_class = self.closure_bind_scope_class(scope_arg, env)?;
        if !crate::types::checker::inference::expr::static_closure::closure_body_free_of_self_scope(
            body,
        ) {
            return None;
        }
        let mut eligible_params = std::collections::HashSet::new();
        let mut declared_params = std::collections::HashSet::new();
        for (param_name, param_type, _default, _by_ref) in params {
            declared_params.insert(param_name.clone());
            let Some(type_expr) = param_type else {
                eligible_params.insert(param_name.clone());
                continue;
            };
            let Ok(PhpType::Object(param_class)) =
                self.resolve_type_expr(type_expr, scope_arg.span)
            else {
                continue;
            };
            if param_class == scope_class || self.is_subclass_of(&param_class, &scope_class) {
                eligible_params.insert(param_name.clone());
            }
        }
        if let Some(variadic) = variadic {
            declared_params.insert(variadic.clone());
        }
        let captured_variables = captures
            .iter()
            .chain(capture_refs)
            .cloned()
            .collect();
        Some(crate::types::checker::BoundScopeContext {
            scope_class,
            this_class: None,
            eligible_params,
            declared_params,
            captured_variables,
            this_receiver_scope: false,
            rebinds_relative_static: false,
        })
    }

    /// Resolves a `Closure::bind` `$scope` argument for the `rebinds_relative_static` shape.
    ///
    /// Deliberately narrower than `closure_bind_scope_class`: only the two spellings
    /// `crate::ir_lower` resolves — `X::class` over a NAMED receiver, and a class-name string
    /// literal — and only when the written name is a declared class VERBATIM. ir_lower looks the
    /// trimmed spelling up in the same class table with no case folding and no alias resolution, so
    /// accepting a case-insensitive or aliased match here would let the checker approve a scope
    /// codegen resolves to a different class. A declined scope simply leaves the rebind unrelaxed.
    fn closure_bind_lockstep_scope_class(&self, scope_arg: &Expr) -> Option<String> {
        let written = match &scope_arg.kind {
            ExprKind::StringLiteral(name) => name.trim_start_matches('\\'),
            ExprKind::ClassConstant {
                receiver: StaticReceiver::Named(name),
            } => name.as_str().trim_start_matches('\\'),
            _ => return None,
        };
        self.classes
            .contains_key(written)
            .then(|| written.to_string())
    }

    /// Resolves a `Closure::bind` `$scope` argument (`X::class` or a class-name string literal) to a
    /// concrete class name, or `None` for a dynamic/unsupported scope expression.
    ///
    /// PHP accepts either a class name (`X::class`, a class-name string) or an OBJECT, whose
    /// class becomes the scope — the form Symfony uses when it passes the rebind target as both
    /// `$newThis` and `$scope`. A scope expression whose class is not statically known stays
    /// `None` so the caller declines to build a context rather than guessing a visibility scope.
    fn closure_bind_scope_class(&mut self, scope_arg: &Expr, env: &TypeEnv) -> Option<String> {
        match &scope_arg.kind {
            ExprKind::StringLiteral(name) => self
                .resolve_callable_array_class_name(name.trim_start_matches('\\'))
                .map(str::to_string),
            ExprKind::ClassConstant { receiver } => self
                .resolve_callable_array_static_receiver_class(receiver, scope_arg.span)
                .ok(),
            _ => self
                .infer_type(scope_arg, env)
                .ok()
                .as_ref()
                .and_then(crate::types::checker::single_object_class_name),
        }
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

/// Returns whether the trailing variadic parameter has an explicit PHP element type declaration.
///
/// Declared variadics such as `mixed ...$args` and `int ...$values` must keep that contract
/// unchanged; observed call arguments only specialize variadics without a type hint.
fn method_variadic_param_is_declared(
    declared_flags: &[bool],
    regular_param_count: usize,
) -> bool {
    declared_flags
        .get(regular_param_count)
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
