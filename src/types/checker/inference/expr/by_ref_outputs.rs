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
