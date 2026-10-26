//! Purpose:
//! Infers primitive, array, branching, cast, and unary expression types.
//!
//! Called from:
//! - "super::Checker::infer_type()" for the basic expression family.
//!
//! Key details:
//! - Preserves PHP array widening, nullability, match joins, and string-offset validation.

use super::{
    is_valid_string_offset_index, merge_match_arm_result_type, Checker,
};
use crate::errors::CompileError;
use crate::parser::ast::{ArrayEntry, Expr, ExprKind};
use crate::types::{merge_array_key_types, normalized_array_key_type, PhpType, TypeEnv};

impl Checker {
    /// Infers types for literals, variables, arrays, branches, unary operators, and casts.
    pub(super) fn infer_basic_expr_type(
        &mut self,
        expr: &Expr,
        env: &TypeEnv,
    ) -> Result<PhpType, CompileError> {
        match &expr.kind {
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
            ExprKind::ArrayReference(inner) => {
                self.infer_type(inner, env)?;
                // `[&$this->pauseExpiry, ...]` (Symfony's `NativeResponse`): the literal's slot
                // shares the property's cell, which must then hold a boxed value.
                if let ExprKind::PropertyAccess { object, property } = &inner.kind {
                    if !crate::types::checker::stmt_check::widen_property_for_element_reference(
                        self, object, property, env,
                    )? {
                        return Err(CompileError::new(
                            expr.span,
                            "A reference to a typed property in an array literal is not supported \
                             by the native backend",
                        ));
                    }
                }
                Ok(PhpType::Mixed)
            }
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
                    PhpType::Union(members)
                        if members.iter().all(is_numeric_negate_member_type) =>
                    {
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
                // PHP's string increment can change the value's type (`"9"++` is
                // `int(10)`), so the pre-form's value is dynamically tagged. EIR lowering
                // gives the local boxed Mixed frame storage for the same reason.
                Some(PhpType::Str) => {
                    self.reject_unboxable_string_incdec(name, expr.span)?;
                    self.record_string_incdec_local(name);
                    Ok(PhpType::Mixed)
                }
                // PHP's `++`/`--` on a float adds or subtracts 1.0 and keeps the float.
                Some(PhpType::Float) => Ok(PhpType::Float),
                Some(PhpType::Bool) | Some(PhpType::False) | Some(PhpType::Void) => {
                    Ok(PhpType::Int)
                }
                Some(PhpType::Union(members)) if members.iter().all(is_inc_dec_member_type) => {
                    Ok(PhpType::Mixed)
                }
                Some(other) => Err(CompileError::new(
                    expr.span,
                    &increment_type_error(name, other),
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
                // The post-forms yield the value the local held BEFORE the update, so a
                // string local still answers `string` even though the update itself can
                // retype the local (see the pre-form arm).
                Some(PhpType::Str) => {
                    self.reject_unboxable_string_incdec(name, expr.span)?;
                    self.record_string_incdec_local(name);
                    Ok(PhpType::Str)
                }
                // The post-forms yield the float the local held before the update.
                Some(PhpType::Float) => Ok(PhpType::Float),
                Some(PhpType::Union(members)) if members.iter().all(is_inc_dec_member_type) => {
                    Ok(PhpType::Mixed)
                }
                Some(other) => Err(CompileError::new(
                    expr.span,
                    &increment_type_error(name, other),
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
                let mut key_ty: Option<PhpType> = None;
                let mut val_ty: Option<PhpType> = None;
                for (k, v) in pairs {
                    let kt = normalized_array_key_type(k, self.infer_type(k, env)?);
                    let vt = self.infer_type(v, env)?;
                    key_ty = Some(match key_ty {
                        Some(current) if current == kt => current,
                        Some(current) => merge_array_key_types(current, kt),
                        None => kt,
                    });
                    val_ty = Some(match val_ty {
                        Some(current) if current == vt => current,
                        Some(_) => PhpType::Mixed,
                        None => vt,
                    });
                }
                Ok(PhpType::AssocArray {
                    key: Box::new(key_ty.unwrap_or(PhpType::Mixed)),
                    value: Box::new(val_ty.unwrap_or(PhpType::Mixed)),
                })
            }
            ExprKind::ArrayLiteralMixed(entries) => {
                // A literal that mixes a spread with explicit keys is always associative: the
                // explicit key rules out packed storage, and the spread means the key set is
                // not a compile-time fact. The KEY type is therefore `Mixed` regardless of how
                // the written keys look -- a spread contributes integer keys of its own, which
                // are renumbered, on top of whatever string keys its source carries.
                let mut val_ty: Option<PhpType> = None;
                for entry in entries {
                    let next = match entry {
                        ArrayEntry::Spread(source) => {
                            // The entry contains an ExprKind::Spread node. Its inference already
                            // returns the source's element type, including nested array elements.
                            self.infer_type(source, env)?
                        }
                        ArrayEntry::Keyed(key, value) => {
                            self.infer_type(key, env)?;
                            self.infer_type(value, env)?
                        }
                        ArrayEntry::Value(value) => self.infer_type(value, env)?,
                    };
                    val_ty = Some(match val_ty {
                        Some(acc) if acc == next => acc,
                        Some(_) => PhpType::Mixed,
                        None => next,
                    });
                }
                Ok(PhpType::AssocArray {
                    key: Box::new(PhpType::Mixed),
                    value: Box::new(val_ty.unwrap_or(PhpType::Mixed)),
                })
            }
            ExprKind::Match {
                subject,
                arms,
                default,
            } => {
                self.infer_type(subject, env)?;
                let mut result_ty: Option<PhpType> = None;
                for (conditions, result) in arms {
                    for c in conditions {
                        self.infer_type(c, env)?;
                    }
                    let arm_env = self.match_arm_narrowed_env(subject, conditions, env)?;
                    let ty = self.match_arm_result_type(result, &arm_env)?;
                    result_ty = Some(match result_ty {
                        Some(acc) => merge_match_arm_result_type(self, acc, ty),
                        None => ty,
                    });
                }
                if let Some(d) = default {
                    let ty = self.match_arm_result_type(d, env)?;
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
                if elems.iter().any(|elem| {
                    let ExprKind::Spread(inner) = &elem.kind else {
                        return false;
                    };
                    match self.infer_type(inner, env).map(|ty| ty.codegen_repr()) {
                        Ok(PhpType::AssocArray { .. } | PhpType::Iterable | PhpType::Mixed) => true,
                        Ok(PhpType::Object(name)) => self.object_type_implements_iterable(&name),
                        _ => false,
                    }
                }) {
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
                let buffer = matches!(arr_ty, PhpType::Buffer(_));
                self.buffer_read_observations.entry(expr.span)
                    .and_modify(|previous| *previous &= buffer)
                    .or_insert(buffer);
                let normalized_idx_ty = normalized_array_key_type(index, idx_ty.clone());
                match &arr_ty {
                    PhpType::Str => {
                        if !is_valid_string_offset_index(index, &idx_ty) {
                            return Err(CompileError::new(
                                expr.span,
                                "String index must be integer",
                            ));
                        }
                        Ok(PhpType::Str)
                    }
                    PhpType::Array(elem_ty) => {
                        if normalized_idx_ty != PhpType::Int {
                            // PHP allows string keys on indexed arrays: the array
                            // promotes to hash at runtime. Return the element type
                            // widened to Mixed so ?? / isset / reads type-check.
                            Ok(PhpType::Mixed)
                        } else if matches!(elem_ty.as_ref(), PhpType::Never) {
                            // `Never` is the bottom element type of an empty array, not the
                            // observable result of an offset read. A missing offset yields null,
                            // and later writes can also populate the same runtime container.
                            Ok(PhpType::Mixed)
                        } else {
                            Ok(*elem_ty.clone())
                        }
                    }
                    PhpType::AssocArray { value, .. } => {
                        // Assoc arrays accept string or int keys
                        Ok(*value.clone())
                    }
                    PhpType::Object(class_name) => {
                        if self.object_type_implements_interface(class_name, "ArrayAccess") {
                            Ok(self.array_access_offset_get_type(class_name))
                        } else if self.object_type_may_gain_array_access(class_name) {
                            // The static type (a contract, a non-final base) does not say
                            // whether the object behind it is indexable; lowering reads it
                            // through the same boxed run-time dispatch a `mixed` receiver uses.
                            Ok(PhpType::Mixed)
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
                                PhpType::Str => {
                                    saw_indexable_member = true;
                                    if !is_valid_string_offset_index(index, &idx_ty) {
                                        first_index_error =
                                            first_index_error.or(Some("String index must be integer"));
                                        continue;
                                    }
                                    result_members.push(PhpType::Str);
                                }
                                PhpType::Array(elem_ty) => {
                                    saw_indexable_member = true;
                                    if normalized_idx_ty != PhpType::Int {
                                        // String key on indexed array: PHP promotes
                                        // to hash at runtime; element may be Mixed.
                                        result_members.push(PhpType::Mixed);
                                    } else if matches!(elem_ty.as_ref(), PhpType::Never) {
                                        result_members.push(PhpType::Mixed);
                                    } else {
                                        result_members.push(*elem_ty.clone());
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
                                    } else if self.object_type_may_gain_array_access(class_name) {
                                        saw_indexable_member = true;
                                        result_members.push(PhpType::Mixed);
                                    }
                                }
                                PhpType::Buffer(elem_ty) => {
                                    saw_indexable_member = true;
                                    if !matches!(idx_ty, PhpType::Int | PhpType::Mixed) {
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
                        if !matches!(idx_ty, PhpType::Int | PhpType::Mixed) {
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
                    // A variadic that received named arguments is typed `iterable` (its storage is
                    // a list or a hash). Refusing the read here failed the specialized re-check
                    // silently and left the first pass's `int` placeholder as the function's
                    // return type: Laravel's `literal(...$arguments)` returned `$arguments[0]` as
                    // an int and cast its `(object)` result to one.
                    PhpType::Iterable => Ok(PhpType::Mixed),
                    // `isset($n['k'])` / `$n['k'] ?? $d` reach through a null base in PHP and
                    // answer `false` / the default; only a probe context may do so.
                    PhpType::Void if self.null_probe_depth > 0 => Ok(PhpType::Void),
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
                    then_env.insert(guard.var.clone(), guard.then_ty);
                    let mut else_env = env.clone();
                    else_env.insert(guard.var, guard.else_ty);
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
                let default_ty = self.match_arm_result_type(default, env)?;
                Ok(merge_match_arm_result_type(self, value_ty, default_ty))
            }
            // A `throw` EXPRESSION never produces a value: it is the bottom type, accepted wherever
            // a value is wanted (`new LogicException(throw new RuntimeException(..))`, an argument
            // of a call, the arm of a `??`), and it must not read as a `null` of type `void`.
            ExprKind::Throw(inner) => {
                let thrown_ty = self.infer_type(inner, env)?;
                match thrown_ty {
                    PhpType::Object(type_name)
                        if self.object_type_implements_throwable(&type_name)
                            || self.unresolved_new_object_defers_to_runtime(inner, &type_name)
                            || self.absent_class_defers_to_runtime(&type_name)
                            || self.object_type_may_be_eval_declared(&type_name) =>
                    {
                        Ok(PhpType::Never)
                    }
                    PhpType::Mixed if self.eval_barrier_active => Ok(PhpType::Never),
                    PhpType::Object(_) => Err(CompileError::new(
                        expr.span,
                        "Type error: throw requires an object implementing Throwable",
                    )),
                    // `Never` reaches here only from a guard the checker has just proven
                    // cannot be taken, so the statement is unreachable rather than wrong.
                    PhpType::Never => Ok(PhpType::Never),
                    ref ty
                        if crate::types::checker::type_compat::type_is_gradual_object_family(ty) =>
                    {
                        Ok(PhpType::Never)
                    }
                    _ => Err(CompileError::new(
                        expr.span,
                        "Type error: throw requires an object value",
                    )),
                }
            }
            ExprKind::Cast { target, expr } => {
                let source_ty = self.infer_type(expr, env)?;
                use crate::parser::ast::CastType;
                Ok(match target {
                    CastType::Int => PhpType::Int,
                    CastType::Float => PhpType::Float,
                    CastType::String => PhpType::Str,
                    CastType::Bool => PhpType::Bool,
                    CastType::Array
                        if matches!(source_ty.codegen_repr(), PhpType::Object(_)) =>
                    PhpType::AssocArray {
                        key: Box::new(PhpType::Str),
                        value: Box::new(PhpType::Mixed),
                    },
                    CastType::Array
                        if matches!(
                            source_ty.codegen_repr(),
                            PhpType::Mixed | PhpType::Union(_)
                        ) => PhpType::Mixed,
                    // A PHP array cast can otherwise produce an empty array, an indexed array
                    // containing any scalar/object value, or preserve an existing hash. The EIR
                    // runtime therefore uses its kind-dispatched `array<mixed>` representation.
                    CastType::Array => PhpType::Array(Box::new(PhpType::Mixed)),
                    // `(object)` is the identity on an object, so the static class survives the
                    // cast; a runtime-typed source may already hold an unrelated class, which
                    // only `mixed` can describe. Every other source becomes a fresh stdClass.
                    CastType::Object
                        if matches!(source_ty.codegen_repr(), PhpType::Object(_)) =>
                    {
                        source_ty.clone()
                    }
                    // A declared `array` is represented as the exact packed-or-associative
                    // storage union. `codegen_repr()` erases that union to Mixed, but every
                    // member still casts to stdClass, so retain the precise result type.
                    CastType::Object if source_ty.is_php_array() => {
                        PhpType::Object("stdClass".to_string())
                    }
                    CastType::Object
                        if matches!(
                            source_ty.codegen_repr(),
                            PhpType::Mixed | PhpType::Union(_)
                        ) =>
                    {
                        PhpType::Mixed
                    }
                    CastType::Object => PhpType::Object("stdClass".to_string()),
                })
            }
            _ => unreachable!("non-basic expression routed to basic inference"),
        }
    }
}

/// Returns whether one union member supports PHP numeric unary negation.
fn is_numeric_negate_member_type(ty: &PhpType) -> bool {
    matches!(
        ty,
        PhpType::Int
            | PhpType::Float
            | PhpType::Bool
            | PhpType::False
            | PhpType::Void
            | PhpType::Mixed
    )
}

/// Returns whether one union member supports PHP's local increment/decrement operators.
fn is_inc_dec_member_type(ty: &PhpType) -> bool {
    matches!(
        ty,
        PhpType::Int
            | PhpType::Float
            | PhpType::Bool
            | PhpType::False
            | PhpType::Void
            | PhpType::Str
            | PhpType::Mixed
    )
}

/// Formats the diagnostic for `++`/`--` applied to a local elephc cannot update in place.
///
/// `int`, `float`, `bool`, `null`, `string`, and boxed `mixed` locals all have an increment
/// path; everything else (arrays, objects, buffers, pointers) reaches this diagnostic.
fn increment_type_error(name: &str, ty: &PhpType) -> String {
    format!("Cannot increment/decrement ${} of type {:?}", name, ty)
}
