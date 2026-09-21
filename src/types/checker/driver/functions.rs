//! Purpose:
//! Implements the checker driver functions phase.
//! Owns one ordered step in building checker state and validating the program before optimization/codegen.
//!
//! Called from:
//! - `crate::types::checker::driver::check_types_impl()`
//!
//! Key details:
//! - Phase order controls diagnostics, available declarations, required libraries, and function-local environments.

use std::collections::{HashMap, HashSet};

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{Expr, Program, StmtKind, TypeExpr};
use crate::source::SourceMode;
use crate::types::FunctionSig;

use super::super::{Checker, FnDecl};

impl Checker {
    /// Collects top-level function declarations from the program, deduplicating by PHP case-insensitive
    /// symbol key. Emits `DuplicateFunction` for repeats and `CannotRedeclareBuiltin` when a user
    /// function shadows a built-in. Stores `FnDecl` records in `fn_decls` and variant groups in
    /// `function_variant_groups`.
    pub(super) fn collect_function_decls(
        &mut self,
        program: &Program,
        errors: &mut Vec<CompileError>,
    ) {
        let mut seen_functions = HashSet::new();
        for stmt in program {
            if let StmtKind::FunctionVariantGroup { name, variants } = &stmt.kind {
                if !seen_functions.insert(php_symbol_key(name)) {
                    errors.push(CompileError::new(
                        stmt.span,
                        &format!("Duplicate function declaration: {}", name),
                    ));
                    continue;
                }
                let builtin = crate::strict_php::with_source_mode(stmt.source_mode, || {
                    crate::types::checker::builtins::canonical_builtin_function_name(name)
                        .filter(|builtin| {
                            crate::types::checker::builtins::catalog::builtin_is_available_for_target(
                                builtin,
                                self.target,
                            )
                        })
                });
                if let Some(builtin) = builtin.filter(|_| stmt.source_mode != SourceMode::Internal) {
                    errors.push(CompileError::new(
                        stmt.span,
                        &format!("Cannot redeclare built-in function: {}", builtin),
                    ));
                    continue;
                }
                self.function_variant_groups
                    .insert(name.clone(), variants.clone());
                continue;
            }
            if let StmtKind::FunctionDecl {
                name,
                params,
                param_attributes,
                variadic,
                variadic_by_ref,
                variadic_type,
                return_type,
                by_ref_return,
                body,
                ..
            } = &stmt.kind
            {
                if !seen_functions.insert(php_symbol_key(name)) {
                    errors.push(CompileError::new(
                        stmt.span,
                        &format!("Duplicate function declaration: {}", name),
                    ));
                    continue;
                }
                let builtin = crate::strict_php::with_source_mode(stmt.source_mode, || {
                    crate::types::checker::builtins::canonical_builtin_function_name(name)
                        .filter(|builtin| {
                            crate::types::checker::builtins::catalog::builtin_is_available_for_target(
                                builtin,
                                self.target,
                            )
                        })
                });
                if let Some(builtin) = builtin.filter(|_| stmt.source_mode != SourceMode::Internal) {
                    errors.push(CompileError::new(
                        stmt.span,
                        &format!("Cannot redeclare built-in function: {}", builtin),
                    ));
                    continue;
                }
                let param_names: Vec<String> =
                    params.iter().map(|(n, _, _, _)| n.clone()).collect();
                let param_type_anns: Vec<Option<TypeExpr>> =
                    params.iter().map(|(_, t, _, _)| t.clone()).collect();
                let defaults: Vec<Option<Expr>> =
                    params.iter().map(|(_, _, d, _)| d.clone()).collect();
                let mut ref_flags: Vec<bool> = params.iter().map(|(_, _, _, r)| *r).collect();
                if variadic.is_some() {
                    ref_flags.push(*variadic_by_ref);
                }
                self.fn_decls.insert(
                    name.clone(),
                    FnDecl {
                        params: param_names,
                        param_types: param_type_anns,
                        param_attributes: param_attributes.clone(),
                        defaults,
                        ref_params: ref_flags,
                        variadic: variadic.clone(),
                        variadic_by_ref: *variadic_by_ref,
                        variadic_type: variadic_type.clone(),
                        return_type: return_type.clone(),
                        by_ref_return: *by_ref_return,
                        span: stmt.span,
                        body: body.clone(),
                        attributes: stmt.attributes.clone(),
                    },
                );
            }
        }
    }

    /// Returns true if `name` resolves to any declared function: user declaration, variant group, or
    /// extern. Resolution is case-insensitive via PHP symbol key matching.
    pub(crate) fn has_function_decl_folded(&self, name: &str) -> bool {
        // `php_symbol_key` is the ASCII fold, so this is the same predicate with nothing
        // allocated. It scans THREE whole tables and is asked per function reference, so the
        // old form allocated a lowercased `String` per declared function per reference.
        self.fn_decls
            .keys()
            .any(|existing| existing.eq_ignore_ascii_case(name))
            || self
                .function_variant_groups
                .keys()
                .any(|existing| existing.eq_ignore_ascii_case(name))
            || self
                .extern_functions
                .keys()
                .any(|existing| existing.eq_ignore_ascii_case(name))
    }

    /// Returns the canonical (case-matching) name of a user function identified by `name` by
    /// case-insensitive lookup in `functions`, `function_variant_groups`, and `fn_decls`. Returns
    /// `None` if no matching function exists.
    pub(crate) fn canonical_function_name_folded(&self, name: &str) -> Option<String> {
        folded_map_key(&self.functions, name)
            .or_else(|| folded_map_key(&self.function_variant_groups, name))
            .or_else(|| folded_map_key(&self.fn_decls, name))
    }

    /// Returns the canonical (case-matching) name of an extern function identified by `name` by
    /// case-insensitive lookup in `extern_functions`. Returns `None` if no matching extern exists.
    pub(crate) fn canonical_extern_function_name_folded(&self, name: &str) -> Option<String> {
        folded_map_key(&self.extern_functions, name)
    }

    /// Resolves type signatures for all user functions that were not already resolved during the
    /// initial pass. Iterates `fn_decls`, calls `initial_function_param_types` then
    /// `resolve_function_signature` for each unchecked function, and finally resolves all variant
    /// groups via `resolve_function_variant_groups`. Appends errors to `errors`.
    pub(super) fn resolve_unchecked_functions(&mut self, errors: &mut Vec<CompileError>) {
        let unchecked: Vec<String> = self
            .fn_decls
            .keys()
            .filter(|name| !self.functions.contains_key(*name))
            .cloned()
            .collect();
        for name in unchecked {
            if let Some(decl) = self.fn_decls.get(&name).cloned() {
                match self.initial_function_param_types(&name, &decl) {
                    Ok(mut param_types) => {
                        Self::widen_unrefined_params_of_an_uncalled_function(
                            &decl,
                            &mut param_types,
                        );
                        if let Err(error) =
                            self.resolve_function_signature(&name, &decl, param_types)
                        {
                            errors.extend(error.flatten());
                        }
                    }
                    Err(error) => errors.extend(error.flatten()),
                }
            }
        }
        self.resolve_function_variant_groups(errors);
    }

    /// Makes an UNCALLED function's undeclared parameters `mixed` instead of the `Int` seed.
    ///
    /// `initial_function_param_types` seeds an undeclared parameter with `PhpType::Int` as a
    /// gradual starting point that the FIRST call site then discards and replaces with the real
    /// argument type. This path is the one where that call site never comes — the function is
    /// resolved precisely because nothing calls it — so the seed is not a starting point, it is
    /// the final answer, and it is a fabrication: PHP reads an undeclared parameter as `mixed`.
    ///
    /// Left alone it does not stay contained either, because the body is still checked and every
    /// call it makes carries the fabricated `int` outward. Twig's deprecated
    /// `twig_array_filter(Environment $env, $array, $arrow)` is called by nothing, and passing
    /// its two `int` locals to `CoreExtension::filter()` locked that method's `$array` and
    /// `$arrow` to `int` for the whole build — so `$arrow($v, $k)` inside it read as "Cannot call
    /// $arrow — not a callable (got Int)" and `new \IteratorIterator($array)` as "expects
    /// Object(Traversable), got Int", on code PHP runs.
    ///
    /// The methods equivalent already behaves this way: `method_body_param_type` answers `mixed`
    /// for an undeclared parameter that no call site has specialized.
    fn widen_unrefined_params_of_an_uncalled_function(
        decl: &crate::types::checker::FnDecl,
        param_types: &mut [(String, crate::types::PhpType)],
    ) {
        for (idx, (_, param_ty)) in param_types.iter_mut().enumerate() {
            let declared = decl.param_types.get(idx).and_then(|ty| ty.as_ref()).is_some();
            let has_default = decl.defaults.get(idx).and_then(|d| d.as_ref()).is_some();
            let by_ref = decl.ref_params.get(idx).copied().unwrap_or(false);
            // Only the seed itself is replaced: a declared `int`, a default, or a by-reference
            // parameter each carry a real fact and keep it.
            if !declared && !has_default && !by_ref && *param_ty == crate::types::PhpType::Int {
                *param_ty = crate::types::PhpType::Mixed;
            }
        }
    }

    /// Iterates all variant groups that are not yet in `functions` and calls
    /// `ensure_function_variant_group_signature` to compute and insert their unified signatures.
    /// Appends errors to `errors`.
    fn resolve_function_variant_groups(&mut self, errors: &mut Vec<CompileError>) {
        let names: Vec<String> = self.function_variant_groups.keys().cloned().collect();
        for name in names {
            if self.functions.contains_key(&name) {
                continue;
            }
            if let Err(error) =
                self.ensure_function_variant_group_signature(&name, crate::span::Span::dummy())
            {
                errors.extend(error.flatten());
            }
        }
    }

    /// Ensures a unified signature exists for a variant group named `name`. If no unified signature
    /// is cached yet, computes a provisional signature from the first variant and inserts it into
    /// `functions`. Then resolves each individual variant's signature and verifies all variants
    /// share an identical signature. On mismatch, returns an error; on success, inserts the unified
    /// signature and returns `Ok`.
    pub(crate) fn ensure_function_variant_group_signature(
        &mut self,
        name: &str,
        span: crate::span::Span,
    ) -> Result<(), CompileError> {
        if self.functions.contains_key(name) {
            return Ok(());
        }
        let variants = self
            .function_variant_groups
            .get(name)
            .cloned()
            .ok_or_else(|| CompileError::new(span, &format!("Undefined function: {}", name)))?;
        let first_variant = variants
            .first()
            .ok_or_else(|| CompileError::new(span, &format!("Function '{}' has no variants", name)))?
            .clone();

        if let Some(provisional) = self.provisional_variant_group_sig(&first_variant)? {
            self.functions.insert(name.to_string(), provisional);
        }

        for variant in &variants {
            if self.functions.contains_key(variant) {
                continue;
            }
            let decl = self.fn_decls.get(variant).cloned().ok_or_else(|| {
                CompileError::new(
                    span,
                    &format!(
                        "Compiler error: function variant '{}' for '{}' has no declaration",
                        variant, name
                    ),
                )
            })?;
            let param_types = self.initial_function_param_types(variant, &decl)?;
            self.resolve_function_signature(variant, &decl, param_types)?;
        }

        let mut sigs = variants.iter().map(|variant| {
            self.functions.get(variant).cloned().ok_or_else(|| {
                CompileError::new(
                    span,
                    &format!(
                        "Compiler error: function variant '{}' for '{}' has no signature",
                        variant, name
                    ),
                )
            })
        });
        let first = sigs
            .next()
            .transpose()?
            .ok_or_else(|| CompileError::new(span, &format!("Function '{}' has no variants", name)))?;
        for sig in sigs {
            let sig = sig?;
            if sig != first {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "Function variants for '{}' must have identical signatures",
                        name
                    ),
                ));
            }
        }
        self.functions.insert(name.to_string(), first);
        Ok(())
    }

    /// Builds a provisional `FunctionSig` for the first variant in a group using its declaration and
    /// initial param types. Used as a placeholder when a unified variant-group signature is needed
    /// before individual variants are fully resolved. Returns `Ok(None)` if no declaration exists
    /// for the variant.
    fn provisional_variant_group_sig(
        &mut self,
        first_variant: &str,
    ) -> Result<Option<FunctionSig>, CompileError> {
        let Some(decl) = self.fn_decls.get(first_variant).cloned() else {
            return Ok(None);
        };
        let param_types = self.initial_function_param_types(first_variant, &decl)?;
        Ok(Some(FunctionSig {
            params: param_types,
            param_type_exprs: decl
                .param_types
                .iter()
                .cloned()
                .chain(decl.variadic.iter().map(|_| decl.variadic_type.clone()))
                .collect(),
            param_attributes: decl.param_attributes.clone(),
            defaults: decl.defaults,
            return_type: crate::types::PhpType::Int,
            declared_return: decl.return_type.is_some(),
            by_ref_return: false,
            ref_params: decl.ref_params,
            declared_params: decl
                .param_types
                .iter()
                .map(|type_ann| type_ann.is_some())
                .chain(decl.variadic.iter().map(|_| decl.variadic_type.is_some()))
                .collect(),
            variadic: decl.variadic,
            deprecation: None,
        }))
    }
}

/// Performs a case-insensitive PHP symbol key lookup on `map` and returns the canonical (case-
/// matching) key if one exists. Used to translate case-insensitive names to their actual declared
/// spelling.
fn folded_map_key<T>(map: &HashMap<String, T>, name: &str) -> Option<String> {
    map.keys()
        .find(|existing| existing.eq_ignore_ascii_case(name))
        .cloned()
}
