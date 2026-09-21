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

/// How many previously produced class tables the method-pass fixpoint remembers.
///
/// Long enough to catch the oscillations that actually occur (the one measured on Symfony has
/// period two), short enough that the memory is a handful of tables rather than a transcript of
/// the whole run. An oscillation longer than this still terminates on the pass budget, exactly as
/// it did before.
const PASS_CYCLE_WINDOW: usize = 4;

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
    /// or inferred from the class signature. Untyped parameters without an observed call-site
    /// specialization are checked as gradual `Mixed`; variadic parameters use
    /// `PhpType::Array(Int)` as a fallback.
    ///
    /// Sets `self.current_class`, `self.current_method`, and `self.current_method_is_static`
    /// during body checking to enable context-sensitive diagnostics.
    pub(super) fn type_check_methods_until_stable(
        &mut self,
        flattened_classes: &[FlattenedClass],
        errors: &mut Vec<CompileError>,
    ) -> Result<(), CompileError> {
        // This loop is 32.61s of the 33.30s type-checking phase, and it is three things per
        // pass: a deep CLONE of the whole class table, the body checking, and up to five deep
        // COMPARISONS of that table (one for stability, four against the cycle window).
        // `ELEPHC_TYPECHECK_TIMES=1` says which, and how many passes it takes to get there.
        let trace = std::env::var("ELEPHC_TYPECHECK_TIMES").is_ok();
        let mut passes = 0usize;
        let (mut clone_secs, mut bodies_secs, mut compare_secs) = (0f64, 0f64, 0f64);
        let mut method_passes_remaining = (flattened_classes.len().max(1) * 2) + 1;
        // Tables this loop has already produced, newest last. A pass that repeats one has added
        // nothing and never will — the next table is a function of this one alone, so the
        // sequence is periodic from here. See `PASS_CYCLE_WINDOW`.
        let mut recent_tables = std::collections::VecDeque::with_capacity(PASS_CYCLE_WINDOW);
        loop {
            passes += 1;
            let mut mark = std::time::Instant::now();
            let classes_before_pass = self.classes.clone();
            if trace {
                clone_secs += mark.elapsed().as_secs_f64();
                mark = std::time::Instant::now();
            }
            let mut pass_errors = Vec::new();
            // Only the LAST round's answer counts: a seed that looked unrefined on an early round
            // can be specialized by a call site the same round walks later.
            self.unspecialized_seed_params.clear();

            for class in flattened_classes {
                for method in &class.methods {
                    if method.is_abstract {
                        continue;
                    }
                    let method_key = php_symbol_key(&method.name);
                    let mut method_env = Self::seed_method_env();
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
                    for (i, (pname, type_ann, _, _)) in method.params.iter().enumerate() {
                        let ty = if let Some(type_ann) = type_ann {
                            let declared = self.resolve_declared_param_type_hint(
                                type_ann,
                                method.span,
                                &format!("Method parameter ${}", pname),
                            )?;
                            // A declared bare `array` remains generic. Call boundaries normalize
                            // concrete indexed/hash payloads while the method body dispatches on
                            // the runtime array kind, so one call site cannot narrow the contract.
                            declared
                        } else {
                            let inferred = sig_params
                                .as_ref()
                                .and_then(|p| p.get(i))
                                .map(|(_, t)| t.clone())
                                .unwrap_or(PhpType::Int);
                            let was_seed = inferred == PhpType::Int;
                            let body_ty = self.method_body_param_type(class, method, i, inferred);
                            // The body is checked with `mixed` for a seed no call site refined,
                            // and the SIGNATURE has to say so too or the backend lowers the
                            // parameter as the raw `int` the seed left behind. Recorded here and
                            // published once the pass loop settles -- on any earlier round the
                            // call site that refines it may simply not have been walked yet.
                            if was_seed && body_ty == PhpType::Mixed {
                                self.unspecialized_seed_params.insert((
                                    class.name.clone(),
                                    method_key.clone(),
                                    method.is_static,
                                    i,
                                ));
                            }
                            body_ty
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
                        // A by-reference parameter this body widens is bound `mixed` for the whole
                        // body — see `Checker::widened_ref_param_env_type`. Only the ENVIRONMENT
                        // moves; `self.classes`' signature keeps the declared type, so every call
                        // site still validates its argument against it.
                        let ty = self.widened_ref_param_env_type(
                            &format!("{}::{}", class.name, method.name),
                            pname,
                            &ty,
                        );
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
                    // A local a body CREATES by writing an array element into it —
                    // `$keys[] = $k` against a name nothing has assigned — exists from that
                    // write onwards, because PHP auto-vivifies the array. Every other scope
                    // seeds those names at entry (`resolve_function_signature` for a function,
                    // `infer_closure_type_with_param_hints` for a closure,
                    // `check_top_level_program` for file scope) and the method pass did not, so
                    // a read after the loop that fills the array was "Undefined variable" here
                    // and lowered fine in the backend, which runs the same scan for every body.
                    // Seeded last: the scan treats everything already in the environment as
                    // bound, so the parameters, the variadic and the promoted constructor
                    // properties must be in place first.
                    Self::seed_vivified_array_locals(&mut method_env, &method.body);

                    self.current_class = Some(class.name.clone());
                    self.current_method = Some(method_key.clone());
                    self.current_method_is_static = method.is_static;
                    self.current_by_ref_return = method.by_ref_return;
                    let loop_storage_scope = format!("{}::{}", class.name, method.name);
                    self.loop_storage_types
                        .retain(|(scope, _), _| scope != &loop_storage_scope);
                    let previous_loop_storage_scope = std::mem::replace(
                        &mut self.current_loop_storage_scope,
                        loop_storage_scope,
                    );
                    let method_ref_params: Vec<String> = method
                        .params
                        .iter()
                        .filter(|(_, _, _, is_ref)| *is_ref)
                        .map(|(name, _, _, _)| name.clone())
                        .collect();
                    // Every parameter is bound unconditionally on entry, so all of them are
                    // recorded at binding depth 0 (a missing entry means "seeded, not bound
                    // here", which is not kill/retype eligible).
                    let method_param_names: Vec<String> = method
                        .params
                        .iter()
                        .map(|(name, _, _, _)| name.clone())
                        .chain(method.variadic.iter().cloned())
                        .collect();
                    // A parameter with a declared type hint is a contract: never kill/retype
                    // eligible inside the body, in either mode.
                    let method_typed_params: Vec<String> = method
                        .params
                        .iter()
                        .filter(|(_, type_ann, _, _)| type_ann.is_some())
                        .map(|(name, _, _, _)| name.clone())
                        .chain(
                            method
                                .variadic
                                .iter()
                                .filter(|_| method.variadic_type.is_some())
                                .cloned(),
                        )
                        .collect();
                    let mut method_errors = Vec::new();
                    // The storage this frame already holds on entry: the parameters. `$this`,
                    // the superglobals and the seeded globals `method_env` also carries are not
                    // this frame's own storage, and none of them is markable anyway.
                    let pre_bound_own_storage: std::collections::HashMap<String, PhpType> =
                        method_param_names
                            .iter()
                            .filter_map(|name| {
                                method_env.get(name).map(|ty| (name.clone(), ty.clone()))
                            })
                            .collect();
                    // A parameter DECLARED `callable` carries no signature anyone can check
                    // against: its real one arrives with the argument. `resolve_function_signature`
                    // records such parameters so a spread call on one is not refused for the
                    // by-reference parameters of whatever signature inference happened to guess —
                    // and it only ever did it for plain FUNCTIONS, so the same parameter in a
                    // METHOD lost the allowance. Symfony's
                    // `PhpFileLoader::callConfigurator(callable $callback, ...)` is exactly that
                    // shape: `$callback(...$arguments)` was refused inside the method and accepted
                    // outside one.
                    let saved_callable_param_names = self.callable_param_names.clone();
                    for (pname, type_ann, _, _) in &method.params {
                        if type_ann.is_some()
                            && method_env.get(pname) == Some(&PhpType::Callable)
                        {
                            self.callable_param_names.insert(pname.clone());
                        }
                    }
                    let body_result = self.with_local_storage_context(
                        method_ref_params,
                        method_param_names,
                        method_typed_params,
                        pre_bound_own_storage,
                        &method.body,
                        |checker| {
                            for s in &method.body {
                                if let Err(error) = checker.check_stmt(s, &mut method_env) {
                                    // Naming the enclosing CLASS is what lets `pipeline` recover
                                    // the FILE: after autoload expansion every spliced file
                                    // shares one line-number space, so `line:col` alone
                                    // identifies nothing. `resolve_function_signature` already
                                    // does this for a plain function; a method body reached the
                                    // reporter untagged, which is why a Symfony preload error
                                    // printed as a bare `error[213:13]`.
                                    method_errors.extend(
                                        error.within_declaration(class.name.as_str()).flatten(),
                                    );
                                }
                            }
                            Ok(())
                        },
                    );
                    self.callable_param_names = saved_callable_param_names;
                    body_result?;
                    if std::env::var("ELEPHC_BACKEND_INVENTORY").as_deref() == Ok("1") {
                        for error in &mut method_errors {
                            error.message = format!(
                                "{}::{}: {}",
                                class.name, method.name, error.message
                            );
                        }
                    }
                    let method_has_errors = !method_errors.is_empty();
                    pass_errors.extend(method_errors);

                    if !method_has_errors {
                        self.update_method_return_type(class, method, &method_env, &mut pass_errors);
                    }
                    self.current_class = None;
                    self.current_method = None;
                    self.current_method_is_static = false;
                    self.current_by_ref_return = false;
                    self.current_loop_storage_scope = previous_loop_storage_scope;
                }
            }

            if trace {
                bodies_secs += mark.elapsed().as_secs_f64();
                mark = std::time::Instant::now();
            }
            let stabilized = self.classes == classes_before_pass;
            if trace && !stabilized {
                // Which signatures actually moved this pass. The loop exits on CYCLE detection,
                // not convergence, so the last passes are re-checking 1000+ classes to chase
                // whatever this names -- and that is the thing worth fixing, not the loop.
                let mut moved: Vec<String> = Vec::new();
                for (class_name, after) in &self.classes {
                    let Some(before) = classes_before_pass.get(class_name) else {
                        moved.push(format!("{class_name}(new)"));
                        continue;
                    };
                    for (method, signature) in &after.methods {
                        if before.methods.get(method) != Some(signature) {
                            moved.push(format!("{class_name}::{method}"));
                        }
                    }
                    for (method, signature) in &after.static_methods {
                        if before.static_methods.get(method) != Some(signature) {
                            moved.push(format!("{class_name}::{method}(static)"));
                        }
                    }
                    if before.callable_method_return_sigs != after.callable_method_return_sigs {
                        moved.push(format!("{class_name}(callable_return_sigs)"));
                    }
                    if before != after && moved.last().map(|last| !last.starts_with(class_name.as_str())).unwrap_or(true) {
                        // A field OUTSIDE the three a pass writes here has moved, which means
                        // something deeper in the checker mutates `ClassInfo` too. Name it by
                        // diffing the debug renderings rather than enumerating forty fields:
                        // this runs only for the handful of classes that are still moving.
                        let (before_text, after_text) =
                            (format!("{before:?}"), format!("{after:?}"));
                        let at = before_text
                            .bytes()
                            .zip(after_text.bytes())
                            .position(|(left, right)| left != right)
                            .unwrap_or(before_text.len().min(after_text.len()));
                        let from = at.saturating_sub(70);
                        moved.push(format!(
                            "{class_name}(other @{at}: {:?} -> {:?})",
                            &before_text[from..before_text.len().min(at + 40)],
                            &after_text[from..after_text.len().min(at + 40)],
                        ));
                    }
                }
                moved.sort();
                eprintln!(
                    "[elephc-methodpass] pass={passes} moved={} first={:?}",
                    moved.len(),
                    &moved[..moved.len().min(12)]
                );
            }
            // A REPEAT is not progress. Symfony's 246-file preload is quiet for every class but
            // one after three passes and then oscillates forever on a single property, which used
            // to burn the whole budget — 1630 passes at ~2s each — to land on a table pass three
            // had already produced. Stopping here ends on a state the loop reached by itself.
            let cycling = !stabilized
                && recent_tables
                    .iter()
                    .any(|table| table == &self.classes);
            let out_of_passes = method_passes_remaining == 0;
            if trace {
                compare_secs += mark.elapsed().as_secs_f64();
            }
            if stabilized || cycling || out_of_passes {
                errors.extend(pass_errors);
                if trace {
                    eprintln!(
                        "[elephc-methodpass] passes={passes} clone={clone_secs:.2}s \
                         bodies={bodies_secs:.2}s compare={compare_secs:.2}s \
                         classes={} stabilized={stabilized} cycling={cycling}",
                        self.classes.len()
                    );
                }
                break;
            }

            while recent_tables.len() >= PASS_CYCLE_WINDOW {
                recent_tables.pop_front();
            }
            // The clone the stabilization check already made, reused rather than repeated.
            recent_tables.push_back(classes_before_pass);
            method_passes_remaining -= 1;
        }
        Ok(())
    }

    /// Returns the body-checking type for an untyped method parameter.
    ///
    /// Class schemas retain `Int` as the unspecialized legacy sentinel. A real call records the
    /// parameter in `param_specialization_seen`, including the homogeneous integer case where the
    /// stored type remains `Int`. Without that evidence PHP's untyped parameter is gradual
    /// `Mixed`; using the sentinel in its body creates false diagnostics and disagrees with the
    /// boxed PHP parameter contract used by EIR lowering.
    fn method_body_param_type(
        &self,
        class: &FlattenedClass,
        method: &ClassMethod,
        index: usize,
        inferred: PhpType,
    ) -> PhpType {
        if inferred != PhpType::Int {
            return inferred;
        }
        let owner = if method.is_static {
            format!("static:{}::{}", class.name, method.name)
        } else {
            format!("{}::{}", class.name, php_symbol_key(&method.name))
        };
        if self.param_specialization_seen.contains(&(owner, index)) {
            inferred
        } else {
            PhpType::Mixed
        }
    }


    /// Publishes `mixed` for every undeclared parameter whose `int` inference seed no call site
    /// refined, so the signature says what the body was checked with.
    ///
    /// `method_body_param_type` already answers `mixed` for such a parameter -- PHP reads an
    /// undeclared parameter as `mixed` -- but it only moved the ENVIRONMENT. The signature kept
    /// the seed, and the signature is what EIR lowers the parameter as, so the body reasoned about
    /// a boxed value while the backend read a raw integer register. `twig/twig`'s
    /// `CoreExtension::filter($env, $isSandboxed, $array, $arrow)` has no compiled caller: its
    /// `$array` reached `new \IteratorIterator($array)` as `Int` and its `$arrow` reached
    /// `new \CallbackFilterIterator(..., $arrow)` as `Int` against a `callable` property, on code
    /// php runs. The free-function half of this rule is
    /// `widen_unrefined_params_of_an_uncalled_function`.
    pub(super) fn publish_unspecialized_method_param_seeds(&mut self) {
        let slots: Vec<(String, String, bool, usize)> =
            self.unspecialized_seed_params.iter().cloned().collect();
        for (class_name, method_key, is_static, index) in slots {
            let Some(class_info) = self.classes.get_mut(&class_name) else {
                continue;
            };
            let table = if is_static {
                &mut class_info.static_methods
            } else {
                &mut class_info.methods
            };
            let Some(sig) = table.get_mut(&method_key) else {
                continue;
            };
            let Some(param) = sig.params.get_mut(index) else {
                continue;
            };
            // Only the seed itself: a later round may have refined it after all.
            if param.1 == PhpType::Int {
                param.1 = PhpType::Mixed;
            }
        }
    }

    /// Builds the PHP-local base environment shared by all method bodies.
    ///
    /// Methods can read request superglobals without a `global` declaration, but
    /// ordinary top-level locals belong to file scope and must not leak into a
    /// method. Explicit `global` statements resolve through `self.top_level_env`.
    fn seed_method_env() -> TypeEnv {
        crate::superglobals::SUPERGLOBALS
            .iter()
            .map(|name| ((*name).to_string(), crate::superglobals::superglobal_type()))
            .collect()
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
    /// A method body containing `yield` is a generator: calling it produces a
    /// `Generator` object regardless of what the body's `return` statements say, so
    /// generator detection short-circuits the whole inference/validation chain the
    /// same way the free-function path in `functions::resolution::signature` does.
    /// Without that short-circuit an unhinted generator method infers `void` (the
    /// body has no value return) and a `: Generator` hint trips the
    /// "must return a value on every path" coverage check.
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
        let effective_return = if crate::types::checker::yield_validation::body_contains_yield(
            &method.body,
        ) {
            match self.generator_method_return_type(class, method) {
                Ok(generator_ty) => generator_ty,
                Err(error) => {
                    pass_errors.extend(error.flatten());
                    self.current_class = None;
                    self.current_method = None;
                    self.current_method_is_static = false;
                    return;
                }
            }
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
                                return_info.strict_types,
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
                    if Self::is_generic_array_hint(&declared) {
                        // Resolved from the individual returns, not from `inferred_return`:
                        // `wider_type` collapses an indexed return joined with a hash one to
                        // `Mixed`, which this guard used to reject, leaving the declared INDEXED
                        // hint to describe a container that is a hash at runtime.
                        Self::generic_array_return_contract(&return_infos)
                        .unwrap_or(declared)
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

    /// Resolves the return type of a method whose body contains `yield`.
    ///
    /// The result is always `Generator`, because that is the object PHP hands back when
    /// the generator method is called. A declared return hint is still resolved and
    /// validated: hints that accept a `Generator` (`Generator`, `Traversable`,
    /// `iterable`, `mixed`, …) pass through, anything else is reported as an
    /// incompatible return type. Unlike the non-generator path there is no
    /// return-coverage check — a generator body legitimately has no `return` at all.
    fn generator_method_return_type(
        &mut self,
        class: &FlattenedClass,
        method: &ClassMethod,
    ) -> Result<PhpType, CompileError> {
        let generator_ty = PhpType::Object("Generator".to_string());
        if let Some(type_ann) = method.return_type.as_ref() {
            let declared = self.resolve_declared_return_type_hint(
                type_ann,
                method.span,
                &format!("Method '{}::{}'", class.name, method.name),
            )?;
            if !self.generator_return_type_accepts(&declared) {
                self.require_compatible_return_type(
                    &declared,
                    &generator_ty,
                    true,
                    true,
                    method.span,
                    &format!("Method '{}::{}' return type", class.name, method.name),
                )?;
            }
        }
        Ok(generator_ty)
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
