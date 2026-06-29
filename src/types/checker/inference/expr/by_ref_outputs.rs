//! Purpose:
//! Resolves which caller variables become defined by being passed to a user-defined
//! function/method/static-method by-reference parameter, mirroring the builtin
//! `preg_match`/`preg_replace` out-parameter handling for user callables.
//!
//! Called from:
//! - `crate::types::checker::Checker::infer_type_with_assignment_effects()` (effects.rs)
//!
//! Key details:
//! - A by-reference parameter that receives an as-yet-undefined plain `$variable`
//!   defines that variable in the caller scope (PHP definite-assignment semantics).
//! - The inserted type matches what call validation enforces: declared parameter
//!   types are inserted verbatim (so the boxed/nullable storage and compatibility
//!   checks see an identical type), while undeclared parameters insert `Mixed`.
//! - Positional argument shapes only: calls using named or spread arguments bail so
//!   the positional parameter mapping cannot be misaligned.

use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{Expr, ExprKind, StaticReceiver};
use crate::types::{FunctionSig, PhpType, TypeEnv};

use super::super::super::Checker;

impl Checker {
    /// Returns `(name, type)` pairs for currently-undefined plain `$variable` arguments that a
    /// user function call binds to by-reference parameters, so the caller scope can define them.
    ///
    /// Returns an empty vector for builtins, extern functions, or unknown callees (their
    /// by-reference semantics are handled by dedicated builtin paths or do not apply).
    pub(crate) fn function_call_by_ref_outputs(
        &self,
        name: &Name,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let Some(canonical) = self.canonical_function_name_folded(name.as_str()) else {
            return Vec::new();
        };
        let Some(sig) = self.functions.get(&canonical) else {
            return Vec::new();
        };
        Self::sig_undefined_by_ref_variable_outputs(sig, args, env)
    }

    /// Returns `(name, type)` pairs for currently-undefined plain `$variable` arguments bound to
    /// by-reference parameters of a static method call (`Class::method(...)`, `self::`/`static::`/
    /// `parent::`). Returns an empty vector when the receiver class or method cannot be resolved.
    pub(crate) fn static_method_call_by_ref_outputs(
        &self,
        receiver: &StaticReceiver,
        method: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let Some(class_name) = self.resolve_static_receiver_class_for_by_ref(receiver) else {
            return Vec::new();
        };
        let Some(class_info) = self.classes.get(&class_name) else {
            return Vec::new();
        };
        let method_key = php_symbol_key(method);
        let Some(sig) = class_info
            .static_methods
            .get(&method_key)
            .or_else(|| class_info.methods.get(&method_key))
        else {
            return Vec::new();
        };
        Self::sig_undefined_by_ref_variable_outputs(sig, args, env)
    }

    /// Returns `(name, type)` pairs for currently-undefined plain `$variable` arguments bound to
    /// by-reference parameters of an instance method call (`$obj->method(...)`), using the
    /// already-inferred receiver type. Returns an empty vector for non-object or unknown receivers.
    pub(crate) fn method_call_by_ref_outputs(
        &self,
        object_type: &PhpType,
        method: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let PhpType::Object(class_name) = object_type else {
            return Vec::new();
        };
        let Some(class_info) = self.classes.get(class_name) else {
            return Vec::new();
        };
        let method_key = php_symbol_key(method);
        let Some(sig) = class_info.methods.get(&method_key) else {
            return Vec::new();
        };
        Self::sig_undefined_by_ref_variable_outputs(sig, args, env)
    }

    /// Returns `(name, type)` promotions for already-defined plain `$variable` arguments a user
    /// function call passes to a declared by-reference parameter whose boxed/nullable storage the
    /// variable's current type cannot provide. The caller scope must promote each variable so the
    /// by-reference writeback is sound (the callee may store any value the parameter permits).
    pub(crate) fn function_call_by_ref_boxed_promotions(
        &self,
        name: &Name,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        // Resolve through `fn_decls` (a complete table populated before body checking), not
        // `self.functions`, which is filled incrementally as bodies are checked and so would
        // miss a callee declared later in source order.
        let canonical = self
            .canonical_function_name_folded(name.as_str())
            .unwrap_or_else(|| name.as_str().to_string());
        let Some(decl) = self.fn_decls.get(&canonical) else {
            return Vec::new();
        };
        if !decl.ref_params.iter().any(|is_ref| *is_ref) {
            return Vec::new();
        }
        if args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. } | ExprKind::Spread(_)))
        {
            return Vec::new();
        }
        let mut promotions = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            let ExprKind::Variable(var_name) = &arg.kind else {
                continue;
            };
            if !decl.ref_params.get(index).copied().unwrap_or(false) {
                continue;
            }
            let Some(type_ann) = decl.param_types.get(index).and_then(|ann| ann.as_ref()) else {
                continue;
            };
            let Some(current_ty) = env.get(var_name) else {
                continue;
            };
            let Ok(param_ty) = self.resolve_declared_param_type_hint(
                type_ann,
                decl.span,
                "by-reference parameter",
            ) else {
                continue;
            };
            if self.by_ref_param_needs_storage_promotion(&param_ty, current_ty) {
                let promoted = self.normalize_union_type(vec![current_ty.clone(), param_ty]);
                promotions.push((var_name.clone(), promoted));
            }
        }
        promotions
    }

    /// Returns by-reference storage promotions for already-defined `$variable` arguments of a
    /// static method call (`Class::method(...)`, `self::`/`static::`/`parent::`), mirroring
    /// `function_call_by_ref_boxed_promotions`. Empty when the receiver class or method is unknown.
    pub(crate) fn static_method_call_by_ref_boxed_promotions(
        &self,
        receiver: &StaticReceiver,
        method: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let Some(class_name) = self.resolve_static_receiver_class_for_by_ref(receiver) else {
            return Vec::new();
        };
        let Some(class_info) = self.classes.get(&class_name) else {
            return Vec::new();
        };
        let method_key = php_symbol_key(method);
        let Some(sig) = class_info
            .static_methods
            .get(&method_key)
            .or_else(|| class_info.methods.get(&method_key))
        else {
            return Vec::new();
        };
        self.sig_defined_by_ref_boxed_promotions(sig, args, env)
    }

    /// Returns by-reference storage promotions for already-defined `$variable` arguments of an
    /// instance method call (`$obj->method(...)`), using the already-inferred receiver type.
    /// Empty for non-object or unknown receivers.
    pub(crate) fn method_call_by_ref_boxed_promotions(
        &self,
        object_type: &PhpType,
        method: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let PhpType::Object(class_name) = object_type else {
            return Vec::new();
        };
        let Some(class_info) = self.classes.get(class_name) else {
            return Vec::new();
        };
        let method_key = php_symbol_key(method);
        let Some(sig) = class_info.methods.get(&method_key) else {
            return Vec::new();
        };
        self.sig_defined_by_ref_boxed_promotions(sig, args, env)
    }

    /// Collects `(name, join-type)` promotions for already-defined plain `$variable` arguments
    /// bound to a declared by-reference parameter that requires boxed/nullable storage the
    /// variable cannot currently provide. The promotion type is the least-upper-bound JOIN of the
    /// variable's current type and the parameter's declared type, which lowers to boxed `Mixed`
    /// (or inline nullable) storage. Bails on named/spread arguments (positional mapping only).
    fn sig_defined_by_ref_boxed_promotions(
        &self,
        sig: &FunctionSig,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        if !sig.ref_params.iter().any(|is_ref| *is_ref) {
            return Vec::new();
        }
        if args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. } | ExprKind::Spread(_)))
        {
            return Vec::new();
        }
        let mut promotions = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            let ExprKind::Variable(var_name) = &arg.kind else {
                continue;
            };
            if !sig.ref_params.get(index).copied().unwrap_or(false) {
                continue;
            }
            if !sig.declared_params.get(index).copied().unwrap_or(false) {
                continue;
            }
            let Some(current_ty) = env.get(var_name) else {
                continue;
            };
            let Some((_, param_ty)) = sig.params.get(index) else {
                continue;
            };
            if self.by_ref_param_needs_storage_promotion(param_ty, current_ty) {
                let promoted =
                    self.normalize_union_type(vec![current_ty.clone(), param_ty.clone()]);
                promotions.push((var_name.clone(), promoted));
            }
        }
        promotions
    }

    /// Returns a best-effort static return type for a call or `new` expression used as an
    /// assignment right-hand side that failed to type-check, so error recovery can bind the
    /// assigned variable to an accurate type instead of the infectious `Mixed`.
    ///
    /// Binding the declared return type (rather than defaulting to `Mixed`) keeps a recovered
    /// binding from spuriously widening unrelated typed code that later observes the variable —
    /// e.g. an object handle that would otherwise become `Mixed` and trip a typed-parameter check.
    /// Returns `None` when the callee or its return type cannot be resolved, in which case the
    /// caller falls back to `Mixed`.
    pub(crate) fn assignment_recovery_call_return_type(&self, value: &Expr) -> Option<PhpType> {
        match &value.kind {
            ExprKind::FunctionCall { name, .. } => {
                let canonical = self.canonical_function_name_folded(name.as_str())?;
                self.functions
                    .get(&canonical)
                    .map(|sig| sig.return_type.clone())
            }
            ExprKind::StaticMethodCall {
                receiver, method, ..
            } => {
                let class_name = self.resolve_static_receiver_class_for_by_ref(receiver)?;
                let class_info = self.classes.get(&class_name)?;
                let method_key = php_symbol_key(method);
                class_info
                    .static_methods
                    .get(&method_key)
                    .or_else(|| class_info.methods.get(&method_key))
                    .map(|sig| sig.return_type.clone())
            }
            ExprKind::NewObject { class_name, .. } => {
                let class_key = php_symbol_key(class_name.as_str().trim_start_matches('\\'));
                self.classes
                    .keys()
                    .find(|existing| php_symbol_key(existing) == class_key)
                    .map(|class| PhpType::Object(class.clone()))
            }
            _ => None,
        }
    }

    /// Resolves a static-call receiver to a concrete class name for by-reference output lookup,
    /// mirroring static method metadata resolution: `Named` resolves case-insensitively,
    /// `self`/`static` use the enclosing class, and `parent` uses its parent.
    fn resolve_static_receiver_class_for_by_ref(
        &self,
        receiver: &StaticReceiver,
    ) -> Option<String> {
        match receiver {
            StaticReceiver::Named(name) => {
                let class_key = php_symbol_key(name.as_str().trim_start_matches('\\'));
                self.classes
                    .keys()
                    .find(|existing| php_symbol_key(existing) == class_key)
                    .cloned()
            }
            StaticReceiver::Self_ | StaticReceiver::Static => self.current_class.clone(),
            StaticReceiver::Parent => self
                .current_class
                .as_ref()
                .and_then(|current| self.classes.get(current))
                .and_then(|class_info| class_info.parent.clone()),
        }
    }

    /// Defines, in `env`, every by-reference output variable produced by a call appearing
    /// anywhere within `expr`, inserting each currently-undefined caller variable.
    ///
    /// `infer_type_with_assignment_effects` evaluates the right operand of `&&`/`||` and each
    /// ternary/`match`/`??` branch in a cloned environment so ordinary assignments cannot leak
    /// past a short-circuit boundary. By-reference out-parameters, however, follow PHP's
    /// (non-flow-sensitive) undefined-variable behavior: once such a call appears in an
    /// expression, the bound variable is treated as defined in the code that runs afterwards
    /// (and inside the guarded body of an `if`/`while`). This re-surfaces those definitions from
    /// the cloned branches into the real environment so later uses are not reported as undefined.
    pub(crate) fn define_nested_by_ref_outputs(&self, expr: &Expr, env: &mut TypeEnv) {
        let mut outputs = Vec::new();
        self.collect_nested_by_ref_outputs(expr, env, &mut outputs);
        for (var, ty) in outputs {
            env.entry(var).or_insert(ty);
        }
    }

    /// Recursively collects `(name, type)` definitions for currently-undefined by-reference
    /// output variables of every call within `expr`, appending them to `out`.
    ///
    /// User functions and static methods resolve their outputs through the shared signature
    /// helpers; the builtin `preg_match`/`preg_replace` output arguments are handled inline to
    /// mirror the dedicated builtin paths. Instance-method receivers are not resolved here (that
    /// would require mutable inference), but their sub-expressions are still scanned. Closure
    /// bodies are intentionally skipped: their by-reference calls bind variables in the closure
    /// scope, not the caller's.
    fn collect_nested_by_ref_outputs(
        &self,
        expr: &Expr,
        env: &TypeEnv,
        out: &mut Vec<(String, PhpType)>,
    ) {
        match &expr.kind {
            ExprKind::FunctionCall { name, args } => {
                let expanded = crate::types::call_args::expand_static_assoc_spread_args(args);
                out.extend(self.function_call_by_ref_outputs(name, &expanded, env));
                let builtin = name.trim_start_matches('\\');
                if builtin.eq_ignore_ascii_case("preg_match") {
                    push_builtin_preg_output(
                        expanded.get(2),
                        PhpType::Array(Box::new(PhpType::Str)),
                        env,
                        out,
                    );
                } else if builtin.eq_ignore_ascii_case("preg_replace") {
                    push_builtin_preg_output(expanded.get(4), PhpType::Int, env, out);
                }
                for arg in &expanded {
                    self.collect_nested_by_ref_outputs(arg, env, out);
                }
            }
            ExprKind::StaticMethodCall {
                receiver,
                method,
                args,
            } => {
                let expanded = crate::types::call_args::expand_static_assoc_spread_args(args);
                out.extend(self.static_method_call_by_ref_outputs(receiver, method, &expanded, env));
                for arg in &expanded {
                    self.collect_nested_by_ref_outputs(arg, env, out);
                }
            }
            ExprKind::MethodCall { object, args, .. }
            | ExprKind::NullsafeMethodCall { object, args, .. } => {
                self.collect_nested_by_ref_outputs(object, env, out);
                for arg in args {
                    self.collect_nested_by_ref_outputs(arg, env, out);
                }
            }
            ExprKind::BinaryOp { left, right, .. }
            | ExprKind::NullCoalesce {
                value: left,
                default: right,
            }
            | ExprKind::ShortTernary {
                value: left,
                default: right,
            }
            | ExprKind::Pipe {
                value: left,
                callable: right,
            } => {
                self.collect_nested_by_ref_outputs(left, env, out);
                self.collect_nested_by_ref_outputs(right, env, out);
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.collect_nested_by_ref_outputs(condition, env, out);
                self.collect_nested_by_ref_outputs(then_expr, env, out);
                self.collect_nested_by_ref_outputs(else_expr, env, out);
            }
            ExprKind::Match {
                subject,
                arms,
                default,
            } => {
                self.collect_nested_by_ref_outputs(subject, env, out);
                for (conditions, result) in arms {
                    for condition in conditions {
                        self.collect_nested_by_ref_outputs(condition, env, out);
                    }
                    self.collect_nested_by_ref_outputs(result, env, out);
                }
                if let Some(default) = default {
                    self.collect_nested_by_ref_outputs(default, env, out);
                }
            }
            ExprKind::InstanceOf { value, .. } => {
                self.collect_nested_by_ref_outputs(value, env, out);
            }
            ExprKind::Negate(inner)
            | ExprKind::Not(inner)
            | ExprKind::BitNot(inner)
            | ExprKind::Throw(inner)
            | ExprKind::ErrorSuppress(inner)
            | ExprKind::Print(inner)
            | ExprKind::Clone(inner)
            | ExprKind::Spread(inner)
            | ExprKind::Cast { expr: inner, .. }
            | ExprKind::PtrCast { expr: inner, .. }
            | ExprKind::BufferNew { len: inner, .. }
            | ExprKind::YieldFrom(inner) => {
                self.collect_nested_by_ref_outputs(inner, env, out);
            }
            ExprKind::Assignment {
                target,
                value,
                result_target,
                ..
            } => {
                self.collect_nested_by_ref_outputs(target, env, out);
                self.collect_nested_by_ref_outputs(value, env, out);
                if let Some(result_target) = result_target {
                    self.collect_nested_by_ref_outputs(result_target, env, out);
                }
            }
            ExprKind::ListUnpack { value, .. } => {
                self.collect_nested_by_ref_outputs(value, env, out);
            }
            ExprKind::ArrayAccess { array, index } => {
                self.collect_nested_by_ref_outputs(array, env, out);
                self.collect_nested_by_ref_outputs(index, env, out);
            }
            ExprKind::ArrayLiteral(elems) => {
                for elem in elems {
                    self.collect_nested_by_ref_outputs(elem, env, out);
                }
            }
            ExprKind::ArrayLiteralAssoc(pairs) => {
                for (key, value) in pairs {
                    self.collect_nested_by_ref_outputs(key, env, out);
                    self.collect_nested_by_ref_outputs(value, env, out);
                }
            }
            ExprKind::NamedArg { value, .. } => {
                self.collect_nested_by_ref_outputs(value, env, out);
            }
            ExprKind::PropertyAccess { object, .. }
            | ExprKind::NullsafePropertyAccess { object, .. } => {
                self.collect_nested_by_ref_outputs(object, env, out);
            }
            ExprKind::DynamicPropertyAccess { object, property }
            | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
                self.collect_nested_by_ref_outputs(object, env, out);
                self.collect_nested_by_ref_outputs(property, env, out);
            }
            ExprKind::ExprCall { callee, args } => {
                self.collect_nested_by_ref_outputs(callee, env, out);
                for arg in args {
                    self.collect_nested_by_ref_outputs(arg, env, out);
                }
            }
            ExprKind::ClosureCall { args, .. }
            | ExprKind::NewObject { args, .. }
            | ExprKind::NewScopedObject { args, .. } => {
                for arg in args {
                    self.collect_nested_by_ref_outputs(arg, env, out);
                }
            }
            _ => {}
        }
    }

    /// Collects `(name, type)` pairs for plain `$variable` arguments that are bound to a
    /// by-reference parameter and are not yet defined in `env`.
    ///
    /// Bails (returns empty) when the call uses named or spread arguments, because the positional
    /// parameter mapping used here would otherwise be misaligned. The inserted type matches call
    /// validation: declared parameter types verbatim, `Mixed` for undeclared parameters.
    fn sig_undefined_by_ref_variable_outputs(
        sig: &FunctionSig,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        if !sig.ref_params.iter().any(|is_ref| *is_ref) {
            return Vec::new();
        }
        if args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. } | ExprKind::Spread(_)))
        {
            return Vec::new();
        }
        let mut outputs = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            let ExprKind::Variable(var_name) = &arg.kind else {
                continue;
            };
            if !sig.ref_params.get(index).copied().unwrap_or(false) {
                continue;
            }
            if env.contains_key(var_name) {
                continue;
            }
            let inserted_ty = if sig.declared_params.get(index).copied().unwrap_or(false) {
                sig.params
                    .get(index)
                    .map(|(_, ty)| ty.clone())
                    .unwrap_or(PhpType::Mixed)
            } else {
                PhpType::Mixed
            };
            outputs.push((var_name.clone(), inserted_ty));
        }
        outputs
    }
}

/// Pushes the by-reference output of a builtin `preg_match`/`preg_replace` argument when it is a
/// plain `$variable` (or a named argument wrapping one) that is not yet defined in `env`.
fn push_builtin_preg_output(
    arg: Option<&Expr>,
    ty: PhpType,
    env: &TypeEnv,
    out: &mut Vec<(String, PhpType)>,
) {
    if let Some(var) = arg.and_then(builtin_preg_output_var) {
        if !env.contains_key(var) {
            out.push((var.clone(), ty));
        }
    }
}

/// Returns the variable name used as a builtin `preg_match`/`preg_replace` output argument,
/// unwrapping a named-argument wrapper.
fn builtin_preg_output_var(arg: &Expr) -> Option<&String> {
    match &arg.kind {
        ExprKind::Variable(name) => Some(name),
        ExprKind::NamedArg { value, .. } => builtin_preg_output_var(value),
        _ => None,
    }
}
