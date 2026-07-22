//! Purpose:
//! Runs method-body validation once class and interface schemas are available.
//! Checks instance/static context, declared returns, visibility-sensitive access, and inherited method contracts.
//!
//! Called from:
//! - `crate::types::checker::driver::functions`
//!
//! Key details:
//! - Method checking depends on flattened class metadata and must preserve `self`, `parent`, and `$this` context.

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::ClassMethod;
use crate::types::{traits::FlattenedClass, FunctionSig, PhpType, TypeEnv};

use super::Checker;

impl Checker {
    /// Runs method-body validation in passes until class type information stabilizes.
    ///
    /// Each pass type-checks every non-abstract method body, collecting return types and
    /// errors. If a pass changes `self.classes` (e.g., via inferred return types), another
    /// pass runs. Iteration stops when types stabilize or `2 * class_count + 1` passes
    /// are exhausted.
    ///
    /// For non-static methods, `$this` is inserted into the per-method `TypeEnv` as an
    /// `Object` of the declaring class. Parameters are resolved against declared type hints
    /// or inferred from the class signature; variadic parameters use `PhpType::Array(Int)`
    /// as a fallback.
    ///
    /// Sets `self.current_class`, `self.current_method`, and `self.current_method_is_static`
    /// during body checking to enable context-sensitive diagnostics.
    pub(super) fn type_check_methods_until_stable(
        &mut self,
        flattened_classes: &[FlattenedClass],
        global_env: &TypeEnv,
        errors: &mut Vec<CompileError>,
    ) -> Result<(), CompileError> {
        let mut method_passes_remaining = (flattened_classes.len().max(1) * 2) + 1;
        loop {
            let classes_before_pass = self.classes.clone();
            let mut pass_errors = Vec::new();

            for class in flattened_classes {
                for method in &class.methods {
                    if method.is_abstract {
                        continue;
                    }
                    let method_key = php_symbol_key(&method.name);
                    let mut method_env: TypeEnv = global_env.clone();
                    if !method.is_static {
                        method_env.insert("this".to_string(), PhpType::Object(class.name.clone()));
                    }
                    let sig_params = if method.is_static {
                        self.classes
                            .get(&class.name)
                            .and_then(|c| c.static_methods.get(&method_key))
                            .map(|s| s.params.clone())
                    } else {
                        self.classes
                            .get(&class.name)
                            .and_then(|c| c.methods.get(&method_key))
                            .map(|s| s.params.clone())
                    };
                    // Params whose resolved type is `callable` — mirrors the free-function
                    // `callable_param_names`/`declared_callable_param_names` split in
                    // `functions::resolution::signature::resolve_function_signature`, scoped
                    // per method by the class-qualified cross-call cache key below.
                    let mut callable_param_names: Vec<String> = Vec::new();
                    let mut declared_callable_param_names: Vec<String> = Vec::new();
                    for (i, (pname, type_ann, _, _)) in method.params.iter().enumerate() {
                        let ty = if let Some(type_ann) = type_ann {
                            let declared = self.resolve_declared_param_type_hint(
                                type_ann,
                                method.span,
                                &format!("Method parameter ${}", pname),
                            )?;
                            // A generic `array` hint is sharpened to the call-site array shape
                            // recorded on the stored signature, mirroring how free-function
                            // `array` parameters are specialized (issue #406). Without this a
                            // method `array` parameter stays an integer-indexed list and rejects
                            // string-key access / mis-encodes associative arrays.
                            if Self::is_generic_array_hint(&declared) {
                                sig_params
                                    .as_ref()
                                    .and_then(|p| p.get(i))
                                    .map(|(_, t)| t.clone())
                                    .filter(|t| {
                                        matches!(
                                            t,
                                            PhpType::Array(_) | PhpType::AssocArray { .. }
                                        )
                                    })
                                    .map(|t| {
                                        Self::specialize_generic_array_param_hint(&declared, &t)
                                    })
                                    .unwrap_or(declared)
                            } else {
                                declared
                            }
                        } else {
                            sig_params
                                .as_ref()
                                .and_then(|p| p.get(i))
                                .map(|(_, t)| t.clone())
                                .unwrap_or(PhpType::Int)
                        };
                        // PHP's __unserialize($data) always receives the associative
                        // array produced by __serialize(); a bare `array` hint resolves
                        // to an indexed Array(Mixed) that rejects $data['key']. Type the
                        // first parameter as a string/int-keyed assoc array so the body
                        // can read string keys, matching the bare hash the unserialize
                        // runtime passes in (kept in sync with build_method_sig). Scoped
                        // to user methods (real span); synthetic SPL bodies keep `array`.
                        let ty = if method_key == "__unserialize" && i == 0 && method.span.line != 0 {
                            PhpType::AssocArray {
                                key: Box::new(PhpType::Mixed),
                                value: Box::new(PhpType::Mixed),
                            }
                        } else {
                            ty
                        };
                        if ty == PhpType::Callable {
                            callable_param_names.push(pname.clone());
                            if type_ann.is_some() {
                                declared_callable_param_names.push(pname.clone());
                            }
                        }
                        method_env.insert(pname.clone(), ty);
                    }
                    if let Some(variadic_name) = &method.variadic {
                        let fallback_ty = if method.variadic_by_ref {
                            PhpType::Array(Box::new(PhpType::Mixed))
                        } else {
                            PhpType::Array(Box::new(PhpType::Int))
                        };
                        let ty = sig_params
                            .as_ref()
                            .and_then(|p| p.get(method.params.len()))
                            .map(|(_, t)| t.clone())
                            .unwrap_or(fallback_ty);
                        method_env.insert(variadic_name.clone(), ty);
                    }
                    if method_key == "__construct" {
                        self.patch_constructor_method_env(class, method, &mut method_env);
                    }

                    self.current_class = Some(class.name.clone());
                    self.current_method = Some(method_key.clone());
                    self.current_method_is_static = method.is_static;
                    self.current_by_ref_return = method.by_ref_return;
                    let method_ref_params: Vec<String> = method
                        .params
                        .iter()
                        .filter(|(_, _, _, is_ref)| *is_ref)
                        .map(|(name, _, _, _)| name.clone())
                        .collect();
                    // Cross-call cache key for this method's OWN callable-typed params:
                    // the DECLARING/flattened-owner class (`class.name` here, since
                    // `flattened_classes` only lists a method under the class that
                    // physically owns its body — inherited-without-override methods are
                    // checked once, under their original declaring class) qualified with
                    // the method name, matching what call-site checking
                    // (`inference::objects::methods`) writes and what the active EIR
                    // lowering (`ir_lower::context::Context::callable_param_signature`)
                    // already reads via `owner_name = "{class}::{method}"`.
                    let method_callable_scope_key = format!("{}::{}", class.name, method_key);
                    // Start this method's body check with an EMPTY slate for every
                    // variable-name-keyed callable side table — see
                    // `Checker::enter_callable_var_scope` for why this is required (methods
                    // previously had NO such scoping at all, unlike free functions, so a
                    // closure assigned to a same-named local in one method leaked into every
                    // other method/function checked afterward).
                    let saved_callable_var_scope = self.enter_callable_var_scope();
                    for pname in &declared_callable_param_names {
                        self.callable_param_names.insert(pname.clone());
                    }
                    for pname in &callable_param_names {
                        if let Some(sig) = self
                            .callable_param_sigs
                            .get(&(method_callable_scope_key.clone(), pname.clone()))
                            .cloned()
                        {
                            self.closure_return_types
                                .insert(pname.clone(), sig.return_type.clone());
                            self.callable_sigs.insert(pname.clone(), sig);
                        }
                        // No cached signature yet: pre-specialization fallback (validated
                        // only by count/by-ref when this method's body invokes it).
                    }
                    let mut method_errors = Vec::new();
                    let body_check_result =
                        self.with_local_storage_context(method_ref_params, |checker| {
                            for s in &method.body {
                                if let Err(error) = checker.check_stmt(s, &mut method_env) {
                                    method_errors.extend(error.flatten());
                                }
                            }
                            Ok(())
                        });
                    if let Err(error) = &body_check_result {
                        // A structural error from `with_local_storage_context` itself (not a
                        // per-statement error collected into `method_errors`) still needs the
                        // callable side tables restored before propagating, same as any other
                        // early-return path.
                        let error = error.clone();
                        for pname in &callable_param_names {
                            if let Some(sig) = self.callable_sigs.get(pname).cloned() {
                                self.callable_param_sigs
                                    .insert((method_callable_scope_key.clone(), pname.clone()), sig);
                            }
                        }
                        self.exit_callable_var_scope(saved_callable_var_scope);
                        return Err(error);
                    }
                    let method_has_errors = !method_errors.is_empty();
                    pass_errors.extend(method_errors);

                    // `update_method_return_type` re-infers `return` expression types (e.g. a
                    // pipe/callable-variable invocation) via `collect_return_infos`, which reads
                    // `self.callable_sigs`/`self.closure_return_types` the SAME way the body
                    // check did — so the callable var scope must stay open through this call,
                    // not just through the body-statement loop above.
                    if !method_has_errors {
                        self.update_method_return_type(class, method, &method_env, &mut pass_errors);
                    }
                    // Persist any specialization this method's body produced for its OWN
                    // declared callable params into the cross-call cache BEFORE restoring the
                    // caller's snapshot.
                    for pname in &callable_param_names {
                        if let Some(sig) = self.callable_sigs.get(pname).cloned() {
                            self.callable_param_sigs
                                .insert((method_callable_scope_key.clone(), pname.clone()), sig);
                        }
                    }
                    self.exit_callable_var_scope(saved_callable_var_scope);
                    self.current_class = None;
                    self.current_method = None;
                    self.current_method_is_static = false;
                    self.current_by_ref_return = false;
                }
            }

            let stabilized = self.classes == classes_before_pass;
            let out_of_passes = method_passes_remaining == 0;
            if stabilized || out_of_passes {
                errors.extend(pass_errors);
                break;
            }

            method_passes_remaining -= 1;
        }
        Ok(())
    }

    /// Patches untyped constructor parameters with property types when the constructor
    /// property-promotion rule applies.
    ///
    /// For each constructor parameter without an explicit type hint, if the class has a
    /// matching promoted property (`constructor_param_to_prop`), that property's declared
    /// type is injected into `method_env` for the parameter and also propagated back into
    /// the class signature's `params[i].1`. Skips parameters that have explicit type
    /// annotations or whose promoted property is redeclared as a normal property.
    fn patch_constructor_method_env(
        &mut self,
        class: &FlattenedClass,
        method: &ClassMethod,
        method_env: &mut TypeEnv,
    ) {
        if let Some(ci) = self.classes.get(&class.name).cloned() {
            for (i, (pname, type_ann, _, _)) in method.params.iter().enumerate() {
                if type_ann.is_some() {
                    continue;
                }
                if let Some(Some(prop_name)) = ci.constructor_param_to_prop.get(i) {
                    if ci.visible_property_is_declared(prop_name) {
                        continue;
                    }
                    if let Some((_, (_, ty))) = ci.visible_property(prop_name) {
                        method_env.insert(pname.clone(), ty.clone());
                        if let Some(ci_mut) = self.classes.get_mut(&class.name) {
                            if let Some(sig) = ci_mut.methods.get_mut("__construct") {
                                if i < sig.params.len() {
                                    sig.params[i].1 = ty.clone();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Infers the return type from method body `return` statements, validates it against
    /// any declared return type hint, and writes the effective return type back into
    /// `self.classes`.
    ///
    /// Return type inference scans `method.body` for `return` statements, widens all
    /// observed types to the common supertype, and falls back to `PhpType::Void` when
    /// the body is empty. If a declared hint exists, `require_declared_return_coverage`
    /// checks for unreachable returns and `require_compatible_return_type` checks each
    /// observed return for assignability to the declared type. A `Never` declared return
    /// suppresses the compatibility check (the body is allowed to have no returns when
    /// it always throws/exits/loops). `Never` combined with a body that *does* contain
    /// return statements produces a compile error. Generic array hints are passed
    /// through as-is to preserve inference.
    ///
    /// Generator methods (bodies containing `yield`/`yield from`) short-circuit to
    /// an effective return type of `Object("Generator")`, mirroring the free-function
    /// path: the declared hint is only validated for `Generator` acceptance (via
    /// `generator_return_type_accepts`), and `require_declared_return_coverage` plus
    /// the per-`return` compatibility checks are skipped because a generator body has
    /// no `return`-on-every-path obligation.
    fn update_method_return_type(
        &mut self,
        class: &FlattenedClass,
        method: &ClassMethod,
        method_env: &TypeEnv,
        pass_errors: &mut Vec<CompileError>,
    ) {
        let mut return_infos = Vec::new();
        let mut callable_return_sigs = Vec::new();
        let mut callable_array_return_sigs = Vec::new();
        for stmt in &method.body {
            self.collect_return_infos(stmt, method_env, &mut return_infos);
            self.collect_return_callable_sigs(stmt, method_env, &mut callable_return_sigs);
            self.collect_return_callable_array_sigs(
                stmt,
                method_env,
                &mut callable_array_return_sigs,
            );
        }
        let raw_inferred = if return_infos.is_empty() {
            None
        } else {
            let mut widest = return_infos[0].ty.clone();
            for return_info in &return_infos[1..] {
                widest = Self::wider_type(&widest, &return_info.ty);
            }
            Some(widest)
        };
        let inferred_return = raw_inferred.clone().unwrap_or(PhpType::Void);
        // Generator methods: a body containing `yield`/`yield from` returns a
        // `Generator` object, NOT the value(s) named by `return`/the declared hint.
        // Mirror the function path (`functions/resolution/signature.rs`): validate the
        // declared hint accepts a `Generator`, set the effective return to `Generator`,
        // and SKIP `require_declared_return_coverage` (a generator body legitimately has
        // no `return` on every path) and the per-return compatibility checks.
        let effective_return = if crate::types::checker::yield_validation::body_contains_yield(
            &method.body,
        ) {
            let generator_ty = PhpType::Object("Generator".to_string());
            if let Some(type_ann) = method.return_type.as_ref() {
                match self.resolve_declared_return_type_hint(
                    type_ann,
                    method.span,
                    &format!("Method '{}::{}'", class.name, method.name),
                ) {
                    Ok(declared) => {
                        if !self.generator_return_type_accepts(&declared) {
                            if let Err(error) = self.require_compatible_return_type(
                                &declared,
                                &generator_ty,
                                true,
                                method.span,
                                &format!("Method '{}::{}' return type", class.name, method.name),
                            ) {
                                pass_errors.extend(error.flatten());
                                self.current_class = None;
                                self.current_method = None;
                                self.current_method_is_static = false;
                                return;
                            }
                        }
                    }
                    Err(error) => {
                        pass_errors.extend(error.flatten());
                        self.current_class = None;
                        self.current_method = None;
                        self.current_method_is_static = false;
                        return;
                    }
                }
            }
            generator_ty
        } else if let Some(type_ann) = method.return_type.as_ref() {
            match self.resolve_declared_return_type_hint(
                type_ann,
                method.span,
                &format!("Method '{}::{}'", class.name, method.name),
            ) {
                Ok(declared) => {
                    if matches!(declared, PhpType::Never)
                        && Self::body_contains_return(&method.body)
                    {
                        pass_errors.push(CompileError::new(
                            method.span,
                            &format!(
                                "Method '{}::{}' declared never must not return",
                                class.name, method.name
                            ),
                        ));
                        self.current_class = None;
                        self.current_method = None;
                        self.current_method_is_static = false;
                        return;
                    }
                    if let Err(error) = self.require_declared_return_coverage(
                        &declared,
                        &method.body,
                        method.span,
                        &format!("Method '{}::{}'", class.name, method.name),
                    ) {
                        pass_errors.extend(error.flatten());
                        self.current_class = None;
                        self.current_method = None;
                        self.current_method_is_static = false;
                        return;
                    }
                    // :never methods are allowed to have no return statements (they always throw/exit/loop).
                    let skip_compat_check = matches!(declared, PhpType::Never);
                    if !skip_compat_check {
                        for return_info in &return_infos {
                            if let Err(error) = self.require_compatible_return_type(
                                &declared,
                                &return_info.ty,
                                return_info.has_value,
                                method.span,
                                &format!("Method '{}::{}' return type", class.name, method.name),
                            ) {
                                pass_errors.extend(error.flatten());
                                self.current_class = None;
                                self.current_method = None;
                                self.current_method_is_static = false;
                                return;
                            }
                        }
                    }
                    if Self::is_generic_array_hint(&declared)
                        && matches!(inferred_return, PhpType::Array(_) | PhpType::AssocArray { .. })
                    {
                        inferred_return
                    } else {
                        declared
                    }
                }
                Err(error) => {
                    pass_errors.extend(error.flatten());
                    self.current_class = None;
                    self.current_method = None;
                    self.current_method_is_static = false;
                    return;
                }
            }
        } else {
            inferred_return
        };
        if !method.is_static {
            if let Some(ci) = self.classes.get_mut(&class.name) {
                if let Some(sig) = ci.methods.get_mut(&php_symbol_key(&method.name)) {
                    sig.return_type = effective_return.clone();
                }
            }
        } else if let Some(ci) = self.classes.get_mut(&class.name) {
            if let Some(sig) = ci.static_methods.get_mut(&php_symbol_key(&method.name)) {
                sig.return_type = effective_return.clone();
            }
        }
        self.update_method_callable_return_metadata(
            &class.name,
            &php_symbol_key(&method.name),
            &effective_return,
            &callable_return_sigs,
            &callable_array_return_sigs,
        );
    }

    /// Updates callable-return metadata for one checked method body.
    fn update_method_callable_return_metadata(
        &mut self,
        class_name: &str,
        method_key: &str,
        return_type: &PhpType,
        callable_return_sigs: &[FunctionSig],
        callable_array_return_sigs: &[FunctionSig],
    ) {
        let Some(class_info) = self.classes.get_mut(class_name) else {
            return;
        };
        if return_type == &PhpType::Callable {
            if let Some(callable_sig) = matching_callable_sig(callable_return_sigs) {
                class_info
                    .callable_method_return_sigs
                    .insert(method_key.to_string(), callable_sig);
            } else {
                class_info.callable_method_return_sigs.remove(method_key);
            }
        } else {
            class_info.callable_method_return_sigs.remove(method_key);
        }
        if is_callable_array_return_type(return_type) {
            if let Some(callable_sig) = matching_callable_sig(callable_array_return_sigs) {
                class_info
                    .callable_array_method_return_sigs
                    .insert(method_key.to_string(), callable_sig);
            } else {
                class_info
                    .callable_array_method_return_sigs
                    .remove(method_key);
            }
        } else {
            class_info
                .callable_array_method_return_sigs
                .remove(method_key);
        }
    }
}

/// Returns true when a method return type is a homogeneous array of callables.
fn is_callable_array_return_type(return_type: &PhpType) -> bool {
    match return_type {
        PhpType::Array(elem_ty) => elem_ty.as_ref() == &PhpType::Callable,
        PhpType::AssocArray { value, .. } => value.as_ref() == &PhpType::Callable,
        _ => false,
    }
}

/// Returns one callable signature only when every return path has the same contract.
fn matching_callable_sig(return_sigs: &[FunctionSig]) -> Option<FunctionSig> {
    let first = return_sigs.first()?.clone();
    if return_sigs.iter().all(|sig| sig == &first) {
        Some(callable_return_codegen_sig(first))
    } else {
        None
    }
}

/// Normalizes untyped mixed parameters in callable-return metadata for codegen.
fn callable_return_codegen_sig(mut sig: FunctionSig) -> FunctionSig {
    for (idx, (_, ty)) in sig.params.iter_mut().enumerate() {
        if !sig.declared_params.get(idx).copied().unwrap_or(false)
            && matches!(ty, PhpType::Mixed)
        {
            *ty = PhpType::Int;
        }
    }
    sig
}
