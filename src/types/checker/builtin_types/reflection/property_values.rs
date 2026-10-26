//! Purpose:
//! Builds ReflectionProperty value access and initialization methods.
//!
//! Called from:
//! - The Reflection checker metadata facade and sibling builders.
//!
//! Key details:
//! - Static and instance guards keep materialized storage and dynamic access distinct.

use super::*;

/// Returns `ReflectionProperty::getValue()` for dynamic public instance reflectors.
///
/// Declared `?object $object = null` exactly as php declares it: the declaration is the contract
/// a subclass override is checked against at run time, and Doctrine's
/// `RuntimeReflectionProperty::getValue(?object $object = null): mixed` was refused as a narrowed
/// parameter against the `mixed` this used to say.
pub(super) fn builtin_reflection_property_get_value_method() -> ClassMethod {
    builtin_reflection_property_get_method(
        "getValue",
        TypeExpr::Nullable(Box::new(object_type())),
        null_expr(),
        false,
    )
}

/// Returns `ReflectionProperty::getRawValue()` for direct backing-slot reads.
///
/// The read itself is `getValue()`'s: both bodies were the same by-name load, which the backend
/// expands over every class that can hold the property. The raw variant keeps its own guards (a
/// failure names `getRawValue`) and then DELEGATES, so the program carries one expansion instead
/// of two -- the same shape `setRawValue()` already has.
pub(super) fn builtin_reflection_property_get_raw_value_method() -> ClassMethod {
    builtin_reflection_property_get_method("getRawValue", object_type(), None, true)
}

/// Builds a ReflectionProperty value reader with the requested object-argument default.
fn builtin_reflection_property_get_method(
    method_name: &str,
    object_type_expr: TypeExpr,
    object_default: Option<Expr>,
    delegate_to_get_value: bool,
) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    let object = variable_expr("object", dummy_span);
    let read = if delegate_to_get_value {
        method_call_expr(
            Expr::new(ExprKind::This, dummy_span),
            "getValue",
            vec![object],
            dummy_span,
        )
    } else {
        reflection_dynamic_object_property(object, dummy_span)
    };
    ClassMethod {
        name: method_name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![(
            "object".to_string(),
            Some(object_type_expr),
            object_default,
            false,
        )],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(mixed_type()),
        by_ref_return: false,
        body: vec![
            reflection_property_static_get_value_return(dummy_span),
            reflection_property_object_required_guard(method_name, dummy_span),
            Stmt::new(StmtKind::Return(Some(read)), dummy_span),
        ],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns `ReflectionProperty::setValue()` for dynamic public instance reflectors.
pub(super) fn builtin_reflection_property_set_value_method() -> ClassMethod {
    builtin_reflection_property_set_method("setValue", mixed_type(), false)
}

/// Returns `ReflectionProperty::setRawValue()` for direct backing-slot writes.
pub(super) fn builtin_reflection_property_set_raw_value_method() -> ClassMethod {
    builtin_reflection_property_set_method("setRawValue", object_type(), true)
}

/// Returns the raw write variant that suppresses lazy-object initialization.
pub(super) fn builtin_reflection_property_set_raw_value_without_lazy_initialization_method(
) -> ClassMethod {
    builtin_reflection_property_set_method(
        "setRawValueWithoutLazyInitialization",
        object_type(),
        true,
    )
}

/// Builds a ReflectionProperty value writer with method-specific diagnostics.
///
/// The write itself is `$object->{$this->__name} = $value`, a by-name store the backend expands
/// over every class that can hold the property. The raw variants run their own guards, so a
/// failure names them, and then DELEGATE the store to `setValue()`: once those guards passed,
/// `setValue()`'s identical ones pass too, and the program carries one expansion instead of
/// three -- about 4 M instructions of the compiled Symfony console.
fn builtin_reflection_property_set_method(
    method_name: &str,
    object_type_expr: TypeExpr,
    delegate_to_set_value: bool,
) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    let object = variable_expr("object", dummy_span);
    let value = variable_expr("value", dummy_span);
    let store = if delegate_to_set_value {
        method_call_expr(
            Expr::new(ExprKind::This, dummy_span),
            "setValue",
            vec![object, value],
            dummy_span,
        )
    } else {
        Expr::new(
            ExprKind::Assignment {
                target: Box::new(reflection_dynamic_object_property(object, dummy_span)),
                value: Box::new(value),
                result_target: None,
                prelude: Vec::new(),
                conditional_value_temp: None,
            },
            dummy_span,
        )
    };
    ClassMethod {
        name: method_name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![
            ("object".to_string(), Some(object_type_expr), None, false),
            ("value".to_string(), Some(mixed_type()), None, false),
        ],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(TypeExpr::Void),
        by_ref_return: false,
        body: vec![
            reflection_property_static_value_guard(method_name, dummy_span),
            reflection_property_object_required_guard(method_name, dummy_span),
            Stmt::new(StmtKind::ExprStmt(store), dummy_span),
        ],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns `ReflectionProperty::isInitialized()` for supported materialized reflectors.
pub(super) fn builtin_reflection_property_is_initialized_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    let static_return = reflection_property_static_is_initialized_return(dummy_span);
    let dynamic_return = Stmt::new(
        StmtKind::If {
            condition: reflection_this_property("__is_dynamic", dummy_span),
            then_body: vec![Stmt::new(StmtKind::Return(true_bool()), dummy_span)],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        dummy_span,
    );
    let defaulted_return = Stmt::new(
        StmtKind::If {
            condition: reflection_this_property("__has_default_value", dummy_span),
            then_body: vec![Stmt::new(StmtKind::Return(true_bool()), dummy_span)],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        dummy_span,
    );
    ClassMethod {
        name: "isInitialized".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("object".to_string(), Some(mixed_type()), null_expr(), false)],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(bool_type()),
        by_ref_return: false,
        body: vec![
            static_return,
            reflection_property_object_required_guard("isInitialized", dummy_span),
            dynamic_return,
            defaulted_return,
            reflection_property_array_cast_is_initialized_return(dummy_span),
        ],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// `isInitialized()` for a typed property without a default on a reflector only known at run
/// time: `(array) $object` omits an uninitialized typed property, exactly as php's does, so the
/// property is initialized when its key -- plain, `\0*\0name` or `\0Class\0name` -- is present.
///
/// The inline-known case lowers to a direct slot probe; this body used to throw instead, and
/// Doctrine's `TypedNoDefaultReflectionProperty::getValue()` (every entity id) took every
/// symfony/demo page with a database query down with it.
fn reflection_property_array_cast_is_initialized_return(span: crate::span::Span) -> Stmt {
    let fields = Expr::new(
        ExprKind::Cast {
            target: crate::parser::ast::CastType::Array,
            expr: Box::new(variable_expr("object", span)),
        },
        span,
    );
    let name = || reflection_this_property("__name", span);
    let key_exists = |key: Expr| {
        Expr::new(
            ExprKind::FunctionCall {
                name: crate::names::Name::unqualified("array_key_exists"),
                args: vec![key, variable_expr("__fields", span)],
            },
            span,
        )
    };
    let or = |left: Expr, right: Expr| {
        Expr::new(
            ExprKind::BinaryOp { left: Box::new(left), op: BinOp::Or, right: Box::new(right) },
            span,
        )
    };
    let protected_key = concat_expr(string_lit("\0*\0", span), name(), span);
    let private_key = concat_expr(
        concat_expr(
            concat_expr(string_lit("\0", span), reflection_this_property("__class", span), span),
            string_lit("\0", span),
            span,
        ),
        name(),
        span,
    );
    let assign_fields = Stmt::new(
        StmtKind::Assign { name: "__fields".to_string(), value: fields },
        span,
    );
    let answer = or(or(key_exists(name()), key_exists(protected_key)), key_exists(private_key));
    Stmt::new(
        StmtKind::Synthetic(vec![assign_fields, Stmt::new(StmtKind::Return(Some(answer)), span)]),
        span,
    )
}

/// `$this->getDeclaringClass()`: the declaring class a static branch reads, built on demand for a
/// compiled reflector and read from its slot for one the eval bridge built.
fn reflection_declaring_class_call(span: crate::span::Span) -> Expr {
    method_call_expr(Expr::new(ExprKind::This, span), "getDeclaringClass", Vec::new(), span)
}

/// Returns a static-property `getValue()` branch backed by the declaring ReflectionClass snapshot.
pub(super) fn reflection_property_static_get_value_return(span: crate::span::Span) -> Stmt {
    Stmt::new(
        StmtKind::If {
            condition: reflection_this_property("__is_static", span),
            then_body: vec![Stmt::new(
                StmtKind::Return(Some(method_call_expr(
                    reflection_declaring_class_call(span),
                    "getStaticPropertyValue",
                    vec![reflection_this_property("__name", span)],
                    span,
                ))),
                span,
            )],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        span,
    )
}

/// Returns a static-property `isInitialized()` branch backed by materialized static values.
pub(super) fn reflection_property_static_is_initialized_return(span: crate::span::Span) -> Stmt {
    Stmt::new(
        StmtKind::If {
            condition: reflection_this_property("__is_static", span),
            then_body: vec![Stmt::new(
                StmtKind::Return(Some(function_call(
                    "array_key_exists",
                    vec![
                        reflection_this_property("__name", span),
                        method_call_expr(
                            reflection_declaring_class_call(span),
                            "getStaticProperties",
                            Vec::new(),
                            span,
                        ),
                    ],
                    span,
                ))),
                span,
            )],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        span,
    )
}

/// Builds a guard for static property value access that still needs inline lowering.
pub(super) fn reflection_property_static_value_guard(method: &str, span: crate::span::Span) -> Stmt {
    Stmt::new(
        StmtKind::If {
            condition: reflection_this_property("__is_static", span),
            then_body: vec![throw_new_reflection_exception(
                string_lit(
                    &format!(
                        "ReflectionProperty::{}() for static properties requires an inline known static property",
                        method
                    ),
                    span,
                ),
                span,
            )],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        span,
    )
}

/// Builds a guard requiring an object argument for instance property value access.
pub(super) fn reflection_property_object_required_guard(method: &str, span: crate::span::Span) -> Stmt {
    Stmt::new(
        StmtKind::If {
            condition: binary_expr(
                variable_expr("object", span),
                BinOp::StrictEq,
                null_value(span),
                span,
            ),
            then_body: vec![throw_new_reflection_exception(
                string_lit(
                    &format!(
                        "ReflectionProperty::{}() requires an object for instance properties",
                        method
                    ),
                    span,
                ),
                span,
            )],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        span,
    )
}

/// Builds `$object->{$this->__name}` for ReflectionProperty value access.
pub(super) fn reflection_dynamic_object_property(object: Expr, span: crate::span::Span) -> Expr {
    Expr::new(
        ExprKind::DynamicPropertyAccess {
            object: Box::new(object),
            property: Box::new(reflection_this_property("__name", span)),
        },
        span,
    )
}

/// Returns `ReflectionProperty::isLazy()` for the non-lazy property model elephc supports.
pub(super) fn builtin_reflection_property_is_lazy_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "isLazy".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("object".to_string(), Some(object_type()), None, false)],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(bool_type()),
        by_ref_return: false,
        body: vec![Stmt::new(StmtKind::Return(false_bool()), dummy_span)],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns `ReflectionProperty::skipLazyInitialization()` as a no-op for non-lazy properties.
pub(super) fn builtin_reflection_property_skip_lazy_initialization_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    let static_modifier = binary_expr(
        binary_expr(
            reflection_this_property("__modifiers", dummy_span),
            BinOp::BitAnd,
            Expr::new(ExprKind::IntLiteral(16), dummy_span),
            dummy_span,
        ),
        BinOp::StrictNotEq,
        Expr::new(ExprKind::IntLiteral(0), dummy_span),
        dummy_span,
    );
    ClassMethod {
        name: "skipLazyInitialization".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("object".to_string(), Some(object_type()), None, false)],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(TypeExpr::Void),
        by_ref_return: false,
        body: vec![
            Stmt::new(
                StmtKind::If {
                    condition: static_modifier,
                    then_body: vec![throw_new_reflection_exception(
                        string_lit(
                            "Can not use skipLazyInitialization on static property",
                            dummy_span,
                        ),
                        dummy_span,
                    )],
                    elseif_clauses: Vec::new(),
                    else_body: None,
                },
                dummy_span,
            ),
            Stmt::new(
                StmtKind::If {
                    condition: reflection_this_property("__is_virtual", dummy_span),
                    then_body: vec![throw_new_reflection_exception(
                        string_lit(
                            "Can not use skipLazyInitialization on virtual property",
                            dummy_span,
                        ),
                        dummy_span,
                    )],
                    elseif_clauses: Vec::new(),
                    else_body: None,
                },
                dummy_span,
            ),
        ],
        span: dummy_span,
        attributes: Vec::new(),
    }
}
