//! Purpose:
//! Resolves which caller variables become defined by being passed to a user-defined
//! function/method/static-method by-reference parameter, and to a builtin by-reference
//! out-parameter (`preg_match`/`preg_match_all`/`preg_replace`/`parse_str`/`proc_open`).
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
//! - Builtin out-parameter types come from `builtin_out_param_type`, which records the
//!   PHP-accurate element type for each known by-ref builtin out-param (defaulting to
//!   `Mixed` for any future builtin whose signature marks `ref_params` without a table
//!   entry).
//! - OUT-ONLY builtin by-ref params (`builtin_out_only_ref_param`: preg_match/preg_match_all
//!   `$matches`, preg_replace/preg_replace_callback `$count`) overwrite the caller variable
//!   wholesale, so they re-type even an ALREADY-defined variable (PHP replaces it regardless of
//!   its prior value — e.g. a subject reused as the out-param). IN-OUT params (sort, array_push, …)
//!   keep the caller's existing type and are only used to define currently-undefined variables.
//! - Named arguments use the canonical call-argument planner before by-reference
//!   outputs are matched; dynamic spread elements remain unpromoted because their
//!   runtime parameter mapping cannot be proven statically.

use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{BinOp, Expr, ExprKind, StaticReceiver};
use crate::types::call_args::{plan_call_args, PlannedRegularArg};
use crate::types::preg_constants::PREG_INT_CONSTANTS;
use crate::types::{FunctionSig, PhpType, TypeEnv};

use super::super::super::Checker;

// Retained for the checker unit tests and pending re-integration after the
// origin/main merge; not all entry points are reachable from the production
// checker paths yet.
#[allow(dead_code)]
impl Checker {
    /// Returns `(name, type)` pairs for currently-undefined plain `$variable` arguments that a
    /// user function call binds to by-reference parameters, so the caller scope can define them.
    ///
    /// Returns an empty vector for builtins, extern functions, or unknown callees (their
    /// by-reference semantics are handled by dedicated builtin paths or do not apply).
    // Retained for the checker unit tests and pending re-integration after the
    // origin/main merge; not reachable from the production checker paths yet.
    #[allow(dead_code)]
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

    /// Returns `(name, type)` pairs for currently-undefined plain `$variable` arguments that a
    /// builtin call binds to by-reference out-parameters, so the caller scope can define them.
    ///
    /// Mirrors `function_call_by_ref_outputs` for builtins: the by-ref param positions come from
    /// the canonical builtin signature's `ref_params`, and the inserted type comes from
    /// `builtin_out_param_type` (which records the PHP-accurate element type for each known
    /// builtin out-param, defaulting to `Mixed`). Returns an empty vector when the builtin has no
    /// signature, no by-ref params, or the call uses named/spread arguments (positional mapping
    /// only).
    pub(crate) fn builtin_call_by_ref_outputs(
        builtin_name: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let Some(sig) = crate::types::builtin_call_sig(builtin_name) else {
            return Vec::new();
        };
        if !sig.ref_params.iter().any(|is_ref| *is_ref) {
            return Vec::new();
        }
        // A live `Spread` operand reorders positional parameters unpredictably, so bail to keep
        // the positional `ref_params` mapping sound. `NamedArg` wrappers are unwrapped (the call
        // validation layer already ensures the name resolves), mirroring the previous hardcoded
        // `preg_match`/`preg_replace` path which unwrapped `matches: $var`.
        if args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::Spread(_)))
        {
            return Vec::new();
        }
        let mut outputs = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            let var_name = match &arg.kind {
                ExprKind::Variable(name) => name,
                ExprKind::NamedArg { value, .. } => match &value.kind {
                    ExprKind::Variable(name) => name,
                    _ => continue,
                },
                _ => continue,
            };
            if !sig.ref_params.get(index).copied().unwrap_or(false) {
                continue;
            }
            // OUT-ONLY by-ref params (preg_match/preg_match_all `$matches`,
            // preg_replace/preg_replace_callback `$count`) overwrite the caller variable
            // wholesale — PHP replaces it regardless of its prior value — so an already-defined
            // variable (e.g. a subject reused as the out-param: `preg_match_all(…, $s, $s)`) must
            // still be re-typed to the out-param output type. IN-OUT params (sort, array_push, …)
            // mutate in place and keep the caller's existing (usually more specific) type, so they
            // are skipped when already defined, preserving today's behavior.
            if env.contains_key(var_name) && !builtin_out_only_ref_param(builtin_name, index) {
                continue;
            }
            let inserted_ty = builtin_out_param_type(builtin_name, index, args);
            outputs.push((var_name.clone(), inserted_ty));
        }
        outputs
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
    /// already-inferred receiver type. Resolves the receiver through classes, interfaces, and
    /// union receivers (see `resolve_method_sig_for_by_ref`). Returns an empty vector for
    /// non-object, unresolved, or unknown receivers.
    pub(crate) fn method_call_by_ref_outputs(
        &self,
        object_type: &PhpType,
        method: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let method_key = php_symbol_key(method);
        let Some(sig) = self.resolve_method_sig_for_by_ref(object_type, &method_key) else {
            return Vec::new();
        };
        Self::sig_undefined_by_ref_variable_outputs(sig, args, env)
    }

    /// Returns undefined caller variables bound to by-reference parameters when invoking a local
    /// whose callable signature was resolved from a closure, first-class callable, or finite
    /// builtin function-name expression.
    pub(crate) fn callable_variable_by_ref_outputs(
        &self,
        var: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let Some(sig) = self.callable_sigs.get(var) else {
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
    /// Resolves the receiver through classes, interfaces, and union receivers (see
    /// `resolve_method_sig_for_by_ref`). Empty for non-object, unresolved, or unknown receivers.
    pub(crate) fn method_call_by_ref_boxed_promotions(
        &self,
        object_type: &PhpType,
        method: &str,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        let method_key = php_symbol_key(method);
        let Some(sig) = self.resolve_method_sig_for_by_ref(object_type, &method_key) else {
            return Vec::new();
        };
        self.sig_defined_by_ref_boxed_promotions(sig, args, env)
    }

    /// Resolves the method signature relevant to by-reference output/promotion analysis for a
    /// method-call receiver type, covering plain class objects, interface objects (a class or
    /// interface method call resolves identically here — both store `ref_params` through the
    /// shared `build_method_sig`), and union receivers.
    ///
    /// A `PhpType::Object` receiver is looked up in `self.classes` first, then `self.interfaces`
    /// (an interface-typed variable, e.g. a constructor-promoted `private MarshallerInterface
    /// $marshaller`, infers to `PhpType::Object("MarshallerInterface")` — the same representation
    /// as a concrete class — so the class-only lookup previously missed every interface-typed
    /// receiver's by-reference out-params). A `PhpType::Union` receiver (`Interface|null`,
    /// `A|B`, …) first tries the single-object-class resolution (nullable receiver), then falls
    /// back to the first union member that resolves the method, mirroring
    /// `infer_method_call_on_object_union`'s "the method only needs to exist on at least one
    /// member" convention. Returns `None` for non-object, unresolved, or unknown receivers.
    fn resolve_method_sig_for_by_ref(
        &self,
        object_type: &PhpType,
        method_key: &str,
    ) -> Option<&FunctionSig> {
        match object_type {
            PhpType::Object(class_name) => self.class_or_interface_method_sig(class_name, method_key),
            PhpType::Union(_) => {
                if let Some(class_name) = self.union_single_object_class(object_type) {
                    return self.class_or_interface_method_sig(&class_name, method_key);
                }
                self.union_object_classes(object_type)
                    .iter()
                    .find_map(|class_name| self.class_or_interface_method_sig(class_name, method_key))
            }
            _ => None,
        }
    }

    /// Looks up a method's `FunctionSig` on a resolved class name, checking `self.classes` then
    /// falling back to `self.interfaces` — a class name and an interface name never collide (PHP
    /// class-like declarations share one global symbol table), so this is an unambiguous,
    /// order-independent lookup across both metadata tables.
    fn class_or_interface_method_sig(&self, class_name: &str, method_key: &str) -> Option<&FunctionSig> {
        if let Some(sig) = self
            .classes
            .get(class_name)
            .and_then(|class_info| class_info.methods.get(method_key))
        {
            return Some(sig);
        }
        self.interfaces
            .get(class_name)
            .and_then(|interface_info| interface_info.methods.get(method_key))
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
                out.extend(Self::builtin_call_by_ref_outputs(builtin, &expanded, env));
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
                if let ExprKind::Variable(var) = &callee.kind {
                    out.extend(self.callable_variable_by_ref_outputs(var, args, env));
                }
                self.collect_nested_by_ref_outputs(callee, env, out);
                for arg in args {
                    self.collect_nested_by_ref_outputs(arg, env, out);
                }
            }
            ExprKind::ClosureCall { var, args } => {
                out.extend(self.callable_variable_by_ref_outputs(var, args, env));
                for arg in args {
                    self.collect_nested_by_ref_outputs(arg, env, out);
                }
            }
            ExprKind::NewObject { args, .. } | ExprKind::NewScopedObject { args, .. } => {
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
    /// Resolves named arguments through the shared call-argument planner so their source variables
    /// are matched against the correct parameter positions. Dynamic spread elements are skipped
    /// because their runtime parameter mapping is not statically known. The inserted type matches
    /// call validation: declared parameter types verbatim, `Mixed` for undeclared parameters.
    fn sig_undefined_by_ref_variable_outputs(
        sig: &FunctionSig,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Vec<(String, PhpType)> {
        if !sig.ref_params.iter().any(|is_ref| *is_ref) {
            return Vec::new();
        }

        let mut outputs = Vec::new();
        let mut collect_output = |index: usize, arg: &Expr| {
            let ExprKind::Variable(var_name) = &arg.kind else {
                return;
            };
            if !sig.ref_params.get(index).copied().unwrap_or(false) {
                return;
            }
            if env.contains_key(var_name) {
                return;
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
        };

        if args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. }))
        {
            let call_span = args
                .first()
                .map(|arg| arg.span)
                .unwrap_or_else(crate::span::Span::dummy);
            let Ok(plan) = plan_call_args(
                sig,
                args,
                call_span,
                false,
                sig.variadic.is_some(),
            ) else {
                return Vec::new();
            };
            for (index, arg) in plan.regular_args.iter().enumerate() {
                if let PlannedRegularArg::Source { expr, .. } = arg {
                    collect_output(index, expr);
                }
            }
        } else {
            if args
                .iter()
                .any(|arg| matches!(arg.kind, ExprKind::Spread(_)))
            {
                return Vec::new();
            }
            for (index, arg) in args.iter().enumerate() {
                collect_output(index, arg);
            }
        }

        outputs
    }
}

/// Returns `true` when builtin `name`'s by-reference parameter at `index` is OUT-ONLY: the callee
/// overwrites the argument wholesale, so PHP replaces the caller variable's value regardless of its
/// prior contents. For such params an already-defined caller variable must be re-typed to the
/// out-param's output type (`builtin_out_param_type`), which is what makes the aliased
/// `preg_match_all('/…/', $s, $s)` shape (subject reused as `$matches`) re-type `$s` to the matches
/// array instead of leaving it typed `string`.
///
/// IN-OUT by-reference params (`sort`, `rsort`, `array_push`, `array_shift`, `array_splice`,
/// `array_walk`, `settype`, `end`/`reset`/…) are deliberately NOT listed: they mutate the argument
/// in place, so the caller keeps its own (usually more specific) element type and the existing
/// skip-if-defined behavior is correct. When it is unclear whether a param is out-only, it is
/// treated as in-out — the safe default that preserves today's behavior. Matching is
/// case-insensitive via `php_symbol_key`, consistent with the rest of the checker.
// Retained for the checker unit tests and pending re-integration after the
// origin/main merge; not reachable from the production checker paths yet.
#[allow(dead_code)]
fn builtin_out_only_ref_param(name: &str, index: usize) -> bool {
    matches!(
        (php_symbol_key(name).as_str(), index),
        ("preg_match", 2)
            | ("preg_match_all", 2)
            | ("preg_replace", 4)
            | ("preg_replace_callback", 4)
    )
}

/// Returns the PHP-accurate type a builtin by-reference out-parameter writes into the caller's
/// variable, used when auto-vivifying a previously-undefined plain `$variable`.
///
/// The by-ref param POSITIONS come from each builtin signature's `ref_params`; this table records
/// the element type the runtime helper writes, so a freshly-defined variable gets a type that
/// downstream indexing/count reads accept (mirroring the types the previous hardcoded
/// preg_match/preg_replace/parse_str paths inserted). Unknown builtins or positions default to
/// `Mixed`, which is compatible with every later use.
// Retained for the checker unit tests and pending re-integration after the
// origin/main merge; not reachable from the production checker paths yet.
#[allow(dead_code)]
fn builtin_out_param_type(builtin: &str, index: usize, args: &[Expr]) -> PhpType {
    let lower = builtin.to_ascii_lowercase();
    match lower.as_str() {
        // With PREG_OFFSET_CAPTURE each preg_match capture becomes [string, int].
        "preg_match" if index == 2 && preg_flags_may_capture_offsets(args) => {
            PhpType::Array(Box::new(PhpType::Array(Box::new(PhpType::Mixed))))
        }
        // preg_match(&$matches): array of full-match + capture-group strings.
        "preg_match" if index == 2 => PhpType::Array(Box::new(PhpType::Str)),
        // With PREG_OFFSET_CAPTURE each preg_match_all leaf becomes [string, int].
        "preg_match_all" if index == 2 && preg_flags_may_capture_offsets(args) => {
            PhpType::Array(Box::new(PhpType::Array(Box::new(PhpType::Array(
                Box::new(PhpType::Mixed),
            )))))
        }
        // preg_match_all(&$matches): array of (full-match list, capture-group list).
        "preg_match_all" if index == 2 => {
            PhpType::Array(Box::new(PhpType::Array(Box::new(PhpType::Str))))
        }
        // preg_replace(&$count): int replacement count.
        "preg_replace" if index == 4 => PhpType::Int,
        // parse_str(&$result): associative array of parsed key => value pairs.
        "parse_str" if index == 1 => PhpType::AssocArray {
            key: Box::new(PhpType::Str),
            value: Box::new(PhpType::Mixed),
        },
        // proc_open(&$pipes): array of process pipe resources.
        "proc_open" if index == 2 => PhpType::Array(Box::new(PhpType::Mixed)),
        _ => PhpType::Mixed,
    }
}

/// Returns whether a preg call's flags can produce `[string, int]` capture pairs.
///
/// Known literal/constant bitmasks retain the precise string-only shape when the
/// offset bit is absent. A dynamic flags expression is conservatively treated as
/// possibly offset-capturing so later indexing uses boxed `Mixed` leaf values.
fn preg_flags_may_capture_offsets(args: &[Expr]) -> bool {
    let Some(flags) = args.get(3) else {
        return false;
    };
    preg_static_int_value(flags).map_or(true, |value| value & 256 != 0)
}

/// Evaluates the literal and preg-constant bitwise expressions accepted as static flag masks.
fn preg_static_int_value(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::IntLiteral(value) => Some(*value),
        ExprKind::ConstRef(name) => PREG_INT_CONSTANTS
            .iter()
            .find_map(|(constant, value)| (*constant == name.as_str()).then_some(*value)),
        ExprKind::Negate(inner) => preg_static_int_value(inner).map(|value| -value),
        ExprKind::BitNot(inner) => preg_static_int_value(inner).map(|value| !value),
        ExprKind::BinaryOp { left, op, right } => {
            let left = preg_static_int_value(left)?;
            let right = preg_static_int_value(right)?;
            match op {
                BinOp::BitAnd => Some(left & right),
                BinOp::BitOr => Some(left | right),
                BinOp::BitXor => Some(left ^ right),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience: the nested-array type `preg_match_all`'s `$matches` out-param produces.
    fn preg_match_all_matches_type() -> PhpType {
        PhpType::Array(Box::new(PhpType::Array(Box::new(PhpType::Str))))
    }

    /// Builds an expression referring to PHP's offset-capture preg flag.
    fn preg_offset_capture_expr() -> Expr {
        Expr::new(
            ExprKind::ConstRef(Name::unqualified("PREG_OFFSET_CAPTURE")),
            crate::span::Span::dummy(),
        )
    }

    /// Verifies `preg_match` offset captures are typed as `[string, int]` pairs.
    #[test]
    fn test_preg_match_offset_capture_output_has_mixed_pair_leaves() {
        let args = [
            Expr::string_lit("/x/"),
            Expr::string_lit("x"),
            Expr::var("matches"),
            preg_offset_capture_expr(),
        ];
        assert_eq!(
            builtin_out_param_type("preg_match", 2, &args),
            PhpType::Array(Box::new(PhpType::Array(Box::new(PhpType::Mixed)))),
        );
    }

    /// Verifies `preg_match_all` offset captures retain the extra occurrence and pair levels.
    #[test]
    fn test_preg_match_all_offset_capture_output_has_nested_mixed_pairs() {
        let args = [
            Expr::string_lit("/x/"),
            Expr::string_lit("xx"),
            Expr::var("matches"),
            Expr::binop(
                Expr::new(
                    ExprKind::ConstRef(Name::unqualified("PREG_SET_ORDER")),
                    crate::span::Span::dummy(),
                ),
                BinOp::BitOr,
                preg_offset_capture_expr(),
            ),
        ];
        assert_eq!(
            builtin_out_param_type("preg_match_all", 2, &args),
            PhpType::Array(Box::new(PhpType::Array(Box::new(PhpType::Array(
                Box::new(PhpType::Mixed),
            ))))),
        );
    }

    /// Verifies a known zero preg flag keeps the precise string-only matches shape.
    #[test]
    fn test_preg_match_zero_flags_keep_string_capture_type() {
        let args = [
            Expr::string_lit("/x/"),
            Expr::string_lit("x"),
            Expr::var("matches"),
            Expr::int_lit(0),
        ];
        assert_eq!(
            builtin_out_param_type("preg_match", 2, &args),
            PhpType::Array(Box::new(PhpType::Str)),
        );
    }

    /// Verifies the OUT-only classifier lists exactly the four wholesale-overwrite builtin
    /// by-ref params (preg_match/preg_match_all `$matches` at index 2, preg_replace/
    /// preg_replace_callback `$count` at index 4) and rejects the wrong index for each, so the
    /// overwrite never fires on a non-out-only position of an out-only builtin.
    #[test]
    fn test_out_only_ref_param_lists_only_wholesale_overwrite_params() {
        assert!(builtin_out_only_ref_param("preg_match", 2));
        assert!(builtin_out_only_ref_param("preg_match_all", 2));
        assert!(builtin_out_only_ref_param("preg_replace", 4));
        assert!(builtin_out_only_ref_param("preg_replace_callback", 4));
        // Wrong index for an out-only builtin is not out-only.
        assert!(!builtin_out_only_ref_param("preg_match_all", 4));
        assert!(!builtin_out_only_ref_param("preg_replace", 2));
    }

    /// Verifies IN-OUT by-ref builtins (sort, array_push, array_shift, array_splice,
    /// array_walk, settype, end, reset) are NOT classified out-only, so an already-defined
    /// caller variable keeps its own (more specific) type instead of being overwritten.
    #[test]
    fn test_out_only_ref_param_excludes_in_out_builtins() {
        for name in [
            "sort",
            "rsort",
            "array_push",
            "array_shift",
            "array_splice",
            "array_walk",
            "settype",
            "end",
            "reset",
        ] {
            assert!(
                !builtin_out_only_ref_param(name, 0),
                "{name} must be treated as in-out, not out-only",
            );
        }
    }

    /// Verifies the classifier matches builtin names case-insensitively (PHP function names are
    /// case-insensitive), consistent with `php_symbol_key` usage elsewhere in the checker.
    #[test]
    fn test_out_only_ref_param_is_case_insensitive() {
        assert!(builtin_out_only_ref_param("PREG_MATCH_ALL", 2));
        assert!(builtin_out_only_ref_param("Preg_Replace", 4));
    }

    /// Verifies the crux fix: an ALREADY-defined caller variable reused as an OUT-only by-ref
    /// out-param (the aliased `preg_match_all('/…/', $s, $s)` subject-as-out shape) STILL
    /// produces an overwrite entry re-typing it to the out-param output type — instead of being
    /// skipped and left as its prior `string` type.
    #[test]
    fn test_out_only_overwrites_already_defined_aliased_variable() {
        let mut env = TypeEnv::new();
        env.insert("s".to_string(), PhpType::Str);
        let args = [
            Expr::string_lit("/\\w/"),
            Expr::var("s"),
            Expr::var("s"),
        ];
        let outputs = Checker::builtin_call_by_ref_outputs("preg_match_all", &args, &env);
        assert_eq!(
            outputs,
            vec![("s".to_string(), preg_match_all_matches_type())],
        );
    }

    /// Verifies an IN-OUT by-ref builtin (`sort`) does NOT emit an entry for an already-defined
    /// `array<int>` variable, so the consumer's overwrite never fires and the specific element
    /// type is preserved (the regression this fix must not cause).
    #[test]
    fn test_in_out_leaves_already_defined_variable_untouched() {
        let mut env = TypeEnv::new();
        env.insert("a".to_string(), PhpType::Array(Box::new(PhpType::Int)));
        let args = [Expr::var("a")];
        let outputs = Checker::builtin_call_by_ref_outputs("sort", &args, &env);
        assert!(
            outputs.is_empty(),
            "sort on an already-defined array must not re-type it: {outputs:?}",
        );
    }

    /// Verifies the pre-existing undefined-variable path is unchanged: a fresh (undefined) plain
    /// `$m` out-param is still defined with the builtin's out-param type, exactly as before.
    #[test]
    fn test_undefined_out_param_still_defined_with_out_type() {
        let mut env = TypeEnv::new();
        env.insert("s".to_string(), PhpType::Str);
        let args = [
            Expr::string_lit("/x/"),
            Expr::var("s"),
            Expr::var("m"),
        ];
        let outputs = Checker::builtin_call_by_ref_outputs("preg_match_all", &args, &env);
        assert_eq!(
            outputs,
            vec![("m".to_string(), preg_match_all_matches_type())],
        );
    }
}
