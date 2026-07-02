//! Purpose:
//! Synthesises the built-in reflection class checker metadata so user code can
//! receive `ReflectionAttribute` instances and query class/member attributes
//! through a small PHP-compatible Reflection surface.
//!
//! Called from:
//! - `crate::types::checker::driver::init` (alongside `inject_builtin_throwables`).
//!
//! Key details:
//! - Property and method bodies are dummies or simple private-slot accessors;
//!   runtime population is handled by codegen-only reflection constructors.

use std::collections::{HashMap, HashSet};

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{
    ClassMethod, ClassProperty, Expr, ExprKind, Stmt, StmtKind, TypeExpr, Visibility,
};
use crate::types::traits::FlattenedClass;
use crate::types::PhpType;

use super::super::Checker;

/// Injects the four built-in reflection types into `class_map` after verifying
/// none are already declared. Each type is a dummy shell; runtime population
/// happens in codegen. Returns an error if any reflection name is already in use.
pub(crate) fn inject_builtin_reflection(
    interface_map: &HashMap<String, super::InterfaceDeclInfo>,
    class_map: &mut HashMap<String, FlattenedClass>,
    trait_names: &HashSet<String>,
) -> Result<(), CompileError> {
    for builtin_name in [
        "ReflectionAttribute",
        "ReflectionClass",
        "ReflectionMethod",
        "ReflectionProperty",
        "ReflectionFunction",
        "ReflectionParameter",
        "ReflectionNamedType",
        "ReflectionType",
        "ReflectionUnionType",
    ] {
        let builtin_key = php_symbol_key(builtin_name);
        if interface_map
            .keys()
            .chain(class_map.keys())
            .chain(trait_names.iter())
            .any(|name| php_symbol_key(name) == builtin_key)
        {
            return Err(CompileError::new(
                crate::span::Span::dummy(),
                &format!("Cannot redeclare built-in reflection type: {}", builtin_name),
            ));
        }
    }

    class_map.insert(
        "ReflectionAttribute".to_string(),
        FlattenedClass {
            name: "ReflectionAttribute".to_string(),
            extends: None,
            implements: Vec::new(),
            is_abstract: false,
            is_final: true,
            is_readonly_class: false,
            properties: vec![
                builtin_property("__name", Visibility::Private, Some(TypeExpr::Str), empty_string()),
                builtin_property("__args", Visibility::Private, Some(array_type()), empty_array()),
                builtin_property("__factory", Visibility::Private, Some(TypeExpr::Int), int_lit(0)),
            ],
            methods: vec![
                builtin_reflection_attribute_constructor_method(),
                builtin_reflection_attribute_get_name_method(),
                builtin_reflection_attribute_get_arguments_method(),
                builtin_reflection_attribute_new_instance_method(),
            ],
            attributes: Vec::new(),
            constants: Vec::new(),
            used_traits: Vec::new(),
        },
    );
    class_map.insert(
        "ReflectionClass".to_string(),
        builtin_reflection_class(),
    );
    let reflection_method = builtin_reflection_owner_class(
        "ReflectionMethod",
        vec![
            // PHP: __construct(object|string $objectOrMethod, ?string $method = null).
            // Relax the first argument to `object|string` (modelled as `mixed`) so
            // `new ReflectionMethod($object, 'method')` no longer fails the "expects Str,
            // got Object" generic check. `method_name` stays required: the reflection
            // constructor's downstream validation (in the checker's inference layer)
            // asserts the method-name arg is present, so an optional/absent second
            // argument would trip that assertion rather than type-check.
            ("class_name", Some(mixed_type()), None, false),
            ("method_name", Some(TypeExpr::Str), None, false),
        ],
    );
    class_map.insert("ReflectionMethod".to_string(), reflection_method);
    // Extend the `ReflectionMethod` shell with a private `__name` slot and
    // three PHP API methods the console probe calls. The slot is populated at
    // codegen from the reflected method's name; the accessors are un-backed
    // stubs (recognition layer only) per the existing reflection-stub policy.
    if let Some(reflection_method) = class_map.get_mut("ReflectionMethod") {
        reflection_method
            .properties
            .push(builtin_property("__name", Visibility::Private, Some(TypeExpr::Str), empty_string()));
        // PHP: `getName(): string` — slot getter on `__name`.
        reflection_method
            .methods
            .push(builtin_reflection_slot_getter("getName", "__name", TypeExpr::Str));
        // PHP: `isPublic(): bool` — un-backed stub returning `true`; scalar
        // returns lower safely on the EIR backend.
        reflection_method
            .methods
            .push(builtin_reflection_literal_method("isPublic", TypeExpr::Bool, bool_lit(true)));
        // PHP: `getClosure(): ?Closure` — un-backed stub returning `null` typed
        // `mixed` (object/closure return → mixed; see `builtin_reflection_property`
        // for the EIR-lowering reason).
        reflection_method
            .methods
            .push(builtin_reflection_literal_method("getClosure", mixed_type(), null_lit()));
        // PHP: `getDeclaringClass(): ReflectionClass` — un-backed stub returning
        // `null` typed `mixed` (object return → mixed; see
        // `builtin_reflection_property` for the EIR-lowering reason).
        reflection_method
            .methods
            .push(builtin_reflection_literal_method("getDeclaringClass", mixed_type(), null_lit()));
        // PHP: `isStatic(): bool` — un-backed stub returning `false`; scalar
        // returns lower safely on the EIR backend.
        reflection_method
            .methods
            .push(builtin_reflection_literal_method("isStatic", TypeExpr::Bool, bool_lit(false)));
    }
    class_map.insert("ReflectionProperty".to_string(), builtin_reflection_property());
    class_map.insert("ReflectionFunction".to_string(), builtin_reflection_function());
    class_map.insert(
        "ReflectionParameter".to_string(),
        builtin_reflection_parameter(),
    );
    class_map.insert(
        "ReflectionNamedType".to_string(),
        builtin_reflection_named_type(),
    );
    class_map.insert("ReflectionType".to_string(), builtin_reflection_type());
    class_map.insert(
        "ReflectionUnionType".to_string(),
        builtin_reflection_union_type(),
    );

    Ok(())
}

/// Builds a `ClassProperty` for a built-in reflection type with the given name,
/// visibility, optional type expression, and optional default value.
fn builtin_property(
    name: &str,
    visibility: Visibility,
    type_expr: Option<TypeExpr>,
    default: Option<Expr>,
) -> ClassProperty {
    ClassProperty {
        name: name.to_string(),
        visibility,
        set_visibility: None,
        type_expr,
        hooks: crate::parser::ast::PropertyHooks::none(),
        readonly: false,
        is_final: false,
        is_static: false,
        is_abstract: false,
        by_ref: false,
        default,
        span: crate::span::Span::dummy(),
        attributes: Vec::new(),
    }
}

/// Returns a `StringLiteral` expression with an empty string value.
fn empty_string() -> Option<Expr> {
    Some(Expr::new(
        ExprKind::StringLiteral(String::new()),
        crate::span::Span::dummy(),
    ))
}

/// Returns an `ArrayLiteral` expression with no elements.
fn empty_array() -> Option<Expr> {
    Some(Expr::new(
        ExprKind::ArrayLiteral(Vec::new()),
        crate::span::Span::dummy(),
    ))
}

/// Returns an `IntLiteral` expression with the given value.
fn int_lit(value: i64) -> Option<Expr> {
    Some(Expr::new(
        ExprKind::IntLiteral(value),
        crate::span::Span::dummy(),
    ))
}

/// Returns a `Null` literal expression.
fn null_lit() -> Option<Expr> {
    Some(Expr::new(ExprKind::Null, crate::span::Span::dummy()))
}

/// Returns a `BoolLiteral` expression with the given value.
fn bool_lit(value: bool) -> Option<Expr> {
    Some(Expr::new(
        ExprKind::BoolLiteral(value),
        crate::span::Span::dummy(),
    ))
}

/// Returns a `TypeExpr` for the unqualified name `array`.
fn array_type() -> TypeExpr {
    TypeExpr::Named(crate::names::Name::unqualified("array"))
}

/// Returns a `TypeExpr` for the unqualified name `mixed`.
fn mixed_type() -> TypeExpr {
    TypeExpr::Named(crate::names::Name::unqualified("mixed"))
}

/// Returns a private parameterless `__construct` method for `ReflectionAttribute`.
fn builtin_reflection_attribute_constructor_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "__construct".to_string(),
        visibility: Visibility::Private,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        variadic: None,
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
fn builtin_reflection_attribute_get_name_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "getName".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        variadic: None,
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
fn builtin_reflection_attribute_get_arguments_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "getArguments".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        variadic: None,
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
fn builtin_reflection_attribute_new_instance_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "newInstance".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        variadic: None,
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
fn builtin_reflection_slot_getter(
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
        variadic: None,
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

/// Returns a public no-arg method that unconditionally returns `value`, typed
/// `return_type`. Used for `ReflectionType`'s base-class stub methods, which
/// have no backing state of their own: concrete subtypes (`ReflectionNamedType`,
/// `ReflectionUnionType`) override with real slot-backed accessors instead of
/// sharing a same-named private property with the parent, which the checker
/// rejects as unsupported private-property shadowing.
fn builtin_reflection_literal_method(
    method_name: &str,
    return_type: TypeExpr,
    value: Option<Expr>,
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
        variadic: None,
        variadic_type: None,
        return_type: Some(return_type),
        by_ref_return: false,
        body: vec![Stmt::new(StmtKind::Return(value), dummy_span)],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public method that unconditionally returns `value` (typed
/// `return_type`) like `builtin_reflection_literal_method`, but accepts a
/// parameter list. Each tuple is `(name, type_expr, default, by_ref)`. Used for
/// un-backed reflection stubs that take arguments (e.g. `setValue`,
/// `ReflectionFunction::invoke`, `ReflectionClass::getProperty`) — the body is a
/// placeholder return and the parameters exist only so named/positional calls
/// type-check against PHP's signatures.
fn builtin_reflection_literal_method_with_params(
    method_name: &str,
    return_type: TypeExpr,
    value: Option<Expr>,
    params: Vec<(&str, Option<TypeExpr>, Option<Expr>, bool)>,
) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: method_name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: params
            .into_iter()
            .map(|(name, ty, default, by_ref)| (name.to_string(), ty, default, by_ref))
            .collect(),
        variadic: None,
        variadic_type: None,
        return_type: Some(return_type),
        by_ref_return: false,
        body: vec![Stmt::new(StmtKind::Return(value), dummy_span)],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns the public `__construct(string $name)` for `ReflectionFunction`. The
/// body is empty; codegen populates the metadata slots from the reflected
/// function's signature.
fn builtin_reflection_function_constructor_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "__construct".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("name".to_string(), Some(TypeExpr::Str), None, false)],
        variadic: None,
        variadic_type: None,
        return_type: None,
        by_ref_return: false,
        body: Vec::new(),
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Builds the `ReflectionFunction` shell with private name/short-name and
/// parameter-count slots plus public accessors. The slots are populated at
/// codegen from the reflected function's signature.
fn builtin_reflection_function() -> FlattenedClass {
    FlattenedClass {
        name: "ReflectionFunction".to_string(),
        extends: None,
        implements: Vec::new(),
        is_abstract: false,
        is_final: true,
        is_readonly_class: false,
        properties: vec![
            builtin_property("__name", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            builtin_property("__short", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            builtin_property("__num_params", Visibility::Private, Some(TypeExpr::Int), int_lit(0)),
            builtin_property(
                "__num_required",
                Visibility::Private,
                Some(TypeExpr::Int),
                int_lit(0),
            ),
            builtin_property("__params", Visibility::Private, Some(array_type()), empty_array()),
        ],
        methods: vec![
            builtin_reflection_function_constructor_method(),
            builtin_reflection_slot_getter("getName", "__name", TypeExpr::Str),
            builtin_reflection_slot_getter("getShortName", "__short", TypeExpr::Str),
            builtin_reflection_slot_getter("getNumberOfParameters", "__num_params", TypeExpr::Int),
            builtin_reflection_slot_getter(
                "getNumberOfRequiredParameters",
                "__num_required",
                TypeExpr::Int,
            ),
            builtin_reflection_slot_getter("getParameters", "__params", array_type()),
            // PHP: getClosureThis(): ?object — the bound `$this` of a closure, or null.
            // No runtime backing yet; returns null, typed `mixed` (covers `?object`).
            builtin_reflection_literal_method("getClosureThis", mixed_type(), null_lit()),
            // PHP: `invoke(?array $args = null): mixed`. Un-backed stub returning
            // `null` typed `mixed`; the nullable-array parameter lets named and
            // positional calls type-check against PHP's signature.
            builtin_reflection_literal_method_with_params(
                "invoke",
                mixed_type(),
                null_lit(),
                vec![(
                    "args",
                    Some(TypeExpr::Nullable(Box::new(array_type()))),
                    null_lit(),
                    false,
                )],
            ),
            // PHP: `getClosureCalledClass(): ?ReflectionClass`. Un-backed stub
            // returning `null` typed `mixed` (object return → mixed).
            builtin_reflection_literal_method(
                "getClosureCalledClass",
                mixed_type(),
                null_lit(),
            ),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
    }
}

/// Builds the `ReflectionParameter` shell with private name/position/optional/
/// variadic slots and public accessors, populated at codegen from the reflected
/// function's signature.
fn builtin_reflection_parameter() -> FlattenedClass {
    FlattenedClass {
        name: "ReflectionParameter".to_string(),
        extends: None,
        implements: Vec::new(),
        is_abstract: false,
        is_final: true,
        is_readonly_class: false,
        properties: vec![
            builtin_property("__name", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            builtin_property("__position", Visibility::Private, Some(TypeExpr::Int), int_lit(0)),
            builtin_property(
                "__optional",
                Visibility::Private,
                Some(TypeExpr::Bool),
                bool_lit(false),
            ),
            builtin_property(
                "__variadic",
                Visibility::Private,
                Some(TypeExpr::Bool),
                bool_lit(false),
            ),
            builtin_property("__has_type", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            builtin_property("__type", Visibility::Private, Some(mixed_type()), null_lit()),
        ],
        methods: vec![
            builtin_reflection_slot_getter("getName", "__name", TypeExpr::Str),
            builtin_reflection_slot_getter("getPosition", "__position", TypeExpr::Int),
            builtin_reflection_slot_getter("isOptional", "__optional", TypeExpr::Bool),
            builtin_reflection_slot_getter("isVariadic", "__variadic", TypeExpr::Bool),
            builtin_reflection_slot_getter("hasType", "__has_type", TypeExpr::Bool),
            builtin_reflection_slot_getter("getType", "__type", mixed_type()),
            // PHP: getDeclaringFunction(): ReflectionFunctionAbstract — the function or
            // method this parameter belongs to. An un-backed stub returning null, typed
            // `mixed`: the reflection EIR backend cannot lower a stub with an object
            // return type (it becomes an unsupported `Void`-to-`Object` runtime call), so
            // `mixed` is used rather than `ReflectionFunction`. Gradual typing still lets
            // callers chain methods on the result.
            builtin_reflection_literal_method("getDeclaringFunction", mixed_type(), null_lit()),
            // PHP: `isDefaultValueAvailable(): bool`. Un-backed stub returning
            // `false`; scalar returns lower safely on the EIR backend.
            builtin_reflection_literal_method(
                "isDefaultValueAvailable",
                TypeExpr::Bool,
                bool_lit(false),
            ),
            // PHP 8.0+ `hasDefaultValue(): bool`. Un-backed stub returning `false`.
            builtin_reflection_literal_method("hasDefaultValue", TypeExpr::Bool, bool_lit(false)),
            // PHP: `getDefaultValue(): mixed`. Un-backed stub returning `null`
            // typed `mixed`.
            builtin_reflection_literal_method("getDefaultValue", mixed_type(), null_lit()),
            // PHP: `getDeclaringClass(): ?ReflectionClass`. Un-backed stub
            // returning `null` typed `mixed` (object return → mixed).
            builtin_reflection_literal_method("getDeclaringClass", mixed_type(), null_lit()),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
    }
}

/// Builds the `ReflectionNamedType` shell: a parameter/return type rendered as a
/// runtime object with a name, nullability flag, and builtin flag. Populated at
/// codegen from the declared type. Extends `ReflectionType` so a value typed
/// `\ReflectionType` can be narrowed to `\ReflectionNamedType` via `instanceof`.
fn builtin_reflection_named_type() -> FlattenedClass {
    FlattenedClass {
        name: "ReflectionNamedType".to_string(),
        extends: Some("ReflectionType".to_string()),
        implements: Vec::new(),
        is_abstract: false,
        is_final: true,
        is_readonly_class: false,
        properties: vec![
            builtin_property("__name", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            builtin_property(
                "__allows_null",
                Visibility::Private,
                Some(TypeExpr::Bool),
                bool_lit(false),
            ),
            builtin_property(
                "__builtin",
                Visibility::Private,
                Some(TypeExpr::Bool),
                bool_lit(false),
            ),
        ],
        methods: vec![
            builtin_reflection_slot_getter("getName", "__name", TypeExpr::Str),
            builtin_reflection_slot_getter("allowsNull", "__allows_null", TypeExpr::Bool),
            builtin_reflection_slot_getter("isBuiltin", "__builtin", TypeExpr::Bool),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
    }
}

/// Builds the `ReflectionType` shell: the abstract base class for PHP's type
/// reflection objects (`ReflectionNamedType`, `ReflectionUnionType`,
/// `ReflectionIntersectionType`). Declares no properties of its own — its
/// `allowsNull()`/`__toString()` stubs are literal-returning placeholders so
/// concrete subtypes stay free to declare their own same-named private slots
/// without tripping the checker's private-property-shadowing restriction.
/// Populated at codegen from the declared type via the concrete subtype.
fn builtin_reflection_type() -> FlattenedClass {
    FlattenedClass {
        name: "ReflectionType".to_string(),
        extends: None,
        implements: Vec::new(),
        is_abstract: true,
        is_final: false,
        is_readonly_class: false,
        properties: Vec::new(),
        methods: vec![
            builtin_reflection_literal_method("allowsNull", TypeExpr::Bool, bool_lit(false)),
            builtin_reflection_literal_method("__toString", TypeExpr::Str, empty_string()),
            // PHP's `ReflectionType` does not declare `getName()` — it lives on
            // `ReflectionNamedType` — but Symfony calls `getType()?->getName()`
            // after an `instanceof ReflectionNamedType` that elephc does not
            // narrow through, so the call resolves against `ReflectionType`.
            // Add a `mixed`-returning stub here so the call type-checks under
            // gradual typing; concrete subtypes override with their slot-backed
            // `getName`. Object return types are unsafe on un-backed reflection
            // stubs (see `builtin_reflection_property`), so `mixed` + `null_lit`
            // is used instead of `?ReflectionNamedType`.
            builtin_reflection_literal_method("getName", mixed_type(), null_lit()),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
    }
}

/// Builds the `ReflectionUnionType` shell: a `ReflectionType` subtype exposing
/// the union's member types through `getTypes(): array`. Populated at codegen
/// from the declared union type.
fn builtin_reflection_union_type() -> FlattenedClass {
    FlattenedClass {
        name: "ReflectionUnionType".to_string(),
        extends: Some("ReflectionType".to_string()),
        implements: Vec::new(),
        is_abstract: false,
        is_final: true,
        is_readonly_class: false,
        properties: vec![builtin_property(
            "__types",
            Visibility::Private,
            Some(array_type()),
            empty_array(),
        )],
        methods: vec![builtin_reflection_slot_getter("getTypes", "__types", array_type())],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
    }
}

/// Builds the `ReflectionClass` shell with a private resolved-name slot,
/// private attribute array slot, public constructor, `getName()`, and
/// `getAttributes()`.
fn builtin_reflection_class() -> FlattenedClass {
    FlattenedClass {
        name: "ReflectionClass".to_string(),
        extends: None,
        implements: Vec::new(),
        is_abstract: false,
        is_final: true,
        is_readonly_class: false,
        properties: vec![
            builtin_property("__name", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            builtin_property(
                "__attrs",
                Visibility::Private,
                Some(array_type()),
                empty_array(),
            ),
        ],
        methods: vec![
            builtin_reflection_owner_constructor_method(vec![(
                "class_name",
                Some(TypeExpr::Str),
                None,
                false,
            )]),
            builtin_reflection_class_get_name_method(),
            builtin_reflection_owner_get_attributes_method(),
            // PHP: `newInstanceWithoutConstructor(): object`. Un-backed stub
            // returning `null` typed `mixed` — object return types cannot be
            // used on un-backed reflection stubs (see `builtin_reflection_property`
            // for the EIR-lowering reason), so `mixed` is the proven-safe return.
            builtin_reflection_literal_method(
                "newInstanceWithoutConstructor",
                mixed_type(),
                null_lit(),
            ),
            // PHP: `getProperty(string $name): ReflectionProperty`. Un-backed
            // stub returning `null` typed `mixed` (object return → mixed); the
            // `name` parameter lets named/positional calls type-check.
            builtin_reflection_literal_method_with_params(
                "getProperty",
                mixed_type(),
                null_lit(),
                vec![("name", Some(TypeExpr::Str), None, false)],
            ),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
    }
}

/// Returns a public `ReflectionClass::getName()` method that returns the
/// resolved reflected class name from the private `__name` slot.
fn builtin_reflection_class_get_name_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "getName".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        variadic: None,
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

/// Builds a `FlattenedClass` for `ReflectionMethod` or `ReflectionProperty`
/// with a private `__attrs` array property and two methods: `__construct`
/// (public, accepting the supplied params) and `getAttributes` (public,
/// returning the `__attrs` array).
fn builtin_reflection_owner_class(
    name: &str,
    constructor_params: Vec<(&str, Option<TypeExpr>, Option<Expr>, bool)>,
) -> FlattenedClass {
    FlattenedClass {
        name: name.to_string(),
        extends: None,
        implements: Vec::new(),
        is_abstract: false,
        is_final: true,
        is_readonly_class: false,
        properties: vec![builtin_property(
            "__attrs",
            Visibility::Private,
            Some(array_type()),
            empty_array(),
        )],
        methods: vec![
            builtin_reflection_owner_constructor_method(constructor_params),
            builtin_reflection_owner_get_attributes_method(),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
    }
}

/// Builds the `ReflectionProperty` shell: the reflection-owner base (private
/// `__attrs` slot, `__construct`, `getAttributes`) plus a `getType()` stub. Its
/// constructor mirrors PHP's
/// `ReflectionProperty::__construct(object|string $class, string $property)`: the first
/// parameter accepts an object instance or a class-name string (modelled as `mixed`
/// under the gradual type system so both forms type-check).
///
/// `getType()` (PHP's `getType(): ?ReflectionType`) is an un-backed stub that returns
/// `null`, typed `mixed`. It is deliberately NOT typed `?ReflectionType`: the reflection
/// EIR backend eagerly lowers every method of a used reflection class as a runtime call,
/// and a stub returning an object type lowers to an unsupported `Void`-to-`Object`
/// runtime call that breaks codegen for *all* reflection programs. `mixed` is the
/// proven-safe stub return (matching `newInstance`) and still lets callers use
/// `getType()`/`getType()?->getName()` under gradual typing.
fn builtin_reflection_property() -> FlattenedClass {
    let mut class = builtin_reflection_owner_class(
        "ReflectionProperty",
        vec![
            ("class_name", Some(mixed_type()), None, false),
            ("property_name", Some(TypeExpr::Str), None, false),
        ],
    );
    // Private name slot populated at codegen from the reflected property's
    // declared name; surfaced through the `getName()` slot getter below.
    class
        .properties
        .push(builtin_property("__name", Visibility::Private, Some(TypeExpr::Str), empty_string()));
    class.methods.push(builtin_reflection_literal_method(
        "getType",
        mixed_type(),
        null_lit(),
    ));
    // PHP: `getName(): string` — slot getter on `__name` (mirrors
    // `ReflectionParameter::getName`).
    class
        .methods
        .push(builtin_reflection_slot_getter("getName", "__name", TypeExpr::Str));
    // PHP: `getDeclaringFunction(): ReflectionFunctionAbstract` — un-backed
    // stub returning `null` typed `mixed` (object return → mixed; see the
    // `ReflectionParameter::getDeclaringFunction` comment for the lowering
    // reason).
    class.methods.push(builtin_reflection_literal_method(
        "getDeclaringFunction",
        mixed_type(),
        null_lit(),
    ));
    // PHP: `setValue(mixed $objectOrValue, mixed $value = UNKNOWN): void`.
    // Un-backed stub: `void` return with a placeholder `return;` and two
    // `mixed` parameters so named/positional calls type-check. The second
    // parameter defaults to `null` (PHP's `UNKNOWN` sentinel is a runtime
    // concern; `null` is the safe syntactic default here).
    class.methods.push(builtin_reflection_literal_method_with_params(
        "setValue",
        TypeExpr::Void,
        None,
        vec![
            ("objectOrValue", Some(mixed_type()), None, false),
            ("value", Some(mixed_type()), null_lit(), false),
        ],
    ));
    // PHP: `getDefaultValue(): mixed` — un-backed stub returning `null` typed
    // `mixed` (the property's declared default value; no runtime backing yet).
    class
        .methods
        .push(builtin_reflection_literal_method("getDefaultValue", mixed_type(), null_lit()));
    // PHP 8.4 `hasDefaultValue(): bool` — un-backed stub returning `false`;
    // scalar returns lower safely on the EIR backend.
    class
        .methods
        .push(builtin_reflection_literal_method("hasDefaultValue", TypeExpr::Bool, bool_lit(false)));
    // PHP: `isDefaultValueAvailable(): bool` — un-backed stub returning `false`.
    // Distinct from `ReflectionParameter::isDefaultValueAvailable` (this is the
    // property-side counterpart); same stub pattern.
    class.methods.push(builtin_reflection_literal_method(
        "isDefaultValueAvailable",
        TypeExpr::Bool,
        bool_lit(false),
    ));
    // PHP: `getDeclaringClass(): ReflectionClass` — un-backed stub returning
    // `null` typed `mixed` (object return → mixed; see the
    // `ReflectionParameter::getDeclaringClass` comment for the lowering reason).
    class
        .methods
        .push(builtin_reflection_literal_method("getDeclaringClass", mixed_type(), null_lit()));
    class
}

/// Builds a public `__construct` method for a reflection owner class using the
/// provided parameter list: each tuple is (name, type_expr, default, by_ref).
fn builtin_reflection_owner_constructor_method(
    params: Vec<(&str, Option<TypeExpr>, Option<Expr>, bool)>,
) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "__construct".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: params
            .into_iter()
            .map(|(name, ty, default, by_ref)| (name.to_string(), ty, default, by_ref))
            .collect(),
        variadic: None,
        variadic_type: None,
        return_type: None,
        by_ref_return: false,
        body: Vec::new(),
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public `getAttributes()` method that returns the private `__attrs`
/// property as an `array` of `ReflectionAttribute` objects.
fn builtin_reflection_owner_get_attributes_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "getAttributes".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: Vec::new(),
        variadic: None,
        variadic_type: None,
        return_type: Some(array_type()),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(
                ExprKind::PropertyAccess {
                    object: Box::new(Expr::new(ExprKind::This, dummy_span)),
                    property: "__attrs".to_string(),
                },
                dummy_span,
            ))),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Overrides the return types on the synthesized reflection class methods inside
/// `checker` to match PHP's actual signatures:
/// - `__construct` → `void`
/// - `getName` / `getArguments` → `string` / `array`
/// - `newInstance` → `mixed`
/// - `getAttributes` → `array<ReflectionAttribute>`
pub(crate) fn patch_builtin_reflection_signatures(checker: &mut Checker) {
    if let Some(class_info) = checker.classes.get_mut("ReflectionAttribute") {
        if let Some(sig) = class_info.methods.get_mut("__construct") {
            sig.return_type = PhpType::Void;
        }
        if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getName")) {
            sig.return_type = PhpType::Str;
        }
        if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getArguments")) {
            // Attribute arguments can be keyed (named arguments / associative
            // arrays), so the result is an associative array of mixed values.
            sig.return_type = PhpType::AssocArray {
                key: Box::new(PhpType::Mixed),
                value: Box::new(PhpType::Mixed),
            };
        }
        if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("newInstance")) {
            sig.return_type = PhpType::Mixed;
        }
    }
    for class_name in ["ReflectionClass", "ReflectionMethod", "ReflectionProperty"] {
        if let Some(class_info) = checker.classes.get_mut(class_name) {
            if let Some(sig) = class_info.methods.get_mut("__construct") {
                sig.return_type = PhpType::Void;
            }
            if class_name == "ReflectionClass" {
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getName")) {
                    sig.return_type = PhpType::Str;
                }
            }
            if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getAttributes")) {
                sig.return_type = PhpType::Array(Box::new(PhpType::Object(
                    "ReflectionAttribute".to_string(),
                )));
            }
        }
    }
    if let Some(class_info) = checker.classes.get_mut("ReflectionFunction") {
        if let Some(sig) = class_info.methods.get_mut("__construct") {
            sig.return_type = PhpType::Void;
        }
        if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getParameters")) {
            sig.return_type = PhpType::Array(Box::new(PhpType::Object(
                "ReflectionParameter".to_string(),
            )));
        }
    }
    if let Some(class_info) = checker.classes.get_mut("ReflectionParameter") {
        if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getType")) {
            // ?ReflectionNamedType — null for untyped parameters.
            sig.return_type = PhpType::Union(vec![
                PhpType::Object("ReflectionNamedType".to_string()),
                PhpType::Void,
            ]);
        }
    }
}
