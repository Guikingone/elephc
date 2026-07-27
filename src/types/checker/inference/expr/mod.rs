//! Purpose:
//! Dispatches expression inference for assignments, class references, closures, and side-effecting forms.
//! Feeds statement checking, function call validation, and optimizer-visible type metadata.
//!
//! Called from:
//! - `crate::types::checker::Checker::infer_type()`
//!
//! Key details:
//! - Inference must preserve PHP evaluation errors and avoid treating effectful expressions as pure type facts.

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{Expr, ExprKind};
use crate::span::Span;
use crate::types::{
    merge_array_key_types, normalized_array_key_type, packed_type_size, PhpType, TypeEnv,
};
mod assignments;
mod by_ref_outputs;
mod class_refs;
mod effects;
pub(crate) mod static_closure;
use super::super::type_compat::type_is_gradual_object_family;
use super::super::builtins::array_arg_is_gradually_acceptable;
use super::super::Checker;
use super::syntactic::wider_type_syntactic;
use static_closure::body_must_not_use_this;
pub(crate) use static_closure::closure_body_uses_this;
impl Checker {
    /// Infers the PHP return type of `expr` in the given `env`.
    ///
    /// This is the top-level dispatcher for expression type inference. It
    /// handles literals, variables, operators, array access, ternaries,
    /// function calls, closures, and all other expression forms. Errors are
    /// returned for type mismatches (e.g. negating a string) or undefined
    /// references. The result feeds statement checking, function call
    /// validation, and optimizer-visible type metadata.
    pub fn infer_type(&mut self, expr: &Expr, env: &TypeEnv) -> Result<PhpType, CompileError> {
        if let Some(evaluated) = self.evaluated_expr_types.get(&expr.span) {
            if evaluated.kind == std::mem::discriminant(&expr.kind) {
                return Ok(evaluated.ty.clone());
            }
        }
        match &expr.kind {
            // `IncludeValue` is a transient parser node fully expanded by the resolver;
            // it can never reach this pass.
            ExprKind::IncludeValue { .. } => unreachable!(
                "ExprKind::IncludeValue must be expanded by the resolver"
            ),
            ExprKind::BoolLiteral(false) => Ok(PhpType::False),
            ExprKind::BoolLiteral(true) => Ok(PhpType::Bool),
            ExprKind::Null => Ok(PhpType::Void),
            ExprKind::StringLiteral(_) => Ok(PhpType::Str),
            ExprKind::IntLiteral(_) => Ok(PhpType::Int),
            ExprKind::FloatLiteral(_) => Ok(PhpType::Float),
            ExprKind::Variable(name) => self.variable_type_or_eval_dynamic(name, expr.span, env),
            ExprKind::Negate(inner) => {
                let ty = self.infer_type(inner, env)?;
                match ty {
                    PhpType::Int => {
                        if matches!(&inner.kind, ExprKind::IntLiteral(_)) {
                            Ok(PhpType::Int)
                        } else {
                            Ok(PhpType::Mixed)
                        }
                    }
                    PhpType::Float => Ok(PhpType::Float),
                    PhpType::Mixed | PhpType::Bool | PhpType::False | PhpType::Void => {
                        Ok(PhpType::Mixed)
                    }
                    _ => Err(CompileError::new(
                        expr.span,
                        "Cannot negate a non-numeric value",
                    )),
                }
            }
            ExprKind::Not(inner) => {
                self.infer_type(inner, env)?;
                Ok(PhpType::Bool)
            }
            ExprKind::ErrorSuppress(inner) => self.infer_type(inner, env),
            ExprKind::Print(inner) => {
                self.infer_type(inner, env)?;
                Ok(PhpType::Int)
            }
            ExprKind::PreIncrement(name) | ExprKind::PreDecrement(name) => match env.get(name) {
                Some(PhpType::Int) => Ok(PhpType::Mixed),
                Some(PhpType::Mixed) => Ok(PhpType::Mixed),
                Some(PhpType::Bool) | Some(PhpType::False) | Some(PhpType::Void) => {
                    Ok(PhpType::Int)
                }
                Some(other) => Err(CompileError::new(
                    expr.span,
                    &format!("Cannot increment/decrement ${} of type {:?}", name, other),
                )),
                None => Err(CompileError::new(
                    expr.span,
                    &format!("Undefined variable: ${}", name),
                )),
            },
            ExprKind::PostIncrement(name) | ExprKind::PostDecrement(name) => match env.get(name) {
                Some(PhpType::Int)
                | Some(PhpType::Bool)
                | Some(PhpType::False)
                | Some(PhpType::Void) => Ok(PhpType::Int),
                Some(PhpType::Mixed) => Ok(PhpType::Mixed),
                Some(other) => Err(CompileError::new(
                    expr.span,
                    &format!("Cannot increment/decrement ${} of type {:?}", name, other),
                )),
                None if self.eval_barrier_active => Ok(PhpType::Int),
                None => Err(CompileError::new(
                    expr.span,
                    &format!("Undefined variable: ${}", name),
                )),
            },
            ExprKind::ArrayLiteralAssoc(pairs) => {
                if pairs.is_empty() {
                    return Err(CompileError::new(
                        expr.span,
                        "Cannot infer type of empty associative array literal",
                    ));
                }
                let mut key_ty = normalized_array_key_type(
                    &pairs[0].0,
                    self.infer_type(&pairs[0].0, env)?,
                );
                let mut val_ty = self.infer_type(&pairs[0].1, env)?;
                for (k, v) in &pairs[1..] {
                    let kt = normalized_array_key_type(k, self.infer_type(k, env)?);
                    let vt = self.infer_type(v, env)?;
                    if kt != key_ty {
                        key_ty = merge_array_key_types(key_ty, kt);
                    }
                    if vt != val_ty {
                        val_ty = PhpType::Mixed;
                    }
                }
                Ok(PhpType::AssocArray {
                    key: Box::new(key_ty),
                    value: Box::new(val_ty),
                })
            }
            ExprKind::Match {
                subject,
                arms,
                default,
            } => {
                self.infer_type(subject, env)?;
                let narrows_between_arms =
                    matches!(subject.kind, ExprKind::BoolLiteral(true));
                let mut fallthrough_env = env.clone();
                let mut result_ty: Option<PhpType> = None;
                for (conditions, result) in arms {
                    let mut arm_env = fallthrough_env.clone();
                    if narrows_between_arms && conditions.len() == 1 {
                        let condition = &conditions[0];
                        self.infer_type(condition, &fallthrough_env)?;
                        if let Some(guard) =
                            self.guard_narrowing(condition, &fallthrough_env)?
                        {
                            arm_env.insert(guard.var.clone(), guard.then_ty.clone());
                            fallthrough_env.insert(guard.var.clone(), guard.else_ty.clone());
                            // Assignment-receiver aliases (`$f = $src`) share the value and take the
                            // same then/else narrowing.
                            for alias in &guard.aliases {
                                arm_env.insert(alias.clone(), guard.then_ty.clone());
                                fallthrough_env.insert(alias.clone(), guard.else_ty.clone());
                            }
                        } else {
                            for (var, then_ty) in
                                self.and_chain_then_narrowings(condition, &fallthrough_env)
                            {
                                arm_env.insert(var, then_ty);
                            }
                        }
                    } else {
                        for condition in conditions {
                            self.infer_type(condition, &fallthrough_env)?;
                        }
                    }
                    let ty = self.match_arm_result_type(result, &arm_env)?;
                    result_ty = Some(match result_ty {
                        Some(acc) => merge_match_arm_result_type(self, acc, ty),
                        None => ty,
                    });
                }
                if let Some(d) = default {
                    let ty = self.match_arm_result_type(d, &fallthrough_env)?;
                    result_ty = Some(match result_ty {
                        Some(acc) => merge_match_arm_result_type(self, acc, ty),
                        None => ty,
                    });
                }
                Ok(result_ty.unwrap_or(PhpType::Void))
            }
            ExprKind::ArrayLiteral(elems) => {
                if elems.is_empty() {
                    return Ok(PhpType::Array(Box::new(PhpType::Never)));
                }
                let mut requires_hash_spread = false;
                for elem in elems {
                    let ExprKind::Spread(inner) = &elem.kind else {
                        continue;
                    };
                    let spread_ty = self.infer_type(inner, env)?;
                    match &spread_ty {
                        PhpType::Array(_) => {}
                        PhpType::AssocArray { .. } | PhpType::Iterable => {
                            requires_hash_spread = true;
                        }
                        PhpType::Mixed | PhpType::Union(_)
                            if array_arg_is_gradually_acceptable(&spread_ty) =>
                        {
                            requires_hash_spread = true;
                        }
                        other => {
                            return Err(CompileError::new(
                                elem.span,
                                &format!("Spread operator requires an array, got {:?}", other),
                            ));
                        }
                    }
                }
                if requires_hash_spread {
                    let value_ty = self.assoc_spread_literal_value_type(elems, env);
                    return Ok(PhpType::AssocArray {
                        key: Box::new(PhpType::Mixed),
                        value: Box::new(value_ty),
                    });
                }
                let mut elem_ty = self.infer_type(&elems[0], env)?;
                for elem in &elems[1..] {
                    let ty = self.infer_type(elem, env)?;
                    if ty != elem_ty {
                        if let Some(merged_ty) = self.merge_array_element_type(&elem_ty, &ty) {
                            elem_ty = merged_ty;
                            continue;
                        }
                        elem_ty = PhpType::Mixed;
                    }
                }
                Ok(PhpType::Array(Box::new(elem_ty)))
            }
            ExprKind::ArrayAccess { array, index } => {
                let arr_ty = self.infer_type(array, env)?;
                let idx_ty = self.infer_type(index, env)?;
                let normalized_idx_ty = normalized_array_key_type(index, idx_ty.clone());
                match &arr_ty {
                    PhpType::Str => {
                        if !string_offset_index_is_gradually_acceptable(index, &idx_ty) {
                            return Err(CompileError::new(
                                expr.span,
                                "String index must be integer",
                            ));
                        }
                        Ok(PhpType::Str)
                    }
                    PhpType::Array(elem_ty) => {
                        if normalized_idx_ty == PhpType::Int {
                            Ok(*elem_ty.clone())
                        } else if array_key_is_gradually_acceptable(&idx_ty, &normalized_idx_ty) {
                            // Gradual typing: a string/Mixed/coercible key into an array the
                            // checker inferred packed/int-keyed is accepted as an associative
                            // read. The key unboxes/normalizes at the access boundary and may
                            // miss (PHP null) or hit, so the value widens to `Mixed`.
                            Ok(PhpType::Mixed)
                        } else {
                            Err(CompileError::new(
                                expr.span,
                                "Array index must be integer",
                            ))
                        }
                    }
                    PhpType::AssocArray { value, .. } => {
                        // Assoc arrays accept string or int keys
                        Ok(*value.clone())
                    }
                    PhpType::Object(class_name) => {
                        if self.object_type_implements_interface(class_name, "ArrayAccess") {
                            Ok(self.array_access_offset_get_type(class_name))
                        } else {
                            Err(CompileError::new(expr.span, "Cannot index non-array"))
                        }
                    }
                    PhpType::Union(members) => {
                        let mut result_members = Vec::new();
                        let mut saw_indexable_member = false;
                        let mut first_index_error = None;
                        for member in members {
                            match member {
                                PhpType::Void => result_members.push(PhpType::Void),
                                // Scalar-only union members (e.g. `int|false`, `float|null`):
                                // indexing them is a PHP miss (`Warning` + `null`), never a
                                // fatal — same probe-verified matrix as the bare-scalar arm
                                // above. Contributes a `Mixed` member so a union like
                                // `int|false` isn't left with an empty result set.
                                PhpType::Bool | PhpType::Int | PhpType::Float => {
                                    saw_indexable_member = true;
                                    result_members.push(PhpType::Mixed);
                                }
                                PhpType::Str => {
                                    saw_indexable_member = true;
                                    if !string_offset_index_is_gradually_acceptable(index, &idx_ty) {
                                        first_index_error =
                                            first_index_error.or(Some("String index must be integer"));
                                        continue;
                                    }
                                    result_members.push(PhpType::Str);
                                }
                                PhpType::Array(elem_ty) => {
                                    saw_indexable_member = true;
                                    if normalized_idx_ty == PhpType::Int {
                                        result_members.push(*elem_ty.clone());
                                    } else if array_key_is_gradually_acceptable(
                                        &idx_ty,
                                        &normalized_idx_ty,
                                    ) {
                                        // String/coercible key on indexed array: PHP
                                        // promotes to hash at runtime; element may be Mixed.
                                        result_members.push(PhpType::Mixed);
                                    } else {
                                        first_index_error =
                                            first_index_error.or(Some("Array index must be integer"));
                                        continue;
                                    }
                                }
                                PhpType::AssocArray { value, .. } => {
                                    saw_indexable_member = true;
                                    result_members.push(*value.clone());
                                }
                                PhpType::Object(class_name) => {
                                    if self.object_type_implements_interface(
                                        class_name,
                                        "ArrayAccess",
                                    ) {
                                        saw_indexable_member = true;
                                        result_members
                                            .push(self.array_access_offset_get_type(class_name));
                                    }
                                }
                                PhpType::Buffer(elem_ty) => {
                                    saw_indexable_member = true;
                                    if idx_ty != PhpType::Int {
                                        first_index_error =
                                            first_index_error.or(Some("Buffer index must be integer"));
                                        continue;
                                    }
                                    match elem_ty.as_ref() {
                                        PhpType::Packed(name) => {
                                            result_members.push(PhpType::Pointer(Some(name.clone())))
                                        }
                                        _ => result_members.push(*elem_ty.clone()),
                                    }
                                }
                                _ => {}
                            }
                        }
                        let has_concrete_result =
                            result_members.iter().any(|member| *member != PhpType::Void);
                        if !has_concrete_result && saw_indexable_member {
                            Err(CompileError::new(
                                expr.span,
                                first_index_error.unwrap_or("Cannot index non-array"),
                            ))
                        } else if result_members.is_empty() {
                            Err(CompileError::new(expr.span, "Cannot index non-array"))
                        } else {
                            Ok(self.normalize_union_type(result_members))
                        }
                    }
                    PhpType::Buffer(elem_ty) => {
                        if idx_ty != PhpType::Int {
                            return Err(CompileError::new(
                                expr.span,
                                "Buffer index must be integer",
                            ));
                        }
                        match elem_ty.as_ref() {
                            PhpType::Packed(name) => Ok(PhpType::Pointer(Some(name.clone()))),
                            _ => Ok(*elem_ty.clone()),
                        }
                    }
                    // Mixed receivers fall through to runtime dispatch. The
                    // boxed payload may carry an indexed array, an assoc
                    // hash, or a stdClass; codegen unboxes and routes to
                    // the right runtime helper. Missing keys decode to
                    // `Mixed(null)` at runtime, mirroring PHP's silent
                    // "undefined index" warning behavior for this very
                    // common idiom (e.g. `json_decode($json, true)["k"]`).
                    PhpType::Mixed => Ok(PhpType::Mixed),
                    // Bare scalar receivers (`false[0]`, `null["k"]`, `5[0]`, `5.5[0]`):
                    // PHP treats indexing a scalar as a miss — a `Warning` plus `null`,
                    // never a fatal (probe-verified: `php -n -r '$x=false; var_dump($x[0]);'`
                    // → `Warning: Trying to access array offset on false` + `NULL`; same for
                    // int/float/null receivers). `Void` also covers the null literal here (this
                    // checker has no separate null type). The write path is unaffected — this
                    // arm only widens reads. Ir_lower boxes the unboxed scalar into a transient
                    // `Mixed` cell (`Op::MixedBox`) before routing through the shared boxed-Mixed
                    // reader, which already yields `Mixed(null)` for any non-array/hash/object tag.
                    PhpType::Bool | PhpType::Int | PhpType::Float | PhpType::Void => {
                        Ok(PhpType::Mixed)
                    }
                    _ => Err(CompileError::new(expr.span, "Cannot index non-array")),
                }
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.infer_type(condition, env)?;
                // Flow-narrowing across the branches: `$x instanceof X ? ... : ...` (and the other
                // recognized guards) narrow `$x` — or a simple `$x->prop` — in the then/else
                // branches. A ternary is a single expression (no intervening writes), so the
                // narrowing is safe to scope to each branch's inference.
                let (then_ty, else_ty) = if let Some(guard) =
                    self.guard_narrowing(condition, env)?
                {
                    let mut then_env = env.clone();
                    then_env.insert(guard.var.clone(), guard.then_ty.clone());
                    let mut else_env = env.clone();
                    else_env.insert(guard.var.clone(), guard.else_ty.clone());
                    // Assignment-receiver aliases (`$f = $src`) share the value and take the same
                    // then/else narrowing.
                    for alias in &guard.aliases {
                        then_env.insert(alias.clone(), guard.then_ty.clone());
                        else_env.insert(alias.clone(), guard.else_ty.clone());
                    }
                    (
                        self.match_arm_result_type(then_expr, &then_env)?,
                        self.match_arm_result_type(else_expr, &else_env)?,
                    )
                } else {
                    (
                        self.match_arm_result_type(then_expr, env)?,
                        self.match_arm_result_type(else_expr, env)?,
                    )
                };
                // Same Mixed/nullable merge as match arms: heterogeneous heap
                // types must not collapse through the Str-absorbing syntactic join.
                Ok(merge_match_arm_result_type(self, then_ty, else_ty))
            }
            ExprKind::ShortTernary { value, default } => {
                let value_ty = self.match_arm_result_type(value, env)?;
                let value_ty = short_ternary_truthy_result_type(self, value_ty);
                let default_ty = self.match_arm_result_type(default, env)?;
                Ok(merge_match_arm_result_type(self, value_ty, default_ty))
            }
            ExprKind::Throw(inner) => {
                let thrown_ty = self.infer_type(inner, env)?;
                match thrown_ty {
                    PhpType::Object(type_name)
                        if self.object_type_implements_throwable(&type_name) =>
                    {
                        Ok(PhpType::Void)
                    }
                    PhpType::Object(_) => Err(CompileError::new(
                        expr.span,
                        "Type error: throw requires an object implementing Throwable",
                    )),
                    // Gradual operands (`Mixed`, `?Object`, object-plus-scalar or
                    // object/`Mixed`-bearing unions) match the `throw` statement arm: the
                    // Throwable contract is only checked at runtime, so accept gradually. A
                    // proven non-object (bare `Int`/`Str`/array/…) stays loud.
                    ref ty if type_is_gradual_object_family(ty) => Ok(PhpType::Void),
                    _ => Err(CompileError::new(
                        expr.span,
                        "Type error: throw requires an object value",
                    )),
                }
            }
            ExprKind::Cast { target, expr } => {
                self.infer_type(expr, env)?;
                use crate::parser::ast::CastType;
                Ok(match target {
                    CastType::Int => PhpType::Int,
                    CastType::Float => PhpType::Float,
                    CastType::String => PhpType::Str,
                    CastType::Bool => PhpType::Bool,
                    CastType::Array => PhpType::Array(Box::new(PhpType::Int)),
                    CastType::Object => PhpType::Object("stdClass".to_string()),
                })
            }
            ExprKind::FunctionCall { name, args } => {
                let name = name.as_str().to_string();
                let args = args.clone();
                if self.extern_functions.contains_key(name.as_str()) {
                    return self.check_extern_function_call(name.as_str(), &args, expr.span, env);
                }
                if let Some(ty) = self.check_builtin(name.as_str(), &args, expr.span, env)? {
                    self.builtin_call_types.insert(expr.span, ty.clone());
                    return Ok(ty);
                }
                self.check_function_call(name.as_str(), &args, expr.span, env)
            }
            ExprKind::BufferNew { element_type, len } => {
                let len_ty = self.infer_type(len, env)?;
                if len_ty != PhpType::Int {
                    return Err(CompileError::new(
                        expr.span,
                        "buffer_new<T>() length must be integer",
                    ));
                }
                let elem_ty = self.resolve_type_expr(element_type, expr.span)?;
                if packed_type_size(&elem_ty, &self.packed_classes).is_none() {
                    return Err(CompileError::new(
                        expr.span,
                        "buffer_new<T>() requires a POD scalar, pointer, or packed class element type",
                    ));
                }
                Ok(PhpType::Buffer(Box::new(elem_ty)))
            }
            ExprKind::BitNot(inner) => {
                let ty = self.infer_type(inner, env)?;
                if matches!(ty, PhpType::Int | PhpType::Bool | PhpType::False | PhpType::Void) {
                    return Ok(PhpType::Int);
                }
                // Runtime-polymorphic unary NOT: a dynamic operand (`Mixed`/union that could hold a
                // string at runtime, e.g. `fread()`'s `string|false`) cannot statically choose
                // between PHP's bytewise-string NOT and integer NOT, so the result type is `Mixed`.
                // `crate::ir_lower::expr` routes exactly this case to `Op::MixedBitwiseNot` /
                // `__rt_mixed_bitwise_not`, mirroring the binary `~`/`&`/`|`/`^` dispatch. A concrete
                // array/object operand cannot be a bitwise operand and stays a loud compile error.
                if super::ops::is_dynamic_bitwise_operand(self, &ty) {
                    return Ok(PhpType::Mixed);
                }
                Err(CompileError::new(
                    expr.span,
                    "Bitwise NOT requires integer operand",
                ))
            }
            ExprKind::NullCoalesce { value, default } => {
                let vt = if let ExprKind::PropertyAccess { object, property } = &value.kind {
                    self.infer_property_null_coalesce_type(
                        object,
                        property,
                        value,
                        env,
                    )?
                } else {
                    self.infer_type(value, env)?
                };
                let dt = self.infer_type(default, env)?;
                if Self::union_contains_void(&vt) {
                    Ok(wider_type_syntactic(&self.strip_void_from_union(&vt), &dt))
                } else {
                    Ok(wider_type_syntactic(&vt, &dt))
                }
            }
            ExprKind::Pipe { value, callable } => {
                self.infer_pipe_type(value, callable, expr, env)
            }
            ExprKind::Assignment {
                target,
                value,
                result_target,
                prelude,
                ..
            } => {
                let mut scoped_env = env.clone();
                self.check_assignment_expression(
                    target,
                    value,
                    result_target.as_deref(),
                    prelude,
                    expr.span,
                    &mut scoped_env,
                )
            }
            ExprKind::ListUnpack { value, .. } => {
                // `[$a, $b] = EXPR` evaluates to EXPR, so the expression's type is the type of
                // EXPR. The destructured target locals are typed by the statement-flow checker
                // (env is immutable here, as for `ExprKind::Assignment`); their element type is
                // recovered at codegen time in `lower_list_unpack_expr`.
                self.infer_type(value, env)
            }
            ExprKind::ConstRef(name) => {
                self.constants
                    .get(name.as_str())
                    .cloned()
                    .or_else(|| self.eval_barrier_active.then_some(PhpType::Mixed))
                    // A platform-conditional PHP predefined constant (e.g. the Windows-only
                    // `PHP_WINDOWS_VERSION_*` family) is genuinely defined on another platform,
                    // not on this AOT target. PHP defers undefined-constant resolution to runtime,
                    // so tolerate the reference as `Mixed` here; `lower_const_ref` lowers it to the
                    // same `__rt_constant` runtime lookup PHP performs, which throws a catchable
                    // `\Error` on a miss (harmless in the dead Windows branches these sit behind).
                    // A truly-undefined constant (typo / genuine gap) is NOT on the curated list
                    // and stays the loud compile-time error below.
                    .or_else(|| {
                        super::super::builtins::is_platform_conditional_constant(name.as_str())
                            .then_some(PhpType::Mixed)
                    })
                    .ok_or_else(|| {
                        CompileError::new(expr.span, &format!("Undefined constant: {}", name))
                    })
            }
            ExprKind::FirstClassCallable(target) => {
                self.infer_first_class_callable_target(target, expr.span, env)?;
                Ok(PhpType::Callable)
            }
            ExprKind::Closure {
                params,
                variadic,
                variadic_by_ref,
                variadic_type: _,
                return_type,
                body,
                is_arrow: _,
                is_static,
                captures,
                capture_refs,
                by_ref_return: _,
            } => {
                if *is_static {
                    body_must_not_use_this(body, expr.span)?;
                }
                self.infer_closure_type(
                    params,
                    variadic,
                    *variadic_by_ref,
                    return_type,
                    body,
                    captures,
                    capture_refs,
                    expr,
                    env,
                )
            }
            ExprKind::Spread(inner) => {
                let ty = self.infer_type(inner, env)?;
                match ty {
                    PhpType::Array(elem_ty) => Ok(*elem_ty),
                    PhpType::AssocArray { value, .. } => Ok(*value),
                    // Gradual boundary: a `Mixed`, an `Iterable`, or a union containing an
                    // array member is accepted just like the array-literal spread arm. The
                    // call-argument spread lowering routes such operands through a runtime
                    // array guard (`Op::MixedToHash` + `array_values()`, which raises a clean
                    // `TypeError` fatal on a non-array payload — never a SIGSEGV) or through
                    // `iterator_to_array()` for a `Traversable`, materializing a concrete
                    // packed `array<mixed>` before the existing concrete-array spread
                    // machinery expands it. The element type is unknown, so it widens to
                    // `Mixed`. A statically-proven non-iterable (int/string/object) stays
                    // loud in the fallthrough arm.
                    PhpType::Mixed | PhpType::Iterable => Ok(PhpType::Mixed),
                    PhpType::Union(_) if array_arg_is_gradually_acceptable(&ty) => {
                        Ok(PhpType::Mixed)
                    }
                    other => Err(CompileError::new(
                        expr.span,
                        &format!("Spread operator requires an array, got {:?}", other),
                    )),
                }
            }
            ExprKind::NamedArg { value, .. } => self.infer_type(value, env),
            ExprKind::ClosureCall { var, args } => {
                self.infer_closure_call_type(var, args, expr, env)
            }
            ExprKind::ExprCall { callee, args } => {
                self.infer_expr_call_type(callee, args, expr, env)
            }
            ExprKind::BinaryOp { left, op, right } => {
                self.infer_binary_op_type(left, op, right, expr, env)
            }
            ExprKind::InstanceOf { value, target } => {
                self.infer_instanceof_type(value, target, expr, env)
            }
            ExprKind::NewObject { class_name, args } => {
                self.infer_new_object_type(class_name.as_str(), args, expr, env)
            }
            ExprKind::Clone(inner) => {
                let ty = self.infer_type(inner, env)?;
                if let PhpType::Object(class_name) = &ty {
                    self.check_clone_visibility(class_name, expr.span)?;
                }
                if clone_operand_may_hold_object(&ty) {
                    Ok(ty)
                } else {
                    Err(CompileError::new(
                        expr.span,
                        "clone requires an object value",
                    ))
                }
            }
            ExprKind::NewDynamic { name_expr, args } => {
                // The class is named at runtime; without a literal class
                // we can't typecheck constructor args or the resulting
                // object's type. Infer the name expression for its side
                // effects + warnings, type-check the args generically, and
                // return Mixed.
                self.infer_type(name_expr, env)?;
                for arg in args {
                    self.infer_type(arg, env)?;
                }
                Ok(PhpType::Mixed)
            }
            ExprKind::NewDynamicObject {
                class_name,
                fallback_class,
                args,
                ..
            } => {
                let class_ty = self.infer_type(class_name, env)?;
                if class_ty != PhpType::Str {
                    return Err(CompileError::new(
                        class_name.span,
                        "Dynamic object factory class must be a string",
                    ));
                }
                self.infer_new_object_type(fallback_class.as_str(), args, expr, env)?;
                self.require_phar_archive_libraries();
                Ok(PhpType::Object(fallback_class.as_str().to_string()))
            }
            ExprKind::PropertyAccess { object, property } => {
                self.infer_property_access_type(object, property, expr, env)
            }
            ExprKind::DynamicPropertyAccess { object, property } => {
                self.infer_dynamic_property_access_type(object, property, expr, env, false)
            }
            ExprKind::NullsafePropertyAccess { object, property } => {
                self.infer_nullsafe_property_access_type(object, property, expr, env)
            }
            ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
                self.infer_dynamic_property_access_type(object, property, expr, env, true)
            }
            ExprKind::StaticPropertyAccess { receiver, property } => {
                self.infer_static_property_access_type(receiver, property, expr)
            }
            ExprKind::DynamicStaticPropertyAccess { receiver, property } => {
                self.infer_dynamic_static_property_access_type(receiver, property, expr, env)
            }
            ExprKind::MethodCall {
                object,
                method,
                args,
            } => self.infer_method_call_type(object, method, args, expr, env),
            ExprKind::NullsafeMethodCall {
                object,
                method,
                args,
            } => self.infer_nullsafe_method_call_type(object, method, args, expr, env),
            ExprKind::NullsafeDynamicMethodCall {
                object,
                method,
                args,
            } => self.infer_nullsafe_dynamic_method_call_type(object, method, args, expr, env),
            ExprKind::StaticMethodCall {
                receiver,
                method,
                args,
            } => self.infer_static_method_call_type(receiver, method, args, expr, env),
            ExprKind::This => self.infer_this_type(expr, env),
            ExprKind::PtrCast {
                target_type,
                expr: inner,
            } => self.infer_ptr_cast_type(target_type, inner, expr, env),
            ExprKind::ClassConstant { receiver } => {
                self.validate_class_constant_receiver(receiver, expr.span)?;
                Ok(PhpType::Str)
            }
            ExprKind::ObjectClassName { object } => {
                let object_type = self.infer_type(object, env)?;
                if !object_class_name_operand_may_hold_object(&object_type) {
                    return Err(CompileError::new(
                        expr.span,
                        &format!("Cannot use \"::class\" on {}", object_type),
                    ));
                }
                Ok(PhpType::Str)
            }
            ExprKind::ScopedConstantAccess { receiver, name } => {
                self.infer_scoped_constant_access(receiver, name, expr)
            }
            ExprKind::DynamicClassConstantAccess { object, name } => {
                self.infer_dynamic_class_constant_access(object, name, expr, env)
            }
            ExprKind::NewScopedObject { receiver, args } => {
                let class_name = match receiver {
                    crate::parser::ast::StaticReceiver::Self_ => {
                        self.current_class.clone().ok_or_else(|| {
                            CompileError::new(
                                expr.span,
                                "Cannot use 'new self()' outside a class context",
                            )
                        })?
                    }
                    crate::parser::ast::StaticReceiver::Static => {
                        let class_name = self.current_class.clone().ok_or_else(|| {
                            CompileError::new(
                                expr.span,
                                "Cannot use 'new static()' outside a class context",
                            )
                        })?;
                        self.validate_late_bound_constructor_targets(&class_name, args, expr, env)?;
                        return Ok(PhpType::Object(class_name));
                    }
                    crate::parser::ast::StaticReceiver::Parent => {
                        let current = self.current_class.as_ref().ok_or_else(|| {
                            CompileError::new(
                                expr.span,
                                "Cannot use 'new parent()' outside a class context",
                            )
                        })?;
                        self.classes
                            .get(current)
                            .and_then(|info| info.parent.clone())
                            .ok_or_else(|| {
                                CompileError::new(
                                    expr.span,
                                    &format!("Class '{}' has no parent class", current),
                                )
                            })?
                    }
                    crate::parser::ast::StaticReceiver::Named(name) => name.as_canonical(),
                };
                self.infer_new_object_type(&class_name, args, expr, env)
            }
            ExprKind::Yield { key, value } => {
                if let Some(k) = key {
                    self.infer_type(k, env)?;
                }
                if let Some(v) = value {
                    self.infer_type(v, env)?;
                }
                Ok(PhpType::Mixed)
            }
            ExprKind::YieldFrom(inner) => {
                let inner_ty = self.infer_type(inner, env)?;
                // `yield from` accepts arrays, the gradual `iterable` family, concrete
                // Iterator/IteratorAggregate objects, and Generator. EIR dispatches an
                // `Iterable` or `Mixed` value at runtime so Generator keeps delegated
                // send/return semantics while arrays and other Traversable values use the
                // existing iterator loop. A statically non-iterable concrete type remains
                // rejected; `Mixed` is the gradual boundary and receives a runtime type check.
                let supported = matches!(
                    inner_ty.codegen_repr(),
                    PhpType::Array(_)
                        | PhpType::AssocArray { .. }
                        | PhpType::Iterable
                        | PhpType::Mixed
                ) || matches!(
                    &inner_ty,
                    PhpType::Object(name) if self.object_type_implements_iterable(name)
                ) || self.type_accepts(&PhpType::Object("Generator".to_string()), &inner_ty);
                if !supported {
                    return Err(CompileError::new(
                        inner.span,
                        &format!(
                            "yield from expects an array or Generator, got {:?}",
                            inner_ty
                        ),
                    ));
                }
                Ok(PhpType::Mixed)
            }
            ExprKind::MagicConstant(_) => {
                unreachable!("MagicConstant must be lowered before type inference")
            }
        }
    }

    /// Returns a variable type, allowing dynamic eval-created locals after an eval barrier.
    fn variable_type_or_eval_dynamic(
        &self,
        name: &str,
        span: Span,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        env.get(name)
            .cloned()
            .or_else(|| self.eval_barrier_active.then_some(PhpType::Mixed))
            .ok_or_else(|| CompileError::new(span, &format!("Undefined variable: ${}", name)))
    }

    /// Returns the element type of a hash-backed array literal spread.
    ///
    /// Extracts value types from concrete spread arrays; gradual and iterable operands contribute
    /// `Mixed`. All spread value types must agree, otherwise `Mixed` is returned. Non-spread
    /// elements are ignored.
    fn assoc_spread_literal_value_type(&mut self, elems: &[Expr], env: &TypeEnv) -> PhpType {
        let mut value_ty = PhpType::Never;
        for elem in elems {
            let ExprKind::Spread(inner) = &elem.kind else {
                continue;
            };
            let next = match self.infer_type(inner, env) {
                Ok(PhpType::Array(elem)) => *elem,
                Ok(PhpType::AssocArray { value, .. }) => *value,
                _ => PhpType::Mixed,
            };
            if matches!(value_ty, PhpType::Never) {
                value_ty = next;
            } else if value_ty != next {
                value_ty = PhpType::Mixed;
            }
        }
        if matches!(value_ty, PhpType::Never) {
            PhpType::Mixed
        } else {
            value_ty
        }
    }

    /// Returns the return type of the `offsetGet` method for `class_name`,
    /// or `Mixed` if no `offsetGet` method is found.
    ///
    /// Looks up `offsetGet` in the class's method table first, then falls back
    /// to the `ArrayAccess` interface. Used when indexing an `Object` that
    /// implements `ArrayAccess`.
    fn array_access_offset_get_type(&self, class_name: &str) -> PhpType {
        self.classes
            .get(class_name)
            .and_then(|class_info| class_info.methods.get("offsetget"))
            .map(|sig| sig.return_type.clone())
            .or_else(|| {
                self.interfaces
                    .get("ArrayAccess")
                    .and_then(|interface_info| interface_info.methods.get("offsetget"))
                    .map(|sig| sig.return_type.clone())
            })
            .unwrap_or(PhpType::Mixed)
    }

    /// Infers a match/ternary arm result type for branch merging. Throw arms
    /// produce no value, so their checker type (`Void`, shared with `null`) is
    /// normalized to `Never` here: the merge must distinguish "arm never yields"
    /// (defer to the other arms) from "arm yields null" (keep the merge nullable).
    fn match_arm_result_type(
        &mut self,
        result: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let ty = self.infer_type(result, env)?;
        if matches!(result.kind, ExprKind::Throw(_)) {
            return Ok(PhpType::Never);
        }
        Ok(ty)
    }
}

/// Returns whether `ty` can carry an object at runtime and may therefore reach PHP's clone
/// runtime guard. Definite scalars, arrays, resources, pointers, and null are rejected by the
/// checker; gradual `Mixed`/`Iterable` values and unions with an object-capable member defer the
/// final object-kind check to `__rt_object_clone`.
fn clone_operand_may_hold_object(ty: &PhpType) -> bool {
    match ty {
        PhpType::Object(_) | PhpType::Mixed | PhpType::Iterable | PhpType::Never => true,
        PhpType::Union(members) => members.iter().any(clone_operand_may_hold_object),
        _ => false,
    }
}

/// Returns whether `ty` can carry an object for PHP's runtime-checked `$value::class` operation.
/// Definite non-object types stay compile errors, while `Mixed` and unions containing an object
/// defer the final tag check to `Op::ObjectClassName`.
fn object_class_name_operand_may_hold_object(ty: &PhpType) -> bool {
    match ty {
        PhpType::Object(_) | PhpType::Mixed => true,
        PhpType::Union(members) => members
            .iter()
            .any(object_class_name_operand_may_hold_object),
        _ => false,
    }
}

impl Checker {
    /// Checks whether the current scope may invoke a class's `__clone` hook.
    ///
    /// PHP permits `__clone` to be non-public, but the actual `clone $object`
    /// expression must obey the hook's visibility when a hook exists.
    fn check_clone_visibility(&self, class_name: &str, span: Span) -> Result<(), CompileError> {
        let normalized = class_name.trim_start_matches('\\');
        let Some(class_info) = self.classes.get(normalized) else {
            return Ok(());
        };
        let key = php_symbol_key("__clone");
        let Some(visibility) = class_info.method_visibilities.get(&key) else {
            return Ok(());
        };
        let declaring_class = class_info
            .method_declaring_classes
            .get(&key)
            .map(String::as_str)
            .unwrap_or(normalized);
        if self.can_access_member(declaring_class, visibility) {
            return Ok(());
        }
        Err(CompileError::new(
            span,
            &format!(
                "Cannot access {} method: {}::__clone",
                Self::visibility_label(visibility),
                normalized
            ),
        ))
    }
}

/// Returns `true` if `index` is a valid string offset index for a string receiver.
///
/// A valid index is an integer type, or a string literal whose value can be
/// parsed as a PHP string offset (e.g. `"0"`, `"-1"`, `"10"`).
fn is_valid_string_offset_index(index: &Expr, idx_ty: &PhpType) -> bool {
    *idx_ty == PhpType::Int
        || *idx_ty == PhpType::Mixed
        || matches!(
            &index.kind,
            ExprKind::StringLiteral(value)
                if crate::types::parse_php_string_offset_literal(value).is_some()
        )
}

/// Merges two match arm result types: identical arms keep their type,
/// `Never`-typed arms (`throw`, normalized at the call site) defer to the
/// other arm's type, `Void`-typed arms (checker `null`) keep the merge
/// nullable so the null arm's value survives return-type-driven coercion.
/// Indexed-array and associative-array pairs retain their common container
/// representation, widening incompatible element/key types to `Mixed`.
/// Object pairs, including supported `false`/null sentinels, retain a normalized
/// union so declared object-union returns and member validation remain precise;
/// every other heterogeneous pair widens to `Mixed` so each arm's runtime value
/// survives instead of being coerced to the first arm's type.
fn merge_match_arm_result_type(checker: &Checker, acc: PhpType, next: PhpType) -> PhpType {
    if acc == next {
        return acc;
    }
    if acc == PhpType::Never {
        return next;
    }
    if next == PhpType::Never {
        return acc;
    }
    if acc == PhpType::Void {
        return nullable_match_arm_type(next);
    }
    if next == PhpType::Void {
        return nullable_match_arm_type(acc);
    }
    match (acc, next) {
        (PhpType::Array(left), PhpType::Array(right)) => {
            let element = checker
                .merge_array_element_type(&left, &right)
                .unwrap_or(PhpType::Mixed);
            return PhpType::Array(Box::new(element));
        }
        (
            PhpType::AssocArray {
                key: left_key,
                value: left_value,
            },
            PhpType::AssocArray {
                key: right_key,
                value: right_value,
            },
        ) => {
            let key = checker
                .merge_array_element_type(&left_key, &right_key)
                .unwrap_or(PhpType::Mixed);
            let value = checker
                .merge_array_element_type(&left_value, &right_value)
                .unwrap_or(PhpType::Mixed);
            return PhpType::AssocArray {
                key: Box::new(key),
                value: Box::new(value),
            };
        }
        (acc, next) => {
            if object_union_match_arm_type(&acc) && object_union_match_arm_type(&next) {
                return merge_object_union_match_arm_types(checker, acc, next);
            }
        }
    }
    PhpType::Mixed
}

/// Removes members that cannot reach the truthy arm of PHP's short ternary operator.
///
/// `null`, literal `false`, and `never` always select the default expression. Other scalar
/// types remain because their static type can contain both truthy and falsy runtime values.
fn short_ternary_truthy_result_type(checker: &Checker, ty: PhpType) -> PhpType {
    match ty {
        PhpType::Void | PhpType::False | PhpType::Never => PhpType::Never,
        PhpType::Union(members) => {
            let reachable = members
                .into_iter()
                .filter(|member| {
                    !matches!(member, PhpType::Void | PhpType::False | PhpType::Never)
                })
                .collect::<Vec<_>>();
            if reachable.is_empty() {
                PhpType::Never
            } else {
                checker.normalize_union_type(reachable)
            }
        }
        other => other,
    }
}

/// Joins object/sentinel branch types at their existing compatible supertype
/// when one accepts the other, otherwise retaining a normalized union. A null
/// member from either side is restored after comparing the non-null members.
fn merge_object_union_match_arm_types(
    checker: &Checker,
    acc: PhpType,
    next: PhpType,
) -> PhpType {
    let nullable = Checker::union_contains_void(&acc) || Checker::union_contains_void(&next);
    let acc_object = checker.strip_void_from_union(&acc);
    let next_object = checker.strip_void_from_union(&next);
    let merged = if checker.type_accepts(&acc_object, &next_object) {
        acc_object
    } else if checker.type_accepts(&next_object, &acc_object) {
        next_object
    } else {
        checker.normalize_union_type(vec![acc_object, next_object])
    };
    if nullable {
        nullable_match_arm_type(merged)
    } else {
        merged
    }
}

/// Returns whether a branch type contains only concrete objects plus supported
/// `false`/null sentinel members, which can be preserved as a checker-level
/// union even though codegen materializes it through boxed `Mixed` storage.
fn object_union_match_arm_type(ty: &PhpType) -> bool {
    match ty {
        PhpType::Object(_) | PhpType::False => true,
        PhpType::Union(members) => members
            .iter()
            .all(|member| matches!(member, PhpType::Object(_) | PhpType::False | PhpType::Void)),
        _ => false,
    }
}

/// Widens a match arm type to also admit PHP null, for merges where another
/// arm is a `null` literal.
fn nullable_match_arm_type(ty: PhpType) -> PhpType {
    match ty {
        PhpType::Mixed => PhpType::Mixed,
        PhpType::Union(members) if members.contains(&PhpType::Void) => PhpType::Union(members),
        PhpType::Union(mut members) => {
            members.push(PhpType::Void);
            PhpType::Union(members)
        }
        other => PhpType::Union(vec![other, PhpType::Void]),
    }
}

/// Returns true when `idx_ty` may index a string offset under elephc's gradual-typing
/// boundary model.
///
/// Beyond a statically-`int` index (or a parseable numeric string literal), this accepts the
/// gradual top type `Mixed`, the scalar families PHP coerces to an integer offset (`bool`,
/// `float`, `string`), the null-like `Void`/`Never` tags, and a union composed solely of
/// those coercible members. EIR codegen emits the same unbox + coerce-to-int guard the
/// typed-parameter path uses (`coerce_to_int_at_span` before `Op::StrCharAt`). Array, object,
/// buffer, resource, pointer, and callable index types stay rejected as real type errors.
fn string_offset_index_is_gradually_acceptable(index: &Expr, idx_ty: &PhpType) -> bool {
    is_valid_string_offset_index(index, idx_ty) || php_type_is_int_offset_coercible(idx_ty)
}

/// Returns true when a value of `ty` can be coerced to an integer string offset at the
/// gradual-typing access boundary: `Mixed`, the scalar families PHP unambiguously casts to an
/// integer offset (`int`, `float`, `bool`), the null-like `Void`/`Never` tags, and unions
/// composed only of those.
///
/// `Str` is deliberately excluded: PHP coerces only a *numeric* string offset, throwing a
/// `TypeError` on a non-numeric one, so a statically-`string` index (a non-numeric string
/// literal, or a `string` variable whose contents are unknown) stays a compile error. A
/// numeric string literal is still accepted earlier through `is_valid_string_offset_index`.
/// Heap container types (`Array`, `Object`, `Buffer`, …) are also rejected.
fn php_type_is_int_offset_coercible(ty: &PhpType) -> bool {
    match ty {
        PhpType::Int
        | PhpType::Float
        | PhpType::Bool
        | PhpType::Mixed
        | PhpType::Void
        | PhpType::Never => true,
        PhpType::Union(members) => members.iter().all(php_type_is_int_offset_coercible),
        _ => false,
    }
}

/// Returns true when a non-`int` key expression type may still be used to
/// index an array the checker inferred packed/int-keyed, under the gradual-typing model.
///
/// A `string`/`Mixed`/coercible-scalar key (or a union of those) is accepted and treated as
/// an associative read: PHP coerces or hashes the key at runtime, so the array is interpreted
/// associatively rather than erroring. A clearly non-key type (`array`/`object`/buffer/…) is
/// still rejected. The `int` case is handled by the caller before this predicate runs.
fn array_key_is_gradually_acceptable(idx_ty: &PhpType, normalized_idx_ty: &PhpType) -> bool {
    php_type_is_array_key_coercible(normalized_idx_ty) || php_type_is_array_key_coercible(idx_ty)
}

/// Returns true when a value of `ty` can act as a PHP array key at the gradual-typing
/// boundary: `int`, `string`, `Mixed`, the scalar families PHP casts to a key (`bool`,
/// `float`), the null-like `Void`/`Never` tags (PHP treats a null key as `""`), and unions
/// composed only of those. Heap container types are rejected.
fn php_type_is_array_key_coercible(ty: &PhpType) -> bool {
    match ty {
        PhpType::Int
        | PhpType::Str
        | PhpType::Mixed
        | PhpType::Bool
        | PhpType::Float
        | PhpType::Void
        | PhpType::Never => true,
        PhpType::Union(members) => members.iter().all(php_type_is_array_key_coercible),
        _ => false,
    }
}
