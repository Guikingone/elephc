//! Purpose:
//! Builds constructors, invocation shells, and simple slot getters.
//!
//! Called from:
//! - The Reflection checker metadata facade and sibling builders.
//!
//! Key details:
//! - Direct codegen-specialized APIs keep conservative synthetic bodies.

use super::*;

/// Returns a private parameterless `__construct` method for `ReflectionAttribute`.
pub(super) fn builtin_reflection_attribute_constructor_method() -> ClassMethod {
    builtin_reflection_private_constructor_method()
}

/// Returns a private parameterless `__construct` for internally materialized reflection objects.
pub(super) fn builtin_reflection_private_constructor_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "__construct".to_string(),
        visibility: Visibility::Private,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: None,
        by_ref_return: false,
        body: Vec::new(),
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public `getName()` method that returns the private `__name` property
/// as a `Str`.
pub(super) fn builtin_reflection_attribute_get_name_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "getName".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(TypeExpr::Str),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(
                ExprKind::PropertyAccess {
                    object: Box::new(Expr::new(ExprKind::This, dummy_span)),
                    property: "__name".to_string(),
                },
                dummy_span,
            ))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public `getArguments()` method that returns the private `__args`
/// property as an `array`.
pub(super) fn builtin_reflection_attribute_get_arguments_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "getArguments".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(TypeExpr::Named(crate::names::Name::unqualified("array"))),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(
                ExprKind::PropertyAccess {
                    object: Box::new(Expr::new(ExprKind::This, dummy_span)),
                    property: "__args".to_string(),
                },
                dummy_span,
            ))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public `newInstance()` method that returns `null` (placeholder until
/// codegen supplies the real implementation).
pub(super) fn builtin_reflection_attribute_new_instance_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "newInstance".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(mixed_type()),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(ExprKind::Null, dummy_span))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public variadic `ReflectionClass::newInstance()` method.
///
/// Direct calls are lowered specially so their source arguments become
/// constructor arguments for the reflected class. The no-argument body keeps
/// indirect calls and metadata emission coherent when no argument forwarding is
/// required.
pub(super) fn builtin_reflection_class_new_instance_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "newInstance".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        param_attributes: Vec::new(),
        variadic: Some("args".to_string()),
        variadic_by_ref: false,
        variadic_type: Some(mixed_type()),
        return_type: Some(object_type()),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(
                ExprKind::NewDynamic {
                    name_expr: Box::new(Expr::new(
                        ExprKind::PropertyAccess {
                            object: Box::new(Expr::new(ExprKind::This, dummy_span)),
                            property: "__name".to_string(),
                        },
                        dummy_span,
                    )),
                    args: Vec::new(),
                },
                dummy_span,
            ))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public variadic `ReflectionMethod::invoke()` method shell.
///
/// Direct AOT calls are lowered specially so the first argument becomes the
/// invocation receiver and the remaining source arguments are normalized
/// against the reflected method's own signature.
pub(super) fn builtin_reflection_method_invoke_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "invoke".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("object".to_string(), Some(mixed_type()), None, false)],
        param_attributes: Vec::new(),
        variadic: Some("args".to_string()),
        variadic_by_ref: false,
        variadic_type: Some(mixed_type()),
        return_type: Some(mixed_type()),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(ExprKind::Null, dummy_span))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public `ReflectionMethod::invokeArgs()` method shell.
///
/// Direct AOT calls are lowered specially so the provided argument array becomes
/// the reflected method's source argument list.
pub(super) fn builtin_reflection_method_invoke_args_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "invokeArgs".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![
            ("object".to_string(), Some(mixed_type()), None, false),
            ("args".to_string(), Some(array_type()), None, false),
        ],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(mixed_type()),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(ExprKind::Null, dummy_span))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns the concrete `getClosure()` contract for a reflected function or method.
///
/// The synthetic body supplies a valid callable for ordinary class lowering; calls on reflection
/// objects with a statically tracked target are replaced by the target-bound callable in EIR.
pub(super) fn builtin_reflection_get_closure_method(for_method: bool) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    let params = if for_method {
        vec![(
            "object".to_string(),
            Some(nullable_object_type("object")),
            null_expr(),
            false,
        )]
    } else {
        Vec::new()
    };
    let fallback_result = if for_method {
        Expr::new(ExprKind::Null, dummy_span)
    } else {
        reflection_function_dynamic_call(dummy_span)
    };
    let fallback = Expr::new(
        ExprKind::Closure {
            params: Vec::new(),
            variadic: Some("args".to_string()),
            variadic_by_ref: false,
            variadic_type: Some(mixed_type()),
            return_type: Some(mixed_type()),
            body: vec![Stmt::new(
                StmtKind::Return(Some(fallback_result)),
                dummy_span,
            )],
            is_arrow: false,
            is_static: for_method,
            by_ref_return: false,
            captures: Vec::new(),
            capture_refs: Vec::new(),
        },
        dummy_span,
    );
    ClassMethod {
        name: "getClosure".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params,
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(TypeExpr::Named(Name::unqualified("Closure"))),
        by_ref_return: false,
        body: vec![Stmt::new(StmtKind::Return(Some(fallback)), dummy_span)],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public static `ReflectionMethod::createFromMethodName()` method shell.
pub(super) fn builtin_reflection_method_create_from_method_name_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "createFromMethodName".to_string(),
        visibility: Visibility::Public,
        is_static: true,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("method".to_string(), Some(TypeExpr::Str), None, false)],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(TypeExpr::Named(Name::unqualified("ReflectionMethod"))),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(
                ExprKind::NewObject {
                    class_name: Name::unqualified("ReflectionMethod"),
                    args: vec![
                        Expr::new(ExprKind::StringLiteral(String::new()), dummy_span),
                        Expr::new(ExprKind::StringLiteral(String::new()), dummy_span),
                    ],
                },
                dummy_span,
            ))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public `setAccessible(bool $accessible)` no-op method shell.
pub(super) fn builtin_reflection_set_accessible_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "setAccessible".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("accessible".to_string(), Some(bool_type()), None, false)],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(TypeExpr::Void),
        by_ref_return: false,
        body: Vec::new(),
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public variadic `ReflectionFunction::invoke()` method shell.
///
/// Direct generated/AOT calls are lowered specially so the variadic source
/// arguments are normalized against the reflected function's own signature.
pub(super) fn builtin_reflection_function_invoke_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "invoke".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        param_attributes: Vec::new(),
        variadic: Some("args".to_string()),
        variadic_by_ref: false,
        variadic_type: Some(mixed_type()),
        return_type: Some(mixed_type()),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(reflection_function_dynamic_call(dummy_span))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public `ReflectionFunction::invokeArgs()` method shell.
///
/// Direct generated/AOT calls are lowered specially so the provided argument
/// array becomes the reflected function's source argument list.
pub(super) fn builtin_reflection_function_invoke_args_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "invokeArgs".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("args".to_string(), Some(array_type()), None, false)],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(mixed_type()),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(reflection_function_dynamic_call(dummy_span))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Builds a dynamic invocation of the retained callable or reflected function name.
fn reflection_function_dynamic_call(span: crate::span::Span) -> Expr {
    let property = |name: &str| {
        Expr::new(
            ExprKind::PropertyAccess {
                object: Box::new(Expr::new(ExprKind::This, span)),
                property: name.to_string(),
            },
            span,
        )
    };
    Expr::new(
        ExprKind::ExprCall {
            callee: Box::new(Expr::new(
                ExprKind::NullCoalesce {
                    value: Box::new(property("__callable")),
                    default: Box::new(property("__name")),
                },
                span,
            )),
            args: vec![Expr::new(
                ExprKind::Spread(Box::new(Expr::new(
                    ExprKind::Variable("args".to_string()),
                    span,
                ))),
                span,
            )],
        },
        span,
    )
}

/// Returns a public `ReflectionClass::newInstanceArgs()` method.
///
/// Direct calls are lowered specially so the provided argument array becomes
/// constructor arguments for the reflected class. The placeholder body keeps
/// the synthetic class metadata coherent for non-special paths.
pub(super) fn builtin_reflection_class_new_instance_args_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "newInstanceArgs".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        // `iterable` rather than bare `array`: the checker resolves `array`
        // to a string-keyed map, which would reject indexed argument lists;
        // Iterable accepts both array shapes.
        params: vec![(
            "args".to_string(),
            Some(TypeExpr::Iterable),
            empty_array(),
            false,
        )],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(mixed_type()),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(ExprKind::Null, dummy_span))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public no-op method that returns the private `property` slot typed
/// `return_type`. Reflection getters are populated at codegen; their bodies just
/// surface the corresponding private slot.
pub(super) fn builtin_reflection_slot_getter(
    method_name: &str,
    property: &str,
    return_type: TypeExpr,
) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: method_name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(return_type),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(
                ExprKind::PropertyAccess {
                    object: Box::new(Expr::new(ExprKind::This, dummy_span)),
                    property: property.to_string(),
                },
                dummy_span,
            ))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns the public callable-or-string constructor for `ReflectionFunction`.
/// The body is empty; codegen populates metadata slots from the reflected target.
pub(super) fn builtin_reflection_function_constructor_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "__construct".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![(
            "function".to_string(),
            Some(TypeExpr::Union(vec![
                TypeExpr::Named(Name::unqualified("Closure")),
                TypeExpr::Str,
            ])),
            None,
            false,
        )],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: None,
        by_ref_return: false,
        body: Vec::new(),
        span: dummy_span,
        attributes: Vec::new(),
    }
}
