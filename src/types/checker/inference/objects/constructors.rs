//! Purpose:
//! Infers object constructors expression types.
//! Validates class, method, constructor, property, and magic-access contracts against schema metadata.
//!
//! Called from:
//! - `crate::types::checker::inference::objects`
//!
//! Key details:
//! - Object inference depends on flattened class metadata, visibility, inheritance, and declared property types.

use crate::errors::CompileError;
use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{CallableTarget, Expr, ExprKind, StaticReceiver};
use crate::types::{fibers, FunctionSig, PhpType, TypeEnv};

use super::super::super::Checker;

impl Checker {
    /// Infers the type of a `new Class(...)` expression.
    ///
    /// Errors on enums, interfaces, abstract classes, or undefined classes.
    /// Validates constructor visibility, normalizes named arguments, checks
    /// the callable signature, and propagates argument types to constructor
    /// properties via `propagate_constructor_arg_type`. Special-cases `Fiber`
    /// (calls `validate_fiber_constructor_args`) and reflection owners
    /// (`ReflectionClass`, `ReflectionMethod`, `ReflectionProperty`).
    pub(crate) fn infer_new_object_type(
        &mut self,
        class_name: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let class_name = class_name.to_string();
        if self.enums.contains_key(class_name.as_str()) {
            return Err(CompileError::new(
                expr.span,
                &format!("Cannot instantiate enum: {}", class_name),
            ));
        }
        if self.interfaces.contains_key(class_name.as_str()) {
            return Err(CompileError::new(
                expr.span,
                &format!("Cannot instantiate interface: {}", class_name),
            ));
        }
        if !self.classes.contains_key(class_name.as_str()) {
            // An unresolved class (not an enum/interface handled above, and unknown everywhere in
            // the closed world) is an absent optional dependency: `new AbsentClass(...)` produces
            // an opaque `PhpType::Mixed` value with a warning instead of erroring. This is only
            // ever reached at runtime inside `class_exists`-guarded (DCE-pruned) code. Argument
            // expressions are still inferred so genuine errors inside them keep surfacing.
            if !self.class_like_exists(class_name.as_str()) {
                self.warn_absent_class(expr.span, class_name.as_str());
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
        if class_name == "Fiber" {
            self.validate_fiber_constructor_args(args, expr, env)?;
        }
        if is_phar_archive_class(&class_name) {
            self.require_phar_archive_libraries();
        }
        if matches!(
            class_name.as_str(),
            "CallbackFilterIterator" | "RecursiveCallbackFilterIterator"
        ) {
            self.specialize_callback_filter_iterator_callback(args)?;
        }
        if is_reflection_owner_class(&class_name) {
            self.validate_reflection_owner_constructor(&class_name, args, expr, env)?;
            return Ok(PhpType::Object(class_name));
        }
        if let Some(class_info) = self.classes.get(class_name.as_str()) {
            if class_info.is_abstract {
                return Err(CompileError::new(
                    expr.span,
                    &format!("Cannot instantiate abstract class: {}", class_name),
                ));
            }
            if let Some(sig) = class_info.methods.get("__construct") {
                if let Some(visibility) = class_info.method_visibilities.get("__construct") {
                    let declaring_class = class_info
                        .method_declaring_classes
                        .get("__construct")
                        .map(String::as_str)
                        .unwrap_or(class_name.as_str());
                    if !self.can_access_member(declaring_class, visibility)
                        && !self.can_construct_internal_iterator_from_builtin_get_iterator(&class_name)
                    {
                        return Err(CompileError::new(
                            expr.span,
                            &format!(
                                "Cannot access {} constructor: {}::__construct",
                                Self::visibility_label(visibility),
                                class_name
                            ),
                        ));
                    }
                }
                let declared_flags =
                    Self::declared_method_param_flags(class_info, "__construct", false);
                let effective_sig = Self::callable_sig_for_declared_params(sig, &declared_flags);
                let param_to_prop = class_info.constructor_param_to_prop.clone();
                let normalized_args = self.normalize_named_call_args(
                    &effective_sig,
                    args,
                    expr.span,
                    &format!("Constructor '{}::__construct'", class_name),
                    env,
                )?;
                let effective_sig = if matches!(
                    class_name.as_str(),
                    "CallbackFilterIterator" | "RecursiveCallbackFilterIterator"
                ) {
                    self.callback_filter_constructor_sig_for_args(
                        &effective_sig,
                        &normalized_args,
                        env,
                    )?
                } else {
                    effective_sig
                };
                self.check_known_callable_call(
                    &effective_sig,
                    &normalized_args,
                    expr.span,
                    env,
                    &format!("Constructor '{}::__construct'", class_name),
                )?;
                for (i, arg) in normalized_args.iter().enumerate() {
                    let arg_ty = self.infer_type(arg, env)?;
                    if param_to_prop.get(i).is_some_and(|mapped| mapped.is_some()) {
                        self.propagate_constructor_arg_type(class_name.as_str(), i, &arg_ty);
                    }
                }
                return Ok(PhpType::Object(class_name));
            } else if !args.is_empty() {
                return Err(CompileError::new(
                    expr.span,
                    &format!(
                        "Constructor '{}::__construct' expects 0 arguments, got {}",
                        class_name,
                        args.len()
                    ),
                ));
            }
        }
        let param_to_prop = self
            .classes
            .get(class_name.as_str())
            .map(|c| c.constructor_param_to_prop.clone())
            .unwrap_or_default();
        for (i, arg) in args.iter().enumerate() {
            let arg_ty = self.infer_type(arg, env)?;
            if param_to_prop.get(i).is_some_and(|mapped| mapped.is_some()) {
                self.propagate_constructor_arg_type(class_name.as_str(), i, &arg_ty);
            }
        }
        Ok(PhpType::Object(class_name))
    }

    /// Records the PHAR bridge and decompression libraries needed by PHAR archive helpers.
    pub(crate) fn require_phar_archive_libraries(&mut self) {
        self.require_builtin_library("elephc_phar");
        self.require_builtin_library("z");
        self.require_builtin_library("bz2");
    }

    /// Returns true when construct internal iterator from builtin get iterator.
    fn can_construct_internal_iterator_from_builtin_get_iterator(&self, class_name: &str) -> bool {
        let get_iterator_key = php_symbol_key("getIterator");
        class_name == "InternalIterator"
            && self.current_class.as_deref() == Some("SplFixedArray")
            && self.current_method.as_deref() == Some(get_iterator_key.as_str())
    }

    /// Validates constructor arguments for reflection owner classes
    /// (`ReflectionClass`, `ReflectionMethod`, `ReflectionProperty`).
    ///
    /// Extracts the reflected class/method/property from string literal args, then delegates to
    /// `validate_reflection_class_attrs`, `validate_reflection_method_attrs`, or
    /// `validate_reflection_property_attrs`. For `ReflectionClass` specifically, a non-literal
    /// `string`-typed first argument is also accepted (routed to the EIR dynamic-name dispatcher,
    /// see `crate::codegen_ir::lower_inst::objects::reflection`), in which case the
    /// attribute-argument-support validation is skipped since the reflected class is not known
    /// until runtime.
    fn validate_reflection_owner_constructor(
        &mut self,
        class_name: &str,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<(), CompileError> {
        let sig = self
            .classes
            .get(class_name)
            .and_then(|class_info| class_info.methods.get("__construct"))
            .cloned()
            .expect("builtin reflection class is missing its constructor signature");
        let normalized_args = self.normalize_named_call_args(
            &sig,
            args,
            expr.span,
            &format!("Constructor '{}::__construct'", class_name),
            env,
        )?;
        self.check_known_callable_call(
            &sig,
            &normalized_args,
            expr.span,
            env,
            &format!("Constructor '{}::__construct'", class_name),
        )?;

        if class_name == "ReflectionFunction" {
            let arg = normalized_args
                .first()
                .expect("ReflectionFunction constructor arity was validated");
            return self.validate_reflection_function_constructor_arg(arg, expr, env);
        }

        let reflected_class =
            self.reflection_class_literal_arg(class_name, &normalized_args[0], env)?;
        match class_name {
            // `reflected_class` is `None` exactly when `reflection_class_literal_arg` accepted a
            // non-literal `string`-typed argument for `ReflectionClass` (dynamic-name
            // construction, routed to the codegen dynamic dispatcher, see
            // `crate::codegen_ir::lower_inst::objects::reflection`). The reflected class is not
            // known at compile time, so the attribute-argument-support validation that only
            // matters for `getAttributes()` is skipped here; codegen resolves the runtime class
            // name and throws a catchable `\ReflectionException` for an unknown name, matching
            // PHP.
            "ReflectionClass" => match reflected_class {
                Some(name) => self.validate_reflection_class_attrs(&name, expr),
                None => Ok(()),
            },
            "ReflectionMethod" => {
                let method_name = self.reflection_member_name_arg(
                    class_name,
                    "method name",
                    normalized_args.get(1),
                    env,
                )?;
                // Either the class or the method name is a non-literal `Str` (routed to the EIR
                // dynamic dispatcher, `crate::codegen_ir::lower_inst::objects::
                // reflection_members::lower_reflection_method_new_dynamic`): the reflected member
                // is not known at compile time, so the attribute-argument-support validation
                // (which only matters for `getAttributes()`) is skipped, matching the
                // `ReflectionClass` dynamic-name precedent. On a MISS at runtime, codegen throws a
                // catchable `\ReflectionException`, matching PHP.
                match (reflected_class, method_name) {
                    (Some(reflected_class), Some(method_name)) => {
                        self.validate_reflection_method_attrs(&reflected_class, &method_name, expr)
                    }
                    _ => Ok(()),
                }
            }
            "ReflectionProperty" => {
                let property_name = self.reflection_member_name_arg(
                    class_name,
                    "property name",
                    normalized_args.get(1),
                    env,
                )?;
                match (reflected_class, property_name) {
                    (Some(reflected_class), Some(property_name)) => {
                        self.validate_reflection_property_attrs(&reflected_class, &property_name, expr)
                    }
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        }
    }

    /// Extracts the class name argument from a reflection constructor call.
    ///
    /// Accepts a string literal or `ClassName::class` constant; returns the resolved class name.
    ///
    /// For `ReflectionClass`/`ReflectionMethod`/`ReflectionProperty` — all three share PHP's real
    /// `object|string` first-argument signature (php -n verified: `new ReflectionClass($obj)` /
    /// `new ReflectionMethod($obj, 'm')` / `new ReflectionProperty($obj, 'p')` are all legal PHP,
    /// and all THREE constructors weak-coerce a non-`object|string` SCALAR the SAME way: `new
    /// ReflectionMethod(42, 'm')` throws `ReflectionException: Class "42" does not exist`, NOT a
    /// `TypeError` — php -n verified there is no PHP-visible behavioral difference between the
    /// three constructors for this argument) — ANY non-literal expression is accepted regardless
    /// of its static type (`Str`, `Mixed`, `Union`, `Object`, or anything else a caller might
    /// pass) and routes to the matching EIR dynamic dispatcher
    /// (`crate::codegen_ir::lower_inst::objects::reflection::lower_reflection_class_new_dynamic`
    /// for `ReflectionClass`, `crate::codegen_ir::lower_inst::objects::reflection_members::
    /// lower_reflection_member_new_dynamic` for `ReflectionMethod`/`ReflectionProperty`), which
    /// performs the ACTUAL runtime type determination this static checker cannot: unbox a
    /// `Mixed`/`Union` operand's runtime tag, resolve an object operand's concrete runtime class,
    /// weak-coerce a scalar/null tag to its `(string)` cast, and throw a real, catchable
    /// `\TypeError` for a genuinely non-coercible runtime tag (php -n verified: `new
    /// ReflectionMethod([1,2], 'm')` throws `TypeError: ReflectionMethod::__construct(): Argument
    /// #1 ($objectOrMethod) must be of type object|string, array given`) — exactly mirroring how
    /// PHP itself only rejects the wrong argument shape at RUNTIME, not at parse/compile time,
    /// for these three constructors.
    ///
    /// `ReflectionFunction` is handled by a separate validator entirely (see
    /// `validate_reflection_function_constructor_arg`) and never reaches this function. The
    /// SEPARATE (member NAME) second argument to `ReflectionMethod`/`ReflectionProperty` is
    /// validated by `reflection_member_name_arg`, not this function, and keeps requiring a
    /// literal-or-non-literal `Str`-typed expression only (no `Mixed`/`Object`/weak-coercion
    /// acceptance — php -n verified PHP does NOT weak-coerce the member-name argument).
    fn reflection_class_literal_arg(
        &mut self,
        reflection_type: &str,
        arg: &Expr,
        env: &TypeEnv,
    ) -> Result<Option<String>, CompileError> {
        let arg_ty = self.infer_type(arg, env)?;
        let raw_class_name = match &arg.kind {
            ExprKind::StringLiteral(class_name) if matches!(arg_ty, PhpType::Str) => {
                class_name.clone()
            }
            ExprKind::ClassConstant { receiver } if matches!(arg_ty, PhpType::Str) => {
                self.resolve_reflection_class_constant(receiver, arg.span)?
            }
            // php -n verified: `ReflectionMethod`/`ReflectionProperty`'s first argument has the
            // EXACT SAME `object|string` weak-coercion semantics as `ReflectionClass`'s (int
            // 42 → `ReflectionException: Class "42" does not exist`, `null` → `Class "" does not
            // exist`, an `array` → a real `\TypeError`, …) — there is no PHP-visible behavioral
            // difference between the three constructors for this argument, so ALL non-literal
            // expressions route to the EIR dynamic dispatcher here regardless of static type
            // (`crate::codegen_ir::lower_inst::objects::reflection_members::
            // lower_reflection_member_new_dynamic`, which performs the actual runtime type
            // determination exactly like `crate::codegen_ir::lower_inst::objects::reflection::
            // lower_reflection_class_new_dynamic` does for `ReflectionClass`).
            _ if matches!(reflection_type, "ReflectionClass" | "ReflectionMethod" | "ReflectionProperty") => {
                return Ok(None)
            }
            _ => {
                return Err(CompileError::new(
                    arg.span,
                    &format!(
                        "{}::__construct() first argument must be a string class name",
                        reflection_type
                    ),
                ));
            }
        };
        self.resolve_reflection_class_name(&raw_class_name)
            .map(|name| Some(name.to_string()))
            .ok_or_else(|| {
                CompileError::new(
                    arg.span,
                    &format!(
                        "{}::__construct(): undefined class '{}'",
                        reflection_type, raw_class_name
                    ),
                )
            })
    }

    /// Extracts a string literal argument from a reflection constructor call.
    ///
    /// The argument must be a `string` literal (dynamic lookup is not yet
    /// supported). Used for method names and property names in reflection
    /// constructors.
    fn reflection_string_literal_arg(
        &mut self,
        reflection_type: &str,
        label: &str,
        arg: Option<&Expr>,
        env: &TypeEnv,
    ) -> Result<String, CompileError> {
        let arg = arg.expect("reflection constructor arity was validated");
        let arg_ty = self.infer_type(arg, env)?;
        if !matches!(arg_ty, PhpType::Str) {
            return Err(CompileError::new(
                arg.span,
                &format!(
                    "{}::__construct() {} argument must be a string",
                    reflection_type, label
                ),
            ));
        }
        match &arg.kind {
            ExprKind::StringLiteral(value) => Ok(value.clone()),
            _ => Err(CompileError::new(
                arg.span,
                &format!(
                    "{}::__construct() requires a string literal {} (dynamic lookup is not yet supported)",
                    reflection_type, label
                ),
            )),
        }
    }

    /// Extracts a method/property-name argument from a `ReflectionMethod`/`ReflectionProperty`
    /// constructor call, ACCEPTING a non-literal `Str`-typed expression (unlike
    /// `reflection_string_literal_arg`, which `ReflectionFunction` still uses unchanged): a
    /// string literal resolves immediately (`Some(name)`, the existing literal-path behavior,
    /// byte-for-byte unaffected); any other `Str`-typed expression returns `None`, signaling the
    /// caller to route to the EIR dynamic two-string dispatcher
    /// (`crate::codegen_ir::lower_inst::objects::reflection_members`) instead of erroring. A
    /// non-`Str` argument is still a compile-time type error — this scope does not attempt PHP's
    /// full weak-coercion behavior for the member-name argument.
    fn reflection_member_name_arg(
        &mut self,
        reflection_type: &str,
        label: &str,
        arg: Option<&Expr>,
        env: &TypeEnv,
    ) -> Result<Option<String>, CompileError> {
        let arg = arg.expect("reflection constructor arity was validated");
        let arg_ty = self.infer_type(arg, env)?;
        if !matches!(arg_ty, PhpType::Str) {
            return Err(CompileError::new(
                arg.span,
                &format!(
                    "{}::__construct() {} argument must be a string",
                    reflection_type, label
                ),
            ));
        }
        match &arg.kind {
            ExprKind::StringLiteral(value) => Ok(Some(value.clone())),
            _ => Ok(None),
        }
    }

    /// Validates the single constructor argument of `new ReflectionFunction($function)`
    /// against PHP's real `Closure|string $function` signature.
    ///
    /// - A `Str`-typed argument keeps the EXISTING literal-only behavior (dynamic string
    ///   lookup is not yet supported): `reflection_string_literal_arg` +
    ///   `validate_reflection_function_target`.
    /// - A first-class callable targeting a plain FREE FUNCTION (`target(...)`) is treated
    ///   EXACTLY like passing that function's name as a string literal (php -n VERIFIED:
    ///   `(new ReflectionFunction($fn(...)))->getName()` returns the real function name, not
    ///   `"{closure}"` — PHP's FCC-created closures ARE fully named, unlike anonymous ones) —
    ///   reuses the same existence validation. The EIR lowering
    ///   (`crate::codegen_ir::lower_inst::objects::reflection::lower_reflection_function_new`)
    ///   mirrors this by resolving the FCC operand's target name the same way a string literal
    ///   operand is resolved.
    /// - A closure LITERAL (`function (...) {...}`/`fn (...) => ...`) passed directly is
    ///   accepted with no name validation (a literal always "exists"); num-params-related slots
    ///   are backed from the closure's own declared params, but `getName`/`getShortName`/
    ///   `getFileName` are gated to throw at runtime (see `builtin_reflection_guarded_method`
    ///   in `crate::types::checker::builtin_types::reflection`) since PHP's real closure-name
    ///   format cannot be soundly reproduced here (php -n VERIFIED PHP 8.5 format:
    ///   `"{closure:FILE:LINE}"` / `"{closure:Class::method():LINE}"` — NOT the bare
    ///   `"{closure}"` some older PHP versions used; the exact scope prefix even differs
    ///   between top-level, free-function, and method contexts).
    /// - Any OTHER Closure-shaped value (a `Closure`-typed variable/parameter/expression whose
    ///   identity is not statically resolvable at this call site, e.g. `function f(Closure $c) {
    ///   new ReflectionFunction($c); }`) is now ACCEPTED (M2 PART A): the runtime closure
    ///   descriptor a `Closure`/`callable`-typed value ALWAYS points at
    ///   (`crate::codegen::callable_descriptor`) already carries the exact same
    ///   signature/name/invocation metadata this constructor bakes for a literal at compile
    ///   time, so `crate::codegen_ir::lower_inst::objects::reflection::
    ///   lower_reflection_function_new_dynamic` reads it at RUNTIME instead — with a tag check
    ///   before touching it (JURY ADDENDUM item 3). A `Mixed`/`Union` value (e.g. a
    ///   `?Closure`-typed local, or one widened to `mixed` by an unrelated branch-merge) is
    ///   ALSO accepted for the SAME reason: the dynamic lowering unboxes it and dispatches on
    ///   the runtime tag, throwing a catchable `\TypeError`/`\ReflectionException` for anything
    ///   that isn't actually a callable descriptor at runtime — never a partial/wrong object.
    /// - Anything ELSE STATICALLY KNOWN to be non-Closure/non-Mixed/non-Str (e.g. a statically
    ///   `int`/`array`/`Object`-typed argument) is still a compile-time type error, mirroring
    ///   PHP's real `TypeError` for this argument (php -n verified: `new
    ///   ReflectionFunction([1,2])` throws `TypeError: ReflectionFunction::__construct():
    ///   Argument #1 ($function) must be of type Closure|string, array given`).
    fn validate_reflection_function_constructor_arg(
        &mut self,
        arg: &Expr,
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<(), CompileError> {
        let arg_ty = self.infer_type(arg, env)?;
        if matches!(arg_ty, PhpType::Str) {
            let function_name = self.reflection_string_literal_arg(
                "ReflectionFunction",
                "function name",
                Some(arg),
                env,
            )?;
            return self.validate_reflection_function_target(&function_name, expr);
        }
        if Self::type_is_closure_shaped(&arg_ty) {
            if let ExprKind::FirstClassCallable(CallableTarget::Function(name)) = &arg.kind {
                let resolved_name = self
                    .canonical_function_name_folded(name.as_str())
                    .unwrap_or_else(|| name.as_str().to_string());
                return self.validate_reflection_function_target(&resolved_name, expr);
            }
            if matches!(arg.kind, ExprKind::Closure { .. }) {
                return Ok(());
            }
            // A dynamically-typed Closure/callable value (not a literal or FCC resolvable at
            // this call site) — accepted; see the doc comment above and
            // `lower_reflection_function_new_dynamic`.
            return Ok(());
        }
        if matches!(arg_ty, PhpType::Mixed | PhpType::Union(_)) {
            // Genuinely unknown at compile time — the dynamic runtime path performs its own
            // tag check and throws a catchable diagnostic for any non-callable payload.
            return Ok(());
        }
        Err(CompileError::new(
            arg.span,
            &format!(
                "ReflectionFunction::__construct(): Argument #1 ($function) must be of type Closure|string, got {:?}",
                arg_ty
            ),
        ))
    }

    /// Returns true when `ty` statically guarantees a Closure-shaped value: either
    /// `PhpType::Callable` (elephc's uniform static type for closure literals, first-class
    /// callables, and any `Closure`/`callable`-hinted parameter — see
    /// `crate::types::checker::type_compat::pointers::resolve_type_expr`, which collapses both
    /// hint spellings to the same variant) or `PhpType::Object("Closure")` (produced by a few
    /// other inference paths, e.g. some declared return/property type resolutions).
    fn type_is_closure_shaped(ty: &PhpType) -> bool {
        matches!(ty, PhpType::Callable)
            || matches!(ty, PhpType::Object(name) if name.trim_start_matches('\\').eq_ignore_ascii_case("Closure"))
    }

    /// Validates that `new ReflectionFunction($name)` targets a known
    /// user-defined function (matched case-insensitively, as PHP function names
    /// are). Builtin functions are not yet reflectable.
    fn validate_reflection_function_target(
        &self,
        function_name: &str,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let key = crate::names::php_symbol_key(function_name.trim_start_matches('\\'));
        let exists = self
            .fn_decls
            .keys()
            .chain(self.functions.keys())
            .any(|name| crate::names::php_symbol_key(name.trim_start_matches('\\')) == key);
        if exists {
            Ok(())
        } else {
            Err(CompileError::new(
                expr.span,
                &format!(
                    "ReflectionFunction::__construct(): undefined function '{}'",
                    function_name
                ),
            ))
        }
    }

    /// Validates that a class's attributes do not have unsupported argument metadata.
    ///
    /// Returns `Ok` if the class has no attribute args or if all args are
    /// supported. Used by `ReflectionClass` constructor validation.
    fn validate_reflection_class_attrs(
        &self,
        class_name: &str,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let Some(class_info) = self.classes.get(class_name) else {
            return Err(CompileError::new(
                expr.span,
                &format!("ReflectionClass::__construct(): undefined class '{}'", class_name),
            ));
        };
        if attributes_have_unsupported_args(&class_info.attribute_names, &class_info.attribute_args)
        {
            return Err(CompileError::new(
                expr.span,
                "ReflectionClass::getAttributes(): class has attribute argument metadata that is not supported yet",
            ));
        }
        Ok(())
    }

    /// Validates that a method's attributes do not have unsupported argument metadata.
    ///
    /// Also checks that the method exists on the class. Used by
    /// `ReflectionMethod` constructor validation.
    fn validate_reflection_method_attrs(
        &self,
        class_name: &str,
        method_name: &str,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let Some(class_info) = self.classes.get(class_name) else {
            return Err(CompileError::new(
                expr.span,
                &format!("ReflectionMethod::__construct(): undefined class '{}'", class_name),
            ));
        };
        let method_key = php_symbol_key(method_name);
        if !class_info.methods.contains_key(&method_key)
            && !class_info.static_methods.contains_key(&method_key)
        {
            return Err(CompileError::new(
                expr.span,
                &format!(
                    "ReflectionMethod::__construct(): undefined method '{}::{}'",
                    class_name, method_name
                ),
            ));
        }
        let empty_names = Vec::new();
        let empty_args = Vec::new();
        let names = class_info
            .method_attribute_names
            .get(&method_key)
            .unwrap_or(&empty_names);
        let args = class_info
            .method_attribute_args
            .get(&method_key)
            .unwrap_or(&empty_args);
        if attributes_have_unsupported_args(names, args) {
            return Err(CompileError::new(
                expr.span,
                "ReflectionMethod::getAttributes(): method has attribute argument metadata that is not supported yet",
            ));
        }
        Ok(())
    }

    /// Validates that a property's attributes do not have unsupported argument metadata.
    ///
    /// Also checks that the property exists on the class (instance or static).
    /// Used by `ReflectionProperty` constructor validation.
    fn validate_reflection_property_attrs(
        &self,
        class_name: &str,
        property_name: &str,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let Some(class_info) = self.classes.get(class_name) else {
            return Err(CompileError::new(
                expr.span,
                &format!("ReflectionProperty::__construct(): undefined class '{}'", class_name),
            ));
        };
        if !class_info.properties.iter().any(|(name, _)| name == property_name)
            && !class_info
                .static_properties
                .iter()
                .any(|(name, _)| name == property_name)
        {
            return Err(CompileError::new(
                expr.span,
                &format!(
                    "ReflectionProperty::__construct(): undefined property '{}::${}'",
                    class_name, property_name
                ),
            ));
        }
        let empty_names = Vec::new();
        let empty_args = Vec::new();
        let names = class_info
            .property_attribute_names
            .get(property_name)
            .unwrap_or(&empty_names);
        let args = class_info
            .property_attribute_args
            .get(property_name)
            .unwrap_or(&empty_args);
        if attributes_have_unsupported_args(names, args) {
            return Err(CompileError::new(
                expr.span,
                "ReflectionProperty::getAttributes(): property has attribute argument metadata that is not supported yet",
            ));
        }
        Ok(())
    }

    /// Resolves a static receiver to a class name for reflection class constant.
    ///
    /// `Named` returns the canonical name. `Self_`/`Static` require a class
    /// context. `Parent` returns the parent of the current class.
    fn resolve_reflection_class_constant(
        &self,
        receiver: &StaticReceiver,
        span: crate::span::Span,
    ) -> Result<String, CompileError> {
        match receiver {
            StaticReceiver::Named(name) => Ok(name.as_canonical()),
            StaticReceiver::Self_ | StaticReceiver::Static => self
                .current_class
                .clone()
                .ok_or_else(|| CompileError::new(span, "Cannot use self::class outside a class context")),
            StaticReceiver::Parent => {
                let current = self.current_class.as_ref().ok_or_else(|| {
                    CompileError::new(span, "Cannot use parent::class outside a class context")
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

    /// Looks up a class name by PHP case-insensitive symbol key.
    ///
    /// Strips leading backslashes and uses `php_symbol_key` for comparison.
    /// Returns the canonical class name string if found.
    fn resolve_reflection_class_name<'a>(&'a self, class_name: &str) -> Option<&'a str> {
        let class_key = php_symbol_key(class_name.trim_start_matches('\\'));
        self.classes
            .keys()
            .find(|existing| php_symbol_key(existing) == class_key)
            .map(String::as_str)
    }

    /// Validates arguments passed to the `Fiber` constructor.
    ///
    /// The first argument must be a callable value. Statically known closures
    /// and first-class callables are validated immediately; descriptor-backed
    /// runtime callable values are accepted so the uniform invoker can use their
    /// runtime signature metadata at Fiber entry time.
    fn validate_fiber_constructor_args(
        &mut self,
        args: &[Expr],
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<(), CompileError> {
        let Some(callback) = args.first() else {
            return Ok(());
        };
        let Some(sig) = self.resolve_fiber_callable_sig(callback, env)? else {
            let callback_ty = self.infer_type(callback, env)?;
            if callback_ty == PhpType::Callable
                || callback_ty == PhpType::Str
                || crate::types::checker::builtins::runtime_callable_array_type(&callback_ty)
            {
                return Ok(());
            }
            return Err(CompileError::new(callback.span, "Fiber callback must be callable"));
        };

        let visible_param_count = match &callback.kind {
            ExprKind::Closure {
                params, variadic, ..
            } => fibers::visible_param_count(params.len(), variadic.is_some()),
            ExprKind::Variable(_) => sig.params.len(),
            ExprKind::FirstClassCallable(_) => sig.params.len(),
            _ => sig.params.len(),
        };

        fibers::validate_callback_signature(&sig, visible_param_count, expr.span)
    }

    /// Resolves the callable signature for Fiber callback forms supported by codegen.
    fn resolve_fiber_callable_sig(
        &mut self,
        callback: &Expr,
        env: &TypeEnv,
    ) -> Result<Option<FunctionSig>, CompileError> {
        if let Some(sig) = self.resolve_expr_callable_sig(callback, env)? {
            return Ok(Some(sig));
        }

        match &callback.kind {
            ExprKind::StringLiteral(name) => self.resolve_fiber_string_callable_sig(name, callback.span, env),
            ExprKind::ArrayLiteral(_) => self.resolve_fiber_callable_array_literal_sig(callback, env),
            ExprKind::Variable(name) => {
                if let Some(target) = self.callable_array_targets.get(name).cloned() {
                    return self.resolve_fiber_callable_array_variable_sig(&target, callback.span, env);
                }
                self.resolve_fiber_invokable_object_sig(callback, env)
            }
            ExprKind::This => self.resolve_fiber_invokable_object_sig(callback, env),
            _ => self.resolve_fiber_invokable_object_sig(callback, env),
        }
    }

    /// Resolves a string callback literal to a function or static-method signature.
    fn resolve_fiber_string_callable_sig(
        &mut self,
        name: &str,
        span: crate::span::Span,
        env: &TypeEnv,
    ) -> Result<Option<FunctionSig>, CompileError> {
        if let Some((class_name, method_name)) = name.split_once("::") {
            let Some(class_name) = self.resolve_fiber_callable_class_name(class_name) else {
                return Ok(None);
            };
            let target = CallableTarget::StaticMethod {
                receiver: StaticReceiver::Named(Name::from(class_name.to_string())),
                method: method_name.to_string(),
            };
            return self.resolve_first_class_callable_sig(&target, span, env).map(Some);
        }

        let function_name = self
            .canonical_function_name_folded(name)
            .or_else(|| crate::name_resolver::canonical_builtin_function_name(name))
            .unwrap_or_else(|| name.trim_start_matches('\\').to_string());
        let target = CallableTarget::Function(Name::from(function_name));
        self.resolve_first_class_callable_sig(&target, span, env).map(Some)
    }

    /// Resolves a literal callable array to a static-method or receiver-bound method signature.
    fn resolve_fiber_callable_array_literal_sig(
        &mut self,
        callback: &Expr,
        env: &TypeEnv,
    ) -> Result<Option<FunctionSig>, CompileError> {
        let Some(target) = self.fiber_callable_array_literal_target(callback, env)? else {
            return Ok(None);
        };
        self.resolve_first_class_callable_sig(&target, callback.span, env)
            .map(Some)
    }

    /// Resolves a tracked callable-array variable when codegen can materialize it without hidden temps.
    fn resolve_fiber_callable_array_variable_sig(
        &mut self,
        target: &CallableTarget,
        span: crate::span::Span,
        env: &TypeEnv,
    ) -> Result<Option<FunctionSig>, CompileError> {
        self.resolve_first_class_callable_sig(target, span, env).map(Some)
    }

    /// Resolves an invokable object callback signature for `$object` or `$this`.
    fn resolve_fiber_invokable_object_sig(
        &mut self,
        callback: &Expr,
        env: &TypeEnv,
    ) -> Result<Option<FunctionSig>, CompileError> {
        let callback_ty = self.infer_type(callback, env)?;
        let Some(class_name) = self.fiber_object_class_name(&callback_ty) else {
            return Ok(None);
        };
        if !self
            .classes
            .get(&class_name)
            .is_some_and(|class_info| class_info.methods.contains_key("__invoke"))
        {
            return Ok(None);
        }
        let target = CallableTarget::Method {
            object: Box::new(callback.clone()),
            method: "__invoke".to_string(),
        };
        self.resolve_first_class_callable_sig(&target, callback.span, env)
            .map(Some)
    }

    /// Builds a first-class-callable target for a supported literal callable array.
    fn fiber_callable_array_literal_target(
        &mut self,
        callback: &Expr,
        env: &TypeEnv,
    ) -> Result<Option<CallableTarget>, CompileError> {
        let Some((receiver, method)) = fiber_callable_array_parts(callback) else {
            return Ok(None);
        };
        if let Some(receiver) = self.fiber_static_callable_receiver(receiver, callback.span)? {
            return Ok(Some(CallableTarget::StaticMethod {
                receiver,
                method: method.to_string(),
            }));
        }
        let receiver_ty = self.infer_type(receiver, env)?;
        if self.fiber_object_class_name(&receiver_ty).is_none() {
            return Ok(None);
        }
        Ok(Some(CallableTarget::Method {
            object: Box::new(receiver.clone()),
            method: method.to_string(),
        }))
    }

    /// Resolves a literal callable-array receiver to a static class receiver.
    fn fiber_static_callable_receiver(
        &self,
        receiver: &Expr,
        span: crate::span::Span,
    ) -> Result<Option<StaticReceiver>, CompileError> {
        let class_name = match &receiver.kind {
            ExprKind::StringLiteral(class_name) => self
                .resolve_fiber_callable_class_name(class_name)
                .map(str::to_string),
            ExprKind::ClassConstant { receiver } => Some(
                self.resolve_fiber_callable_static_receiver_class(receiver, span)?,
            ),
            _ => None,
        };
        Ok(class_name.map(|class_name| StaticReceiver::Named(Name::from(class_name))))
    }

    /// Resolves `self`, `parent`, `static`, or named class receivers for Fiber callable arrays.
    fn resolve_fiber_callable_static_receiver_class(
        &self,
        receiver: &StaticReceiver,
        span: crate::span::Span,
    ) -> Result<String, CompileError> {
        match receiver {
            StaticReceiver::Named(class_name) => self
                .resolve_fiber_callable_class_name(class_name.as_str())
                .map(str::to_string)
                .ok_or_else(|| {
                    CompileError::new(span, &format!("Undefined class: {}", class_name.as_str()))
                }),
            StaticReceiver::Self_ | StaticReceiver::Static => {
                self.current_class.clone().ok_or_else(|| {
                    CompileError::new(span, "Cannot use self::class outside class scope")
                })
            }
            StaticReceiver::Parent => {
                let current = self.current_class.as_ref().ok_or_else(|| {
                    CompileError::new(span, "Cannot use parent::class outside class scope")
                })?;
                self.classes
                    .get(current)
                    .and_then(|class_info| class_info.parent.clone())
                    .ok_or_else(|| {
                        CompileError::new(
                            span,
                            &format!("Class '{}' has no parent class", current),
                        )
                    })
            }
        }
    }

    /// Resolves a class name case-insensitively for Fiber callable strings and arrays.
    fn resolve_fiber_callable_class_name<'a>(&'a self, class_name: &str) -> Option<&'a str> {
        let class_key = php_symbol_key(class_name.trim_start_matches('\\'));
        self.classes
            .keys()
            .find(|existing| php_symbol_key(existing) == class_key)
            .map(String::as_str)
    }

    /// Extracts an object class name from a Fiber callback type.
    fn fiber_object_class_name(&self, ty: &PhpType) -> Option<String> {
        match ty {
            PhpType::Object(class_name) => Some(class_name.clone()),
            _ => None,
        }
    }

    /// Provides the Specialize callback filter iterator callback helper used by the constructors module.
    fn specialize_callback_filter_iterator_callback(
        &mut self,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let Some(callback) = args.get(1) else {
            return Ok(());
        };

        if self.expr_call_complex_callee_needs_runtime_capture(callback)
            && !crate::types::checker::builtins::callback_supports_complex_descriptor_env(callback)
        {
            return Err(CompileError::new(
                callback.span,
                "CallbackFilterIterator callback does not support complex expressions that select captured callables at runtime",
            ));
        }

        match &callback.kind {
            ExprKind::FirstClassCallable(crate::parser::ast::CallableTarget::Function(name)) => {
                self.specialize_callback_filter_function(name.as_str(), callback.span)?;
            }
            ExprKind::Variable(var_name) => {
                if let Some(crate::parser::ast::CallableTarget::Function(name)) =
                    self.first_class_callable_targets.get(var_name).cloned()
                {
                    self.specialize_callback_filter_function(name.as_str(), callback.span)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Adjusts callback-filter constructor typing for callable-array callback variables.
    fn callback_filter_constructor_sig_for_args(
        &mut self,
        sig: &crate::types::FunctionSig,
        args: &[Expr],
        env: &TypeEnv,
    ) -> Result<crate::types::FunctionSig, CompileError> {
        let Some(callback) = args.get(1) else {
            return Ok(sig.clone());
        };
        let actual_ty = self.infer_type(callback, env)?;
        if !self.callback_filter_accepts_runtime_callable_array(callback, &actual_ty) {
            return Ok(sig.clone());
        }
        let mut adjusted_sig = sig.clone();
        if let Some((_, callback_ty)) = adjusted_sig.params.get_mut(1) {
            *callback_ty = actual_ty;
        }
        Ok(adjusted_sig)
    }

    /// Returns true when CallbackFilterIterator codegen can resolve a runtime callable array.
    fn callback_filter_accepts_runtime_callable_array(
        &self,
        callback: &Expr,
        actual_ty: &PhpType,
    ) -> bool {
        match &callback.kind {
            ExprKind::Variable(var_name) => {
                self.callable_array_targets.contains_key(var_name)
                    || crate::types::checker::builtins::runtime_callable_array_type(actual_ty)
            }
            ExprKind::ArrayLiteral(elems) if elems.len() == 2 => {
                crate::types::checker::builtins::runtime_callable_array_type(actual_ty)
            }
            _ => false,
        }
    }

    /// Provides the Specialize callback filter function helper used by the constructors module.
    fn specialize_callback_filter_function(
        &mut self,
        name: &str,
        span: crate::span::Span,
    ) -> Result<(), CompileError> {
        if crate::name_resolver::is_builtin_function(name) {
            return Ok(());
        }

        let name = self
            .canonical_function_name_folded(name)
            .unwrap_or_else(|| name.to_string());
        if !self.functions.contains_key(name.as_str()) {
            if let Some(decl) = self.fn_decls.get(name.as_str()).cloned() {
                let param_types = self.initial_function_param_types(&name, &decl)?;
                self.resolve_function_signature(&name, &decl, param_types)?;
            }
        }

        let Some(sig) = self.functions.get_mut(name.as_str()) else {
            return Err(CompileError::new(
                span,
                &format!("Undefined function for CallbackFilterIterator callback: {}", name),
            ));
        };
        let callback_arg_types = [
            PhpType::Mixed,
            PhpType::Mixed,
            PhpType::Object("Iterator".to_string()),
        ];
        for (idx, callback_arg_ty) in callback_arg_types.into_iter().enumerate() {
            if idx >= sig.params.len() {
                break;
            }
            if !sig.declared_params.get(idx).copied().unwrap_or(false)
                && !sig.ref_params.get(idx).copied().unwrap_or(false)
                && sig.params[idx].1 == PhpType::Int
            {
                sig.params[idx].1 = callback_arg_ty;
            }
        }
        Ok(())
    }

    /// Infers the type of an enum case access (`EnumName::Case`).
    ///
    /// Errors if the enum or case is undefined. Returns `PhpType::Object(enum_name)`.
    pub(crate) fn infer_enum_case_type(
        &mut self,
        enum_name: &str,
        case_name: &str,
        expr: &Expr,
    ) -> Result<PhpType, CompileError> {
        let enum_name = enum_name.to_string();
        let enum_info = self.enums.get(enum_name.as_str()).ok_or_else(|| {
            CompileError::new(expr.span, &format!("Undefined enum: {}", enum_name))
        })?;
        if !enum_info.cases.iter().any(|case| case.name == *case_name) {
            return Err(CompileError::new(
                expr.span,
                &format!("Undefined enum case: {}::{}", enum_name, case_name),
            ));
        }
        Ok(PhpType::Object(enum_name))
    }
}

/// Returns receiver and method from `[receiver, "method"]` Fiber callbacks.
fn fiber_callable_array_parts(expr: &Expr) -> Option<(&Expr, &str)> {
    let ExprKind::ArrayLiteral(elems) = &expr.kind else {
        return None;
    };
    if elems.len() != 2 {
        return None;
    }
    let ExprKind::StringLiteral(method) = &elems[1].kind else {
        return None;
    };
    Some((&elems[0], method.as_str()))
}

/// Returns `true` if `class_name` is a reflection owner class
/// (`ReflectionClass`, `ReflectionMethod`, `ReflectionProperty`).
fn is_reflection_owner_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "ReflectionClass"
            | "ReflectionMethod"
            | "ReflectionProperty"
            | "ReflectionFunction"
    )
}

/// Returns `true` if `class_name` is backed by the PHAR bridge.
fn is_phar_archive_class(class_name: &str) -> bool {
    matches!(class_name, "Phar" | "PharData")
}

/// Returns `true` if the attribute name/arg slices are mismatched or any
/// arg is `None` (indicating unsupported metadata).
fn attributes_have_unsupported_args(
    names: &[String],
    args: &[Option<Vec<crate::types::AttrArgEntry>>],
) -> bool {
    names.len() != args.len() || args.iter().any(Option::is_none)
}
